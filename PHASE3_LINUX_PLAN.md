# Phase 3: Linux Implementation Plan

## Overview

Phase 3 involves two major components:
1. **Cross-Platform GUI**: Port the existing Windows Tauri GUI to work on Linux (and eventually macOS)
2. **IBus Integration**: Create a Linux input method engine using IBus framework

## Part 1: Cross-Platform GUI Port

### 1.1 Project Restructuring - Gradual Migration Approach

To minimize disruption to the existing Windows build pipeline, we'll use a gradual migration approach:

```
Current Structure (Unchanged):
keymagic-windows/
└── gui-tauri/           # Keep existing Windows GUI working

New Structure (Addition):
keymagic-shared/         # New shared cross-platform components
├── gui/                 # New cross-platform GUI (fresh start)
│   ├── src-tauri/
│   ├── src/
│   └── package.json
└── common/              # Shared Rust code (future)
    └── src/
```

#### Migration Strategy

**Phase 1: Parallel Development**
1. Keep `keymagic-windows/gui-tauri` fully functional
2. Create new `keymagic-shared/gui` as a clean cross-platform project
3. No Windows build scripts need updating initially
4. Windows continues using its existing GUI

**Phase 2: Feature Parity**
1. Port features incrementally to cross-platform GUI
2. Test thoroughly on Linux and Windows
3. Maintain both GUIs during transition

**Phase 3: Consolidation** (Future)
1. Once cross-platform GUI is stable and feature-complete
2. Update Windows to use `keymagic-shared/gui`
3. Remove `keymagic-windows/gui-tauri`
4. Update all Windows build scripts at once

#### Benefits of This Approach

- **Zero disruption**: Windows builds continue working unchanged
- **Lower risk**: Can test cross-platform GUI thoroughly before switching
- **Easier rollback**: If issues arise, Windows GUI is unaffected
- **Clean architecture**: Start fresh with proper platform abstractions
- **Gradual validation**: Test each platform independently

#### Initial Project Structure

```
keymagic-shared/gui/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── platform/
│   │   │   ├── mod.rs      # Platform trait definitions
│   │   │   ├── windows.rs  # Windows implementation
│   │   │   ├── linux.rs    # Linux implementation
│   │   │   └── macos.rs    # macOS stub (future)
│   │   ├── core/           # Shared business logic
│   │   │   ├── keyboard_manager.rs
│   │   │   ├── config.rs
│   │   │   └── hotkey.rs
│   │   └── commands.rs     # Tauri commands
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                    # Frontend (shared across platforms)
│   ├── index.html
│   ├── css/
│   ├── js/
│   └── assets/
└── package.json
```

### 1.2 Backend Architecture

Create a platform abstraction layer to handle OS-specific differences:

```rust
// keymagic-shared/gui/src-tauri/src/platform/mod.rs
pub trait PlatformBackend: Send + Sync {
    // Configuration storage
    fn load_config(&self) -> Result<Config>;
    fn save_config(&self, config: &Config) -> Result<()>;
    
    // Keyboard management
    fn get_keyboards_dir(&self) -> PathBuf;
    fn get_keyboard_files(&self) -> Result<Vec<PathBuf>>;
    
    // IME integration
    fn notify_ime_update(&self) -> Result<()>;
    fn is_ime_running(&self) -> bool;
    
    // System integration
    fn get_config_dir(&self) -> PathBuf;
    fn supports_language_profiles(&self) -> bool;
    fn supports_composition_mode(&self) -> bool;
    
    // Platform info
    fn get_platform_name(&self) -> &'static str;
    fn get_platform_specific_features(&self) -> PlatformFeatures;
}
```

### 1.3 Linux Backend Implementation

```rust
// keymagic-shared/gui/src-tauri/src/platform/linux.rs
pub struct LinuxBackend {
    config_dir: PathBuf,
    keyboards_dir: PathBuf,
}

impl LinuxBackend {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or("Failed to get config directory")?
            .join("keymagic");
        
        let keyboards_dir = PathBuf::from("/usr/share/keymagic/keyboards");
        
        Ok(Self { config_dir, keyboards_dir })
    }
}
```

### 1.4 Configuration Storage

Replace Windows Registry with file-based configuration:

```toml
# Linux: ~/.config/keymagic/config.toml
# macOS: ~/Library/Preferences/keymagic/config.toml

[general]
start_with_system = true
check_for_updates = true
last_update_check = "2024-01-20T10:00:00Z"

[keyboards]
active = "myanmar3"
last_used = ["myanmar3", "zawgyi"]

[[keyboards.installed]]
id = "myanmar3"
name = "Myanmar3"
filename = "myanmar3.km2"
hotkey = "ctrl+shift+m"
hash = "abc123..."

[[keyboards.installed]]
id = "zawgyi"
name = "Zawgyi"
filename = "zawgyi.km2"
hotkey = "ctrl+shift+z"
hash = "def456..."

[composition_mode]
enabled_processes = ["firefox", "chromium", "code"]
```

### 1.5 Global Hotkey Implementation

Use `global-hotkey` crate for cross-platform hotkey support:

```rust
// keymagic-shared/gui/src-tauri/src/hotkey/linux.rs
use global_hotkey::{GlobalHotKeyManager, HotKey};

pub struct LinuxHotkeyManager {
    manager: GlobalHotKeyManager,
    registered_hotkeys: HashMap<String, HotKey>,
}

impl HotkeyBackend for LinuxHotkeyManager {
    fn register_hotkey(&mut self, keyboard_id: &str, hotkey_str: &str) -> Result<()> {
        let hotkey = parse_hotkey(hotkey_str)?;
        self.manager.register(hotkey)?;
        self.registered_hotkeys.insert(keyboard_id.to_string(), hotkey);
        Ok(())
    }
}
```

### 1.6 D-Bus Integration for GUI-IBus Communication

```rust
// keymagic-shared/gui/src-tauri/src/platform/linux/dbus.rs
use zbus::{Connection, dbus_interface};

pub struct KeyMagicDBusService {
    keyboard_manager: Arc<Mutex<KeyboardManager>>,
}

#[dbus_interface(name = "org.keymagic.Service")]
impl KeyMagicDBusService {
    async fn switch_keyboard(&self, keyboard_id: &str) -> zbus::fdo::Result<()> {
        let manager = self.keyboard_manager.lock().unwrap();
        manager.set_active_keyboard(keyboard_id)?;
        
        // Notify IBus engine
        self.notify_ibus_engine(keyboard_id).await?;
        Ok(())
    }
    
    async fn get_active_keyboard(&self) -> zbus::fdo::Result<String> {
        let manager = self.keyboard_manager.lock().unwrap();
        Ok(manager.get_active_keyboard().unwrap_or_default())
    }
    
    #[dbus_interface(signal)]
    async fn keyboard_changed(&self, keyboard_id: &str) -> zbus::Result<()>;
}
```

### 1.7 Frontend Adaptations

Minimal changes to support cross-platform:

```javascript
// src/js/platform.js
export const Platform = {
    async getInfo() {
        return await invoke('get_platform_info');
    },
    
    async getKeyLabel(key) {
        const info = await this.getInfo();
        const labels = {
            'windows': { meta: 'Win', alt: 'Alt', enter: 'Enter' },
            'linux': { meta: 'Super', alt: 'Alt', enter: 'Return' },
            'macos': { meta: '⌘', alt: '⌥', enter: '⏎' }
        };
        return labels[info.os][key] || key;
    },
    
    async shouldShowFeature(feature) {
        const info = await this.getInfo();
        return info.features[feature] !== false;
    }
};

// Usage in settings.js
if (await Platform.shouldShowFeature('language_profiles')) {
    // Show Windows-specific language settings
}
```

### 1.8 Build Configuration

Update Cargo.toml for cross-platform support:

```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.48", features = ["Win32_Foundation", "Win32_UI_Input_KeyboardAndMouse"] }
winreg = "0.50"

[target.'cfg(target_os = "linux")'.dependencies]
global-hotkey = "0.2"
zbus = "3.14"
freedesktop-entry-parser = "1.3"

[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
objc = "0.2"
```

## Part 2: IBus Integration

### 2.1 IBus Engine Structure

```
keymagic-ibus/
├── src/
│   ├── lib.rs              # Rust coordination
│   ├── engine/
│   │   ├── keymagic-ibus.c     # Main IBus engine
│   │   ├── keymagic-ibus.h     # Engine header
│   │   ├── keymagic-bridge.c   # FFI bridge
│   │   └── keymagic-bridge.h   # Bridge header
│   └── build.rs            # Build configuration
├── data/
│   ├── keymagic.xml        # IBus component file
│   └── icons/              # Status icons
└── Makefile                # C compilation
```

### 2.2 IBus Component Registration

```xml
<!-- data/keymagic.xml -->
<?xml version="1.0" encoding="utf-8"?>
<component>
    <name>org.keymagic.IBus</name>
    <description>KeyMagic Input Method</description>
    <exec>/usr/lib/ibus-keymagic/ibus-engine-keymagic --ibus</exec>
    <version>2.0.0</version>
    <author>KeyMagic Team</author>
    <license>GPL-3.0</license>
    <homepage>https://keymagic.net</homepage>
    <textdomain>keymagic</textdomain>
    
    <engines>
        <engine>
            <name>keymagic</name>
            <language>my</language>
            <license>GPL-3.0</license>
            <author>KeyMagic Team</author>
            <icon>/usr/share/keymagic/icons/keymagic.png</icon>
            <layout>us</layout>
            <longname>KeyMagic</longname>
            <description>KeyMagic Input Method Engine</description>
            <rank>99</rank>
        </engine>
    </engines>
</component>
```

### 2.3 IBus Engine Implementation

```c
// engine/keymagic-ibus.c
typedef struct _IBusKeyMagicEngine IBusKeyMagicEngine;
typedef struct _IBusKeyMagicEngineClass IBusKeyMagicEngineClass;

struct _IBusKeyMagicEngine {
    IBusEngine parent;
    
    // KeyMagic engine instance
    keymagic_engine_t *km_engine;
    
    // Current keyboard
    gchar *current_keyboard_id;
    
    // Preedit handling
    IBusText *preedit_text;
    gboolean preedit_visible;
};

static gboolean
ibus_keymagic_engine_process_key_event(IBusEngine *engine,
                                       guint keyval,
                                       guint keycode,
                                       guint modifiers)
{
    IBusKeyMagicEngine *km_engine = (IBusKeyMagicEngine *)engine;
    
    // Convert IBus key event to KeyMagic format
    uint8_t km_keycode = ibus_to_keymagic_keycode(keycode);
    uint8_t km_modifiers = ibus_to_keymagic_modifiers(modifiers);
    
    // Process through KeyMagic engine
    keymagic_output_t output;
    int result = keymagic_engine_process_key(
        km_engine->km_engine,
        km_keycode,
        km_modifiers,
        &output
    );
    
    if (result && output.text_len > 0) {
        // Handle output
        handle_keymagic_output(engine, &output);
        keymagic_free_output(&output);
        return TRUE;
    }
    
    return FALSE;
}
```

### 2.4 D-Bus Communication with GUI

```c
// engine/dbus-client.c
static void
notify_gui_keyboard_changed(const gchar *keyboard_id)
{
    GDBusConnection *connection;
    GError *error = NULL;
    
    connection = g_bus_get_sync(G_BUS_TYPE_SESSION, NULL, &error);
    if (connection == NULL) {
        g_warning("Failed to get D-Bus connection: %s", error->message);
        g_error_free(error);
        return;
    }
    
    g_dbus_connection_emit_signal(
        connection,
        NULL, // destination
        "/org/keymagic/Service",
        "org.keymagic.Service",
        "KeyboardChanged",
        g_variant_new("(s)", keyboard_id),
        &error
    );
}
```

## Implementation Timeline

### Week 1-2: Cross-Platform GUI Setup
- [ ] Create new `keymagic-shared/gui` project structure
- [ ] Initialize fresh Tauri project with cross-platform in mind
- [ ] Design and implement platform abstraction traits
- [ ] Create Linux and Windows backend skeletons
- [ ] Set up build configuration for multi-platform

### Week 3-4: Linux Backend Implementation
- [ ] Implement file-based configuration
- [ ] Add XDG directory support
- [ ] Implement global hotkey support
- [ ] Create D-Bus service

### Week 5-6: Frontend Adaptations
- [ ] Add platform detection
- [ ] Hide Windows-specific features
- [ ] Adapt UI labels and icons
- [ ] Test on major Linux distros

### Week 7-8: IBus Engine Development
- [ ] Create IBus engine skeleton
- [ ] Implement FFI bridge to keymagic-core
- [ ] Add preedit text handling
- [ ] Implement keyboard switching

### Week 9-10: Integration and Testing
- [ ] Connect GUI with IBus via D-Bus
- [ ] Test keyboard switching flow
- [ ] Fix platform-specific bugs
- [ ] Performance optimization

### Week 11-12: Packaging and Documentation
- [ ] Create .deb package
- [ ] Create .rpm package
- [ ] Write installation guide
- [ ] Document API for developers

## Testing Strategy

### Unit Tests
- Platform abstraction layer
- Configuration management
- Hotkey parsing and handling

### Integration Tests
- GUI-IBus communication
- Keyboard switching
- Configuration persistence

### Manual Testing
- Test on Ubuntu 22.04, 24.04
- Test on Fedora 39
- Test on Arch Linux
- Test with various applications (Firefox, Chrome, VS Code, Terminal)

## Deliverables

1. **keymagic-shared/gui**: Cross-platform Tauri application
2. **keymagic-ibus**: IBus input method engine
3. **Installation packages**: .deb, .rpm, AUR
4. **Documentation**: User guide and developer documentation
5. **Test suite**: Automated tests for CI/CD

## Success Criteria

- [ ] GUI runs on Linux without Windows-specific errors
- [ ] IBus engine loads and processes keyboard input
- [ ] Keyboard switching works via hotkeys
- [ ] Configuration persists across restarts
- [ ] System tray integration works
- [ ] Preedit text displays correctly
- [ ] Works with major Linux applications