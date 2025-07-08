# KeyMagic Tauri GUI

A modern configuration manager for KeyMagic built with Tauri 2.0.

## Features

- **Modern UI**: Clean, responsive interface built with web technologies
- **Keyboard Management**: Add, remove, and activate keyboard layouts
- **Settings**: Configure startup options and system integration
- **Native Performance**: Rust backend with web frontend
- **Windows Integration**: Registry persistence and native file dialogs

## Architecture

- **Frontend**: HTML, CSS, and vanilla JavaScript
- **Backend**: Rust with Tauri framework
- **Data Persistence**: Windows Registry
- **Keyboard Engine**: keymagic-core FFI

## Development

### Prerequisites

- Rust toolchain
- Node.js and npm
- Tauri CLI: `cargo install tauri-cli --version '^2.0.0'`

### Building

```bash
# Development build with hot reload
cargo tauri dev

# Production build
cargo tauri build
```

### Project Structure

```
gui-tauri/
├── src/                    # Frontend (HTML/CSS/JS)
│   ├── index.html         # Main HTML
│   ├── styles.css         # Styling
│   └── main.js            # JavaScript logic
└── src-tauri/             # Backend (Rust)
    ├── src/
    │   ├── lib.rs         # Tauri app setup
    │   ├── commands.rs    # Tauri commands
    │   └── keyboard_manager.rs
    └── tauri.conf.json    # Tauri configuration
```

## UI Components

### Sidebar Navigation
- Keyboards page
- Settings page
- About page
- KeyMagic enable/disable toggle

### Keyboard Management
- Card-based keyboard list
- Visual status indicators
- Hotkey display
- Add/Remove operations

### Settings
- Start with Windows
- System tray options
- Hotkey configuration

## Technology Stack

- **Tauri 2.0**: App framework
- **Rust**: Backend logic
- **Web Technologies**: Frontend UI
- **windows-rs**: Windows API integration
- **keymagic-core**: Keyboard engine