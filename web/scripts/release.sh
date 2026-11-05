#!/usr/bin/env bash
# Package the Windows and Linux input methods of both products, add their
# Android APKs and the macOS zip and iOS IPA, and copy them to public/downloads/
# for the site; src/lib/downloads.json records names and sizes.
# Needs cargo-xwin (Windows), the fcitx5 headers (FCITX5_INCLUDE or pkg-config)
# and gh. The APK is the signed release build from the Android workflow
# (artifact seokdok-release-apk of the latest successful push run on main); set
# SEOKDOK_APK to a local signed app-seokdok-release.apk to use that instead. The macOS
# zip is the ad-hoc signed app from the macOS workflow, or SEOKDOK_MACOS_ZIP, and
# the IPA the unsigned build from the iOS workflow, or SEOKDOK_IPA. The Windows
# installer takes the settings app the same way: see crates/seokdok-tsf/package.sh.
# 音讀 takes artifacts eumdok-release-apk and eumdok-settings-app, or EUMDOK_APK and
# EUMDOK_SETTINGS_APP.
set -euo pipefail
web="$(cd "$(dirname "$0")/.." && pwd)"
root="$(cd "$web/.." && pwd)"
"$root/crates/seokdok-tsf/package.sh"
SEOKDOK_SETTINGS_APP="${EUMDOK_SETTINGS_APP:-}" "$root/crates/seokdok-tsf/package.sh" eumdok
"$root/crates/seokdok-fcitx5/package.sh"
"$root/crates/seokdok-fcitx5/package.sh" eumdok
version=$(cargo metadata --no-deps --format-version 1 --manifest-path "$root/Cargo.toml" | jq -r '.packages[] | select(.name=="seokdok-ffi") | .version')
win="seokdok-${version}-x86_64-windows-setup.exe"
linux="seokdok-${version}-x86_64-linux-fcitx5.tar.gz"
android="seokdok-${version}-android.apk"
ewin="eumdok-${version}-x86_64-windows-setup.exe"
elinux="eumdok-${version}-x86_64-linux-fcitx5.tar.gz"
eandroid="eumdok-${version}-android.apk"
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

eapk="${EUMDOK_APK:-}"
if [ -z "$eapk" ]; then
  etmp="$(mktemp -d)"
  run=$(gh run list --workflow android.yml --branch main --status success --json databaseId,event -q '[.[] | select(.event == "push")][0].databaseId')
  gh run download "$run" --name eumdok-release-apk --dir "$etmp" ||
    { echo "run $run has no eumdok-release-apk; the signing secrets are not set" >&2; exit 1; }
  eapk="$(ls "$etmp"/*.apk)"
fi
[ -f "$eapk" ] || { echo "no 音讀 APK at $eapk" >&2; exit 1; }

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
cp "$root/target/stage/$win" "$root/target/stage/$linux" "$root/target/stage/$ewin" "$root/target/stage/$elinux" "$web/public/downloads/"
cp "$apk" "$web/public/downloads/$android"
cp "$eapk" "$web/public/downloads/$eandroid"
cp "$mac" "$web/public/downloads/$macos"
cp "$ipa" "$web/public/downloads/$ios"
jq -n --arg v "$version" --arg w "$win" --arg l "$linux" --arg a "$android" --arg m "$macos" --arg i "$ios" \
  --arg ew "$ewin" --arg el "$elinux" --arg ea "$eandroid" \
  --argjson ews "$(stat -c %s "$web/public/downloads/$ewin")" \
  --argjson els "$(stat -c %s "$web/public/downloads/$elinux")" \
  --argjson eas "$(stat -c %s "$web/public/downloads/$eandroid")" \
  --argjson ws "$(stat -c %s "$web/public/downloads/$win")" \
  --argjson ls "$(stat -c %s "$web/public/downloads/$linux")" \
  --argjson as "$(stat -c %s "$web/public/downloads/$android")" \
  --argjson ms "$(stat -c %s "$web/public/downloads/$macos")" \
  --argjson is "$(stat -c %s "$web/public/downloads/$ios")" \
  '{version: $v, windows: {file: $w, size: $ws}, linux: {file: $l, size: $ls}, android: {file: $a, size: $as}, macos: {file: $m, size: $ms}, ios: {file: $i, size: $is}, eumdok: {windows: {file: $ew, size: $ews}, linux: {file: $el, size: $els}, android: {file: $ea, size: $eas}}}' >"$web/src/lib/downloads.json"
# Cloudflare refuses an asset over 25 MiB, and the whole deploy with it.
for f in "$web/public/downloads/"*; do
  [ "$(stat -c %s "$f")" -le 26214400 ] || { echo "$f is over 25 MiB: the site cannot serve it" >&2; exit 1; }
done
cat "$web/src/lib/downloads.json"
