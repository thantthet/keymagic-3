# KeyMagic TSF (Text Services Framework)

Windows Text Services Framework integration for KeyMagic IME.

## Overview

This component provides the core IME functionality for Windows by integrating with the Text Services Framework (TSF). It's built using Rust with C++ TSF integration.

## Building

From this directory:
```cmd
cargo build --release
```

Or use the build script:
```cmd
build_windows.bat
```

## Installation

Run as Administrator:
```cmd
install.bat
```

## Features

- Full TSF integration for Windows IME support
- Rust-based engine with C++ TSF wrapper
- Debug logging via OutputDebugString
- Automatic keyboard loading from registry
- Support for all Windows applications

## Project Structure

```
tsf/
├── src/                    # Source files
│   ├── ffi.rs             # Rust FFI implementation
│   ├── lib.rs             # Rust library entry
│   ├── KeyMagicTextService.cpp/h  # TSF implementation
│   ├── DllMain.cpp        # DLL entry point
│   └── ...
├── include/               # Public headers
│   └── keymagic_ffi.h
├── build.rs              # Rust build script
├── Cargo.toml           # Rust project config
└── *.bat                # Build/install scripts
```

## Development

### Debug Build
```cmd
cargo build
```

### View Debug Output
Use DebugView.exe to see real-time debug messages with filter "[KeyMagic]"

### Uninstall
```cmd
force_uninstall.bat
```

## Registry Integration

The TSF reads keyboard configuration from:
`HKEY_CURRENT_USER\Software\KeyMagic\Keyboards`

Use the KeyMagic Manager GUI to configure keyboards.