"""Refresh the count column of the native dictionaries from the chat corpus.

Usage: uv run tools/attest.py

Runs `seokdok attest` over data/private/corpus.txt (one message per line,
made from data/private/telegram.jsonl) and writes each entry's count back to
its line in dict/*.tsv. A corpus token is credited to the most trusted
entries that produce it (corpus/editorial before community before gloss).
"""
import collections
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PRIV = ROOT / "data" / "private"


def main():
    corpus = PRIV / "corpus.txt"
    with corpus.open("w", encoding="utf-8") as f:
        for line in (PRIV / "telegram.jsonl").open(encoding="utf-8"):
            f.write(json.loads(line)["text"].replace("\n", " ") + "\n")
    out, un = PRIV / "attest.tsv", PRIV / "unexplained.tsv"
    subprocess.run(["cargo", "run", "-q", "--release", "-p", "seokdok-cli", "--", "--dict", str(ROOT / "dict"),
                    "attest", str(corpus), str(out), str(un)], check=True, cwd=ROOT)
    counts = collections.defaultdict(dict)
    for line in out.read_text(encoding="utf-8").splitlines()[1:]:
        file, lineno, _lemma, _w, count = line.split("\t")
        counts[file][int(lineno)] = int(count)
    for file in ["corpus.tsv", "editorial.tsv", "community.tsv", "eonhae.tsv", "suggested.tsv", "gloss.tsv", "kun-ja.tsv", "kun-ja-weak.tsv"]:
        p = ROOT / "dict" / file
        if not p.exists():
            continue
        lines = p.read_text(encoding="utf-8").split("\n")
        for i, l in enumerate(lines):
            if not l or l.startswith("#") or l.startswith("lemma\t"):
                continue
            f = l.split("\t")
            while len(f) < 7:
                f.append("")
            c = counts[file].get(i + 1)
            f[5] = str(c) if c else ""
            lines[i] = "\t".join(f)
        p.write_text("\n".join(lines), encoding="utf-8")
        print(f"{file}: {len(counts[file])} entries attested", file=sys.stderr)


if __name__ == "__main__":
    main()
