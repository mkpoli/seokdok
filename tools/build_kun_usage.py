"""Build the corpus of historical 訓讀 usage: Hanja printed with a native reading beside them.

Usage: uv run tools/build_kun_usage.py [--survey FILE]

Reads the Korean Wikisource dump named in data/sources/README.md. Writes
data/kun-usage.tsv (every distinct use; the keyboards never load it) and
dict/kun-attested.tsv (the uses that map to a modern word).

The books print readings three ways: {{윗주|飛|나}}는, 人(사람) and 군포내(軍浦場).
A reading the character's sound explains (勞動 로동, 天 텬) is skipped, allowing for
old spellings and 두음법칙. --survey lists every book of the dump with its count of
readings left over, for finding more books.
"""
import argparse
import bz2
import collections
import json
import re
import sys
import unicodedata
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))
from build_eonhae import DICT, DUMP, SRC, Att, Modern, class_of, is_sino_gloss, lemma_candidates, meaning_evidence, norm, page_number, readings, strong, syllables, word_is_sino, written_of  # noqa: E402

CACHE = SRC / "wikisource-ko" / "kun-pages.jsonl"
USAGE = ROOT / "data" / "kun-usage.tsv"
HAN = "㐀-䶿一-鿿豈-﫿\U00020000-\U0002FFFF"
HANGUL = "가-힣ᄀ-ᇿꥠ-ꥼힰ-ퟻ〮〯"

SIC_OPEN, SIC_MID, SIC_CLOSE = "", "", ""
RUBY_OPEN, RUBY_MID, RUBY_CLOSE = "", "", ""
RUBY_NAMES = "윗주|윗주2|아랫주|ruby|Ruby"


def strip_markup(text):
    """Wiki text as running text: rubies and {{sic}} become marked spans the parsers read."""
    text = re.sub(r"<noinclude>.*?</noinclude>", " ", text, flags=re.S)
    text = re.sub(r"<ref[^>]*>.*?</ref>|<ref[^>]*/>|<score.*?</score>|<!--.*?-->", " ", text, flags=re.S)
    text = re.sub(r"\[\[(?:파일|File|그림|Image):[^\n]*?\]\]", " ", text)
    text = re.sub(r"\[\[[^\]|]*\|([^\]]*)\]\]", r"\1", text)
    text = re.sub(r"\[\[([^\]]*)\]\]", r"\1", text)
    text = re.sub(r"<br\s*/?>", "\n", text)
    for _ in range(4):  # templates nest
        text = re.sub(r"\{\{[Ss][Ii][Cc]\|([^|{}]*)\|([^|{}]*)\}\}", lambda m: SIC_OPEN + m[1] + SIC_MID + m[2] + SIC_CLOSE, text)
        text = re.sub(rf"\{{\{{(?:{RUBY_NAMES})\|([^|{{}}]*)\|([^|{{}}]*)(?:\|[^{{}}]*)?\}}\}}", lambda m: RUBY_OPEN + m[1].strip() + RUBY_MID + m[2].strip() + RUBY_CLOSE, text)
        text = re.sub(r"\{\{이체자\|([^|{}]*)(\|[^{}]*)?\}\}", r"\1", text)
        text = re.sub(r"\{\{드러냄표\|([^|{}]*)(\|[^{}]*)?\}\}", r"\1", text)
        text = re.sub(r"\{\{(?:[^|{}]*작게|크게|더 ?크게|더더 ?크게|더더더 ?크게|가운데|오른쪽|왼쪽)\|([^|{}]*)(\|[^{}]*)?\}\}", r"\1", text)
        text = re.sub(r"\{\{[^{}]*\}\}", " ", text)
    text = re.sub(r"<[^>]+>", "", text)
    return re.sub(r"'''?|^[:;*#=]+|=+\s*$", "", text, flags=re.M)


def printed(s):
    """What the page prints: the first half of {{sic}}."""
    return re.sub(f"{SIC_OPEN}([^{SIC_MID}]*){SIC_MID}[^{SIC_CLOSE}]*{SIC_CLOSE}", r"\1", s)


def corrected(s):
    """What the transcriber says was meant: the second half of {{sic}}."""
    return re.sub(f"{SIC_OPEN}[^{SIC_MID}]*{SIC_MID}([^{SIC_CLOSE}]*){SIC_CLOSE}", r"\1", s)


HAN_ONLY = re.compile(f"^[{HAN}]+$")
HANGUL_READING = re.compile(rf"^[{HANGUL}{SIC_OPEN}{SIC_MID}{SIC_CLOSE} ]+$")
RUBY_RE = re.compile(f"{RUBY_OPEN}([^{RUBY_MID}]*){RUBY_MID}([^{RUBY_CLOSE}]*){RUBY_CLOSE}")
SPAN = f"{SIC_OPEN}{SIC_MID}{SIC_CLOSE}"
HANGUL_RUBY = rf"{RUBY_OPEN}([{HANGUL}{SPAN}]+){RUBY_MID}([{HAN}]+){RUBY_CLOSE}"
HANGUL_RUN = re.compile(rf"(?:{HANGUL_RUBY.replace('(', '').replace(')', '')}){{2,}}")


def join_rubies(text, sounds):
    """하(一)루(日): a word in Hangul with its Hanja over each syllable is one use. A syllable the character
    sounds out stands alone: 답(答)물(水) is 水 물, and 동(同)포(胞)쇠(凋)함(殘) is 凋殘 쇠함."""
    def join(m):
        out, run = [], []

        def flush():
            if run:
                out.append(RUBY_OPEN + "".join(p[0] for p in run) + RUBY_MID + "".join(p[1] for p in run) + RUBY_CLOSE)
                run.clear()
        for reading, hanja in re.findall(HANGUL_RUBY, m.group()):
            syls = split_syllables(corrected(reading))
            if syls and len(hanja) == 1 and len(syls) == 1 and sound_key(syls[0]) in {sound_key(s) for s in sounds.get(hanja, ())}:
                flush()
                out.append(RUBY_OPEN + reading + RUBY_MID + hanja + RUBY_CLOSE)
            else:
                run.append((reading, hanja))
        flush()
        return "".join(out)
    return HANGUL_RUN.sub(join, text)


HAN_THEN_READING = re.compile(rf"([{HAN}]+)[(（]([{HANGUL}{SPAN} ]+)[)）]")
READING_THEN_HAN = re.compile(rf"((?:[{HANGUL}]|{SIC_OPEN}[^{SIC_CLOSE}]*{SIC_CLOSE})+)[(（]([{HAN}]+)[)）]")


class Use:
    """One printed reading: `pattern` is ruby, han-reading or reading-han."""
    __slots__ = ("hanja", "reading", "meant", "pattern", "tail", "quote")

    def __init__(self, hanja, reading, pattern, tail, quote):
        self.hanja = hanja
        self.reading = printed(reading).strip()
        self.meant = corrected(reading).strip()
        self.pattern, self.tail, self.quote = pattern, tail, quote


def flat(text):
    """Rubies shown by their base, so a quote reads as the line does."""
    return RUBY_RE.sub(r"\1", printed(text))


def grapheme_cut(s, i):
    """The nearest place at or before i that does not split a syllable of conjoining jamo."""
    while 0 < i < len(s) and unicodedata.category(s[i]) == "Mn" or 0 < i < len(s) and 0x1160 <= ord(s[i]) <= 0x11FF or 0 < i < len(s) and 0xD7B0 <= ord(s[i]) <= 0xD7FB:
        i -= 1
    return i


def make_quote(before, shown, after, width=8):
    """The shortest quote showing the use: a few letters either side, within the line."""
    before = re.split(r"[\n.。!?|]", before)[-1]
    after = re.split(r"[\n.。!?|]", after)[0]
    start = grapheme_cut(before, max(0, len(before) - width))
    end = grapheme_cut(after, min(len(after), width))
    return re.sub(r"\s+", " ", before[start:] + shown + after[:end]).strip()


def uses_of(text, sounds):
    """Every printed reading on a page."""
    s = join_rubies(strip_markup(text), sounds)
    out = []
    for m in RUBY_RE.finditer(s):
        base, top = m.group(1), m.group(2)
        before, after = s[max(0, m.start() - 80): m.start()], s[m.end(): m.end() + 80]
        tail = re.match(f"[{HANGUL}]*", printed(after)).group()
        if HAN_ONLY.match(base) and HANGUL_READING.match(top):
            out.append(Use(base, top, "ruby", tail, make_quote(flat(before), f"{base}({printed(top).strip()})", flat(after))))
        elif HAN_ONLY.match(top) and HANGUL_READING.match(base):
            out.append(Use(top, base, "over", tail, make_quote(flat(before), f"{printed(base).strip()}({top})", flat(after))))
    plain = RUBY_RE.sub(r"\1", s)
    for m in HAN_THEN_READING.finditer(plain):
        before, after = plain[max(0, m.start() - 80): m.start()], plain[m.end(): m.end() + 80]
        tail = re.match(f"[{HANGUL}]*", printed(after)).group()
        out.append(Use(m.group(1), m.group(2), "han-reading", tail, make_quote(printed(before), printed(m.group()), printed(after))))
    for m in READING_THEN_HAN.finditer(plain):
        before, after = plain[max(0, m.start() - 80): m.start()], plain[m.end(): m.end() + 80]
        tail = re.match(f"[{HANGUL}]*", printed(after)).group()
        out.append(Use(m.group(2), m.group(1), "reading-han", tail, make_quote(printed(before), printed(m.group()), printed(after))))
    return out


# ---- Reading by sound ------------------------------------------------------

def load_sounds():
    """Character → its Sino-Korean sounds, from sino.tsv and Unihan's kHangul."""
    sounds = collections.defaultdict(set)
    for name in ("sino.tsv", "etym.tsv"):
        for line in (DICT / name).open(encoding="utf-8"):
            if line.startswith("#") or line.startswith("reading\t"):
                continue
            f = line.rstrip("\n").split("\t")
            if len(f) > 1 and len(f[1]) == 1 and len(f[0]) == 1:
                sounds[f[1]].add(f[0])
    with zipfile.ZipFile(SRC / "unihan" / "Unihan.zip") as z:
        with z.open("Unihan_Readings.txt") as f:
            for raw in f:
                line = raw.decode("utf-8")
                if "\tkHangul\t" in line:
                    cp, _, val = line.rstrip("\n").split("\t", 2)
                    for item in val.split():
                        sounds[chr(int(cp[2:], 16))].add(item.split(":")[0])
    return sounds


L_MERGE = {"ᄁ": "ᄀ", "ᄄ": "ᄃ", "ᄈ": "ᄇ", "ᄊ": "ᄉ", "ᄍ": "ᄌ", "ᅀ": "ᄉ",
           "ᄡ": "ᄇ", "ᄭ": "ᄀ", "ᄯ": "ᄃ", "ᄲ": "ᄇ", "ᄶ": "ᄌ", "ᅙ": "ᄒ"}
V_ONE = {"ᆞ": "ᅡ", "ᆡ": "ᅢ", "ᅫ": "ᅰ", "ᅬ": "ᅰ", "ᅤ": "ᅢ", "ᅦ": "ᅢ",
         "ᅨ": "ᅢ", "ᅴ": "ᅵ", "ᆈ": "ᅰ", "ᆔ": "ᅱ"}
GLIDE = {"ᅣ": "ᅡ", "ᅧ": "ᅥ", "ᅭ": "ᅩ", "ᅲ": "ᅮ"}
CORONAL = {"ᄃ", "ᄐ", "ᄌ", "ᄎ", "ᄉ"}
PALATAL = {"ᄃ": "ᄌ", "ᄐ": "ᄎ"}
FRONT = {"ᅡ": "ᅣ", "ᅥ": "ᅧ", "ᅩ": "ᅭ", "ᅮ": "ᅲ"}


def sound_key(syl):
    """A syllable under the spellings the era allows: 텬 천, 뎡 정, 샹 상, 로 노, 량 양, ᄒᆞᆫ 한, 긔 기."""
    s = norm(syl)
    lead = "".join(c for c in s if 0x1100 <= ord(c) < 0x1160 or 0xA960 <= ord(c) < 0xA97D)
    vowel = "".join(c for c in s if 0x1160 <= ord(c) < 0x11A8 or 0xD7B0 <= ord(c) < 0xD7C7)
    coda = "".join(c for c in s if 0x11A8 <= ord(c) < 0x1200 or 0xD7CB <= ord(c) < 0xD7FC)
    lead = L_MERGE.get(lead, lead)
    if len(lead) > 1:
        lead = {"ᄉᄀ": "ᄀ", "ᄉᄃ": "ᄃ", "ᄉᄇ": "ᄇ", "ᄇᄉ": "ᄉ"}.get(lead, lead)
    if vowel == "ᆞᅵ":
        vowel = "ᅢ"
    vowel = V_ONE.get(vowel, vowel)
    vowel = {"ᅩᅡ": "ᅪ", "ᅮᅥ": "ᅯ"}.get(vowel, vowel)
    if lead == "ᄅ":  # 로 노, 량 양, 리 이: 두음법칙 in either direction
        lead = "ᄂ"
    # ㄴ before i and y falls, so 녀 여 and 니 이 are one sound; 너 is another.
    if lead == "ᄂ" and (vowel == "ᅵ" or vowel in GLIDE):
        lead = "ᄋ"
    if lead in PALATAL and (vowel == "ᅵ" or vowel in GLIDE or vowel == "ᅢ"):
        lead = PALATAL[lead]
    if lead in CORONAL and vowel in GLIDE:
        vowel = GLIDE[vowel]
    if lead in ("ᄌ", "ᄎ") and vowel == "ᅳ":  # 즉 직, 측 칙
        vowel = "ᅵ"
    if vowel == "ᅳ" and lead in ("ᄆ", "ᄇ", "ᄑ"):  # 믈 물, 븡 붕
        vowel = "ᅮ"
    coda = {"ᆺ": "ᆮ", "ᇀ": "ᆮ", "ᆽ": "ᆮ", "ᆾ": "ᆮ", "ᇰ": "ᆼ", "ᇁ": "ᆸ"}.get(coda, coda)
    return lead + vowel + coda


def split_syllables(reading):
    """Syllables of a reading, spaces gone; None when it has anything that is no syllable."""
    return syllables(norm(reading.replace(" ", "")))


def align(hanja, reading_syls, sounds):
    """How many characters a syllable of the reading sounds out, in order, and how many syllables that leaves."""
    keys = [sound_key(s) for s in reading_syls]
    n, m = len(hanja), len(keys)
    best = [[0] * (m + 1) for _ in range(n + 1)]
    for i in range(1, n + 1):
        char_keys = {sound_key(s) for s in sounds.get(hanja[i - 1], ())}
        for j in range(1, m + 1):
            best[i][j] = max(best[i - 1][j], best[i][j - 1], best[i - 1][j - 1] + (keys[j - 1] in char_keys))
    return best[n][m]


def contains_run(hanja, reading_syls, sounds):
    """The characters sound out as an unbroken run of the syllables: 낙동강변(洛東江), 의발착역(發着驛)."""
    n = len(hanja)
    for start in range(0, len(reading_syls) - n + 1):
        run = reading_syls[start: start + n]
        if all(sound_key(s) in {sound_key(x) for x in sounds.get(c, ())} for c, s in zip(hanja, run)):
            return True
    return False


def classify(use, sounds):
    """sound: the characters explain the reading. mixed: some do. meaning: none does."""
    syls = split_syllables(use.meant or use.reading)
    if not syls:
        return None
    if contains_run(use.hanja, syls, sounds) and (use.pattern == "reading-han" or len(syls) == len(use.hanja)):
        return "sound"
    # 齊(젯)나라, 周(쥿)나라: the character's sound with the genitive ㅅ.
    if len(syls) == 1 and syls[0].endswith("ᆺ") and contains_run(use.hanja, [syls[0][:-1]], sounds):
        return "sound"
    hits = align(use.hanja, syls, sounds)
    if hits == 0:
        return "meaning"
    if hits == len(syls):  # every syllable is some character's sound: 天下萬國(텬만국) drops one
        return "sound"
    if use.pattern in ("han-reading", "reading-han") and hits == len(use.hanja):  # 要地(요ᄒᆡ디): the characters all sound, one syllable added
        return "sound"
    if len(syls) == len(use.hanja) and all(near_sound(c, s, sounds) for c, s in zip(use.hanja, syls)):
        return "sound"  # 天堂地獄(쳔당디욱), 水剌(슈라): the sound with a slip in one letter
    return "mixed"


def near_sound(char, syl, sounds):
    """The syllable shares two of three parts with one of the character's sounds. Only for a word whose other
    characters sound out, since 馬 말 and 墨 먹 are as close and are readings by meaning."""
    a = jamo_parts(sound_key(syl))
    for s in sounds.get(char, ()):
        b = jamo_parts(sound_key(s))
        if sum(x == y for x, y in zip(a, b)) >= 2:
            return True
    return False


def jamo_parts(key):
    lead = "".join(c for c in key if 0x1100 <= ord(c) < 0x1160 or 0xA960 <= ord(c) < 0xA97D)
    vowel = "".join(c for c in key if 0x1160 <= ord(c) < 0x11A8 or 0xD7B0 <= ord(c) < 0xD7C7)
    coda = "".join(c for c in key if 0x11A8 <= ord(c) < 0x1200 or 0xD7CB <= ord(c) < 0xD7FC)
    return lead, vowel, coda


# ---- The dump --------------------------------------------------------------

def walk_dump():
    with bz2.open(DUMP, "rb") as f:
        for _, el in ET.iterparse(f, events=("end",)):
            if el.tag.rsplit("}", 1)[-1] != "page":
                continue
            title = ns = text = None
            for c in el:
                tag = c.tag.rsplit("}", 1)[-1]
                if tag == "title":
                    title = c.text
                elif tag == "ns":
                    ns = c.text
                elif tag == "revision":
                    for r in c:
                        if r.tag.rsplit("}", 1)[-1] == "text":
                            text = r.text
            el.clear()
            if title and text:
                yield title, ns, text


def book_key(title):
    return re.sub(r"^(페이지|Page):", "", title).split("/")[0]


def field(text, name):
    m = re.search(rf"^\s*\|\s*{name}\s*=(.*)$", text, re.M)
    return m.group(1).strip() if m else ""


def survey(path):
    """Every book of the dump, with its readings left after the sound test, for choosing BOOKS by eye."""
    sounds = load_sounds()
    meta, stats, samples = {}, collections.defaultdict(collections.Counter), collections.defaultdict(list)
    for title, ns, text in walk_dump():
        if ns == "252":
            meta[title.split(":", 1)[1]] = (field(text, "제목"), field(text, "저자"), field(text, "연도"))
            continue
        if ns not in ("0", "250"):
            continue
        key = book_key(title)
        if ns == "0" and "머리말" in text[:400] and key not in meta:
            meta[key] = (field(text, "제목"), field(text, "저자"), field(text, "연도"))
        for u in uses_of(text, sounds):
            kind = classify(u, sounds)
            stats[key][f"{u.pattern}:{kind}"] += 1
            if kind == "meaning" and len(samples[key]) < 40:
                samples[key].append(f"{u.hanja}={u.reading}")
    with open(path, "w", encoding="utf-8") as w:
        rows = sorted(stats.items(), key=lambda kv: -sum(n for k, n in kv[1].items() if k.endswith(":meaning")))
        for key, c in rows:
            mean = sum(n for k, n in c.items() if k.endswith(":meaning"))
            w.write("\t".join([key, str(mean), json.dumps(dict(c), ensure_ascii=False), *map(str, meta.get(key, ("", "", ""))), " ".join(samples[key])]) + "\n")


# ---- Books ------------------------------------------------------------------

# Scanned books (Wikisource's 페이지: namespace) dated by their index page to this span: 國漢文 and the
# early modern press. Earlier books gloss characters by the sound, and dict/eonhae.tsv reads them as primers.
YEARS = (1880, 1945)
# Index pages that lack a year, with the year the file name gives; and the titles as the books print them.
YEAR_OF = {"하믈레트 현철 1920.pdf": "1920", "그루쇼 표류긔(1925) 제임스 게일, 이원모 역.pdf": "1925", "삼천리 제1호(1929).pdf": "1929"}
TITLE_OF = {
    "CNTS-00047778824 勞動夜學讀本. 第1- 俞吉濬(朝鮮) 著.pdf": "勞動夜學讀本",
    "경부철도가.pdf": "京釜鐵道歌",
    "CNTS-00047818803 화즁화.pdf": "화즁화",
}


def clean_field(v):
    v = re.sub(r"\[\[(?:[^|\]]*\|)?([^\]]*)\]\]", r"\1", v)
    v = re.sub(r"\{\{[^|{}]*\|([^{}]*)\}\}", r"\1", v)
    return re.sub(r"\s+", " ", v).strip()


def extract():
    """Index data of every scanned book, then the pages that print a reading."""
    if CACHE.exists() and CACHE.stat().st_mtime > DUMP.stat().st_mtime:
        return
    rx = re.compile(rf"(?:{RUBY_NAMES})\||[{HAN}][(（][{HANGUL} ]|[{HANGUL}][(（][{HAN}]")
    with CACHE.open("w", encoding="utf-8") as w:
        for title, ns, text in walk_dump():
            if ns == "252":
                meta = {k: clean_field(field(text, k)) for k in ("제목", "저자", "편집자", "연도")}
                w.write(json.dumps({"index": title.split(":", 1)[1], **meta}, ensure_ascii=False) + "\n")
            elif ns == "250" and rx.search(text):
                q = re.search(r'pagequality level="(\d)"', text)
                w.write(json.dumps({"title": title, "quality": q.group(1) if q else "", "text": text}, ensure_ascii=False) + "\n")


def read_cache():
    index, pages = {}, []
    with CACHE.open(encoding="utf-8") as f:
        for line in f:
            o = json.loads(line)
            if "index" in o:
                index[o["index"]] = o
            else:
                pages.append(o)
    return index, pages


def book_of(key, index):
    """(title, year, author) of a scanned book in the span; None for any other."""
    meta = index.get(key)
    if meta is None or not re.search(r"\.(pdf|djvu)$", key, re.I):
        return None
    m = re.search(r"\d{4}", YEAR_OF.get(key) or meta["연도"])
    if not m or not YEARS[0] <= int(m.group()) <= YEARS[1]:
        return None
    author = meta["저자"]
    if not author and meta["편집자"]:
        author = meta["편집자"] + " (編)"
    issue = re.fullmatch(r"조선어학회 (한글 \(.*\))\.pdf", key)
    return TITLE_OF.get(key) or (issue and issue.group(1)) or meta["제목"] or re.sub(r"\.(pdf|djvu)$", "", key), m.group(), author


# ---- What is no reading -----------------------------------------------------

GRAMMAR = set("은 는 이 가 을 를 의 에 와 과 도 로 며 니 야 라 고 나 면 서 요 오 다 하 ᄒᆞ 한 ᄒᆞᆫ 홈 함 제 조 항 호 년 월 일".split())


def is_noise(use, syls):
    """A parenthesis after running Hangul is often a remark on the sentence: 一=은, 八月一日=선전포고를하든날."""
    if use.pattern in ("ruby", "over"):
        return False
    if len(syls) > 5 or len(syls) > len(use.hanja) + 3:
        return True
    if len(syls) == 1 and use.reading in GRAMMAR:
        return True
    return bool(re.fullmatch("[一二三四五六七八九十百千萬零〇]+", use.hanja))


# A parenthesis directly after running Hangul takes in the particle before the word: 와곰(熊), 의솔(松).
PARTICLES = ("와", "과", "의", "도", "을", "를", "에", "은", "는", "이", "며", "ᄂᆞᆫ")


def drop_particle(reading, m):
    """Without a leading particle, when only that leaves a word."""
    syls = split_syllables(reading)
    if not syls or len(syls) < 2 or unicodedata.normalize("NFC", syls[0]) not in PARTICLES:
        return reading
    if any(t in m.kpos or t in m.hand_lemmas for t, _ in readings(norm(reading))[:8]):
        return reading
    rest = "".join(syls[1:])
    return unicodedata.normalize("NFC", rest) if any(t in m.modern for t, _ in readings(rest)[:40]) else reading


# Books whose rubies and parentheses are nearly all foreign names or slips: only these uses are readings.
ONLY = {
    "신약마가젼복음셔언해.pdf": {("癩", "문둥")},
    "디구략론.djvu": {("夜", "밤"), ("晝", "낫"), ("水", "물"), ("陸", "흙"), ("麻布", "뵈"), ("布", "뵈"), ("鐵", "쇠"), ("馬", "ᄆᆞᆯ"), ("赤銅", "구리"), ("龜巖", "거복바외")},
    "CNTS-00076805887 西遊見聞.pdf": {("海草", "가ᄉᆞ리"), ("亞鉛", "함셕")},
    "孔夫子言行錄.djvu": {("其", "그")},
    "(고대소설)쳥루지열녀.djvu": {("車", "슈레")},
    "CNTS-00047818150 (세계대전)비행젼쟁.pdf": {("鼠", "쥐"), ("岩石", "바위돌"), ("血戰", "피싸홈"), ("長", "길이"), ("石簇", "활촉")},
    "이인직 혈의 누(상편).pdf": set(),
    "天文學.djvu": set(),
    "CNTS-00076805732 (신소셜) 누구의 죄.pdf": set(),
    "(녀자귀감)고진감래.djvu": set(),
    "(正本)集註周易.djvu": set(),
    "이육사 (1941) 중국문학 오십년사.pdf": set(),
    "지킬과 하이드 (1920) - 릴리어스 호튼 언더우드 역.pdf": set(),
    "섬동지전 - 두껍전 (덕여서림, 1914).djvu": set(),
}
# A reading part sound, part meaning is kept in 勞動夜學讀本, whose rubies are set with care. In the other books
# nearly every such row is a misprinted sound or a phrase; these are the exceptions.
KEEP_MIXED = {("軍浦場", "군포내"), ("烏山", "오믜"), ("五十五尺", "오십오댜"), ("新羅興德王時", "신라흥덕왕ᄯᅢ"), ("韓日間貿易", "한일사이무역"), ("米所出", "쌀소튤"),
              ("一老婆", "한노파"), ("初都邑", "텻도읍"), ("石柱礎", "돌듀툐"), ("詩趣", "시맛"), ("南半區", "남반편"), ("人山", "사람산"), ("船醫", "배의원"),
              ("二十九番", "스물아홉번"), ("一月三十日", "한달ᄉᆞᆷ십일")}
# Read by hand and found to be no reading: a remark, a slip of the printer, a character over the wrong syllable, a loanword.
NOT_A_READING = {
    ("內一局", "내포일"), ("驛", "두"), ("晝", "밤"), ("入港出船船", "입항출항선"), ("傳", "뎐하난"),
    ("基督信者", "크리스챤"), ("感傷的", "센치멘탈"), ("麵麭", "ᄲᅡᆼ"), ("董花", "바이올렛"), ("床", "테불"), ("寶石", "다이야몬드"),
    ("名詞", "받침이 없는"), ("註一", "으로"), ("對象", "뜻"), ("建築用語", "중깃"), ("文法學上", "말본의"), ("聲音學上", "소리갈의"), ("北平", "뻐핑"), ("周時經", "한힌샘"),
    ("狗名", "백산이"), ("月印千江之曲", "사ᄅᆞᄆᆡ"), ("月印千江之曲", "여횔쩌긔"), ("月印干江之曲", "두누ᇆᄌᆞ"), ("龍飛御天歌", "집"), ("天涯水氣中", "가온ᄃᆡ"),
    ("子音", "소리"), ("淸音", "소리"), ("牙音", "소리"), ("舌音", "소리"), ("齒音", "소리"), ("促音", "소리"), ("習慣音", "소리"),
    ("彌", "밍"), ("快", "쾡"), ("漂", "표ᇢ"), ("蚪", "뀨ᇢ"), ("日本里數", "ᄶᅳᆷ"), ("中略", "태어나서"), ("十四行畧", "하는모양과"), ("三千里", "우리"),
    ("裏", "굴속"), ("取拔", "ᄎᆔ급"), ("光", "흐르는빗"), ("年", "나는나히"), ("力", "ᄒᆞ는힘"), ("目", "군대의눈"), ("市場人", "군"), ("裁縫工", "군"),
    ("族", "기"), ("頁", "엽"), ("本義", "토다"), ("本義", "홈은"), ("漁網", "조막이"), ("晝", "은낫이"), ("蛙", "게고리"), ("底", "빝"), ("薪", "섭"), ("畑", "밫"), ("炭", "숱"), ("僧", "숭"),
    ("欲爲", "의고싶"), ("人", "사람이"), ("魂", "넋이"), ("利点", "수"), ("段", "칸"),
    ("戰車", "수레"), ("乾自", "근지"), ("乃終", "임"), ("彈力", "힘"), ("佛蘭西", "프랑쓰국"), ("白耳義", "벨기국"), ("禁", "검"), ("土耳其", "튀르크"), ("語調", "악센트"), ("隧道", "턴넬"),
    ("宇", "임씨"), ("穰", "임씨"), ("月子", "다리"), ("鷲", "게우"), ("望", "발랄"),
    ("火", "차환"), ("老嫗", "고루"), ("白鷺", "겻헤"), ("紅顔", "옥빈"), ("十", "심"), ("鵲", "딕"), ("船", "항"), ("故", "로"), ("曰", "욀"), ("欠", "험"), ("大", "경"),
}


# ---- Modern lemmas ----------------------------------------------------------

def stem_of(lemma, pos):
    return lemma[:-1] if pos in ("verb", "adj") and lemma.endswith("다") else lemma


def written_for(lemma, pos, hanja, covered):
    """The dictionary spelling the use supports: Hanja over the syllables the reading covers, the rest in Hangul.
    動(움작)인다 gives 動이다, 勤(부지)런히 gives 勤런히, 坐(안)진 gives 坐다."""
    stem = stem_of(lemma, pos)
    rest = stem[covered:] if covered < len(stem) else ""
    return hanja + rest + ("다" if pos in ("verb", "adj") else "")


def lemma_of(hanja, reading, tails, m):
    """The modern word of a use, when the form leaves one and its meaning agrees: (lemma, pos) or None."""
    if " " in reading:
        return None
    syls = split_syllables(reading)
    if not syls:
        return None
    best = collections.defaultdict(int)
    tries = [(norm(reading), 3)]
    for tail, _ in tails.most_common(4):
        ts = split_syllables(tail)
        if ts:
            tries.append((norm(reading) + ts[0], 1))
            if len(ts) > 1:
                tries.append((norm(reading) + ts[0] + ts[1], 1))
    for old, weight in tries:
        a = Att(hanja, old, "", "", "", "")
        for lem, pos, _ in lemma_candidates(a, m):
            if any(is_sino_gloss(lem, c, m) for c in hanja) or word_is_sino(lem, m):
                continue
            if len(hanja) > 1 and pos != "noun":
                continue
            w = written_for(lem, pos, hanja, len(syls))
            ev = meaning_evidence(lem, pos, written_of(lem, pos, hanja), hanja, m, a.old) + meaning_evidence(lem, pos, w, hanja, m, a.old)
            if strong(ev):
                best[(lem, pos)] += weight
    if not best:
        return None
    top = max(best.values())
    winners = [k for k, v in best.items() if v == top]
    return winners[0] if len(winners) == 1 else None


# ---- Build ------------------------------------------------------------------

class Row:
    __slots__ = ("hanja", "reading", "meant", "kind", "book", "year", "author", "page", "quote", "count", "tails", "pattern", "lemma")

    def __init__(self, u, kind, book, year, author, page):
        self.hanja, self.reading, self.meant, self.kind, self.pattern = u.hanja, u.reading, u.meant, kind, u.pattern
        self.book, self.year, self.author, self.page, self.quote = book, year, author, page, u.quote
        self.count, self.tails, self.lemma = 0, collections.Counter(), None


def collect(m):
    extract()
    index, pages = read_cache()
    sounds = load_sounds()
    rows, covered = {}, collections.Counter()
    for p in pages:
        key = book_key(p["title"])
        info = book_of(key, index)
        if not info:
            continue
        covered[key] += 1
        book, year, author = info
        for u in uses_of(p["text"], sounds):
            kind = classify(u, sounds)
            syls = split_syllables(u.meant or u.reading)
            if kind in (None, "sound") or not syls or is_noise(u, syls):
                continue
            if u.pattern == "reading-han":
                u.reading = drop_particle(u.reading, m)
                u.meant = drop_particle(u.meant, m)
            pair = (u.hanja, u.reading)
            if key in ONLY and pair not in ONLY[key]:
                continue
            if pair in NOT_A_READING:
                continue
            if kind == "mixed" and pair not in KEEP_MIXED and book != "勞動夜學讀本":
                continue
            k = (u.hanja, u.reading, book)
            r = rows.get(k)
            if r is None:
                r = rows[k] = Row(u, kind, book, year, author, p["title"])
            elif len(u.quote) < len(r.quote):
                r.quote, r.page = u.quote, p["title"]
            r.count += 1
            r.tails[u.tail] += 1
    return rows, covered


USAGE_HEADER = """# Historical 訓讀 usage: Hanja printed with a native Korean reading beside them, one row per distinct use.
# Built by tools/build_kun_usage.py from Korean Wikisource's transcriptions (dump of 2026-11-01).
# Licence: CC BY-SA 4.0 transcription of public-domain books. Most pages are marked unproofread there.
# kind: meaning (no syllable is the character's sound), mixed (some are). reading is as printed; corrected is the
# transcriber's {{sic}} where given. lemma is the modern word when known, otherwise blank. Not loaded by the keyboards.
"""


POS = {"n": "noun", "v": "verb", "a": "adj", "d": "adv", "t": "det", "u": "num", "p": "pron"}


def hand_lemmas():
    """tools/kun_lemmas.txt: (hanja, reading) → (lemma, pos), or None to clear a wrong automatic match."""
    out = {}
    for line in (ROOT / "tools" / "kun_lemmas.txt").open(encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        hanja, forms, lemma, pos = line.split()
        for form in forms.split(","):
            out[(hanja, unicodedata.normalize("NFC", form))] = None if lemma == "-" else (lemma, POS[pos])
    return out


def attach_lemmas(rows, m):
    hand = hand_lemmas()
    for r in rows.values():
        key = (r.hanja, unicodedata.normalize("NFC", r.meant))
        found = hand[key] if key in hand else lemma_of(r.hanja, r.meant, r.tails, m)
        if not found or r.kind != "meaning":
            continue
        lemma, pos = found
        cover = None if r.pattern == "ruby" else 99  # a Hangul word in the text carries its Hanja over the whole of it
        r.lemma = (lemma, pos, written_for(lemma, pos, r.hanja, cover or len(split_syllables(r.meant) or [])))


DICT_HEADER = """# 訓讀 attested in Korean books of 1882-1945: Hanja printed with a native reading beside them.
# Each note gives book, year, Wikisource page and the shortest quote, reading as printed. Built by tools/build_kun_usage.py;
# data/kun-usage.tsv has every use, these rows are the ones that map to a modern word and no better tier has.
# Licence: CC BY-SA 4.0 transcription of public-domain books, Korean Wikisource dump of 2026-11-01. Most pages are
# unproofread there, so a row can carry a transcriber's slip; each note names the page to check. Ranks below eonhae.tsv.
"""


def better_tiers():
    """(lemma, written) of the tiers above this one, and the lemma with each Hanja they lead with."""
    pairs, leading = set(), set()
    for name in ("corpus.tsv", "editorial.tsv", "community.tsv", "eonhae.tsv"):
        for line in (DICT / name).open(encoding="utf-8"):
            if line.startswith("#") or line.startswith("lemma\t"):
                continue
            f = line.split("\t")
            if len(f) > 1 and f[1]:
                pairs.add((f[0], f[1]))
                run = re.match(f"[{HAN}]+", f[1])
                if run:
                    leading.add((f[0], run.group()))
    return pairs, leading


# Attested, but ranking them would take a common word from its usual Hanja.
NOT_FOR_THE_DICTIONARY = {
    ("갖다", "備"),  # 갖고 is 持; 備 is 갖추다, and 통일안's 갖다(備) only shows a final ㅈ
    ("가락", "音調"),  # took 국수가락 and 손가락 with it
    ("쓰임", "用法"),  # a grammar term of 한글, read 용법
    ("다다르다", "臨"),  # 到 stays usual
    ("된장", "味噲"),  # 된醬 stays usual; 味噲 is a Japanese word in 배비장전's list
    ("이끌다", "引"),  # 導 stays usual
}


def write_dict(rows, m):
    pairs, leading = better_tiers()
    cites = collections.defaultdict(list)
    for r in sorted(rows, key=lambda r: (r.year, r.book, len(r.quote))):
        if not r.lemma or r.kind != "meaning":
            continue
        lemma, pos, written = r.lemma
        if (lemma, written) in pairs or (lemma, r.hanja) in leading or (lemma, r.hanja) in NOT_FOR_THE_DICTIONARY:
            continue
        cites[(lemma, r.hanja, pos)].append((written, r))
    out = []
    for (lemma, hanja, pos), items in cites.items():
        written = min((w for w, _ in items), key=len)  # 守다, from 守(직히)나니라, before 守키다, from 守(직)히어라
        seen, notes = set(), []
        for _, r in items:
            if r.book not in seen:
                seen.add(r.book)
                notes.append(f"{r.book} {r.year}, {PAGE_PREFIX.sub('', r.page)}: {r.quote}")
        out.append((lemma, written, pos, class_of(lemma, pos, m), "; ".join(notes[:3])))
    out.sort(key=lambda x: (x[0], x[1]))
    with (DICT / "kun-attested.tsv").open("w", encoding="utf-8") as f:
        f.write(DICT_HEADER)
        f.write("lemma\twritten\tpos\tclass\tsource\tcount\tnote\n")
        for lemma, written, pos, cls, note in out:
            f.write(f"{lemma}\t{written}\t{pos}\t{cls}\tkun-attested\t\t{note}\n")
    print(f"dictionary rows {len(out)}", file=sys.stderr)


PAGE_PREFIX = re.compile(r"^(페이지|Page):")


def build():
    m = Modern()
    rows, covered = collect(m)
    attach_lemmas(rows, m)
    ordered = sorted(rows.values(), key=lambda r: (r.year, r.book, page_number(r.page), r.hanja))
    write_dict(ordered, m)
    with USAGE.open("w", encoding="utf-8") as f:
        f.write(USAGE_HEADER)
        f.write("hanja\treading\tcorrected\tkind\tlemma\tbook\tyear\tauthor\tpage\tcount\tquote\n")
        for r in ordered:
            fixed = r.meant if r.meant != r.reading else ""
            f.write("\t".join([r.hanja, r.reading, fixed, r.kind, r.lemma[0] if r.lemma else "", r.book, r.year, r.author, r.page, str(r.count), r.quote]) + "\n")
    print(f"books {len(covered)}, rows {len(ordered)}", file=sys.stderr)
    for book, n in collections.Counter(r.book for r in ordered).most_common():
        print(f"{n}\t{book}", file=sys.stderr)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--survey", help="list every book of the dump with its count of readings left over")
    args = ap.parse_args()
    if args.survey:
        survey(args.survey)
        return
    build()


if __name__ == "__main__":
    main()
