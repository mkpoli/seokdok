# seokdok 釋讀

An input method for 釋讀, an experimental Korean orthography practised in
釋讀會 (풀어읽기모임): Sino-Korean words in Hanja by sound (音讀), native
words in Hanja by meaning (訓讀), and the inflected rest in Hangul (送韓字).
Type Korean, convert, and get

```
하늘을 봤다            → 天을 見ㅆ다
처음 오신 것을 환영합니다 → 初음 來신 事ㅅ을 歡迎합니다
어떻게 읽어요           → 何떻게 讀어요
```

The spelling rules and their options are in
[docs/orthography.md](docs/orthography.md).

## Layout

- `crates/seokdok-core`: the engine. Hangul is decomposed into jamo; a lattice
  of Sino-Korean words, 訓讀 nominals with particles, and 訓讀 stems with
  endings is searched for the cheapest analyses, which are then written in the
  configured style.
- `crates/seokdok-cli`: `seokdok` command for trying conversions and checking
  dictionaries.
- `crates/seokdok-session`: the platform-independent IME: key handling,
  conversion state, candidates, settings and the user's custom.json.
- `crates/seokdok-tsf`: the Windows IME (Text Services Framework).
- `crates/seokdok-fcitx5`: the Linux IME, an fcitx5 addon. `build.sh` links
  the Rust core into `libseokdok.so`, `e2e.sh` drives it through a private
  fcitx5 over D-Bus, `package.sh` stages the release tarball.
- `crates/seokdok-wasm`: WebAssembly bindings (`Engine`, `Composer`) for
  JavaScript, built with `wasm-pack`.
- `web/`: https://seokdok.mkpo.li/, an in-browser IME with a 두벌식 keyboard,
  a reader and the landing page (Svelte, Cloudflare static assets).
  `bun run build` in `web/` builds the WASM package, packs the dictionaries
  and builds the site; `bun run deploy` publishes it.
- `dict/`: dictionaries, see [dict/README.md](dict/README.md).
- `tools/`: scripts that rebuild `dict/` from the sources.

## Trying it

```sh
cargo run --release -p seokdok-cli -- convert 그렇네요
cargo run --release -p seokdok-cli -- segments 이런 것도 있네요
cargo run --release -p seokdok-cli -- --morpheme --resyllabify convert 있으면 어려워요
```

`seokdok read 天을 見ㅆ다` reads 釋讀 text back into Hangul (하늘을 봤다), and
`seokdok add 즐기다 樂기다 verb` adds a word to `dict/user.tsv`.

Options: `--morpheme` (形態式), `--letter` (leftover vowels as letters),
`--coda` (末音添記 everywhere), `--resyllabify` (在쓰면), `--hada` (爲 for
하다).

## Rebuilding the dictionaries

```sh
uv run tools/extract_telegram.py <telegram-export-dir> data/private/telegram.jsonl
uv run tools/build_dict.py   # community.tsv, gloss.tsv, sino.tsv
uv run tools/attest.py       # refresh corpus counts in dict/*.tsv
```

The sources are described in [data/sources/README.md](data/sources/README.md);
`data/` is not committed except that README.

## Licence

MIT © 2026 mkpoli, except the dictionaries derived from other works; see
[dict/README.md](dict/README.md).
