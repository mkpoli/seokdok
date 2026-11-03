"""Build the shipped dictionaries in dict/ from the sources in data/.

Usage: uv run tools/build_dict.py

Inputs (see data/sources/README.md for provenance and licences):
  data/private/sheet/gid0.csv          標準國語釋讀辭典 (community sheet export)
  data/private/telegram.jsonl          釋讀會 chat corpus (counts only)
  data/sources/libhangul/hanja.txt     Sino-Korean words and 訓音 glosses (BSD-3)
  data/sources/unihan/unihan_kHangul_kKorean.tsv  readings (Unicode licence)
  data/sources/kaikki-korean/*.jsonl   Wiktionary headwords, used only to check
                                       that a gloss-derived lemma exists

Outputs:
  dict/community.tsv  訓讀 entries from the community dictionary
  dict/gloss.tsv      訓讀 entries derived from single-character 訓音 glosses
  dict/sino.tsv       音讀 words with corpus counts
"""
import collections
import csv
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DICT = ROOT / "dict"
SRC = ROOT / "data" / "sources"
PRIV = ROOT / "data" / "private"

HANJA = r"[㐀-䶿一-鿿豈-﫿\U00020000-\U0003134f]"
SBASE = 0xAC00


def syl(c):
    return 0xAC00 <= ord(c) <= 0xD7A3


def parts(c):
    i = ord(c) - SBASE
    return i // 588, (i % 588) // 28, i % 28


def make(l, v, t):
    return chr(SBASE + (l * 21 + v) * 28 + t)


T_RIEUL, T_BIEUP, T_SIOS, T_NIEUN = 8, 17, 19, 4


def corpus_texts():
    p = PRIV / "telegram.jsonl"
    if not p.exists():
        return []
    return [json.loads(l)["text"] for l in p.open(encoding="utf-8")]


def lexicon():
    """Headword → set of POS from Wiktionary (existence check only)."""
    lex = collections.defaultdict(set)
    for p in (SRC / "kaikki-korean").glob("*.jsonl"):
        with p.open(encoding="utf-8") as f:
            for line in f:
                d = json.loads(line)
                pos = d.get("pos")
                if pos in ("noun", "verb", "adj", "adv", "pron", "num"):
                    lex[d["word"]].add(pos)
    return lex


# ---------------------------------------------------------------- community

POS_KO = {"명사", "대명사", "수사", "조수사", "의존명사", "부사", "관형사", "감탄사", "동사", "형용사"}


def build_community():
    src = PRIV / "sheet" / "gid0.csv"
    rows = list(csv.reader(src.open(encoding="utf-8")))[1:]
    out = []
    skipped = []
    for r in rows:
        pos, lemma, written = r[0].strip(), r[1].strip(), r[2].strip()
        if pos not in POS_KO or not lemma or not written:
            skipped.append((lemma, written, "empty"))
            continue
        note = ""
        if "[" in written:
            note = written[written.index("[") :].strip("[] ")
            written = written[: written.index("[")]
        if "□" in written:
            skipped.append((lemma, written, "漢字未定"))
            continue
        for w in [x.strip() for x in written.split(",") if x.strip()]:
            # 複數登載 lists bare characters; a verb's character takes 다.
            if pos in ("동사", "형용사") and not w.endswith("다"):
                w += "다"
            # A noun variant written as "凍음" for 얼음 keeps its tail.
            out.append((lemma, w, pos, "", "community", "", note))
    return out, skipped


# -------------------------------------------------------------------- gloss

D_IRR = {"들을": "듣", "걸을": "걷", "물을": "묻", "실을": "싣", "깨달을": "깨닫", "일컬을": "일컫"}


def gloss_lemmas(word, lex):
    """Lemma candidates (lemma, pos) for one 訓 gloss word."""
    out = []
    if not word or not all(syl(c) for c in word):
        return out
    has = lambda w: w in lex
    vpos = lambda w: ("adj" if "adj" in lex.get(w, ()) and "verb" not in lex.get(w, ()) else "verb")
    last = word[-1]
    l, v, t = parts(last)
    if word in D_IRR:
        lem = D_IRR[word] + "다"
        return [(lem, "verb")]
    if word.endswith("을") and len(word) >= 2:
        stem = word[:-1]
        pl, pv, pt = parts(stem[-1])
        if pt == 0:
            # 나을 → 낫다 (ㅅ irregular)
            s_irr = stem[:-1] + make(pl, pv, T_SIOS)
            if has(s_irr + "다"):
                return [(s_irr + "다", vpos(s_irr + "다"))]
        lem = stem + "다"
        if has(lem) or len(stem) >= 1:
            return [(lem, vpos(lem))]
    if t == 0 and word.endswith("울") is False:
        pass
    if word.endswith("울") and len(word) >= 2:
        # 아름다울 → 아름답다 (ㅂ irregular)
        pl, pv, pt = parts(word[-2])
        if pt == 0:
            b = word[:-2] + make(pl, pv, T_BIEUP) + "다"
            if has(b) or len(word) >= 3:
                out.append((b, vpos(b)))
                return out
    if t == T_RIEUL:
        vowel_stem = word[:-1] + make(l, v, 0) + "다"
        l_stem = word + "다"
        cands = [w for w in (vowel_stem, l_stem) if has(w)]
        if not cands:
            # Unverified: multi-syllable ㄹ adnominals are mostly vowel stems.
            cands = [vowel_stem] if len(word) >= 2 else []
        out.extend((w, vpos(w)) for w in cands)
        if word in lex and lex[word] & {"noun"}:
            out.append((word, "noun"))
        return out
    if t == T_NIEUN and len(word) >= 1 and (word[:-1] + make(l, v, 0) + "다") in lex:
        lem = word[:-1] + make(l, v, 0) + "다"
        return [(lem, vpos(lem))]
    # Otherwise a noun (or other nominal).
    pos = "noun"
    if word in lex:
        ps = lex[word]
        pos = "noun" if "noun" in ps else ("adv" if "adv" in ps else ("pron" if "pron" in ps else "noun"))
    return [(word, pos)]


def build_gloss(lex, sino_readings):
    out = []
    seen = set()
    for line in (SRC / "libhangul" / "hanja.txt").open(encoding="utf-8"):
        if line.startswith("#"):
            continue
        f = line.rstrip("\n").split(":", 2)
        if len(f) < 3 or len(f[1]) != 1 or not f[2]:
            continue
        reading, han, gloss = f
        for g in gloss.split(","):
            g = g.strip()
            if not g.endswith(" " + reading) and not g.endswith(" " + reading.replace("ㄹ", "")):
                # Glosses end with the character's reading (하늘 천).
                m = re.match(r"^(.*)\s+(\S)$", g)
                if not m or not syl(m.group(2)):
                    continue
                word = m.group(1)
            else:
                word = g[: g.rindex(" ")]
            if " " in word or re.search(HANJA, word) or len(word) > 5:
                continue
            for lemma, pos in gloss_lemmas(word, lex):
                # A Sino-Korean gloss (비유하다, 향낭) is already converted by sound.
                base = lemma[:-2] if lemma.endswith("하다") else (lemma[:-1] if pos in ("verb", "adj") else lemma)
                if len(base) >= 2 and base in sino_readings:
                    continue
                written = han + ("다" if pos in ("verb", "adj") else "")
                key = (lemma, written)
                if key in seen:
                    continue
                seen.add(key)
                out.append((lemma, written, pos, "", "gloss", "", f"{g} ({han})"))
    return out


# --------------------------------------------------------------------- sino

def build_sino(texts):
    words = {}
    for line in (SRC / "libhangul" / "hanja.txt").open(encoding="utf-8"):
        if line.startswith("#"):
            continue
        f = line.rstrip("\n").split(":", 2)
        if len(f) < 2 or not f[0] or not f[1]:
            continue
        reading, han = f[0], f[1]
        if not all(syl(c) for c in reading) or len(reading) != len(han):
            continue
        words.setdefault((reading, han), 0)
    # Corpus counts: count every Hanja substring up to 6 characters.
    sub = collections.Counter()
    for t in texts:
        for run in re.findall(HANJA + "+", t):
            n = len(run)
            for i in range(n):
                for j in range(i + 1, min(n, i + 6) + 1):
                    sub[run[i:j]] += 1
    out = []
    for (reading, han) in words:
        out.append((reading, han, sub.get(han, 0), "lexicon"))
    out.sort(key=lambda x: (x[0], -x[2], x[1]))
    return out


def write(path, header, rows, preamble):
    with path.open("w", encoding="utf-8") as f:
        for p in preamble:
            f.write(f"# {p}\n")
        f.write("\t".join(header) + "\n")
        for r in rows:
            f.write("\t".join(str(x) for x in r) + "\n")
    print(f"{path.relative_to(ROOT)}: {len(rows)} rows", file=sys.stderr)


def main():
    DICT.mkdir(exist_ok=True)
    texts = corpus_texts()
    comm, skipped = build_community()
    write(DICT / "community.tsv", ["lemma", "written", "pos", "class", "source", "count", "note"], comm, [
        "標準國語釋讀辭典 (釋讀會 community dictionary), Google Sheets export.",
        "https://docs.google.com/spreadsheets/d/1J7bPuQ4buOM17hvLR6UHLfcrERfYhaVVj1H-3szh7J0",
        "Compiled by 釋讀會 members from 2022-06; no licence stated. Redistribution needs the editors' consent.",
    ])
    for s in skipped:
        print("community skipped:", *s, file=sys.stderr)
    lex = lexicon()
    for r in comm:
        lex[r[0]].add("verb" if r[2] == "동사" else "adj" if r[2] == "형용사" else "noun")
    sino = build_sino(texts)
    sino_readings = {r[0] for r in sino}
    gl = build_gloss(lex, sino_readings)
    write(DICT / "gloss.tsv", ["lemma", "written", "pos", "class", "source", "count", "note"], gl, [
        "訓讀 candidates derived from single-character 訓音 glosses in libhangul hanja.txt",
        "(BSD-3-Clause, Copyright (c) 2005,2006 Choe Hwanjin; see dict/README.md).",
        "Lemmas reconstructed from adnominal glosses (읽을 → 읽다); checked against Wiktionary",
        "headwords where available. Unchecked guesses are kept: this tier ranks last.",
    ])
    write(DICT / "sino.tsv", ["reading", "hanja", "count", "source"], sino, [
        "Sino-Korean words and characters from libhangul hanja.txt (BSD-3-Clause,",
        "Copyright (c) 2005,2006 Choe Hwanjin; see dict/README.md).",
        "count: occurrences in the 釋讀會 chat corpus (Telegram export to 2026-10-29).",
    ])


if __name__ == "__main__":
    main()
