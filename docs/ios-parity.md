# The iOS keyboard against the system's

What the 釋讀 keyboard does next to Apple's Korean keyboard, measured by running one script against both in the same text field. Numbers come from GitHub's macOS runners, on simulators: iOS 18.5 (Xcode 16.4) and iOS 26.5 (Xcode 26.6), on iPhone 16 Pro at 402 x 874, with iPhone SE (3rd generation) and iPhone 16 Pro Max on iOS 26.5 for sizes. Apple's come from runs [42007197459](https://github.com/mkpoli/seokdok/actions/runs/42007197459) and [42028393000](https://github.com/mkpoli/seokdok/actions/runs/42028393000), ours from [42073192377](https://github.com/mkpoli/seokdok/actions/runs/42073192377).

The script drove touches through XCTest, logged every change in the field from inside the field's app, and recorded each test with `simctl io recordVideo`. Screenshots are 3x. Recordings run at about 30 frames a second, so times read from them hold to about 30 ms. Times are relative to a marker touch in the same recording, so they compare the two keyboards and are not absolute. A simulator's times are not a phone's.

## Geometry

| | Apple iOS 26.5 | Apple iOS 18.5 | Ours iOS 26.5 | Ours iOS 18.5 |
| --- | --- | --- | --- | --- |
| Keyboard frame, 402 pt wide | 335 pt | 336 pt | 353 pt, see the limits | 336 pt |
| Keyboard frame, 440 pt wide | 345 pt | | 363 pt | |
| Keyboard frame, 375 pt wide | 260 pt | | 277 pt | |
| Landscape frame, 874 pt wide | 208 pt | 219 pt | 208 pt | 219 pt |
| Letter key at 402 pt, from pixels | 33 x 43 | 34 x 43 | 32.3 x 43 | 34.7 x 43 |
| First key's left edge | 7 pt | 3 pt | 7.3 pt | 2.7 pt |
| Row pitch | 54 pt (56 at 440 pt) | 54 pt | same | same |
| First row's top | 591 | 591 | 591 | 591 |
| Corner radius | about 8 pt | about 5 pt | 8 pt | 5 pt |
| Fill, light | white, all keys | white; function keys 171,177,186 | same | same |
| Fill, dark | 61, all keys | | 61 | |
| Pressed, dark | 125 | | 125 | |
| Shadow | none | 1 pt | none | 1 pt |
| Number pad frame | 308 pt | 291 pt | 308 pt | 291 pt |

## Behaviour

| Scenario | Apple | Ours, before | Ours, now |
| --- | --- | --- | --- |
| 안녕하세요 key by key | the same text at each of 12 steps | same | same |
| Tap Delete on 안녕 | 안녀, 안ㄴ, 안, 아, ㅇ | same | same |
| Held Delete: first repeat, on the runs' clock | 0.55 s | 0.45 s | 0.55 s |
| Held Delete: repeats | 10 a second | 12.5 | 10 |
| Held Delete: after 20 repeats | a word, 6 a second | no change | a word, 6 a second |
| Held Delete, composing | a syllable at a time | a letter at a time | a syllable at a time |
| Shift, then two letters | ㅃㅂ | same | same |
| Two quick taps of Shift | off again; no lock in Korean | locks | locks, see below |
| 123 | changes the keys as the finger lands | on the lift | as the finger lands |
| Slide from 123 to a digit | types it, letters return | nothing | types it, letters return |
| Space or an apostrophe after a symbol | letters return | stays on symbols | letters return |
| Return label | by field, a word before iOS 26, a symbol from it | 改行 | by field, in Hanja before iOS 26 |
| Field wants text and has none | Return greyed | no | greyed |
| Two Spaces after a word or digit | `. `, at gaps up to at least 2 s | two spaces | `. ` |
| Latin capitals by the field's trait | yes | no | yes |
| Numbers and punctuation field | symbols first | letters first | symbols first |
| Number pad | 4 rows, letters under digits, no Return | 5 rows, Return | as Apple's |
| URL field | `.` `/` `.com`, no space bar | `.` `/` and a space bar | as Apple's |
| Quote key | curls, `‘` at a start | straight | curls in prose fields |
| Tap in a gap between keys | nearest key | same | same |
| Tap in a middle row's margin | the end key | nothing | the end key |
| Shake | undo alert | same | same |
| Long press | a tray, on doubles, ㅒ ㅖ and some symbols | pop-up only | a tray, and on every key with a corner letter |
| Space held, start of the fade | 514-518 ms | 672 ms | 516-520 ms |
| Slow 32 pt slide over digits | 11 characters | 3 | 7 |
| Slide up on the trackpad | moves between lines | no | no, see the limits |
| Tap right after the trackpad | types | types | types |
| Pop-up appears | 67-128 ms | 40-65 ms | 43-59 ms |
| Pop-up after a 0.35 s press | gone at 400-425 ms | 360 ms | 395 ms |

## Performance

| | Apple | Ours |
| --- | --- | --- |
| Field update after a key's lift, marker-relative, single keys | 60-71 ms | 55-69 ms |
| First text after focus, tapping from the start, 3 launches | 0.42-0.65 s | 0.42-0.57 s |
| Footprint after typing | host 37-39 MB, `kbd` 26-28 MB, `InputUI` 29-38 MB, all system processes | `SeokdokKeyboard` 57-62 MB, Debug, no test runner |
| Footprint over six launches of the host | | before: +3 MB each. Now 60-62 MB on iOS 26.5, 57-59 MB on iOS 18.5 |

The extension peaked at 62 MB here, above the 56.8 MB the README recorded and inside the published estimates of 48-60 MB at the top end; those figures are not Apple's, and a simulator enforces no limit. With XCTest attached over a long session it reached 103-113 MB (before: 362-408 MB).

**Not measured.** Frame drops while typing fast, and per-key latency in bursts: multi-finger sequences do not keep their spacing in XCTest's event synthesizer, so bursts arrived faster than scheduled. Every key of a 37-key burst did reach the field in order on both keyboards.

## What a custom keyboard cannot do

Quotes are from Apple's [App Extension Programming Guide, Custom Keyboard](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html), read 2026-11-05.

| Apple's keyboard | Limit | Closest here |
| --- | --- | --- |
| Pop-ups above the keyboard over the top row | "it is not possible to display key artwork above the top edge of a custom keyboard's primary view" | The pop-up stops at the edge, about 12 pt short; the bar leaves it room |
| Dictation | "no access to the device microphone, so dictation input is not possible" | The system's own mic key shows under the keys |
| Secure and phone fields | "the system temporarily replaces your custom keyboard with the system keyboard" | That |
| Settings for capitals, the period shortcut, caps lock | "does not have access to most of the general keyboard settings" | Field traits decide; the shortcut is always on |
| Selection, inline corrections | "it cannot select text", "cannot offer inline autocorrection controls near the insertion point" | None |
| Keyboard Feedback switches | Apple does not say; see `ios/README.md` | On with Full Access |

Two more were seen in these runs, and Apple's documents do not mention them:

- On iOS 26 a custom keyboard's frame is 18 pt above its view: the system adds 92 pt to the height it is given and uses 74. A height of 243 pt matches Apple's 335 pt with the keys in the same place, but leaves the bar 26 pt and a top-row pop-up 25 pt, so this keeps 261 pt. Changing it is a matter of `KeyLayout.firstRow` and `barHeight`, for the owner to decide.
- `documentContextBeforeInput` showed no line break in these fields, and `adjustTextPosition` has no up and down, so the trackpad stays on its line.

## Left different on purpose

- Return in 釋 and 混 confirms before it acts, because that is where a conversion or a raw word is kept.
- Two quick taps of Shift lock it, which runs of old letters need; Apple's Korean keyboard has no lock.
- Email and URL fields type Latin, where Apple's stay on Hangul with `@` and `.com` beside them.
- Marked text carries the composition, so some apps underline it. Apple's composes without marks.
- `₩` on the first symbol page where a US-region simulator shows `$`, and `·` on the second where it shows `•`.

## Better than Apple's

- Every key with a corner letter offers it on a long press, so ㅿ, ㆁ, ㆆ, ㆍ and the sibilants need no Shift.
- Conversion, candidates, segments and learning, none of which Apple's has.
- The pop-up appears about 30 ms sooner in the recordings.

## To try on a phone

A simulator cannot answer these.

1. Two thumbs at full speed. XCTest's multi-touch sends a stray tap for each overlapping finger, on both keyboards, so rollover is unmeasured. Letters must come in order, with Space too.
2. The click and haptic on a key, on Delete's repeats and at the trackpad's start, and whether repeats should sound.
3. Thumb comfort at the bottom row, and on iOS 26 the 18 pt taller keyboard against the room it gives the bar and pop-ups.
4. Move through ten apps with the keyboard up, then look in Settings → Privacy & Security → Analytics & Improvements → Analytics Data for `JetsamEvent` entries. Watch for a blank flash in the keys as apps change.
5. Trackpad speed on Hangul text.
6. Long-press trays at each edge, on a key under a resting thumb.
7. Globe to another keyboard while composing: the text should stay where it was typed.

## Running it again

The harness lived on a temporary branch, since CI stays tests, type check and build. Its last commit is `1fad15b` until the branch is deleted, with `ios/UITests/ParityTests.swift`, `ios/ParityHost/`, `tools/ios-parity/run.sh` and `.github/workflows/ios-parity.yml`.
