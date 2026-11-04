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
| Option-Return, F7, Tab | convert (Tab pages through candidates); the first two can be changed, see below |
| ↓ / ↑, Shift-Space | next / previous candidate |
| 1–9, click | choose a candidate |
| ← → | move between segments; Shift-← → resize the segment |
| Return | commit as shown |
| Esc | close the candidates, then back to the Hangul; while typing, discard it |
| Option alone | Korean ↔ Latin |
| Left Control + left Option, tapped | 釋讀 → 國漢文混用 → 韓㐎 專用 |
| 英数 / かな (JIS) | Latin / back to Korean |

The three modes are input sources of their own: 釋讀 converts, 한글 types
Hangul without conversion, A passes keys through. Pick one in the input menu
or from the input method's own menu. On macOS the selected input source
decides the mode, so `default_mode` in config.toml has no effect here. The
script chord writes `conversion.script` and lands in 釋讀 mode; the input
method's own menu names the script.

### Switching modes

macOS keeps Control-Space, Control-Option-Space and the Globe or Fn key for
itself and switches input sources with them, so an input method never sees
them. Pressing them moves between 釋讀, 釋讀 한글, 釋讀 A and your other sources.
With “Use the Caps Lock key to switch to and from …” on (System Settings ›
Keyboard › Input Sources › Edit…), Caps Lock moves between 釋讀 and your Latin
source. See
[Write in another language on Mac](https://support.apple.com/guide/mac-help/mchlp1406/mac).

Inside the input method the keys come from `[input.keys]` in config.toml, and
the three main lists are also in the settings window:

| List | Default | |
|---|---|---|
| `toggle` | `Alt` | Korean ↔ Latin, back to whichever of 釋讀 and 한글 was last used |
| `latin` | `Eisu` | to Latin |
| `korean` | `Kana` | from Latin back to Korean |
| `script_cycle` | `LCtrl+LAlt` | 釋讀 → 國漢文混用 → 韓㐎 專用 → 釋讀 |
| `seokdok`, `hangul` | empty | straight to that mode |
| `convert` | `Alt+Enter`, `F7` | convert |

A modifier written alone means tapping it with no other key. Left Control and
left Option tapped together is not a shortcut in any application, because
shortcuts need a key besides the modifiers. A Korean PC keyboard sends 한/영
and 한자 with the codes of かな and 英数 (Apple documents this for the USB
LANG1 and LANG2 keys), so on such a keyboard `toggle = ["Alt", "Kana"]`,
`korean = []` and `latin = []` give 한/영 its usual meaning.

**設定 / Settings…** in the input menu opens the settings. They are saved
as `~/Library/Application Support/seokdok/config.toml`, the same file the
other front ends read (every key is documented in it), and apply the next
time a text field gains focus. `custom.json`, exported from
https://seokdok.mkpo.li/#custom, goes in the same folder, as does `learned.tsv`,
the candidates a person picks ([docs/learning.md](../docs/learning.md)). An input
method is told only about secure entry, so email, URL and number fields can be
learned from here.

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
