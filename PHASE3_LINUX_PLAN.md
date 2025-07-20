# Phase 3: Linux Implementation Plan

## Overview

Phase 3 involves two major components:
1. **Cross-Platform GUI**: Port the existing Windows Tauri GUI to work on Linux (and eventually macOS)
2. **IBus Integration**: Create a Linux input method engine using IBus framework

## Current Development Status

### Part 1: Cross-Platform GUI Port - ✅ **COMPLETED**

#### ✅ Completed Tasks (Latest Update: December 20, 2024):

1. **Project Structure Setup**
   - Created `keymagic-shared/gui` with proper directory structure
   - Initialized Tauri v2 project with cross-platform configuration
   - Added to workspace in root Cargo.toml

2. **Platform Abstraction Layer**
   - Implemented `Platform` trait with all required methods
   - Created full backends for Linux, Windows, and macOS
   - File-based configuration for Linux/macOS (TOML)
   - Registry-based configuration for Windows with full feature parity

3. **Core Components Ported**
   - `KeyboardManager` - Cross-platform keyboard management
   - `commands.rs` - All Tauri command handlers implemented
   - Frontend files (HTML, CSS, JS) fully ported from Windows GUI
   - Icon resources and capabilities configuration
   - Platform detection and conditional UI elements

4. **Dynamic System Tray**
   - **Implemented dynamic tray menu** with keyboard list
   - Active keyboard shown with checkmark (✓)
   - Keyboard switching directly from tray menu
   - `update_tray_menu` command for frontend integration
   - Tooltip shows "KeyMagic - {active keyboard name}"

5. **Windows Feature Parity**
   - Full Windows registry support (read/write)
   - Settings management (autostart, composition mode)
   - First-run detection and bundled keyboards
   - Composition mode process list management
   - Platform-specific UI adaptations

6. **Build Verification**
   - ✅ Builds successfully on Linux (Ubuntu 24.04 ARM64)
   - ✅ Builds successfully on macOS
   - ✅ Fixed Tauri API initialization issue (`withGlobalTauri`)
   - ✅ All commands properly connected to platform backends

#### 🚀 Ready for Testing:

1. **Cross-Platform GUI**
   - Run with `cargo tauri dev` in `keymagic-shared/gui`
   - All basic keyboard management features working
   - Platform-specific features properly abstracted
   - Windows features hidden on non-Windows platforms

#### ✅ Additional Features Completed (January 20, 2025):

1. **Global Hotkey Implementation**
   - ✅ Integrated Tauri v2 `global-shortcut` plugin for all platforms
   - ✅ Hotkey registration with keyboard switching
   - ✅ Default hotkey extraction from KM2 files
   - ✅ User hotkey preferences preserved (including explicit None)
   - ✅ Event-based hotkey handling with proper state synchronization

2. **State Synchronization**
   - ✅ Fixed keyboard activation state sync between UI components
   - ✅ Unified event system (`active_keyboard_changed`)
   - ✅ Consistent updates across keyboard list, tray menu, and hotkeys

3. **Dynamic Tray Icon**
   - ✅ Active keyboard icon displayed as system tray icon
   - ✅ Falls back to default icon when no keyboard has icon data
   - ✅ Icon updates automatically when switching keyboards
   - ✅ Supports BMP, PNG, and JPEG icon formats from KM2 files

#### ⏳ Remaining Tasks for Full Feature Parity:

1. **Keyboard Features**
   - Import wizard full implementation
   - Update checking with GitHub releases
   - Keyboard validation improvements

2. **Platform Integrations**
   - D-Bus communication for IBus (Linux)
   - Language profile management (Windows TSF)
   - Registry notification system (Windows)

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

### ✅ Week 5-6: Feature Parity (COMPLETED - December 20, 2024)
- [x] Dynamic tray menu with keyboard switching
- [x] Platform-specific feature flags
- [x] Windows registry support
- [x] Settings management
- [x] First-run detection
- [x] All Tauri commands implemented
- [x] Frontend platform detection
- [x] Global hotkey support (COMPLETED - July 20, 2025)
- [x] Keyboard metadata extraction (COMPLETED - July 20, 2025)

### ⏳ Week 7-8: Missing Features Implementation
- [ ] Implement HUD/notification system
- [ ] Add file association support (.km2 files)
- [ ] Implement automatic update checking
- [ ] Add keyboard validation command
- [ ] Create bundled keyboard import

### ⏳ Week 9-10: IBus Engine Development
- [ ] Create IBus engine skeleton
- [ ] Implement FFI bridge to keymagic-core
- [ ] Add preedit text handling
- [ ] Implement keyboard switching
- [ ] D-Bus integration for GUI communication

### ⏳ Week 11-12: Integration and Testing
- [ ] Connect GUI with IBus via D-Bus
- [ ] Implement language profile management
- [ ] Test keyboard switching flow
- [ ] Fix platform-specific bugs
- [ ] Performance optimization

### ⏳ Week 13-14: Packaging and Documentation
- [ ] Create .deb package with file associations
- [ ] Create .rpm package with file associations
- [ ] Set up automatic update infrastructure
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

## Missing Features from Windows GUI

After comparing the new cross-platform GUI with the original Windows implementation, the following features need to be implemented for feature parity:

### Critical Features

1. **HUD (Heads-Up Display) Notifications**
   - Visual feedback when switching keyboards
   - Notification when minimizing to tray
   - Platform-specific implementation needed (Linux: libnotify, macOS: NSUserNotification)

2. **File Association Support**
   - Handle .km2 files opened via double-click
   - Command-line file argument parsing
   - Drag-and-drop file import to main window

3. **Language Profile Management**
   - Windows: TSF integration for system language profiles
   - Linux: IBus language profile management
   - macOS: Input source management

4. **Update System**
   - Automatic update checking on startup (5-second delay)
   - Platform-specific update manifests
   - Update signature verification
   - Silent background checking

### Important Features

5. **Advanced Keyboard Management**
   - `validate_keyboards` command to check/remove invalid keyboards
   - Keyboard icon extraction from embedded resources
   - Key processing enable/disable at runtime

6. **System Integration**
   - Windows: Registry notification system for TSF instances
   - Linux: D-Bus signals for IBus instances
   - Inter-process communication for IME synchronization

7. **Import/Export Enhancements**
   - `import_bundled_keyboard` command
   - Bundled keyboard status tracking (New/Updated/Modified)
   - SHA-256 hash comparison for updates

8. **First-Run Experience**
   - First minimize notification
   - Guided setup for new users
   - Platform-specific onboarding

### Platform-Specific Features

9. **Windows-Specific**
   - Low-level keyboard hook (currently using Tauri plugin)
   - Registry-based settings (currently using TOML/registry hybrid)
   - Elevated permissions for language profile updates

10. **Linux-Specific**
   - IBus engine integration
   - Desktop notification integration
   - XDG autostart support

## Next Steps

1. ✅ **Implement Global Hotkeys** - Critical for keyboard switching (COMPLETED)
2. ✅ **Keyboard Metadata Support** - Icons, descriptions from KM2 files (COMPLETED)
3. **Implement HUD/Notifications** - Critical for user feedback
4. **Add File Association Support** - Important for user workflow
5. **Begin IBus Engine Development** - Start with basic engine skeleton
6. **Implement Update System** - Automatic checking and platform manifests
7. **Create Installation Packages** - .deb and .rpm for distribution
8. **Add Missing Keyboard Management Features** - Validation, icon extraction
9. **Implement Language Profile Management** - Platform-specific integration

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