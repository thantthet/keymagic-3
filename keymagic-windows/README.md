# KeyMagic Windows Implementation

This directory contains the Windows-specific implementation of KeyMagic, including:

- **TSF Text Service** - Windows Text Services Framework IME implementation
- **Configuration Manager** - Native Win32 GUI for managing keyboards and settings

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
│       ├── Registry.*    # Registration helpers
│       └── Globals.*     # Global state
│
├── gui/                   # Configuration Manager (Rust)
│   ├── Cargo.toml        # Rust package configuration
│   ├── build.rs          # Build script for resources
│   ├── src/              # Rust source files
│   │   ├── main.rs       # Application entry point
│   │   ├── app.rs        # Application state
│   │   └── window.rs     # Main window implementation
│   └── resources/        # Application resources
│       └── app.manifest  # Windows manifest
│
└── installer/            # Installation package (future)

```

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
   cd keymagic-windows/gui
   cargo build --release
   ```

## Installation

1. Register the TSF DLL:
   ```cmd
   regsvr32 KeyMagicTSF.dll
   ```

2. Run the Configuration Manager:
   ```cmd
   keymagic-config.exe
   ```

## Development Status

### Phase 5.1: Foundation Setup ✅
- [x] Create project structure
- [x] Set up build systems
- [x] Implement basic COM server
- [x] Create basic Win32 window

### Phase 5.2: Core Functionality (Next)
- [ ] Implement key processing pipeline
- [ ] Basic keyboard management
- [ ] Initial TSF-GUI integration

### Phase 5.3: Advanced Features
- [ ] Smart backspace support
- [ ] Language bar integration
- [ ] System tray functionality
- [ ] Hotkey configuration

### Phase 5.4: Testing and Polish
- [ ] Unit tests
- [ ] Integration testing
- [ ] Performance optimization

### Phase 5.5: Deployment
- [ ] Installer creation
- [ ] Documentation
- [ ] Release preparation

## Architecture Notes

The implementation follows a simplified text handling strategy:
- The TSF always displays the engine's composing text as the composition string
- Text is committed on space, enter, tab, or focus loss
- The engine is reset after each commit
- Action types from the engine are ignored in favor of this simpler model

This approach significantly reduces TSF complexity while maintaining full compatibility with KeyMagic's rule-based engine.