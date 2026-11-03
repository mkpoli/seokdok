# Dictionaries

Tab-separated files loaded by the engine. Lines starting with `#` are
comments; the header line names the columns.

| File | Kind | Source | Licence |
|---|---|---|---|
| `user.tsv`, `user-sino.tsv` | your own entries (not shipped) | you | yours |
| `corpus.tsv` | 訓讀 spellings attested in the 釋讀會 chat | project editors | MIT (this repository) |
| `editorial.tsv` | 訓讀 entries added by the editors, with reasons | project editors | MIT |
| `community.tsv` | 標準國語釋讀辭典 | 釋讀會 members | none stated; see below |
| `gloss.tsv` | 訓讀 candidates from 訓音 glosses (하늘 천 → 하늘 = 天) | libhangul | BSD-3-Clause |
| `sino.tsv` | 音讀 words and characters | libhangul | BSD-3-Clause |

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
  trailing lone consonant (在ㅆ다, 事ㅅ) is always written out. See
  [docs/orthography.md](../docs/orthography.md) §2.
- `pos`: noun pron num bound adv det interj verb adj, or the Korean names
  (명사, 동사 …).
- `class`: inflection class, guessed when blank: `reg`, `b` (ㅂ), `d` (ㄷ),
  `s` (ㅅ), `h` (ㅎ), `reu` (르), `reo` (러), `ani` (아니다).
- `source`: `user`, `corpus`, `editorial`, `community`, `gloss`.
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
  compiled by 釋讀會 members from June 2022, fetched 2026-11-03. No licence is
  stated. Publishing this repository with `community.tsv` requires the
  editors' consent. Seven rows are skipped with a warning (compound verbs
  such as 奪아去다, rows with lemma/written mismatches).
- **釋讀會 chat corpus** (t.me/pimsdh), export to 2026-10-29: used for counts
  and for choosing `corpus.tsv` spellings. The messages themselves are not in
  this repository.
