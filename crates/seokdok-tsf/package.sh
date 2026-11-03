#!/usr/bin/env bash
# Build the Windows IME on Linux (cargo-xwin) and stage a release zip:
#   target/stage/seokdok-<version>-x86_64-windows/
#     seokdok.dll  install.ps1  dict/*.tsv  LICENSE  README.txt
# Usage: crates/seokdok-tsf/package.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
version=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name=="seokdok-tsf") | .version')
cargo xwin build --release --target x86_64-pc-windows-msvc -p seokdok-tsf
name="seokdok-${version}-x86_64-windows"
stage="target/stage/${name}"
rm -rf "$stage"
mkdir -p "$stage/dict"
cp target/x86_64-pc-windows-msvc/release/seokdok.dll "$stage/"
cp crates/seokdok-tsf/install.ps1 "$stage/"
cp LICENSE "$stage/LICENSE"
for f in dict/*.tsv; do
  case "$(basename "$f")" in user*.tsv) ;; *) cp "$f" "$stage/dict/" ;; esac
done
cp dict/README.md dict/LICENSE-fusion.txt "$stage/dict/"
cat > "$stage/README.txt" <<'TXT'
釋讀 seokdok — Windows input method

Install:   right-click install.ps1 > Run with PowerShell (asks for administrator rights)
Uninstall: powershell -ExecutionPolicy Bypass -File install.ps1 -Uninstall
Then choose 釋讀 (seokdok) with Win+Space. Korean must be in Windows' language list.

Keys: type 두벌식 Hangul; Space converts, Space again opens the candidates;
1-9 pick, Tab/PgDn page, ←→ move between words, Shift+←→ resize a word,
Enter confirms, Esc closes / reverts. Han/Eng key or a tap of Alt: 釋讀 ↔ A.

Settings: %APPDATA%\seokdok\config.toml (or the language-bar menu).
Own words and rules: export them from https://seokdok.mkpo.li/#custom and save
the file as %APPDATA%\seokdok\custom.json. Switch input method away and back to reload.

Code: MIT. Dictionaries: see dict\README.md (libhangul BSD-3, FusionIME GPL-3.0,
JMdict CC BY-SA 4.0, 釋讀會 community dictionary).
TXT
(cd target/stage && rm -f "${name}.zip" && python3 -c "import shutil,sys; shutil.make_archive(sys.argv[1], 'zip', '.', sys.argv[1])" "$name")
ls -la "target/stage/${name}.zip"
