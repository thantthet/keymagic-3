# KeyMagic TSF Debugging Guide

## Quick Debug Steps

1. **Build Debug Version**
   ```cmd
   cd keymagic-windows
   build_debug.bat
   ```

2. **Install Debug DLL**
   ```cmd
   REM Run as Administrator
   install_debug.bat
   ```

3. **View Debug Output**

### Method 1: DebugView (Recommended)
1. Download DebugView from [Windows Sysinternals](https://docs.microsoft.com/en-us/sysinternals/downloads/debugview)
2. Run DebugView.exe as Administrator
3. Enable: Capture → Capture Win32
4. Filter for "[KeyMagic]" to see only KeyMagic messages
5. Switch to KeyMagic IME and type - you should see debug messages

### Method 2: Log Files
Debug logs are written to:
```
%TEMP%\KeyMagicTSF_YYYYMMDD_HHMMSS.log
```

Example: `C:\Users\YourName\AppData\Local\Temp\KeyMagicTSF_20250703_141523.log`

## Common Debug Messages

### Successful Initialization
```
[KeyMagic] KeyMagicTextService constructor called
[KeyMagic] Engine created: 0x1234567890
[KeyMagic] Enter: Activate
[KeyMagic] Activating TextService, ClientId: 1
```

### Key Processing
```
[KeyMagic] Enter: OnKeyDown
[KeyMagic] Modifiers: Shift=0, Ctrl=0, Alt=0, Caps=0
[KeyMagic] KeyEvent: vk=0x41 char='a' shift=0 ctrl=0 alt=0
[KeyMagic] Key consumed: YES
[KeyMagic] Engine action: type=1, text='က'
```

## Windows Event Logs

1. Open Event Viewer (eventvwr.msc)
2. Navigate to: Applications and Services Logs → Microsoft → Windows → TextServicesFramework
3. Look for errors related to KeyMagic CLSID: {12345678-1234-1234-1234-123456789ABC}

## Enable TSF System Debugging

### Registry Method (Advanced)
1. Open Registry Editor (regedit) as Administrator
2. Navigate to: `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\CTF\SystemShared`
3. Create DWORD: `EnableLogging` = 1
4. Restart the computer

### ETW Tracing (For Microsoft Support)
```cmd
logman create trace TSF -p {3C4E9FBB-A827-4FD1-BDDA-B129A02B3E44} -o tsf.etl
logman start TSF
REM Reproduce the issue
logman stop TSF
```

## Troubleshooting Common Issues

### Issue: IME Not Appearing in Language Settings
1. Check if DLL is registered:
   ```cmd
   reg query HKCR\CLSID\{12345678-1234-1234-1234-123456789ABC}
   ```
2. Re-register:
   ```cmd
   regsvr32 "C:\Program Files\KeyMagic\keymagic_windows.dll"
   ```

### Issue: No Debug Output
1. Verify debug build was used
2. Check if IME is actually being loaded:
   - Task Manager → Details → Look for ctfmon.exe
   - Use Process Explorer to see if keymagic_windows.dll is loaded

### Issue: Keys Not Being Processed
Check debug output for:
- "No engine available" - Engine initialization failed
- "Engine returned error" - Engine processing error
- No "OnKeyDown" messages - IME not receiving key events

## Advanced Debugging

### Attach Debugger
1. Install Visual Studio with C++ debugging tools
2. Debug → Attach to Process → Select the application using the IME
3. Set breakpoints in KeyMagicTextService.cpp

### Enable Verbose TSF Logging
```cmd
set TF_DEBUG=1
set TF_DEBUGLOG=C:\tsf_debug.log
```

### Check TSF Profile Registration
```powershell
Get-WinUserLanguageList | ForEach-Object { $_.InputMethodTips }
```

## Performance Profiling

Use Windows Performance Toolkit:
```cmd
wpr -start CPU
REM Type with KeyMagic
wpr -stop keymagic.etl
wpa keymagic.etl
```

## Debug Build vs Release Build

- Debug build includes detailed logging but is slower
- Release build has minimal logging for production use
- Always test with release build before deployment

## Reporting Issues

When reporting issues, include:
1. Debug log file
2. Windows version (winver.exe)
3. Steps to reproduce
4. Expected vs actual behavior
5. Any error messages from Event Viewer