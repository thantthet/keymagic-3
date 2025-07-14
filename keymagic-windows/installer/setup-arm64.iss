; KeyMagic Windows Installer Script - ARM64 Version
; Inno Setup 6.x

#define MyAppName "KeyMagic 3"
#define MyAppVersion "0.0.4"
#define MyAppPublisher "KeyMagic"
#define MyAppURL "https://github.com/thantthet/keymagic-v3"
#define MyAppExeName "keymagic.exe"
#define MyAppArch "ARM64"
#define MyAppVersionSuffix StringChange(MyAppVersion, '.', '_')
#define TSFDLLName "KeyMagicTSF_" + MyAppVersionSuffix + ".dll"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
AppId={{7A9B3C2D-4E5F-6789-ABCD-123456789012}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\..\LICENSE.md
OutputDir=.\output
OutputBaseFilename=KeyMagic3-Setup-{#MyAppVersion}-arm64
SetupIconFile=..\resources\icons\keymagic.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=arm64
ArchitecturesAllowed=arm64
DisableProgramGroupPage=yes
PrivilegesRequired=admin
MinVersion=10.0
UninstallRestartComputer=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Messages]
UninstallStatusLabel=Uninstalling %1. Please wait...%n%nNote: A logout will be required to complete the removal of the Text Services Framework components.

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[InstallDelete]
; Delete existing keyboards directory to ensure clean installation of bundled keyboards
Type: filesandordirs; Name: "{app}\keyboards"

[Files]
; GUI Application
Source: "..\target\aarch64-pc-windows-msvc\release\gui-tauri.exe"; DestDir: "{app}"; DestName: "keymagic.exe"; Flags: ignoreversion

; TSF DLL - ARM64 only (with version suffix)
Source: "..\tsf\build-arm64\Release\KeyMagicTSF.dll"; DestDir: "{app}\TSF"; DestName: "{#TSFDLLName}"; Flags: ignoreversion

; Resources
Source: "..\resources\icons\*"; DestDir: "{app}\resources\icons"; Flags: ignoreversion recursesubdirs createallsubdirs

; Production keyboards (included with installer)
Source: "keyboards\*.km2"; DestDir: "{app}\keyboards"; Flags: ignoreversion skipifsourcedoesntexist

; License and documentation
Source: "..\..\LICENSE.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; KeyMagic application settings only (TSF registration handled by regsvr32)
Root: HKCU; Subkey: "Software\KeyMagic"; Flags: uninsdeletekeyifempty
Root: HKCU; Subkey: "Software\KeyMagic\Settings"; Flags: uninsdeletekeyifempty
Root: HKCU; Subkey: "Software\KeyMagic\Keyboards"; Flags: uninsdeletekeyifempty

; Set StartWithWindows to 1 on install
Root: HKCU; Subkey: "Software\KeyMagic\Settings"; ValueType: string; ValueName: "StartWithWindows"; ValueData: "1"; Flags: uninsdeletevalue

; Set FirstRunScanKeyboards flag to trigger import wizard on first launch
Root: HKCU; Subkey: "Software\KeyMagic\Settings"; ValueType: dword; ValueName: "FirstRunScanKeyboards"; ValueData: "1"; Flags: createvalueifdoesntexist

; Add to Windows Run registry for auto-start
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "KeyMagic"; ValueData: "{app}\{#MyAppExeName}"; Flags: uninsdeletevalue

[Run]
; Download and install WebView2 if needed
Filename: "{tmp}\MicrosoftEdgeWebview2Setup.exe"; Parameters: "/silent /install"; StatusMsg: "Installing Microsoft Edge WebView2 Runtime..."; Flags: waituntilterminated; Check: ShouldInstallWebView2; BeforeInstall: DownloadWebView2

; Register TSF DLL (cleanup of old versions is handled automatically)
Filename: "regsvr32.exe"; Parameters: "/s ""{app}\TSF\{#TSFDLLName}"""; StatusMsg: "Registering Text Services Framework..."; Flags: runhidden; BeforeInstall: CleanupOldTSF

; Launch application after installation
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Unregister TSF DLL before uninstall
Filename: "regsvr32.exe"; Parameters: "/s /u ""{app}\TSF\{#TSFDLLName}"""; RunOnceId: "UnregTSF"; Flags: runhidden

#include "common.iss"