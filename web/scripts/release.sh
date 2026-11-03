#!/usr/bin/env bash
# Package the Windows and Linux input methods and copy them to
# public/downloads/ for the site; src/lib/downloads.json records names and sizes.
# Needs cargo-xwin (Windows) and the fcitx5 headers (FCITX5_INCLUDE or pkg-config).
set -euo pipefail
web="$(cd "$(dirname "$0")/.." && pwd)"
root="$(cd "$web/.." && pwd)"
"$root/crates/seokdok-tsf/package.sh"
"$root/crates/seokdok-fcitx5/package.sh"
version=$(cargo metadata --no-deps --format-version 1 --manifest-path "$root/Cargo.toml" | jq -r '.packages[] | select(.name=="seokdok-ffi") | .version')
win="seokdok-${version}-x86_64-windows.zip"
linux="seokdok-${version}-x86_64-linux-fcitx5.tar.gz"
rm -rf "$web/public/downloads"
mkdir -p "$web/public/downloads"
cp "$root/target/stage/$win" "$root/target/stage/$linux" "$web/public/downloads/"
jq -n --arg v "$version" --arg w "$win" --arg l "$linux" \
  --argjson ws "$(stat -c %s "$web/public/downloads/$win")" \
  --argjson ls "$(stat -c %s "$web/public/downloads/$linux")" \
  '{version: $v, windows: {file: $w, size: $ws}, linux: {file: $l, size: $ls}}' >"$web/src/lib/downloads.json"
cat "$web/src/lib/downloads.json"
