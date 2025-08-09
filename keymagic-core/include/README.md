# KeyMagic Core Shared Header

This directory contains the unified C header file for the KeyMagic Core library that can be used across all platforms.

## Files

- `keymagic_core.h` - Main unified C/C++ header for FFI interface

## Usage

### For Platform Integrations

Each platform project should include this header instead of maintaining their own copy:

```c
#include <keymagic_core.h>
```

### Platform-Specific Notes

#### Windows
- The header includes Windows-specific functions when `_WIN32` is defined
- Functions like `keymagic_engine_process_key_win()` accept Windows VK codes directly

#### macOS
- Use the standard functions with internal VirtualKey codes
- Convert from macOS key codes to VirtualKey enum values

#### Linux/IBus
- Use the standard functions with internal VirtualKey codes
- Convert from GDK keycodes to VirtualKey enum values

### Key Features

1. **Unified Type Definitions**
   - Standard result codes (`KeyMagicResult`)
   - Common structures (`ProcessKeyOutput`, `HotkeyInfo`)
   - VirtualKey enum for platform-independent key codes

2. **Platform Abstraction**
   - Core functions work with internal VirtualKey codes
   - Platform-specific variants available (e.g., Windows VK codes)

3. **Memory Management**
   - Clear ownership rules (caller must free returned strings)
   - Dedicated free function (`keymagic_free_string`)

4. **Complete API Coverage**
   - Engine management
   - Keyboard loading (file and memory)
   - Key processing
   - State management
   - Metadata access
   - Utility functions

## Migration Guide

To migrate from platform-specific headers:

1. Replace includes:
   ```c
   // Old
   #include "ffi_bridge.h"        // IBus
   #include "KeyMagic-Bridging-Header.h"  // macOS
   #include "keymagic_ffi.h"      // Windows
   
   // New
   #include <keymagic_core.h>
   ```

2. Update type names if needed:
   - Result codes now use `KeyMagicResult_` prefix
   - VirtualKey values use `KeyMagic_VK_` prefix
   - Action types use `KeyMagicAction_` prefix

3. Platform-specific code remains isolated behind preprocessor checks

## Versioning

The header is versioned along with the KeyMagic Core library. Use `keymagic_get_version()` to query the runtime version.