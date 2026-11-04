"""Build dict/wikt-native.tsv: common native words English Wiktionary lists that still come out in Hangul.

Usage: uv run tools/wikt_native.py [--candidates FILE]

A headword qualifies when it is native (no Hanja form, no borrowing), has a current sense and a frequency in
FusionIME's dictionary, and the engine's first candidate for it, without this file, is the word itself in Hangul. Its
spelling comes from:

  compound  Wiktionary's etymology splits the word, and the parts have rows in the hand-made or attested tiers
  eumhun    a character's 訓音 in Wiktionary or libhangul names the word, and an English sense is shared
  japanese  dict/kun-ja.tsv links the word, and Unihan's definition shares that sense

Every candidate was read: NO_ROW lists the words whose spelling was wrong or unsure, CHOSEN the ones whose parts were a
homograph's. A row is `suggested`, the lowest tier that beats leaving a two-syllable word in Hangul. Needs a release
build of seokdok-cli (SEOKDOK_BIN overrides the path).
"""
import argparse
import collections
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from build_dict import gloss_lemmas  # noqa: E402
from build_eonhae import tokens  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
DICT = ROOT / "dict"
SRC = ROOT / "data" / "sources"
BIN = os.environ.get("SEOKDOK_BIN", str(ROOT / "target" / "release" / "seokdok"))
OUT = DICT / "wikt-native.tsv"
HANGUL = re.compile(r"^[가-힣]+$")
HAN = re.compile(r"^[㐀-鿿豈-﫿]+$")
NOT_CURRENT = {"form-of", "obsolete", "archaic", "dialectal", "North-Korea", "Yukjin", "Jeju", "alt-of", "abbreviation", "slang", "vulgar", "derogatory"}
POS = {"noun": "noun", "adv": "adv", "verb": "verb", "adj": "adj", "num": "num", "pron": "pron", "det": "det"}
# Tiers a compound's part may take its Hanja from, best first.
PART_TIERS = ("corpus", "community", "editorial", "eonhae", "kun-attested", "suggested")
PART_POS = {"noun", "명사", "num", "수사", "det", "관형사", "pron", "대명사", "bound", "의존명사"}

HEADER = """# Common native words that had no usable first candidate. Lemmas, parts of compounds and English senses from
# English Wiktionary (kaikki.org extract, wiktextract; Ylonen, LREC 2022), CC BY-SA 4.0 and GFDL; 訓音 from Wiktionary and
# libhangul (BSD-3-Clause); JMdict (EDRDG), CC BY-SA 4.0. Built by tools/wikt_native.py. Each note names what supports the
# row. Every row was read by hand, but the choice of Hanja is this project's, so this file ranks with `suggested`.
"""

# Read, and the spelling was wrong or unsure. The value says why.
NO_ROW = {
    **dict.fromkeys("품앗이 품삯 몸살 쓸데 길손 왕골 김밥 샛길 알밤 꾀병 깃대 단짝 귓불 맷집 떡밥 나이테 뼈대 덩치 둥치 망울 검불 탈것 마중 생이 쪽박 쪽배".split(), "a part is a homograph or the sense does not carry over"),
    **dict.fromkeys("속셈 끝장 몸집 활개 눈살 놀음 손뼉 여드름 짝사랑 방아쇠 갯벌 물집 톱밥 설빔 이야깃거리 엉덩방아 장수하늘소 아우성 삐삐".split(), "no part-by-part reading makes this word"),
    "들이": "a suffix and a verb form far more often than a noun",
    "지라": "an ending far more often than the spleen",
    "입김": "김 is steam here, and no source spells breath this way",
    "둘째": "a row of its own elsewhere",
    "마흔": "a row of its own elsewhere",
    "두셋": "셋 has no row of its own; 두세 二三 is the common form",
}
# The right spelling where a part had a homograph's Hanja, or the word has a better character. lemma -> (written, parts).
CHOSEN = {
    "새해": ("新年", "새 新, 해 年"),
    "그해": ("其年", "그 其, 해 年"),
    "새싹": ("新芽", "새 新, 싹 芽"),
    "뒤쪽": ("後쪽", "뒤 後"),
    "앞쪽": ("前쪽", "앞 前"),
    "위쪽": ("上쪽", "위 上"),
    "아래쪽": ("下쪽", "아래 下"),
    "몸무게": ("體重", "몸 體, 무겁다 重다"),
    "눈보라": ("雪보라", "눈 雪"),
    "눈덩이": ("雪덩이", "눈 雪"),
    "눈송이": ("雪송이", "눈 雪"),
    "막내딸": ("末女", "막내 末, 딸 女"),
    "아들딸": ("子女", "아들 子, 딸 女"),
    "발밑": ("足下", "발 足, 밑돌다 下돌다"),
    "코밑": ("鼻下", "코 鼻, 밑돌다 下돌다"),
    "벌집": ("蜂巢", "벌 蜂, 둥지 巢"),
    "꽃병": ("花甁", "꽃 花, 병 甁"),
    "칫솔": ("齒솔", "이 齒"),
    "뱃놀이": ("舟놀이", "배 舟"),
    "절터": ("寺터", "절 寺"),
    "칼질": ("刀질", "칼 刀"),
    "신맛": ("酸味", "시다 酸, 맛 味"),
    "부엌칼": ("竈刀", "부엌 竈, 칼 刀"),
    "쇠뿔": ("牛角", "소 牛, 뿔 角"),
    "쇠뼈": ("牛骨", "소 牛, 뼈 骨"),
    "쇠똥": ("牛糞", "소 牛, 똥 糞"),
    "말똥": ("馬糞", "말 馬, 똥 糞"),
    "밤송이": ("栗송이", "밤 栗"),
    "배꽃": ("梨花", "배 梨, 꽃 花"),
    "신발": ("靴", "신발 靴 “shoes” (JMdict)"),
    "벽돌": ("甎", "벽돌 전 甎 (libhangul)"),
    "방울": ("鈴", "방울 령 鈴 (libhangul)"),
    "담요": ("毯", "담요 담 毯 (libhangul)"),
    "진흙": ("泥", "진흙 니 泥 (libhangul)"),
    "알몸": ("倮", "알몸 라 倮 (libhangul)"),
    "낚싯대": ("竿", "낚싯대 간 竿 (libhangul)"),
    "흉터": ("痕", "흉터 흔 痕 (Wiktionary)"),
    "숫돌": ("砥", "숫돌 지 砥 (libhangul)"),
}
# Verbs and adjectives no first candidate reaches, with their parts. (lemma, written, pos, parts)
EXTRA = [
    ("저렇다", "彼렇다", "adj", "저 彼 in editorial.tsv, as 이렇다 此렇다 in corpus.tsv"),
    ("예스럽다", "古스럽다", "adj", "예 古 in eonhae.tsv; “archaic” (Wiktionary)"),
]


def native_headwords():
    """word -> (pos, senses, compound parts), for words a Korean speaker would call native."""
    words = {}
    with (SRC / "kaikki-korean" / "kaikki.org-dictionary-Korean.jsonl").open(encoding="utf-8") as f:
        for line in f:
            d = json.loads(line)
            w, pos = d.get("word", ""), POS.get(d.get("pos"))
            if not pos or not HANGUL.match(w) or len(w) < 2:
                continue
            if pos in ("verb", "adj") and (not w.endswith("다") or len(w) < 3):
                continue
            forms = d.get("forms") or []
            templates = d.get("etymology_templates") or []
            if any("hanja" in (x.get("tags") or []) for x in forms):
                continue
            if any(t["name"] in ("bor", "bor+", "lbor", "obor", "ubor", "ko-etym-sino", "ko-etym-Sino", "ko-etym-nativised") for t in templates):
                continue
            senses = []
            for s in d.get("senses") or []:
                if set(s.get("tags") or []) & NOT_CURRENT or s.get("form_of") or s.get("alt_of"):
                    continue
                senses += s.get("glosses") or []
            senses = [g for g in senses if not re.search(r"form of|contraction|short for|clipping|synonym of|abbreviation|Root of", g)]
            if not senses:
                continue
            parts = None
            for t in templates:
                if t["name"] in ("com", "com+", "compound", "af", "affix", "suffix", "suf", "prefix", "pre"):
                    a = t["args"]
                    got = [a[k] for k in sorted((k for k in a if k.isdigit()), key=int)][1:]
                    if len([p for p in got if p]) >= 2:
                        parts = [p for p in got if p]
                        break
            if w not in words:
                words[w] = (pos, senses, parts)
    return words


def fusion_frequency():
    freq = collections.Counter()
    with (SRC / "fusion-ime" / "ko_dict_all.tsv").open(encoding="utf-8") as f:
        for line in f:
            c = line.rstrip("\n").split("\t")
            if len(c) >= 3 and c[2].isdigit():
                freq[c[0]] = max(freq[c[0]], int(c[2]))
    return freq


def first_candidates(words):
    """What the engine puts first without this file, so a rebuild sees the same gaps."""
    with tempfile.TemporaryDirectory() as d:
        for p in DICT.iterdir():
            if p != OUT:
                (Path(d) / p.name).symlink_to(p.resolve())
        out = subprocess.run([BIN, "--dict", d, "convert"], input="\n".join(words) + "\n", capture_output=True, text=True, check=True).stdout
    first = [l[4:].split("\t")[0] for l in out.splitlines() if l.startswith(" 0.")]
    assert len(first) == len(words)
    return dict(zip(words, first))


def rows_of(name):
    p = DICT / f"{name}.tsv"
    for line in p.open(encoding="utf-8"):
        if line.startswith("#") or line.startswith("lemma\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) >= 5:
            yield f


def unihan_definitions():
    import zipfile

    defs = {}
    with zipfile.ZipFile(SRC / "unihan" / "Unihan.zip") as z:
        for raw in z.read("Unihan_Readings.txt").decode("utf-8").splitlines():
            c = raw.split("\t")
            if len(c) == 3 and c[1] == "kDefinition":
                defs[chr(int(c[0][2:], 16))] = tokens(c[2])
    return defs


def eumhun_rows(words):
    """lemma -> {char: citation} from Wiktionary's eumhun and libhangul's 訓音."""
    lex = collections.defaultdict(set)
    for w, (pos, _, _) in words.items():
        lex[w].add(pos)
    out = collections.defaultdict(dict)

    def note(gloss, char, where):
        word = " ".join(gloss.split()[:-1])
        if not word or " " in word or not all("가" <= c <= "힣" for c in word):
            return
        for lemma, _ in gloss_lemmas(word, lex):
            if lemma in words:
                out[lemma].setdefault(char, f"{gloss} {char} ({where})")

    with (SRC / "kaikki-korean" / "kaikki.org-dictionary-Korean.jsonl").open(encoding="utf-8") as f:
        for line in f:
            d = json.loads(line)
            if d.get("pos") == "character":
                for x in d.get("forms") or []:
                    if "eumhun" in (x.get("tags") or []):
                        note(re.sub(r"\s*\([^)]*\)", "", x["form"]), d["word"], "Wiktionary")
    with (SRC / "libhangul" / "hanja.txt").open(encoding="utf-8") as f:
        for line in f:
            c = line.rstrip("\n").split(":", 2)
            if len(c) == 3 and len(c[1]) == 1 and c[2] and not line.startswith("#"):
                for g in c[2].split(","):
                    note(g.strip(), c[1], "libhangul")
    return out


def part_hanja():
    """Part of a compound -> (Hanja, tier), the best row."""
    best = {}
    for tier in PART_TIERS:
        for f in rows_of(tier):
            if f[2] not in PART_POS or f[0] in best:
                continue
            han = re.match(r"[㐀-鿿豈-﫿]+", f[1])
            # Only a row that covers the whole part: 初음 leaves 음 uncovered.
            if han and han.group() == f[1]:
                best[f[0]] = (f[1], tier)
    return best


def compound_spelling(parts, best):
    """The parts' Hanja run together, the last part free to stay in Hangul: 앞+쪽 前쪽. None when a middle part has no row."""
    clean = []
    for p in parts:
        p = re.sub(r"<[^>]*>", "", p).split("#")[0].strip()
        if p in ("-ㅅ-", "ㅅ", "-") or not p:
            continue
        clean.append(p.lstrip("*^"))
    if len(clean) < 2:
        return None
    written, sources = "", []
    for i, p in enumerate(clean):
        m = re.match(r"^(-?[가-힣]+-?)\(([㐀-鿿豈-﫿]+)\)$", p)
        if m:
            written += m.group(2)
            sources.append(f"{m.group(1).strip('-')} {m.group(2)} (Wiktionary)")
            continue
        p = p.strip("-")
        if not HANGUL.match(p):
            return None
        if p in best:
            written += best[p][0]
            sources.append(f"{p} {best[p][0]} ({best[p][1]})")
        elif i == len(clean) - 1 and written:
            written += p
        else:
            return None
    return written, sources, clean


def candidates():
    words = native_headwords()
    freq = fusion_frequency()
    loans = {f.split("\t")[0] for f in (DICT / "loanwords.tsv").open(encoding="utf-8") if not f.startswith("#")}
    wanted = [w for w in words if freq[w] > 0 and w not in loans]
    first = first_candidates(wanted)
    gaps = [w for w in wanted if first[w] == w]
    print(f"{len(words)} native headwords, {len(wanted)} with a frequency, {len(gaps)} first in Hangul", file=sys.stderr)
    defs = unihan_definitions()
    eum = eumhun_rows(words)
    best = part_hanja()
    japanese = collections.defaultdict(list)
    for f in rows_of("kun-ja"):
        japanese[f[0]].append(f)
    have = set()
    for tier in ("corpus", "editorial", "community", "eonhae", "kun-attested", "suggested"):
        for f in rows_of(tier):
            have.add((f[0], f[1]))
    rows = []
    no_source = []
    for w in gaps:
        pos, senses, parts = words[w]
        en = set().union(*(tokens(g) for g in senses))
        verb = pos in ("verb", "adj")
        got = []
        for ch, cite in eum.get(w, {}).items():
            if en & defs.get(ch, set()):
                got.append((ch + ("다" if verb else ""), "eumhun", cite, freq_key(ch)))
        for f in japanese.get(w, ()):
            ch = re.match(r"[㐀-鿿豈-﫿]+", f[1])
            if ch and len(ch.group()) == 1 and en & defs.get(ch.group(), set()):
                got.append((f[1], "japanese", f"{f[6]}; Wiktionary “{senses[0][:40]}”", freq_key(ch.group())))
        if parts and not verb:
            c = compound_spelling(parts, best)
            if c:
                got.append((c[0], "compound", "compound " + "+".join(c[2]) + " (Wiktionary); " + ", ".join(c[1]), -1))
        if not got:
            no_source.append(w)
        rows.append((w, pos, senses[0][:60], freq[w], got))
    rows.sort(key=lambda r: -r[3])
    return rows, no_source, have


_char_freq = None


def freq_key(ch):
    """Characters in daily use first: how often the character appears in FusionIME's Hanja words."""
    global _char_freq
    if _char_freq is None:
        _char_freq = collections.Counter()
        for line in (DICT / "fusion.tsv").open(encoding="utf-8"):
            f = line.rstrip("\n").split("\t")
            if len(f) > 2 and f[2].isdigit():
                for c in f[1]:
                    _char_freq[c] += int(f[2])
    return _char_freq[ch]


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--candidates", help="write every candidate to this file and stop, for reading")
    args = ap.parse_args()
    rows, no_source, have = candidates()
    if args.candidates:
        with open(args.candidates, "w", encoding="utf-8") as f:
            for w, pos, sense, fr, got in rows:
                f.write(f"{w}\t{pos}\t{fr}\t{sense}\t" + " | ".join(f"{g[0]} {g[1]} {g[2]}" for g in got) + "\n")
        print(f"{sum(1 for r in rows if r[4])} words with a candidate, {len(no_source)} without", file=sys.stderr)
        return
    out = []
    for w, pos, _, _, got in rows:
        if w in NO_ROW:
            continue
        if w in CHOSEN:
            written, parts = CHOSEN[w]
            if (w, written) not in have:
                out.append((w, written, pos, f"{parts}; read by hand"))
            continue
        kinds = collections.defaultdict(list)
        for g in got:
            kinds[g[1]].append(g)
        # A compound's parts say more than one character for the whole; failing that, the commonest character.
        pick = kinds["compound"][:1] or sorted(kinds["eumhun"], key=lambda g: -g[3])[:1] or sorted(kinds["japanese"], key=lambda g: -g[3])[:1]
        for g in pick:
            if (w, g[0]) not in have:
                out.append((w, g[0], pos, g[2]))
    out += [(w, written, pos, parts) for w, written, pos, parts in EXTRA]
    with OUT.open("w", encoding="utf-8") as f:
        f.write(HEADER + "lemma\twritten\tpos\tclass\tsource\tcount\tnote\n")
        for w, written, pos, note in sorted(out):
            f.write(f"{w}\t{written}\t{pos}\t\tsuggested\t\t{note}\n")
    print(f"{len(out)} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
