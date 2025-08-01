; KeyMagic Windows Installer Script - Unified x64 and ARM64
; Inno Setup 6.x

#define MyAppName "KeyMagic 3"
#define MyAppVersion "0.0.7"
#define MyAppPublisher "KeyMagic"
#define MyAppURL "https://github.com/thantthet/keymagic-v3"
#define MyAppExeName "keymagic.exe"
#define MyAppVersionSuffix StringChange(MyAppVersion, '.', '_')

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
OutputBaseFilename=KeyMagic3-Setup-{#MyAppVersion}
SetupIconFile=..\..\resources\icons\keymagic.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible arm64
ArchitecturesAllowed=x64 arm64
DisableProgramGroupPage=yes
PrivilegesRequired=admin
MinVersion=7.0
UninstallRestartComputer=yes
ChangesAssociations=yes

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
; GUI Application (architecture-specific)
Source: "..\..\target\x86_64-pc-windows-msvc\release\keymagic-gui.exe"; DestDir: "{app}"; DestName: "keymagic.exe"; Check: IsX64; Flags: ignoreversion
Source: "..\..\target\aarch64-pc-windows-msvc\release\keymagic-gui.exe"; DestDir: "{app}"; DestName: "keymagic.exe"; Check: IsARM64; Flags: ignoreversion

; Tray Manager Application (architecture-specific)
Source: "..\tray-manager\build-x64\bin\Release\keymagic-tray.exe"; DestDir: "{app}"; Check: IsX64; Flags: ignoreversion
Source: "..\tray-manager\build-arm64\bin\Release\keymagic-tray.exe"; DestDir: "{app}"; Check: IsARM64; Flags: ignoreversion

; TSF DLLs - x64 (single DLL in versioned subdirectory)
Source: "..\tsf\build-x64\Release\KeyMagicTSF_x64.dll"; DestDir: "{app}\TSF\{#MyAppVersionSuffix}"; Check: IsX64; Flags: ignoreversion

; TSF DLLs - ARM64 (ARM64X forwarder and implementation DLLs in versioned subdirectory)
Source: "..\tsf\build-arm64x\KeyMagicTSF.dll"; DestDir: "{app}\TSF\{#MyAppVersionSuffix}"; Check: IsARM64; Flags: ignoreversion
Source: "..\tsf\build-arm64x\KeyMagicTSF_arm64.dll"; DestDir: "{app}\TSF\{#MyAppVersionSuffix}"; Check: IsARM64; Flags: ignoreversion
Source: "..\tsf\build-arm64x\KeyMagicTSF_x64.dll"; DestDir: "{app}\TSF\{#MyAppVersionSuffix}"; Check: IsARM64; Flags: ignoreversion

; Resources (common)
Source: "..\..\resources\icons\*"; DestDir: "{app}\resources\icons"; Flags: ignoreversion recursesubdirs createallsubdirs

; Keyboard icon for TSF language profile (install to LOCALAPPDATA)
Source: "..\..\resources\icons\keymagic-keyboard.ico"; DestDir: "{localappdata}\KeyMagic"; Flags: ignoreversion

; Production keyboards (included with installer) from centralized location
Source: "..\..\keyboards\bundled\*.km2"; DestDir: "{app}\keyboards"; Flags: ignoreversion

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

; Set StartWithWindows to 0 on install (tray manager will handle startup)
Root: HKCU; Subkey: "Software\KeyMagic\Settings"; ValueType: string; ValueName: "StartWithWindows"; ValueData: "0"; Flags: uninsdeletevalue

; Version-based first-run detection is now used instead of FirstRunScanKeyboards flag
; The app will automatically detect first run based on version comparison

; Add Tray Manager to Windows Run registry for auto-start (not the GUI app)
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "KeyMagicTray"; ValueData: "{app}\keymagic-tray.exe"; Flags: uninsdeletevalue

; File association for .km2 files
Root: HKCR; Subkey: ".km2"; ValueType: string; ValueName: ""; ValueData: "KeyMagicKeyboard"; Flags: uninsdeletevalue
Root: HKCR; Subkey: "KeyMagicKeyboard"; ValueType: string; ValueName: ""; ValueData: "KeyMagic Keyboard Layout"; Flags: uninsdeletekey
Root: HKCR; Subkey: "KeyMagicKeyboard\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\resources\icons\keymagic-file.ico"; Flags: uninsdeletekey
Root: HKCR; Subkey: "KeyMagicKeyboard\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Flags: uninsdeletekey

[Run]
; Download and install WebView2 if needed
Filename: "{tmp}\MicrosoftEdgeWebview2Setup.exe"; Parameters: "/silent /install"; StatusMsg: "Installing Microsoft Edge WebView2 Runtime..."; Flags: waituntilterminated; Check: ShouldInstallWebView2; BeforeInstall: DownloadWebView2

; Register TSF DLL (cleanup of old versions is handled automatically)
; For x64, register the single DLL
Filename: "regsvr32.exe"; Parameters: "/s ""{app}\TSF\{#MyAppVersionSuffix}\KeyMagicTSF_x64.dll"""; StatusMsg: "Registering Text Services Framework..."; Check: IsX64; Flags: runhidden; BeforeInstall: CleanupOldTSF
; For ARM64, register the ARM64X forwarder DLL
Filename: "regsvr32.exe"; Parameters: "/s ""{app}\TSF\{#MyAppVersionSuffix}\KeyMagicTSF.dll"""; StatusMsg: "Registering Text Services Framework..."; Check: IsARM64; Flags: runhidden; BeforeInstall: CleanupOldTSF

; Always launch tray manager after installation
Filename: "{app}\keymagic-tray.exe"; StatusMsg: "Starting KeyMagic Tray Manager..."; Flags: nowait runhidden

; Optionally launch GUI application after installation
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Unregister TSF DLL before uninstall
; For x64, unregister the single DLL
Filename: "regsvr32.exe"; Parameters: "/s /u ""{app}\TSF\{#MyAppVersionSuffix}\KeyMagicTSF_x64.dll"""; Check: IsX64; RunOnceId: "UnregTSF"; Flags: runhidden
; For ARM64, unregister the ARM64X forwarder DLL
Filename: "regsvr32.exe"; Parameters: "/s /u ""{app}\TSF\{#MyAppVersionSuffix}\KeyMagicTSF.dll"""; Check: IsARM64; RunOnceId: "UnregTSF"; Flags: runhidden

// Include common functions and procedures
#include "common.iss"