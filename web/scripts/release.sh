#!/usr/bin/env bash
# Package the Windows and Linux input methods, add the Android APK and copy
# them to public/downloads/ for the site; src/lib/downloads.json records names
# and sizes.
# Needs cargo-xwin (Windows), the fcitx5 headers (FCITX5_INCLUDE or pkg-config)
# and gh. The APK is the signed release build from the Android workflow
# (artifact seokdok-release-apk of the latest successful push run on main); set
# SEOKDOK_APK to a local signed app-seokdok-release.apk to use that instead. The Windows
# installer takes the settings app the same way: see crates/seokdok-tsf/package.sh.
set -euo pipefail
web="$(cd "$(dirname "$0")/.." && pwd)"
root="$(cd "$web/.." && pwd)"
"$root/crates/seokdok-tsf/package.sh"
"$root/crates/seokdok-fcitx5/package.sh"
version=$(cargo metadata --no-deps --format-version 1 --manifest-path "$root/Cargo.toml" | jq -r '.packages[] | select(.name=="seokdok-ffi") | .version')
win="seokdok-${version}-x86_64-windows-setup.exe"
linux="seokdok-${version}-x86_64-linux-fcitx5.tar.gz"
android="seokdok-${version}-android.apk"

apk="${SEOKDOK_APK:-}"
if [ -z "$apk" ]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  run=$(gh run list --workflow android.yml --branch main --status success --json databaseId,event -q '[.[] | select(.event == "push")][0].databaseId')
  gh run download "$run" --name seokdok-release-apk --dir "$tmp" ||
    { echo "run $run has no seokdok-release-apk; the signing secrets are not set" >&2; exit 1; }
  apk="$tmp/app-seokdok-release.apk"
fi
[ -f "$apk" ] || { echo "no APK at $apk" >&2; exit 1; }

rm -rf "$web/public/downloads"
mkdir -p "$web/public/downloads"
cp "$root/target/stage/$win" "$root/target/stage/$linux" "$web/public/downloads/"
cp "$apk" "$web/public/downloads/$android"
jq -n --arg v "$version" --arg w "$win" --arg l "$linux" --arg a "$android" \
  --argjson ws "$(stat -c %s "$web/public/downloads/$win")" \
  --argjson ls "$(stat -c %s "$web/public/downloads/$linux")" \
  --argjson as "$(stat -c %s "$web/public/downloads/$android")" \
  '{version: $v, windows: {file: $w, size: $ws}, linux: {file: $l, size: $ls}, android: {file: $a, size: $as}}' >"$web/src/lib/downloads.json"
# Cloudflare refuses an asset over 25 MiB, and the whole deploy with it.
for f in "$web/public/downloads/"*; do
  [ "$(stat -c %s "$f")" -le 26214400 ] || { echo "$f is over 25 MiB: the site cannot serve it" >&2; exit 1; }
done
cat "$web/src/lib/downloads.json"
