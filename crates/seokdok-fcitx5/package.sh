#!/usr/bin/env bash
# Build the fcitx5 addon and stage a release tarball:
#   target/stage/<id>-<version>-x86_64-linux-fcitx5/
#     lib/lib<id>.so  share/fcitx5/{addon,inputmethod}/<id>.conf
#     dict/*.tsv  install.sh  product  LICENSE  README.txt
# Usage: crates/seokdok-fcitx5/package.sh [seokdok|eumdok]  (FCITX5_INCLUDE as for build.sh)
set -euo pipefail
id="${1:-seokdok}"
cd "$(dirname "$0")/../.."
version=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name=="seokdok-ffi") | .version')
crates/seokdok-fcitx5/build.sh "$id"
name="${id}-${version}-x86_64-linux-fcitx5"
stage="target/stage/${name}"
rm -rf "$stage"
mkdir -p "$stage/lib" "$stage/share/fcitx5/addon" "$stage/share/fcitx5/inputmethod" "$stage/dict"
cp "target/fcitx5/lib$id.so" "$stage/lib/"
crates/seokdok-fcitx5/conf.sh "$id" addon >"$stage/share/fcitx5/addon/$id.conf"
crates/seokdok-fcitx5/conf.sh "$id" im >"$stage/share/fcitx5/inputmethod/$id.conf"
cp crates/seokdok-fcitx5/install.sh "$stage/"
printf 'id=%s\nhanja=%s\n' "$id" "$(jq -r .hanja "products/$id.json")" >"$stage/product"
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
Enter confirms, Esc closes / reverts. Hangul key, Shift+Space or a tap of
Right Alt: Korean ↔ Latin. A tap of left Ctrl + left Alt, or Shift+Hangul:
釋讀 → 國漢文混用 → 韓㐎 專用, in 釋讀 mode. Hanja key, F9 or F7 converts. While
釋讀 is composing, Shift+Space is a word break. With more than one input method in the fcitx5 group, fcitx5
itself takes Ctrl+Space, Hangul and 半角/全角 to switch to and from the first one.
[input.keys] in ~/.config/seokdok/config.toml holds these lists.

Old Hangul: Shift+K ㆍ, Shift+A ㅿ, Shift+D ㆁ, Shift+G ㆆ, Shift+Z X C V B N ᄼ ᄾ ᅎ ᅐ ᅔ ᅕ.
Clusters from modern letters (ㅅㄱ → ᄭ, ㅂㅇ → ㅸ) need old_clusters = true in config.toml.
Guide: https://seokdok.mkpo.li/#old

Settings: ~/.config/seokdok/config.toml.
Own words and rules: export them from https://seokdok.mkpo.li/#custom and save
the file as ~/.config/seokdok/custom.json. Switch input method away and back to reload.

Code: MIT. Dictionaries: see dict/README.md (libhangul BSD-3, FusionIME GPL-3.0,
JMdict CC BY-SA 4.0, 釋讀會 community dictionary).
TXT
if [ "$id" = eumdok ]; then
  sed -i -e 's/釋讀 → 國漢文混用 → 韓㐎 專用, in 釋讀 mode/國漢文混用 ↔ 韓㐎 專用/' \
    -e 's/釋讀 seokdok/音讀 eumdok/' -e 's/釋讀\([^會]\)/音讀\1/g' -e 's#config/seokdok#config/eumdok#g' "$stage/README.txt"
fi
tar -C target/stage -czf "target/stage/${name}.tar.gz" "$name"
ls -la "target/stage/${name}.tar.gz"
