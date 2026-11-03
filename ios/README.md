# 釋讀 for iOS

A custom keyboard (`SeokdokKeyboard`, an app extension) and the app that
carries it (`Seokdok`). The keyboard runs the same Rust session as the other
front ends through the C interface in `crates/seokdok-ffi`; the structure
follows azooKey's (https://github.com/azooKey/azooKey, MIT): a
`UIInputViewController` that writes the text being composed as the
document's marked text and inserts confirmed text.

## Build (macOS with Xcode)

Needs Xcode 15 or later, rustup and Homebrew, on an Apple silicon Mac (the
simulator library is built for arm64 only).

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
ios/build-rust.sh                 # → ios/SeokdokCore.xcframework
brew install xcodegen
cd ios && xcodegen                # → ios/Seokdok.xcodeproj, Info.plists, entitlements
open Seokdok.xcodeproj
```

In Xcode, choose your team under Signing & Capabilities for both targets,
and change the bundle IDs (`li.mkpo.seokdok`, `li.mkpo.seokdok.keyboard`)
and the App Group (`group.li.mkpo.seokdok`) to ones your team can register.
The App Group is also named in `Keyboard/KeyboardViewController.swift` and
`App/SeokdokApp.swift`. Run the `Seokdok` scheme on a device or simulator,
then enable the keyboard: Settings → General → Keyboard → Keyboards → Add
New Keyboard → 釋讀.

Rerun `ios/build-rust.sh` after changing the Rust code, and `xcodegen`
after changing `project.yml`.

## Using it

- Type 두벌식. Each key shows its Shift letter in the corner; old letters
  (ㆍ ㅿ ㆁ ㆆ and the sibilants ᄼ ᄾ ᅎ ᅐ ᅔ ᅕ) are tinted. Shift types one
  letter; a double tap locks it.
- 變換 (Space) converts the composition and shows the focused segment's
  candidates, with their notes, in the bar; Space again moves to the next
  one. Tap a candidate to confirm through that segment, ‹ › to page.
- The strip above the bar shows the composition. Tap the Hangul to convert
  it (also when Space only types a space); tap a segment to focus it; − and
  + shorten or lengthen the focused segment.
- 確定 (Return) confirms everything. Backspace undoes the conversion, then
  deletes letters; hold it to repeat.
- The mode key cycles 釋 (convert to 釋讀) → 한 (Hangul only) → A (Latin).
- 123 opens digits and punctuation, #+= the second page.

## Settings and Full Access

The app writes its settings as `config.toml` (the format of
`crates/seokdok-session/src/config.rs`) into the App Group container. The
keyboard reads it, with `custom.json` and `user.tsv` if present there, each
time it appears.

A keyboard reaches the shared container only with Full Access (Apple's App
Extension Programming Guide, Custom Keyboard, Table 8-1: without it "No
shared container with containing app"). Without Full Access the keyboard
still works fully, on the default settings; it never uses the network.

## Layout

- `Keyboard/Session.swift`: the Swift face of the C interface.
- `Keyboard/KeyboardViewController.swift`: keys → session → marked text,
  commits, the strip and the candidate bar; drops the composition when the
  text changes outside the keyboard.
- `Keyboard/KeyboardView.swift`: the strip, the candidate bar, the 두벌식
  and symbol layers.
- `App/SeokdokApp.swift`: how to enable the keyboard, and the settings.
- The dictionaries (`../dict`) are copied into the extension as `dict/`.

## Memory

iOS ends keyboard extensions that use too much memory; Apple does not
publish the limit, and it varies by device. The engine with every
dictionary uses about 59 MB once loaded, which is also the peak while
loading (measured on Linux, 2026-11-04, with a small program calling
`seokdok_session::engine_host::load_engine` and reading `VmHWM`). Most of
it is the Sino-Korean tables (`sino.tsv` and `fusion.tsv`): without both
the engine takes about 25 MB; without `fusion.tsv` alone, about 58 MB. UIKit
adds its own share on a device. Measure with Instruments (Allocations,
and the extension's memory gauge in Xcode) before relying on it.

## Licences

The bundled dictionaries keep their own licences (`../dict/README.md`).

## Status

The Rust library cross-compiles from Linux for `aarch64-apple-ios` and
`aarch64-apple-ios-sim` and exports every function in `seokdok.h`; the key
sequences the keyboard sends were checked against it through a C program on
Linux. The Swift code and the Xcode project have not been compiled yet:
that needs a Mac. Before release the app also needs an icon.
