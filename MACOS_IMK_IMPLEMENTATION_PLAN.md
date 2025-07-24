# macOS Input Method Kit (IMK) Implementation Plan for KeyMagic

## Overview

This document outlines the implementation plan for integrating KeyMagic with macOS using the Input Method Kit (IMK) framework. The plan is based on analysis of the existing Windows TSF and Linux IBus implementations.

## Architecture Overview

### Component Structure
```
keymagic-macos/
├── CMakeLists.txt          # Build configuration
├── src/
│   ├── KMInputController.h     # IMKInputController subclass
│   ├── KMInputController.m     # Key event handling & composition
│   ├── KMServer.h             # IMKServer wrapper
│   ├── KMServer.m             # Server initialization
│   ├── KMFFIBridge.h          # FFI interface to Rust
│   ├── KMFFIBridge.c          # FFI implementation
│   ├── KMKeyMapping.h         # Key code conversion
│   ├── KMKeyMapping.m         # macOS to VirtualKey mapping
│   ├── KMConfiguration.h      # Config management
│   ├── KMConfiguration.m      # Keyboard loading & settings
│   └── main.m                 # Entry point
├── include/
│   └── keymagic_ffi.h         # Shared FFI header
├── Info.plist                 # Input method metadata
└── resources/
    └── icon.icns              # Application icon
```

## Implementation Components

### 1. FFI Bridge Design

Based on the Windows and Linux implementations, the FFI bridge will expose:

```c
// Engine lifecycle
KeyMagicEngine* keymagic_engine_new(void);
void keymagic_engine_free(KeyMagicEngine* engine);

// Keyboard management
bool keymagic_engine_load_keyboard(KeyMagicEngine* engine, const char* path);
void keymagic_engine_unload_keyboard(KeyMagicEngine* engine);

// Key processing
KeyMagicResult keymagic_engine_process_key_mac(
    KeyMagicEngine* engine,
    uint16_t keycode,      // macOS virtual keycode
    uint32_t modifiers,    // NSEvent modifier flags
    const char* chars      // Character representation
);

// State management
void keymagic_engine_reset(KeyMagicEngine* engine);
char* keymagic_engine_get_composition(KeyMagicEngine* engine);
void keymagic_engine_set_composition(KeyMagicEngine* engine, const char* text);
```

### 2. IMKInputController Implementation

The main input controller will handle:

#### Key Event Processing
```objc
- (BOOL)handleEvent:(NSEvent*)event client:(id)sender {
    // 1. Filter key events (only keyDown)
    // 2. Extract keycode, modifiers, characters
    // 3. Call FFI bridge
    // 4. Process result (update marked text, commit, etc.)
    // 5. Return YES if processed
}
```

#### Composition Management
```objc
// Marked text (composition) display
- (void)updateMarkedText:(NSString*)text {
    NSAttributedString* markedText = [self createMarkedString:text];
    [client setMarkedText:markedText
        selectionRange:NSMakeRange(text.length, 0)
        replacementRange:NSMakeRange(NSNotFound, 0)];
}

// Commit text
- (void)commitText:(NSString*)text {
    [client insertText:text replacementRange:NSMakeRange(NSNotFound, 0)];
    [self clearMarkedText];
}
```

#### State Synchronization
- On activate: Load keyboard, reset engine
- On deactivate: Commit pending text, save state
- On client change: Reset composition

### 3. Key Code Mapping

Map macOS virtual keycodes to KeyMagic VirtualKey enum:

```objc
typedef struct {
    uint16_t macKeyCode;
    VirtualKey kmVirtualKey;
} KeyMapping;

static const KeyMapping keyMappings[] = {
    { kVK_ANSI_A, VK_KEY_A },
    { kVK_ANSI_B, VK_KEY_B },
    // ... complete mapping table
};
```

### 4. Configuration Management

#### Shared Configuration with GUI
- Read from same location as GUI: `~/Library/Preferences/net.keymagic/config.toml`
- Keyboards stored in: `~/Library/Application Support/KeyMagic/Keyboards/`
- Monitor config file for changes using DispatchSource
- Load active keyboard on startup and config changes

#### Configuration Structure (TOML)
```toml
[general]
start_with_system = false
check_for_updates = true

[keyboards]
active = "myanmar3"
last_used = ["myanmar3", "zawgyi"]

[[keyboards.installed]]
id = "myanmar3"
name = "Myanmar3 Unicode"
filename = "myanmar3.km2"
hash = "abc123..."

[composition_mode]
enabled_processes = ["Safari", "Chrome", "Firefox"]
```

#### Implementation
- `KMConfiguration.swift` handles all config management
- Singleton pattern for easy access throughout IMK
- Automatic reload on config file changes
- Fallback to searching keyboards directory by ID

### 5. Composition State Management

Based on both implementations, maintain:

1. **Engine State**
   - Persistent composing buffer in Rust engine
   - Active states for state-based rules
   - History for smart backspace

2. **IMK State**
   - Current marked text range
   - Client connection
   - Active keyboard ID

3. **Synchronization Points**
   - Client changes (new text field)
   - Keyboard switches
   - Focus changes
   - Mouse events

### 6. Build System

#### CMake Configuration
```cmake
cmake_minimum_required(VERSION 3.10)
project(KeyMagicMacOS)

# Find required frameworks
find_library(FOUNDATION Foundation)
find_library(CARBON Carbon)
find_library(INPUTMETHODKIT InputMethodKit)

# Link with Rust static library
add_library(keymagic_core STATIC IMPORTED)
set_target_properties(keymagic_core PROPERTIES
    IMPORTED_LOCATION "${CMAKE_SOURCE_DIR}/../target/release/libkeymagic_core.a"
)

# Build input method bundle
add_executable(KeyMagic MACOSX_BUNDLE ${SOURCES})
target_link_libraries(KeyMagic
    ${FOUNDATION}
    ${CARBON}
    ${INPUTMETHODKIT}
    keymagic_core
)
```

### 7. Installation and Registration

#### Info.plist Configuration
```xml
<key>InputMethodConnectionName</key>
<string>KeyMagic_Connection</string>
<key>InputMethodServerControllerClass</key>
<string>KMInputController</string>
<key>tsInputMethodCharacterRepertoireKey</key>
<array>
    <string>my-MM</string>
    <string>und-Mymr</string>
</array>
```

#### Installation Script
```bash
#!/bin/bash
# Copy to input methods directory
cp -R KeyMagic.app ~/Library/Input\ Methods/
# Register with system
/System/Library/Frameworks/Carbon.framework/Versions/A/Support/TISRegisterInputSource ~/Library/Input\ Methods/KeyMagic.app
```

## Implementation Phases

### Phase 1: Basic Setup (Week 1)
- [x] Create project structure
- [ ] Set up CMake build system
- [ ] Implement basic FFI bridge
- [ ] Create minimal IMKInputController

### Phase 2: Core Functionality (Week 2-3)
- [ ] Implement key event handling
- [ ] Add key code mapping
- [ ] Integrate with keymagic-core
- [ ] Basic marked text display

### Phase 3: State Management (Week 4)
- [ ] Implement composition management
- [ ] Add state synchronization
- [ ] Handle focus/client changes
- [ ] Implement commit triggers

### Phase 4: Configuration (Week 5)
- [ ] Add keyboard loading
- [ ] Implement settings management
- [ ] Support hot-reload
- [ ] Error handling

### Phase 5: Polish & Testing (Week 6)
- [ ] Add logging/debugging
- [ ] Create unit tests
- [ ] Integration testing
- [ ] Documentation

## Key Differences from Other Platforms

### Compared to Windows TSF:
- IMK is simpler, no edit sessions needed
- Direct marked text API instead of composition objects
- Simpler threading model (main thread only)

### Compared to Linux IBus:
- No GObject/GLib dependencies
- Native Objective-C instead of C
- Built-in marked text support
- Simpler configuration (NSUserDefaults)

## Testing Strategy

1. **Unit Tests**
   - Key mapping correctness
   - FFI bridge functionality
   - Configuration management

2. **Integration Tests**
   - Full key processing flow
   - State synchronization
   - Keyboard switching

3. **Manual Testing**
   - Various macOS applications
   - Different keyboard layouts
   - Edge cases (rapid typing, etc.)

## Success Criteria

- Functional parity with Windows/Linux implementations
- Native macOS look and feel
- Stable operation across macOS versions
- Performance target: <10ms key processing latency
- Proper state management and synchronization