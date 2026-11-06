# Seokdok 釋讀

An input method for 釋讀, the Korean spelling played with in 釋讀會 (풀어읽기모임): Sino-Korean words in Hanja by sound (音讀), native words in Hanja by meaning (訓讀), and the inflected rest in Hangul (末音添記). It runs in the browser at [seokdok.mkpo.li](https://seokdok.mkpo.li/) and installs as a system input method on Windows, macOS, Linux (fcitx5), Android and iOS.

```
하늘을 봤다              → 天을 見ㅆ다
처음 오신 것을 환영합니다  → 初음 來신 事ㅅ을 歡迎합니다
옛한글로 양심을 적다       → 古ㅅ韓㐎로 良心을 記다
```

In 國漢文混用 mode only Sino-Korean words become Hanja: 선생님께서 공부하셨다 → 先生님께서 工夫하셨다. In 韓㐎 專用 mode nothing is converted: 두벌식 typing with old Hangul, Space types a space, Enter commits, and there is no candidate window. The `script` setting in config.toml (`seokdok`, `sino_only`, `hangul_only`) picks among the three. The spelling rules and their options are in [docs/orthography.md](docs/orthography.md). Old Hangul such as ᄉᆞ랑 is accepted as input, and ᄒᆞ, ᄂᆞᆫ and ᄅᆞᆯ are offered after Hanja (追加ᄒᆞᆯ까요); see [§8](docs/orthography.md#8-old-spellings).

## Install

Downloads are on [seokdok.mkpo.li/#install](https://seokdok.mkpo.li/#install).

- **Windows** (x64, Arm64): run the setup, then pick 釋讀 with Win+Space.
- **Linux** (x86_64, aarch64): unpack the tarball and run `install.sh` (per user, no sudo), restart fcitx5 and add 釋讀 in fcitx5-configtool.
- **Android**: download the APK, allow installing from unknown sources when asked, open the 釋讀 app, turn the keyboard on and pick it as the input method; see [android/README.md](android/README.md).
- **macOS**: unzip into `~/Library/Input Methods`, clear the quarantine flag and register it; see [macos/README.md](macos/README.md).
- **iOS**: sideload the unsigned IPA with your Apple ID, then add the keyboard in Settings; see [ios/README.md](ios/README.md).

The Arm64 builds and the macOS app have not run on real hardware yet, and the Windows Arm64 installer is unsigned.

Space converts, Space again lists candidates, 1–9 picks, Enter confirms, Esc cancels. A character with many homophones can be typed by its gloss: 살찔비, 비살찔 or 살찔 비 offer 肥. Settings live in `config.toml`: `%APPDATA%\seokdok\` on Windows, `~/Library/Application Support/seokdok/` on macOS, `~/.config/seokdok/` elsewhere. Your own rules and words go in `custom.json` beside it; the Customize section of the site exports one. A candidate you pick twice comes first for that reading; the picks stay on the device in `learned.tsv` beside `config.toml`, `input.learn = false` turns this off, and the settings have a button to forget them ([docs/learning.md](docs/learning.md)). On Windows, right-click the 釋 icon of the input indicator on the taskbar to switch between 釋, 混 and 한 and change frequent settings; `windows.menu_items` or the Taskbar menu page of the settings app choose what that menu shows. `windows.candidate_style` (`windows11`, `classic`) and `windows.candidate_theme` (`system`, `light`, `dark`), also on the Typing page of the settings app, set how the candidate window looks.

## 音讀

音讀 (Eumdok, 음독) is a second input method built from this repository, for 國漢文混用 only: Sino-Korean words in Hanja, native words and everything else in Hangul, with no 訓讀 in the candidates or the settings. It shares the engine, the dictionaries and the session with 釋讀 and installs beside it with its own identifiers and settings folder (`eumdok`). Its site and downloads for the same platforms are at [eumdok.mkpo.li](https://eumdok.mkpo.li/). `products/<id>.json` holds what differs; `crates/seokdok-product` makes it available to Rust, where `--features eumdok` selects the product, and the build scripts of each front end read the same file.

## Mode keys

There are three modes: 釋讀, plain 한글 and Latin. In 釋讀 mode the script is 釋讀, 國漢文混用 or 韓㐎 專用. The keys are the ones people know from the Korean tools on each platform, as far as the system lets an input method see them. They live in `[input.keys]` in `config.toml`, with defaults per platform; a modifier written alone means tapping it with no other key.

| | Korean ↔ Latin | Next script | Convert | To Latin / back |
|---|---|---|---|---|
| Windows | 한/영, right Alt, 半角/全角 | Shift+한/영, left Ctrl + left Alt | 한자, right Ctrl, 変換, F7 | 無変換 / 変換 |
| macOS | Option | left Control + left Option | Option-Return, F7 | 英数 / かな |
| Linux (fcitx5) | Hangul, Shift+Space, right Alt, 半角/全角 | Shift+Hangul, left Ctrl + left Alt | Hanja, F9, 変換, F7 | 無変換 / 変換 |
| Android (hardware keyboard) | 한/영, Shift+Space, 半角/全角 | Shift+한/영, left Ctrl + left Alt | 한자, 変換, F7 | 無変換 / 変換 |

The script key goes 釋讀 → 國漢文混用 → 韓㐎 專用 → 釋讀, in every running application, and from Latin or plain 한글 it lands in 釋讀 mode at the next script. Plain 한글 stays in the Windows and macOS menus, and in the `hangul` list of `[input.keys]`. iOS has the on-screen mode key only, 釋 → 混 → 한 → A → 釋, as does Android's on-screen keyboard. Left Alt alone is left to application menus on Windows and Linux; add `Alt` to `toggle` to use it. Windows Korean keyboard drivers turn right Alt and right Ctrl (type 1) or Shift+Space and Ctrl+Space (type 3) into 한/영 and 한자 before the input method sees them. AltGr never counts as right Alt.

On Linux and Android, Shift+Space toggles except while 釋讀 is composing, when it stays a space inside the composition and the previous candidate in a conversion. With more than one input method in its group, fcitx5 itself takes Ctrl+Space, Hangul, 半角/全角 and a tap of left Shift to switch.

[docs/mode-keys.md](docs/mode-keys.md) has each platform's existing keys with sources and the reasons for these defaults. Per-platform details are in [macos/README.md](macos/README.md), [android/README.md](android/README.md) and [ios/README.md](ios/README.md).

## Usage

```sh
cargo run --release -p seokdok-cli -- convert 그렇네요
cargo run --release -p seokdok-cli -- segments 이런 것도 있네요
cargo run --release -p seokdok-cli -- read 天을 見ㅆ다
cargo run --release -p seokdok-cli -- add 즐기다 樂기다 verb
```

`convert` lists whole-phrase candidates, `segments` the best segmentation with alternatives for each part, `read` reads 釋讀 back into Hangul (하늘을 봤다), and `add` appends a word to `dict/user.tsv`. Options: `--sino` (國漢文混用), `--morpheme` (形態式: 見았다), `--letter` (leftover vowels as letters: 見ㅏ라), `--coda` (末音添記 everywhere), `--resyllabify` (在쓰면), `--hada` (爲 for 하다), `--old-ha` (ᄒᆞ before 하).

## Layout

- `crates/seokdok-core`: the engine. Hangul is decomposed into jamo, and a lattice of Sino-Korean words, 訓讀 nouns with particles and 訓讀 stems with endings is searched for the cheapest analyses.
- `crates/seokdok-product`: what tells 釋讀 and 音讀 apart, from `products/*.json`.
- `crates/seokdok-session`: the platform-independent IME: keys, composition, candidates, learned picks, `config.toml` and `custom.json`.
- `crates/seokdok-tsf`: the Windows IME (Text Services Framework) and its installer.
- `crates/seokdok-fcitx5`: the fcitx5 addon. `build.sh` builds it, `e2e.sh` drives it through a private fcitx5 over D-Bus, `package.sh` stages the tarball.
- `crates/seokdok-ffi`: a C interface (`include/seokdok.h`) for fcitx5, iOS and macOS. `crates/seokdok-jni` wraps it for Android.
- `crates/seokdok-wasm`: WebAssembly bindings for the site.
- `crates/seokdok-cli`: the `seokdok` command above.
- `ios/`, `macos/`, `android/`: the keyboard, input method and keyboard for each platform, each with its own README.
- `web/`: both sites (Svelte): the in-browser IME, a 두벌식 keyboard, a converter, a reader, and the dictionary browser at `/dict`.
- `dict/`: the dictionaries and their licences, see [dict/README.md](dict/README.md).
- `tools/`: scripts that rebuild `dict/` from the sources in [data/sources/README.md](data/sources/README.md).

## Development

```sh
cargo test --workspace --exclude seokdok-tsf
cd web && bun install && bun run build && bun test src
```

`bun run build` builds the WASM package, packs the dictionaries and builds the site. `bun run deploy` publishes it to Cloudflare, and `bun run release` packages the x64 Windows and x86_64 Linux input methods of both products, adds the Android, macOS, iOS and Arm64 builds from the latest workflow runs on `main`, writes `web/src/lib/downloads.json`, and puts the files in `web/public/downloads/` (not committed).

```sh
uv run tools/extract_telegram.py <telegram-export-dir> data/private/telegram.jsonl
uv run tools/build_dict.py
uv run tools/attest.py
```

These rebuild `community.tsv`, `gloss.tsv` and `sino.tsv` and refresh the usage counts. They replace counts and first candidates that were reviewed by hand, so words from a new chat export go in as hand rows instead. `uv run tools/build_eonhae.py` rebuilds `eonhae.tsv` from the Wikisource dump and saved 나무위키 pages named in `data/sources/README.md`; run `attest.py` after it to restore that file's counts. `uv run tools/build_kun_usage.py` rebuilds `data/kun-usage.tsv` and `kun-attested.tsv` from the Wikisource dump; `attest.py` fills in that file's counts too. `uv run tools/people_names.py fetch` and `build` rebuild `people.tsv`. `uv run tools/place_names.py build` rebuilds `place-names.tsv` from the saved Wikipedia page named in `data/sources/README.md`. The chat export stays in `data/private/` and is never committed.

## References

- 釋讀會 (풀어읽기모임), Telegram group: https://t.me/pimsdh. Its spellings and usage counts come from the group's chat.
- 標準國語釋讀辭典, the group's community dictionary (Google Sheets).
- Choe Hwanjin (2005–2006) [libhangul](https://github.com/libhangul/libhangul) `hanja.txt`.
- [JMdict](https://www.edrdg.org/jmdict/j_jmdict.html), English Wiktionary via [kaikki.org](https://kaikki.org/), FusionIME, OpenCC, [Wikidata](https://www.wikidata.org/) and [Korean Wikipedia](https://ko.wikipedia.org/); see [dict/README.md](dict/README.md) for what each supplies.

## Sponsor

I build Seokdok on my own, with no funding behind it, and it's free for everyone. If it's useful to you, a tip on [Ko-fi](https://ko-fi.com/mkpoli) or [OFUSE](https://ofuse.me/mkpoli) gives me time to keep improving it. Companies interested in sponsoring can write to mkpoli@mkpo.li.

[![Support me on Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/mkpoli)

## License

Code: MIT © 2026 mkpoli. Each dictionary keeps the licence of the work it comes from; see [dict/README.md](dict/README.md).
