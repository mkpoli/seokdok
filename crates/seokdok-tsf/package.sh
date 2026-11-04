#!/usr/bin/env bash
# Build the Windows IME on Linux (cargo-xwin) and its installer:
#   target/stage/seokdok-<version>-x86_64-windows-setup.exe
# The installer is compiled by Inno Setup 6.7+ on the Windows side of WSL
# (winget install JRSoftware.InnoSetup). Usage: crates/seokdok-tsf/package.sh
# The settings app is built by the Windows settings workflow, since WinUI builds
# only on Windows (artifact seokdok-settings-app of the latest successful push
# run on main); set SEOKDOK_SETTINGS_APP to a local copy of that folder to use
# it instead.
set -euo pipefail
cd "$(dirname "$0")/../.."
root="$PWD"
version=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name=="seokdok-tsf") | .version')
command -v llvm-rc >/dev/null || export PATH="$root/tools/rc:$PATH"
touch crates/seokdok-tsf/resources.rc
cargo xwin build --release --target x86_64-pc-windows-msvc -p seokdok-tsf
# Without a resource compiler the build only warns, and the IME ships with no icons.
dll="$root/target/x86_64-pc-windows-msvc/release/seokdok.dll"
python3 -c "import sys; d=open(sys.argv[1],'rb').read(4096); sys.exit(b'.rsrc' not in d)" "$dll" ||
  { echo "seokdok.dll has no resources: llvm-rc failed; see the warnings above" >&2; exit 1; }

app="${SEOKDOK_SETTINGS_APP:-}"
if [ -z "$app" ]; then
  app="$(mktemp -d)"
  trap 'rm -rf "$app"' EXIT
  run=$(gh run list --workflow windows-settings.yml --branch main --status success --json databaseId,event -q '[.[] | select(.event == "push")][0].databaseId')
  [ -n "$run" ] || { echo "no successful run of windows-settings.yml on main" >&2; exit 1; }
  built=$(gh run view "$run" --json headSha -q .headSha)
  [ "$built" = "$(git rev-parse HEAD)" ] || echo "note: the settings app comes from $built, the input method from $(git rev-parse HEAD)" >&2
  gh run download "$run" --name seokdok-settings-app --dir "$app"
fi

stage="target/stage/windows"
crates/seokdok-tsf/stage.sh "$app"

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
