; Inno Setup 6.7 script for the 釋讀 (seokdok) Windows input method. Built by
; crates/seokdok-tsf/package.sh, which stages the files and runs:
;   ISCC.exe /DMyAppVersion=0.1.0 /DStage=<stage folder> seokdok.iss
; The stage folder holds seokdok.dll, dict\*.tsv and LICENSE.

#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif
#ifndef Stage
  #define Stage "stage"
#endif
#define Sides(T) "art\side-" + T + "-164x314.png,art\side-" + T + "-205x393.png,art\side-" + T + "-246x471.png,art\side-" + T + "-328x628.png,art\side-" + T + "-410x785.png"
#define Marks(T) "art\mark-" + T + "-55.png,art\mark-" + T + "-69.png,art\mark-" + T + "-83.png,art\mark-" + T + "-110.png,art\mark-" + T + "-138.png"

[Setup]
AppId={{041E579D-22D4-46E1-8D5C-B9BF7D6EC789}
AppName=釋讀 seokdok
AppVersion={#MyAppVersion}
AppVerName=釋讀 seokdok {#MyAppVersion}
VersionInfoVersion={#MyAppVersion}
AppPublisher=mkpoli
AppPublisherURL=https://seokdok.mkpo.li/
AppSupportURL=https://github.com/mkpoli/seokdok
DefaultDirName={autopf}\seokdok
DisableProgramGroupPage=yes
DisableDirPage=yes
DisableReadyPage=yes
#ifdef Preview
; A look at the wizard without elevation (screenshots only, never shipped).
PrivilegesRequired=lowest
#else
PrivilegesRequired=admin
#endif
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputBaseFilename=seokdok-{#MyAppVersion}-x86_64-windows-setup
Compression=lzma2/max
SolidCompression=yes
LicenseFile={#Stage}\LICENSE
UninstallDisplayName=釋讀 seokdok
UninstallDisplayIcon={app}\seokdok.dll
ShowLanguageDialog=auto
; The loaded DLL is moved aside (MoveLoadedDll), so nobody's apps need closing.
CloseApplications=no
RestartApplications=no
; The look: Windows 11 styling that follows the system's light or dark mode,
; with the 釋讀 poster as the side panel.
#ifdef Appearance
WizardStyle=modern {#Appearance} windows11 includetitlebar
#else
WizardStyle=modern dynamic windows11 includetitlebar
#endif
WizardImageFile={#Sides("light")}
WizardImageFileDynamicDark={#Sides("dark")}
WizardSmallImageFile={#Marks("light")}
WizardSmallImageFileDynamicDark={#Marks("dark")}
WizardImageBackColor=#24348c
WizardImageBackColorDynamicDark=#0c1124

[Languages]
Name: "korean"; MessagesFile: "i18n\Korean.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[InstallDelete]
Type: files; Name: "{app}\seokdok-old-*.dll"

[UninstallDelete]
Type: files; Name: "{app}\seokdok-old-*.dll"

[Files]
; regserver calls DllRegisterServer on install and DllUnregisterServer on
; uninstall. A DLL loaded by a running app can be renamed but not overwritten:
; the old one moves aside, running apps keep it until reopened, and no reboot
; is needed. Setup deletes the moved copies it can on the next update.
Source: "{#Stage}\seokdok.dll"; DestDir: "{app}"; BeforeInstall: MoveLoadedDll; Flags: ignoreversion regserver restartreplace uninsrestartdelete
; The engine reads dict\*.tsv next to the DLL.
Source: "{#Stage}\dict\*"; DestDir: "{app}\dict"; Flags: ignoreversion recursesubdirs
Source: "{#Stage}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "korean-keyboard.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
; Settings entry point; the input indicator's menu is the other.
Name: "{autoprograms}\釋讀 seokdok 設定"; Filename: "{sys}\rundll32.exe"; Parameters: """{app}\seokdok.dll"",ShowSettings"; WorkingDir: "{app}"

[Run]
; Korean keyboards are per user: add 釋讀 for the user who ran Setup.
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\korean-keyboard.ps1"""; Flags: runasoriginaluser runhidden waituntilterminated; StatusMsg: "{cm:AddKeyboard}"
Filename: "https://seokdok.mkpo.li/#install"; Description: "{cm:OpenGuide}"; Flags: postinstall shellexec skipifsilent unchecked

[UninstallRun]
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\korean-keyboard.ps1"" -Remove"; Flags: runhidden waituntilterminated; RunOnceId: "RemoveKeyboard"

[CustomMessages]
korean.AddKeyboard=한국어 키보드에 釋讀을 추가하는 중…
japanese.AddKeyboard=韓国語のキーボードに釋讀を追加しています…
english.AddKeyboard=Adding 釋讀 to your Korean keyboards…
korean.OpenGuide=사용법 보기 (seokdok.mkpo.li)
japanese.OpenGuide=使い方を見る (seokdok.mkpo.li)
english.OpenGuide=Open the guide (seokdok.mkpo.li)

[Messages]
korean.WelcomeLabel2=釋讀 입력기를 설치합니다. 한글로 치면 한자어는 음으로, 고유어는 뜻으로 漢字를 붙여 적습니다.%n%n설치 뒤 Win+Space에서 한국어 아래의 釋讀을 고르면 됩니다.
japanese.WelcomeLabel2=釋讀入力システムをインストールします。ハングルで入力すると、漢字語は音で、固有語は意味で漢字にします。%n%nインストール後、Win+Space で韓国語の下にある釋讀を選んでください。
english.WelcomeLabel2=This installs the 釋讀 input method: type Hangul, and Sino-Korean words come out in Hanja by sound, native words by meaning.%n%nAfterwards, pick 釋讀 under Korean with Win+Space.
korean.FinishedLabel=설치가 끝났습니다. Win+Space에서 한국어 아래의 釋讀을 고르세요. 이미 열려 있던 앱은 다시 열면 새 버전을 씁니다.
japanese.FinishedLabel=インストールが完了しました。Win+Space で韓国語の下にある釋讀を選んでください。すでに開いていたアプリは、開き直すと新しい版を使います。
english.FinishedLabel=Done. Pick 釋讀 under Korean with Win+Space. Apps that were already open use the new version once reopened.

[Code]
procedure MoveLoadedDll();
var
  Dll: String;
begin
  Dll := ExpandConstant('{app}\seokdok.dll');
  if FileExists(Dll) then
    RenameFile(Dll, ExpandConstant('{app}\seokdok-old-') + GetDateTimeString('yyyymmddhhnnss', #0, #0) + '.dll');
end;
