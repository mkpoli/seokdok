# Build the settings app into target/settings-app: the app, the runtime it
# carries, seokdok_ffi.dll and what it reads beside itself. Used on a Windows
# runner by windows-settings.yml and release-windows.yml.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$out = Join-Path $root 'target\settings-app'
Set-Location $root

cargo build --release --target x86_64-pc-windows-msvc -p seokdok-ffi
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.*)"' | Select-Object -First 1).Matches[0].Groups[1].Value
dotnet publish windows\SeokdokSettings\SeokdokSettings.csproj -c Release -o $out "-p:Version=$version"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Copy-Item target\x86_64-pc-windows-msvc\release\seokdok_ffi.dll $out
Remove-Item "$out\*.pdb" -ErrorAction SilentlyContinue
$size = (Get-ChildItem $out -Recurse -File | Measure-Object Length -Sum).Sum
Write-Host "settings app: $([math]::Round($size / 1MB, 1)) MB in $out"
