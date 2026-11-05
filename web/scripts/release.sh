#!/usr/bin/env bash
# Package the Windows and Linux input methods, add the Android APK and macOS zip and copy
# them to public/downloads/ for the site; src/lib/downloads.json records names
# and sizes.
# Needs cargo-xwin (Windows), the fcitx5 headers (FCITX5_INCLUDE or pkg-config)
# and gh. The APK is the signed release build from the Android workflow
# (artifact seokdok-release-apk of the latest successful push run on main); set
# SEOKDOK_APK to a local signed app-seokdok-release.apk to use that instead. The macOS
# zip is the ad-hoc signed app from the macOS workflow, or SEOKDOK_MACOS_ZIP, and
# the IPA the unsigned build from the iOS workflow, or SEOKDOK_IPA. The Windows
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
macos="seokdok-${version}-macos.zip"
ios="seokdok-${version}-ios-unsigned.ipa"

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

mac="${SEOKDOK_MACOS_ZIP:-}"
if [ -z "$mac" ]; then
  tmpm="$(mktemp -d)"
  run=$(gh run list --workflow macos.yml --branch main --status success --json databaseId,event -q '[.[] | select(.event == "push")][0].databaseId')
  gh run download "$run" --name Seokdok-unsigned-app --dir "$tmpm"
  mac="$tmpm/Seokdok-unsigned.zip"
fi
[ -f "$mac" ] || { echo "no macOS zip at $mac" >&2; exit 1; }

ipa="${SEOKDOK_IPA:-}"
if [ -z "$ipa" ]; then
  tmpi="$(mktemp -d)"
  run=$(gh run list --workflow ios.yml --branch main --status success --json databaseId,event -q '[.[] | select(.event == "push")][0].databaseId')
  gh run download "$run" --name Seokdok-unsigned-ipa --dir "$tmpi"
  ipa="$tmpi/Seokdok-unsigned.ipa"
fi
[ -f "$ipa" ] || { echo "no iOS IPA at $ipa" >&2; exit 1; }

rm -rf "$web/public/downloads"
mkdir -p "$web/public/downloads"
cp "$root/target/stage/$win" "$root/target/stage/$linux" "$web/public/downloads/"
cp "$apk" "$web/public/downloads/$android"
cp "$mac" "$web/public/downloads/$macos"
cp "$ipa" "$web/public/downloads/$ios"
jq -n --arg v "$version" --arg w "$win" --arg l "$linux" --arg a "$android" --arg m "$macos" --arg i "$ios" \
  --argjson ws "$(stat -c %s "$web/public/downloads/$win")" \
  --argjson ls "$(stat -c %s "$web/public/downloads/$linux")" \
  --argjson as "$(stat -c %s "$web/public/downloads/$android")" \
  --argjson ms "$(stat -c %s "$web/public/downloads/$macos")" \
  --argjson is "$(stat -c %s "$web/public/downloads/$ios")" \
  '{version: $v, windows: {file: $w, size: $ws}, linux: {file: $l, size: $ls}, android: {file: $a, size: $as}, macos: {file: $m, size: $ms}, ios: {file: $i, size: $is}}' >"$web/src/lib/downloads.json"
# Cloudflare refuses an asset over 25 MiB, and the whole deploy with it.
for f in "$web/public/downloads/"*; do
  [ "$(stat -c %s "$f")" -le 26214400 ] || { echo "$f is over 25 MiB: the site cannot serve it" >&2; exit 1; }
done
cat "$web/src/lib/downloads.json"
