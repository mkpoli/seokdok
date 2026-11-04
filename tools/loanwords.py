"""Korean words borrowed from European languages, so the engine leaves them in
Hangul: 페이지, 파일, 메시지, 버전, 피자 stay as typed and do not become 頁 or 八日.

Usage: uv run tools/loanwords.py   (SEOKDOK_SOURCES=DIR to read a data/sources elsewhere)
Input: data/sources/kaikki-korean/kaikki.org-dictionary-Korean.jsonl
       (wiktextract, enwiktionary dump 2026-10-01; CC BY-SA 4.0 / GFDL)
Output: dict/loanwords.tsv in the 音讀 format (`reading hanja count source
        language`), with the Hangul word itself in the `hanja` column.

A headword is kept when every one of its entries is a loan:
- some entry has a `bor`, `bor+` or `lbor` template from a European language
  (or Turkish, Arabic, Hindi), and
- no entry has a Sino-Korean, native or nativised etymology template, and no
  entry borrows from Japanese or Chinese (사진, 가방 and 담배 are not here),
- it has two or more syllables, and
- the chat does not write it with Hanja: no `lexicon` row of sino.tsv has a
  chat count of ten or more (파일 is 八日 six times and stays a loanword), and
  no corpus, editorial or community 訓讀 entry covers it, and
- it is not a native verb or adjective with an ending (올리고 is 올리다 + 고,
  not the prefix oligo-).
Words the Wiktionary extract lacks but the chat and this site use as
loanwords are listed in EXTRA.
"""
import json
import os
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCES = Path(os.environ.get("SEOKDOK_SOURCES", ROOT / "data" / "sources"))
SRC = SOURCES / "kaikki-korean" / "kaikki.org-dictionary-Korean.jsonl"
DICT = ROOT / "dict"
OUT = DICT / "loanwords.tsv"
HANGUL = re.compile(r"^[가-힣]{2,}$")
LOAN = {"bor", "bor+", "lbor"}
LANGS = {"en", "fr", "de", "nl", "pt", "it", "es", "ru", "la", "el", "sv", "da", "no", "fi", "pl", "cs", "hu", "tr", "ar", "hi", "fa"}
NOT_LOAN = {"ko-etym-sino", "ko-etym-Sino", "ko-sino", "ko-etym-native", "ko-etym-nativised", "inh", "ko-etym-hanja"}
EAST_ASIA = {"ja", "zh", "ltc", "och", "cmn", "yue", "okm", "ko", "oko", "ain"}
# Endings that make a loanword look like a verb form.
ENDINGS = {"고", "면", "네", "요", "게", "서", "던", "자", "는", "은", "을", "도", "만", "기", "며", "나", "거나", "지만", "는데", "다가", "니까", "죠"}
# Loanwords that look like a verb form and are kept anyway (피자 is 피다 + 자 only on paper).
KEEP = {"피자"}
# A reading the chat writes with Hanja this many times keeps its Hanja (sino.tsv's counts).
MIN_CHAT = 10
# Loanwords the Wiktionary extract lacks.
EXTRA = {"빌드": "en"}


def chat_hanja():
    """Readings the chat writes with Hanja at least MIN_CHAT times (sino.tsv's own
    counts), and readings with a 訓讀 entry from the corpus, editors or community."""
    sino, native = {}, set()
    for line in (DICT / "sino.tsv").open(encoding="utf-8"):
        c = line.rstrip("\n").split("\t")
        if len(c) > 3 and not line.startswith("#") and c[3] == "lexicon" and c[2].isdigit() and int(c[2]) >= MIN_CHAT:
            sino[c[0]] = max(sino.get(c[0], 0), int(c[2]))
    for name in ("corpus.tsv", "editorial.tsv", "community.tsv"):
        for line in (DICT / name).open(encoding="utf-8"):
            c = line.rstrip("\n").split("\t")
            if len(c) > 1 and not line.startswith("#"):
                native.add(c[0])
    return sino, native


def verb_lemmas():
    """Dictionary forms of the native verbs and adjectives in the 訓讀 files."""
    out = set()
    for name in ("corpus.tsv", "editorial.tsv", "community.tsv", "suggested.tsv", "wikt-native.tsv", "gloss.tsv", "kun-ja.tsv", "kun-ja-weak.tsv"):
        for line in (DICT / name).open(encoding="utf-8"):
            c = line.rstrip("\n").split("\t")
            if len(c) > 2 and not line.startswith("#") and c[2] in ("verb", "adj", "동사", "형용사"):
                out.add(c[0])
    return out


def inflected(word, lemmas):
    """Whether the word is a native verb's stem with an ending (올리고 = 올리다 + 고)."""
    return word not in KEEP and any(word[:k] + "다" in lemmas and word[k:] in ENDINGS for k in range(1, len(word)))


def main():
    langs = {}
    bad = set()
    for line in SRC.open(encoding="utf-8"):
        d = json.loads(line)
        w = d.get("word", "")
        if not HANGUL.match(w):
            continue
        found = set()
        for t in d.get("etymology_templates") or []:
            name = t.get("name")
            lang = (t.get("args") or {}).get("2")
            if name in NOT_LOAN:
                bad.add(w)
            elif name in LOAN and lang in EAST_ASIA:
                bad.add(w)
            elif name in LOAN and lang in LANGS:
                found.add(lang)
        if found:
            langs.setdefault(w, set()).update(found)
    sino, native = chat_hanja()
    lemmas = verb_lemmas()
    rows = {w: sorted(l)[0] for w, l in langs.items() if w not in bad and w not in sino and w not in native and not inflected(w, lemmas)}
    for w in sorted(set(langs) - bad - set(rows)):
        why = "chat " + str(sino[w]) + "x" if w in sino else "a 訓讀 entry" if w in native else "a verb form"
        print(f"left to the converter: {w} ({why})")
    for w, l in EXTRA.items():
        rows.setdefault(w, l)
    with OUT.open("w", encoding="utf-8") as f:
        f.write(
            "# Korean words borrowed from European languages, kept in Hangul. From English Wiktionary\n"
            "# (via kaikki.org / wiktextract, enwiktionary dump 2026-10-01; Ylonen, \"Wiktextract\",\n"
            "# LREC 2022): headwords whose every entry is a `bor` from a European language. Licence:\n"
            "# CC BY-SA 4.0 (https://en.wiktionary.org/wiki/Wiktionary:Copyrights). Built by\n"
            "# tools/loanwords.py. The `hanja` column repeats the Hangul on purpose; the last column\n"
            "# is the source language (Wiktionary's code).\n"
            "reading\thanja\tcount\tsource\tlanguage\n"
        )
        for w in sorted(rows):
            f.write(f"{w}\t{w}\t\tloanword\t{rows[w]}\n")
    print(f"{len(rows)} loanwords -> {OUT}")


if __name__ == "__main__":
    main()
