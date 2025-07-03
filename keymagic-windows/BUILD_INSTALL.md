# Building and Installing KeyMagic Windows TSF DLL

## Prerequisites

### 1. Install Rust
Download and install Rust from https://rustup.rs/
```powershell
# After installation, add Windows target if not already present
rustup target add x86_64-pc-windows-msvc
```

### 2. Install Visual Studio
- Download Visual Studio 2019 or later (Community edition is fine)
- During installation, select "Desktop development with C++"
- Make sure Windows 10 SDK is included

### 3. Install Build Tools
```powershell
# Install cargo-make (optional, for easier building)
cargo install cargo-make
```

## Building the DLL

### Option 1: Using Cargo (Recommended)

```powershell
# Navigate to the keymagic-windows directory
cd keymagic-windows

# Build release version
cargo build --release

# The DLL will be located at:
# target\release\keymagic_windows.dll
```

### Option 2: Debug Build (for development)

```powershell
# Build debug version with symbols
cargo build

# The DLL will be at:
# target\debug\keymagic_windows.dll
```

### Troubleshooting Build Issues

If you encounter build errors:

1. **Missing Windows SDK**
   ```powershell
   # Check if cl.exe is available
   where cl
   # If not found, run Visual Studio Developer Command Prompt
   ```

2. **Linking Errors**
   - Ensure Visual Studio is properly installed
   - Try running from "Developer Command Prompt for VS"

3. **Rust Compilation Errors**
   ```powershell
   # Update Rust toolchain
   rustup update
   # Clean and rebuild
   cargo clean
   cargo build --release
   ```

## Installing the TSF DLL

### 1. Copy Required Files

Create an installation directory and copy files:
```powershell
# Create directory (e.g., in Program Files)
mkdir "C:\Program Files\KeyMagic"

# Copy the DLL
copy target\release\keymagic_windows.dll "C:\Program Files\KeyMagic\"

# Copy any KM2 keyboard files you want to use
copy path\to\your\keyboard.km2 "C:\Program Files\KeyMagic\"
```

### 2. Register the DLL (Administrator Required)

**Important**: You must run Command Prompt as Administrator for this step.

```powershell
# Open Command Prompt as Administrator
# Navigate to installation directory
cd "C:\Program Files\KeyMagic"

# Register the DLL
regsvr32 keymagic_windows.dll

# You should see: "DllRegisterServer in keymagic_windows.dll succeeded."
```

### 3. Configure Windows to Use the IME

1. **Open Windows Settings**
   - Press `Win + I`
   - Go to "Time & Language" → "Language"

2. **Add Input Method**
   - Click on your language (e.g., "English (United States)")
   - Click "Options"
   - Under "Keyboards", click "Add a keyboard"
   - Look for "KeyMagic" in the list
   - Select it to add

3. **Switch to KeyMagic IME**
   - Use `Win + Space` to cycle through input methods
   - Or click the language indicator in the system tray
   - Select "KeyMagic"

## Uninstalling

### 1. Remove from Windows Settings
- Go to Settings → Time & Language → Language
- Click your language → Options
- Find "KeyMagic" under keyboards
- Click it and select "Remove"

### 2. Unregister the DLL (Administrator Required)
```powershell
# Run as Administrator
cd "C:\Program Files\KeyMagic"
regsvr32 /u keymagic_windows.dll
```

### 3. Delete Files
```powershell
# Remove the installation directory
rmdir /s "C:\Program Files\KeyMagic"
```

## Testing the Installation

### 1. Basic Test
- Open Notepad
- Switch to KeyMagic IME (Win + Space)
- Try typing - characters should appear

### 2. Load a Keyboard Layout
Currently, you need to modify the code to specify which KM2 file to load, as the UI for keyboard selection is not yet implemented.

### 3. Debug Mode
If the IME isn't working:

1. **Check Event Viewer**
   - Open Event Viewer (Win + X, V)
   - Look under Windows Logs → Application
   - Check for errors related to keymagic_windows.dll

2. **Enable TSF Logging**
   ```powershell
   # Set environment variable for TSF debugging
   set TF_DEBUG=1
   ```

3. **Use Debug Build**
   - Build with `cargo build` (without --release)
   - Register the debug DLL instead
   - Attach debugger to the process using the IME

## Development Tips

### Quick Iteration
For development, create a batch script to rebuild and re-register:

```batch
@echo off
REM rebuild_install.bat
cargo build --release
if %errorlevel% neq 0 exit /b %errorlevel%

echo Unregistering old DLL...
regsvr32 /s /u "C:\Program Files\KeyMagic\keymagic_windows.dll"

echo Copying new DLL...
copy /y target\release\keymagic_windows.dll "C:\Program Files\KeyMagic\"

echo Registering new DLL...
regsvr32 /s "C:\Program Files\KeyMagic\keymagic_windows.dll"

echo Done!
```

### Testing without Full Installation
For testing the FFI layer without TSF:
```powershell
# Compile the test program
cl examples\test_ffi.c /I include /link target\release\keymagic_windows.lib

# Run test
test_ffi.exe
```

## Known Limitations

1. **No UI for keyboard selection** - Currently hardcoded or needs manual modification
2. **No settings persistence** - Settings are lost on restart
3. **Limited composition UI** - Basic functionality only
4. **No language bar icon** - Uses default icon

## Next Steps

After successful installation, you'll need to:
1. Load a KM2 keyboard file (currently requires code modification)
2. Test with various applications (Notepad, Word, browsers)
3. Report any issues or crashes

For development and debugging, see the main README.md for more details.