# KeyMagic TSF Text Service

This directory contains the Windows Text Services Framework (TSF) implementation for KeyMagic.

## Building

### Prerequisites
- Visual Studio 2022 or later
- CMake 3.20 or later
- Windows SDK
- Rust toolchain (for building keymagic-core)

### Build Steps

1. First build keymagic-core:
   ```cmd
   cd ../..
   cargo build --release
   ```

2. Generate the Visual Studio project:
   ```cmd
   cd keymagic-windows/tsf
   mkdir build
   cd build
   cmake -G "Visual Studio 17 2022" -A x64 ..
   ```

3. Build the TSF DLL:
   ```cmd
   cmake --build . --config Release
   ```

## Installation

1. Register the DLL:
   ```cmd
   regsvr32 KeyMagicTSF.dll
   ```

2. The text service will appear in Windows language settings.

## Uninstallation

```cmd
regsvr32 /u KeyMagicTSF.dll
```

## Development

The TSF implementation follows the simplified text handling strategy:
- Always displays the engine's composing text as the composition string
- Commits text on space, enter, tab, or focus loss
- Resets the engine after each commit

Key files:
- `KeyMagicTextService.cpp/h` - Main TSF implementation
- `keymagic_ffi.h` - FFI interface to keymagic-core
- `DllMain.cpp` - DLL entry points and COM registration