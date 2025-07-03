# KeyMagic Windows Integration

This crate provides Windows Text Services Framework (TSF) integration for KeyMagic.

## Architecture

The Windows integration consists of three main components:

1. **Rust FFI Layer** (`ffi.rs`) - Provides C-compatible functions to interact with the KeyMagic core engine
2. **C++ TSF Implementation** (`KeyMagicTextService.cpp`) - Implements the Windows TSF interfaces
3. **DLL Entry Points** (`DllMain.cpp`) - Handles COM registration and DLL lifecycle

## Building

### Prerequisites

- Windows 10 SDK or later
- Visual Studio 2019 or later with C++ workload
- Rust with `x86_64-pc-windows-msvc` target

### Build Steps

```bash
# On Windows
cargo build --release --target x86_64-pc-windows-msvc
```

This will produce `keymagic_windows.dll` in `target/x86_64-pc-windows-msvc/release/`.

## Installation

1. Build the DLL as described above
2. Register the DLL as Administrator:
   ```cmd
   regsvr32 keymagic_windows.dll
   ```
3. The KeyMagic input method will appear in Windows language settings

## Development

### Testing the FFI Layer

```bash
# Compile test program (requires C compiler)
cl examples/test_ffi.c /I include /link keymagic_windows.lib

# Run test
test_ffi.exe
```

### Debugging

1. Attach debugger to the application using the IME
2. Set breakpoints in the C++ code
3. Monitor debug output in Visual Studio

## Implementation Status

- [x] FFI bridge to KeyMagic core
- [x] Basic TSF framework
- [x] COM registration
- [ ] Full composition handling
- [ ] Display attribute provider
- [ ] Language bar integration
- [ ] Settings UI
- [ ] Keyboard layout loading UI

## Known Issues

- Composition string display needs full implementation
- No UI for loading keyboard layouts yet
- Language bar icon not implemented

## References

- [Text Services Framework](https://docs.microsoft.com/windows/win32/tsf/text-services-framework)
- [TSF Sample](https://github.com/microsoft/Windows-classic-samples/tree/master/Samples/Win7Samples/winui/input/tsf/textservice)