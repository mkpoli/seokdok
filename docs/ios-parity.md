# The iOS keyboard against the system's

What the 釋讀 keyboard does next to Apple's Korean keyboard, measured by running one script against both in the same text field. "Before" is the keyboard as of commit `4cd3e69`; "after" is the same with the fixes from PRs #90 to #106, which were open when this was written. 釋 and 混 are the input modes that convert to Hanja, 한 the Hangul-only one, A the Latin one; the bar is the candidate bar over the keys.

Numbers come from simulators on GitHub's macOS runners: iOS 18.5 (Xcode 16.4) and iOS 26.5 (Xcode 26.6), iPhone 16 Pro at 402 x 874, with iPhone SE (3rd generation) and iPhone 16 Pro Max on iOS 26.5 for sizes. Apple's and "before" are from runs [42007197459](https://github.com/mkpoli/seokdok/actions/runs/42007197459) and [42028393000](https://github.com/mkpoli/seokdok/actions/runs/42028393000) (the extras job on iOS 26.5; its iOS 18.5 job failed to build), "after" from [42118026395](https://github.com/mkpoli/seokdok/actions/runs/42118026395) and [42130187452](https://github.com/mkpoli/seokdok/actions/runs/42130187452), with every PR of the series merged.

The script drove touches through XCTest, logged every change in the field from inside the field's app, and recorded each test with `simctl io recordVideo`. Recordings run at about 30 frames a second, so times read from them hold to about 30 ms. Those times are relative to a marker touch in the same recording, so they compare the two keyboards and are not absolute. A simulator's times are not a phone's. Colours are 0-255 grey unless three values are given.

## Geometry

| | Apple iOS 26.5 | Apple iOS 18.5 | After, iOS 26.5 | After, iOS 18.5 |
| --- | --- | --- | --- | --- |
| Keyboard frame, 402 pt wide | 335 pt | 336 pt | 353 pt, see "What a custom keyboard cannot do" | 336 pt |
| Keyboard frame, 440 pt wide | 345 pt | | 363 pt | |
| Keyboard frame, 375 pt wide | 260 pt | | 277 pt | |
| Landscape frame, 874 pt wide | 208 pt | 219 pt | 208 pt | 219 pt |
| Letter key at 402 pt wide | 33 x 43 pt | 34 x 43 pt | 32.8 x 43 pt | 33.9 x 43 pt |
| First key's left edge | 7 pt | 3 pt | 7 pt | 3 pt |
| Row pitch | 54 pt, 56 pt at 440 pt wide | 54 pt | same | same |
| First row's top at 402 pt wide | y = 591 | y = 591 | y = 591 | y = 591 |
| Corner radius | about 8 pt | about 5 pt | 8 pt | 5 pt |
| Fill, light | white, all keys | white; function keys 171,177,186 | white, all keys | white; function keys 169,175,186 |
| Fill, dark | 61, all keys | | 61 | |
| Pressed, dark | 125 | | 125 | |
| Shadow | none | 1 pt | none | 1 pt |
| Number pad frame | 308 pt | 291 pt | 308 pt | 291 pt |

## Behaviour

| Scenario | Apple | Before | After |
| --- | --- | --- | --- |
| 안녕하세요 key by key | the same text at each of 12 steps | same | same |
| Tap Delete on 안녕 | 안녀, 안ㄴ, 안, 아, ㅇ | same | same |
| Held Delete: first repeat, marker-relative | 0.55 s | 0.45 s | 0.55 s |
| Held Delete: repeats | 10 a second | 12.5 | 10 |
| Held Delete: after 20 repeats | a word, 6 a second | no change | a word, 6 a second |
| Held Delete, composing | a syllable at a time | a letter at a time | a syllable at a time |
| Shift, then two letters | ㅃㅂ | same | same |
| Two quick taps of Shift | off again; no lock in Korean | locks | locks, on purpose |
| 123 | changes the keys as the finger lands | on the lift | as the finger lands |
| Key back to the letters | 한글 | 가 | 한글 |
| Slide from 123 to a digit | types it, letters return | nothing | types it, letters return |
| Space or an apostrophe after a symbol | letters return | stays on symbols | letters return |
| Return label | by field: a word before iOS 26, a symbol from it | 改行 | by field: mostly Hanja before iOS 26, the same symbols from it |
| Field wants text and has none | Return greyed | no | greyed |
| Two Spaces after a word or digit | `. `, at gaps up to at least 2 s | two spaces | `. ` |
| Latin capitals by the field's trait | yes | no | yes |
| Numbers and punctuation field | symbols first | letters first | symbols first |
| Number pad | 4 rows, letters under digits, no Return | 5 rows, Return | Apple's keys |
| URL field's bottom row | `.` `/` `.com`, no space bar | `.` `/` and a space bar | Apple's keys |
| Quote key | curls, `‘` at a start | straight | curls in prose fields |
| Tap in a gap between keys | nearest key | same | same |
| Tap in the middle row's margin | the end key | nothing | the end key |
| Shake | undo alert | same | same |
| Long press | a tray on doubles, ㅒ, ㅖ and some symbols | pop-up only | a tray there, and on every key with a Shift letter in its corner |
| Tray on iOS 26 | floats 5 pt over the key, which stays in view | | same |
| Space held, start of the fade, marker-relative | about 515 ms | 672 ms | about 520 ms |
| Slow 32 pt trackpad slide over digits | 11 characters | 3 | 7 |
| Slide up on the trackpad | moves between lines | no | no, see the limits below |
| Tap right after the trackpad | types | types | types |
| Pop-up appears, marker-relative | 67-128 ms | 40-65 ms | 43-59 ms |
| Pop-up gone after a 0.35 s press | at about 400-425 ms | 360 ms | about 395 ms |

## Performance

| | Apple | Ours |
| --- | --- | --- |
| Field update after a key's lift, marker-relative, single keys | 60-71 ms | 55-69 ms |
| First text after focus, tapping from the start, 3 launches | 0.42-0.65 s | 0.42-0.57 s |
| Footprint after typing | host 37-39 MB, `kbd` 26-28 MB, `InputUI` 29-38 MB, all system processes | `SeokdokKeyboard`, below |

| `SeokdokKeyboard` footprint, simulator, Debug | Now | Peak |
| --- | --- | --- |
| `main` at `82bc2d3`, iOS 18.5, CI's fresh boot | 55.5 MB | 58.3 MB |
| Each PR with those before it, same step | 55.5-55.9 MB | 58.1-58.6 MB |
| Six launches of the host, every fix, iOS 26.5 | 60-62 MB | 62 MB |
| Six launches of the host, every fix, iOS 18.5 | 56-58 MB | 59 MB |
| Before, eight launches, iOS 26.5 | 59 MB rising 3 MB each to 80 MB | 80 MB |
| A long session with XCTest attached | 103-113 MB, before 362-408 MB | |

Published estimates put the limit at about 48-60 MB (`ios/README.md`). They are not Apple's, a simulator enforces none, and these peaks sit at or above the top of that range, so a phone may still end the keyboard.

**Not measured.** Frame drops while typing fast, and per-key latency in bursts: sequences of several fingers do not keep their spacing in XCTest's event synthesizer, so bursts arrived faster than scheduled. Non-overlapping bursts of 37 keys reached the field in order on both keyboards.

## What a custom keyboard cannot do

Quotes are from Apple's [App Extension Programming Guide, Custom Keyboard](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html), last updated 2017 and read 2026-11-05. The behaviour of secure and phone fields was seen in these runs too; the others rest on the guide.

| Apple's keyboard | Limit | Closest here |
| --- | --- | --- |
| Pop-ups above the keyboard over the top row | "it is not possible to display key artwork above the top edge of a custom keyboard's primary view" | The pop-up stops at the edge, about 12 pt short; the bar leaves it room |
| Dictation | "have no access to the device microphone, so dictation input is not possible" (the sentence says "in iOS 8.0") | The system's own mic key shows under the keys |
| Secure fields | "the system temporarily replaces your custom keyboard with the system keyboard" | The system's keyboard |
| Phone fields | "the system temporarily replaces your keyboard with the appropriate, standard system keyboard" | The system's keyboard |
| Settings for capitals, the period shortcut, caps lock | "does not have access to most of the general keyboard settings" | Field traits decide; the shortcut is always on |
| Selection, inline corrections | "it cannot select text", "cannot offer inline autocorrection controls near the insertion point" | None |
| Keyboard Feedback haptics | The guide says nothing; see `ios/README.md` | On with Full Access |

Two more were seen in these runs:

- On iOS 26 a custom keyboard's frame reaches about 18 pt above its view in portrait (17 pt at 375 pt wide); in landscape the frames match Apple's. A height of 243 pt gives Apple's 335 pt with the keys in the same place, but leaves the bar 26 pt and a top-row pop-up about 21 pt, so the keyboard keeps 261 pt. `KeyLayout.firstRow` and `barHeight` are the numbers to change for the other choice.
- `documentContextBeforeInput` showed no line break in these fields and `adjustTextPosition` has no up and down, so the trackpad stays on its line.

## Left different on purpose

- Return in 釋 and 混 confirms before it acts, because that is where a conversion or a raw word is kept.
- Two quick taps of Shift lock it, which runs of old letters need; Apple's Korean keyboard has no lock.
- Email and URL fields type Latin, where Apple's stay on Hangul beside the same `@ .` and `. / .com` keys.
- Marked text carries the composition, so some apps underline it. Apple's composes without marks.
- `₩` on the first symbol page where a US-region simulator shows `$`, and `·` on the second where it shows `•`.
- A Twitter field keeps Return, which confirms a conversion. Apple's has `@` `#` in its place.

The caps of the two quote keys look different too: Apple's draw slanted ticks, ours straight marks. Both type the same curled quotes. After a slide from 123 to a digit and back, Apple's typed `1` on iOS 26.5 and `-` on iOS 18.5; ours typed `-` on both.

## Beyond Apple's

- Every key with a corner letter offers it on a long press, so ㅿ, ㆁ, ㆆ, ㆍ and the sibilants need no Shift.
- Conversion to Hanja, with segments that can be resized.

## To try on a phone

A simulator cannot answer these.

1. Two thumbs at full speed. XCTest sends a stray tap for each overlapping finger, on both keyboards, so rollover is unmeasured. Letters must come in order, with Space too.
2. The click and haptic on a key, on Delete's repeats and at the trackpad's start, and whether repeats should sound.
3. Thumb comfort at the bottom row, and on iOS 26 the 18 pt taller keyboard against the room it gives the bar and pop-ups.
4. Move through ten apps with the keyboard up, then look in Settings → Privacy & Security → Analytics & Improvements → Analytics Data for `JetsamEvent` entries. Watch for a blank flash in the keys as apps change.
5. Trackpad speed on Hangul text.
6. Long-press trays at each edge, and a tap with a thumb resting on a key.
7. The globe to another keyboard while composing: the text should stay where it was typed.

## The script

It lived on a temporary branch, since CI stays tests, type check and build, and this document's run links record what it did. The last commit, `b261bf6`, holds `ios/UITests/ParityTests.swift`, `ios/ParityHost/`, `tools/ios-parity/run.sh` and `.github/workflows/ios-parity.yml`.
