; Inno Setup 6.7 script for the Windows input methods. Built by
; crates/seokdok-tsf/package.sh, which stages the files and runs:
;   ISCC.exe /DMyAppVersion=0.1.0 /DStage=<stage folder> seokdok.iss
; The stage folder holds the DLL, dict\*.tsv, LICENSE, settings\, the settings
; app, and product.iss, the product's identifiers from products/<id>.json.
; What the wizard says is in messages-<id>.iss.
; An Arm64 stage defines Arm64 in product.iss. It holds <id>.dll, an Arm64X
; forwarder, with <id>-arm64.dll and <id>-x64.dll: the forwarder is the one COM
; server, so native and emulated x64 applications both reach their own DLL.

#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif
#ifndef Stage
  #define Stage "stage"
#endif
#include Stage + "\product.iss"
#define Art ProductId == "seokdok" ? "art\" : "art\" + ProductId + "\"
#define Sides(T) Art + "side-" + T + "-164x314.png," + Art + "side-" + T + "-205x393.png," + Art + "side-" + T + "-246x471.png," + Art + "side-" + T + "-328x628.png," + Art + "side-" + T + "-410x785.png"
#define Marks(T) Art + "mark-" + T + "-55.png," + Art + "mark-" + T + "-69.png," + Art + "mark-" + T + "-83.png," + Art + "mark-" + T + "-110.png," + Art + "mark-" + T + "-138.png"

[Setup]
AppId={{{#AppGuid}}
AppName={#ProductName}
AppVersion={#MyAppVersion}
AppVerName={#ProductName} {#MyAppVersion}
VersionInfoVersion={#MyAppVersion}
AppPublisher=mkpoli
AppPublisherURL=https://seokdok.mkpo.li/
AppSupportURL=https://github.com/mkpoli/seokdok
DefaultDirName={autopf}\{#ProductId}
DisableProgramGroupPage=yes
DisableDirPage=yes
DisableReadyPage=yes
#ifdef Preview
; A look at the wizard without elevation (screenshots only, never shipped).
PrivilegesRequired=lowest
#else
PrivilegesRequired=admin
#endif
#ifdef Arm64
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
OutputBaseFilename={#ProductId}-{#MyAppVersion}-arm64-windows-setup
#else
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputBaseFilename={#ProductId}-{#MyAppVersion}-x86_64-windows-setup
#endif
Compression=lzma2/ultra64
LZMANumFastBytes=273
LZMAUseSeparateProcess=yes
SolidCompression=yes
LicenseFile={#Stage}\LICENSE
UninstallDisplayName={#ProductName}
#ifdef Arm64
UninstallDisplayIcon={app}\{#ProductId}-arm64.dll
#else
UninstallDisplayIcon={app}\{#ProductId}.dll
#endif
ShowLanguageDialog=auto
; The loaded DLL is moved aside (MoveLoadedDll), so nobody's apps need closing.
CloseApplications=no
RestartApplications=no
; The look: Windows 11 styling that follows the system's light or dark mode,
; with the product's poster as the side panel.
#ifdef Appearance
WizardStyle=modern {#Appearance} windows11 includetitlebar
#else
WizardStyle=modern dynamic windows11 includetitlebar
#endif
WizardImageFile={#Sides("light")}
WizardImageFileDynamicDark={#Sides("dark")}
WizardSmallImageFile={#Marks("light")}
WizardSmallImageFileDynamicDark={#Marks("dark")}
WizardImageBackColor={#BackLight}
WizardImageBackColorDynamicDark={#BackDark}

[Languages]
Name: "korean"; MessagesFile: "i18n\Korean.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[InstallDelete]
Type: files; Name: "{app}\{#ProductId}-old-*.dll"
#ifdef Arm64
Type: files; Name: "{app}\{#ProductId}-*-old-*.dll"
#endif
Type: filesandordirs; Name: "{app}\settings"

[UninstallDelete]
Type: files; Name: "{app}\{#ProductId}-old-*.dll"
#ifdef Arm64
Type: files; Name: "{app}\{#ProductId}-*-old-*.dll"
#endif

[Files]
; regserver calls DllRegisterServer on install and DllUnregisterServer on
; uninstall. A DLL loaded by a running app can be renamed but not overwritten:
; the old one moves aside, running apps keep it until reopened, and no reboot
; is needed. Setup deletes the moved copies it can on the next update.
#ifdef Arm64
; The forwarder is registered; the two DLLs it forwards to are only copied.
Source: "{#Stage}\{#ProductId}-arm64.dll"; DestDir: "{app}"; BeforeInstall: MoveLoadedDll('{#ProductId}-arm64'); Flags: ignoreversion restartreplace uninsrestartdelete
Source: "{#Stage}\{#ProductId}-x64.dll"; DestDir: "{app}"; BeforeInstall: MoveLoadedDll('{#ProductId}-x64'); Flags: ignoreversion restartreplace uninsrestartdelete
Source: "{#Stage}\{#ProductId}.dll"; DestDir: "{app}"; BeforeInstall: MoveLoadedDll('{#ProductId}'); Flags: ignoreversion regserver restartreplace uninsrestartdelete
#else
Source: "{#Stage}\{#ProductId}.dll"; DestDir: "{app}"; BeforeInstall: MoveLoadedDll('{#ProductId}'); Flags: ignoreversion regserver restartreplace uninsrestartdelete
#endif
; The engine reads dict\*.tsv next to the DLL.
Source: "{#Stage}\dict\*"; DestDir: "{app}\dict"; Flags: ignoreversion recursesubdirs
Source: "{#Stage}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "korean-keyboard.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\settings\*"; DestDir: "{app}\settings"; Flags: ignoreversion recursesubdirs

[Icons]
; Settings entry point; the input indicator's menu is the other.
Name: "{autoprograms}\{#ProductName} 設定"; Filename: "{app}\settings\{#SettingsExe}.exe"; WorkingDir: "{app}\settings"

[Run]
; Korean keyboards are per user: add the input method for the user who ran Setup.
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\korean-keyboard.ps1"" -Clsid {#Clsid} -ProfileId {#ProfileGuid}"; Flags: runasoriginaluser runhidden waituntilterminated; StatusMsg: "{cm:AddKeyboard}"
Filename: "{#Guide}"; Description: "{cm:OpenGuide}"; Flags: postinstall shellexec skipifsilent unchecked

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/f /im {#SettingsExe}.exe"; Flags: runhidden waituntilterminated; RunOnceId: "CloseSettings"
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\korean-keyboard.ps1"" -Clsid {#Clsid} -ProfileId {#ProfileGuid} -Remove"; Flags: runhidden waituntilterminated; RunOnceId: "RemoveKeyboard"

#include "messages-" + ProductId + ".iss"

[Code]
// A settings window left open would hold its files against the update.
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Code: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/f /im {#SettingsExe}.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  Result := '';
end;

procedure MoveLoadedDll(Name: String);
var
  Dll: String;
begin
  Dll := ExpandConstant('{app}\') + Name + '.dll';
  if FileExists(Dll) then
    RenameFile(Dll, ExpandConstant('{app}\') + Name + '-old-' + GetDateTimeString('yyyymmddhhnnss', #0, #0) + '.dll');
end;
