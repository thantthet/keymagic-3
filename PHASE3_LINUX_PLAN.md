# Phase 3: Linux Implementation Plan

## Overview

Phase 3 involves two major components:
1. **Cross-Platform GUI**: Port the existing Windows Tauri GUI to work on Linux (and eventually macOS)
2. **IBus Integration**: Create a Linux input method engine using IBus framework

## Current Development Status

### Part 1: Cross-Platform GUI Port - ✅ **COMPLETED**

#### ✅ Completed Tasks:

1. **Project Structure Setup**
   - Created `keymagic-shared/gui` with proper directory structure
   - Initialized Tauri v2 project with cross-platform configuration
   - Added to workspace in root Cargo.toml

2. **Platform Abstraction Layer**
   - Implemented `Platform` trait with all required methods
   - Created backends for Linux, Windows, and macOS (stub)
   - File-based configuration for Linux/macOS (TOML)
   - Registry-based configuration for Windows

3. **Core Components Ported**
   - `KeyboardManager` - Cross-platform keyboard management
   - `commands.rs` - Tauri command handlers
   - Frontend files (HTML, CSS, JS) copied from Windows GUI
   - Icon resources and capabilities configuration

4. **Dynamic System Tray**
   - **Implemented dynamic tray menu** with keyboard list
   - Active keyboard shown with checkmark (✓)
   - Keyboard switching directly from tray menu
   - `update_tray_menu` command for frontend integration
   - Tooltip shows "KeyMagic - {active keyboard name}"

5. **Build Verification**
   - ✅ Builds successfully on Linux (Ubuntu 24.04 ARM64)
   - ✅ Builds successfully on macOS
   - ✅ Fixed duplicate tray icon issue on macOS

#### 🔄 In Progress:

1. **Feature Parity with Windows GUI**
   - Some commands still need to be ported as needed
   - Keyboard metadata support (icons, descriptions)
   - Hotkey normalization logic

#### ⏳ Pending Tasks:

1. **Global Hotkey Implementation**
   - Linux: Use `global-hotkey` crate
   - Platform-specific hotkey registration

2. **D-Bus Integration**
   - Communication between GUI and IBus engine
   - Currently using placeholder implementation

3. **Additional Features**
   - Keyboard validation and cleanup
   - Bundled keyboard support
   - Update checking functionality

### Part 2: IBus Integration - ⏳ **NOT STARTED**

The IBus engine implementation has not been started yet. This will be the next major phase after completing the GUI feature parity.

## Updated Implementation Timeline

### ✅ Week 1-2: Cross-Platform GUI Setup (COMPLETED)
- [x] Create new `keymagic-shared/gui` project structure
- [x] Initialize fresh Tauri project with cross-platform in mind
- [x] Design and implement platform abstraction traits
- [x] Create Linux and Windows backend skeletons
- [x] Set up build configuration for multi-platform

### ✅ Week 3-4: Core Implementation (COMPLETED)
- [x] Implement file-based configuration
- [x] Add XDG directory support
- [x] Port frontend files from Windows GUI
- [x] Implement dynamic system tray
- [x] Test builds on Linux and macOS

### 🔄 Week 5-6: Feature Parity (IN PROGRESS)
- [x] Dynamic tray menu with keyboard switching
- [ ] Global hotkey support
- [ ] Keyboard metadata extraction
- [ ] Platform-specific feature flags

### ⏳ Week 7-8: IBus Engine Development
- [ ] Create IBus engine skeleton
- [ ] Implement FFI bridge to keymagic-core
- [ ] Add preedit text handling
- [ ] Implement keyboard switching

### ⏳ Week 9-10: Integration and Testing
- [ ] Connect GUI with IBus via D-Bus
- [ ] Test keyboard switching flow
- [ ] Fix platform-specific bugs
- [ ] Performance optimization

### ⏳ Week 11-12: Packaging and Documentation
- [ ] Create .deb package
- [ ] Create .rpm package
- [ ] Write installation guide
- [ ] Document API for developers

## Technical Details of Completed Work

### Platform Abstraction
```rust
// Implemented in keymagic-shared/gui/src-tauri/src/platform/mod.rs
pub trait Platform: Send + Sync {
    fn load_config(&self) -> Result<Config>;
    fn save_config(&self, config: &Config) -> Result<()>;
    fn get_keyboards_dir(&self) -> PathBuf;
    fn get_keyboard_files(&self) -> Result<Vec<PathBuf>>;
    fn notify_ime_update(&self, keyboard_id: &str) -> Result<()>;
    fn is_ime_running(&self) -> bool;
    fn switch_keyboard(&self, keyboard_id: &str) -> Result<()>;
    // ... more methods
}
```

### Dynamic Tray Implementation
```rust
// Implemented in keymagic-shared/gui/src-tauri/src/tray.rs
pub fn build_tray_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    // Builds menu with keyboard list
    // Active keyboard marked with ✓
    // Handles keyboard switching on click
}
```

### Configuration Storage (Linux/macOS)
```toml
# Stored in ~/.config/keymagic/config.toml (Linux)
# ~/Library/Preferences/keymagic/config.toml (macOS)
[general]
start_with_system = false
check_for_updates = true

[keyboards]
active = "myanmar3"
installed = [
    { id = "myanmar3", name = "Myanmar3", filename = "myanmar3.km2", ... }
]
```

## Next Steps

1. **Implement Global Hotkeys** - Critical for keyboard switching
2. **Add Keyboard Metadata Support** - Icons, descriptions from KM2 files
3. **Begin IBus Engine Development** - Start with basic engine skeleton
4. **Create Installation Packages** - .deb and .rpm for distribution

## Success Criteria Progress

- [x] GUI runs on Linux without Windows-specific errors
- [ ] IBus engine loads and processes keyboard input
- [ ] Keyboard switching works via hotkeys
- [x] Configuration persists across restarts
- [x] System tray integration works
- [ ] Preedit text displays correctly
- [ ] Works with major Linux applications

## Notes

- Using gradual migration approach - Windows GUI remains unchanged
- Cross-platform GUI provides good foundation but needs feature additions
- Platform differences handled through abstraction layer
- Frontend code is fully shared across platforms