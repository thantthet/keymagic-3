# IBus Implementation Plan

## Overview

Implementation of Linux input method support using IBus framework. This follows the single-engine approach where one IBus engine dynamically switches between keyboard layouts based on configuration file monitoring.

## Architecture

### Core Design Principles
- **Single IBus Engine**: One engine instance handles all keyboard layouts
- **Direct Config Reading**: Engine reads `~/.config/keymagic/config.toml` directly
- **File-Based Communication**: No D-Bus needed between GUI and engine
- **On-Demand Loading**: Keyboards loaded only when needed
- **Silent Error Handling**: Failed keyboards eat keys without visual errors
- **Bundled Installation**: Ships with GUI package

### Component Structure

```
keymagic-ibus/
├── src/
│   ├── engine.c           # Main IBus engine implementation
│   ├── config.c           # TOML config parsing
│   ├── ffi_bridge.c       # Rust keymagic-core FFI
│   └── main.c             # IBus component registration
├── data/
│   └── keymagic.xml       # IBus component definition
└── Makefile               # Build configuration
```

## Implementation Tasks

### ✅ Phase 1: Architecture & Setup
- [x] Design IBus engine architecture with direct config file reading
- [x] Create project structure for keymagic-ibus
- [ ] Set up build system integration with GUI

### ✅ Phase 2: Core Engine
- [x] Implement IBus engine skeleton with basic structure
- [x] Add FFI bridge to keymagic-core
- [x] Implement config file parsing (TOML)
- [x] Add config file monitoring for keyboard layout changes

### ✅ Phase 3: Key Processing
- [x] Implement preedit text handling following TSF pattern
- [x] Add key event processing pipeline
- [x] Implement on-demand keyboard loading
- [x] Add silent error handling - eat keys when no valid keyboard

### ⏳ Phase 4: Integration
- [ ] Add IBus engine registration and bundle with GUI build
- [ ] Create installation scripts
- [ ] Add desktop integration
- [x] Testing and debugging - Engine builds and runs successfully!

## Technical Specifications

### IBus Engine Structure
```c
struct KeyMagicEngine {
    IBusEngine parent;
    
    // Core engine
    EngineHandle* km_engine;        // FFI handle to keymagic-core (NULL if none)
    char* active_keyboard_id;       // Current keyboard ID from config
    char* keyboard_path;            // Path to current .km2 file
    
    // Config monitoring
    GFileMonitor* config_monitor;   // Monitor ~/.config/keymagic/config.toml
    char* config_path;              // Path to config file
    
    // Error handling
    gboolean keyboard_load_failed;  // Track if current keyboard failed to load
    
    // Preedit state
    IBusText* preedit_text;         // Current preedit text
    gboolean preedit_visible;       // Whether preedit is shown
};
```

### Preedit Behavior (Following TSF Pattern)

Based on `CompositionEditSession.cpp`:

1. **Composing State**: 
   - Show engine's composing text as preedit (underlined)
   - Update preedit on each key event with engine output

2. **Commit Triggers**:
   - Engine returns empty composing text
   - Special keys: Enter, Tab, Escape
   - Unprocessed keys (engine didn't handle)

3. **Reset Conditions**:
   - Focus changes
   - Navigation keys (Arrow keys, Home, End, etc.)
   - Explicit reset calls

### Config Integration

```c
// Config file structure (matches linux.rs)
typedef struct {
    char* active_keyboard;          // keyboards.active
    // Other fields as needed
} KeyMagicConfig;

// File monitoring callback
static void on_config_changed(GFileMonitor* monitor, GFile* file, 
                              GFile* other_file, GFileMonitorEvent event_type, 
                              KeyMagicEngine* engine) {
    // 1. Parse updated TOML config
    // 2. Check if keyboards.active changed
    // 3. Unload current keyboard if different
    // 4. Load new keyboard on next key event (lazy loading)
}
```

### Error Handling Strategy

```c
static gboolean process_key_event(IBusEngine* engine, guint keyval, guint keycode, 
                                  guint modifiers) {
    KeyMagicEngine* km_engine = (KeyMagicEngine*)engine;
    
    // Silent error handling - eat printable keys when no valid keyboard
    if (!km_engine->km_engine || km_engine->keyboard_load_failed) {
        if (is_printable_ascii_key(keyval)) {
            return TRUE;  // Eat the key
        }
        return FALSE;     // Pass through non-printable keys
    }
    
    // Normal key processing with engine...
    ProcessKeyOutput output = {0};
    KeyMagicResult result = keymagic_engine_process_key(
        km_engine->km_engine, keyval, /* ... */);
    
    if (result == KeyMagicResult_Success) {
        // Update preedit based on output.composing_text
        update_preedit(km_engine, output.composing_text);
        
        // Check commit conditions
        if (should_commit_composition(keyval, output)) {
            commit_and_clear_preedit(km_engine);
        }
        
        return output.is_processed;
    } else {
        // Engine error - mark as failed and eat future keys
        km_engine->keyboard_load_failed = TRUE;
        return is_printable_ascii_key(keyval);
    }
}
```

### Build Integration

```makefile
# keymagic-ibus/Makefile
SOURCES = src/engine.c src/config.c src/ffi_bridge.c src/main.c
CFLAGS = $(shell pkg-config --cflags ibus-1.0 glib-2.0)
LIBS = $(shell pkg-config --libs ibus-1.0 glib-2.0) -lkeymagic_core

ibus-engine-keymagic: $(SOURCES)
	$(CC) $(CFLAGS) -o $@ $(SOURCES) $(LIBS)

install:
	install -D ibus-engine-keymagic $(DESTDIR)/usr/lib/ibus-keymagic/ibus-engine-keymagic
	install -D data/keymagic.xml $(DESTDIR)/usr/share/ibus/component/keymagic.xml
```

```toml
# keymagic-shared/gui/Cargo.toml - Add build script
[build-dependencies]
# Build IBus engine during GUI build

# build.rs will:
# 1. Compile IBus engine
# 2. Link against keymagic-core
# 3. Prepare for bundled installation
```

## Installation & Registration

### Package Contents
```
/usr/lib/ibus-keymagic/
├── ibus-engine-keymagic     # Engine executable
└── libkeymagic_core.so      # Rust library

/usr/share/ibus/component/
└── keymagic.xml             # IBus component definition

/usr/share/applications/
└── keymagic-gui.desktop     # GUI application

~/.config/keymagic/
├── config.toml              # User configuration
└── keyboards/               # User keyboard files
```

### Registration Process
```bash
# During installation:
ibus restart
ibus register-component /usr/share/ibus/component/keymagic.xml

# User activation:
ibus-setup  # Add KeyMagic to input methods
```

## Communication Flow

### Startup Sequence
1. IBus starts `ibus-engine-keymagic`
2. Engine reads `~/.config/keymagic/config.toml`
3. Sets up file monitor on config
4. Prepares for keyboard loading (lazy)

### Normal Operation
1. User types key → IBus → Engine
2. Engine loads keyboard if not loaded
3. Engine processes key via keymagic-core FFI
4. Engine updates preedit or commits text
5. IBus displays result to application

### Configuration Changes
1. User changes keyboard in GUI
2. GUI updates `config.toml`
3. File monitor detects change
4. Engine notes keyboard change
5. Next key event triggers keyboard reload

### Error Recovery
1. Keyboard loading fails
2. Engine sets `keyboard_load_failed = TRUE`
3. Engine eats all printable keys silently
4. User fixes configuration
5. File monitor detects fix
6. Engine retries on next key event

## Development Timeline

- **Week 1**: Project setup, build system, basic engine skeleton
- **Week 2**: Config parsing, file monitoring, FFI bridge
- **Week 3**: Key processing pipeline, preedit handling
- **Week 4**: Error handling, testing, integration with GUI build
- **Week 5**: Installation scripts, packaging, documentation

## Notes

- No D-Bus communication needed between GUI and engine
- Following TSF preedit behavior ensures consistency across platforms
- Silent error handling prevents user confusion
- File-based communication is simpler and more reliable
- Bundled installation simplifies deployment