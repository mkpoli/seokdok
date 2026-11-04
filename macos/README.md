# 釋讀 for macOS

An input method built on InputMethodKit. It runs the same Rust session as
the other front ends through the C interface in `crates/seokdok-ffi`: keys
go to the session, the text being composed is the document's marked text,
and the candidates open in a window under it.

## Build

On a Mac with Xcode 15 or later and Rust installed through rustup:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
macos/build-rust.sh               # → macos/build/lib/libseokdok_ffi.a (arm64 + x86_64)
brew install xcodegen
cd macos
xcodegen                          # → macos/Seokdok.xcodeproj
xcodebuild -project Seokdok.xcodeproj -scheme Seokdok -configuration Release \
  -derivedDataPath build/DerivedData build
```

The app is `macos/build/DerivedData/Build/Products/Release/Seokdok.app`.
It is signed to run locally; to give it to others, set your team and a
Developer ID in `project.yml` and notarize it.

## Install

From `macos/`:

```sh
mkdir -p ~/Library/Input\ Methods
killall Seokdok 2>/dev/null; rm -rf ~/Library/Input\ Methods/Seokdok.app
cp -R build/DerivedData/Build/Products/Release/Seokdok.app ~/Library/Input\ Methods/
```

1. Log out and log back in.
2. Open System Settings › Keyboard › Input Sources › Edit…, press **+**,
   choose Korean, and add **釋讀** (and **釋讀 한글**, **釋讀 A** to switch
   modes from the input menu).
3. Choose 釋讀 in the input menu of the menu bar.

Instead of logging out, you can register it from where it is:

```sh
~/Library/Input\ Methods/Seokdok.app/Contents/MacOS/Seokdok --register
```

To update, run the install commands again: `killall` stops the old copy,
and macOS starts the new one when the input method is next used.

## Use

| Key | |
|---|---|
| letters | 두벌식, by key position |
| Space | convert; again for the next candidate |
| Option-Return, F7, Tab | convert (Tab pages through candidates) |
| ↓ / ↑, Shift-Space | next / previous candidate |
| 1–9, click | choose a candidate |
| ← → | move between segments; Shift-← → resize the segment |
| Return | commit as shown |
| Esc | close the candidates, then back to the Hangul; while typing, discard it |
| Option alone | 釋讀 ↔ Latin (setting “Tap Option to switch”) |
| 英数 / かな (JIS) | Latin / back to Hangul |

The three modes are input sources of their own: 釋讀 converts, 한글 types
Hangul without conversion, A passes keys through. Pick one in the input menu
or from the input method's own menu. On macOS the selected input source
decides the mode, so `default_mode` in config.toml has no effect here.

**設定 / Settings…** in the input menu opens the settings. They are saved
as `~/Library/Application Support/seokdok/config.toml`, the same file the
other front ends read (every key is documented in it), and apply the next
time a text field gains focus. `custom.json`, exported from
https://seokdok.mkpo.li/#custom, goes in the same folder.

## Layout

- `Seokdok/main.swift`: starts the IMK server; `--register` adds the input
  source.
- `Seokdok/InputController.swift`: the `IMKInputController`: keys → session
  → marked text, commits, the candidate window, the modes and the menu.
- `Seokdok/KeyMap.swift`: key codes to session keys.
- `Seokdok/CandidateWindow.swift`: the candidate panel.
- `Seokdok/SettingsWindow.swift`: the settings window (SwiftUI).
- `Seokdok/Session.swift`: the Swift face of the C interface, and
  config.toml read and written through it.
- `Seokdok/Info.plist`: the IMK keys and the three input modes; their names
  are in `Resources/*.lproj/InfoPlist.strings`.
- `Seokdok/Resources/*.tiff`: menu icons, made by `make_icons.py`.
- The dictionaries (`../dict`, without `user*.tsv`) are copied into the app
  at build time.

The IMK usage was checked against azooKey-Desktop
(https://github.com/azooKey/azooKey-Desktop, MIT), macSKK
(https://github.com/mtgto/macSKK, GPL-3.0) and Squirrel
(https://github.com/rime/squirrel, GPL-3.0), read on 2026-11-04: mode keys
that equal their TISInputSourceID, `selectMode`, `overrideKeyboard`,
`--register`. No code is copied from them.

## Status

CI (`.github/workflows/macos.yml`) builds the universal Rust library and the
app on a macOS runner, checks that the bundle holds the dictionaries, and
links the library into a small program (`smoke/smoke.c`) that converts
안녕 to 安寧. The app is uploaded unsigned. The input method itself has not
been run: registering it and typing into a text field needs a logged-in Mac.
