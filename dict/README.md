# Dictionaries

Tab-separated files loaded by the engine. Lines starting with `#` are
comments; the header line names the columns.

| File | Kind | Source | Licence |
|---|---|---|---|
| `user.tsv`, `user-sino.tsv` | your own entries (not shipped) | you | yours |
| `corpus.tsv` | 訓讀 spellings attested in the 釋讀會 chat | project editors | MIT (this repository) |
| `editorial.tsv` | 訓讀 entries added by the editors, with reasons | project editors | MIT |
| `community.tsv` | 標準國語釋讀辭典 | 釋讀會 members | used with the editors' permission |
| `eonhae.tsv` | 訓讀 attested in historical books: character primers and 諺解 | Korean Wikisource and 나무위키 transcriptions, English Wiktionary | CC BY-SA 4.0; citations of 나무위키 CC BY-NC-SA 2.0 KR |
| `kun-attested.tsv` | 訓讀 printed with a native reading beside the Hanja in books of 1882-1945 | Korean Wikisource transcriptions | CC BY-SA 4.0 |
| `suggested.tsv` | 訓讀 for native words with no hand-made entry, machine-suggested, and words the chat types most, each with its source | language model, unreviewed; English Wiktionary, JMdict, libhangul | MIT; the rows from `tools/chat_gaps.tsv` CC BY-SA 4.0 |
| `place-names.tsv` | Native Korean place names with their Hanja, spelled in Hangul from a romanized list | Japanese Wikipedia, 朝鮮地名音訓一覧 | CC BY-SA 4.0 |
| `kun-ja.tsv` | 訓讀 found through Japanese 訓読み words with the same English gloss | JMdict, Wiktionary, OpenCC | CC BY-SA 4.0 (JMdict) |
| `kun-ja-weak.tsv` | weaker Japanese links, kept so no candidate is lost | JMdict, Wiktionary, OpenCC | CC BY-SA 4.0 (JMdict) |
| `gloss.tsv` | 訓讀 candidates from 訓音 glosses (하늘 천 → 하늘 = 天) | libhangul | BSD-3-Clause |
| `wikt-native.tsv` | Common native words that came out in Hangul: compounds of existing rows, or a 訓音 or JMdict link | English Wiktionary, libhangul, JMdict | CC BY-SA 4.0 |
| `editorial-sino.tsv` | Words starting in Hangul with a Hanja part (두벌式), and the first candidate among homophones, set by the editors | project editors | MIT |
| `fusion.tsv` | 音讀 words with usage frequencies | FusionIME (十八字) | GPL-3.0 |
| `sino.tsv` | 音讀 words and characters, plus words attested in the chat | libhangul, 釋讀會 chat | BSD-3-Clause |
| `etym.tsv` | Hanja spellings the files above lack, including words Sino-Korean only by origin (대충 大總, 재미 滋味, 김치 沉菜) | English Wiktionary (kaikki.org) | CC BY-SA 4.0 |
| `people.tsv` | Well-known people's names and their Hanja | Wikidata, Korean Wikipedia | CC0 1.0, CC BY-SA 4.0 |
| `loanwords.tsv` | Korean words borrowed from European languages, which stay in Hangul (페이지, 파일, 피자) | English Wiktionary (kaikki.org) | CC BY-SA 4.0 |
| `hun.tsv` | Keys for finding a character by its 訓音 gloss (살찔비 肥) | libhangul | BSD-3-Clause |

Earlier files win ties: an entry in `user.tsv` ranks above the same word
elsewhere.

## Native entries (`*.tsv` except `sino.tsv`)

```
lemma	written	pos	class	source	count	note
읽다	讀다	verb		community	292
있다	在ㅆ다	adj		editorial	15	final ㅆ always written
처음	初음	noun		corpus	42
```

- `written` is the 釋讀 dictionary form. Leading Hanja cover the start of the
  lemma; trailing Hangul repeats what they leave uncovered (初음, 囚두다); a
  trailing lone consonant (在ㅆ다, 事ㅅ) is always written out. A compound
  verb keeps its inner Hangul between the characters (奪아去다). See
  [docs/orthography.md](../docs/orthography.md) §2.
- `pos`: noun pron num bound adv det interj verb adj, or the Korean names
  (명사, 동사 …).
- `class`: inflection class, guessed when blank: `reg`, `b` (ㅂ), `d` (ㄷ),
  `s` (ㅅ), `h` (ㅎ), `reu` (르), `reo` (러), `ani` (아니다).
- `source`: `user`, `corpus`, `editorial`, `community`, `eonhae`, `kun-attested`, `suggested`, `place-name`, `person-name`, `gloss`.
- `count`: corpus attestations, refreshed by `tools/attest.py`. Attested
  entries rank by count.

## Sino-Korean entries (`sino.tsv`, `user-sino.tsv`)

```
reading	hanja	count	source
한국어	韓國語	18	lexicon
```

## Adding words

Put them in `user.tsv` (or `%APPDATA%\seokdok\user.tsv` for the Windows IME)
with `source` set to `user`. Check a file with
`cargo run -p seokdok-cli -- check`.

## Licences and provenance

- **Historical books** (`eonhae.tsv`, `tools/build_eonhae.py`): each row's note
  says where to check it. `primer:` rows come from books that gloss one
  character at a time: 訓蒙字會 (崔世珍, 1527; the copy transcribed is the 1613
  reprint at 奎章閣), 光州千字文 (1575), 新增類合 (柳希春, 1576), 石峯千字文 (韓濩, 1583; a later undated
  printing), 類合 (the 1913 新舊書林 printing), 倭語類解 (洪舜明, 18th century),
  譯語類解 (1690; Wikisource dates its copy, 奎章閣 古3912-5, to 1775) and the 物名
  of 詩經諺解 (1820 printing). `aligned:` rows come from books printing the Chinese
  next to its Korean: 小學諺解 (1587), 孝經諺解 (1590), 論語諺解, 中庸諺解 and 詩經諺解,
  書傳諺解 (1820 printings), 孟子諺解 (1693 printing), 大學諺解 (1695 printing),
  周易諺解 (1862 printing), 十九史略諺解 (1772) and 古文眞寶諺解 (英祖 reign); a row
  needs three passages in two books, and its note quotes the shortest of each. The
  books are in the public domain; the text is Korean Wikisource's transcription
  (dump of 2026-11-01), CC BY-SA 4.0, https://ko.wikisource.org/. Most pages are
  marked unproofread there. An old word becomes a modern lemma through the folds of
  `crates/seokdok-core/src/fold.rs`, the modern forms Wikisource's editors give
  beside 訓蒙字會, or the Middle Korean descendants in English Wiktionary (kaikki.org,
  extract of 2026-10-30, CC BY-SA 4.0). The lemma must be a headword of Wiktionary or
  FusionIME, and its meaning must agree with the character's: through another
  dictionary here, or an English sense it shares with Unihan's definition; rows
  resting on the libhangul gloss alone need a form with one possible reading. Rows
  with Hanja the word already has by hand are left out. A hand check of 150 random
  rows from Wikisource found 3 wrong (2.0%), since removed; expect a similar share in
  the rest.

  新增類合 (3,000 characters) and 訓蒙字會's first volume also come from 나무위키:
  https://namu.wiki/w/신증유합/상권, https://namu.wiki/w/신증유합/하권 and
  https://namu.wiki/w/훈몽자회/상권, saved 2026-11-04, CC BY-NC-SA 2.0 KR. Each citation
  gives the page with the entry number (新增類合) or section (訓蒙字會) and repeats the
  licence. This source is weaker than Wikisource: anyone can edit it, the pages name no
  copy of the book or scan, and 新增類合's entries have no folio. Its 訓蒙字會 spells 82 of
  841 glosses that both sites have differently from Wikisource (티ᇰᄌᆞ, ᄐᆡᆼᄌᆞ), so
  when both have an entry the row cites both. The books are in the public domain;
  whether a gloss copied from these pages carries the wiki's licence is uncertain, so a
  commercial redistributor should drop the rows that cite only 나무위키. A hand check of
  120 random rows new from these pages and the newly read Wikisource sections found 12
  wrong (10.0%), mostly Sino-Korean glosses (복 祉, 창 牖) or a wrong modern word for
  the old form (ᄀᆞᄅᆞᆷ 代 as 가름); every new row was then read and the wrong ones
  excluded in the tool.

  Old forms also reach modern words through a table of what English Wiktionary states:
  another form of an entry, an etymology's first attestation, a Descendants section
  (wiktextract, `raw-wiktextract-data.jsonl.gz` dated 2026-11-04, sha256
  `5ff2c072f8099f2d2ce484bd6957bf179f046d2ad37bc36555edf8bb84cb5291`; Ylonen,
  "Wiktextract", LREC 2022; CC BY-SA 4.0 and GFDL). Only links the table calls `attested`
  are read, for lemmas with a current sense; a descendant Wiktionary tags as reshaped is
  left out, and so are readings of Hanja. The meaning check above still decides. A row
  that needed the table says so in its note. Attested there means Wiktionary says so; its
  citations were not checked against the books. Every row this added or changed was read
  by hand: 137 candidates, 9 wrong (6.6%), since excluded, all homographs or phrases.

- **Printed readings** (`kun-attested.tsv`, `tools/build_kun_usage.py`): books of
  1882-1945 that print a native reading beside Hanja: above it ({{윗주|飛|나}}는 in
  勞動夜學讀本, 俞吉濬, 1908), below or above a Hangul word (京釜鐵道歌, 崔南善, 1908;
  地球略論, 1897) or in parentheses (군포내(軍浦場) in 화즁화, 1912; 어깨(肩) in the
  spelling manuals of 1912-1933). Every use is in `data/kun-usage.tsv`, which stays
  outside this folder because the iOS keyboard copies the folder whole. Readings a
  character's sound explains are skipped, allowing for old spellings and 두음법칙, and
  so are foreign names and remarks in parentheses. Scanned books whose Wikisource index
  dates them to 1880-1945 are covered; typed-in texts with no scan are left out, since
  their Hanja may be an editor's. A row here is a use with a modern word that no higher
  tier spells the same way; lemmas come from `tools/kun_lemmas.txt`, read by hand, or
  from the folds and evidence of `build_eonhae.py`. Each note gives book, year, page and
  quote. The books are in the public domain; the text is Korean Wikisource's
  transcription (dump of 2026-11-01), CC BY-SA 4.0, and most pages are unproofread
  there. A hand check of 120 random usage rows against their pages found 5 wrong
  (4.2%), all removed with others of the same kind; a second sample of 100 found 1 (1.0%), also removed.

- **Place names** (`place-names.tsv`, `tools/place_names.py`, `tools/place_lemmas.tsv`): Japanese
  Wikipedia, 朝鮮地名音訓一覧, revision 111083341 of 2026-09-20, CC BY-SA 4.0,
  https://ja.wikipedia.org/w/index.php?oldid=111083341. A secondary source with one reference,
  朝鮮総督府「朝鮮地方行政区画一覧（明治四十四年）」『朝鮮地名資料集成〔3〕』草風館、1994年, and the page carries
  a single-source warning. Of its eight columns only Hanja, 小藤・金澤 romanization and province are
  filled, so no Hangul is attested there: each lemma is transliterated from the romanization, with
  ă read as ㅏ and ăi as ㅐ, and a plain word given today's spelling (바회 바위, 놉흔 높은); the note then
  gives the literal form. The Hanja are as the page has them, Japanese forms included (広, 仏), and
  some are visibly misread from print (長瑞 for Chyang-dan); rows like that are left out. The file
  has only readings a character's sound does not explain, read by hand. A word that already has a
  common meaning is left out (금실, 사물, 배치). Every row was reread against the page: 1 of 262 wrong
  (0.4%), 九月山 paired with its alternative name's reading, removed.
  The rows serve typing only: reading Hanja back into Hangul keeps the sound (高亭 고정).

- **Wiktionary** (`etym.tsv`, `tools/etym_hanja.py`): English Wiktionary's
  Korean entries as extracted by kaikki.org (wiktextract; enwiktionary dump of
  2026-10-01, extracted 2026-10-30, fetched 2026-11-03; Ylonen, "Wiktextract",
  LREC 2022). Three kinds of row: words Wiktionary marks as nativised from a
  Sino-Korean word (`ko-etym-nativised`, kept only when that word starts like
  the headword), words coined in Korea from Chinese morphemes (`der` + `ltc-l`),
  and Hanja forms as long as the word. Only pairs missing from `fusion.tsv` and
  `sino.tsv` are kept. CC BY-SA 4.0
  (https://en.wiktionary.org/wiki/Wiktionary:Copyrights). Wiktionary's
  etymologies are crowd-edited and some are disputed (사랑 思量); the file
  records them as Wiktionary has them.

- **People** (`people.tsv`, `tools/people_names.py`): Wikidata (CC0) lists humans of a Korean or Chinese
  state with at least 6 sitelinks and a Korean Wikipedia article; the Hanja come from the opening section of
  that article (CC BY-SA 4.0), and a form is kept only when each character's sound matches the Hangul name.
  Each note gives the Wikidata item and article revision. A wiki is a secondary source. Sound checks use `sino.tsv`
  and Unihan's kHangul. Rows rank below every common word sharing their reading.

- **Loanwords** (`loanwords.tsv`, `tools/loanwords.py`): headwords of the same
  English Wiktionary extract (kaikki.org, enwiktionary dump of 2026-10-01,
  fetched 2026-11-03; Ylonen, "Wiktextract", LREC 2022) whose every entry is a
  `bor`, `bor+` or `lbor` template from a European language, Turkish, Arabic,
  Hindi or Persian, with two or more syllables. Words with a Sino-Korean, native
  or nativised etymology template, words borrowed from Japanese or Chinese
  (사진), words that the chat writes with Hanja ten times or more, and words
  that the corpus, editorial or community 訓讀 files list are left out. The
  engine keeps a listed word, alone or with a particle or 하다, in Hangul
  instead of offering Hanja homophones (페이지 is not 페理智). 빌드 is added by
  hand because the extract lacks it. CC BY-SA 4.0
  (https://en.wiktionary.org/wiki/Wiktionary:Copyrights). Wiktionary's
  etymology templates are crowd-edited and can be wrong.

- **FusionIME (十八字)** by Lee0701, https://github.com/Lee0701/FusionIME
  commit `450829df` (2026-10-31), `ko_dict_all.tsv`: GPL-3.0, full text in
  `LICENSE-fusion.txt`. FusionIME states its Korean data comes from
  표준국어대사전 (stdict.korean.go.kr) and libhangul. Definitions are removed;
  frequencies are damped to `round(cbrt(frequency))` because they come from a
  different corpus than the chat counts. `fusion.tsv` stays under GPL-3.0
  wherever it is distributed; the code remains MIT.
- **Common native words** (`wikt-native.tsv`, `tools/wikt_native.py`): native headwords of
  English Wiktionary (kaikki.org extract, wiktextract; Ylonen, LREC 2022; CC BY-SA 4.0 and
  GFDL) with a frequency in FusionIME's dictionary whose first candidate was the word
  itself in Hangul. A row runs together the Hanja of the parts Wiktionary's etymology names
  (눈앞 目前, 여름철 夏철), or takes a character whose 訓音 in Wiktionary or libhangul names the
  word, or one `kun-ja.tsv` links with a sense Unihan's definition shares. No source states
  the compound's spelling itself: it is built from its parts. Every candidate was read; the
  tool lists the rejected ones with reasons. The file's source is `suggested`, the lowest
  tier that beats a two-syllable word left in Hangul. Words with no source stay in Hangul.

- **Machine suggestions** (`suggested.tsv`, 2026-11-03): native verbs,
  adjectives, adverbs and nouns that English Wiktionary lists without a hanja
  form (kaikki.org extract) and that no hand-made dictionary covers were given
  to a language model with their English sense and the libhangul characters
  whose 訓音 gloss names them; it proposed one to three characters by meaning
  and skipped words it was unsure of; a second, coverage-first pass gave
  candidates for nearly all the words skipped the first time (only particles,
  interjections, mimetic words and loanwords stay out). Every row passed the engine's parser;
  none was reviewed by a person, so the tier ranks below the chat corpus,
  editorial and community entries and is labelled as unreviewed in the IME.
  The word list came from `tools/native_candidates.py`; no Wiktionary text is
  included.

  The rows at the end of the file come from `tools/chat_gaps.tsv`: native words among
  the 20,000 most frequent chat tokens whose first candidate stayed Hangul or was a
  poor guess. Each note names what supports the spelling: a Hanja form or sense in
  English Wiktionary (kaikki.org extract, CC BY-SA 4.0), a Middle Korean form Wiktionary
  links to the modern word, a JMdict gloss (EDRDG, CC BY-SA 4.0), a libhangul 訓音 or
  another row here. Every one was read by hand. Counts stay blank where the spelling
  belongs to another word or reads as Sino-Korean: the chat's uses of 無 are not uses
  of 업시, nor its 二十 uses of 스물.
- **Through Japanese** (`kun-ja.tsv`, `tools/japanese_kun.py`): a Korean
  native word and a Japanese word spelled with one kanji (plus okurigana) in
  JMdict that share an English gloss in their main senses are linked, and the
  kanji, converted to its Korean form with OpenCC's `JPShinjitaiCharacters`
  (Apache-2.0), becomes a candidate: 태어나다 ↔ 産まれる → 產다. JMdict is by
  the Electronic Dictionary Research and Development Group, CC BY-SA 4.0,
  https://www.edrdg.org/edrdg/licence.html (fetched 2026-11-03), so this file
  is under CC BY-SA 4.0. Homonyms make some links wrong (무섭다 → 侘 through
  "worried"); the tier ranks with the glosses.
- **Chat-attested Sino-Korean words** in `sino.tsv` (source `corpus`): Hanja
  runs used at least five times in the chat that no lexicon lists (釋讀,
  釋讀會, 末音添記). Readings are each character's most common reading at
  that position in libhangul words, so a rare reading may be wrong.

- **libhangul** `data/hanja/hanja.txt`, commit `23d2cd9d` (2026-10-11):
  Copyright (c) 2005,2006 Choe Hwanjin. All rights reserved.
  Redistribution and use in source and binary forms, with or without
  modification, are permitted provided that the following conditions are met:
  (1) redistributions of source code must retain the above copyright notice,
  this list of conditions and the following disclaimer; (2) redistributions in
  binary form must reproduce the above copyright notice, this list of
  conditions and the following disclaimer in the documentation and/or other
  materials provided with the distribution; (3) neither the name of the author
  nor the names of its contributors may be used to endorse or promote products
  derived from this software without specific prior written permission. THIS
  SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
  ANY EXPRESS OR IMPLIED WARRANTIES ARE DISCLAIMED. Full header:
  https://github.com/libhangul/libhangul/blob/23d2cd9d10d516d523d60c562dbcc7ea08139a09/data/hanja/hanja.txt
  `gloss.tsv` lemmas are reconstructed from its glosses (읽을 → 읽다), and were
  checked for existence against English Wiktionary headwords (kaikki.org
  extract, 2026-10-30); no Wiktionary text is included. Reconstruction is
  heuristic and some lemmas are wrong (e.g. 깃바다), which is why this tier
  ranks last.
- **標準國語釋讀辭典**, Google Sheets
  https://docs.google.com/spreadsheets/d/1J7bPuQ4buOM17hvLR6UHLfcrERfYhaVVj1H-3szh7J0,
  compiled by 釋讀會 members from June 2022, fetched 2026-11-03, used with the
  editors' permission. Five rows are skipped with a warning because lemma and
  written form disagree (가득한 = 滿하다, 거리 = 街다 as a verb, 끼다 = 介이다,
  말 = 言하다, 사랑 = 살다).
- **釋讀會 chat corpus** (t.me/pimsdh), export to 2026-10-29: used for counts
  and for choosing `corpus.tsv` spellings. The messages themselves are not in
  this repository.
