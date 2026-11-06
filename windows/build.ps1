# Build the settings app of a product into target/settings-app (target/settings-app-<id>
# for a product other than seokdok, with -arm64 added for -Arch arm64): the app, the
# runtime it carries, seokdok_ffi.dll and what it reads beside itself. Used on a
# Windows runner by windows-settings.yml, windows-arm64.yml and release-windows.yml.
param([string]$Product = 'seokdok', [ValidateSet('x64', 'arm64')][string]$Arch = 'x64')
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root
$definition = Get-Content "products\$Product.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$exe = $definition.windows.settings_exe
$suffix = if ($Arch -eq 'arm64') { '-arm64' } else { '' }
$out = Join-Path $root $(if ($Product -eq 'seokdok') { "target\settings-app$suffix" } else { "target\settings-app-$Product$suffix" })
$triple = if ($Arch -eq 'arm64') { 'aarch64-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }
$features = if ($Product -eq 'seokdok') { @() } else { @('--features', $Product) }

rustup target add $triple
cargo build --release --target $triple -p seokdok-ffi @features
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.*)"' | Select-Object -First 1).Matches[0].Groups[1].Value
# Two products share the project folder, and what it compiled names the assembly.
Remove-Item windows\SeokdokSettings\obj, windows\SeokdokSettings\bin -Recurse -Force -ErrorAction SilentlyContinue
dotnet publish windows\SeokdokSettings\SeokdokSettings.csproj -c Release -o $out "-p:Version=$version" "-p:SeokdokProduct=$Product" "-p:SeokdokArch=$Arch" "-p:AssemblyName=$exe"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Copy-Item target\$triple\release\seokdok_ffi.dll $out
Remove-Item "$out\*.pdb" -ErrorAction SilentlyContinue
$size = (Get-ChildItem $out -Recurse -File | Measure-Object Length -Sum).Sum
Write-Host "settings app: $([math]::Round($size / 1MB, 1)) MB in $out"
