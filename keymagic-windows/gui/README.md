# KeyMagic Configuration Manager

A native Windows GUI application for managing KeyMagic keyboards and settings.

## Features

- Manage installed KeyMagic keyboards
- Add/remove keyboard layouts (.km2 files)
- Configure hotkeys for keyboard switching
- System tray integration
- Keyboard preview and testing

## Building

### Prerequisites
- Rust toolchain with MSVC target
- Windows SDK

### Build Steps

```cmd
cd keymagic-windows/gui
cargo build --release
```

The executable will be created at `target/release/keymagic-config.exe`

## Development

The application uses windows-rs for native Win32 API access.

Key modules:
- `main.rs` - Application entry point and message loop
- `app.rs` - Application state management
- `window.rs` - Main window implementation
- `keyboard_manager.rs` - Keyboard management using keymagic-core FFI
- `registry.rs` - Windows registry operations
- `tray.rs` - System tray functionality

## Architecture

The GUI directly uses the keymagic-core FFI interface to:
- Load and validate .km2 files
- Extract keyboard metadata and icons
- Test keyboard input (preview functionality)