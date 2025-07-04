# KeyMagic GUI Implementation Guide

## Overview
The KeyMagic Configuration Manager is a native Windows application built with Rust and the windows-rs crate. It provides a graphical interface for managing KeyMagic keyboard layouts.

## Architecture

### Component Structure
```
keymagic-config
├── main.rs              # Entry point and message loop
├── app.rs               # Application state management
├── window.rs            # Main window implementation
├── keyboard_list.rs     # ListView control for keyboards
└── keyboard_manager_simple.rs  # Keyboard management logic
```

### Key Design Decisions

1. **Native Win32**: Direct Win32 API usage for maximum compatibility
2. **Rust Safety**: Memory safety with Arc/Mutex for shared state
3. **Simple Architecture**: MVP approach for initial implementation

## Implementation Details

### 1. Main Window (window.rs)

#### Window Creation
```rust
pub struct MainWindow {
    hwnd: HWND,
    app: Arc<App>,
    keyboard_manager: Arc<Mutex<KeyboardManager>>,
    list_view: RefCell<Option<KeyboardListView>>,
}
```

#### Menu Structure
- **File Menu**
  - Add Keyboard... (Ctrl+O)
  - Remove Keyboard (Del)
  - Exit (Alt+F4)
- **Keyboard Menu**
  - Activate (Enter)
  - Configure... (Future)
- **Help Menu**
  - About... (F1)

#### Window Procedure
Handles all window messages:
- `WM_CREATE`: Initialize ListView
- `WM_COMMAND`: Process menu commands
- `WM_NOTIFY`: Handle ListView notifications
- `WM_SIZE`: Resize ListView
- `WM_DESTROY`: Cleanup

### 2. Keyboard ListView (keyboard_list.rs)

#### ListView Configuration
```rust
// Extended styles for better appearance
LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES | LVS_EX_DOUBLEBUFFER
```

#### Columns
1. **Name**: Keyboard display name
2. **Description**: Keyboard description
3. **Hotkey**: Activation hotkey (future)
4. **Status**: Active/Enabled/Disabled

#### Interactions
- **Double-click**: Activate keyboard
- **Selection**: Enable remove/activate options
- **Refresh**: Update after changes

### 3. Keyboard Manager (keyboard_manager_simple.rs)

#### Core Functionality
```rust
pub struct KeyboardManager {
    keyboards: HashMap<String, KeyboardInfo>,
    active_keyboard: Option<String>,
}
```

#### Operations
1. **Load Keyboard**
   - Validate .km2 file using keymagic-core
   - Extract metadata (simplified for now)
   - Add to collection

2. **Remove Keyboard**
   - Remove from collection
   - Update active if needed

3. **Set Active**
   - Track active keyboard
   - Future: Notify TSF component

### 4. File Dialog Integration

#### Open File Dialog
```rust
let mut ofn = std::mem::zeroed::<OPENFILENAMEW>();
ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
ofn.hwndOwner = self.hwnd;
ofn.lpstrFilter = PCWSTR(filter_w.as_ptr());
ofn.lpstrFile = PWSTR(file_path_buffer.as_mut_ptr());
ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY;
```

## State Management

### Thread Safety
- `Arc<Mutex<KeyboardManager>>` for shared state
- `RefCell` for ListView (single-threaded UI)

### Data Flow
1. User action → Window procedure
2. Window procedure → Keyboard manager
3. Keyboard manager → Update state
4. State change → ListView refresh

## Error Handling

### User-Friendly Messages
```rust
MessageBoxW(
    self.hwnd,
    PCWSTR(msg_w.as_ptr()),
    w!("Error"),
    MB_OK | MB_ICONERROR,
);
```

### Error Categories
1. **File Errors**: Invalid/missing keyboard files
2. **Load Errors**: Corrupted keyboard data
3. **System Errors**: Registry, memory, etc.

## Building and Deployment

### Build Requirements
- Rust toolchain (MSRV: 1.70)
- Windows SDK
- Visual Studio build tools

### Build Command
```bash
cargo build --release
```

### Output
- Executable: `target/release/keymagic-config.exe`
- No additional runtime dependencies

## Future Enhancements

### Phase 5.3 Features
1. **Registry Persistence**
   - Save keyboard list
   - Remember window position
   - Store user preferences

2. **System Tray**
   - Minimize to tray
   - Quick keyboard switching
   - Status notifications

3. **Hotkey Support**
   - Global hotkey registration
   - Per-keyboard hotkeys
   - Conflict detection

### UI Improvements
1. **Keyboard Preview**
   - Visual layout display
   - Character mappings
   - Test input area

2. **Settings Dialog**
   - Global preferences
   - TSF options
   - Auto-start configuration

3. **Keyboard Import**
   - Batch import
   - Download from repository
   - Export/backup

## Testing Guidelines

### Manual Testing
1. **Add Keyboard**
   - Valid .km2 file
   - Invalid file
   - Cancel operation

2. **Remove Keyboard**
   - With confirmation
   - Active keyboard
   - Last keyboard

3. **Activate Keyboard**
   - Double-click
   - Menu option
   - No selection

### Automated Testing
- Unit tests for keyboard manager
- Integration tests with mock FFI
- UI automation (future)

## Known Limitations

### Current Implementation
1. **No Registry**: Using in-memory storage
2. **No Icons**: Keyboard icons not displayed
3. **No Hotkeys**: Not yet implemented
4. **English Only**: No localization

### Platform Limitations
- Windows 10/11 only
- 64-bit only (currently)
- Requires TSF support

## Troubleshooting

### Common Issues

1. **"Failed to load keyboard"**
   - Check file is valid .km2
   - Ensure file is accessible
   - Verify keymagic-core compatibility

2. **ListView not updating**
   - Check refresh called
   - Verify state changes
   - Look for exceptions

3. **Window not appearing**
   - Check window class registration
   - Verify message loop running
   - Look for initialization errors

### Debug Mode
Set environment variable for verbose logging:
```
set KEYMAGIC_DEBUG=1
```

## Code Examples

### Adding a Menu Item
```rust
AppendMenuW(
    menu, 
    MF_STRING, 
    ID_NEW_ITEM as usize, 
    w!("&New Item\tCtrl+N")
)?;
```

### Handling ListView Selection
```rust
let selected = SendMessageW(
    self.hwnd, 
    LVM_GETNEXTITEM, 
    WPARAM(-1i32 as usize), 
    LPARAM(LVNI_SELECTED as isize)
);
```

### Showing Error Dialog
```rust
let msg = format!("Operation failed: {}", error);
let msg_w: Vec<u16> = msg.encode_utf16()
    .chain(std::iter::once(0))
    .collect();
MessageBoxW(hwnd, PCWSTR(msg_w.as_ptr()), w!("Error"), MB_OK);
```

This implementation provides a solid foundation for the KeyMagic configuration interface, with room for growth and enhancement in future phases.