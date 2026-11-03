#!/usr/bin/env bash
# Build the fcitx5 addon and stage a release tarball:
#   target/stage/seokdok-<version>-x86_64-linux-fcitx5/
#     lib/libseokdok.so  share/fcitx5/{addon,inputmethod}/seokdok.conf
#     dict/*.tsv  install.sh  LICENSE  README.txt
# Usage: crates/seokdok-fcitx5/package.sh  (FCITX5_INCLUDE as for build.sh)
set -euo pipefail
cd "$(dirname "$0")/../.."
version=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name=="seokdok-ffi") | .version')
crates/seokdok-fcitx5/build.sh
name="seokdok-${version}-x86_64-linux-fcitx5"
stage="target/stage/${name}"
rm -rf "$stage"
mkdir -p "$stage/lib" "$stage/share/fcitx5/addon" "$stage/share/fcitx5/inputmethod" "$stage/dict"
cp target/fcitx5/libseokdok.so "$stage/lib/"
cp crates/seokdok-fcitx5/addon/seokdok-addon.conf "$stage/share/fcitx5/addon/seokdok.conf"
cp crates/seokdok-fcitx5/addon/seokdok-im.conf "$stage/share/fcitx5/inputmethod/seokdok.conf"
cp crates/seokdok-fcitx5/install.sh "$stage/"
cp LICENSE "$stage/LICENSE"
for f in dict/*.tsv; do
  case "$(basename "$f")" in user*.tsv) ;; *) cp "$f" "$stage/dict/" ;; esac
done
cp dict/README.md dict/LICENSE-fusion.txt "$stage/dict/"
cat > "$stage/README.txt" <<'TXT'
釋讀 seokdok — input method for fcitx5 (Linux)

Install:   ./install.sh            (this user, no root)
           sudo ./install.sh --system
Uninstall: ./install.sh --uninstall  [--system]
Then restart fcitx5 (fcitx5 -rd) and add 釋讀 in fcitx5-configtool
(uncheck "Only Show Current Language" to find it). Built against fcitx5 5.1.

Keys: type 두벌식 Hangul; Space converts, Space again opens the candidates;
1-9 pick, Tab/PgDn page, ←→ move between words, Shift+←→ resize a word,
Enter confirms, Esc closes / reverts. Hangul key or a tap of Alt: 釋讀 ↔ A.
Hanja key or F7 converts.

Old Hangul: Shift+K ㆍ, Shift+A ㅿ, Shift+D ㆁ, Shift+G ㆆ, Shift+Z X C V B N ᄼ ᄾ ᅎ ᅐ ᅔ ᅕ.
Clusters from modern letters (ㅅㄱ → ᄭ, ㅂㅇ → ㅸ) need old_clusters = true in config.toml.
Guide: https://seokdok.mkpo.li/#old

Settings: ~/.config/seokdok/config.toml.
Own words and rules: export them from https://seokdok.mkpo.li/#custom and save
the file as ~/.config/seokdok/custom.json. Switch input method away and back to reload.

Code: MIT. Dictionaries: see dict/README.md (libhangul BSD-3, FusionIME GPL-3.0,
JMdict CC BY-SA 4.0, 釋讀會 community dictionary).
TXT
tar -C target/stage -czf "target/stage/${name}.tar.gz" "$name"
ls -la "target/stage/${name}.tar.gz"
