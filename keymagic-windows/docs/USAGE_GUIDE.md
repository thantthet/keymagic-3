# KeyMagic Windows TSF Usage Guide

## Complete Setup Process

### 1. Build the Components

```cmd
# Build the TSF DLL
cd keymagic-windows
build_windows.bat

# Build the Manager GUI
cd manager
build_manager.bat
```

### 2. Install the TSF

Run as Administrator:
```cmd
cd keymagic-windows
install.bat
```

### 3. Add Keyboards

1. Run `manager\KeyMagicManager.exe`
2. Click "Add..." to browse for .km2 keyboard files
3. Select a keyboard and click "Activate"

### 4. Enable in Windows

1. Go to Settings → Time & Language → Language
2. Click on your language (e.g., English) and select Options
3. Click "Add a keyboard"
4. Find and select "KeyMagic" from the list
5. Use Win+Space to switch between keyboards

## Workflow Summary

```
1. Build Components
   ├── TSF DLL (keymagic_windows.dll)
   └── Manager GUI (KeyMagicManager.exe)

2. Install TSF
   └── Registers DLL with Windows

3. Configure Keyboards
   ├── Run Manager GUI
   ├── Add .km2 files
   └── Activate desired keyboard

4. Use KeyMagic
   ├── Select KeyMagic in language bar
   └── Type using the activated keyboard layout
```

## Debugging

If KeyMagic isn't working:

1. **Check if TSF is loaded**:
   - Run DebugView.exe as Administrator
   - Look for "[KeyMagic]" messages

2. **Check keyboard loading**:
   - Should see: "Loading active keyboard from registry"
   - Should see: "Keyboard loaded successfully"

3. **Common issues**:
   - No keyboard activated: Use Manager to activate one
   - Keyboard file not found: Check file path in registry
   - TSF not loading: Reinstall using install.bat

## Testing with Sample Keyboard

You'll need a .km2 file to test. You can:
1. Use an existing .km2 file from KeyMagic 2
2. Create one using the kms2km2 converter
3. Download sample keyboards from the KeyMagic website

## Registry Locations

- Keyboards: `HKEY_CURRENT_USER\Software\KeyMagic\Keyboards`
- TSF Registration: `HKEY_CLASSES_ROOT\CLSID\{12345678-1234-1234-1234-123456789ABC}`

## Uninstalling

To completely remove KeyMagic:

1. Run `force_uninstall.bat` as Administrator
2. Delete the keyboards from registry
3. Remove from Windows language settings