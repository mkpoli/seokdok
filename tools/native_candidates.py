"""List native Korean words that have no hand-made 訓讀 entry yet, for
assigning Hanja by meaning.

Usage: uv run tools/native_candidates.py > data/private/native-candidates.tsv

Columns: lemma, pos, chat (how often the chats use the word's stem in Hangul),
gloss_hanja (characters whose libhangul 訓 gloss names this word), sense
(first English sense from Wiktionary, context for the assigner only).
Words come from English Wiktionary's Korean headwords (kaikki.org extract)
that carry no hanja form, so they are native or loan words.
"""
import collections
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "data" / "sources"
PRIV = ROOT / "data" / "private"
DICT = ROOT / "dict"

HANGUL = re.compile(r"^[가-힣]+$")
# Senses that mark inflections, variants, letter or note names, not words.
JUNK = re.compile(
    r"form of|infinitive of|adnominal of|nominal of|conjugat|contraction of|abbreviation of|"
    r"nonstandard|misspelling|dialect|archaic|obsolete|name of the|letter|jamo|solf|musical note|"
    r"romanization|syllable|used only|only used|North Korea",
    re.I,
)


def chat_prefixes():
    pre = collections.Counter()
    for name in ("telegram.jsonl", "telegram-hanja.jsonl"):
        p = PRIV / name
        if not p.exists():
            continue
        for line in p.open(encoding="utf-8"):
            for w in re.findall("[가-힣]+", json.loads(line)["text"]):
                for k in range(1, min(len(w), 5) + 1):
                    pre[w[:k]] += 1
    return pre


def covered():
    """Lemmas with a hand-made (non-gloss) entry already."""
    out = set()
    for f in ("corpus.tsv", "editorial.tsv", "community.tsv"):
        for line in (DICT / f).open(encoding="utf-8"):
            if line.startswith("#") or line.startswith("lemma\t"):
                continue
            out.add(line.split("\t")[0])
    return out


def gloss_hanja():
    """lemma → characters whose gloss-derived entry names it (gloss.tsv)."""
    m = collections.defaultdict(list)
    for line in (DICT / "gloss.tsv").open(encoding="utf-8"):
        if line.startswith("#") or line.startswith("lemma\t"):
            continue
        f = line.rstrip("\n").split("\t")
        han = re.match(r"[㐀-鿿豈-﫿\U00020000-\U0003134f]+", f[1])
        if han and han.group(0) not in m[f[0]]:
            m[f[0]].append(han.group(0))
    return m


def main():
    pre = chat_prefixes()
    have = covered()
    gh = gloss_hanja()
    rows = {}
    with (SRC / "kaikki-korean" / "kaikki.org-dictionary-Korean.jsonl").open(encoding="utf-8") as f:
        for line in f:
            d = json.loads(line)
            pos, word = d.get("pos"), d.get("word", "")
            if pos not in ("verb", "adj", "adv", "noun") or not HANGUL.match(word) or word in have:
                continue
            if any("hanja" in (x.get("tags") or []) for x in d.get("forms") or []):
                continue
            senses = [g for s in d.get("senses") or [] for g in s.get("glosses") or []]
            if not senses or JUNK.search(senses[0]):
                continue
            if pos in ("noun", "adv") and len(word) < 2:
                continue
            if pos in ("verb", "adj") and not word.endswith("다"):
                continue
            stem = word[:-1] if pos in ("verb", "adj") else word
            if len(stem) < 1:
                continue
            chat = pre.get(stem, 0)
            # Nouns and adverbs only when the chats use them; all verbs and adjectives.
            if pos in ("noun", "adv") and chat < 3:
                continue
            key = (word, pos)
            if key not in rows:
                rows[key] = (word, pos, chat, ",".join(gh.get(word, [])), senses[0][:90].replace("\t", " "))
    out = sorted(rows.values(), key=lambda r: (-r[2], r[0]))
    w = sys.stdout
    w.write("lemma\tpos\tchat\tgloss_hanja\tsense\n")
    for r in out:
        w.write("\t".join(str(x) for x in r) + "\n")
    print(f"{len(out)} candidates", file=sys.stderr)


if __name__ == "__main__":
    main()
