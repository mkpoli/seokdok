"""Find 訓讀 for native Korean words through Japanese 訓読み.

Usage: uv run tools/japanese_kun.py

A Korean word and a Japanese native word that share an English gloss are
taken to mean the same: 태어나다 "to be born" ↔ 生まれる / 産まれる "to be born".
The kanji of the Japanese spelling (生, 產 in Korean forms) become the
candidates. Only Japanese words spelled one kanji + okurigana (verbs,
adjectives) or one kanji alone (nouns) count, which keeps to 訓読み words;
Sino-Japanese compounds and する verbs are left out.

Inputs:
  data/sources/kaikki-korean/*.jsonl      Korean headwords and English senses (Wiktionary)
  data/sources/jmdict/JMdict_e.gz         JMdict (EDRDG, CC BY-SA 4.0)
  data/sources/opencc/JPShinjitaiCharacters.txt   shinjitai → traditional (OpenCC, Apache-2.0)
  data/sources/libhangul/hanja.txt        the Korean Hanja set (only characters with a Korean reading)
Output: dict/kun-ja.tsv (source "japanese", ranked with the gloss tier).
"""
import collections
import gzip
import json
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "data" / "sources"
DICT = ROOT / "dict"

KANJI = r"[㐀-鿿豈-﫿]"
ONE_KANJI_OKURI = re.compile(rf"^({KANJI})([ぁ-ゟ]+)$")
ONE_KANJI = re.compile(rf"^({KANJI})$")
HANGUL = re.compile(r"^[가-힣]+$")
# Glosses too general to tie two words together.
GENERIC = {"do", "be", "make", "go", "come", "have", "get", "put", "take", "give", "say", "see",
           "one", "thing", "person", "place", "time", "way", "it", "this", "that", "become"}


def norm(g):
    """'to be born (of a child)' → 'born'; 'hot; heated' → ['hot', 'heated']."""
    out = []
    for part in re.split(r"[;,]", re.sub(r"\([^)]*\)", "", g.lower())):
        p = part.strip().strip(".!")
        p = re.sub(r"^(to be |to become |to get |to |be |a |an |the )", "", p).strip()
        p = re.sub(r"\s+", " ", p)
        if p and len(p) > 2 and p not in GENERIC and len(p.split()) <= 4:
            out.append(p)
    return out


def korean_hanja():
    have = set()
    for line in (SRC / "libhangul" / "hanja.txt").open(encoding="utf-8"):
        f = line.split(":")
        if len(f) >= 2 and len(f[1]) == 1:
            have.add(f[1])
    return have


def shinjitai():
    m = {}
    for line in (SRC / "opencc" / "JPShinjitaiCharacters.txt").open(encoding="utf-8"):
        if line.startswith("#") or "\t" not in line:
            continue
        k, v = line.rstrip("\n").split("\t")
        m[k] = v.split()
    return m


def to_korean(ch, sj, have):
    for c in [*reversed(sj.get(ch, [])), ch]:
        if c in have:
            return c
    return None


def jmdict(sj, have):
    """gloss → Counter of (class, korean_char) with a weight; also an example word."""
    index = collections.defaultdict(collections.Counter)
    example = {}
    with gzip.open(SRC / "jmdict" / "JMdict_e.gz") as f:
        for _, el in ET.iterparse(f, events=("end",)):
            if el.tag != "entry":
                continue
            kebs = [(k.findtext("keb"), [p.text for p in k.findall("ke_pri")], [i.text for i in k.findall("ke_inf")])
                    for k in el.findall("k_ele")]
            senses = el.findall("sense")
            pos_all = [p.text or "" for s in senses for p in s.findall("pos")]
            for keb, pri, inf in kebs:
                if not keb or any("rarely" in (i or "") or "search-only" in (i or "") for i in inf):
                    continue
                m = ONE_KANJI_OKURI.match(keb)
                single = ONE_KANJI.match(keb)
                if not (m or single):
                    continue
                ch = to_korean((m or single).group(1), sj, have)
                if not ch:
                    continue
                common = 1.5 if pri else 1.0
                for si, s in enumerate(senses[:2]):
                    # The first sense is the word's main meaning.
                    weight = common * (1.0 if si == 0 else 0.5)
                    pos = " ".join(p.text or "" for p in s.findall("pos")) or " ".join(pos_all)
                    if m and ("verb" in pos and "auxiliary" not in pos):
                        cls = "verb"
                    elif m and "adjective (keiyoushi)" in pos:
                        cls = "adj"
                    elif single and "noun" in pos:
                        cls = "noun"
                    else:
                        continue
                    for g in s.findall("gloss"):
                        for key in norm(g.text or ""):
                            index[key][(cls, ch, si == 0)] += weight
                            example.setdefault((cls, ch, key), keb)
            el.clear()
    return index, example


def korean_words():
    """(lemma, pos, [normalized glosses]) for native headwords (no hanja form)."""
    out = {}
    with (SRC / "kaikki-korean" / "kaikki.org-dictionary-Korean.jsonl").open(encoding="utf-8") as f:
        for line in f:
            d = json.loads(line)
            pos, word = d.get("pos"), d.get("word", "")
            if pos not in ("verb", "adj", "noun") or not HANGUL.match(word):
                continue
            if pos in ("verb", "adj") and not word.endswith("다"):
                continue
            if any("hanja" in (x.get("tags") or []) for x in d.get("forms") or []):
                continue
            glosses = []
            for si, s in enumerate((d.get("senses") or [])[:2]):
                for g in s.get("glosses") or []:
                    if re.search(r"form of|infinitive|adnominal|alternative|dialect|archaic|obsolete", g, re.I):
                        continue
                    glosses.extend((k, si == 0) for k in norm(g))
            if glosses:
                out.setdefault((word, pos), set()).update(glosses)
    return out


def existing():
    pairs = set()
    for name in ("corpus.tsv", "editorial.tsv", "community.tsv", "suggested.tsv", "gloss.tsv"):
        p = DICT / name
        if p.exists():
            for line in p.open(encoding="utf-8"):
                c = line.split("\t")
                if len(c) > 1 and not line.startswith("#"):
                    pairs.add((c[0], c[1]))
    return pairs


def sino_readings():
    out = set()
    for line in (DICT / "sino.tsv").open(encoding="utf-8"):
        c = line.split("\t")
        if len(c) > 1 and not line.startswith("#"):
            out.add(c[0])
    return out


def honorific(word, written, natives, sino):
    """A word in the honorific suffix 님 keeps the suffix in Hangul (부처님 →
    佛님). When the stem is a Sino-Korean word and no native one (수령님,
    공주님), the Japanese match is only a translation of the whole: dropped,
    since the Sino word + 님 already converts. One-syllable stems (장님, 마님)
    are left as they are."""
    if not word.endswith("님") or len(word) < 3:
        return written
    stem = word[:-1]
    if stem in sino and stem not in natives:
        return None
    return written + "님"


def main():
    have = korean_hanja()
    sj = shinjitai()
    index, example = jmdict(sj, have)
    # A gloss shared by too many characters says little.
    spread = {k: len(v) for k, v in index.items()}
    words = korean_words()
    seen = existing()
    natives = {w for w, _ in seen}
    sino = sino_readings()
    rows = []
    weak_rows = []
    for (word, pos), glosses in sorted(words.items()):
        cls = "noun" if pos == "noun" else ("adj" if pos == "adj" else "verb")
        score = collections.Counter()
        why = {}
        main = collections.Counter()
        for g, kmain in glosses:
            wide = spread.get(g, 0)
            if wide > 24:
                continue
            for (c, ch, jmain), w in index.get(g, {}).items():
                # Korean adjectives often gloss as Japanese verbs and the reverse.
                if c == cls or {c, cls} <= {"verb", "adj"}:
                    score[ch] += w * (1.0 if kmain else 0.5) * (1.0 if wide <= 8 else 0.6)
                    if kmain and jmain and wide <= 8:
                        main[ch] += 1
                        why.setdefault(ch, (g, example.get((c, ch, g), "")))
        # Strong: a main-sense match on both sides and enough weight. Weak: the
        # rest that still share a gloss; kept, ranked lower.
        strong = [ch for ch, s in score.most_common(3) if main[ch] and s >= 1.5]
        weak = [ch for ch, s in score.most_common(6) if ch not in strong and s >= 1.0][:3]
        for ch in weak:
            written = honorific(word, ch + ("다" if pos in ("verb", "adj") else ""), natives, sino)
            if written is None or (word, written) in seen:
                continue
            seen.add((word, written))
            g = next((g for g, _ in glosses if any(k[1] == ch for k in index.get(g, {}))), "")
            weak_rows.append((word, written, pos, "", "japanese-weak", "", f"“{g}” (JMdict, weak)"))
        for ch in strong:
            written = honorific(word, ch + ("다" if pos in ("verb", "adj") else ""), natives, sino)
            if written is None or (word, written) in seen:
                continue
            seen.add((word, written))
            g, ex = why[ch]
            rows.append((word, written, pos, "", "japanese", "", f"{ex} “{g}” (JMdict)"))
    with (DICT / "kun-ja.tsv").open("w", encoding="utf-8") as f:
        f.write("# 訓讀 found through Japanese 訓読み: a Korean word (English Wiktionary sense) and a Japanese\n")
        f.write("# native word spelled one kanji (+ okurigana) in JMdict share an English gloss. JMdict:\n")
        f.write("# EDRDG, CC BY-SA 4.0 (https://www.edrdg.org/edrdg/licence.html); kanji converted to Korean\n")
        f.write("# forms with OpenCC JPShinjitaiCharacters (Apache-2.0). Unreviewed; ranks with the gloss tier.\n")
        f.write("lemma\twritten\tpos\tclass\tsource\tcount\tnote\n")
        for r in rows:
            f.write("\t".join(r) + "\n")
    with (DICT / "kun-ja-weak.tsv").open("w", encoding="utf-8") as f:
        f.write("# Weaker links through Japanese 訓読み: a shared English gloss without a main-sense match on\n")
        f.write("# both sides. Kept so no plausible candidate is lost; ranks below kun-ja.tsv. JMdict CC BY-SA 4.0.\n")
        f.write("lemma\twritten\tpos\tclass\tsource\tcount\tnote\n")
        for r in weak_rows:
            f.write("\t".join(r) + "\n")
    print(f"{len(rows)} strong rows, {len(weak_rows)} weak rows", file=sys.stderr)


if __name__ == "__main__":
    main()
