; Common functions and procedures shared between x64 and ARM64 installers

[Code]
var
  NeedWebView2: Boolean;

// Convert version string to DLL suffix (e.g., "0.0.2" -> "0_0_2")
function GetVersionSuffix(Version: String): String;
begin
  Result := Version;
  StringChangeEx(Result, '.', '_', True);
end;

// Schedule a file for deletion on next Windows restart
procedure ScheduleFileForDeletion(FileName: String);
begin
  // Use MoveFileEx with MOVEFILE_DELAY_UNTIL_REBOOT flag
  // This schedules the file for deletion on next restart
  if not RenameFile(FileName, FileName + '.delete') then
  begin
    Log('Could not rename file for deletion: ' + FileName);
  end
  else
  begin
    // Add to pending file operations
    RestartReplace(FileName + '.delete', '');
  end;
end;

// Helper to schedule entire directory for deletion
procedure ScheduleDirectoryForDeletion(DirPath: String);
var
  FindRec: TFindRec;
  FilePath: String;
begin
  // Schedule all files in directory for deletion
  if FindFirst(DirPath + '\*', FindRec) then
  begin
    try
      repeat
        if (FindRec.Name <> '.') and (FindRec.Name <> '..') then
        begin
          FilePath := DirPath + '\' + FindRec.Name;
          if (FindRec.Attributes and FILE_ATTRIBUTE_DIRECTORY = 0) then
          begin
            ScheduleFileForDeletion(FilePath);
          end;
        end;
      until not FindNext(FindRec);
    finally
      FindClose(FindRec);
    end;
  end;
end;

// Helper to clean up old-style DLLs in TSF root (for backward compatibility)
procedure CleanupOldTSFDLLsInRoot(TSFDir: String);
var
  FindRec: TFindRec;
  DLLPath: String;
  ResultCode: Integer;
begin
  // Find all KeyMagicTSF*.dll files in root TSF directory
  if FindFirst(TSFDir + '\KeyMagicTSF*.dll', FindRec) then
  begin
    try
      repeat
        DLLPath := TSFDir + '\' + FindRec.Name;
        Log('Found old-style TSF DLL in root: ' + DLLPath);
        
        // Try to unregister it first
        Exec('regsvr32.exe', '/s /u "' + DLLPath + '"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
        
        // Try to delete it
        if not DeleteFile(DLLPath) then
        begin
          Log('Scheduling for deletion on restart: ' + DLLPath);
          ScheduleFileForDeletion(DLLPath);
        end
        else
        begin
          Log('Deleted old-style TSF DLL: ' + DLLPath);
        end;
      until not FindNext(FindRec);
    finally
      FindClose(FindRec);
    end;
  end;
end;

// Find and schedule old TSF version directories for deletion
procedure CleanupOldTSFVersions(TSFDir: String; CurrentVersionSuffix: String);
var
  FindRec: TFindRec;
  VersionDir: String;
  ResultCode: Integer;
begin
  // Find all subdirectories in TSF directory
  if FindFirst(TSFDir + '\*', FindRec) then
  begin
    try
      repeat
        // Skip . and .. directories
        if (FindRec.Name <> '.') and (FindRec.Name <> '..') and 
           (FindRec.Attributes and FILE_ATTRIBUTE_DIRECTORY <> 0) then
        begin
          // Skip the current version directory
          if FindRec.Name <> CurrentVersionSuffix then
          begin
            VersionDir := TSFDir + '\' + FindRec.Name;
            Log('Found old TSF version directory: ' + VersionDir);
            
            // Try to unregister the forwarder DLL if it exists
            if FileExists(VersionDir + '\KeyMagicTSF.dll') then
            begin
              Exec('regsvr32.exe', '/s /u "' + VersionDir + '\KeyMagicTSF.dll"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
            end;
            
            // Try to delete the entire directory
            if not DelTree(VersionDir, True, True, True) then
            begin
              // If deletion fails, try to schedule individual files for deletion
              Log('Failed to delete directory, scheduling files for deletion on restart: ' + VersionDir);
              ScheduleDirectoryForDeletion(VersionDir);
            end
            else
            begin
              Log('Deleted old TSF version directory: ' + VersionDir);
            end;
          end
          else
          begin
            Log('Keeping current version directory: ' + FindRec.Name);
          end;
        end;
      until not FindNext(FindRec);
    finally
      FindClose(FindRec);
    end;
  end;
  
  // Also clean up any old-style DLLs in the TSF root directory
  CleanupOldTSFDLLsInRoot(TSFDir);
end;

// Check if Windows 10 or later
function IsWindows10OrLater: Boolean;
var
  Version: TWindowsVersion;
begin
  GetWindowsVersionEx(Version);
  Result := (Version.Major >= 10);
end;

// Check if WebView2 Runtime is installed
function IsWebView2Installed: Boolean;
var
  ResultStr: String;
  Version: String;
begin
  Result := False;
  
  // Check for WebView2 by looking for the registry key
  // First check the fixed version runtime
  if RegQueryStringValue(HKLM32, 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Version) or
     RegQueryStringValue(HKLM64, 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Version) then
  begin
    if Version <> '' then
    begin
      Result := True;
      Exit;
    end;
  end;
  
  // Check for evergreen runtime
  if RegQueryStringValue(HKLM32, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Version) or
     RegQueryStringValue(HKLM64, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Version) then
  begin
    if Version <> '' then
    begin
      Result := True;
      Exit;
    end;
  end;
  
  // Alternative check: Look for WebView2Loader.dll
  if FileExists(ExpandConstant('{sys}\WebView2Loader.dll')) then
  begin
    Result := True;
    Exit;
  end;
end;

// Check if KeyMagic is running
function IsKeyMagicRunning(): Boolean;
var
  ResultCode: Integer;
begin
  // Use tasklist with filter - returns 0 if found, 1 if not found
  if not Exec('cmd.exe', '/C "tasklist /FI "IMAGENAME eq keymagic.exe" 2>nul | find /I "keymagic.exe" >nul"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) then
  begin
    // If exec fails, assume it's not running
    Result := False;
  end
  else
  begin
    // ResultCode = 0 means the process was found
    Result := (ResultCode = 0);
  end;
end;

// Check if KeyMagic Tray Manager is running
function IsTrayManagerRunning(): Boolean;
var
  ResultCode: Integer;
begin
  // Use tasklist with filter - returns 0 if found, 1 if not found
  if not Exec('cmd.exe', '/C "tasklist /FI "IMAGENAME eq keymagic-tray.exe" 2>nul | find /I "keymagic-tray.exe" >nul"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) then
  begin
    // If exec fails, assume it's not running
    Result := False;
  end
  else
  begin
    // ResultCode = 0 means the process was found
    Result := (ResultCode = 0);
  end;
end;

// Custom initialization
function InitializeSetup(): Boolean;
var
  ErrorCode: Integer;
begin
  Result := True;
  
  // Check for Windows 10 or later
  if not IsWindows10OrLater then
  begin
    MsgBox('KeyMagic requires Windows 10 or later.', mbError, MB_OK);
    Result := False;
    Exit;
  end;
  
  // Check for WebView2 Runtime
  if not IsWebView2Installed then
  begin
    if MsgBox('Microsoft Edge WebView2 Runtime is required for KeyMagic but is not installed.' + #13#10 + #13#10 + 
              'Would you like to download and install it now?' + #13#10 + #13#10 +
              'Note: This will download the installer from Microsoft (~2MB) and install WebView2 Runtime (~100MB).', 
              mbConfirmation, MB_YESNO) = IDYES then
    begin
      // Download and run WebView2 installer
      // The installer will be downloaded during the installation process
      // Set a flag to trigger the download in the Run section
      NeedWebView2 := True;
      Result := True; // Continue with installation, WebView2 will be installed during Run phase
    end
    else
    begin
      MsgBox('Installation cannot continue without WebView2 Runtime.' + #13#10 + #13#10 +
             'Please install Microsoft Edge WebView2 Runtime manually from:' + #13#10 +
             'https://developer.microsoft.com/microsoft-edge/webview2/', mbError, MB_OK);
      Result := False;
      Exit;
    end;
  end;
  
  // Check if KeyMagic or Tray Manager is running (for upgrades)
  while IsKeyMagicRunning() or IsTrayManagerRunning() do
  begin
    case MsgBox('KeyMagic is currently running. Please close it before continuing with the installation.' + #13#10 + #13#10 + 
                'Click "Yes" to close KeyMagic automatically.' + #13#10 +
                'Click "No" to close it manually and retry.' + #13#10 +
                'Click "Cancel" to abort installation.', mbError, MB_YESNOCANCEL) of
      IDYES:
        begin
          // Try to terminate both KeyMagic and Tray Manager
          Exec('taskkill.exe', '/F /IM keymagic.exe', '', SW_HIDE, ewWaitUntilTerminated, ErrorCode);
          Exec('taskkill.exe', '/F /IM keymagic-tray.exe', '', SW_HIDE, ewWaitUntilTerminated, ErrorCode);
          Sleep(1000); // Give it a moment to close
        end;
      IDNO:
        begin
          // User will close it manually, just continue the loop
        end;
      IDCANCEL:
        begin
          Result := False;
          Exit;
        end;
    end;
  end;
end;

// Initialize uninstall
function InitializeUninstall(): Boolean;
var
  ErrorCode: Integer;
begin
  Result := True;
  
  // Check if KeyMagic or Tray Manager is running
  while IsKeyMagicRunning() or IsTrayManagerRunning() do
  begin
    case MsgBox('KeyMagic is currently running. Please close it before continuing with the uninstallation.' + #13#10 + #13#10 + 
                'Click "Yes" to close KeyMagic automatically.' + #13#10 +
                'Click "No" to close it manually and retry.' + #13#10 +
                'Click "Cancel" to abort uninstallation.', mbError, MB_YESNOCANCEL) of
      IDYES:
        begin
          // Try to terminate both KeyMagic and Tray Manager
          Exec('taskkill.exe', '/F /IM keymagic.exe', '', SW_HIDE, ewWaitUntilTerminated, ErrorCode);
          Exec('taskkill.exe', '/F /IM keymagic-tray.exe', '', SW_HIDE, ewWaitUntilTerminated, ErrorCode);
          Sleep(1000); // Give it a moment to close
        end;
      IDNO:
        begin
          // User will close it manually, just continue the loop
        end;
      IDCANCEL:
        begin
          Result := False;
          Exit;
        end;
    end;
  end;
end;

// Check function for WebView2 installer in Run section
function ShouldInstallWebView2: Boolean;
begin
  Result := NeedWebView2;
end;

// Download WebView2 installer
procedure DownloadWebView2();
var
  DownloadPage: TDownloadWizardPage;
  Url: String;
begin
  // WebView2 bootstrapper URL - same for all architectures, bootstrapper auto-detects
  Url := 'https://go.microsoft.com/fwlink/p/?LinkId=2124703';
  
  try
    DownloadPage := CreateDownloadPage(SetupMessage(msgWizardPreparing), SetupMessage(msgPreparingDesc), nil);
    DownloadPage.Clear;
    DownloadPage.Add(Url, 'MicrosoftEdgeWebview2Setup.exe', '');
    DownloadPage.Show;
    try
      DownloadPage.Download;
    finally
      DownloadPage.Hide;
    end;
  except
    // If download fails, show error but continue (user can install manually later)
    MsgBox('Failed to download WebView2 installer. You may need to install it manually later.', mbError, MB_OK);
  end;
end;

// Called before registering the new TSF DLL
procedure CleanupOldTSF();
var
  TSFDir: String;
  CurrentVersionSuffix: String;
begin
  TSFDir := ExpandConstant('{app}\TSF');
  CurrentVersionSuffix := ExpandConstant('{#MyAppVersionSuffix}');
  
  // Call the new cleanup function that handles versioned subdirectories
  CleanupOldTSFVersions(TSFDir, CurrentVersionSuffix);
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

// Architecture detection functions
function IsX64: Boolean;
begin
  Result := Is64BitInstallMode and (ProcessorArchitecture = paX64);
end;

function IsARM64: Boolean;
begin
  Result := Is64BitInstallMode and (ProcessorArchitecture = paARM64);
end;