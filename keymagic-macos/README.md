# KeyMagic3 macOS Input Method

This is the macOS implementation of KeyMagic3 using the Input Method Kit (IMK) framework.

**Note**: This is a standalone macOS project that is not part of the Rust cargo workspace. It uses Swift to integrate with macOS IMK and directly links against the keymagic-core static library.

## Architecture

The implementation consists of:

1. **Swift IMK Components** (`src/swift/`) - Direct integration with keymagic-core
   - `KMInputController.swift` - Main IMK input controller
   - `KeycodeMapping.swift` - macOS virtual keycode to VirtualKey conversion
   - `main.swift` - Application entry point
2. **App Bundle** - Standard macOS input method application bundle

The macOS implementation directly links against keymagic-core's static library:
- Swift calls keymagic-core FFI functions directly via bridging header
- Key mapping is handled in Swift
- No intermediate C bridge needed
- Simpler architecture with fewer layers

## Building

### Prerequisites

- Xcode Command Line Tools
- Rust toolchain
- Swift compiler

### Build Commands

```bash
# Build everything
make all

# Build only the core Rust library
make build-core

# Build only the Swift app bundle
make build-swift

# Clean build artifacts
make clean
```

## Installation

### Automated Installation

```bash
# Install to ~/Library/Input Methods
./install.sh

# Or using make
make install
```

### Manual Installation

1. Build the project: `make all`
2. Copy the app bundle: `cp -R build/KeyMagic3.app ~/Library/Input\ Methods/`
3. Register with the system: `/System/Library/Frameworks/Carbon.framework/Versions/A/Support/TISRegisterInputSource ~/Library/Input\ Methods/KeyMagic3.app`
4. Log out and log back in
5. Add KeyMagic3 in System Preferences > Keyboard > Input Sources

## Uninstallation

```bash
# Automated uninstall
./uninstall.sh

# Or using make
make uninstall
```

## Development

For development, use:

```bash
make dev  # Build and install in one step
```

## Features Implemented

- ✅ FFI bridge to keymagic-core
- ✅ Key event handling and processing
- ✅ Composition/marked text management
- ✅ macOS virtual keycode to VirtualKey mapping
- ✅ Basic IMK integration
- ✅ Installation/uninstallation scripts

## TODO

- [ ] Keyboard configuration management
- [ ] Preferences window
- [ ] Keyboard switching UI
- [ ] Icon and resources
- [ ] Code signing and notarization
- [ ] Distribution package (.dmg)

## Project Structure

```
keymagic-macos/
├── Makefile              # Build system
├── Info.plist            # IMK bundle metadata
├── README.md             # This file
├── install.sh            # Installation script
├── uninstall.sh          # Uninstallation script
├── .gitignore            # Git ignore file
├── build/                # Build output
│   └── KeyMagic3.app/     # IMK app bundle
└── src/
    └── swift/            # Swift IMK implementation
        ├── KeyMagic-Bridging-Header.h
        ├── KMInputController.swift
        ├── KeycodeMapping.swift
        └── main.swift
```

## Architecture Details

### Direct FFI Integration

Swift directly calls keymagic-core FFI functions through the bridging header:

1. **FFI Functions** exposed via bridging header:
   - `keymagic_engine_new()` - Create engine
   - `keymagic_engine_load_keyboard()` - Load .km2 files
   - `keymagic_engine_process_key()` - Process key events
   - `keymagic_engine_reset()` - Reset state
   - etc.

2. **Key Mapping** handled in Swift:
   - `KeycodeMapping.swift` - Contains the macOS virtual keycode → VirtualKey mapping
   - Extension on `UInt16` provides clean conversion
   - Handles Command key filtering (IME should not process Cmd combinations)

### Key Processing Flow

1. IMK sends key events to `KMInputController.handle(_:client:)`
2. Swift extracts keycode, modifiers, and characters
3. Convert macOS keycode to VirtualKey using Swift extension
4. Direct FFI call to keymagic_engine_process_key()
5. Engine returns action (insert, delete, etc.)
6. Swift updates the marked text or commits text to the client

### State Management

- Composing text is maintained in the Rust engine
- Swift syncs this with IMK's marked text display
- State resets on focus changes and mode switches