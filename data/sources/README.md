# Data sources

Raw downloads. Nothing here is executed. "MIT/BSD repo" asks whether the file may ship inside an MIT- or BSD-licensed repository. Fetched 2026-11-03 (UTC).

| Source | License | MIT/BSD repo |
|---|---|---|
| libhangul `hanja.txt`, `mssymbol.txt` | BSD-3-Clause (file headers) | yes, keep notice |
| libhangul `freq-*.txt`, `compat-table.txt` | no header; repo is LGPL-2.1 | unclear |
| Unihan (kHangul, kKorean) | Unicode License v3 | yes, keep notice |
| FusionIME `ko_dict_all.tsv`, `ko_bigrams.tsv` | GPL-3.0 repo; contents derived from stdict (terms unverified) and libhangul | no |
| kaikki.org Korean JSONL | CC BY-SA 4.0 + GFDL (Wiktionary) | no (share-alike) |
| 우리말샘 / 표준국어대사전 | CC BY-SA family (unverified); bulk download not confirmed | no |
| Microsoft IME | proprietary | no |
| Korean Wikisource dump (old books) | books public domain; site CC BY-SA 4.0 | as a derived list under CC BY-SA, with credit |
| kaikki.org Middle Korean JSONL | CC BY-SA 4.0 + GFDL (Wiktionary) | no (share-alike) |
| 나무위키 신증유합, 훈몽자회 pages | books public domain; site CC BY-NC-SA 2.0 KR | uncertain; citations say so |

## 1. libhangul — `libhangul/`

- URL: https://github.com/libhangul/libhangul/tree/23d2cd9d10d516d523d60c562dbcc7ea08139a09/data/hanja
- Commit: `23d2cd9d10d516d523d60c562dbcc7ea08139a09` (2026-10-11).
- Files: `hanja.txt`, `mssymbol.txt`, `freq-hanja.txt`, `freq-hanjaeo.txt`, `compat-table.txt`, `merge.py.txt` (renamed, not run), `Makefile.am`, plus repo `COPYING` (LGPL-2.1) and `AUTHORS`.
- License of the data: the headers of `hanja.txt` and `mssymbol.txt` carry a separate BSD-3-Clause notice, unlike the LGPL-2.1 code:
  > Copyright (c) 2005,2006 Choe Hwanjin (mssymbol.txt: 2005-2014)
  > Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met: 1. Redistributions of source code must retain the above copyright notice ... 3. Neither the name of the author nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.

  Header: https://github.com/libhangul/libhangul/blob/23d2cd9d10d516d523d60c562dbcc7ea08139a09/data/hanja/hanja.txt
- `freq-hanja.txt`, `freq-hanjaeo.txt`, `compat-table.txt` have no license header and are not listed in `Makefile.am` (only `hanja.txt`, `mssymbol.txt` are installed). They fall under the repo's LGPL-2.1 by default; treat as not redistributable until the author is asked.
- Redistributable in MIT/BSD repo: `hanja.txt`, `mssymbol.txt` yes, with the copyright notice and conditions kept.
- Format (`#` comment header of 26 lines, then `reading:hanja:gloss`):
  ```
  천:天:하늘 천
  한국어:韓國語:
  한국:韓國:대한민국
  ```
  Single-character keys carry 訓音 as `훈 음` (`독:讀:읽을 독`); multi-reading characters list several glosses (`두:讀:구절 두, 읽을 두`). Multi-character entries have an empty or short note.
- `hanja.txt`: 303,496 entries; 28,472 single-character, of which 7,757 have a gloss. 
- `mssymbol.txt` (`ㄱ:！:`, 989 symbol entries), `freq-hanja.txt` (`㐀:0`, 27,496 lines, per-character frequency), `freq-hanjaeo.txt` (`一一査得:4000095`, 219,287 lines, per-word score), `compat-table.txt` (`0xF900<TAB>0x8C48`, 997 compatibility-ideograph mappings).
- Weakness: the BSD header covers the whole file but the origin of individual entries is undocumented. Glosses exist for only about 7.7k characters. Native-word 訓讀 (하늘→天) exists only through the single-character gloss; no word-level 하늘→天 entry (`하늘` is absent from `hanja.txt`).

## 2. Unihan — `unihan/`

- URL: https://www.unicode.org/Public/UCD/latest/ucd/Unihan.zip (Unicode 18.0.0, `Unihan_Readings.txt` dated 2026-07-31; zip last-modified 2026-08-05).
- Derived: `unihan_kHangul_kKorean.tsv` (kHangul and kKorean lines of `Unihan_Readings.txt`, unmodified, tab-separated `codepoint field value`), 17,575 lines (8,525 kHangul, 9,050 kKorean).
  ```
  U+5929	kHangul	천:0E
  U+5929	kKorean	CHEN
  U+7214	kHangul	희:1N
  ```
- License: Unicode License v3, saved as `unihan/license.txt` (from https://www.unicode.org/license.txt): "Permission is hereby granted, free of charge, to any person obtaining a copy of data files ... to deal in the Data Files or Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, and/or sell copies ... provided that either (a) this copyright and permission notice appear with all copies ..."
- Redistributable in MIT/BSD repo: yes, with the notice.
- Weakness: readings only (kHangul has the 음 plus a source-set code; kKorean is Yale romanization). No 訓.

## 3. FusionIME — `fusion-ime/`

- Package `ee.oyatl.ime.fusion.free` is built from https://github.com/Lee0701/FusionIME (README links the Play listing).
- Commit: `450829dff2b10b90f540290c96de81a9e68168d4` (2026-10-31). Data files added 2026-10-30 (commit `59f97e72` "Add Korean dict sources").
- License: GPL-3.0 (`fusion-ime/LICENSE`). README: "Dictionary Data: Korean: Korean Standard Dictionary (stdict.korean.go.kr), libhangul".
- Files: `ko_dict_all.tsv` (38.8 MB), `ko_bigrams.tsv` (10.9 MB), `GenKoreanDict.kt.txt` (generator, renamed, not run), `README.md`, `LICENSE`, `PRIVACY.md`. The compiled `.bin` files in the app are not fetched.
- `ko_dict_all.tsv` format: 5 tab-separated columns `reading hanja frequency eumhun definition`; 395,057 lines, 335,869 with hanja in column 2, 109,867 of those with frequency > 0, 7,757 single-character hanja with eumhun, 241,895 with a stdict-style definition.
  ```
  천	天	49550	하늘 천	하늘 천\n[명사] 우리나라 성(姓)의 하나. ...
  한국어	韓國語	1313		[명사] 한국인이 사용하는 언어. ...
  하늘	하늘	159		
  ```
- `ko_bigrams.tsv`: `token<TAB>token<TAB>count`, 748,303 lines; contents are noisy (`鋼	電機	122`) and the corpus is not stated.
- Redistributable in MIT/BSD repo: no. The repo is GPL-3.0, and the definitions look copied from 표준국어대사전 / 우리말샘 text whose terms are listed under section 5. The single-character eumhun count (7,757) matches libhangul's glossed single characters, so it probably derives from it; use libhangul directly.
- Weakness: no statement of how the TSV was built; its per-entry provenance is unknown. Frequencies come from an unnamed corpus.
- Other Lee0701 repos checked: `Converter` (Apache-2.0, app only), `Converter-Dict-Data` (no license, small station/name lists), `hanja` (no license, a site), `mozc_ko_Kore` (Mozc fork). Not downloaded.

## 4. kaikki.org Korean — `kaikki-korean/`

- URL: https://kaikki.org/dictionary/Korean/kaikki.org-dictionary-Korean.jsonl (207,649,961 bytes; 64,457 lines). Extracted 2026-10-30 from the enwiktionary dump of 2026-10-01 with wiktextract; fetched 2026-11-03T08:17Z (`fetched.txt`). Free disk before: 171 GB.
- License: Wiktionary text, "dual-licensed ... under both the Creative Commons Attribution-ShareAlike 4.0 International License (CC-BY-SA) and the GNU Free Documentation License (GFDL)" (https://en.wiktionary.org/wiki/Wiktionary:Copyrights). kaikki.org asks for citation of Ylonen, "Wiktextract", LREC 2022.
- Redistributable in MIT/BSD repo: no as a file or derived list (share-alike). Facts alone (a hanja spelling) are not copyrightable, but the dataset as a whole carries the license; keep it outside the MIT tree or ship under CC BY-SA.
- Format: one JSON object per line. Useful fields: `word`, `pos`, `forms[{form,tags}]`, `senses[].glosses`.
  - Hanja form of a Korean word (`forms` tag `hanja`): 18,449 distinct words.
    `{"pos":"noun","forms":[{"form":"韓國語","tags":["hanja"]}],"head_templates":[{"name":"ko-noun","args":{"hanja":"韓國語"}}]}` (word `한국어`)
  - Hanja character (`pos":"character"`): 9,933 distinct characters; 2,906 with an `eumhun` form.
    `{"word":"天","pos":"character","forms":[{"form":"하늘 천","roman":"haneul cheon","tags":["eumhun"]}]}`
    `{"word":"犬","pos":"character","forms":[{"form":"개 견","tags":["eumhun"]}]}`
- Weakness: only 2,906 eumhun versus 7,757 in libhangul; Korean-word entries with hanja are mostly nouns, with no frequency. This is the English Wiktionary Korean section only, not ko.wiktionary.

## 4a. Korean Wikisource — `wikisource-ko/`

- URL: https://dumps.wikimedia.org/kowikisource/20261101/kowikisource-20261101-pages-articles.xml.bz2 (149,540,223 bytes; last modified 2026-11-01 15:58 GMT). `books.jsonl` is the pages `tools/build_eonhae.py` keeps from it. Both stay out of git.
- License: Wikisource's text is under CC BY-SA 4.0 (https://ko.wikisource.org/wiki/위키문헌:저작권_정책); the books themselves carry `{{PD-old}}` or `{{PD-old-100}}` on their pages, 類合 (1913) `{{PD-anon-US}}`.
- Pages: `페이지:<scan>/<n>` hold the transcription of one scan page, with `<pagequality level="1">` where nobody has proofread it (most of them). 훈몽자회/<section> adds folio headings and the editors' modern forms: `: {{크게|{{Wikt|身}}}} 몸〮 신(몸 신)`.
- Formats read: 訓蒙字會 `{{윗주|芭|파|크게=예}} || 반쵸파`; 千字文 and 類合 `{{더더더크게|天}} 하ᄂᆞᆯ 텬`; 倭語類解 `;江 / :물강 / :고우 / :○예`; 譯語類解 `;文卷 / :운ᄀᆑᆫ / :ᄝᅳᆫᄀᆑᆫ / :○公事글월`; 詩經諺解 物名 `檉{{더 작게|쳥갯버들}}`. Aligned: a line of Hanja with readings, then its Korean (`:` in most scans).
- The dump also holds 續明義錄諺解 (`續明義錄諺解.djvu`), 諭中外大小臣庶綸音 (`유중외대소신서윤음`, `윤음언해.pdf`) and other 綸音, 正俗諺解, 三綱行實圖, 杜詩諺解 (partial), 才物譜: Korean without a matching Chinese line, or Chinese glossed in Chinese, so they are not parsed yet. 明義錄諺解 is absent.
- `kun-pages.jsonl` is the index pages and printed-reading pages `tools/build_kun_usage.py` keeps from the dump, also out of git. Formats read: `{{윗주|飛|나}}는` (勞動夜學讀本), `{{윗주|하|一}}{{윗주|루|日}}` and `{{아랫주|쇠|鐵}}` (京釜鐵道歌, 地球略論), `{{윗주2|…}}`, `{{ruby|…}}` and running `人(사람)`, `군포내(軍浦場)`. `{{sic|믜|뫼}}` holds the printed letter, then the correction. `--survey FILE` lists every book of the dump with its readings left after the sound test.

## 4b. kaikki.org Middle Korean — `kaikki-middle-korean/`

- URL: https://kaikki.org/dictionary/Middle%20Korean/kaikki.org-dictionary-MiddleKorean.jsonl (2,250,185 bytes, 1,033 entries, 647 with `descendants`; last modified 2026-10-30). robots.txt disallows only `/dictionary/search/*`. License as in section 4.

## 4c. 나무위키 and a Wikisource page — `namu-wiki/`, `wikisource-ko-pages/`

- URLs, read in a browser and saved 2026-11-04 as Playwright accessibility snapshots (rendered text, each Hanja and its gloss in the tree): https://namu.wiki/w/신증유합/상권 (`namu-wiki/yuhap-sang.yml`, last edited 2026-06-19), https://namu.wiki/w/신증유합/하권 (`namu-wiki/yuhap-ha.yml`), https://namu.wiki/w/훈몽자회/상권 (`namu-wiki/hunmong-sang.yml`), https://ko.wikisource.org/wiki/훈몽자회/樹木 (`wikisource-ko-pages/ws-tree.yml`, kept to compare with the dump's page; not read by the tool). Not refetched: save them again by hand, not with a crawler. They stay out of git.
- License: each 나무위키 page states CC BY-NC-SA 2.0 KR (`이 저작물은 CC BY-NC-SA 2.0 KR`). The books are public domain.
- Format: `新增類合` rows of entry number, character, gloss with reading (`314 / 芋 / 토란 우`); 新增類合 has 1,000 entries in 상권 and 2,000 in 하권, numbered 1 to 3000. `훈몽자회/상권` has rows of character and gloss under section headings (`松 / 솔〮 쇼ᇰ`), and the gloss often links to the article on the modern word (`솔` → 소나무).
- Weakness: an anonymously edited wiki's transcription, naming no copy and no scan, with no folios for 新增類合; a few cells carry the editors' notes (`처엄 초 원간 초`, `旦 아ᄎᆞᆷ 단 今音 됴`), and row 2397 has stray markup (`[[偸`). Which printing the 원간 note compares against is uncertain.

## 5. Other sources (reported, not downloaded)

- ᄎᆞ자쎠 / Chocassye (https://find.됬.xyz, by ᄎᆞᆷ괴): robots.txt allows everything; the About page states no licence or terms for its text and says the Hangul texts come from 국립국어원 역사 자료 종합 정비, 석독구결 from kohico.kr (sktot), some from 디지털 장서각. The code is MIT (github.com/chom-kwoy/chocassye). The corpus, github.com/chom-kwoy/chocassye-corpus, has 1,502 files by century (`data/16C/1527_훈몽자회_예산문고본.xml`, `1576_신증유합.xml`, `1583_천자문석봉.xml`, `data/18C/1730_유합(송광사판).xml`, …) and no licence file. Used here only to find books; nothing from it is included.
- 모두의 말뭉치 (https://kli.korean.go.kr/corpus/main/requestMain.do): 국어 역사 자료 말뭉치 2023 v2.0 holds 1,153 Hangul documents from the 15th century to the early 20th; 2024 and 2025 add letters and early 20th-century prints. Needs an account and an application per corpus; approval sends an agreement. The sample application says publishing results needs the institute's approval by official letter. No redistribution terms are visible before approval.
- 21세기 세종계획 historical corpus: listed there only as a notice board; no download found.
- 표준국어대사전 (https://stdict.korean.go.kr/join/copyrightPolicy.do) and 우리말샘 (https://opendict.korean.go.kr/service/copyrightPolicy), read 2026-11-04: both state CC BY-SA 2.0 KR for everything not marked otherwise, and that dictionary examples with a cited source belong to their original holders. Neither page mentions bulk download. Open APIs need a key.
- 한국고전종합DB (https://db.itkc.or.kr): I could not reach its terms page (404) and found no statement of its licence. Not used.
- anamdong/middle_korean_database_viewer: a viewer whose manifest lists 493 files from a corpus named `eon` and `kawasaki`; no licence. Not used.
- hse-linguistics/middle-korean-corpus: a few test sentences of 月印釋譜; no licence. Not used.

- 우리말샘 (https://opendict.korean.go.kr): a third-party summary says content is CC BY-SA (2.0 KR or 3.0; sources conflict) and bulk download needs login. I could not load an official terms page (useGuide returned 404), so this is unverified. A Hugging Face derivative (`hac541309/woori_spring_dict`) is 2.77 GB and labelled `cc-by-sa-3.0` in metadata, `cc-by-sa-2.0` in its text; too large and unverified, not downloaded.
- 표준국어대사전 Open API (https://stdict.korean.go.kr/openapi/openApiInfo.do): needs an API key, paged (max 100 per request, start ≤ 1000); no license text on that page and no bulk download there. FusionIME's file suggests someone obtained the data, but I found no official license statement.
- suminb/hanja (PyPI `hanja` 0.15.1, `hanja/table.yml`, 522 KB): hanja character to hangul reading only, no 訓; LICENSE.txt present, not inspected; superseded by Unihan/libhangul. npm `hanja` (MIT, github.com/rockpicado/hanja): not inspected for data; its README lists unrelated stroke data.
- ibus-hangul, fcitx5-hangul, nabi: use libhangul's `hanja.txt`; no separate dataset. Google Input Tools attribution page says its Korean hanja mapping comes from a BSD `hanja.txt` (libhangul's).

## 6. Microsoft IME Korean hanja dictionary

No Microsoft statement on export or redistribution was found; Microsoft's documentation describes only the IME API (Windows IME escapes) and says the hangul-hanja escape is for the Korean editor and not for other apps. The dictionary is proprietary, shipped in the OS, with no export tool I know of. Treat as not redistributable.
