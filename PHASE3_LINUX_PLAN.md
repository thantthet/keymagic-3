# Phase 3: Linux Implementation Plan

## Overview

Phase 3 involves two major components:
1. **Cross-Platform GUI**: Port the existing Windows Tauri GUI to work on Linux (and eventually macOS)
2. **IBus Integration**: Create a Linux input method engine using IBus framework

## Current Development Status

### Part 1: Cross-Platform GUI Port - ✅ **COMPLETED**

#### ✅ Completed Tasks (Latest Update: January 21, 2025):

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

#### ✅ Additional Features Completed (January 20-21, 2025):

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

4. **Version Checking and Update System**
   - ✅ Implemented automatic update checking on startup (5-second delay)
   - ✅ GitHub releases integration with platform-specific manifests
   - ✅ Remind me later functionality (24-hour delay)
   - ✅ Dynamic version display from Cargo.toml
   - ✅ Cross-platform update notification UI

5. **System Integration Features**
   - ✅ Start with system using Tauri autostart plugin
   - ✅ Works across Windows (registry), macOS (Launch Agent), Linux (XDG autostart)
   - ✅ HUD notification when keyboard switched (already implemented)
   - ✅ Windows TSF notification for keyboard changes
   - ✅ First minimize notification to tray

6. **File and Package Management**
   - ✅ File association support for .km2 files (Windows)
   - ✅ Bundled keyboard support for Linux and macOS packages
   - ✅ Import wizard implementation (fully functional)
     - ✅ Complete UI with keyboard selection
     - ✅ Shows keyboard status (New/Updated/Unchanged/Modified)
     - ✅ Smart selection defaults (auto-selects New and Updated)
     - ✅ Platform-specific bundled keyboard locations
     - ✅ Version-based scanning detection for all platforms
     - ✅ Automatically shows on new install or app update

### Part 2: IBus Integration - ✅ **COMPLETED**

The IBus engine has been fully implemented and is production-ready.

#### ✅ Completed IBus Features (January 21, 2025):

1. **Core IBus Engine**
   - ✅ Complete C implementation in `keymagic-ibus/`
   - ✅ FFI bridge to keymagic-core Rust library
   - ✅ Single engine handles all keyboard layouts
   - ✅ File-based configuration monitoring
   - ✅ On-demand keyboard loading

2. **Key Processing**
   - ✅ Proper keycode mapping for Linux
   - ✅ Modifier key support
   - ✅ Preedit text handling following TSF pattern
   - ✅ Error recovery mechanisms (silent failure)

3. **Integration Features**
   - ✅ File-based communication with GUI (no D-Bus needed)
   - ✅ GFileMonitor for config.toml changes
   - ✅ Keyboard hot-switching without restart
   - ✅ Language profile management

4. **Packaging and Distribution**
   - ✅ Debian package (.deb) creation
   - ✅ RPM package (.rpm) creation
   - ✅ Installation scripts and desktop integration
   - ✅ IBus component XML definition
   - ✅ Built packages in `dist/` directory

5. **Build Infrastructure**
   - ✅ Docker-based multi-architecture build system
   - ✅ Makefile for native builds
   - ✅ Static linking with keymagic-core
   - ✅ Debug and release configurations

#### ⏳ Remaining Minor Tasks:

1. **Documentation**
   - Keyboard validation command improvements
   - Update signature verification
   - Installation guide documentation
   - API documentation for developers

2. **Platform Integration**
   - macOS IMK integration (future phase)

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

### ✅ Week 7-8: Missing Features Implementation (COMPLETED - January 20, 2025)
- [x] Implement HUD/notification system (was already done)
- [x] Add file association support (.km2 files)
- [x] Implement automatic update checking
- [ ] Add keyboard validation command
- [x] Create bundled keyboard import

### ✅ Week 9-10: IBus Engine Development (COMPLETED - January 21, 2025)
- [x] Create IBus engine skeleton
- [x] Implement FFI bridge to keymagic-core
- [x] Add preedit text handling
- [x] Implement keyboard switching
- [x] File monitoring for GUI communication (GFileMonitor)

### ✅ Week 11-12: Integration and Testing (COMPLETED - January 21, 2025)
- [x] Connect GUI with IBus via file monitoring (config.toml)
- [x] Implement language profile management
- [x] Test keyboard switching flow
- [x] Fix platform-specific bugs
- [x] Performance optimization

### ✅ Week 13-14: Packaging and Documentation (MOSTLY COMPLETED - January 21, 2025)
- [x] Create .deb package with file associations
- [x] Create .rpm package with file associations
- [x] Set up automatic update infrastructure
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
last_scanned_version = "0.0.1"  # Version-based bundled keyboard scanning

[keyboards]
active = "myanmar3"
installed = [
    { id = "myanmar3", name = "Myanmar3", filename = "myanmar3.km2", ... }
]
```

### Version-Based Bundled Keyboard Scanning

The import wizard now uses version-based detection instead of simple first-run flags:
- Compares app version with `last_scanned_version` in config
- Shows import wizard when: current version > last scanned version
- Updates `last_scanned_version` after scanning completes
- Ensures users see new bundled keyboards after app updates

## Technical Details

### IBus Engine Architecture

The IBus engine (`keymagic-ibus/`) follows a clean architecture:

```
keymagic-ibus/
├── src/
│   ├── engine.c         # Main IBus engine implementation
│   ├── ffi_bridge.c     # FFI bridge to Rust keymagic-core
│   ├── config.c         # Configuration file handling
│   ├── keycode_map.c    # Linux keycode mapping
│   └── toml.c          # TOML parser for config files
├── build-pkg.sh        # Package building script
├── Makefile           # Build configuration
└── dist/              # Built packages
```

### Configuration File Format

The IBus engine monitors `~/.config/keymagic/config.toml`:

```toml
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

After comparing the new cross-platform GUI with the original Windows implementation, most features have been implemented:

### ✅ Completed Features

1. **HUD (Heads-Up Display) Notifications** ✅ **COMPLETED**
   - Visual feedback when switching keyboards
   - Notification when minimizing to tray
   - Platform-specific implementation (Windows: native HUD, others: Tauri window)

2. **File Association Support** ✅ **COMPLETED**
   - Handle .km2 files opened via double-click
   - Command-line file argument parsing
   - Bundled keyboard support for packages

3. **Language Profile Management** ✅ **COMPLETED**
   - Windows: TSF integration for system language profiles
   - Linux: IBus language profile management via config file monitoring
   - Configuration-based profile switching

4. **Update System** ✅ **COMPLETED**
   - Automatic update checking on startup (5-second delay)
   - Platform-specific update manifests
   - Silent background checking
   - Remind me later functionality

5. **Import Wizard and Version-Based Scanning** ✅ **COMPLETED**
   - ✅ Version-based scanning detection for all platforms
   - ✅ Scans on new install or app update
   - ✅ Consistent behavior across Windows, Linux, and macOS
   - ✅ Renamed methods for clarity:
     - `is_first_run()` → `should_scan_bundled_keyboards()`
     - `clear_first_run_flag()` → `mark_bundled_keyboards_scanned()`
   - ✅ Updated frontend commands to match new naming

### ⏳ Remaining Features

6. **Advanced Keyboard Management**
   - [ ] `validate_keyboards` command to check/remove invalid keyboards
   - [ ] Update signature verification

7. **Documentation**
   - [ ] Installation guide for end users
   - [ ] API documentation for developers
   - [ ] Contribution guidelines

## Next Steps

With Phase 3 largely complete, the remaining tasks are:

1. **Keyboard Validation** - Implement the `validate_keyboards` command
2. **Documentation** - Write user and developer documentation
3. **Update Signatures** - Add cryptographic verification for updates
4. **macOS IMK Integration** - Future phase for macOS native input method

## Success Criteria Progress

- [x] GUI runs on Linux without Windows-specific errors
- [x] IBus engine loads and processes keyboard input
- [x] Keyboard switching works via hotkeys
- [x] Configuration persists across restarts
- [x] System tray integration works
- [x] Preedit text displays correctly
- [x] Works with major Linux applications
- [x] Debian and RPM packages available
- [x] File associations work correctly
- [x] Update system functional

## Phase 3 Status: 95% COMPLETE

The Linux implementation is essentially complete and production-ready. Both the cross-platform GUI and IBus engine are fully functional with all major features implemented, including version-based bundled keyboard scanning.

## Notes

- Using gradual migration approach - Windows GUI remains unchanged
- Cross-platform GUI provides good foundation but needs feature additions
- Platform differences handled through abstraction layer
- Frontend code is fully shared across platforms