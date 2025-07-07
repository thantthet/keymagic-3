# KeyMagic GUI - Slint Implementation

A modern, cross-platform GUI for KeyMagic Configuration Manager built with Slint.

## Overview

This is a rewrite of the original Win32 GUI using the Slint UI framework. It provides a modern, clean interface while maintaining all the functionality of the original application.

## Features

- **Modern UI Design**: Clean sidebar navigation with intuitive layout
- **Keyboard Management**: Add, remove, and activate keyboards
- **Real-time Preview**: Test keyboards with live feedback
- **Settings**: Configure global hotkeys and preferences
- **System Integration**: Windows tray support and global hotkeys

## Building

### Prerequisites

- Rust 1.70+ with MSVC toolchain
- Windows SDK
- CMake (for Slint)

### Build Commands

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run
cargo run
```

## Architecture

- **UI Layer**: Slint components for declarative UI
- **Application Logic**: Rust code handling business logic
- **Platform Integration**: Windows-specific features via windows-rs
- **Keyboard Engine**: Integration with keymagic-core FFI

## Project Structure

```
gui-slint/
├── src/
│   ├── main.rs          # Entry point
│   ├── app.rs           # Application state and logic
│   ├── keyboard_manager.rs # Keyboard management
│   ├── models.rs        # Data models for Slint
│   └── ...              # Other modules
├── ui/
│   ├── main_window.slint # Main window definition
│   ├── components/       # Reusable UI components
│   └── theme/           # Styling and theming
└── build.rs             # Build script
```

## Key Improvements

1. **Modern Design**: Sidebar navigation with clear visual hierarchy
2. **Better UX**: Card-based layouts, smooth animations, visual feedback
3. **Maintainability**: Declarative UI with clear separation of concerns
4. **Theming**: Centralized theme system for consistent styling
5. **Responsive**: Flexible layouts that adapt to window size

## Status

Currently in active development. See [SLINT-GUI-REWRITE-PLAN.md](../SLINT-GUI-REWRITE-PLAN.md) for detailed progress tracking.

## License

Same as KeyMagic project.