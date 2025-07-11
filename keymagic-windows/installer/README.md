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

### Quick Build

Run the build script from this directory:
```cmd
build-installer.bat
```

This script will:
1. Build x64 TSF DLL
2. Build ARM64 TSF DLL
3. Build GUI application (x64 only)
4. Verify all build artifacts
5. Create the installer executable

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

# Create installer
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" setup.iss
```

## Installer Features

- **Multi-architecture support**: Detects system architecture and installs appropriate TSF DLL
- **x64 and ARM64 TSF DLLs**: Both architectures included in installer
- **x64 GUI only**: GUI application is x64 only as requested
- **Automatic TSF registration**: Registers the correct TSF DLL based on system architecture
- **Clean uninstall**: Properly unregisters TSF and cleans up all files

## Output

The installer executable will be created in the `output` subdirectory:
```
output\KeyMagic3-0.0.1-Setup.exe
```

## Architecture Detection

The installer automatically detects the system architecture:
- On ARM64 Windows: Installs ARM64 TSF DLL + x64 GUI
- On x64 Windows: Installs x64 TSF DLL + x64 GUI

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