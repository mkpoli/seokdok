#!/usr/bin/env bash
# Build the Windows IME on Linux (cargo-xwin) and its installer:
#   target/stage/seokdok-<version>-x86_64-windows-setup.exe
# The installer is compiled by Inno Setup 6.7+ on the Windows side of WSL
# (winget install JRSoftware.InnoSetup). Usage: crates/seokdok-tsf/package.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
root="$PWD"
version=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name=="seokdok-tsf") | .version')
cargo xwin build --release --target x86_64-pc-windows-msvc -p seokdok-tsf

stage="target/stage/windows"
crates/seokdok-tsf/stage.sh

# Compile on Windows: copy the script, artwork and stage to its temp folder.
wintemp=$(cmd.exe /c 'echo %TEMP%' 2>/dev/null | tr -d '\r')
work="$(wslpath "$wintemp")/seokdok-installer"
rm -rf "$work"
mkdir -p "$work"
cp -r crates/seokdok-tsf/installer/. "$work/"
cp -r "$stage" "$work/stage"
iscc=$(powershell.exe -NoProfile -Command '
  @("$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1' | tr -d '\r')
[ -n "$iscc" ] || { echo "Inno Setup not found: winget install JRSoftware.InnoSetup" >&2; exit 1; }
(cd /tmp && powershell.exe -NoProfile -Command "& '$iscc' /Q /DMyAppVersion=$version /DStage=stage '/O$wintemp\\seokdok-installer\\out' '$wintemp\\seokdok-installer\\seokdok.iss'; exit \$LASTEXITCODE")
out="seokdok-${version}-x86_64-windows-setup.exe"
cp "$work/out/$out" "target/stage/$out"
ls -la "$root/target/stage/$out"
