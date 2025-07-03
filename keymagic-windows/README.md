# KeyMagic Windows

Windows implementation of KeyMagic IME using Text Services Framework (TSF) and Rust.

## Overview

This project provides a complete KeyMagic input method solution for Windows, consisting of:

1. **TSF Integration** - A Windows Text Services Framework DLL that provides the actual IME functionality
2. **Manager GUI** - A Windows application for managing keyboard layouts

## Components

### TSF Integration (`keymagic_windows.dll`)

The core IME implementation that:
- Integrates with Windows Text Services Framework
- Processes keyboard input using the KeyMagic engine
- Loads keyboard layouts from .km2 files
- Provides debug output for troubleshooting

**Key Features:**
- Written in Rust with C++ TSF integration
- Supports all Windows applications
- Debug logging via OutputDebugString
- Automatic keyboard loading from registry

### Manager GUI (`KeyMagicManager.exe`)

A user-friendly Windows application that:
- Manages KeyMagic keyboard layouts
- Allows adding/removing .km2 keyboard files
- Activates keyboards for use with the TSF
- Stores configuration in Windows registry

**Key Features:**
- Simple ListView interface
- File browser for .km2 files
- One-click keyboard activation
- Persistent keyboard storage

## Quick Start

### Prerequisites

- Windows 10/11 (ARM64 or x64)
- Visual Studio 2022 or Build Tools for Visual Studio
- Rust toolchain (install from https://rustup.rs)
- Administrator privileges (for TSF installation)

### Building

1. **Build the TSF DLL:**
   ```cmd
   cd tsf
   build_windows.bat
   ```

2. **Build the Manager GUI:**
   ```cmd
   cd manager
   build_with_vs.bat
   ```

### Installation

1. **Install the TSF (Run as Administrator):**
   ```cmd
   cd tsf
   install.bat
   ```

2. **Configure keyboards:**
   - Run `manager\KeyMagicManager.exe`
   - Click "Add..." to add .km2 keyboard files
   - Select a keyboard and click "Activate"

3. **Enable in Windows:**
   - Go to Settings → Time & Language → Language
   - Click on your language and select Options
   - Add KeyMagic as a keyboard
   - Use Win+Space to switch between keyboards

## Project Structure

```
keymagic-windows/
├── tsf/                    # Text Services Framework component
│   ├── src/               # TSF source files
│   ├── include/           # Public headers
│   ├── build_windows.bat  # TSF build script
│   ├── install.bat        # TSF install script
│   └── Cargo.toml         # Rust project config
├── manager/                # GUI Manager application
│   ├── KeyMagicManager.exe
│   └── source/build files
└── docs/                   # Documentation
```

## Documentation

- [Build & Installation Guide](docs/BUILD_INSTALL.md)
- [Usage Guide](docs/USAGE_GUIDE.md)
- [TSF Debugging Guide](docs/TSF_DEBUGGING.md)
- [Uninstall Help](docs/UNINSTALL_HELP.md)

## Development

### Debugging

1. Use DebugView to see real-time debug output
2. Check `%TEMP%\KeyMagicTSF_*.log` for log files
3. See [TSF Debugging Guide](docs/TSF_DEBUGGING.md) for details

### Testing

1. Build debug version with `cargo build`
2. Use included test programs in `examples/`
3. Test with various Windows applications

## License

[License information here]

## Contributing

[Contributing guidelines here]