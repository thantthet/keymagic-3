# KeyMagic Windows Build System

## Quick Start

```batch
# Build everything
make.bat build

# Register TSF (run as Administrator)
make.bat register

# Run test environment
make.bat test

# Check status
make.bat status
```

## Build Commands

### Build Configurations

The build system supports both Debug and Release configurations:
- **Release** (default): Optimized build for distribution
- **Debug**: Unoptimized build with debug symbols

Use the `--debug` or `--release` flags with any command:

```batch
# Build in Debug mode
make.bat build --debug

# Build in Release mode (default)
make.bat build --release
make.bat build              # Same as --release
```

### Building Components

```batch
# Build all components (TSF + GUI)
make.bat build              # Release mode
make.bat build all          # Release mode
make.bat build --debug      # Debug mode
make.bat build all --debug  # Debug mode

# Build only TSF
make.bat build tsf          # Release mode
make.bat build tsf --debug  # Debug mode

# Build only GUI  
make.bat build gui          # Release mode
make.bat build gui --debug  # Debug mode
```

### Cleaning

```batch
# Clean all build artifacts
make.bat clean              # Clean Release builds
make.bat clean all          # Clean Release builds
make.bat clean --debug      # Clean Debug builds
make.bat clean all --debug  # Clean Debug builds

# Clean only TSF
make.bat clean tsf          # Clean Release TSF
make.bat clean tsf --debug  # Clean Debug TSF

# Clean only GUI
make.bat clean gui          # Clean Release GUI
make.bat clean gui --debug  # Clean Debug GUI
```

### Registration (Requires Administrator)

```batch
# Register TSF with Windows
make.bat register           # Register Release build
make.bat register --debug   # Register Debug build

# Unregister TSF
make.bat unregister
```

**Development-Friendly Registration**:
- The DLL is copied to a temporary location with a random name before registration
- This prevents file locking issues when rebuilding
- You can rebuild immediately after registration without unregistering first
- Old temp DLLs are automatically cleaned up
- Both Debug and Release builds can be registered independently

### Testing

```batch
# Launch interactive test environment
make.bat test               # Test Release build
make.bat test --debug       # Test Debug build

# Check system status
make.bat test status        # Check Release build status
make.bat test status --debug # Check Debug build status
# or
make.bat status
make.bat status --debug
```

### Setup

```batch
# Setup development environment (downloads icons, copies test files)
make.bat setup
make.bat setup dev

# Download icons only
make.bat setup icons
```

## Directory Structure

```
keymagic-windows/
├── make.bat                    # Main build script
├── scripts/
│   ├── config.bat             # Build configuration
│   ├── functions.bat          # Shared functions
│   └── tools/
│       ├── download-icons.ps1     # Icon downloader
│       └── check-diagnostics.ps1  # Diagnostic tool
├── tsf/                       # Text Services Framework
│   └── build/                 # CMake build output
│       ├── Debug/            # Debug build artifacts
│       └── Release/          # Release build artifacts
├── gui/                       # Configuration Manager  
│   └── target/               # Cargo build output
│       ├── debug/            # Debug build artifacts
│       └── release/          # Release build artifacts
├── resources/
│   └── icons/                # Downloaded icons
└── keyboards/                # Test keyboards
```

## Prerequisites

- Windows 10/11 (ARM64 or x64)
- Visual Studio 2022 with C++ support
- CMake 3.20 or later
- Rust toolchain (stable)
- Administrator privileges (for TSF registration)

## Typical Workflow

### First Time Setup

```batch
# 1. Setup development environment (downloads icons)
make.bat setup

# 2. Build everything
make.bat build

# 3. Register TSF (as Administrator)
make.bat register

# 4. Test
make.bat test
```

### Development Iteration

```batch
# Make changes to code...

# Rebuild changed component (Debug mode for faster builds)
make.bat build tsf --debug   # No need to unregister first!

# Test changes
make.bat test --debug
```

**No More "DLL is in use" Errors!**
- The registration system uses temporary copies, so you can rebuild while TSF is registered
- Each build configuration (Debug/Release) maintains its own registration
- Automatic cleanup of old temporary DLLs prevents disk space issues

### Clean Rebuild

```batch
# Clean everything
make.bat clean

# Rebuild
make.bat build
```

## Advanced Usage

### PowerShell Diagnostics

```powershell
# Basic diagnostics
powershell -File scripts\tools\check-diagnostics.ps1

# Detailed diagnostics with logs
powershell -File scripts\tools\check-diagnostics.ps1 -Detailed
```

### Manual Icon Download

```powershell
powershell -ExecutionPolicy Bypass -File scripts\tools\download-icons.ps1
```

## Troubleshooting

### TSF Registration Fails
- Ensure running as Administrator
- Check if DLL exists: `make.bat status`
- Try unregister first: `make.bat unregister`

### Build Fails
- Check Visual Studio 2022 is installed
- Verify CMake is in PATH
- For GUI: ensure Rust is installed

### Test Environment Issues
- Run `make.bat status` to check system state
- Ensure ctfmon.exe is running
- Check Windows Settings > Time & Language > Language

## Configuration

Edit `scripts\config.bat` to customize:
- Build paths
- CMake generator
- Registry keys
- Test keyboard location

## Extending the Build System

To add new commands:
1. Add command routing in `make.bat`
2. Implement command function
3. Update help text
4. Document in this README

Example:
```batch
:package
:: Package for distribution
echo Packaging KeyMagic...
:: Implementation here
goto :eof
```