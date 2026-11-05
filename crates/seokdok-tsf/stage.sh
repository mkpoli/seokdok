#!/usr/bin/env bash
# Collect the files the Windows installer carries into target/stage/windows
# (target/stage/windows-<id> for a product other than seokdok): the DLL (already
# built for x86_64-pc-windows-msvc), LICENSE, the shipped dictionaries, the
# settings app, the folder given as the first argument, and product.iss for the
# installer script. Used by package.sh and by the release workflow.
set -euo pipefail
app="$(realpath "${1:?usage: stage.sh <settings app folder> [product]}")"
id="${2:-seokdok}"
cd "$(dirname "$0")/../.."
product="products/$id.json"
exe="$(jq -r .windows.settings_exe "$product")"
[ -f "$app/$exe.exe" ] || { echo "no $exe.exe in $app" >&2; exit 1; }
stage="target/stage/windows"
[ "$id" = seokdok ] || stage="target/stage/windows-$id"
rm -rf "$stage"
mkdir -p "$stage/dict"
cp target/x86_64-pc-windows-msvc/release/seokdok.dll "$stage/$id.dll"
jq -r '[
  "#define ProductId \"\(.id)\"",
  "#define ProductName \"\(.hanja) \(.id)\"",
  "#define AppGuid \"\(.windows.app_id)\"",
  "#define Clsid \"\(.windows.clsid)\"",
  "#define ProfileGuid \"\(.windows.profile)\"",
  "#define SettingsExe \"\(.windows.settings_exe)\"",
  "#define Guide \"\(.windows.setup_guide)\"",
  "#define BackLight \"\(.windows.setup_back_light)\"",
  "#define BackDark \"\(.windows.setup_back_dark)\""
] | join("\n")' "$product" >"$stage/product.iss"
cp LICENSE "$stage/LICENSE"
for f in dict/*.tsv; do
  case "$(basename "$f")" in user*.tsv) ;; *) cp "$f" "$stage/dict/" ;; esac
done
cp dict/README.md dict/LICENSE-fusion.txt "$stage/dict/"
cp -r "$app" "$stage/settings"
