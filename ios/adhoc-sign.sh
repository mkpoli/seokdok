#!/usr/bin/env bash
# Ad-hoc sign a device build of Seokdok.app with its entitlements, extension
# first. A sideloading tool reads them before re-signing with the user's
# Apple ID.
#
#   ios/adhoc-sign.sh path/to/Seokdok.app
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
app="$1"
codesign --force --sign - --timestamp=none \
  --entitlements "$here/Keyboard/SeokdokKeyboard.entitlements" \
  "$app/PlugIns/SeokdokKeyboard.appex"
codesign --force --sign - --timestamp=none \
  --entitlements "$here/App/Seokdok.entitlements" \
  "$app"
codesign --display --entitlements - "$app/PlugIns/SeokdokKeyboard.appex"
