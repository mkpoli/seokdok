#!/usr/bin/env bash
# Collect the files the Windows installer carries into target/stage/windows
# (target/stage/windows-<id> for a product other than seokdok): the DLL (already
# built for x86_64-pc-windows-msvc), LICENSE, the shipped dictionaries, the
# settings app, the folder given as the first argument, and product.iss for the
# installer script. Used by package.sh and by the release workflow.
# With arch arm64 the stage is target/stage/windows[-<id>]-arm64 and holds the
# Arm64X forwarder <id>.dll (crates/seokdok-tsf/forwarder.ps1) with the DLLs
# it forwards to, <id>-arm64.dll and <id>-x64.dll.
set -euo pipefail
app="$(realpath "${1:?usage: stage.sh <settings app folder> [product] [x64|arm64]}")"
id="${2:-seokdok}"
arch="${3:-x64}"
cd "$(dirname "$0")/../.."
product="products/$id.json"
exe="$(jq -r .windows.settings_exe "$product")"
[ -f "$app/$exe.exe" ] || { echo "no $exe.exe in $app" >&2; exit 1; }
stage="target/stage/windows"
[ "$id" = seokdok ] || stage="target/stage/windows-$id"
[ "$arch" = x64 ] || stage="$stage-$arch"
rm -rf "$stage"
mkdir -p "$stage/dict"
if [ "$arch" = arm64 ]; then
  cp "target/forwarder/$id.dll" "$stage/$id.dll"
  cp target/aarch64-pc-windows-msvc/release/seokdok.dll "$stage/$id-arm64.dll"
  cp target/x86_64-pc-windows-msvc/release/seokdok.dll "$stage/$id-x64.dll"
else
  cp target/x86_64-pc-windows-msvc/release/seokdok.dll "$stage/$id.dll"
fi
jq -r '[
  "#define ProductId \"\(.id)\"",
  "#define ProductName \"\(.hanja) \(.latin)\"",
  "#define AppGuid \"\(.windows.app_id)\"",
  "#define Clsid \"\(.windows.clsid)\"",
  "#define ProfileGuid \"\(.windows.profile)\"",
  "#define SettingsExe \"\(.windows.settings_exe)\"",
  "#define Guide \"\(.windows.setup_guide)\"",
  "#define BackLight \"\(.windows.setup_back_light)\"",
  "#define BackDark \"\(.windows.setup_back_dark)\""
] | join("\n")' "$product" >"$stage/product.iss"
[ "$arch" = x64 ] || echo "#define Arm64" >>"$stage/product.iss"
cp LICENSE "$stage/LICENSE"
for f in dict/*.tsv; do
  case "$(basename "$f")" in user*.tsv) ;; *) cp "$f" "$stage/dict/" ;; esac
done
cp dict/README.md dict/LICENSE-fusion.txt "$stage/dict/"
cp -r "$app" "$stage/settings"
