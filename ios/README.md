# 釋讀 for iOS

A custom keyboard (`SeokdokKeyboard`, an app extension) and the app that
carries it (`Seokdok`). The keyboard runs the same Rust session as the other
front ends through the C interface in `crates/seokdok-ffi`; the structure
follows azooKey's (https://github.com/azooKey/azooKey, MIT): a
`UIInputViewController` that writes the text being composed as the
document's marked text and inserts confirmed text.

## Build (macOS with Xcode)

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
ios/build-rust.sh                 # → ios/SeokdokCore.xcframework
brew install xcodegen
cd ios && xcodegen                # → ios/Seokdok.xcodeproj
open Seokdok.xcodeproj
```

Set your team under Signing for both targets, and change the App Group
(`group.li.mkpo.seokdok`) and bundle IDs (`li.mkpo.*`) to ones your team can
register. Run on a device, then enable it: Settings → General → Keyboard →
Keyboards → Add New Keyboard → 釋讀. Turn on Allow Full Access for the
keyboard to read the settings the app writes; without it the defaults apply.

## Layout

- `Keyboard/Session.swift`: the Swift face of the C interface.
- `Keyboard/KeyboardViewController.swift`: keys → session → marked text,
  commits and the candidate bar.
- `Keyboard/KeyboardView.swift`: 두벌식 keys with the Shift layer in each
  key's corner (old letters tinted), and the candidate bar.
- `App/SeokdokApp.swift`: how to enable the keyboard, and the settings,
  written as `config.toml` into the App Group.
- The dictionaries (`../dict`) are bundled into the extension.

## Memory

Keyboard extensions get little memory (a few tens of MB). The engine with
every dictionary uses about 53 MB once loaded and 63 MB at the peak of
loading (measured on Linux with `cargo run --release -p seokdok-core
--example memory`); measure on a device with Instruments before relying on
it.

## Status

The Rust library cross-compiles for `aarch64-apple-ios` and
`aarch64-apple-ios-sim` and exports the interface in `seokdok.h`. The Swift
code and the Xcode project have not been compiled yet: they need a Mac.
