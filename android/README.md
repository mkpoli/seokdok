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

## Turn it on

Open 釋讀 from the launcher, tap 키보드 設定 열기 and enable 釋讀, then
pick it with 入力 方法 고르기 (or hold the space bar while typing).

## Keys

- 두벌식 letters; each key shows the jamo it types and, in its corner, the
  one on the Shift layer. Shift (double-tap to lock) gives the tense
  consonants, ㅒ ㅖ, and the old letters (ㆍ ㅿ ㆁ ㆆ ᄼ ᄾ ᅎ ᅐ ᅔ ᅕ), tinted.
- 韓 / A switches 釋讀 and Latin. 123 opens digits and punctuation.
- Space (變換) converts; again walks the candidates. Tap a candidate to take
  it; ‹ › turn the page. 確定 (Enter) commits.
- With a phrase in several segments, ◂ ▸ move between them; hold them to
  shrink or grow the segment.
- Password, e-mail, URL and number fields bypass the session.

## Files

- `app/src/main/java/li/mkpo/seokdok/`
  - `Native.kt`: the JNI declarations; `Session.kt`: their Kotlin face.
  - `SeokdokService.kt`: keys → session → composing text, commits, strip.
  - `KeyboardView.kt`, `KeyView.kt`: the candidate strip and the keys.
  - `Files.kt`: copies the dictionaries from the assets to
    `filesDir/dict` on first run and after each update; settings live in
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
config.toml. The Kotlin code and the Gradle build have not been compiled
yet, and nothing has run on a device: that needs the Android SDK.
