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

## 5. Other sources (reported, not downloaded)

- 우리말샘 (https://opendict.korean.go.kr): a third-party summary says content is CC BY-SA (2.0 KR or 3.0; sources conflict) and bulk download needs login. I could not load an official terms page (useGuide returned 404), so this is unverified. A Hugging Face derivative (`hac541309/woori_spring_dict`) is 2.77 GB and labelled `cc-by-sa-3.0` in metadata, `cc-by-sa-2.0` in its text; too large and unverified, not downloaded.
- 표준국어대사전 Open API (https://stdict.korean.go.kr/openapi/openApiInfo.do): needs an API key, paged (max 100 per request, start ≤ 1000); no license text on that page and no bulk download there. FusionIME's file suggests someone obtained the data, but I found no official license statement.
- suminb/hanja (PyPI `hanja` 0.15.1, `hanja/table.yml`, 522 KB): hanja character to hangul reading only, no 訓; LICENSE.txt present, not inspected; superseded by Unihan/libhangul. npm `hanja` (MIT, github.com/rockpicado/hanja): not inspected for data; its README lists unrelated stroke data.
- ibus-hangul, fcitx5-hangul, nabi: use libhangul's `hanja.txt`; no separate dataset. Google Input Tools attribution page says its Korean hanja mapping comes from a BSD `hanja.txt` (libhangul's).

## 6. Microsoft IME Korean hanja dictionary

No Microsoft statement on export or redistribution was found; Microsoft's documentation describes only the IME API (Windows IME escapes) and says the hangul-hanja escape is for the Korean editor and not for other apps. The dictionary is proprietary, shipped in the OS, with no export tool I know of. Treat as not redistributable.
