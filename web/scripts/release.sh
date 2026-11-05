#!/usr/bin/env bash
# Package the Windows and Linux input methods of both products, add their
# Android APKs and copy them to public/downloads/ for the site;
# src/lib/downloads.json records names and sizes.
# Needs cargo-xwin (Windows), the fcitx5 headers (FCITX5_INCLUDE or pkg-config)
# and gh. The APK is the signed release build from the Android workflow
# (artifact seokdok-release-apk of the latest successful push run on main); set
# SEOKDOK_APK to a local signed app-release.apk to use that instead. The Windows
# installer takes the settings app the same way: see crates/seokdok-tsf/package.sh.
# 音讀 takes artifact eumdok-release-apk, or EUMDOK_APK.
set -euo pipefail
web="$(cd "$(dirname "$0")/.." && pwd)"
root="$(cd "$web/.." && pwd)"
"$root/crates/seokdok-tsf/package.sh"
"$root/crates/seokdok-tsf/package.sh" eumdok
"$root/crates/seokdok-fcitx5/package.sh"
"$root/crates/seokdok-fcitx5/package.sh" eumdok
version=$(cargo metadata --no-deps --format-version 1 --manifest-path "$root/Cargo.toml" | jq -r '.packages[] | select(.name=="seokdok-ffi") | .version')
win="seokdok-${version}-x86_64-windows-setup.exe"
linux="seokdok-${version}-x86_64-linux-fcitx5.tar.gz"
android="seokdok-${version}-android.apk"
ewin="eumdok-${version}-x86_64-windows-setup.exe"
elinux="eumdok-${version}-x86_64-linux-fcitx5.tar.gz"
eandroid="eumdok-${version}-android.apk"

apk="${SEOKDOK_APK:-}"
if [ -z "$apk" ]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  run=$(gh run list --workflow android.yml --branch main --status success --json databaseId,event -q '[.[] | select(.event == "push")][0].databaseId')
  gh run download "$run" --name seokdok-release-apk --dir "$tmp" ||
    { echo "run $run has no seokdok-release-apk; the signing secrets are not set" >&2; exit 1; }
  apk="$tmp/app-release.apk"
fi
[ -f "$apk" ] || { echo "no APK at $apk" >&2; exit 1; }

eapk="${EUMDOK_APK:-}"
if [ -z "$eapk" ]; then
  etmp="$(mktemp -d)"
  run=$(gh run list --workflow android.yml --branch main --status success --json databaseId,event -q '[.[] | select(.event == "push")][0].databaseId')
  gh run download "$run" --name eumdok-release-apk --dir "$etmp" ||
    { echo "run $run has no eumdok-release-apk; the signing secrets are not set" >&2; exit 1; }
  eapk="$(ls "$etmp"/*.apk)"
fi
[ -f "$eapk" ] || { echo "no 音讀 APK at $eapk" >&2; exit 1; }

rm -rf "$web/public/downloads"
mkdir -p "$web/public/downloads"
cp "$root/target/stage/$win" "$root/target/stage/$linux" "$root/target/stage/$ewin" "$root/target/stage/$elinux" "$web/public/downloads/"
cp "$apk" "$web/public/downloads/$android"
cp "$eapk" "$web/public/downloads/$eandroid"
jq -n --arg v "$version" --arg w "$win" --arg l "$linux" --arg a "$android" \
  --arg ew "$ewin" --arg el "$elinux" --arg ea "$eandroid" \
  --argjson ews "$(stat -c %s "$web/public/downloads/$ewin")" \
  --argjson els "$(stat -c %s "$web/public/downloads/$elinux")" \
  --argjson eas "$(stat -c %s "$web/public/downloads/$eandroid")" \
  --argjson ws "$(stat -c %s "$web/public/downloads/$win")" \
  --argjson ls "$(stat -c %s "$web/public/downloads/$linux")" \
  --argjson as "$(stat -c %s "$web/public/downloads/$android")" \
  '{version: $v, windows: {file: $w, size: $ws}, linux: {file: $l, size: $ls}, android: {file: $a, size: $as}, eumdok: {windows: {file: $ew, size: $ews}, linux: {file: $el, size: $els}, android: {file: $ea, size: $eas}}}' >"$web/src/lib/downloads.json"
# Cloudflare refuses an asset over 25 MiB, and the whole deploy with it.
for f in "$web/public/downloads/"*; do
  [ "$(stat -c %s "$f")" -le 26214400 ] || { echo "$f is over 25 MiB: the site cannot serve it" >&2; exit 1; }
done
cat "$web/src/lib/downloads.json"
