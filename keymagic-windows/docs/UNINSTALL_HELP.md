# KeyMagic TSF Uninstall Help

## Quick Force Uninstall

Run `force_uninstall.bat` as Administrator.

## Manual Uninstall Steps

If the automated script doesn't work, follow these manual steps:

### 1. Safe Mode Uninstall (Most Effective)

1. **Boot into Safe Mode**:
   - Hold Shift and click Restart
   - Troubleshoot → Advanced options → Startup Settings → Restart
   - Press 4 for Safe Mode

2. **In Safe Mode**:
   ```cmd
   cd "C:\Program Files\KeyMagic"
   del /f keymagic_windows.dll
   ```

3. Restart normally

### 2. Using Process Explorer

1. Download [Process Explorer](https://docs.microsoft.com/en-us/sysinternals/downloads/process-explorer)
2. Run as Administrator
3. Find → Find Handle or DLL → Search for "keymagic"
4. Kill any processes using the DLL
5. Delete the file

### 3. Command Line Force Delete

Run Command Prompt as Administrator:

```cmd
# Stop TSF services
net stop TabletInputService
taskkill /F /IM ctfmon.exe
taskkill /F /IM TextInputHost.exe

# Take ownership and delete
takeown /f "C:\Program Files\KeyMagic\keymagic_windows.dll"
icacls "C:\Program Files\KeyMagic\keymagic_windows.dll" /grant administrators:F
del /f /q "C:\Program Files\KeyMagic\keymagic_windows.dll"

# Restart TSF
start ctfmon.exe
```

### 4. Registry Cleanup

Run Registry Editor (regedit) as Administrator and delete:

1. `HKEY_CLASSES_ROOT\CLSID\{12345678-1234-1234-1234-123456789ABC}`
2. `HKEY_LOCAL_MACHINE\SOFTWARE\Classes\CLSID\{12345678-1234-1234-1234-123456789ABC}`
3. Search for "KeyMagic" and delete any related entries

### 5. Using MoveFile on Reboot

If file is locked, schedule deletion on reboot:

```cmd
# Download MoveFile utility
# https://docs.microsoft.com/en-us/sysinternals/downloads/movefile

movefile "C:\Program Files\KeyMagic\keymagic_windows.dll" ""
```

### 6. Alternative: Rename First

Sometimes renaming works when deletion doesn't:

```cmd
cd "C:\Program Files\KeyMagic"
ren keymagic_windows.dll keymagic_windows.old
del /f keymagic_windows.old
```

## Troubleshooting

### Error: "Access Denied"
- Make sure you're running as Administrator
- TSF service is still using the file
- Another process has the file locked

### Error: "File in use"
1. Check what's using it:
   ```powershell
   Get-Process | Where-Object {$_.Modules.FileName -like '*keymagic*'}
   ```

2. Disable in Language Settings first:
   - Settings → Time & Language → Language
   - Click your language → Options
   - Remove KeyMagic keyboard

### Nuclear Option: Disable TSF Temporarily

**Warning**: This affects all IMEs temporarily

```cmd
# Disable TSF
sc config TabletInputService start= disabled
net stop TabletInputService

# Delete the file
del /f "C:\Program Files\KeyMagic\keymagic_windows.dll"

# Re-enable TSF
sc config TabletInputService start= auto
net start TabletInputService
```

## After Successful Uninstall

1. Clean up registry (run `regedit`):
   - Search for "KeyMagic" and remove entries
   - Search for "{12345678-1234-1234-1234-123456789ABC}" and remove

2. Remove from language settings if still visible

3. Restart Windows to ensure clean state