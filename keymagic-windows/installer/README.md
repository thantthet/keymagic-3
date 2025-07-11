# KeyMagic Windows Installer

This directory contains the installer configuration and build scripts for KeyMagic Windows.

## Prerequisites

1. **Inno Setup 6.x** - Download from https://jrsoftware.org/isdl.php
2. **Visual Studio 2022** - For building TSF components
3. **Rust** - For building the GUI application
4. **CMake** - For building TSF DLLs

## Keyboards

Production keyboards should be placed in the `keyboards` directory before building the installer:
- Place `.km2` files in `./keyboards/`
- These keyboards will be installed to `C:\Program Files\KeyMagic\keyboards\`

## Building the Installer

### Architecture-Specific Installers

Due to Windows registry redirection, we now build separate installers for each architecture:

#### Build x64 Installer Only
```cmd
build-installer-x64.bat
```

#### Build ARM64 Installer Only
```cmd
build-installer-arm64.bat
```

#### Build All Installers
```cmd
build-installer-all.bat
```

### What Each Script Does

**x64 Installer** (`build-installer-x64.bat`):
1. Builds x64 TSF DLL
2. Builds GUI application (x64)
3. Creates x64-specific installer
4. Output: `KeyMagic3-0.0.1-x64-Setup.exe`

**ARM64 Installer** (`build-installer-arm64.bat`):
1. Builds ARM64 TSF DLL
2. Builds GUI application (x64 - runs via emulation)
3. Creates ARM64-specific installer
4. Output: `KeyMagic3-0.0.1-ARM64-Setup.exe`

### Manual Build Steps

If you need to build components individually:

```cmd
# Build x64 TSF
..\make.bat build x64 Release

# Build ARM64 TSF
..\make.bat build arm64 Release

# Build GUI
cd ..\gui-tauri
build.bat
cd ..\installer

# Create x64 installer
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" setup-x64.iss

# Create ARM64 installer
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" setup-arm64.iss
```

## Installer Features

- **Architecture-specific installers**: Separate installers for x64 and ARM64 to avoid registry redirection issues
- **Correct TSF registration**: Each installer registers the appropriate architecture DLL
- **x64 GUI**: GUI application is x64 (runs natively on x64, via emulation on ARM64)
- **Automatic TSF registration**: Registers TSF DLL during installation
- **Clean uninstall**: Properly unregisters TSF and cleans up all files

## Output

The installer executables will be created in the `output` subdirectory:
```
output\KeyMagic3-0.0.1-x64-Setup.exe    # For x64 Windows
output\KeyMagic3-0.0.1-ARM64-Setup.exe  # For ARM64 Windows
```

## Architecture Notes

- **x64 Installer**: Contains x64 TSF DLL + x64 GUI
- **ARM64 Installer**: Contains ARM64 TSF DLL + x64 GUI (runs via emulation)
- Users should download the installer matching their Windows architecture

## Testing

After installation, verify:
1. TSF is registered: Check in Windows Text Services
2. GUI launches: Start menu or desktop shortcut
3. TSF works: Test in any text input field

## Development Scripts

### PowerShell TSF Registration
For development testing without full installation:
```powershell
# Run as Administrator
.\scripts\register-tsf.ps1 -Architecture Auto
```

## Troubleshooting

### Build Failures
- Ensure all prerequisites are installed
- Check that previous builds completed successfully
- Verify file paths in setup.iss

### Registration Issues
- Must run installer as Administrator
- Check Windows Event Log for TSF registration errors
- Verify DLL architecture matches system

### Missing Files
- Icon file is optional (warning if missing)
- Test keyboards are optional
- All other files are required