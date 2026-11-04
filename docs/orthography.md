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
| 末音添記, 吐 | The rest of the word, in Hangul | 超**무**, 讀**었다**, 天**을** | 너무, 읽었다, 하늘을 |

The Hangul has two names. 末音添記 (말음첨기, the chat's term, 35 uses) is the last sound written after a word read by meaning, a practice named from 鄕札 spellings such as 夜音 for 밤. 吐 (40 uses) is a grammatical particle or ending. Members compare the whole with Japanese 送り仮名 (오쿠리가나).

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
此런 (50), 何슨 (35), 尙려 (오히려), 超무 (너무), 過무 (너무). 車魄 notes that the group has
yet to settle these ("副詞類도 언젠가 一番에 釋讀 表記를 考案해야겠죠",
2026-10-28).

### 6.4 하다

seokdok writes 하다 in Hangul, after a Sino-Korean word or on its own
(安寧하세요, 했다), and lists 爲 as the next candidate (安寧爲세요, 爲ㅆ다).
The `hada_kun` option makes 爲 the first choice. The community dictionary
writes 하다 on its own as 爲다.

### 6.5 Other amounts of Hangul after the Hanja

An entry gives one written form (超무, 初음, 近다), and the same word can be
written with less or more Hangul after the Hanja. For a noun, adverb or
pronoun written with a single Hanja, the input method lists these forms behind
the dictionary form, in this order:

1. the last syllable (何게);
2. every syllable but the first (尙히려);
3. the last letter alone, as a jamo (超ㅜ, 初ㅁ, 何ㅔ);
4. the Hanja alone (超, 初).

A form equal to an earlier one is listed once. A particle follows whichever
form is chosen. The forms are made from the entry when candidates are listed,
and none of them is a dictionary entry or counted as one. They are
*(constructed)*.

| Word | Forms, in order |
|---|---|
| 너무 | 超무, 超ㅜ, 超 |
| 처음 | 初음, 初ㅁ, 初 |
| 처음에 | 初음에, 初ㅁ에, 初에 |
| 어떻게 | 何떻게, 何게, 何ㅔ, 何 |
| 말씀 | 言씀, 言ㅁ, 言 |
| 오히려 | 尙려, 尙히려, 尙ㅕ, 尙 |

A one-syllable word has the letter and the Hanja alone (我ㅏ, 我). An
inflected word, when the Hanja stand for two syllables or more, writes the
word's own syllables after the Hanja, from the last covered one or from the
second: 가까웠다 is 近웠다 or 近까웠다, 가깝고 is 近고 or 近깝고, and 마셔라 is
飮라 or 飮셔라. These forms are the same whatever the style options.

The reader (釋讀文 → 한글) reads these forms back: 超ㅜ is 너무, 初ㅁ에 is
처음에, 近까웠다 is 가까웠다. It reads the Hanja alone by sound, as 超 = 초. A
form that is also a dictionary form of another word is read as that word:
何게 is 어드럭하게, 言ㅁ is 말함 and 初ㅁ is 숫함.

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

A run that holds old letters is looked up by the modern letters they became,
and by the classes of pronounced spellings in 8.3. At most 24 readings of such
a run are converted (12 for a run typed in modern letters), ranked by the
conversion's cost plus a penalty for each rewrite, and what was typed stays one
of the candidates. The folds of old letters:

| Old letter | Modern letter | Example in the chat |
|---|---|---|
| ㆍ (U+119E) | ㅏ or ㅡ; as a guess ㅗ or ㅜ | ᄉᆞ랑 = 사랑, ᄂᆞᆫ = 는, ᄒᆞᆰ = 흙, 아ᄆᆞ리 = 아무리 (3), ᄃᆞᆯ = 돌 (16) |
| ᆡ (U+11A1, ㆍ+ㅣ) | ㅐ; as a guess ㅔ | ᄇᆡ = 배, ᄒᆡ = 해 (171 in all) |
| ㅿ (U+1140) | ㅅ, or ㅇ as a guess | ᅀᅵ = 시 (71), 거ᅀᅵ = 거의 |
| ㆁ (U+114C) | ㅇ | ᅌᅵ = 이 (22) |
| ㅸ (U+112B) | ㅂ; as a guess ㅇ | 25 |
| ᄭ (U+112D), ᄢ | ㄲ | 28 |
| ᄯ (U+112F), ᄠ (U+1120), ᄣ | ㄸ | ᄯᅩ = 또 (41), ᄠᅳᆮ = 뜻 (8) |
| ᄲ (U+1132), ᄤ | ㅃ | 8 |
| ᄶ (U+1136), ᄧ (U+1127), ᄦ | ㅉ | ᄶ 1, ᄧ 2 |
| ᄡ (U+1121), ᄥ | ㅆ | 14 |
| final ㆁ (U+11F0), final ㅿ (U+11EB) | final ㅇ, final ㅅ | ᄉᆡᇰ = 생 (41), 6 |

The folds are the usual correspondences between Middle Korean spelling and
today's, applied letter by letter; no single reference was used, and the
chat spellings above are the evidence for each. They serve only to find the
word. ㅿ also became ㅇ in many words (겨울, 마음, 거의), tried as a guess. A letter
outside the table (such as ᄓ) gives no reading, and the run stays as typed.

Examples: ᄉᆞ랑 offers 愛, 사ᄅᆞᆷ offers 人, 말ᄊᆞᆷ offers 言씀, 있ᄂᆞᆫ
offers 有ᄂᆞᆫ and 在ㅆᄂᆞᆫ. When the old letters are all function syllables of the
table in 8.1, the candidates that keep them as typed come first: ᄒᆞ고
stays first, then 爲고, and 추가ᄒᆞᆯ까요 gives 追加ᄒᆞᆯ까요.

### 8.3 Pronounced spellings

Spellings of the era were often written as pronounced, with modern syllables.
Each class below rewrites one or two neighbouring syllables into the spelling
of today, so it combines with the folds above and with the others (사ᄅᆞ미 is
사람이: a fold and a carry). A class gives a reading only; the engine converts
it, and a guess counts only when words cover all of it. The evidence column
names where each class comes from: counts are occurrences of those spellings
in the chat; "usual correspondence" marks a rewrite that rests on the known
history of the spelling and is not attested there.

| Class | Rewrites | Examples | Evidence |
|---|---|---|---|
| Palatalisation | ㄷ ㅌ before ㅑ ㅕ ㅛ become ㅈ ㅊ before ㅏ ㅓ ㅗ | 텬하 = 천하, 뎌 = 저, 됴 = 조, 댜 = 자, 텨 = 처 | chat: 20, 9, 25, 1, 7 |
| Palatalisation of 디 티 | 디 티 become 지 치 | 디나거ᄂᆞᆯ = 지나거늘 | chat: ᄒᆞ디, 디만, 엇디 and others, a handful; tried only for words left unconverted, as loanwords have them |
| Glide after a sibilant | ㅅ ㅈ ㅊ before ㅑ ㅕ ㅛ ㅠ lose the glide | 셔울 = 서울, 죠션 = 조선, 슈작 = 수작 | chat: 4, 1, 1; the rest usual correspondence |
| Final ㅅ as ㅆ | a final ㅅ after ㅏ ㅐ ㅓ ㅕ ㅣ is ㅆ (not in 것) | 갓다 = 갔다, 잇다 = 있다, 엇다 = 었다 | chat: 3, 6, 2 |
| Seven finals | ㅂ for ㅄ ㅍ; ㅅ ㄷ for ㅈ ㅊ ㅌ ㅆ; for old input also ㄱ ㄴ ㄹ for their clusters | 업시 = 없이, 업ᄂᆞᆫ = 없는, 놉다 = 높다, 갑ᄋᆞᆫ = 값은 | chat: 4, 5, 3, 1; ㅅ ㄷ ㄱ ㄴ ㄹ rewrites are usual correspondence |
| ㅎ fusion | ㅎ and a following ㄷ ㄱ ㅈ ㅂ written ㅌ ㅋ ㅊ ㅍ | 됴타 = 좋다, 됴코 = 좋고, 만타 = 많다, 안코 = 않고 | chat: 됴타 2, 됴쿠 1, 됴쿶 1; 만타, 안코 usual correspondence |
| Carry | a final written as the onset of the next syllable, with ㅡ also read ㅏ | 사르미 = 사람이, 그르슬 = 그릇을, 노ᄑᆞᆫ = 높은, 됴흔 = 좋은 | chat: 노ᄑᆞᆫ 2, 아ᄌᆞ미 1; the rest usual correspondence |
| Initial ㄴ | ㄴ for ㅇ before ㅣ ㅑ ㅕ ㅛ ㅠ at the start | 니르다 = 이르다, 닙다 = 입다, 녀름 = 여름 | chat: 니르 5, 녀름 1 |
| ㅡ after a labial | ㅁ ㅂ ㅍ with ㅡ become ㅜ | 믈 = 물, 블 = 불, 플 = 풀 | usual correspondence; the chat's 므레 (2) is 물에 |

The strong classes, palatalisation, final ㅅ as ㅆ and ㅎ fusion at the end of a
word, are tried for every run that shows them, converted or not, and give their
readings behind the first candidate (잇다 stays 續다, with 有다 behind it; 만타
stays 萬朶, with 多다 behind it). The others are tried only for a run of at most
six syllables that conversion left as typed, which keeps their cost off words
the dictionary already reads: a run with no mark of any class costs nothing.
Words such as 곳 (處) and 믈 (typed first) therefore keep their reading.

A syllable that modern words do not use (ㄷ ㅌ before ㅑ ㅛ, or before ㅕ with a
final other than ㅆ) puts the converted reading first and what was typed second,
when the reading is two syllables or more and every word of it is an entry with
a count of at least three (the chat's count, or FusionIME's frequency where the
chat has none): 텬하 gives 天下, 됴타 gives 好다. A reading of one syllable or
from a rare entry is offered behind what was typed. 뎌 and 텨 alone are the
endings of 견뎌 and 버텨, so they are not in that set. In the 20000 most
frequent Hangul tokens of the chat this changes the first candidate of 10 words
(둏도다, 됴션, 텬동, 됴타, 텬문, 됴션인, 됴하셔, 뎡쥬, 됴흔, 텬운); 둏, 뎍, 댜, 됴한, 됴츙 and
뎜다 keep what was typed first. All are spellings of the era; none is a
loanword or a name. Guesses add to the candidates of a run: every modern
candidate and the literal input stay selectable.

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
- Historical books, read through Korean Wikisource's transcriptions (dump of
  2026-11-01, CC BY-SA 4.0, https://ko.wikisource.org/): 訓蒙字會 (1527; 1613
  reprint), 光州千字文 (1575), 石峯千字文 (1583; later printing), 類合 (1913 printing),
  倭語類解 (18th century), 譯語類解 (1690), and the 諺解 listed in `dict/README.md`: the
  citations in `dict/eonhae.tsv`; books of 1882-1945 that print a native reading beside
  Hanja, cited in `dict/kun-attested.tsv` with every use in `data/kun-usage.tsv`. Most of these pages are unproofread on Wikisource,
  so a row can carry a transcriber's slip; each note names the page to check.
- 新增類合 (1576) and the first volume of 訓蒙字會, read through 나무위키's transcriptions
  (saved 2026-11-04, CC BY-NC-SA 2.0 KR, https://namu.wiki/w/신증유합/상권,
  https://namu.wiki/w/신증유합/하권, https://namu.wiki/w/훈몽자회/상권): an anonymously
  edited wiki naming no copy of the book; `dict/README.md` gives the check.
