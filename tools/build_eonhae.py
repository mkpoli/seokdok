"""Build dict/eonhae.tsv: 訓讀 readings attested in historical Korean books.

Usage: uv run tools/build_eonhae.py [--audit DIR]

Reads the Korean Wikisource dump in data/sources/wikisource-ko/ and the saved
나무위키 pages in data/sources/namu-wiki/ (the books' transcriptions), and for the
modern side English Wiktionary via kaikki.org, Unihan and the other dict/ files.
See data/sources/README.md for how to fetch them.

Primer rows come from one-gloss-per-character books (天 하ᄂᆞᆯ 텬). Aligned rows
come from passages printing the Chinese next to its 諺解. An old word becomes a
modern lemma through the folds of crates/seokdok-core/src/fold.rs or through
Wiktionary's Middle Korean descendants; the lemma must exist in a modern source,
and its meaning must agree with the character's (see `meaning_evidence`).
"""
import argparse
import bz2
import collections
import difflib
import json
import re
import sys
import unicodedata
import urllib.parse
import zipfile
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DICT = ROOT / "dict"
SRC = ROOT / "data" / "sources"
DUMP = SRC / "wikisource-ko" / "kowikisource-20261101-pages-articles.xml.bz2"
CACHE = SRC / "wikisource-ko" / "books.jsonl"
NAMU = SRC / "namu-wiki"
# Stated in each citation of these pages: a wiki's transcription under a licence narrower than the file's.
NAMU_TERMS = "CC BY-NC-SA 2.0 KR, unproofread wiki text"

# Books read from the dump. key: the Wikisource page title prefix.
PRIMER_PREFIXES = ("訓蒙字會.pdf", "훈몽자회/", "광주천자문.pdf", "CNTS-00047980571 千字文.pdf", "유합.djvu", "倭語類解", "譯語類解", "詩經諺解 제1책.djvu")
# The date is that of the copy Wikisource transcribes, when it says.
ALIGNED_BOOKS = {
    "소학언해/": ("小學諺解", "1587"),
    "論語諺解 권": ("論語諺解", "1820本"),
    "맹자언해 권": ("孟子諺解", "1693本"),
    "詩經諺解 제": ("詩經諺解", "1820本"),
    "書傳諺解 권": ("書傳諺解", "1820本"),
    "中庸諺解.djvu": ("中庸諺解", "1820本"),
    "大學諺解.djvu": ("大學諺解", "1695本"),
    "효경언해": ("孝經諺解", "1590"),
    "쥬역언해 제": ("周易諺解", "1862本"),
    "十九史略諺解 권": ("十九史略諺解", "1772"),
    "고문진보언해 권": ("古文眞寶諺解", "英祖代"),
    "고문진보언해 후집": ("古文眞寶諺解", "英祖代"),
}


def wanted(title):
    t = re.sub(r"^(페이지|Page):", "", title)
    return t.startswith(PRIMER_PREFIXES) or t.startswith(tuple(ALIGNED_BOOKS))


def extract_pages():
    """The pages of the listed books, cached as JSON lines."""
    if CACHE.exists() and CACHE.stat().st_mtime > Path(__file__).stat().st_mtime:
        return [json.loads(l) for l in CACHE.open(encoding="utf-8")]
    out = []
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
            if ns in ("0", "250") and title and wanted(title):
                out.append({"title": title, "ns": ns, "text": text or ""})
    with CACHE.open("w", encoding="utf-8") as w:
        for p in out:
            w.write(json.dumps(p, ensure_ascii=False) + "\n")
    return out


def page_number(title):
    m = re.search(r"/(\d+)$", title)
    return int(m.group(1)) if m else 0


# ---- Old Hangul ----------------------------------------------------------

TONE = re.compile("[\u302E\u302F\u00B7\u30FB]")
L = set(range(0x1100, 0x1160)) | set(range(0xA960, 0xA97D))
V = set(range(0x1160, 0x11A8)) | set(range(0xD7B0, 0xD7C7))
T = set(range(0x11A8, 0x1200)) | set(range(0xD7CB, 0xD7FC))


def norm(s):
    """Conjoining jamo with tone dots removed."""
    return TONE.sub("", unicodedata.normalize("NFD", s))


def syllables(s):
    """Split normalized text into L+V+T* units; None on anything else."""
    cs = list(s)
    out, i = [], 0
    while i < len(cs):
        j = i
        while j < len(cs) and ord(cs[j]) in L:
            j += 1
        if j == i:
            return None
        k = j
        while k < len(cs) and ord(cs[k]) in V:
            k += 1
        if k == j:
            return None
        while k < len(cs) and ord(cs[k]) in T:
            k += 1
        out.append("".join(cs[i:k]))
        i = k
    return out


L_CLUSTER = {"\u1109\u1100": "\u1101", "\u1109\u1103": "\u1104", "\u1109\u1107": "\u1108", "\u1109\u110C": "\u110D",
             "\u1107\u1109": "\u110A", "\u1107\u1100": "\u1101", "\u1107\u1103": "\u1104", "\u1107\u110C": "\u110D",
             "\u1107\u110A": "\u110A", "\u1107\u1109\u1100": "\u1101", "\u1107\u1109\u1103": "\u1104",
             "\u1107\u1109\u1107": "\u1108", "\u1107\u1109\u110C": "\u110D", "\u1100\u1100": "\u1101",
             "\u1103\u1103": "\u1104", "\u1107\u1107": "\u1108", "\u1109\u1109": "\u110A", "\u110C\u110C": "\u110D",
             "\u1102\u1102": "\u1102", "\u1112\u1112": "\u1112"}
L_ONE = {"\u1140": ["\u1109"], "\u114C": ["\u110B"], "\u112B": ["\u1107", "\u110B"], "\u112D": ["\u1101"],
         "\u111E": ["\u1101"], "\u1122": ["\u1101"], "\u112F": ["\u1104"], "\u1120": ["\u1104"], "\u1123": ["\u1104"],
         "\u1132": ["\u1108"], "\u1124": ["\u1108"], "\u1136": ["\u110D"], "\u1127": ["\u110D"], "\u1126": ["\u110D"],
         "\u1121": ["\u110A"], "\u1125": ["\u110A"], "\u114C": ["\u110B"], "\u1159": ["\u1112"], "\u1158": ["\u1112"]}
V_SEQ = {"\u1169\u1161": "\u116A", "\u1169\u1162": "\u116B", "\u1169\u1175": "\u116C", "\u116E\u1165": "\u116F",
         "\u116E\u1166": "\u1170", "\u116E\u1175": "\u1171", "\u1173\u1175": "\u1174", "\u1161\u1175": "\u1162",
         "\u1165\u1175": "\u1166", "\u1175\u1161": "\u1163", "\u1175\u1165": "\u1167", "\u1175\u1169": "\u116D",
         "\u1175\u116E": "\u1172"}
T_ONE = {"\u11EB": "\u11BA", "\u11F0": "\u11BC", "\u11F9": "\u11C2", "\u11EE": "\u11BC", "\u11F4": "\u11C1"}
T_SEQ = {"\u11AF\u11A8": "\u11B0", "\u11AF\u11B7": "\u11B1", "\u11AF\u11B8": "\u11B2", "\u11AF\u11BA": "\u11B3",
         "\u11AF\u11C0": "\u11B4", "\u11AF\u11C1": "\u11B5", "\u11AB\u11BD": "\u11AC", "\u11AB\u11C2": "\u11AD",
         "\u11AF\u11C2": "\u11B6", "\u11B8\u11BA": "\u11B9", "\u11A8\u11BA": "\u11AA"}


def fold_unit(u):
    """The modern (L, V, T) readings of one syllable, with a penalty: fold_l, fold_v, fold_t of fold.rs."""
    ls = "".join(c for c in u if ord(c) in L)
    vs = "".join(c for c in u if ord(c) in V)
    ts = "".join(c for c in u if ord(c) in T)
    if len(ls) > 1:
        ls = L_CLUSTER.get(ls)
        if not ls:
            return []
    lead = L_ONE.get(ls, [ls] if 0x1100 <= ord(ls) <= 0x1112 else [])
    if len(ts) == 2:
        ts = T_SEQ.get(ts, "")
        if not ts:
            return []
    if ts:
        ts = T_ONE.get(ts, ts)
        if not 0x11A8 <= ord(ts) <= 0x11C2:
            return []
    if vs == "\u119E":
        vowels = [("\u1161", 0), ("\u1173", 0.05), ("\u1169", 0.5), ("\u116E", 0.5)]
    elif vs in ("\u11A1", "\u119E\u1175"):
        vowels = [("\u1162", 0), ("\u1166", 0.3), ("\u1174", 0.4)]
    elif len(vs) == 1 and 0x1161 <= ord(vs) <= 0x1175:
        vowels = [(vs, 0)]
    elif vs in V_SEQ:
        vowels = [(V_SEQ[vs], 0)]
    else:
        return []
    return [(l, v, ts or None, p) for l in lead for v, p in vowels]


def to_syl(l, v, t):
    return chr(0xAC00 + (ord(l) - 0x1100) * 588 + (ord(v) - 0x1161) * 28 + (ord(t) - 0x11A7 if t else 0))


TENSE = {"\u1100": "\u1101", "\u1103": "\u1104", "\u1107": "\u1108", "\u1109": "\u110A", "\u110C": "\u110D"}
PLAIN = {"\u1163": "\u1161", "\u1167": "\u1165", "\u116D": "\u1169", "\u1172": "\u116E"}
CODA_OF = {"\u1100": "\u11A8", "\u1102": "\u11AB", "\u1103": "\u11AE", "\u1105": "\u11AF", "\u1106": "\u11B7",
           "\u1107": "\u11B8", "\u1109": "\u11BA", "\u110C": "\u11BD", "\u110E": "\u11BE", "\u110F": "\u11BF",
           "\u1110": "\u11C0", "\u1111": "\u11C1", "\u1112": "\u11C2"}


def rewrites(u, last=False):
    """Era spellings of one syllable (l, v, t): PALATAL, GLIDE, LABIAL, FINAL of fold.rs, and the
    regular vowel raising of later syllables (나모 나무, 구룸 구름), which a dictionary hit must confirm."""
    l, v, t, p = u
    yield u
    if l in ("\u1103", "\u1110") and v in PLAIN:
        yield ("\u110C" if l == "\u1103" else "\u110E", PLAIN[v], t, p + 0.2)
    if l in ("\u1103", "\u1110") and v == "\u1175":
        yield ("\u110C" if l == "\u1103" else "\u110E", v, t, p + 0.2)
    if l in ("\u1109", "\u110C", "\u110E") and v in PLAIN:
        yield (l, PLAIN[v], t, p + 0.2)
    if l in ("\u1106", "\u1107", "\u1111") and v == "\u1173":
        yield (l, "\u116E", t, p + 0.2)
    if v == "\u1174":  # 싀어미 시어미, 긔러기 기러기
        yield (l, "\u1175", t, p + 0.2)
    if t == "\u11BA":
        for alt in ("\u11BD", "\u11BE", "\u11C0", "\u11BB"):
            yield (l, v, alt, p + 0.3)
    if t == "\u11AE":
        for alt in ("\u11BA", "\u11BB", "\u11BD", "\u11BE", "\u11C0"):
            yield (l, v, alt, p + 0.3)
    if t == "\u11B8":
        for alt in ("\u11B9", "\u11C1"):
            yield (l, v, alt, p + 0.3)
    if t == "\u11A8":
        yield (l, v, "\u11BF", p + 0.3)
        yield (l, v, "\u11A9", p + 0.3)


def raise_vowel(u):
    l, v, t, p = u
    yield u
    if v == "\u1169":
        yield (l, "\u116E", t, p + 0.25)
    if v == "\u116E":
        yield (l, "\u1173", t, p + 0.25)
    if v == "\u1165":
        yield (l, "\u1166", t, p + 0.25)
    if v == "\u1167":
        yield (l, "\u1166", t, p + 0.25)
        yield (l, "\u1168", t, p + 0.25)


def readings(word, cap=600):
    """Modern-letter readings of an old word (conjoining jamo), cheapest first. Mirrors fold.rs:
    each syllable's folds and rewrites, the initial ㄴ of 녀름 and 니블, plus a carried coda (머글 먹을)."""
    syls = syllables(word)
    if not syls or len(syls) > 7:
        return []
    states = [([], 0.0)]
    for i, s in enumerate(syls):
        units = []
        for u in fold_unit(s):
            for r in rewrites(u):
                if i > 0:
                    units.extend(raise_vowel(r))
                    if r[0] in TENSE:  # 말ᄉᆞᆷ 말씀
                        units.append((TENSE[r[0]], r[1], r[2], r[3] + 0.45))
                    if r[0] == "\u1112":  # 부헝이 부엉이
                        units.append(("\u110B", r[1], r[2], r[3] + 0.45))
                    if s.find("\u119E") >= 0 and r[0] in ("\u110C", "\u110E") and r[1] == "\u1173":  # 아ᄎᆞᆷ 아침
                        units.append((r[0], "\u1175", r[2], r[3] + 0.3))
                else:
                    units.append(r)
                    if r[0] == "\u1102" and (r[1] == "\u1175" or r[1] in PLAIN):
                        units.append(("\u110B", r[1], r[2], r[3] + 0.2))
        if not units:
            return []
        nxt = []
        for prefix, cost in states:
            for u in units:
                nxt.append((prefix + [u], cost + u[3]))
        nxt.sort(key=lambda x: x[1])
        states = nxt[:cap]
    out = {}
    for units, cost in states:
        text = "".join(to_syl(l, v, t) for l, v, t, _ in units)
        if text not in out or cost < out[text]:
            out[text] = cost
    return sorted(out.items(), key=lambda x: x[1])


def jamo(syl):
    s = ord(syl) - 0xAC00
    t = s % 28
    return chr(0x1100 + s // 588), chr(0x1161 + (s % 588) // 28), chr(0x11A7 + t) if t else None


def verb_stems(text):
    """Dictionary forms an adnominal reading can come from: 울 울다, 될 되다, 먹을 먹다, 무를 묻다, 더울 덥다, 큰 크다."""
    out = []
    last = text[-1]
    l, v, t = jamo(last)
    head = text[:-1]
    if t in ("\u11AF", "\u11AB"):
        out.append(head + to_syl(l, v, None) + "다")  # 될 → 되다, 큰 → 크다
    if t == "\u11AF":
        out.append(text + "다")  # 울 → 울다
    if t == "\u11AB":
        out.append(head + to_syl(l, v, "\u11AF") + "다")  # ㄹ-stem adjectives: 긴 → 길다
    if head and t == "\u11AF" and v in ("\u1173", "\u1161", "\u1169", "\u116E", "\u1165"):
        pl, pv, pt = jamo(head[-1])
        if pt is None:
            if l in CODA_OF:
                out.append(head[:-1] + to_syl(pl, pv, CODA_OF[l]) + "다")  # 머글 → 먹다, 안즐 → 앉다
            if l == "\u1105":
                out.append(head[:-1] + to_syl(pl, pv, "\u11AE") + "다")  # 무를 → 묻다
                out.append(head[:-1] + head[-1] + "르다")  # 니를 → 이르다
            if l == "\u110B":
                out.append(head[:-1] + to_syl(pl, pv, "\u11B8") + "다")  # 더울 → 덥다
                out.append(head[:-1] + to_syl(pl, pv, "\u11BA") + "다")  # 지을 → 짓다
        elif l == "\u110B":
            out.append(head + "다")  # 먹을 → 먹다
        elif l in CODA_OF:
            cluster = T_SEQ.get(pt + CODA_OF[l])
            if cluster:
                out.append(head[:-1] + to_syl(pl, pv, cluster) + "다")  # 업슬 → 없다
    return out


# ---- Primers --------------------------------------------------------------

HAN = "\u3400-\u4DBF\u4E00-\u9FFF\uF900-\uFAFF"
HANGUL = "\uAC00-\uD7A3\u1100-\u11FF\uA960-\uA97C\uD7B0-\uD7FB\u302E\u302F"


class Att:
    """One attestation: the book glosses `char` as the old word `old`."""
    __slots__ = ("char", "old", "eum", "book", "year", "loc", "modern", "text", "tier")

    def __init__(self, char, old, eum, book, year, loc, modern="", text="", tier="primer"):
        self.char, self.old, self.eum, self.book, self.year, self.loc = char, norm(old), norm(eum), book, year, loc
        self.modern, self.text, self.tier = modern, text, tier


def plain(cell):
    """Wiki text without templates that only mark up: {{이체자|X}} is X, {{SIC|a|b}} is the corrected b."""
    cell = re.sub(r"\{\{SIC\|[^|{}]*\|([^|{}]*)\}\}", r"\1", cell)
    cell = re.sub(r"\{\{이체자\|([^|{}]*)(\|[^{}]*)?\}\}", r"\1", cell)
    cell = re.sub(r"\{\{refn\|[^{}]*\}\}", "", cell)
    cell = re.sub(r"<ref[^>]*>.*?</ref>", "", cell)
    return cell


def strip_eum(cell, eum):
    """The gloss of 달〯오 with the reading 오."""
    c, e = norm(cell), norm(eum)
    return c[: -len(e)] if e and c.endswith(e) and len(c) > len(e) else ""


def parse_hunmong_table(page, out):
    n = page_number(page["title"])
    for ch, eum, cell in re.findall(r"\{\{윗주\|([^|]+)\|([^|}]+)(?:\|[^}]*)?\}\}\s*\|\|\s*([^\n|]*)", plain(page["text"])):
        first = cell.split("<br>")[0].strip()
        old = strip_eum(first, eum)
        if len(ch) == 1 and old:
            out.append(Att(ch, old, eum, "訓蒙字會", "1527", f"訓蒙字會.pdf/{n}"))
    # The other markup of the same book: {{훈몽자회|주제=天文 / |天|하ᄂᆞᆯ〮|텬|note ...}}.
    for block in re.findall(r"\{\{훈몽자회\s*(.*?)\}\}(?=\{\{|\s*$|\n)", plain(page["text"]), flags=re.S):
        for line in block.splitlines():
            parts = line.lstrip("|").split("|")
            if len(parts) >= 3 and len(parts[0]) == 1 and "=" not in parts[0] and parts[1].strip():
                out.append(Att(parts[0], parts[1].strip(), parts[2].strip(), "訓蒙字會", "1527", f"訓蒙字會.pdf/{n}"))


def parse_hunmong_page(page, out):
    """훈몽자회/天倫 and so on: `: {{크게|{{Wikt|身}}}} 몸〮 신(몸 신)` or, in 樹木 and others, `: {{크게|枳}} 티ᇰ〮ᄌᆞ〮 기〯(탱자 지)`,
    under folio headings; in the parentheses the Wikisource editors put the gloss in modern letters and, after a
    semicolon, the modern 訓音."""
    folio = ""
    section = page["title"].split("/")[-1]
    for line in plain(page["text"]).splitlines():
        m = re.match(r"==\s*([상중하]\d+[AB])\s*==", line)
        if m:
            folio = m.group(1)
            continue
        m = re.match(r":\s*\{\{크게\|(?:\{\{[Ww]ikt\|)?([^{}|]+)(?:\}\})?\}\}\s*([^\s(\[]+)\s+([^\s(\[]+)\s*[(\[]([^)\]]*)[)\]]", line)
        if not m:
            continue
        ch, old, eum, par = m.groups()
        if len(ch) != 1:
            continue
        part = par.split(";")[0].strip()
        modern = part.split("(")[0].split("·")[0].split(",")[0].strip()
        if ";" not in par:
            # (몸 신): the gloss and its reading in one.
            modern = re.sub(r"\s+\S+$", "", modern)
        out.append(Att(ch, old, eum, "訓蒙字會", "1527", f"{section} {folio}", modern=modern.replace(" ", "")))


def parse_big_lines(page, book, year, out):
    """{{더더더크게|天}} 하ᄂᆞᆯ 텬 (千字文 editions, 類合)."""
    n = page_number(page["title"])
    base = re.sub(r"^[^:]+:", "", page["title"]).split("/")[0]
    for ch, rest in re.findall(r"\{\{더더더크게\|((?:\{\{[^{}]*\}\}|[^{}])*)\}\}\s*([^\n{]*)", plain(page["text"])):
        ch = re.sub(r"[°\s]", "", ch)
        words = rest.strip().split()
        if len(ch) != 1 or len(words) != 2:
            continue
        gloss, eum = words
        if gloss.startswith("("):
            continue  # the transcriber's parenthesis: a doubtful reading
        out.append(Att(ch, gloss, eum, book, year, f"{base}/{n}"))


def parse_waeeo(page, out):
    """;江 / :물강 / :고우 / :○예: the first line is gloss and reading run together."""
    n = page_number(page["title"])
    base = re.sub(r"^[^:]+:", "", page["title"]).split("/")[0]
    lines = plain(page["text"]).splitlines()
    for i, l in enumerate(lines):
        if not l.startswith(";") or i + 1 >= len(lines) or not lines[i + 1].startswith(":"):
            continue
        ch = l[1:].strip()
        run = norm(lines[i + 1][1:].strip())
        syl = syllables(run)
        if len(ch) == 1 and syl and len(syl) >= 2:
            out.append(Att(ch, "".join(syl[:-1]), syl[-1], "倭語類解", "18C", f"{base}/{n}"))


def parse_yeogeo(page, out):
    """;天 / :텬 / :텬 / :○하ᄂᆞᆯ: a Chinese word, its two readings, then the Korean after ○."""
    n = page_number(page["title"])
    base = re.sub(r"^[^:]+:", "", page["title"]).split("/")[0]
    lines = plain(page["text"]).splitlines()
    for i, l in enumerate(lines):
        if not l.startswith(";"):
            continue
        head = l[1:].strip()
        gloss = next((x for x in lines[i + 1 : i + 5] if x.startswith(":○")), None)
        if gloss is None:
            continue
        for g in re.split(r"[、,]|又云|又|○", gloss[2:]):
            g = g.strip()
            if g and re.fullmatch(f"[{HANGUL}]+", g) and re.fullmatch(f"[{HAN}]+", head):
                out.append(Att(head, g, "", "譯語類解", "1690", f"{base.split(' 古')[0]} {base.split(' ')[-1].split('.')[0]}/{n}"))


def parse_mulmyeong(page, out):
    """詩經諺解 物名: 檉{{더 작게|쳥갯버들}}, the reading then the gloss, alternatives split by ○."""
    n = page_number(page["title"])
    if n < 2 or n >= 45:
        return
    for ch, small in re.findall(rf"([{HAN}])\{{\{{더 작게\|([^{{}}]*(?:\{{\{{[^{{}}]*\}}\}}[^{{}}]*)*)\}}\}}", plain(page["text"])):
        small = re.sub(r"\{\{[^{}]*\}\}", "○", small)
        syl = syllables(norm(small.split("○")[0]))
        if not syl or len(syl) < 2:
            continue
        eum = syl[0]
        for i, part in enumerate(small.split("○")):
            s = syllables(norm(part))
            if not s:
                continue
            word = "".join(s[1:]) if i == 0 else "".join(s)
            if word:
                out.append(Att(ch, word, eum, "詩經諺解 物名", "1613", f"詩經諺解 제1책/{n}"))


def primers(pages):
    out = []
    for p in pages:
        t = re.sub(r"^(페이지|Page):", "", p["title"])
        if t.startswith("訓蒙字會.pdf/"):
            parse_hunmong_table(p, out)
        elif t.startswith("훈몽자회/"):
            parse_hunmong_page(p, out)
        elif t.startswith("광주천자문.pdf/"):
            parse_big_lines(p, "光州千字文", "1575", out)
        elif t.startswith("CNTS-00047980571 千字文.pdf/"):
            parse_big_lines(p, "石峯千字文", "1583", out)
        elif t.startswith("유합.djvu/"):
            parse_big_lines(p, "類合", "1913", out)
        elif t.startswith("倭語類解"):
            parse_waeeo(p, out)
        elif t.startswith("譯語類解"):
            parse_yeogeo(p, out)
        elif t.startswith("詩經諺解 제1책.djvu/"):
            parse_mulmyeong(p, out)
    return out


def snapshot_rows(path):
    """Table rows of a saved accessibility snapshot: (level-3 heading above, [(cell text, [(link text, target)])])."""
    section, row, row_indent, cell_indent = "", None, 0, 0
    rows = []
    lines = path.read_text(encoding="utf-8").splitlines()
    for i, line in enumerate(lines):
        indent = len(line) - len(line.lstrip())
        item = line.strip()
        if row is not None and indent <= row_indent:
            rows.append((section, row))
            row = None
        if item.startswith("- heading [level=3"):
            text = next((x.strip() for x in lines[i + 1 : i + 8] if x.strip().startswith("- text: ")), "- text: ")
            section = text[len("- text: "):]
        elif item.startswith("- row ["):
            row, row_indent, cell_indent = [], indent, indent + 2
        elif row is not None and indent == cell_indent and item.startswith("- cell"):
            m = re.match(r'- cell(?: "(.*)")? \[ref=[^\]]*\]', item)
            row.append([m.group(1) or "", []])
        elif row:
            m = re.match(r'- link "(.*)" \[ref', item)
            if m:
                target = lines[i + 1].strip() if i + 1 < len(lines) else ""
                target = target[len("- /url: "):] if target.startswith("- /url: ") else ""
                row[-1][1].append((m.group(1), urllib.parse.unquote(target).removeprefix("/w/")))
                row[-1][0] += " " + m.group(1)
            elif item.startswith("- text: "):
                row[-1][0] += " " + item[len("- text: "):].strip('"')
    return rows


def gloss_pairs(text, fits):
    """The (gloss, reading) pairs of a cell: 처엄 초; 남녁 남 앏 남; 밧골 역 쉬울 이 去聲. Han characters, 원간 and what
    follows them are the transcriber's notes (今音 됴, 又 音 완, 一 作 욕심 욕). A gloss of several words is a phrase
    (믈 ᄲᅳ릴 쇄), which no lemma can be, and a cell with a word left over is not understood, so neither gives a pair."""
    words = text.split()
    for i, w in enumerate(words):
        if re.search(f"[{HAN}]", w) or w == "원간":
            words = words[:i]
            break
    if len(words) == 1:  # 벋뉴, 초ᄒᆞᄅᆞ삭: run together
        syl = syllables(norm(words[0]))
        words = ["".join(syl[:-1]), syl[-1]] if syl and len(syl) >= 2 else []
    pairs, gloss = [], []
    for w in words:
        syl = syllables(norm(w))
        if gloss and syl and len(syl) == 1 and fits(w):
            pairs.append((gloss, w))
            gloss = []
        else:
            gloss.append(w)
    return [] if gloss else [(g[0], e) for g, e in pairs if len(g) == 1]


def namu_primers(m):
    """新增類合 and 訓蒙字會 上 on 나무위키: rows of number, character, gloss with its reading. A cell can hold variants
    of the character (煙烟), or print one before the gloss (鶯 굇고리 ᄋᆡᆼ); a link on a 訓蒙字會 gloss goes to the
    article on the modern word (가랍나모 → 떡갈나무), which serves as the Wikisource editors' parentheses do."""
    out = []
    for name, book, year, title, numbered in (("yuhap-sang.yml", "新增類合", "1576", "신증유합/상권", True),
                                              ("yuhap-ha.yml", "新增類合", "1576", "신증유합/하권", True),
                                              ("hunmong-sang.yml", "訓蒙字會", "1527", "훈몽자회/상권", False)):
        for section, cells in snapshot_rows(NAMU / name):
            if len(cells) != (3 if numbered else 2):
                continue
            chars = re.findall(f"[{HAN}\U00020000-\U000323AF]", cells[-2][0])
            text = cells[-1][0].strip()
            lead = re.match(rf"((?:[{HAN}]\s+)+)", text)
            if lead:
                chars += lead.group(1).split()
                text = text[lead.end():]
            chars = list(dict.fromkeys(unicodedata.normalize("NFC", c) for c in chars))
            if not chars or (numbered and not cells[0][0].strip().isdigit()):
                continue

            def fits(eum):
                # Gloss and reading alone leave one way to read the cell; primer_rows judges the reading as for any book.
                return len(text.split()) <= 2 or (chars[0] in m.char_readings and reading_starts_alike(chars[0], eum, m))

            pairs = gloss_pairs(text, fits)
            where = cells[0][0].strip() if numbered else section
            loc = f"나무위키 {title} {where} ({NAMU_TERMS})"
            for gloss, eum in pairs:
                targets = [t for lt, t in cells[-1][1] if lt == gloss and re.fullmatch("[\uAC00-\uD7A3]+", t)]
                for ch in chars:
                    out.append(Att(ch, gloss, eum, book, year, loc, modern=targets[0] if targets else ""))
    return out


# ---- The modern side ------------------------------------------------------

KOREAN_POS = {"명사": "noun", "대명사": "pron", "수사": "num", "의존명사": "bound", "부사": "adv", "관형사": "det",
              "감탄사": "interj", "동사": "verb", "형용사": "adj"}
WORD_POS = {"noun": "noun", "verb": "verb", "adj": "adj", "adv": "adv", "num": "num", "pron": "pron", "det": "det"}
STOP = set("""a an the of to in on at by for from with and or as is be being been it its that this these those not no
one ones some any all thing things something someone person people man used use using make made
do does done have has get got give go goes come put take let way kind sort type state states country countries
name names surname family chinese korean hanja form etc also e g i see especially usually often generally very
more most less part piece place time times good bad big small great little long short high low up down out
over under off into become becomes cause causes who which what when where how you your he she they we us
character characters word words variant variants classifier measure radical sense obsolete archaic old new
simplified traditional abbreviation general common other another same each every many much few
kingdom dynasty ancient""".split())


STOP_STEMS = STOP | {w[:-1] for w in STOP if w.endswith("e")} | {"mak", "tak", "giv", "com", "hav", "becom", "plac", "tim"}


def tokens(text):
    out = set()
    for w in re.findall(r"[a-z]+", text.lower()):
        if len(w) < 3 or w in STOP:
            continue
        for suf in ("ing", "ies", "ed", "es", "s", "ly"):
            if w.endswith(suf) and len(w) - len(suf) >= 3:
                w = w[: -len(suf)]
                break
        if w.endswith("e") and len(w) > 3:
            w = w[:-1]
        if w not in STOP_STEMS:
            out.add(w)
    return out


class Modern:
    def __init__(self):
        self.pos = collections.defaultdict(set)  # lemma -> parts of speech
        self.senses = collections.defaultdict(set)  # (lemma, pos) -> English tokens
        self.modern = set()  # headwords of a modern dictionary
        self.kpos = collections.defaultdict(set)  # lemma -> parts of speech in Wiktionary
        self.hand_lemmas = set()  # lemmas with a hand-made row
        self.wikt_hanja = collections.defaultdict(set)  # word -> its Hanja forms in Wiktionary
        self.wikt_native = set()  # words with a Wiktionary entry that has no Hanja form
        self.pairs = collections.defaultdict(set)  # (lemma, written) -> dict tiers
        self.lemma_rows = collections.defaultdict(list)  # lemma -> (written, pos, class, source)
        self.sino = collections.defaultdict(set)  # reading -> Hanja words
        self.char_def = collections.defaultdict(set)  # char -> English tokens
        self.origin_hanja = set()  # words Sino-Korean by origin (etym.tsv, and stdict through FusionIME)
        self.char_freq = collections.Counter()
        self.char_readings = collections.defaultdict(set)
        self.old = collections.defaultdict(set)  # old word (normalized) -> modern lemmas (Wiktionary)
        self.old_senses = collections.defaultdict(set)  # old word -> English tokens
        self._kaikki()
        self._middle_korean()
        self._dict_files()
        self._unihan()

    def _kaikki(self):
        for line in (SRC / "kaikki-korean" / "kaikki.org-dictionary-Korean.jsonl").open(encoding="utf-8"):
            o = json.loads(line)
            w, pos = o.get("word", ""), o.get("pos")
            if pos == "character":
                for s in o.get("senses", []):
                    for g in s.get("glosses", []):
                        m = re.findall(r"“([^”]*)”", g)
                        if m:
                            self.char_def[w] |= tokens(" ".join(m))
                continue
            if pos not in WORD_POS or not w or not all(0xAC00 <= ord(c) <= 0xD7A3 for c in w):
                continue
            pos = WORD_POS[pos]
            current = False
            for s in o.get("senses", []):
                tags = s.get("tags") or []
                if "form-of" in tags or set(tags) & {"obsolete", "archaic", "dialectal", "North-Korea", "Yukjin", "Jeju"}:
                    continue
                current = True
                for g in s.get("glosses", []):
                    self.senses[(w, pos)] |= tokens(g)
            if current:
                self.pos[w].add(pos)
                self.kpos[w].add(pos)
                self.modern.add(w)
                if not any("hanja" in (f.get("tags") or []) for f in o.get("forms", [])):
                    self.wikt_native.add(w)
            for f in o.get("forms", []):
                if "hanja" in (f.get("tags") or []):
                    self.sino[w].add(f.get("form", ""))
                    self.wikt_hanja[w].add(f.get("form", ""))
            et = o.get("etymology_text") or ""
            if "Sino-Korean" in et[:40]:
                continue
            for old in re.findall(r"[Ff]rom Middle Korean ([^\s,.;()]+)|as Middle Korean ([^\s,.;()]+)", et):
                old = norm(old[0] or old[1])
                if syllables(old.removesuffix("다") or old):
                    self.old[old].add(w)

    def _middle_korean(self):
        p = SRC / "kaikki-middle-korean" / "kaikki.org-dictionary-MiddleKorean.jsonl"
        for line in p.open(encoding="utf-8"):
            o = json.loads(line)
            w = norm(o.get("word", ""))
            for d in o.get("descendants") or []:
                if d.get("lang_code") == "ko" and d.get("word") and all(0xAC00 <= ord(c) <= 0xD7A3 for c in d["word"]):
                    self.old[w].add(d["word"])
            for s in o.get("senses", []):
                for g in s.get("glosses", []):
                    self.old_senses[w] |= tokens(g)

    def _dict_files(self):
        for name in ("corpus.tsv", "editorial.tsv", "community.tsv", "suggested.tsv", "gloss.tsv", "kun-ja.tsv", "kun-ja-weak.tsv"):
            for line in (DICT / name).open(encoding="utf-8"):
                if line.startswith("#") or line.startswith("lemma\t"):
                    continue
                f = line.rstrip("\n").split("\t")
                if len(f) < 5:
                    continue
                lemma, written, pos, cls, source = f[:5]
                pos = KOREAN_POS.get(pos, pos)
                self.pairs[(lemma, written)].add(source)
                self.lemma_rows[lemma].append((written, pos, cls, source))
                if source != "japanese-weak":
                    self.pos[lemma].add(pos)
                if source in ("corpus", "editorial", "community"):
                    self.modern.add(lemma)
                    self.hand_lemmas.add(lemma)
        for line in (SRC / "fusion-ime" / "ko_dict_all.tsv").open(encoding="utf-8"):
            f = line.split("\t", 5)
            if f[0]:
                self.modern.add(f[0])
                if len(f) > 4:
                    m = re.match(r"\[([^\]]+)\]", f[4])
                    if m and m.group(1) in KOREAN_POS:
                        self.pos[f[0]].add(KOREAN_POS[m.group(1)])
        for name in ("fusion.tsv", "sino.tsv", "etym.tsv"):
            for line in (DICT / name).open(encoding="utf-8"):
                if line.startswith("#") or line.startswith("reading\t"):
                    continue
                f = line.rstrip("\n").split("\t")
                if len(f) < 2:
                    continue
                self.sino[f[0]].add(f[1])
                if name == "etym.tsv":
                    self.origin_hanja.add(f[0])
                if len(f[1]) == 1 and name == "sino.tsv":
                    self.char_readings[f[1]].add(f[0])
                if len(f[1]) == 1 and len(f) > 2 and f[2].isdigit():
                    self.char_freq[f[1]] += int(f[2])

    def _unihan(self):
        with zipfile.ZipFile(SRC / "unihan" / "Unihan.zip") as z:
            with z.open("Unihan_Readings.txt") as f:
                for raw in f:
                    line = raw.decode("utf-8")
                    if "\tkDefinition\t" in line:
                        cp, _, d = line.rstrip("\n").split("\t", 2)
                        self.char_def[chr(int(cp[2:], 16))] |= tokens(d)


def written_of(lemma, pos, han):
    """The dictionary form in Hanja, as gloss.tsv writes it: 먹다 食다, 가득하다 滿하다, 하늘 天."""
    if pos in ("verb", "adj"):
        return han + ("하다" if lemma.endswith("하다") and len(lemma) > 2 else "다")
    return han


def skeleton(s):
    """Sounds of a word with its vowels merged, to tell a word's descendant from a synonym."""
    out = []
    for c in linear(s):
        o = ord(c)
        if o in V:
            if not out or out[-1] != "V":
                out.append("V")
        else:
            out.append({"\u1101": "\u1100", "\u1104": "\u1103", "\u1108": "\u1107", "\u110A": "\u1109", "\u110D": "\u110C",
                        "\u1110": "\u1103", "\u110E": "\u110C", "\u1140": "\u1109", "\u110F": "\u1100", "\u1111": "\u1107",
                        "\u112B": "\u1107"}.get(c, c))
    return "".join(out)


def descends(old, modern):
    """Whether a modern word in the editors' parentheses continues the old gloss (긔려기 기러기), or only
    translates it (할미 → 어머니, 도로래 → 땅강아지)."""
    a, b = skeleton(old), skeleton(modern)
    if not a or not b or (a[0] != b[0] and {a[0], b[0]} != {"\u1102", "V"}):
        return False
    return difflib.SequenceMatcher(None, a, b).ratio() >= 0.7


def lemma_candidates(a, m):
    """Modern lemmas an old gloss can be: (lemma, pos, how)."""
    out = []
    seen = set()

    def add(lemma, pos, how):
        if m.kpos.get(lemma) and pos not in m.kpos[lemma]:
            return
        if pos in ("verb", "adj") and not lemma.endswith("다"):
            return
        if lemma in m.modern and (lemma, pos) not in seen:
            seen.add((lemma, pos))
            out.append((lemma, pos, how))

    def try_word(text, how):
        for pos in m.pos.get(text, ()):
            if pos not in ("verb", "adj"):
                add(text, pos, how)
        for stem in verb_stems(text):
            for pos in m.pos.get(stem, ()):
                if pos in ("verb", "adj"):
                    add(stem, pos, how)

    old = a.old
    plain_reading = readings(old)[:1]
    if a.modern and all(0xAC00 <= ord(c) <= 0xD7A3 for c in a.modern) and descends(old, a.modern) and not (plain_reading and plain_reading[0][0] in m.modern):
        try_word(a.modern, "wikisource")
    for lem in m.old.get(old, ()):
        for pos in m.pos.get(lem, ()):
            if pos not in ("verb", "adj"):
                add(lem, pos, "wiktionary")
    # An old verb in the Middle Korean list: 머글 for 먹다, by its sounds.
    for lem in old_verb_stems(old, m):
        for pos in m.pos.get(lem, ()):
            if pos in ("verb", "adj"):
                add(lem, pos, "wiktionary")
    # 이실 for 있다, 있을 in the era's spelling.
    for lem, pos in IRREGULAR.get(old, ()):
        add(lem, pos, "known")
    for text, _ in readings(old):
        try_word(text, "fold")
    # The genitive ㅅ of a gloss: 가짓 지, 도릿 도.
    if old.endswith("\u11BA") and syllables(old[:-1]):
        for text, _ in readings(old[:-1]):
            for pos in m.pos.get(text, ()):
                if pos == "noun":
                    add(text, pos, "fold")
    return out


CODA_ONSET = {0x11A8: "\u1100", 0x11AB: "\u1102", 0x11AE: "\u1103", 0x11AF: "\u1105", 0x11B7: "\u1106", 0x11B8: "\u1107",
              0x11BA: "\u1109", 0x11BC: "", 0x11F0: "", 0x11BD: "\u110C", 0x11BE: "\u110E", 0x11BF: "\u110F",
              0x11C0: "\u1110", 0x11C1: "\u1111", 0x11C2: "\u1112", 0x11EB: "\u1140", 0x11B0: "\u1105\u1100",
              0x11B1: "\u1105\u1106", 0x11B2: "\u1105\u1107", 0x11B3: "\u1105\u1109", 0x11B9: "\u1107\u1109",
              0x11AA: "\u1100\u1109", 0x11AC: "\u1102\u110C", 0x11AD: "\u1102\u1112", 0x11B6: "\u1105\u1112"}


IRREGULAR = {norm("이실"): [("있다", "adj")], norm("이슬"): [("있다", "adj")],
             norm("버힐"): [("베다", "verb")], norm("베힐"): [("베다", "verb")]}


def linear(s):
    """Sounds without syllable boundaries: 머글 and 먹 share a prefix. Codas become onsets; the ㅇ onset goes."""
    out = []
    for c in unicodedata.normalize("NFD", s):
        o = ord(c)
        if o in T:
            out.append(CODA_ONSET.get(o, c))
        elif c != "\u110B":
            out.append(c)
    return "".join(out)


_old_verbs = None


def old_verb_stems(old, m):
    """Modern descendants of Middle Korean verbs whose stem, plus an adnominal ending, spells this gloss."""
    global _old_verbs
    if _old_verbs is None:
        _old_verbs = collections.defaultdict(set)
        for w, lems in m.old.items():
            if w.endswith("다") and len(w) > 1:
                _old_verbs[linear(w[:-1])] |= lems
    lin = linear(old)
    out = set()
    for ending in ("\u1105", "\u119E\u1105", "\u1173\u1105", "\u1102", "\u119E\u1102", "\u1173\u1102"):
        if lin.endswith(ending):
            out |= _old_verbs.get(lin[: -len(ending)], set())
    return out


def is_sino_gloss(lemma, char, m):
    """The gloss is itself Sino-Korean: 臣 신하, 讓 ᄉᆡ양, 穀 곡식 (a Hanja word for it holds the character),
    or starts with the character's own reading: 甚 심히, 盞 잔."""
    base = lemma.removesuffix("하다") or lemma
    if any(char in h for h in m.sino.get(base, set()) | m.sino.get(lemma, set())):
        return True
    return any(lemma.startswith(r) for r in m.char_readings.get(char, ()))


STRONG_PAIR = {"corpus", "editorial", "community", "suggested", "japanese"}


def meaning_evidence(lemma, pos, written, chars, m, old=""):
    """What says this lemma means these characters: another dictionary's pair, or an English sense in common."""
    rows = {src for w, p, _, src in m.lemma_rows.get(lemma, ()) if w == written and p == pos}
    ev = sorted(rows)
    defs = set().union(*(m.char_def.get(c, set()) for c in chars))
    shared = m.senses.get((lemma, pos), set()) & defs
    if shared:
        ev.append("en:" + ",".join(sorted(shared)[:3]))
    shared = m.old_senses.get(old, set()) & defs
    if shared and lemma in m.old.get(old, ()):
        ev.append("okm:" + ",".join(sorted(shared)[:3]))
    return ev


def strong(ev):
    return any(e.split(":")[0] in STRONG_PAIR or e.startswith("en:") for e in ev)


# ---- Aligned text ---------------------------------------------------------

def clean_wiki(text):
    text = plain(text)
    text = re.sub(r"<noinclude>.*?</noinclude>", "\n", text, flags=re.S)
    text = re.sub(r"\{\{ruby\|([^|{}]*)\|([^|{}]*)\}\}", r"\1\2", text)
    text = re.sub(r"\{\{더 ?작게\|([^{}]*)\}\}", r"\1", text)
    text = re.sub(r"\{\{작게\|([^{}]*)\}\}", r"\1", text)
    text = re.sub(r"\{\{[^{}]*\}\}", "", text)
    text = re.sub(r"<br\s*/?>", "\n", text)
    text = re.sub(r"<sub>.*?</sub>", "", text, flags=re.S)
    text = re.sub(r"<[^>]+>", "", text)
    return text


def split_blocks(text):
    """Lines of a book in order, each marked translation when it starts with `:` or a full-width space."""
    out = []
    for raw in text.split("\n"):
        line = raw.rstrip()
        if not line.strip() or line.startswith(("=", "{|", "|")):
            continue
        out.append(line)
    return out


def han_share(line):
    """Han characters against the Hangul left once each Han character's own reading is removed."""
    t = norm(line)
    no_ruby = re.sub(rf"([{HAN}])[\u1100-\u1112\u1121-\u1140\u114C\u1159][\u1161-\u1175\u119E\u11A1]+[\u11A8-\u11FF]*", r"\1", t)
    han = len(re.findall(f"[{HAN}]", no_ruby))
    hangul = len(re.findall("[\u1161-\u1175\u119E\u11A1]", no_ruby))
    return han / (han + hangul) if han + hangul else 0, han + hangul


class Passage:
    __slots__ = ("book", "year", "loc", "src", "kor", "chars", "kor_han")

    def __init__(self, book, year, loc, src, kor):
        self.book, self.year, self.loc = book, year, loc
        self.src, self.kor = src, kor
        self.kor_han = set(re.findall(f"[{HAN}]", kor))
        self.chars = set(re.findall(f"[{HAN}]", src)) - self.kor_han


def passages(pages):
    """(Chinese, 諺解) pairs: a line that is mostly Han characters followed by one that is mostly Hangul."""
    out = []
    grouped = collections.defaultdict(list)
    for p in pages:
        t = re.sub(r"^(페이지|Page):", "", p["title"])
        for key, (book, year) in ALIGNED_BOOKS.items():
            if t.startswith(key):
                grouped[(book, year)].append((page_number(t), t, p))
                break
    for (book, year), plist in grouped.items():
        plist.sort(key=lambda x: (x[1].rsplit("/", 1)[0], x[0]))
        for _, title, page in plist:
            if book == "詩經諺解" and "제1책" in title and page_number(title) < 45:
                continue  # 物名, read as a primer
            loc = re.sub(r"\.(djvu|pdf)", "", title)
            lines = split_blocks(clean_wiki(page["text"]))
            prev = None
            for line in lines:
                body = line.lstrip(":　 ○").strip()
                share, size = han_share(body)
                if size < 4:
                    prev = None
                    continue
                if share >= 0.55 and not line.startswith(":"):
                    prev = body if prev is None else prev + body
                elif prev is not None and (share < 0.45 or line.startswith(":")):
                    out.append(Passage(book, year, loc, prev, norm(body)))
                    prev = None
                else:
                    prev = None
    return out


VOWEL_KEY = {"\u119E": "A", "\u1161": "A", "\u1163": "A", "\u1173": "A", "\u1165": "E", "\u1167": "E", "\u1169": "O", "\u116D": "O",
             "\u116E": "O", "\u1172": "O", "\u1175": "I", "\u1174": "I", "\u1162": "Y", "\u1164": "Y", "\u1166": "Y", "\u1168": "Y",
             "\u11A1": "Y", "\u116A": "W", "\u116F": "W", "\u1171": "U", "\u116C": "U", "\u116B": "W", "\u1170": "W"}
CONS_KEY = {"\u1101": "\u1100", "\u1104": "\u1103", "\u1108": "\u1107", "\u110A": "\u1109", "\u110D": "\u110C", "\u1110": "\u110E",
            "\u1103": "\u110C", "\u1140": "\u1109", "\u1121": "\u1109", "\u112D": "\u1100", "\u111E": "\u1100", "\u112F": "\u110C",
            "\u1120": "\u110C", "\u1132": "\u1107", "\u1136": "\u110C", "\u1127": "\u110C", "\u1111": "\u1107", "\u112B": "\u1107",
            "\u110F": "\u1100", "\u114C": ""}


def key(s):
    """A lossy key under which an old spelling and its modern word agree (ᄉᆞ랑 사랑, 업ᄉᆞᆫ 없, 머글 먹)."""
    out = []
    for c in linear(s):
        if c in VOWEL_KEY:
            out.append(VOWEL_KEY[c])
        else:
            out.append(CONS_KEY.get(c, c))
    return "".join(out)


def aligned_counts(ps, m):
    """For every native lemma and character: the passages that have the character in the Chinese and the
    lemma's stem in the Korean. A lemma's stem shorter than three sounds is too easily found by chance."""
    by_key = collections.defaultdict(set)
    for lemma in m.pos:
        if lemma not in m.modern:
            continue
        for pos in m.pos[lemma]:
            stem = lemma[:-1] if pos in ("verb", "adj") and lemma.endswith("다") else lemma
            k = key(stem)
            if len(k) >= 3:
                by_key[k].add(lemma)
    lengths = sorted({len(k) for k in by_key})
    d_char, d_lemma, joint = collections.Counter(), collections.Counter(), collections.defaultdict(list)
    for i, p in enumerate(ps):
        for c in p.chars:
            d_char[c] += 1
        found = set()
        for run in re.findall(r"[\u1100-\u11FF]+", p.kor):
            syls = syllables(run)
            if not syls:
                continue
            k = key("".join(syls[:6]))
            for n in lengths:
                if n > len(k):
                    break
                if k[:n] in by_key:
                    # 아니ᄒᆞ고 and 그러ᄒᆞ뇨 are 아니 and 그러하다, no forms of 아니다 and 그러다.
                    if k[n:].startswith("\u1112"):
                        found |= {w for w in by_key[k[:n]] if not w.endswith("다")}
                    else:
                        found |= by_key[k[:n]]
        for lemma in found:
            d_lemma[lemma] += 1
            for c in p.chars:
                joint[(lemma, c)].append(i)
    return d_char, d_lemma, joint


def quote(text, needle=None, width=14):
    text = re.sub(r"\s+", " ", text)
    if needle and needle in text:
        i = text.index(needle)
        return text[max(0, i - width // 2): i + width // 2]
    return text[:width]


def quote_word(kor, lemma, width=24):
    """A piece of the Korean around the word, cut between syllables."""
    for k in (key(lemma.removesuffix("다")), key(lemma)):
        for m in re.finditer(r"[\u1100-\u11FF]+", kor):
            syls = syllables(m.group())
            if not syls or not key("".join(syls[:6])).startswith(k):
                continue
            left, right = max(0, m.start() - 8), min(len(kor), m.start() + width)
            while left > 0 and (ord(kor[left]) in V or ord(kor[left]) in T):
                left -= 1
            while right < len(kor) and (ord(kor[right]) in V or ord(kor[right]) in T):
                right += 1
            return re.sub(r"\s+", " ", kor[left:right]).strip()
    return quote(kor, None, width)


def strip_readings(src):
    return re.sub(rf"([{HAN}])[\u1100-\u1112\u1121-\u1140\u114C\u1159][\u1161-\u1175\u119E\u11A1]+[\u11A8-\u11FF]*", r"\1", norm(src))


def aligned_rows(ps, m, audit=None):
    """Rows from aligned text: at least three passages in at least two books, the pair strongly associated
    (Dice) and among the best for both sides, and the meaning confirmed as for primers."""
    d_char, d_lemma, joint = aligned_counts(ps, m)
    scored = []
    for (lemma, c), idx in joint.items():
        if len(idx) < 3 or len({ps[i].book for i in idx}) < 2:
            continue
        dice = 2 * len(idx) / (d_char[c] + d_lemma[lemma])
        if dice >= 0.08 and len(lemma.removesuffix("다")) >= 2:
            scored.append((dice, lemma, c, idx))
    best_c, best_l = collections.defaultdict(list), collections.defaultdict(list)
    for dice, lemma, c, idx in scored:
        best_c[c].append((dice, lemma))
        best_l[lemma].append((dice, c))
    rows = {}
    for dice, lemma, c, idx in scored:
        if lemma not in [x[1] for x in sorted(best_c[c], reverse=True)[:3]] or dice < 0.6 * max(x[0] for x in best_l[lemma]):
            continue
        if is_sino_gloss(lemma, c, m) or word_is_sino(lemma, m):
            continue
        stem = lemma.removesuffix("다")
        # 눈알 reads as 눈 + 을, 만남 as 만나 + ㅁ: a shorter word already has this character.
        if any(m.pairs.get((stem[:n], c)) or m.pairs.get((stem[:n] + "다", c + "다")) for n in range(1, len(stem))):
            continue
        for pos in sorted(m.pos[lemma], key=lambda p: (p not in m.kpos.get(lemma, ()), POS_ORDER(p))):
            if pos not in ("noun", "verb", "adj", "adv", "num", "pron") or (pos in ("verb", "adj") and not lemma.endswith("다")):
                continue
            w = written_of(lemma, pos, c)
            ev = meaning_evidence(lemma, pos, w, c, m)
            if audit is not None:
                audit.append((lemma, w, pos, round(dice, 3), len(idx), ev))
            if not any(e.split(":")[0] in STRONG_PAIR or e == "gloss" or e.startswith("en:") for e in ev):
                continue
            # The shortest passage of each of two books, quoted.
            quoted, books = [], set()
            for i in sorted(idx, key=lambda i: len(ps[i].src) + len(ps[i].kor)):
                p = ps[i]
                if p.book in books:
                    continue
                books.add(p.book)
                quoted.append(f"{p.book} {p.year} ({p.loc}): {quote(strip_readings(p.src), c, 16)} → {quote_word(p.kor, lemma)}")
                if len(quoted) == 2:
                    break
            rows[(lemma, w, pos)] = (dice, len(idx), len({ps[i].book for i in idx}), quoted, ev)
            break
    return rows


# ---- Selection ------------------------------------------------------------

def POS_ORDER(p):
    return ["noun", "verb", "adj", "adv", "pron", "num", "det", "bound", "interj"].index(p) if p in ("noun", "verb", "adj", "adv", "pron", "num", "det", "bound", "interj") else 9


def word_is_sino(lemma, m):
    """Wiktionary spells the word in Hanja (군사 軍士, 보배 寶貝), so it needs no 訓讀."""
    if lemma in m.hand_lemmas:
        return False
    base = lemma.removesuffix("하다") if lemma.endswith("하다") and len(lemma) > 2 else lemma
    if base in m.origin_hanja:
        return True
    return bool(m.wikt_hanja.get(base)) and base not in m.wikt_native


def reading_fits(att, m):
    """Whether the reading the book prints can be the character's: if not, the transcription has the wrong
    character (倭語類解 `掬 ᄭᅵᆯ 옹` for 擁)."""
    if not att.eum or not syllables(att.eum) or len(att.char) != 1 or att.char not in m.char_readings:
        return True
    merged = str.maketrans({"\u1103": "\u110C", "\u1105": "\u1102", "\u1140": "\u110C"})  # 뎨 제, 령 녕, ᅀᅵᆷ 임
    a = skeleton(att.eum).translate(merged).removeprefix("\u1102")
    return any(difflib.SequenceMatcher(None, a, skeleton(r).translate(merged).removeprefix("\u1102")).ratio() > 0.5 for r in m.char_readings[att.char])


def reading_starts_alike(char, eum, m):
    """A word of a cell can be the reading only if it starts as the character's reading does: 글 ᄒᆞᆫ ᄆᆞᄎᆞᆷ 편 is one
    phrase, whatever ᄒᆞᆫ and 편 share."""
    if not reading_fits(Att(char, "", eum, "", "", ""), m):
        return False
    merged = str.maketrans({"\u1103": "\u110C", "\u1105": "\u1102", "\u1140": "\u110C"})
    onset = skeleton(eum).translate(merged).removeprefix("\u1102")[:1]
    return any(skeleton(r).translate(merged).removeprefix("\u1102")[:1] == onset for r in m.char_readings[char])


def primer_rows(atts, m, rejects=None):
    """Rows from primers. Evidence for the meaning is a hand-made or machine pair from an independent method, or
    an English sense shared with the character. libhangul's gloss, being the same kind of reconstruction, counts
    only where the form leaves one reading: a noun, or a verb with no other lemma to be."""
    rows = collections.defaultdict(list)
    for x in atts:
        if not syllables(x.old) or (x.eum and x.old == x.eum):
            continue
        if len(x.char) > 1 and (len(x.char) != 2 or x.char[-1] in "的了著來去"):
            continue
        if not reading_fits(x, m):
            if rejects is not None:
                rejects.append((x, "reading does not fit the character"))
            continue
        res = []
        for lem, pos, how in lemma_candidates(x, m):
            if any(is_sino_gloss(lem, c, m) for c in x.char) or word_is_sino(lem, m):
                continue
            if len(x.char) > 1 and pos != "noun":
                continue
            w = written_of(lem, pos, x.char)
            res.append((lem, pos, w, how, meaning_evidence(lem, pos, w, x.char, m, x.old)))
        strongs = [r for r in res if strong(r[4])]
        paired = [r for r in strongs if any(e in STRONG_PAIR for e in r[4])]
        if len(paired) >= 1 and len(strongs) > len(paired):
            strongs = paired
        # 날 生 and 덜 減 are adnominals of 나다 and 덜다, whatever a noun 날 or an adverb 덜 shares in English.
        if any(r[1] in ("verb", "adj") for r in strongs):
            strongs = [r for r in strongs if r[1] in ("verb", "adj") or any(e in STRONG_PAIR for e in r[4])]
        if strongs:
            for lem, pos, w, how, ev in strongs:
                rows[(lem, w, pos)].append((x, how, ev, False))
        else:
            weak = [r for r in res if any(e == "gloss" for e in r[4])]
            if len(weak) == 1 and len(x.char) == 1:
                lem, pos, w, how, ev = weak[0]
                verbs = [r for r in res if r[1] in ("verb", "adj")]
                if pos in ("verb", "adj") and (len(verbs) > 1 or x.old.endswith("\u11AB")):
                    if rejects is not None:
                        rejects.append((x, "ambiguous verb"))
                    continue
                rows[(lem, w, pos)].append((x, how, ev, True))
            elif rejects is not None:
                rejects.append((x, "no meaning evidence" if res else "no modern lemma"))
    # A row resting on the gloss alone needs a form that is not an accident: two books for a short word or one
    # ending like an adnominal, an open syllable never for a verb.
    kept = {}
    for k, v in rows.items():
        lem, w, pos = k
        books = {y[0].book for y in v}
        if all(y[3] for y in v):
            old = v[0][0].old
            short = len(lem) == 1 or (pos in ("verb", "adj") and len(lem) == 2)
            if (short or old.endswith("\u11AF")) and len(books) < 2:
                if rejects is not None:
                    rejects.append((v[0][0], "gloss alone, one book"))
                continue
        kept[k] = v
    return kept


# Attested, but the modern word has moved away from the book's sense or the mapping is wrong; found by hand.
EXCLUDE = {
    ("끼니", "時"),  # ᄞᅵ니 was a time of day; 끼니 is a meal
    ("물개", "獺"),  # 물ᄀᆡ was the otter; 물개 is the fur seal
    ("틀다", "機다"),  # 틀 is the loom, a noun
    ("보름", "壁"),  # ᄇᆞᄅᆞᆷ 壁 is the wall of 바람벽
    ("옳다", "右다"),  # 올ᄒᆞ다 was also the right hand; 옳다 no longer is
    ("글귀", "句"),  # the transcribed gloss 굿굿 is no form of 글귀
    ("며느리", "姑"),  # passages about 舅姑 and the daughter-in-law
    ("붙들다", "抱다"),  # the page's 抱 붇들 부 is a slip for 扶
    ("이내", "嵐"),  # the book's 연애 is 煙靄, and 이내 is a different word
    ("열", "行"),  # 녈 is the old verb 녀다, go; 열 here is the Sino-Korean row
    ("꼴", "菅"),  # 골 is the rush; 꼴 is a shape, or fodder from another word
    ("덜", "減"),  # the book's 덜 is the adnominal of 덜다, which has its own row
    ("가름", "代"),  # ᄀᆞᄅᆞᆷ 代 is 갈음, a substitute; 가름 is a division
    ("걷다", "斂다"),  # 거둘 is 거두다, which has its own row
    ("겹다", "冬다"),  # 겨을 is 겨울
    ("나다", "經다"), ("날다", "經다"), ("놀다", "經다"),  # ᄂᆞᆯ 經 is 날, the warp, a noun
    ("낱", "晌"),  # 낫 晌 is 낮
    ("들다", "穿다"),  # 들올 is the old verb for 뚫다
    ("마지막", "卒"),  # ᄆᆞᄎᆞᆷ is 마침
    ("바리다", "抛다"),  # ᄇᆞ릴 is 버리다
    ("배", "棹"),  # ᄇᆡᆺ is a genitive with its noun lost: 棹 is an oar
    ("복", "祚"), ("복", "祉"),  # 福; the fish 복 stays
    ("창", "牖"),  # 窓; the weapon 창 stays
}
# Headwords Wiktionary or FusionIME lists that are no modern form of the old word, or are Sino-Korean in origin.
EXCLUDE_LEMMA = {
    "가라치", "가이", "곰소", "구무", "기례기", "나베", "나조", "너름", "닙다", "두리", "부치", "우", "퐅", "신다리",  # old or dialect forms
    "비화", "인", "동관", "동정", "돈피", "재주", "보배",  # 琵琶, 印, 同官, 同正, 獤皮, 才操, 寶貝
    "베이다",  # 베힐 is 베다
    "도로래",  # now 땅강아지
    "괴",  # survives only as the old word for a cat
    "소곰", "호매",  # old or dialect forms of 소금, 호미
    "되놈",  # the book's 되 is only the first part of this word
    "아람",  # ᄋᆞᄅᆞᆷ 私 survives in 아람치; 아람 alone is a ripe chestnut
    "공", "노적", "노하다", "심하다", "약하다", "원쑤", "죄", "평지",  # 功, 露積, 怒, 甚, 弱, 怨讐, 罪, 平地
}


def citation(att):
    entry = " ".join(x for x in (att.char, att.old, att.eum) if x)
    edition = {"訓蒙字會": "1527", "光州千字文": "1575", "新增類合": "1576", "石峯千字文": "1583", "類合": "1913", "倭語類解": "18世紀", "譯語類解": "1690", "詩經諺解 物名": "1820本"}[att.book]
    return f"{att.book} {edition}, {att.loc}: {entry}"


def build(audit_dir=None):
    pages = extract_pages()
    m = Modern()
    atts = primers(pages) + namu_primers(m)
    rejects = []
    prim = primer_rows(atts, m, rejects)
    ps = passages(pages)
    audit_aligned = []
    alig = aligned_rows(ps, m, audit_aligned)
    print(f"primer attestations {len(atts)}, candidate rows {len(prim)}; passages {len(ps)}, aligned rows {len(alig)}", file=sys.stderr)
    hand = {"corpus", "editorial", "community"}
    out, stats = [], collections.Counter()
    by_lemma = collections.defaultdict(list)
    written_seen = set()
    for (lem, w, pos), v in sorted(prim.items(), key=lambda kv: POS_ORDER(kv[0][2])):
        if (lem, w) in EXCLUDE or lem in EXCLUDE_LEMMA or (lem, w) in written_seen:
            continue
        written_seen.add((lem, w))
        if m.pairs.get((lem, w), set()) & hand:
            stats["already hand-made"] += 1
            continue
        books = {y[0].book for y in v}
        seen, cites = set(), []
        for y in sorted(v, key=lambda y: (y[0].year, y[0].book, NAMU_TERMS in y[0].loc)):
            transcription = (y[0].book, NAMU_TERMS in y[0].loc)
            if transcription not in seen:
                seen.add(transcription)
                cites.append(citation(y[0]))
        evidence = {e.split(":")[0] for y in v for e in y[2]}
        weight = (len(books), len(evidence), m.char_freq[w.removesuffix("다")[:1]] if len(w) else 0)
        by_lemma[lem].append((0, weight, lem, w, pos, "primer: " + "; ".join(cites)))
    for (lem, w, pos), (dice, n, nbooks, quoted, ev) in alig.items():
        if (lem, w) in EXCLUDE or lem in EXCLUDE_LEMMA or (lem, w) in written_seen:
            continue
        written_seen.add((lem, w))
        if m.pairs.get((lem, w), set()) & hand:
            stats["already hand-made"] += 1
            continue
        if (lem, w, pos) in prim:
            stats["aligned, also in a primer"] += 1
            continue
        by_lemma[lem].append((1, (nbooks, n, 0), lem, w, pos, f"aligned ({n} passages, {nbooks} books): " + "; ".join(quoted)))
    for lem in sorted(by_lemma):
        items = sorted(by_lemma[lem], key=lambda r: (r[0], tuple(-x for x in r[1])))
        if len(items) > MAX_PER_LEMMA:
            stats["over the per-word limit"] += len(items) - MAX_PER_LEMMA
        for tier, _, lemma, w, pos, note in items[:MAX_PER_LEMMA]:
            stats["primer" if tier == 0 else "aligned"] += 1
            out.append((lemma, w, pos, class_of(lemma, pos, m), note))
    path = DICT / "eonhae.tsv"
    with path.open("w", encoding="utf-8") as f:
        f.write(HEADER)
        f.write("lemma\twritten\tpos\tclass\tsource\tcount\tnote\n")
        for lemma, w, pos, cls, note in out:
            f.write(f"{lemma}\t{w}\t{pos}\t{cls}\teonhae\t\t{note}\n")
    print(dict(stats), file=sys.stderr)
    if audit_dir:
        d = Path(audit_dir)
        d.mkdir(parents=True, exist_ok=True)
        with (d / "rejected.tsv").open("w", encoding="utf-8") as f:
            for x, why in rejects:
                f.write(f"{x.char}\t{x.old}\t{x.book}\t{x.loc}\t{why}\n")
        with (d / "aligned-candidates.tsv").open("w", encoding="utf-8") as f:
            for row in audit_aligned:
                f.write("\t".join(str(c) for c in row) + "\n")
    return out


MAX_PER_LEMMA = 8


def class_of(lemma, pos, m):
    """The inflection class another dictionary gives the word; blank lets the engine guess."""
    if pos not in ("verb", "adj"):
        return ""
    for _, p, cls, _ in m.lemma_rows.get(lemma, ()):
        if p == pos and cls:
            return cls
    return ""


HEADER = """# 訓讀 readings attested in historical Korean books. Each note says where to check the row.
# primer: a character primer glosses the character with the old form of this word (book, year, page, entry).
# aligned: 諺解 passages print the Chinese next to the Korean; the note gives the count and quotes the shortest.
# Text from the transcriptions on Korean Wikisource, dump of 2026-11-01, and on 나무위키, saved 2026-11-04.
# Built by tools/build_eonhae.py. Ranks below the hand-made tiers.
# CC BY-SA 4.0 (Wikisource, Wiktionary); citations of 나무위키 say CC BY-NC-SA 2.0 KR, an anonymously edited
# wiki's transcription that names no copy of the book and has not been read against a scan. See dict/README.md.
"""


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--audit", help="write rejected candidates to this directory (keep it outside the repository)")
    args = ap.parse_args()
    build(args.audit)


if __name__ == "__main__":
    main()
