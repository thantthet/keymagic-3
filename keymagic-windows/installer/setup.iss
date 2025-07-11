; KeyMagic Windows Installer Script
; Inno Setup 6.x

#define MyAppName "KeyMagic 3"
#define MyAppVersion "0.0.1"
#define MyAppPublisher "KeyMagic"
#define MyAppURL "https://github.com/thantthet/keymagic-v3"
#define MyAppExeName "keymagic.exe"

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
OutputBaseFilename=KeyMagic3-{#MyAppVersion}-Setup
SetupIconFile=..\resources\icons\keymagic.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64os
ArchitecturesAllowed=x64os arm64
DisableProgramGroupPage=yes
PrivilegesRequired=admin
MinVersion=10.0

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; GUI Application (x64 built, but runs on all platforms)
Source: "..\target\x86_64-pc-windows-msvc\release\gui-tauri.exe"; DestDir: "{app}"; DestName: "keymagic.exe"; Flags: ignoreversion

; TSF DLLs - Install both architectures
Source: "..\tsf\build-x64\Release\KeyMagicTSF.dll"; DestDir: "{app}\TSF\x64"; Flags: ignoreversion
Source: "..\tsf\build-ARM64\Release\KeyMagicTSF.dll"; DestDir: "{app}\TSF\ARM64"; Flags: ignoreversion; Check: FileExists(ExpandConstant('{#SourcePath}..\tsf\build-ARM64\Release\KeyMagicTSF.dll'))

; Resources
Source: "..\resources\icons\*"; DestDir: "{app}\resources\icons"; Flags: ignoreversion recursesubdirs createallsubdirs

; Production keyboards (included with installer)
Source: "keyboards\*.km2"; DestDir: "{app}\keyboards"; Flags: ignoreversion

; License and documentation
Source: "..\..\LICENSE.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; Registry entries for TSF
Root: HKCR; Subkey: "CLSID\{{094A562B-D08B-4CAF-8E95-8F8031CFD24C}}"; Flags: uninsdeletekey
Root: HKCR; Subkey: "CLSID\{{094A562B-D08B-4CAF-8E95-8F8031CFD24C}}"; ValueType: string; ValueName: ""; ValueData: "KeyMagic Text Service"
Root: HKCR; Subkey: "CLSID\{{094A562B-D08B-4CAF-8E95-8F8031CFD24C}}\InprocServer32"; ValueType: string; ValueName: ""; ValueData: "{code:GetTSFDllPath}"
Root: HKCR; Subkey: "CLSID\{{094A562B-D08B-4CAF-8E95-8F8031CFD24C}}\InprocServer32"; ValueType: string; ValueName: "ThreadingModel"; ValueData: "Apartment"

; KeyMagic application settings
Root: HKCU; Subkey: "Software\KeyMagic"; Flags: uninsdeletekeyifempty
Root: HKCU; Subkey: "Software\KeyMagic\Settings"; Flags: uninsdeletekeyifempty
Root: HKCU; Subkey: "Software\KeyMagic\Keyboards"; Flags: uninsdeletekeyifempty

[Run]
; Register TSF DLL
Filename: "regsvr32.exe"; Parameters: "/s ""{code:GetTSFDllPath}"""; StatusMsg: "Registering Text Services Framework..."; Flags: runhidden

; Launch application after installation
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Unregister TSF DLL before uninstall
Filename: "regsvr32.exe"; Parameters: "/s /u ""{code:GetTSFDllPath}"""; RunOnceId: "UnregTSF"; Flags: runhidden

[Code]
var
  TSFDllPath: String;

// Function to detect system architecture
function IsARM64: Boolean;
var
  ProcessorArchitecture: String;
begin
  Result := False;
  if RegQueryStringValue(HKEY_LOCAL_MACHINE, 
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'PROCESSOR_ARCHITECTURE', ProcessorArchitecture) then
  begin
    Result := (ProcessorArchitecture = 'ARM64');
  end;
end;

// Get the appropriate TSF DLL path based on architecture
function GetTSFDllPath(Param: String): String;
begin
  if TSFDllPath = '' then
  begin
    if IsARM64 then
      TSFDllPath := ExpandConstant('{app}\TSF\ARM64\KeyMagicTSF.dll')
    else
      TSFDllPath := ExpandConstant('{app}\TSF\x64\KeyMagicTSF.dll');
  end;
  Result := TSFDllPath;
end;

// Check if we're installing on 64-bit Windows
function Is64BitInstallMode: Boolean;
begin
  Result := Is64BitInstallMode;
end;

// Check if a file exists (for conditional file installation)
function FileExists(FileName: String): Boolean;
begin
  Result := FileExists(FileName);
end;

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

// Show architecture info during installation
procedure InitializeWizard();
var
  ArchLabel: TLabel;
begin
  ArchLabel := TLabel.Create(WizardForm);
  ArchLabel.Parent := WizardForm.WelcomePage;
  ArchLabel.Left := WizardForm.WelcomeLabel2.Left;
  ArchLabel.Top := WizardForm.WelcomeLabel2.Top + WizardForm.WelcomeLabel2.Height + 20;
  ArchLabel.Caption := 'System Architecture: ';
  if IsARM64 then
    ArchLabel.Caption := ArchLabel.Caption + 'ARM64'
  else
    ArchLabel.Caption := ArchLabel.Caption + 'x64';
  ArchLabel.Font.Style := [fsBold];
end;