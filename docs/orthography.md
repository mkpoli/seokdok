# 釋讀 orthography: rules and variants

This document defines the spelling that seokdok produces. It describes how
members of 釋讀會 (풀어읽기모임) write, so that an input method can reproduce
it. The group has no fixed rules (「作文 規則 無，意味傳達 第一」, group rules),
so where members differ, the variants are kept as options instead of choosing
one for everyone.

Every example below is attested in the group's chat or the community
dictionary unless it is marked *(constructed)*. Counts are from the chat export
of 2022-06-17 to 2026-10-29 (see [Sources](#sources)).

## 1. Three kinds of material

A 釋讀 sentence is Korean grammar written with three kinds of material.

| Kind | What it writes | Example | Reading |
|---|---|---|---|
| 音讀 | A Sino-Korean word, in Hanja by sound | 規則, 歡迎 | 규칙, 환영 |
| 訓讀 | A native word or stem, in Hanja by meaning | 讀, 天, 有 | 읽-, 하늘, 있- |
| 送韓字 | The rest of the word, in Hangul | 讀**었다**, 天**을** | 읽었다, 하늘을 |

Particles, endings and words with no chosen character stay in Hangul.
Sino-Korean verbs keep 하다 in Hangul by default (安寧하세요, 153 times), with
爲 as an option (安寧爲세요, 安寧為세요; §6.4).

## 2. The 訓讀 entry

Each native word has a dictionary entry: its Hangul lemma and its written
form, e.g. 읽다 = 讀다. The written form says how much of the word the Hanja
cover:

- **Whole stem** (the default): 읽다 = 讀다, 다르다 = 異다, 하늘 = 天.
- **Part of the stem**, with the rest repeated in Hangul: 가두다 = 囚두다,
  모르다 = 䚳르다, 처음 = 初음, 반갑다 = 歡갑다, 오랜만 = 久ㄴ만. The community
  dictionary uses this for words that would otherwise be confusable: "語幹
  마지막 音節만 내어 쓰자" (멜캬 金, 2022-06-20).
- **Whole stem with its final consonant written out** (末音添記): 있다 =
  在ㅆ다, 것 = 事ㅅ, 그렇다 = 然ㅎ다, 좀 = 少ㅁ.

- **Two characters in a compound verb**, with the Hangul between them:
  빼앗아가다 = 奪아去다, 일어나다 = 起어出다 (community dictionary). Only the
  last part inflects: 奪아去ㅆ다.

One word may have several entries (있다 = 有다, 在다, 在ㅆ다), and one
character may serve several words (見 = 보다, 보이다 as 見이다).

## 3. The core rule

Write the Hanja for the part of the word the entry covers, then write in
Hangul everything else that is pronounced.

To make "everything else" exact, both the word and the stem are split into
jamo, with compound vowels split into their parts (ㅘ = ㅗ+ㅏ, ㅕ = ㅣ+ㅓ,
ㅐ = ㅏ+ㅣ, ㅔ = ㅓ+ㅣ) and compound finals into theirs (ㄻ = ㄹ+ㅁ). The
Hangul part is what remains after the covered jamo are removed, regrouped
into syllables. Anything left that does not begin with an initial consonant is
written as a bare letter:

| Word | Stem | Remainder | Written | Count |
|---|---|---|---|---|
| 다른 | 다르 | ㄴ | 異ㄴ | 49 |
| 전 (저+는) | 저 | ㄴ | 自ㄴ | 54 |
| 내 (나+의) | 나 | ㅣ | 我ㅣ | 21 |
| 좀 | 조 | ㅁ | 少ㅁ | 17 |
| 그렇네요 | 그러 | ㅎ네요 | 然ㅎ네요 | 46 |
| 재밌네요 | 재미 | ㅆ네요 | 滋味ㅆ네요 | 13 |
| 갔다 | 가 | ㅆ다 | 去ㅆ다 | community |

## 4. Inflection

Stems change shape when endings attach: vowels contract (보+아 → 봐), and
irregular stems alternate (춥- → 추워, 다르- → 달라, 듣- → 들어, 알- → 아는).
How the Hangul part handles this is the main point on which writers differ.
seokdok offers two styles.

### 4.1 Surface style (default)

The Hanja stand for the stem *as pronounced in this word*, whatever shape it
takes; the Hangul part is the rest of the pronounced word. This is the style
of the paradigms in 標準國語釋讀辭典. (爲 for 하다 appears here when the 爲
entry is chosen; see §6.4.)

| Word | Written |
|---|---|
| 봤다 | 見ㅆ다 |
| 했다 | 爲ㅆ다 |
| 달랐다 | 異ㅆ다 |
| 가까웠다 | 近웠다 |
| 걸었다 | 步었다 |
| 몰랐다 | 䚳랐다 |
| 아는 | 知는 |
| 그래서 | 然서 |
| 해요 | 爲요 |

### 4.2 Morpheme style

The Hanja stand for the stem's dictionary shape; the ending is written as it
attaches to that shape, without contraction. The reader applies the
inflection rules. This style shows the morphemes, and is the one mkpoli chose
for ㅂ stems ("ㅂ->우 so remove 우", 2026-10-28).

| Word | Written |
|---|---|
| 봤다 | 見았다 (16 times as 見았) |
| 어려워요 | 難어요 |
| 쉬워서 | 易어서 |
| 가셨다 | 去셨다 |
| 썼다 | 書었다 *(constructed)*; 書어도 is attested |

Both styles agree whenever the stem keeps its shape: 讀었다, 有네요, 好은.

### 4.3 Leftover vowels (surface style)

When a stem vowel contracts with the ending (보+아 → 봐, 마시+어 → 마셔), the
surface style can either let the Hanja take the whole merged syllable or write
the leftover vowel as a letter.

| Option | 봐라 | 마셔라 | 와서 |
|---|---|---|---|
| Absorb (default) | 見라 | 飮라 | 來서 |
| Letter | 見ㅏ라 | 飮ㅓ라 | 來ㅏ서 |

Both occur: 見라 and 飮ㅓ라 in the community dictionary, 見ㅏ가 and 來ㅏ서 in
the chat.

## 5. Writing the final consonant (末音添記)

Writing out a stem's final consonant after its Hanja is the most debated
choice in the group. 멜캬 金 dislikes it ("末音添記 되게 嫌어하는 便",
2022-06-18); 車魄 favours it ("末音添記를 好아해서", 2022-06-24). seokdok
handles it at two levels.

- **Per entry.** An entry written with the consonant (在ㅆ다, 事ㅅ, 然ㅎ다)
  always writes it: 在ㅆ는, 事ㅅ을, 然ㅎ네요.
- **Everywhere** (option `coda_always`). Every whole-stem entry whose stem ends
  in a single final consonant writes it: 有ㅆ네요, 知ㄹ고 *(constructed)*. Stems
  ending in a compound final (읽-, 않-) are left alone.

A written-out consonant before an ending that begins with ㅇ can be written as
a letter or moved into the next syllable (option `resyllabify`):

| Default | Resyllabified |
|---|---|
| 在ㅆ으면 | 在쓰면 |
| 在ㅆ어서 | 在써서 |

Both appear in mkpoli's messages of 2026-10-28/29.

## 6. Nominals and particles

### 6.1 Particles

Particles stay in Hangul and follow the pronounced word: 天을, 漢字를,
規則이. After a vowel-final pronoun, contracted particles are bare letters:
自ㄴ (전), 自ㄹ (절), 我ㅣ (내), 我ㅣ가 (내가).

### 6.2 Words with partial cover

A noun entry that covers only part of the word writes the rest in Hangul
before the particle: 初음에, 久ㄴ만에, 言씀을.

### 6.3 Adverbs and determiners

Adverbs, determiners and pronouns take entries like nouns: 何떻게 (65),
此런 (50), 何슨 (35), 尙려 (오히려), 超무 (너무). 車魄 notes that the group has
yet to settle these ("副詞類도 언젠가 一番에 釋讀 表記를 考案해야겠죠",
2026-10-28).

### 6.4 하다

seokdok writes 하다 in Hangul, after a Sino-Korean word or on its own
(安寧하세요, 했다), and lists 爲 as the next candidate (安寧爲세요, 爲ㅆ다).
The `hada_kun` option makes 爲 the first choice. The community dictionary
writes 하다 on its own as 爲다.

## 7. Options summary

| Option | Values | Default | Effect |
|---|---|---|---|
| `style` | surface, morpheme | surface | §4 |
| `fragment` | absorb, letter | absorb | §4.3 |
| `coda_always` | true, false | false | §5 |
| `resyllabify` | true, false | false | §5 |
| `hada_kun` | true, false | false | §6.4 |
| `old_ha` | true, false | false | §8.1 |

Whatever the options, the input method lists the other spellings as
alternative candidates, so a writer can switch for a single word.

## 8. Old spellings

Some members write function syllables in old Hangul: ᄒᆞ고, 追加ᄒᆞᆯ까요,
工夫ᄒᆞᄂᆞᆫ, 字ᄅᆞᆯ. seokdok offers these spellings and accepts old letters as
input. Old letters are conjoining jamo (U+1100 block), the ones the old-letter
keys type.

### 8.1 Offered

In the Hangul after Hanja, and in every inflection of the light verb 하다,
ten syllables have an old spelling. The counts are occurrences of each old
syllable in the chat export (telegram.jsonl, 2022-06-17 to 2026-11-03).

| Modern | Old | Count |
|---|---|---|
| 하 | ᄒᆞ | 247 |
| 한 | ᄒᆞᆫ | 43 |
| 할 | ᄒᆞᆯ | 14 |
| 함 | ᄒᆞᆷ | 5 |
| 합 | ᄒᆞᆸ | 2 |
| 해 | ᄒᆡ | 19 |
| 는 | ᄂᆞᆫ | 58 |
| 를 | ᄅᆞᆯ | 41 |
| 은 | ᄋᆞᆫ | 10 |
| 을 | ᄋᆞᆯ | 14 |

The table holds only spellings the chat uses. 했 has no old spelling there
(ᄒᆡᆻ occurs 0 times), so it stays as it is.

The old spelling of the best candidate comes third, or last in a shorter
list:

| Input | Candidates |
|---|---|
| 하고 | 하고, 爲고, ᄒᆞ고 |
| 추가할까요 | 追加할까요, 追加爲ㄹ까요, 追加ᄒᆞᆯ까요 |
| 공부하는 | 工夫하는, 工部하는, 工夫ᄒᆞᄂᆞᆫ |
| 이것을 | 此것을, 此事ㅅ을, 此것ᄋᆞᆯ |
| 字를 | 字를, 字㔹, 字ᄅᆞᆯ |

In 國漢文混用 (`sino_only`) the rule applies too: 공부하셨다
offers 工夫ᄒᆞ셨다.

With the option `old_ha` (setting 하 → ᄒᆞ in the settings screens), the
old spelling of 하 한 할 함 합 해 comes first and the modern spelling right
behind it: 하고 gives ᄒᆞ고, 하고; 공부한 gives 工夫ᄒᆞᆫ, 工夫한. 는, 를, 은 and 을
keep their places.

### 8.2 Accepted

A run that holds old letters is looked up by the modern letters they became.
At most eight readings of a run are converted, and what was typed stays one of
the candidates. The folds:

| Old letter | Modern letter | Example in the chat |
|---|---|---|
| ㆍ (U+119E) | ㅏ or ㅡ, both tried | ᄉᆞ랑 = 사랑, ᄂᆞᆫ = 는, ᄒᆞᆰ = 흙 |
| ᆡ (U+11A1, ㆍ+ㅣ) | ㅐ | ᄇᆡ = 배, ᄒᆡ = 해 (171 in all) |
| ㅿ (U+1140) | ㅅ | ᅀᅵ = 시 (71) |
| ㆁ (U+114C) | ㅇ | ᅌᅵ = 이 (22) |
| ㅸ (U+112B) | ㅂ | 25 |
| ᄭ (U+112D) | ㄲ | 28 |
| ᄯ (U+112F) | ㄸ | ᄯᅩ = 또 (41) |
| ᄲ (U+1132) | ㅃ | 8 |
| ᄶ (U+1136) | ㅉ | 1 |
| ᄡ (U+1121) | ㅆ | 14 |
| final ㆁ (U+11F0), final ㅿ (U+11EB) | final ㅇ, final ㅅ | ᄉᆡᇰ = 생 (41), 6 |

The folds are the usual correspondences between Middle Korean spelling and
today's, applied letter by letter; no single reference was used, and the
chat spellings above are the evidence for each. They serve only to find the
word. ㅿ also became ㅇ in some words (겨울), which is not tried. A letter
outside the table (such as ᄓ) gives no reading, and the run stays as typed.

Examples: ᄉᆞ랑 offers 愛, 사ᄅᆞᆷ offers 人, 말ᄊᆞᆷ offers 言씀, 있ᄂᆞᆫ
offers 有ᄂᆞᆫ and 在ㅆᄂᆞᆫ. When the old letters are all function syllables of the
table in 8.1, the candidates that keep them as typed come first: ᄒᆞ고
stays first, then 爲고, and 추가ᄒᆞᆯ까요 gives 追加ᄒᆞᆯ까요.

Spellings of the era that are made of modern syllables are tried when a word
was not converted as typed, and their results come after what was typed:

| Typed | Read as | Count |
|---|---|---|
| ㄷ, ㅌ before ㅑ ㅕ ㅛ: 텬하, 뎌, 됴, 댜, 텨 | ㅈ, ㅊ before ㅏ ㅓ ㅗ: 천하, 저, 조, 자, 처 | 20, 9, 25, 1, 7 |
| a final ㅅ after ㅏ ㅐ ㅓ ㅕ ㅣ: 갓다, 잇다, 엇다 | a final ㅆ: 갔다, 있다, 었다 | 3, 6, 2 |

텬하 offers 天下. 됴타 is read as 조타 (操舵); the dictionary has no 좋다 for
it, so 好다 is not offered. 샤 and 셔 as 사 and 서 are not read, because no
source for them was found.

`read` turns the function syllables of 8.1 back into modern ones (工夫ᄒᆞᄂᆞᆫ reads
공부하는) and leaves other old letters as they are.

## 9. Not yet covered

- **Archaic Hangul beyond §8** (ᄒᆞ옵니다, 讀了ᄒᆞ엿ᄉᆞ옵니다): only the ten
  syllables of §8.1 are written in old letters. Other old letters are
  accepted as input but not offered.
- **Gugyeol letters for bare jamo** (用ㅂ니다 → 用邑니다) were proposed once
  (2022-09-18) and are not supported.
- **Classical Chinese word order** (此處 頗有興趣) is outside the scope of an
  input method.

## Sources

- 釋讀會 Telegram group (t.me/pimsdh), chat export 2022-06-17 to 2026-10-29
  (49,880 text messages; private, not redistributed). Quoted messages:
  멜캬 金, message 1101, 2022-06-20 (dictionary principles 1–6); 멜캬 金,
  message 321, 2022-06-18; 車魄, message 2795, 2022-06-24; 車魄, 2026-10-28
  (adverbs); gugyeol proposal, message 11576, 2022-09-18 (deleted account).
- 標準國語釋讀辭典, 釋讀會 community dictionary, Google Sheets,
  https://docs.google.com/spreadsheets/d/1J7bPuQ4buOM17hvLR6UHLfcrERfYhaVVj1H-3szh7J0
  (fetched 2026-11-03; no licence stated, about 800 entries with paradigms).
- mkpoli, conversation with ChatGPT "Korean readings", 2026-10-28 to
  2026-11-02 (exported 2026-11-03): the 在ㅆ convention and the ㅂ-stem
  morpheme spelling. The model's own suggestions in that conversation are not
  used as evidence.
- libhangul `data/hanja/hanja.txt`, BSD-3-Clause, Copyright (c) 2005, 2006
  Choe Hwanjin: Sino-Korean words and 訓音 glosses.
