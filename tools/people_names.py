#!/usr/bin/env python3
"""Well-known people's names and their Hanja, from Wikidata and Korean Wikipedia.

Wikidata lists the people: humans of a Korean or Chinese state with an article on Korean
Wikipedia. Each person's Hanja comes from that article's first sentence, else its infobox. A form is
taken only when every character's sound matches the name's Hangul, so a name without Hanja
in its article, or written in Hangul only, gets no row.

    uv run tools/people_names.py fetch [data dir]   query Wikidata, save the articles' openings
    uv run tools/people_names.py build [data dir]   write dict/people.tsv

The data dir defaults to data/sources/people.
"""
import csv
import json
import re
import sys
import time
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "dict/people.tsv"
AGENT = "seokdok-dict/0.1 (https://github.com/mkpoli/seokdok)"
MIN_LINKS = 6

KOREAN_STATES = {
    "Q884": "South Korea", "Q423": "North Korea", "Q18097": "Korea", "Q28233": "Korean Empire", "Q28179": "Joseon",
    "Q28208": "Goryeo", "Q28456": "Silla", "Q28370": "Goguryeo", "Q28428": "Baekje", "Q28405": "Gojoseon", "Q28322": "Balhae",
}
CHINESE_STATES = {
    "Q8733": "Qing", "Q9903": "Ming", "Q7313": "Yuan", "Q7462": "Song", "Q9683": "Tang", "Q7405": "Sui", "Q7209": "Han",
    "Q7183": "Qin", "Q35216": "Zhou", "Q274488": "Eastern Wu", "Q102289832": "Shu Han",
}

HEADER = """# Well-known people's names and their Hanja. People: Wikidata (CC0), humans of a Korean or Chinese state with at least
# {links} sitelinks and an article on Korean Wikipedia, queried {date}. Hanja: that article's opening section
# (CC BY-SA 4.0): its first sentence, else the infobox's name fields, kept only when every character's sound matches the name's Hangul. A wiki is a secondary source.
# Rows sharing a reading are ordered by sitelinks. The note gives the Wikidata item and the article's revision.
reading\thanja\tcount\tsource\tnote
"""

HAN = re.compile(r"[㐀-䶿一-鿿豈-﫿\U00020000-\U0003FFFF]+")
BOX_FIELD = re.compile(r"^\s*\|\s*[^=\n]*(?:한자|원어|본명|이름|hanja)[^=\n]*=([^\n]*)$", re.M)
# Misprints in the article, checked against other references.
WRONG = {("마조도일", "馬祖導一"), ("손흠", "孫欽")}
SOFT_VOWELS = {2, 6, 7, 12, 17, 20}  # ㅑ ㅕ ㅖ ㅛ ㅠ ㅣ


def fetch_url(url: str, accept: str = "application/json") -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": AGENT, "Accept": accept})
    with urllib.request.urlopen(req, timeout=80) as r:
        return r.read()


def sparql(state: str) -> str:
    return f"""SELECT ?item ?ko ?title ?links WHERE {{
  ?item wdt:P27 wd:{state}; wdt:P31 wd:Q5; wikibase:sitelinks ?links. FILTER(?links>={MIN_LINKS})
  ?article schema:about ?item; schema:isPartOf <https://ko.wikipedia.org/>; schema:name ?title.
  ?item rdfs:label ?ko. FILTER(LANG(?ko)="ko")
}}"""


def fetch(data: Path) -> None:
    data.mkdir(parents=True, exist_ok=True)
    for state in {**KOREAN_STATES, **CHINESE_STATES}:
        out = data / f"wd-{state}.csv"
        if out.exists() and out.stat().st_size > 20:
            continue
        for attempt in range(4):
            url = "https://query.wikidata.org/sparql?" + urllib.parse.urlencode({"query": sparql(state)})
            try:
                body = fetch_url(url, "text/csv")
            except OSError as e:
                print(state, e, file=sys.stderr)
                time.sleep(30)
                continue
            if body.startswith(b"item,"):
                out.write_bytes(body)
                break
            time.sleep(30)
        print(state, out.stat().st_size if out.exists() else "failed", flush=True)
        time.sleep(3)
    leads = data / "leads.jsonl"
    seen = set()
    if leads.exists():
        seen = {json.loads(line)["title"] for line in leads.open(encoding="utf-8")}
    titles = sorted({row["title"] for row in people(data)} - seen)
    with leads.open("a", encoding="utf-8") as f:
        for i in range(0, len(titles), 50):
            q = urllib.parse.urlencode({
                "action": "query", "format": "json", "formatversion": 2, "redirects": 1, "prop": "revisions",
                "rvprop": "ids|content", "rvslots": "main", "rvsection": 0, "titles": "|".join(titles[i:i + 50]),
            })
            reply = json.loads(fetch_url("https://ko.wikipedia.org/w/api.php?" + q))["query"]
            back = {r["to"]: r["from"] for r in reply.get("redirects", [])}
            for page in reply["pages"]:
                rev = (page.get("revisions") or [None])[0]
                if rev:
                    f.write(json.dumps({"title": back.get(page["title"], page["title"]), "rev": rev["revid"],
                                        "text": rev["slots"]["main"]["content"][:30000]}, ensure_ascii=False) + "\n")
            f.flush()
            print(i + 50, "of", len(titles), flush=True)
            time.sleep(1)


def people(data: Path) -> list[dict]:
    rows = {}
    for path in data.glob("wd-*.csv"):
        with path.open(encoding="utf-8", newline="") as f:
            for row in csv.DictReader(f):
                row["q"] = row["item"].rsplit("/", 1)[1]
                row["links"] = int(row["links"])
                rows[row["q"]] = row
    return list(rows.values())


def sound(syllable: str) -> tuple[int, int, int]:
    """A syllable's sound with the initial-sound rule undone, so 李 matches both 이 and 리."""
    code = ord(syllable) - 0xAC00
    onset, vowel, coda = code // 588, code // 28 % 21, code % 28
    if onset == 5:  # ㄹ
        onset = 11 if vowel in SOFT_VOWELS else 2
    if onset == 2 and vowel in SOFT_VOWELS:  # ㄴ
        onset = 11
    return onset, vowel, coda


def readings() -> dict[str, set[tuple[int, int, int]]]:
    table: dict[str, set] = {}
    with (ROOT / "dict/sino.tsv").open(encoding="utf-8") as f:
        for line in f:
            parts = line.split("\t")
            if len(parts) > 1 and len(parts[0]) == 1 and len(parts[1]) == 1 and "가" <= parts[0] <= "힣":
                table.setdefault(parts[1], set()).add(sound(parts[0]))
    source = ROOT / "data/sources/unihan/Unihan.zip"
    if not source.exists():
        source = Path.home() / "projects/Korean/seokdok/data/sources/unihan/Unihan.zip"
    with zipfile.ZipFile(source) as z:
        for raw in z.read("Unihan_Readings.txt").decode("utf-8").splitlines():
            parts = raw.split("\t")
            if len(parts) == 3 and parts[1] == "kHangul":
                char = chr(int(parts[0][2:], 16))
                for reading in parts[2].split():
                    syllable = reading.split(":")[0]
                    if len(syllable) == 1 and "가" <= syllable <= "힣":
                        table.setdefault(char, set()).add(sound(syllable))
    return table


def simplified() -> set[str]:
    """Characters with only other characters as their traditional forms in Unihan: 绵, not 后."""
    source = ROOT / "data/sources/unihan/Unihan.zip"
    if not source.exists():
        source = Path.home() / "projects/Korean/seokdok/data/sources/unihan/Unihan.zip"
    chars = set()
    with zipfile.ZipFile(source) as z:
        for raw in z.read("Unihan_Variants.txt").decode("utf-8").splitlines():
            parts = raw.split("\t")
            if len(parts) == 3 and parts[1] == "kTraditionalVariant":
                char = chr(int(parts[0][2:], 16))
                if char not in {chr(int(v[2:], 16)) for v in parts[2].split()}:
                    chars.add(char)
    return chars


def spelling(text: str, name: str, table: dict) -> str | None:
    """The first run of Han characters in the text whose sounds spell the name."""
    want = [sound(c) for c in name]
    for run in HAN.findall(text):
        if len(run) == len(name) and all(s in table.get(c, ()) for c, s in zip(run, want)):
            return run
    return None


def hanja_in(text: str, name: str, table: dict) -> tuple[str | None, str | None]:
    """The name's Hanja from the opening sentence, and from the infobox's name fields.

    Only those places speak of the person: a hatnote above them may gloss a homophone (성혼 成婚).
    """
    text = re.sub(r"<ref.*?(</ref>|/>)", "", text, flags=re.S)
    lead = None
    for bold in re.finditer(r"'''(.+?)'''", text):
        if name in re.sub(r"[\s\[\]]", "", bold.group(1)):
            lead = spelling(text[bold.end():bold.end() + 200].split("다. ")[0], name, table)
            break
    box = None
    for field in BOX_FIELD.finditer(text):
        box = spelling(field.group(1), name, table)
        if box:
            break
    return lead, box


def build(data: Path) -> None:
    table = readings()
    avoid = simplified()
    leads = {}
    with (data / "leads.jsonl").open(encoding="utf-8") as f:
        for line in f:
            lead = json.loads(line)
            leads[lead["title"]] = lead
    rows = {}
    skipped = 0
    conflicts = []
    for person in sorted(people(data), key=lambda p: (-p["links"], p["q"])):
        name = person["ko"].replace(" ", "")
        lead = leads.get(person["title"])
        if not lead or not 2 <= len(name) <= 6 or not all("가" <= c <= "힣" for c in name):
            skipped += 1
            continue
        opening, box = hanja_in(lead["text"], name, table)
        forms = [form for form in (opening, box) if form and not avoid.intersection(form)]
        if len(set(forms)) > 1:
            conflicts.append(" ".join([name, *forms]))
        hanja = forms[0] if forms else None
        if not hanja or (name, hanja) in WRONG:
            skipped += 1
            continue
        rows.setdefault((name, hanja), (person["links"], person["q"], lead["rev"]))
    date = max(p.stat().st_mtime for p in data.glob("wd-*.csv"))
    with OUT.open("w", encoding="utf-8") as f:
        f.write(HEADER.format(links=MIN_LINKS, date=time.strftime("%Y-%m-%d", time.gmtime(date))))
        for (name, hanja), (links, q, rev) in sorted(rows.items(), key=lambda r: (r[0][0], -r[1][0], r[0][1])):
            f.write(f"{name}\t{hanja}\t0\tperson-name\t{q} ko:{rev}\n")
    print(f"{len(rows)} rows, {skipped} people without a matching form", file=sys.stderr)
    print("opening sentence and infobox differ, the sentence kept:", *conflicts, sep="\n  ", file=sys.stderr)


if __name__ == "__main__":
    command = sys.argv[1] if len(sys.argv) > 1 else ""
    directory = Path(sys.argv[2]) if len(sys.argv) > 2 else ROOT / "data/sources/people"
    if command == "fetch":
        fetch(directory)
    elif command == "build":
        build(directory)
    else:
        sys.exit(__doc__)
