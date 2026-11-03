"""Merge machine-suggested 訓讀 rows into dict/suggested.tsv.

Usage: uv run tools/merge_suggested.py data/private/assign/out-*.tsv

Each row is checked with the engine's own parser (`seokdok check`); invalid
rows, rows whose lemma/written pair is already in a hand-made dictionary, and
duplicates are dropped. Source is forced to "suggested".
"""
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DICT = ROOT / "dict"
POS = {"verb", "adj", "adv", "noun", "pron", "det", "num", "interj"}


def existing():
    pairs = set()
    for f in ("corpus.tsv", "editorial.tsv", "community.tsv"):
        for line in (DICT / f).open(encoding="utf-8"):
            if line.startswith("#") or line.startswith("lemma\t"):
                continue
            c = line.split("\t")
            if len(c) > 1:
                pairs.add((c[0], c[1]))
    return pairs


def main():
    have = existing()
    rows, seen = [], set()
    for p in sys.argv[1:]:
        for line in Path(p).open(encoding="utf-8"):
            c = [x.strip() for x in line.rstrip("\n").split("\t")]
            if len(c) < 3 or c[0] == "lemma" or c[2] not in POS:
                continue
            lemma, written, pos = c[0], c[1], c[2]
            note = c[6] if len(c) > 6 else ""
            key = (lemma, written)
            if key in have or key in seen:
                continue
            seen.add(key)
            rows.append((lemma, written, pos, "", "suggested", "", note.replace("\t", " ")))
    # Validate with the engine: write candidates alone and read back errors.
    with tempfile.TemporaryDirectory() as d:
        body = "lemma\twritten\tpos\tclass\tsource\tcount\tnote\n" + "".join("\t".join(r) + "\n" for r in rows)
        (Path(d) / "user.tsv").write_text(body, encoding="utf-8")
        out = subprocess.run(
            ["cargo", "run", "-q", "--release", "-p", "seokdok-cli", "--", "--dict", d, "check"],
            cwd=ROOT, capture_output=True, text=True, check=True,
        ).stdout
    bad = {int(l.split(":")[1]) for l in out.splitlines() if l.startswith("user.tsv:")}
    good = [r for i, r in enumerate(rows, start=2) if i not in bad]
    with (DICT / "suggested.tsv").open("w", encoding="utf-8") as f:
        f.write("# 訓讀 suggestions for native words with no hand-made entry, proposed by a language model\n")
        f.write("# from Wiktionary senses and libhangul 訓音 glosses (2026-11-03). Unreviewed: this tier ranks\n")
        f.write("# below the chat corpus, editorial and community entries. Validated by the engine's parser only.\n")
        f.write("lemma\twritten\tpos\tclass\tsource\tcount\tnote\n")
        for r in good:
            f.write("\t".join(r) + "\n")
    print(f"{len(rows)} rows, {len(bad)} invalid, {len(good)} written", file=sys.stderr)


if __name__ == "__main__":
    main()
