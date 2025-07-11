; KeyMagic Windows Installer Script - x64 Version
; Inno Setup 6.x

#define MyAppName "KeyMagic 3"
#define MyAppVersion "0.0.1"
#define MyAppPublisher "KeyMagic"
#define MyAppURL "https://github.com/thantthet/keymagic-v3"
#define MyAppExeName "keymagic.exe"
#define MyAppArch "x64"

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
OutputBaseFilename=KeyMagic3-{#MyAppVersion}-x64-Setup
SetupIconFile=..\resources\icons\keymagic.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64
ArchitecturesAllowed=x64
DisableProgramGroupPage=yes
PrivilegesRequired=admin
MinVersion=10.0

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; GUI Application (x64)
Source: "..\target\x86_64-pc-windows-msvc\release\gui-tauri.exe"; DestDir: "{app}"; DestName: "keymagic.exe"; Flags: ignoreversion

; TSF DLL - x64 only
Source: "..\tsf\build-x64\Release\KeyMagicTSF.dll"; DestDir: "{app}\TSF"; Flags: ignoreversion

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

[Run]
; Register TSF DLL
Filename: "regsvr32.exe"; Parameters: "/s ""{app}\TSF\KeyMagicTSF.dll"""; StatusMsg: "Registering Text Services Framework..."; Flags: runhidden

; Launch application after installation
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Unregister TSF DLL before uninstall
Filename: "regsvr32.exe"; Parameters: "/s /u ""{app}\TSF\KeyMagicTSF.dll"""; RunOnceId: "UnregTSF"; Flags: runhidden

[Code]
// Check if Windows 10 or later
function IsWindows10OrLater: Boolean;
var
  Version: TWindowsVersion;
begin
  GetWindowsVersionEx(Version);
  Result := (Version.Major >= 10);
end;

// Custom initialization
function InitializeSetup(): Boolean;
begin
  Result := True;
  
  // Check for Windows 10 or later
  if not IsWindows10OrLater then
  begin
    MsgBox('KeyMagic requires Windows 10 or later.', mbError, MB_OK);
    Result := False;
  end;
end;

// Clean up any temporary TSF registrations
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  TempPath: String;
  FindRec: TFindRec;
begin
  if CurUninstallStep = usUninstall then
  begin
    // Clean up any temporary TSF directories
    TempPath := ExpandConstant('{tmp}');
    if FindFirst(TempPath + '\KeyMagicTSF_*', FindRec) then
    begin
      try
        repeat
          DelTree(TempPath + '\' + FindRec.Name, True, True, True);
        until not FindNext(FindRec);
      finally
        FindClose(FindRec);
      end;
    end;
  end;
end;