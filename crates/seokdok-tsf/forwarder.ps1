# Build the Arm64X pure forwarder <id>.dll into target/forwarder. On Arm64
# Windows an x64 (emulated) process loads the x64 DLL and a native process the Arm64 one,
# from the single path the COM server is registered under. The forwarder holds
# no code: it sends x64 callers to <id>-x64.dll and Arm64 callers to <id>-arm64.dll.
# Needs the MSVC Arm64 tools on PATH (an Arm64 developer prompt, or
# ilammy/msvc-dev-cmd with arch amd64_arm64). Recipe:
# https://learn.microsoft.com/en-us/windows/arm/arm64x-build#building-an-arm64x-pure-forwarder-dll
param([string]$Id = 'seokdok')
$ErrorActionPreference = 'Stop'
$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$work = Join-Path $root "target\forwarder-work\$Id"
$out = Join-Path $root 'target\forwarder'
Remove-Item $work -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $work, $out | Out-Null
Set-Location $work

$exports = 'DllGetClassObject', 'DllCanUnloadNow', 'DllRegisterServer', 'DllUnregisterServer'
foreach ($arch in 'x64', 'arm64') {
    $lines = @('EXPORTS') + ($exports | ForEach-Object { "    $_ = $Id-$arch.$_" })
    Set-Content "$Id-$arch.def" $lines -Encoding ascii
}
Set-Content empty.cpp '' -Encoding ascii

function Run { & $args[0] $args[1..($args.Length - 1)]; if ($LASTEXITCODE -ne 0) { throw "failed: $args" } }
Run cl /nologo /c /Foempty_arm64.obj empty.cpp
Run cl /nologo /c /arm64EC /Foempty_x64.obj empty.cpp
Run link /nologo /lib /machine:x64 "/def:$Id-x64.def" "/out:$Id-x64.lib"
Run link /nologo /lib /machine:arm64 "/def:$Id-arm64.def" "/out:$Id-arm64.lib"
Run link /nologo /dll /noentry /machine:arm64x "/defArm64Native:$Id-arm64.def" "/def:$Id-x64.def" empty_arm64.obj empty_x64.obj "/out:$out\$Id.dll" "$Id-arm64.lib" "$Id-x64.lib"
Get-Item "$out\$Id.dll"
