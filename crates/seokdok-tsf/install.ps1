<#
  install.ps1 - install and register seokdok.

  Self-elevates (UAC). Copies seokdok.dll and the dict folder into
  "%ProgramFiles%\seokdok" and registers the DLL. The profile is registered
  under ko-KR and enabled by default; it appears in the input switcher
  (Win+Space) under Korean. Korean must be in the Windows language list.

  A DLL that is still loaded by an app is renamed aside before the new one is
  copied.

  Usage:
    .\install.ps1              # install + register
    .\install.ps1 -Uninstall   # unregister + remove

  Run it from target\stage (made by build.ps1), from a release zip, or from
  crates\seokdok-tsf in a checkout that has been built.
#>
param([switch]$Uninstall)
$ErrorActionPreference = 'Stop'

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    $a = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$PSCommandPath`"")
    if ($Uninstall) { $a += '-Uninstall' }
    Start-Process powershell.exe -Verb RunAs -ArgumentList $a
    return
}

$installDir = Join-Path $env:ProgramFiles 'seokdok'
$dst        = Join-Path $installDir 'seokdok.dll'

function Move-LockedAside([string]$path) {
    if (-not (Test-Path $path)) { return }
    & regsvr32.exe /u /s $path
    $old = Join-Path (Split-Path $path) ("seokdok-old-{0}.dll" -f (Get-Random))
    try {
        Rename-Item $path $old -Force
    } catch {
        throw "$path is locked (still loaded) and could not be renamed. Sign out and in, or reboot once, then retry."
    }
}

if ($Uninstall) {
    Move-LockedAside $dst
    Remove-Item $installDir -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "seokdok unregistered and removed."
    Read-Host "Press Enter to close"
    return
}

# Locate the DLL and the dict folder: release layout (next to this script) or
# a built checkout.
$repo = Resolve-Path (Join-Path $PSScriptRoot '..\..') -ErrorAction SilentlyContinue
$src = @(
    (Join-Path $PSScriptRoot 'seokdok.dll'),
    (Join-Path $repo 'target\x86_64-pc-windows-msvc\release\seokdok.dll')
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $src) { throw "seokdok.dll not found. Run build.ps1 first, or unzip the release." }
$dict = @(
    (Join-Path $PSScriptRoot 'dict'),
    (Join-Path $repo 'dict')
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $dict) { throw "dict folder not found next to the script or in the checkout." }

New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Move-LockedAside $dst
Copy-Item $src $dst -Force
$dictDst = Join-Path $installDir 'dict'
Remove-Item $dictDst -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $dictDst | Out-Null
Copy-Item (Join-Path $dict '*.tsv') $dictDst -Exclude 'user.tsv', 'user-sino.tsv'
Write-Host "Installed -> $installDir (seokdok.dll, dict\)"

& regsvr32.exe /s $dst
if ($LASTEXITCODE -ne 0) {
    throw "regsvr32 failed to register $dst (exit $LASTEXITCODE). Sign out and in, or reboot once, then retry."
}
Write-Host "Registered."

# Add the profile to the user's Korean input methods (Settings > Time &
# language > Language > Korean > Keyboards), so it shows in Win+Space.
try {
    $clsid = '{AAF18869-67ED-48A1-A3F6-42E961E994A1}'
    $prof = (Get-ChildItem "HKLM:\SOFTWARE\Microsoft\CTF\TIP\$clsid\LanguageProfile\0x00000412").PSChildName | Select-Object -First 1
    $tip = "0412:$clsid$prof"
    $list = Get-WinUserLanguageList
    $ko = $list | Where-Object { $_.LanguageTag -like 'ko*' } | Select-Object -First 1
    if (-not $ko) {
        $list.Add('ko')
        $ko = $list | Where-Object { $_.LanguageTag -like 'ko*' } | Select-Object -First 1
    }
    if ($ko.InputMethodTips -notcontains $tip) { $ko.InputMethodTips.Add($tip) }
    Set-WinUserLanguageList $list -Force -WarningAction SilentlyContinue
    Write-Host "Enabled under Korean."
} catch {
    Write-Host "Could not add it to the Korean keyboards; add it in Settings > Language > Korean > Keyboards."
}
Write-Host ""
Write-Host "Switch input (Win+Space) to 釋讀 (seokdok), listed under Korean."
Write-Host "User dictionaries go in %APPDATA%\seokdok (user.tsv, user-sino.tsv)."
Read-Host "Press Enter to close"
