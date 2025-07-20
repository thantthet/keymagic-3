# KeyMagic IBus Engine

Linux input method engine for KeyMagic using the IBus framework.

## Overview

This IBus engine provides KeyMagic input method support on Linux systems. It features:

- **Single Engine Architecture**: One engine handles all keyboard layouts
- **File-Based Configuration**: Monitors `~/.config/keymagic/config.toml` for changes
- **On-Demand Loading**: Keyboards loaded only when needed
- **Silent Error Handling**: Failed keyboards eat keys without visual errors
- **Preedit Support**: Following TSF implementation pattern for consistency

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Application   │◄──►│   IBus Daemon    │◄──►│ KeyMagic Engine │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                         │
                                                         ▼
                                                ┌─────────────────┐
                                                │ keymagic-core   │
                                                │  (Rust FFI)    │
                                                └─────────────────┘
```

## Project Structure

```
keymagic-ibus/
├── src/
│   ├── main.c           # IBus component and main entry point
│   ├── engine.h/.c      # Core engine implementation
│   ├── config.h/.c      # TOML configuration parsing
│   └── ffi_bridge.h/.c  # Rust keymagic-core FFI bridge
├── data/
│   └── keymagic.xml     # IBus component definition
├── Makefile             # Build configuration
└── README.md            # This file
```

## Building

### Dependencies

- IBus development headers (`libibus-1.0-dev`)
- GLib development headers (`libglib2.0-dev`)
- KeyMagic core library (`keymagic-core`) - built from source

### Build Commands

```bash
# Check dependencies
make check-deps

# Build engine (statically linked with keymagic-core)
make

# Debug build
make debug

# Install to system
sudo make install

# Development install (with IBus registration)
sudo make dev-install
```

The engine is statically linked with `keymagic-core`, so no runtime dependencies on the Rust library are needed.

## Installation

### System Installation

```bash
# Build and install
make && sudo make install

# Register with IBus
ibus restart
ibus register-component /usr/share/ibus/component/keymagic.xml

# Add to input methods via ibus-setup
ibus-setup
```

### File Locations

After installation:

```
/usr/lib/ibus-keymagic/
└── ibus-engine-keymagic     # Engine executable (statically linked)

/usr/share/ibus/component/
└── keymagic.xml             # IBus component definition

~/.config/keymagic/
├── config.toml              # User configuration
└── keyboards/               # User keyboard files
```

## Configuration

The engine reads configuration from `~/.config/keymagic/config.toml`:

```toml
[general]
start_with_system = false
check_for_updates = true

[keyboards]
active = "myanmar3"                    # Current active keyboard
last_used = ["myanmar3", "zawgyi"]

[composition_mode]
enabled_processes = ["firefox", "code"]
```

### Configuration Monitoring

The engine monitors the config file for changes using GFileMonitor:

1. **Startup**: Load initial configuration and active keyboard
2. **File Change**: Detect config file modifications
3. **Keyboard Switch**: Reload if `keyboards.active` changed
4. **Error Handling**: Mark keyboard as failed if loading fails

## Key Processing Flow

### Normal Operation

1. **Key Event**: IBus sends key event to engine
2. **Keyboard Check**: Load keyboard if not loaded or changed
3. **Engine Processing**: Call keymagic-core via FFI
4. **Preedit Update**: Show composing text as underlined preedit
5. **Commit Decision**: Commit text based on engine output and key type

### Error Handling

```c
if (!engine->km_engine || engine->keyboard_load_failed) {
    // Silent error mode - eat printable keys
    return keymagic_engine_is_printable_ascii(keyval);
}
```

### Preedit Behavior (Following TSF)

- **Show Preedit**: When engine has composing text
- **Commit Triggers**:
  - Engine returns empty composing text
  - Special keys: Enter, Tab, Escape
  - Unprocessed keys (engine didn't handle)
- **Reset Conditions**: Focus changes, navigation keys

## FFI Integration

The engine communicates with the Rust `keymagic-core` library through a C FFI bridge:

```c
// Load keyboard
EngineHandle* handle = keymagic_ffi_load_keyboard("/path/to/keyboard.km2");

// Process key
KeyProcessingResult result;
KeyMagicResult status = keymagic_ffi_process_key(handle, keyval, keycode, modifiers, &result);

// Update preedit based on result.composing_text
keymagic_engine_update_preedit(engine, result.composing_text);
```

## Development

### Testing

#### Debug Mode (Without System Installation)

The engine supports debug mode for testing without full IBus registration:

```bash
# Test without installing
./test-debug.sh

# In another terminal:
ibus engine keymagic-debug

# List available engines
ibus list-engine | grep keymagic
```

#### System Testing

```bash
# Install for testing
sudo make dev-install

# View logs
journalctl -f | grep keymagic

# Debug with GDB
gdb /usr/lib/ibus-keymagic/ibus-engine-keymagic
```

#### Manual Testing

```bash
# Run engine directly (debug mode)
G_MESSAGES_DEBUG=all ./ibus-engine-keymagic --verbose

# Run engine as IBus component
./ibus-engine-keymagic --ibus --verbose
```

### Debugging

The engine supports verbose logging:

```bash
# Enable debug logging
export G_MESSAGES_DEBUG=all
/usr/lib/ibus-keymagic/ibus-engine-keymagic --ibus --verbose
```

### Uninstall

```bash
# Remove installation
sudo make uninstall

# Restart IBus
ibus restart
```

## Integration with GUI

The IBus engine integrates with the cross-platform GUI through:

- **Shared Configuration**: Both read `~/.config/keymagic/config.toml`
- **File Monitoring**: Engine detects GUI-initiated keyboard switches
- **No D-Bus**: Simple file-based communication
- **Bundled Installation**: Engine installed with GUI package

## Platform Compliance

This implementation follows Linux desktop standards:

- **XDG Base Directory**: Config in `~/.config`, data in `~/.local/share`
- **IBus Protocol**: Standard IBus engine interface
- **GObject**: Proper GLib/GObject integration
- **Desktop Integration**: Works with GNOME, KDE, and other desktops