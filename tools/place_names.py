#!/usr/bin/env python3
"""Native Korean place names from Japanese Wikipedia's 朝鮮地名音訓一覧.

The page gives each name's Hanja and a romanization in its 小藤・金澤 column; its Hangul
column is empty. This tool reads the saved wikitext, turns each romanization into Hangul
syllable by syllable, and marks the readings that a character's sound does not explain.

    uv run tools/place_names.py candidates     every native-looking reading, for reading by hand
    uv run tools/place_names.py build          dict/place-names.tsv

`build` takes only the readings listed in tools/place_lemmas.tsv, which were read by hand.
"""
import collections
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REV = 111083341
REV_DATE = "2026-09-20"
SOURCE = ROOT / "data/sources/ja-wikipedia" / f"朝鮮地名音訓一覧-rev{REV}.wikitext"
LEMMAS = ROOT / "tools/place_lemmas.tsv"
OUT = ROOT / "dict/place-names.tsv"
BOOK = "ja.wikipedia 朝鮮地名音訓一覧"

HEADER = f"""# Native Korean place names: Hanja in their traditional forms, with the name in Hangul.
# Secondary source: Japanese Wikipedia, 朝鮮地名音訓一覧, revision {REV} of {REV_DATE}, column 小藤・金澤, CC BY-SA 4.0,
# https://ja.wikipedia.org/w/index.php?oldid={REV}. The page cites 朝鮮総督府「朝鮮地方行政区画一覧（明治四十四年）」『朝鮮地名資料集成〔3〕』草風館、1994年
# and carries a single-source warning. It gives romanization only: the Hangul here is transliterated from it by
# tools/place_names.py, spelled in today's vowels and otherwise as romanized. Each note quotes the romanization and the page's province.
# Ranks below kun-attested.tsv.
lemma\twritten\tpos\tclass\tsource\tcount\tnote
"""

ONSETS = {"chh": "ㅊ", "kh": "ㅋ", "th": "ㅌ", "ph": "ㅍ", "kk": "ㄲ", "tt": "ㄸ", "pp": "ㅃ", "ss": "ㅆ", "ch": "ㅈ", "dj": "ㅈ", "j": "ㅈ",
          "k": "ㄱ", "g": "ㄱ", "n": "ㄴ", "t": "ㄷ", "d": "ㄷ", "r": "ㄹ", "l": "ㄹ", "m": "ㅁ", "p": "ㅂ", "b": "ㅂ", "s": "ㅅ", "h": "ㅎ"}
# ă is the old vowel ㆍ, and ăi its ㆎ: the page's spelling of 말 and 개 as Măl and kogăi.
VOWELS = {"yöi": "ㅖ", "uăi": "ㅙ", "oăi": "ㅙ", "oai": "ㅙ", "eui": "ㅢ", "uöi": "ㅞ", "yăi": "ㅒ", "yai": "ㅒ", "ăi": "ㅐ", "ai": "ㅐ", "äi": "ㅐ",
          "oa": "ㅘ", "uö": "ㅝ", "yö": "ㅕ", "ya": "ㅑ", "yo": "ㅛ", "yu": "ㅠ", "öi": "ㅔ", "oi": "ㅚ", "ui": "ㅟ", "eu": "ㅡ",
          "ö": "ㅓ", "o": "ㅗ", "u": "ㅜ", "i": "ㅣ", "a": "ㅏ", "ă": "ㅏ", "ä": "ㅏ"}
CODAS = {"ng": "ㅇ", "lk": "ㄺ", "lp": "ㄼ", "lm": "ㄻ", "k": "ㄱ", "n": "ㄴ", "t": "ㄷ", "l": "ㄹ", "m": "ㅁ", "p": "ㅂ", "s": "ㅅ"}
CHO = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ"
JUNG = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ"
JONG = " ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ"
PLAIN = {"ㅑ": "ㅏ", "ㅕ": "ㅓ", "ㅛ": "ㅗ", "ㅠ": "ㅜ", "ㅒ": "ㅐ", "ㅖ": "ㅔ"}
ROMAN_VOWEL = "aăäiouöey"
# Japanese forms on the page, for finding a character's Korean sound.
OLD_FORM = dict(zip("辺霊渓広鉄満済麦楽鶏旧宝沢鉱関竜帰県台区画応駅塩価会絵岳気挙峡郷経継権国黒斎参残歯児実写寿収従処将称焼証乗条状図数声静専浅戦銭総双壮装増蔵続体単断遅昼庁鎮転伝灯当徳独読発豊万薬訳余与様乱覧両緑礼歴恋炉労郎湾",
                    "邊靈溪廣鐵滿濟麥樂鷄舊寶澤鑛關龍歸縣臺區畫應驛鹽價會繪嶽氣擧峽鄕經繼權國黑齋參殘齒兒實寫壽收從處將稱燒證乘條狀圖數聲靜專淺戰錢總雙壯裝增藏續體單斷遲晝廳鎭轉傳燈當德獨讀發豐萬藥譯餘與樣亂覽兩綠禮歷戀爐勞郞灣"))


TRADITIONAL = {**OLD_FORM, **dict(zip("仏国徳楼黒", "佛國德樓黑"))}


def traditional(row):
    """A row with its Hanja column in traditional forms."""
    f = row.split("\t")
    f[1] = "".join(TRADITIONAL.get(c, c) for c in f[1])
    return "\t".join(f)


def syllables(piece):
    """One hyphen-separated piece of romanization as Hangul, or None if it does not parse."""
    s, out, i = piece.lower(), [], 0
    while i < len(s):
        onset = "ㅇ"
        for k in sorted(ONSETS, key=len, reverse=True):
            if s.startswith(k, i):
                onset, i = ONSETS[k], i + len(k)
                break
        for k in sorted(VOWELS, key=len, reverse=True):
            if s.startswith(k, i):
                vowel, i = VOWELS[k], i + len(k)
                break
        else:
            return None
        if onset in "ㅅㅆㅈㅉㅊ":  # 1900s spelling: Chyö-jyu, Syöm
            vowel = PLAIN.get(vowel, vowel)
        coda = " "
        for k in sorted(CODAS, key=len, reverse=True):
            if s.startswith(k, i) and (i + len(k) == len(s) or s[i + len(k)] not in ROMAN_VOWEL):
                coda, i = CODAS[k], i + len(k)
                break
        out.append(chr(0xAC00 + CHO.index(onset) * 588 + JUNG.index(vowel) * 28 + JONG.index(coda)))
    return out


def to_hangul(roman):
    out = []
    for piece in roman.split("-"):
        got = syllables(piece.strip())
        if got is None:
            return None
        out += got
    return out


def parse_page(text):
    """(line number, Hanja names, romanized readings, province) for each table row."""
    for no, line in enumerate(text.split("\n"), 1):
        if not line.startswith("!") or "||" not in line:
            continue
        cells = line.split("||")
        head = re.sub(r"\[\[[^\]|]*\|", "", cells[0].lstrip("!\t "))
        names = [n.strip() for n in re.findall(r"[^\[\]（）()]+", head) if n.strip()]
        romans = [r.strip() for r in re.split(r"[()（）,]", cells[2]) if r.strip()]
        yield no, names, romans, cells[7].strip()


def jamo(h):
    x = ord(h) - 0xAC00
    return CHO[x // 588], JUNG[x % 588 // 28]


LOOSE_ONSET = {"ㄷ": "ㅈ", "ㅊ": "ㅈ", "ㅌ": "ㅈ", "ㄸ": "ㅈ", "ㅉ": "ㅈ", "ㅋ": "ㄱ", "ㄲ": "ㄱ", "ㅍ": "ㅂ", "ㅃ": "ㅂ", "ㅆ": "ㅅ", "ㄹ": "ㄴ", "ㅇ": "ㄴ"}
LOOSE_VOWEL = {"ㅑ": "ㅏ", "ㅕ": "ㅓ", "ㅛ": "ㅗ", "ㅠ": "ㅜ", "ㅡ": "ㅜ", "ㅓ": "ㅜ", "ㅗ": "ㅜ", "ㅐ": "ㅏ", "ㅒ": "ㅏ", "ㅔ": "ㅏ", "ㅖ": "ㅏ",
               "ㅚ": "ㅏ", "ㅟ": "ㅣ", "ㅢ": "ㅣ", "ㅘ": "ㅏ", "ㅝ": "ㅜ", "ㅙ": "ㅏ", "ㅞ": "ㅏ"}


def same_sound(a, b):
    """Loose on purpose: the page's spelling, 두음법칙 and assimilation all blur a sound."""
    (oa, va), (ob, vb) = jamo(a), jamo(b)
    return LOOSE_ONSET.get(oa, oa) == LOOSE_ONSET.get(ob, ob) and LOOSE_VOWEL.get(va, va) == LOOSE_VOWEL.get(vb, vb)


def sino_table():
    table = collections.defaultdict(set)
    with (ROOT / "dict/sino.tsv").open(encoding="utf-8") as f:
        for line in f:
            r = line.split("\t")
            if len(r) >= 2 and len(r[0]) == 1 and len(r[1]) == 1:
                table[r[1]].add(r[0])
    return table


def explained(hanja, syls, sino):
    """How many of the characters a syllable's sound accounts for, in order."""
    reads = [sino.get(c, set()) | sino.get(OLD_FORM.get(c, c), set()) for c in hanja]
    best = [[0] * (len(syls) + 1) for _ in range(len(hanja) + 1)]
    for i in range(len(hanja) - 1, -1, -1):
        for j in range(len(syls) - 1, -1, -1):
            hit = any(same_sound(r, syls[j]) for r in reads[i])
            best[i][j] = max(best[i + 1][j], best[i][j + 1], best[i + 1][j + 1] + hit)
    return best[0][0]


def candidates():
    sino = sino_table()
    text = SOURCE.read_text(encoding="utf-8")
    for no, names, romans, province in parse_page(text):
        for roman in romans:
            syls = to_hangul(roman)
            if syls is None:
                print(f"UNPARSED\t{no}\t{names[0]}\t{roman}")
            elif explained(names[0], syls, sino) < len(names[0]) or len(syls) != len(names[0]):
                print(f"{no}\t{names[0]}\t{roman}\t{''.join(syls)}\t{province}")


def build():
    pages = collections.defaultdict(list)
    for _no, names, romans, province in parse_page(SOURCE.read_text(encoding="utf-8")):
        for roman in romans:
            pages[names[0], roman].append(province)
    better = set()
    for name in ("corpus", "editorial", "community", "eonhae", "kun-attested"):
        for line in (ROOT / "dict" / f"{name}.tsv").read_text(encoding="utf-8").splitlines():
            better.add(tuple(line.split("\t")[:2]))
    rows = []
    for line in LEMMAS.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        hanja, roman, lemma = line.split("\t")
        provinces = "・".join(dict.fromkeys(pages[hanja, roman]))
        assert provinces, (hanja, roman)
        if (lemma, hanja) in better:
            continue
        literal = "".join(to_hangul(roman))
        spelled = "" if literal == lemma else f"; transliterates as {literal}"
        rows.append(f"{lemma}\t{hanja}\tnoun\t\tplace-name\t\t{BOOK} rev{REV} (secondary, Wikipedia), 小藤・金澤: {hanja} {roman}, {provinces}{spelled}")
    # The page prints Japanese forms; Korean writes the traditional ones.
    rows = [traditional(r) for r in rows]
    OUT.write_text(HEADER + "\n".join(sorted(rows)) + "\n", encoding="utf-8")
    print(len(rows), "rows")


if __name__ == "__main__":
    {"candidates": candidates, "build": build}[sys.argv[1]]()
