# KeyMagic Windows Implementation

This directory contains the Windows-specific implementation of KeyMagic, including:

- **TSF Text Service** - Windows Text Services Framework IME implementation (C++)
- **Configuration Manager** - Tauri 2.0-based GUI for managing keyboards and settings

## Project Structure

```
keymagic-windows/
├── tsf/                    # TSF Text Service (C++)
│   ├── CMakeLists.txt     # CMake build configuration
│   ├── include/           # Header files
│   │   └── keymagic_ffi.h # FFI interface to keymagic-core
│   └── src/              # C++ source files
│       ├── DllMain.cpp   # DLL entry points
│       ├── ClassFactory.* # COM class factory
│       ├── KeyMagicTextService.* # Main TSF implementation
│       ├── CompositionManager.* # TSF composition handling
│       ├── DisplayAttribute.* # Text formatting (underline)
│       ├── EditSession.* # TSF edit session handlers
│       ├── Registry.*    # Registration helpers
│       └── Globals.*     # Global state
│
├── gui-tauri/             # Configuration Manager (Tauri 2.0)
│   ├── src-tauri/        # Rust backend
│   │   ├── Cargo.toml    # Rust package configuration
│   │   └── src/          # Rust source files
│   │       ├── main.rs   # Tauri application entry
│   │       ├── registry.rs # Windows registry operations
│   │       ├── keyboard_manager.rs # Keyboard management
│   │       └── hud.rs    # Native Windows HUD
│   ├── src/              # Frontend (HTML/CSS/JS)
│   │   ├── index.html    # Main UI
│   │   ├── styles.css    # Styling
│   │   └── main.js       # Frontend logic
│   └── tauri.conf.json   # Tauri configuration
│
└── installer/            # Installation package (future)

```

## Version Management

KeyMagic Windows uses a centralized version management system. See [VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md) for details.

- **Current Version**: 3.0.0
- **Version File**: `version.txt`
- **Update Script**: `update-version.bat`
- **Check Script**: `check-versions.bat`

## Building

### Prerequisites
- Visual Studio 2022 or later
- CMake 3.20+
- Rust toolchain (stable-msvc)
- Windows SDK

### Build All Components

1. Build keymagic-core (from repository root):
   ```cmd
   cargo build --release
   ```

2. Build TSF Text Service:
   ```cmd
   cd keymagic-windows/tsf
   mkdir build && cd build
   cmake -G "Visual Studio 17 2022" -A x64 ..
   cmake --build . --config Release
   ```

3. Build Configuration Manager:
   ```cmd
   cd keymagic-windows/gui-tauri
   npm install
   npm run tauri build
   ```

## Installation

1. Register the TSF DLL:
   ```cmd
   regsvr32 KeyMagicTSF.dll
   ```

2. Run the Configuration Manager:
   ```cmd
   keymagic.exe
   ```

## Development Status

### Phase 5.1: Foundation Setup ✅
- [x] Create project structure
- [x] Set up build systems
- [x] Implement basic COM server
- [x] Create basic Win32 window (replaced with Tauri)

### Phase 5.2: Core Functionality ✅
- [x] Implement key processing pipeline (SendInput-based)
- [x] Basic keyboard management
- [x] Initial TSF-GUI integration via registry
- [x] Engine integration via FFI
- [x] Registry monitoring for real-time updates
- [x] Thread-safe operations with CRITICAL_SECTION
- [x] Debug logging system

### Phase 5.3: Advanced Features (In Progress)
- [x] System tray functionality (Tauri GUI)
- [x] Hotkey configuration (Tauri GUI)
- [x] Language bar integration (TSF)
- [x] Auto-update functionality (Tauri)
- [x] Native Windows HUD for notifications
- [ ] Smart backspace support (partially implemented)
- [ ] Full composition support (currently using SendInput approach)

### Phase 5.4: Testing and Polish
- [ ] Unit tests
- [ ] Integration testing
- [ ] Performance optimization

### Phase 5.5: Deployment
- [ ] Installer creation
- [ ] Documentation
- [ ] Release preparation

## Architecture Notes

### TSF Implementation
The TSF text service uses two processing approaches:
1. **SendInput-based (Primary)**: Uses Windows SendInput API for direct text manipulation
   - Processes keys through the engine
   - Sends backspaces and text directly to applications
   - Handles special keys (Space/Enter/Tab for commit, Escape for cancel)
   - Uses extra info signatures to prevent recursive processing

2. **TSF Edit Sessions (Fallback)**: Traditional TSF approach for compatibility
   - Uses TSF context manipulation
   - Better support for TSF-aware applications

### GUI Implementation
The Configuration Manager uses Tauri 2.0 (instead of native Win32):
- **Frontend**: HTML/CSS/JavaScript for modern UI
- **Backend**: Rust with Tauri framework
- **Features**: System tray, hotkey management, auto-updates, native HUD
- **Integration**: Direct FFI to keymagic-core, Windows Registry for persistence

### Registry Integration
Both components share configuration via Windows Registry:
- Location: `HKEY_CURRENT_USER\Software\KeyMagic\Settings`
- TSF monitors registry for real-time updates
- GUI manages keyboard registration and settings

This architecture provides a modern, maintainable solution while ensuring compatibility with Windows applications.