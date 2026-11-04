#!/usr/bin/env bash
# Collect the files the Windows installer carries into target/stage/windows:
# seokdok.dll (already built for x86_64-pc-windows-msvc), LICENSE, the shipped
# dictionaries and the settings app, the folder given as the first argument.
# Used by package.sh and by the release workflow.
set -euo pipefail
app="$(realpath "${1:?usage: stage.sh <settings app folder>}")"
[ -f "$app/SeokdokSettings.exe" ] || { echo "no SeokdokSettings.exe in $app" >&2; exit 1; }
cd "$(dirname "$0")/../.."
stage="target/stage/windows"
rm -rf "$stage"
mkdir -p "$stage/dict"
cp target/x86_64-pc-windows-msvc/release/seokdok.dll "$stage/"
cp LICENSE "$stage/LICENSE"
for f in dict/*.tsv; do
  case "$(basename "$f")" in user*.tsv) ;; *) cp "$f" "$stage/dict/" ;; esac
done
cp dict/README.md dict/LICENSE-fusion.txt "$stage/dict/"
cp -r "$app" "$stage/settings"
