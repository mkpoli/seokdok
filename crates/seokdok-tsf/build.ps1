<#
  build.ps1 - build seokdok.dll (x64, release, MSVC) and stage the install
  folder (dll + dict) under target\stage.

  Prereqs: rustup and the MSVC build tools + Windows SDK ("Desktop development
  with C++"). If link.exe or rc.exe is not found, run from the "x64 Native
  Tools Command Prompt for VS 2022".
#>
$ErrorActionPreference = 'Stop'
$repo = Resolve-Path (Join-Path $PSScriptRoot '..\..')
Set-Location $repo

Write-Host "==> rustup target add x86_64-pc-windows-msvc"
rustup target add x86_64-pc-windows-msvc

Write-Host "==> cargo build --release --target x86_64-pc-windows-msvc -p seokdok-tsf"
cargo build --release --target x86_64-pc-windows-msvc -p seokdok-tsf
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

$dll = Join-Path $repo 'target\x86_64-pc-windows-msvc\release\seokdok.dll'
$stage = Join-Path $repo 'target\stage'
Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path (Join-Path $stage 'dict') | Out-Null
Copy-Item $dll $stage
# The engine loads dict\*.tsv next to the DLL. user*.tsv belong in %APPDATA%\seokdok.
Copy-Item (Join-Path $repo 'dict\*.tsv') (Join-Path $stage 'dict') -Exclude 'user.tsv', 'user-sino.tsv'
Copy-Item (Join-Path $PSScriptRoot 'install.ps1') $stage
Copy-Item (Join-Path $PSScriptRoot 'LICENSE') $stage

Write-Host ""
Write-Host "Built: $dll"
Write-Host "Staged: $stage  (seokdok.dll + dict\)"
if (Get-Command dumpbin.exe -ErrorAction SilentlyContinue) {
    & dumpbin.exe /nologo /exports $dll |
        Select-String 'DllGetClassObject|DllCanUnloadNow|DllRegisterServer|DllUnregisterServer|ShowSettings'
}
Write-Host ""
Write-Host "Next:  $stage\install.ps1   (copies to Program Files and registers; asks for UAC)"
