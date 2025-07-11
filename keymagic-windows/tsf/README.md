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
- `KeyMagicGuids.h` - Contains all GUIDs/CLSIDs for the TSF components

### GUIDs and CLSIDs

The project uses the following unique identifiers:
- **CLSID_KeyMagicTextService**: `{094A562B-D08B-4CAF-8E95-8F8031CFD24C}` - COM class ID for the text service
- **GUID_KeyMagicProfile**: `{C29D9340-87AA-4149-A1CE-F6ACAA8AF30B}` - TSF language profile identifier
- **GUID_KeyMagicLangBarButton**: `{9756F03C-080F-4692-B779-25DBEC1FE48F}` - Language bar button identifier
- **GUID_KeyMagicDisplayAttributeInput**: `{2839B100-4CB8-4079-B44B-8032D4C70342}` - Display attribute for composing text

These GUIDs were generated using Windows PowerShell (`[guid]::NewGuid()`) to ensure uniqueness. If you need to generate new GUIDs for any reason, use:
```powershell
[guid]::NewGuid().ToString()
```