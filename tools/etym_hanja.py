"""Hanja spellings of Korean words from English Wiktionary that the other
音讀 dictionaries lack, above all words that are Sino-Korean only by origin:
대충 ← 大總 (nativised from 대총), 재미 ← 滋味, 김치 ← 沉菜 (coined in Korea
from Chinese morphemes).

Usage: uv run tools/etym_hanja.py
Input: data/sources/kaikki-korean/kaikki.org-dictionary-Korean.jsonl
       (wiktextract, enwiktionary dump 2026-10-01; CC BY-SA 4.0 / GFDL)
Output: dict/etym.tsv in the 音讀 format, shipped under CC BY-SA 4.0.

Taken, per entry:
- `ko-etym-nativised` (the argument written in Hanja), e.g. 대충, 재미, when
  its Sino-Korean word starts like the headword (the template also appears in
  comparisons inside other etymologies);
- `der` from Chinese with `ltc-l` morphemes, one per syllable, e.g. 김치;
- a Hanja form (tag `hanja`) as long as the word, e.g. 대충 大總.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "data" / "sources" / "kaikki-korean" / "kaikki.org-dictionary-Korean.jsonl"
DICT = ROOT / "dict"
HAN = re.compile(r"^[㐀-䶿一-鿿豈-﫿\U00020000-\U0003ffff]+$")
HANGUL = re.compile(r"^[가-힣]+$")
POS = {"noun", "adv", "intj", "num", "det", "pron", "adj", "verb"}


def existing():
    have = set()
    for name in ("fusion.tsv", "sino.tsv"):
        for line in (DICT / name).open(encoding="utf-8"):
            c = line.rstrip("\n").split("\t")
            if len(c) > 1 and not line.startswith("#"):
                have.add((c[0], c[1]))
    return have


def onset(syllable):
    o = (ord(syllable) - 0xAC00) // 588
    # ㄹ ㄴ ㅇ alternate at the start of a word (령감 → 넝감, 래일 → 내일).
    return {5: 2, 11: 2}.get(o, o)


def candidates(d):
    word = d["word"]
    out = {}
    temps = d.get("etymology_templates") or []
    for t in temps:
        if t.get("name") == "ko-etym-nativised":
            args = t.get("args", {}).values()
            sino = [v for v in args if HANGUL.match(v)]
            # The template also appears in comparisons ("cf. 霹靂 → 벼락"); keep
            # it only when its Sino-Korean word starts like the headword.
            if not sino or onset(sino[0][0]) != onset(word[0]) or len(sino[0]) != len(word):
                continue
            for v in args:
                if HAN.match(v):
                    out.setdefault(v, "nativised")
    if any(t.get("name") == "der" and t.get("args", {}).get("2") == "zh" for t in temps):
        parts = [t["args"].get("1", "") for t in temps if t.get("name") == "ltc-l"]
        joined = "".join(parts)
        if parts and HAN.match(joined) and len(joined) == len(word):
            out.setdefault(joined, "coined")
    for f in d.get("forms") or []:
        h = f.get("form", "")
        if "hanja" in (f.get("tags") or []) and HAN.match(h) and len(h) == len(word):
            out.setdefault(h, "form")
    return out


def main():
    have = existing()
    rows = {}
    for line in SRC.open(encoding="utf-8"):
        d = json.loads(line)
        word = d.get("word", "")
        if d.get("pos") not in POS or not HANGUL.match(word):
            continue
        # Verbs and adjectives: the Hanja stem of a 하다 word is already a noun entry.
        if d.get("pos") in ("verb", "adj"):
            continue
        for h, kind in candidates(d).items():
            if (word, h) not in have:
                rows.setdefault((word, h), kind)
    order = {"nativised": 0, "coined": 1, "form": 2}
    with (DICT / "etym.tsv").open("w", encoding="utf-8") as f:
        f.write("# Hanja spellings from English Wiktionary (via kaikki.org / wiktextract, enwiktionary dump\n")
        f.write("# 2026-10-01; Ylonen, \"Wiktextract\", LREC 2022) missing from fusion.tsv and sino.tsv,\n")
        f.write("# including words Sino-Korean only by origin (대충 大總, 재미 滋味, 김치 沉菜).\n")
        f.write("# Licence: CC BY-SA 4.0 (https://en.wiktionary.org/wiki/Wiktionary:Copyrights). Built by\n")
        f.write("# tools/etym_hanja.py. source: nativised | coined | form (see the script).\n")
        f.write("reading\thanja\tcount\tsource\n")
        for (w, h), k in sorted(rows.items(), key=lambda r: (order[r[1]], r[0])):
            f.write(f"{w}\t{h}\t\t{k}\n")
    counts = {k: sum(1 for v in rows.values() if v == k) for k in order}
    print(len(rows), counts, file=sys.stderr)


if __name__ == "__main__":
    main()
