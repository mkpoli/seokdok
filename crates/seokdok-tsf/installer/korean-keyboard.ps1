# Add 釋讀 to the current user's Korean keyboards (Settings > Time & language
# > Language > Korean > Keyboards), adding Korean when it is missing, so it
# shows in Win+Space. -Remove takes it out again. Run by the installer as the
# user who started it.
param([switch]$Remove)
$ErrorActionPreference = 'Stop'
$clsid = '{AAF18869-67ED-48A1-A3F6-42E961E994A1}'
$tip = "0412:$clsid{5A2C201D-E01B-475F-A23D-3A6A64D996A4}"
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
