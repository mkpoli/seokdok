# Add the input method to the current user's Korean keyboards (Settings > Time & language
# > Language > Korean > Keyboards), adding Korean when it is missing, so it
# shows in Win+Space. -Remove takes it out again. Run by the installer as the
# user who started it.
param([Parameter(Mandatory)][string]$Clsid, [Parameter(Mandatory)][string]$ProfileId, [switch]$Remove)
$ErrorActionPreference = 'Stop'
$tip = "0412:{$Clsid}{$ProfileId}"
$list = Get-WinUserLanguageList
$ko = $list | Where-Object { $_.LanguageTag -like 'ko*' } | Select-Object -First 1
if ($Remove) {
    if ($ko -and $ko.InputMethodTips -contains $tip) {
        [void]$ko.InputMethodTips.Remove($tip)
        Set-WinUserLanguageList $list -Force -WarningAction SilentlyContinue
    }
    return
}
if (-not $ko) {
    $list.Add('ko')
    $ko = $list | Where-Object { $_.LanguageTag -like 'ko*' } | Select-Object -First 1
}
if ($ko.InputMethodTips -notcontains $tip) { $ko.InputMethodTips.Add($tip) }
Set-WinUserLanguageList $list -Force -WarningAction SilentlyContinue
