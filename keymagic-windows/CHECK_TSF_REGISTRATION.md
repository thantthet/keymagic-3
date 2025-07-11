# How to Check if KeyMagic TSF Text Service is Registered and Running

## Registry Locations

KeyMagic TSF Text Service registration can be verified in the following registry locations:

### 1. COM Server Registration
Check if the COM server is registered:
```
HKEY_CLASSES_ROOT\CLSID\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}
```
Should contain:
- Default value: "KeyMagic Text Service"
- InprocServer32 subkey with path to KeyMagicTSF.dll
- ThreadingModel: "Apartment"

### 2. TSF Text Input Processor Registration
Check the CTF (Common Text Framework) registry:
```
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\CTF\TIP\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}
```
Should contain:
- LanguageProfile\0x0455\{87654321-4321-4321-4321-CBA987654321}
  - Description: "KeyMagic Text Service"
  - Language ID: 0x0455 (Myanmar)

On 64-bit systems, also check:
```
HKEY_LOCAL_MACHINE\SOFTWARE\Wow6432Node\Microsoft\CTF\TIP\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}
```

## How to Verify Registration

### 1. Using Registry Editor
1. Open Registry Editor (regedit.exe)
2. Navigate to the paths above
3. Verify the keys and values exist

### 2. Using Command Line
Check COM registration:
```cmd
reg query "HKEY_CLASSES_ROOT\CLSID\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}"
```

Check TSF registration:
```cmd
reg query "HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\CTF\TIP\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}" /s
```

### 3. Check if TSF is Running
1. Open Task Manager (Ctrl+Alt+Delete)
2. Look for `ctfmon.exe` process - this manages TSF services
3. If not running, TSF services won't work

### 4. View Installed Input Methods

#### Through Settings (Windows 10/11):
1. Settings → Time & Language → Language
2. Click on preferred language
3. Options → Add a keyboard
4. Check if KeyMagic appears in the list

#### Through Control Panel:
1. Control Panel → Clock and Region → Region
2. Administrative tab → Change system locale → Advanced
3. Text Services and Input Languages

#### Through Language Bar:
1. Right-click on taskbar → Toolbars → Language bar
2. Click on language indicator in system tray
3. Language preferences → Advanced keyboard settings

## Registration Commands

### Register the TSF Service:
```cmd
regsvr32 "C:\Path\To\KeyMagicTSF.dll"
```

### Unregister the TSF Service:
```cmd
regsvr32 /u "C:\Path\To\KeyMagicTSF.dll"
```

## Troubleshooting

### If KeyMagic doesn't appear in input methods:
1. Verify DLL is registered: `regsvr32` should show success message
2. Check Event Viewer for COM registration errors
3. Ensure ctfmon.exe is running
4. Restart Windows Explorer or reboot

### Common Issues:
- **Missing from language list**: TSF profile not properly registered
- **COM errors**: Check if running regsvr32 as administrator
- **ctfmon.exe not running**: Enable it through Task Scheduler or Services

### Enable ctfmon.exe:
1. Task Scheduler → Create Basic Task
2. Name: "CTF Loader"
3. Trigger: At log on
4. Action: Start program → `C:\Windows\System32\ctfmon.exe`
5. Check "Run with highest privileges"

## Programmatic Verification

You can also verify registration programmatically using PowerShell:

```powershell
# Check COM registration
Get-ItemProperty "HKLM:\SOFTWARE\Classes\CLSID\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}" -ErrorAction SilentlyContinue

# Check TSF registration
Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\CTF\TIP\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}" -ErrorAction SilentlyContinue

# List all registered TIPs
Get-ChildItem "HKLM:\SOFTWARE\Microsoft\CTF\TIP"
```

## Expected Registry Structure

When properly registered, you should see:

```
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\CTF\TIP\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}
    └── LanguageProfile
        └── 0x00000455  (Myanmar Language ID)
            └── {87654321-4321-4321-4321-CBA987654321}
                ├── (Default) = "KeyMagic Text Service"
                ├── IconFile = "C:\Path\To\KeyMagicTSF.dll"
                ├── IconIndex = 0
                └── Enable = 1
```