; Inno Setup script for seokdok: installs the TSF DLL and the dict folder next
; to it, and registers the DLL.
;
; Build (after build.ps1 or cargo build --release --target x86_64-pc-windows-msvc -p seokdok-tsf):
;   ISCC.exe /DMyAppVersion=0.1.0 crates\seokdok-tsf\installer\seokdok.iss
; Output:
;   <repo>\seokdok-<version>-x86_64-windows-setup.exe

#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif
#define MyAppName "seokdok"
#define MyAppPublisher "mkpoli"

[Setup]
AppId={{041E579D-22D4-46E1-8D5C-B9BF7D6EC789}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
VersionInfoVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\seokdok
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
OutputDir=..\..\..
OutputBaseFilename=seokdok-{#MyAppVersion}-x86_64-windows-setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
LicenseFile=..\LICENSE
UninstallDisplayName={#MyAppName}
UninstallDisplayIcon={app}\seokdok.dll
ShowLanguageDialog=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "korean"; MessagesFile: "i18n\Korean.isl"

[Files]
; regserver calls DllRegisterServer on install and DllUnregisterServer on
; uninstall. A DLL loaded by a running app cannot be replaced at once, so
; restartreplace schedules the swap for the next reboot.
Source: "..\..\..\target\x86_64-pc-windows-msvc\release\seokdok.dll"; DestDir: "{app}"; Flags: ignoreversion regserver restartreplace uninsrestartdelete
; The engine reads dict\*.tsv next to the DLL. user.tsv and user-sino.tsv are
; per-user files in %APPDATA%\seokdok and never ship.
Source: "..\..\..\dict\*.tsv"; DestDir: "{app}\dict"; Excludes: "user.tsv,user-sino.tsv"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
; Settings entry point; Windows 11 hides the language bar menu.
Name: "{autoprograms}\seokdok Settings"; Filename: "{sys}\rundll32.exe"; Parameters: """{app}\seokdok.dll"",ShowSettings"; WorkingDir: "{app}"; Comment: "釋讀 seokdok - Settings"

[Messages]
english.FinishedLabel=seokdok was installed. Switch input (Win+Space) to 釋讀 (seokdok), listed under Korean. Korean must be in the Windows language list.
japanese.FinishedLabel=seokdokをインストールしました。入力切替（Win+Space）で釋讀 (seokdok)（「韓国語」の下に表示されます）を選んでください。Windowsの言語一覧に韓国語が必要です。
korean.FinishedLabel=seokdok가 설치되었습니다. 입력 전환(Win+Space)에서 釋讀 (seokdok)(“한국어” 아래에 표시됨)를 선택하세요. Windows 언어 목록에 한국어가 있어야 합니다.
