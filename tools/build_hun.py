"""Build dict/hun.tsv: 訓音 keys for finding a character by its gloss.

Usage: uv run tools/build_hun.py

Each gloss in libhangul hanja.txt gives a key, the gloss with its spaces
removed: 비:肥:살찔 비 → 살찔비 肥. A gloss with no reading of its own takes
the entry's (강:強:강할 → 강할강). A gloss written with the other initial-sound
form of the reading gets a key for each (양:良:어질 량 → 어질량, 어질양).
A key with several characters lists the ones used most in sino.tsv first.
"""

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "data" / "sources" / "libhangul" / "hanja.txt"
HANGUL = re.compile("[가-힣]+")
BASE = 0xAC00


def parts(syllable: str) -> tuple[int, int]:
    """(initial, vowel and final) of a Hangul syllable."""
    n = ord(syllable) - BASE
    return n // 588, n % 588


def initial_sound_pair(a: str, b: str) -> bool:
    """Whether two readings differ only by 두음법칙: 량/양, 녀/여, 라/나."""
    if len(a) != 1 or len(b) != 1 or not HANGUL.fullmatch(a + b):
        return False
    (ia, ra), (ib, rb) = parts(a), parts(b)
    return ra == rb and ia != ib and {ia, ib} <= {2, 5, 11}


def keys(reading: str, gloss: str) -> list[str]:
    words = gloss.split()
    if not words or not all(HANGUL.fullmatch(w) for w in words):
        return []
    last = words[-1]
    if last == reading:
        return ["".join(words)]
    if initial_sound_pair(last, reading):
        return ["".join(words), "".join(words[:-1]) + reading]
    return ["".join(words) + reading]


def usage() -> dict[tuple[str, str], int]:
    counts = {}
    for line in (ROOT / "dict" / "sino.tsv").open(encoding="utf-8"):
        f = line.rstrip("\n").split("\t")
        if not line.startswith("#") and len(f) >= 3 and len(f[1]) == 1 and f[2].isdigit():
            counts[(f[0], f[1])] = int(f[2])
    return counts


def main() -> None:
    counts = usage()
    rows = set()
    for line in SOURCE.open(encoding="utf-8"):
        if line.startswith("#"):
            continue
        f = line.rstrip("\n").split(":", 2)
        if len(f) < 3 or len(f[1]) != 1:
            continue
        reading, hanja, glosses = f
        for gloss in re.split(r"\s*[,;]\s*", glosses):
            for key in keys(reading, gloss):
                rows.add((key, hanja, counts.get((reading, hanja), 0)))
    best = {}
    for key, hanja, n in rows:
        best[(key, hanja)] = max(n, best.get((key, hanja), 0))
    with (ROOT / "dict" / "hun.tsv").open("w", encoding="utf-8") as out:
        out.write("# 訓音 keys from libhangul hanja.txt (BSD-3-Clause,\n")
        out.write("# Copyright (c) 2005,2006 Choe Hwanjin; see dict/README.md).\n")
        out.write("# Built by tools/build_hun.py. Most used character first within a key.\n")
        out.write("reading\thanja\tcount\tsource\n")
        for (key, hanja), n in sorted(best.items(), key=lambda r: (r[0][0], -r[1], r[0][1])):
            out.write(f"{key}\t{hanja}\t\thun\n")


if __name__ == "__main__":
    main()
