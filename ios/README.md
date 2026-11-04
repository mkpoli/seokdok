# 釋讀 for iOS

A custom keyboard (`SeokdokKeyboard`, an app extension) and the app that
carries it (`Seokdok`). The keyboard runs the same Rust session as the other
front ends through the C interface in `crates/seokdok-ffi`; the structure
follows azooKey's (https://github.com/azooKey/azooKey, MIT): a
`UIInputViewController` that writes the text being composed as the
document's marked text and inserts confirmed text.

## Install on an iPhone without a Mac

GitHub Actions builds an unsigned app (`.github/workflows/ios.yml`). A
sideloading tool signs it with your Apple ID and installs it over USB. These
steps use Sideloadly on Windows.

1. On Windows, install iTunes and iCloud from the
   [Sideloadly page](https://sideloadly.io/), which links Apple's web versions.
   Sideloadly does not work with the Microsoft Store versions: uninstall them
   first. Install
   [Sideloadly](https://sideloadly.io/SideloadlySetup64.exe) too.
2. Connect the iPhone by USB, unlock it, open iTunes and tap **Trust** on the
   iPhone.
3. On the iPhone, turn on Settings → Privacy & Security → Developer Mode (iOS
   16 or later) and let it restart. Without a switch there, try step 5 once:
   installing a signed app can make it appear.
4. Open the
   [iOS workflow](https://github.com/mkpoli/seokdok/actions/workflows/ios.yml)
   while signed in to GitHub, choose the newest green run on `main`, download
   the `Seokdok-unsigned-ipa` artifact and unzip it to get
   `Seokdok-unsigned.ipa`. Artifacts expire; with none left, the repository's
   owner can start a new build with **Run workflow** on that page.
5. Drag the IPA onto Sideloadly, enter your Apple ID and press **Start**. Keep
   the app extensions: removing them removes the keyboard.
6. On the iPhone, open Settings → General → VPN & Device Management, tap your
   Apple ID and trust it.
7. Open Settings → General → Keyboard → Keyboards → Add New Keyboard → 釋讀.
   Allow Full Access only to use the app's settings (see below).
8. Open the 釋讀 app, tap the Try it field, and hold the globe key to pick 釋讀.

With a free Apple ID the app stops working after 7 days: repeat step 5 with
the same Apple ID. Apple also allows 3 sideloaded apps at once and 10 App IDs
per 7 days ([Sideloadly FAQ](https://sideloadly.io/faq), read 2026-11-04). The
app and the keyboard each have their own bundle ID, so an install should use
two App IDs; neither Sideloadly nor
[AltStore's FAQ](https://faq.altstore.io/altstore-classic/app-ids) gives the
figure.

Sideloadly's documentation does not say how it treats App Groups. The app
shows a note above its settings when its signature grants no shared container;
the keyboard then runs on the defaults. The note cannot see the keyboard's
signature: if settings have no effect with Full Access on, the keyboard may
have no group of its own.

If the keyboard disappears while loading or typing and the system keyboard
returns, iOS has probably ended it for its memory use (see Memory). Look for
`JetsamEvent` entries in Settings → Privacy & Security → Analytics &
Improvements → Analytics Data; they may need Share iPhone Analytics turned on.

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

In `project.yml`, set `DEVELOPMENT_TEAM` (`xcodegen` overwrites a team chosen in
Xcode) and change the bundle IDs and every `group.li.mkpo.seokdok` to ones your
team can register. The code reads the group from Info.plist, then from the
embedded provisioning profile. Run the `Seokdok` scheme on a device or simulator, then
enable the keyboard: Settings → General → Keyboard → Keyboards → Add New
Keyboard → 釋讀.

Rerun `ios/build-rust.sh` after changing the Rust code, and `xcodegen`
after changing `project.yml`.

## Using it

- Type 두벌식. Each key shows its Shift letter in the corner, including the
  old letters (ㆍ ㅿ ㆁ ㆆ and the sibilants ᄼ ᄾ ᅎ ᅐ ᅔ ᅕ). Shift types one
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
- `App/SeokdokApp.swift`: a field to try the keyboard, how to enable it, and
  the settings.
- `Shared/AppGroup.swift`: finds the App Group the signature grants.
- `Shared/OldHangul.swift`, `Shared/OldHangul.otf`: old Hangul glyphs for the
  keyboard's own labels, which iOS fonts lack. Text sent to other apps shows
  in their fonts, so old letters can appear there as empty boxes.
- `UITests/KeyboardUITests.swift`: types 안녕 through the real extension on a
  simulator and checks the field holds 安寧.
- The dictionaries (`../dict`) are copied into the extension as `dict/`.

## Memory

iOS ends keyboard extensions that use too much memory; Apple does not
publish the limit, and it varies by device. Developer write-ups put it at
about 48 to 60 MB ([React Native docs archive](https://reactnative-archive-august-2023.netlify.app/docs/app-extensions),
[Fleksy](https://fleksy.com/blog/limitations-of-custom-keyboards-on-ios),
[a DEV Community post](https://dev.to/tbds_2dadf2b626f315902eae/the-three-hard-constraints-of-an-ios-keyboard-extension-46af),
all read 2026-11-04; none cites Apple, and the figures disagree).

The extension's physical footprint is 66 MB once loaded, with a 68.5 MB peak
(`footprint` and `vmmap` on a Debug build on an iPhone simulator in CI,
2026-11-04; the Rust library is a release build either way, and each iOS run
records this in the `simulator-screenshots` artifact). A simulator applies no
such limit, and its figures differ from a device's, so this has to be tried on
an iPhone. It is above those published estimates, so a device may end the
keyboard.

Most of it is the Sino-Korean tables (`sino.tsv` and `fusion.tsv`). The engine
alone uses about 59 MB (Linux, 2026-11-04, a small program calling
`seokdok_session::engine_host::load_engine` and reading `VmHWM`): without both
files about 25 MB; without `fusion.tsv` alone, about 58 MB.

## Licences

The bundled dictionaries keep their own licences (`../dict/README.md`).
`Shared/OldHangul.otf` is a subset of Noto Sans CJK KR under the SIL Open Font
License 1.1, stated in the font's name table.

## Status

CI builds the app for devices and passes the UI test on a simulator. It has not
run on a physical iPhone yet. Before release the app also needs an icon.
