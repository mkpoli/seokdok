# 釋讀 for Android

An input method (`SeokdokService`, an `InputMethodService`) and its settings
screen. The keyboard runs the same Rust session as the other front ends:
`crates/seokdok-jni` wraps the C interface of `crates/seokdok-ffi` as the
native methods of `li.mkpo.seokdok.Native`. The text being composed is the
editor's composing text (`InputConnection.setComposingText`), confirmed text
is committed with `commitText`, and the candidates fill the strip above the
keys.

## Build

Needs a JDK 17 or newer, the Android SDK (platform 35, build tools) and NDK
28, Rust with the Android targets, and cargo-ndk:

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk
sdkmanager "platforms;android-35" "build-tools;35.0.0" "ndk;28.0.13004108"
export ANDROID_HOME=$HOME/Android/Sdk      # where the SDK is
```

The Gradle wrapper is not committed; create it once with Gradle 8.11 or
newer (or open `android/` in Android Studio, which reads
`gradle/wrapper/gradle-wrapper.properties`):

```sh
cd android
gradle wrapper --gradle-version 8.11.1
./gradlew assembleDebug                    # → app/build/outputs/apk/debug/app-debug.apk
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

`assembleDebug` runs `cargo ndk` from the repository root for arm64-v8a,
armeabi-v7a and x86_64 (always `--release`: an unoptimised engine loads the
dictionaries far more slowly) and copies `dict/*.tsv` into the APK's assets.
To build only the Rust library:

```sh
cargo ndk --platform 26 -t arm64-v8a -t armeabi-v7a -t x86_64 \
  -o android/app/build/generated/seokdok/jniLibs/debug build --release -p seokdok-jni
```

## Release APK

`.github/workflows/android.yml` builds a release APK signed with the project
key when the repository has three secrets, and the debug APK otherwise (always
for pull requests):

| Secret | Content |
| --- | --- |
| `ANDROID_KEYSTORE_BASE64` | the keystore file, base64 |
| `ANDROID_KEYSTORE_PASSWORD` | keystore password |
| `ANDROID_KEY_ALIAS` | key alias |

The signed APK is the artifact `seokdok-release-apk`; `web/scripts/release.sh`
downloads it into the site's downloads. Locally, `assembleRelease` reads the
same values from `SEOKDOK_KEYSTORE` (path), `SEOKDOK_KEYSTORE_PASSWORD` and
`SEOKDOK_KEY_ALIAS`; without `SEOKDOK_KEYSTORE` it produces an unsigned APK that cannot be installed. Keep the keystore outside
the repository and back it up: an update must be signed with the same key as
the installed version.

## Turn it on

Open 釋讀 from the launcher, tap 키보드 設定 열기 and enable 釋讀, then
pick it with 入力 方法 고르기 (or hold the space bar while typing).

## Keys

- 두벌식 letters; each key shows the jamo it types and, in its corner, the
  one on the Shift layer. Shift (double-tap to lock) gives the tense
  consonants, ㅒ ㅖ, and the old letters (ㆍ ㅿ ㆁ ㆆ ᄼ ᄾ ᅎ ᅐ ᅔ ᅕ).
- The mode key goes 釋 → 混 → 한 → A → 釋: the script, written to the settings, then Latin. 123 opens digits and punctuation.
- Space (變換) converts; again walks the candidates. Tap a candidate to take
  it; ‹ › turn the page. 確定 (Enter) commits.
- With a phrase in several segments, ◂ ▸ move between them; hold them to
  shrink or grow the segment.
- Password, e-mail, URL and number fields bypass the session.

### Hardware keyboards

Keys typed on a hardware keyboard go through the same session, and the window
opens only while candidates are showing. These lists are `[input.keys]` in
config.toml; the first three can be edited in the app:

| List | Default | |
|---|---|---|
| `toggle` | 한/영, 半角/全角, Shift+Space | Korean ↔ Latin, back to whichever of 釋讀 and 한글 was last used |
| `script_cycle` | left Ctrl + left Alt tapped, Shift+한/영 | 釋讀 → 國漢文混用 → 韓㐎 專用 → 釋讀, landing in 釋讀 mode |
| `convert` | 한자, 変換, F7 | convert |
| `latin` | 無変換 | to Latin |
| `korean` | 変換 | from Latin back to Korean |
| `seokdok`, `hangul` | empty | straight to that mode |

While 釋讀 is composing, Shift+Space stays a word break inside the composition
and the previous candidate in a conversion. Otherwise it toggles.

The system keeps some keys for itself. Android's window manager takes
Ctrl+Space and the language-switch key for the hardware keyboard layout, so an
input method never sees them. Samsung's One UI also lists Shift+Space and left
Alt+Shift as language shortcuts, with the choice under the physical keyboard
settings' “Change language shortcut”; turn Shift+Space off there for it to
reach 釋讀, or use 한/영.

## Files

- `app/src/main/java/li/mkpo/seokdok/`
  - `Native.kt`: the JNI declarations; `Session.kt`: their Kotlin face.
  - `SeokdokService.kt`: keys → session → composing text, commits, strip.
  - `KeyboardView.kt`, `KeyView.kt`: the candidate strip and the keys.
  - `Files.kt`: copies the dictionaries from the assets to
    `filesDir/dict` on first run and after each update, on a background
    thread (the strip shows 辭典 讀는 中… and keys wait until it is done);
    settings live in
    `filesDir/config`.
  - `SettingsActivity.kt`, `ConfigText.kt`: the settings, written as
    `config.toml` through the session's own writer
    (crates/seokdok-session/src/config.rs documents every key), and the
    custom.json import.
- `app/src/main/res/xml/method.xml`: the input method and its ko-KR subtype.

## Status

`crates/seokdok-jni` builds and passes its tests on Linux and type-checks
for `aarch64-linux-android`, `armv7-linux-androideabi` and
`x86_64-linux-android`. Its native methods were exercised from a JVM
(`java -Xcheck:jni`) through a Java mirror of `Native.kt`: typing, converting,
the candidate page, picking, committing, Latin mode, and reading and writing
config.toml. The APK builds on GitHub Actions
(`.github/workflows/android.yml`, artifacts `seokdok-debug-apk` and
`seokdok-release-apk`). The debug APK was installed on a BlueStacks instance (Android 9, x86_64) and driven
through the soft keyboard with `adb shell input tap`: 안녕 + Space gives 安寧,
수있다 gives 手有다 with candidate lists for each segment, the Shift layer types
ㅿ ㆁ ㆆ ㆍ, and 韓㐎 專用 leaves Hangul unconverted. It has not run on a phone, on
arm64, or in a release build.
