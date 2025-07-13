; Common functions and procedures shared between x64 and ARM64 installers

[Code]
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

// Find and schedule old TSF DLLs for deletion
procedure CleanupOldTSFDLLs(TSFDir: String; CurrentDLLName: String);
var
  FindRec: TFindRec;
  DLLPath: String;
  ResultCode: Integer;
begin
  // Find all KeyMagicTSF_*.dll files
  if FindFirst(TSFDir + '\KeyMagicTSF_*.dll', FindRec) then
  begin
    try
      repeat
        // Skip the current version DLL
        if FindRec.Name <> CurrentDLLName then
        begin
          DLLPath := TSFDir + '\' + FindRec.Name;
          Log('Found old TSF DLL: ' + DLLPath);
          
          // Try to unregister it first (ignore errors as it might fail if in use)
          Exec('regsvr32.exe', '/s /u "' + DLLPath + '"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
          
          // Try to delete it
          if not DeleteFile(DLLPath) then
          begin
            // If deletion fails (likely because it's in use), schedule for deletion on restart
            Log('Scheduling for deletion on restart: ' + DLLPath);
            ScheduleFileForDeletion(DLLPath);
          end
          else
          begin
            Log('Deleted old TSF DLL: ' + DLLPath);
          end;
        end;
      until not FindNext(FindRec);
    finally
      FindClose(FindRec);
    end;
  end;
end;

// Check if Windows 10 or later
function IsWindows10OrLater: Boolean;
var
  Version: TWindowsVersion;
begin
  GetWindowsVersionEx(Version);
  Result := (Version.Major >= 10);
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
  
  // Check if KeyMagic is running (for upgrades)
  while IsKeyMagicRunning() do
  begin
    case MsgBox('KeyMagic is currently running. Please close it before continuing with the installation.' + #13#10 + #13#10 + 
                'Click "Yes" to close KeyMagic automatically.' + #13#10 +
                'Click "No" to close it manually and retry.' + #13#10 +
                'Click "Cancel" to abort installation.', mbError, MB_YESNOCANCEL) of
      IDYES:
        begin
          // Try to terminate KeyMagic
          Exec('taskkill.exe', '/F /IM keymagic.exe', '', SW_HIDE, ewWaitUntilTerminated, ErrorCode);
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
  
  // Check if KeyMagic is running
  while IsKeyMagicRunning() do
  begin
    case MsgBox('KeyMagic is currently running. Please close it before continuing with the uninstallation.' + #13#10 + #13#10 + 
                'Click "Yes" to close KeyMagic automatically.' + #13#10 +
                'Click "No" to close it manually and retry.' + #13#10 +
                'Click "Cancel" to abort uninstallation.', mbError, MB_YESNOCANCEL) of
      IDYES:
        begin
          // Try to terminate KeyMagic
          Exec('taskkill.exe', '/F /IM keymagic.exe', '', SW_HIDE, ewWaitUntilTerminated, ErrorCode);
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