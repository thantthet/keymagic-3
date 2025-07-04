# KeyMagic Windows TSF Implementation Plan - Phase 5

## Executive Summary

This document provides a comprehensive plan for implementing a Windows Text Services Framework (TSF) based IME for KeyMagic, along with a native Win32 GUI configuration manager. The implementation will use the existing keymagic-core FFI interface.

### Key Implementation Approach

The TSF implementation uses a **simplified text handling strategy**:
- **Always display the engine's composing text** as the composition string (with underline)
- **Ignore action types** returned by the engine (no need to handle Insert/Delete/BackspaceDeleteAndInsert)
- **Commit text triggered by**:
  - Space key (when composing text ends with space OR engine doesn't process the space)
  - Enter/Tab keys (commit current composing text)
  - Focus loss (automatically commit pending text)
  - Escape key (cancel composition)
- **Reset engine** after each commit to clear the composing buffer

This approach significantly simplifies the TSF implementation while maintaining full compatibility with KeyMagic's rule-based engine.

## Architecture Overview

### System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Windows System                        │
├─────────────────────────────────────────────────────────┤
│                Text Services Framework (TSF)             │
├─────────────────────────────────────────────────────────┤
│              KeyMagic TSF Text Service (C++)            │
│  ┌─────────────────────────────────────────────────┐   │
│  │ • ITfTextInputProcessor                          │   │
│  │ • ITfThreadMgrEventSink                         │   │
│  │ • ITfKeyEventSink                               │   │
│  │ • ITfCompositionSink                            │   │
│  │ • ITfDisplayAttributeProvider                   │   │
│  └─────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│            keymagic-core FFI (C ABI)                    │
│  • keymagic_engine_new/free                             │
│  • keymagic_engine_load_keyboard                        │
│  • keymagic_engine_process_key                          │
│  • keymagic_engine_get_composition                      │
│  • keymagic_engine_reset                                │
├─────────────────────────────────────────────────────────┤
│                 keymagic-core (Rust)                     │
│         (Keyboard parsing, rule matching, etc.)          │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│    KeyMagic Configuration Manager (Native Win32 GUI)    │
│  ┌─────────────────────────────────────────────────┐   │
│  │ • Keyboard list and management                   │   │
│  │ • Add/Remove keyboards (.km2 files)              │   │
│  │ • Hotkey configuration                           │   │
│  │ • System tray integration                        │   │
│  │ • Direct use of keymagic-core FFI                │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **Native Windows GUI**: Using windows-rs for native Win32 GUI instead of cross-platform frameworks
2. **Direct FFI Usage**: Both TSF and GUI components will use the existing keymagic-core FFI
3. **Persistent Composing Text**: The engine maintains composing text across all key events
4. **Thread Safety**: TSF requires thread-safe implementation as calls can come from multiple threads

## Directory Structure

```
keymagic-windows/
├── Cargo.toml                    # Workspace for Rust components
├── README.md                     
├── tsf/                          # TSF Text Service (C++)
│   ├── CMakeLists.txt           
│   ├── include/
│   │   └── keymagic_ffi.h       # C header for keymagic-core FFI
│   ├── src/
│   │   ├── KeyMagicTextService.cpp  # Main TSF implementation
│   │   ├── KeyMagicTextService.h
│   │   ├── KeyEventSink.cpp     
│   │   ├── KeyEventSink.h
│   │   ├── Composition.cpp      
│   │   ├── Composition.h
│   │   ├── DisplayAttribute.cpp  
│   │   ├── DisplayAttribute.h
│   │   ├── ClassFactory.cpp     # COM class factory
│   │   ├── ClassFactory.h
│   │   ├── DllMain.cpp          # DLL entry points
│   │   ├── Registry.cpp         
│   │   ├── Registry.h
│   │   ├── Globals.cpp          
│   │   ├── Globals.h
│   │   └── KeyMagicTSF.def      
│   └── build.rs                 # Build script for C++ compilation
├── gui/                         # Configuration Manager
│   ├── Cargo.toml              
│   ├── build.rs                # Windows resources compilation
│   ├── src/
│   │   ├── main.rs             
│   │   ├── app.rs              # Application state management
│   │   ├── window.rs           # Main window using windows-rs
│   │   ├── keyboard_list.rs    # Keyboard ListView control
│   │   ├── settings.rs         # Settings dialog
│   │   ├── tray.rs            # System tray functionality
│   │   ├── registry.rs        # Windows registry operations
│   │   ├── keyboard_manager.rs # Keyboard management using FFI
│   │   └── ffi.rs             # Import keymagic-core FFI
│   └── resources/
│       ├── app.manifest       # Windows application manifest
│       ├── app.rc            # Resource file
│       └── icons/            # Application icons
└── installer/
    ├── setup.iss             # Inno Setup script
    └── scripts/              # Installation helper scripts
```

## Technical Implementation Details

### TSF Text Processing Flow

```
User Key Press
    ↓
ITfKeyEventSink::OnKeyDown
    ↓
keymagic_engine_process_key (FFI)
    ↓
ProcessKeyOutput {
    action_type: (ignored by TSF)
    text: (ignored by TSF)
    delete_count: (ignored by TSF)
    composing_text: UTF-8 string (ALWAYS present, this is what we display)
    is_processed: bool (determines if key is consumed)
}
    ↓
Decision Logic:
- If SPACE key AND (composing ends with space OR is_processed=false):
  → Commit text (with space if needed)
  → Reset engine
- Else:
  → Update composition display with composing_text
```

### Critical Engine Behavior

1. **Composing Text Persistence**:
   - The engine ALWAYS maintains and returns composing text
   - Composing text accumulates across ALL key events
   - Example: Type "k" → composing="k", then "a" → composing="က"
   - Only cleared via explicit reset() or set_composing_text()

2. **Simplified TSF Text Handling**:
   - **Always show composing text**: Display whatever the engine returns as composition
   - **Ignore action types**: Don't process Insert/Delete/Replace actions from engine
   - **Space key triggers commit**: 
     - If composing text ends with space → commit the text (including space)
     - If space key pressed but engine doesn't process it (is_processed=false) → commit current composing text + append space
   - **Other commit triggers**: Focus loss, Enter key, explicit user action

3. **State Management**:
   - Engine states are transient (per-key event)
   - Composing buffer is persistent
   - TSF must call reset() on focus loss and after committing text

### FFI Integration

#### C++ Header (keymagic_ffi.h)
```c
typedef struct EngineHandle EngineHandle;

typedef enum {
    KeyMagicResult_Success = 0,
    KeyMagicResult_ErrorInvalidHandle = -1,
    KeyMagicResult_ErrorInvalidParameter = -2,
    KeyMagicResult_ErrorEngineFailure = -3,
    KeyMagicResult_ErrorUtf8Conversion = -4,
    KeyMagicResult_ErrorNoKeyboard = -5,
} KeyMagicResult;

typedef struct {
    int action_type;      // 0=None, 1=Insert, 2=Delete, 3=DeleteAndInsert
    char* text;           // UTF-8, null-terminated
    int delete_count;     
    char* composing_text; // UTF-8, null-terminated, ALWAYS present
    int is_processed;     // 0=false, 1=true
} ProcessKeyOutput;

// Core functions
EngineHandle* keymagic_engine_new(void);
void keymagic_engine_free(EngineHandle* handle);
KeyMagicResult keymagic_engine_load_keyboard(EngineHandle* handle, const char* km2_path);
KeyMagicResult keymagic_engine_load_keyboard_from_memory(
    EngineHandle* handle, const uint8_t* km2_data, size_t data_len);
KeyMagicResult keymagic_engine_process_key(
    EngineHandle* handle,
    int key_code,
    char character,
    int shift, int ctrl, int alt, int caps_lock,
    ProcessKeyOutput* output);
void keymagic_free_string(char* s);
KeyMagicResult keymagic_engine_reset(EngineHandle* handle);
char* keymagic_engine_get_composition(EngineHandle* handle);
const char* keymagic_get_version(void);
```

### TSF Implementation Details

#### Language Bar Integration with Icons
```cpp
// Language bar button with keyboard icon
class CKeyMagicLangBarButton : public ITfLangBarItemButton {
private:
    HICON m_hIcon;  // Current keyboard icon
    
public:
    STDAPI GetIcon(HICON *phIcon) {
        // Load icon from current keyboard
        *phIcon = m_hIcon ? m_hIcon : LoadDefaultIcon();
        return S_OK;
    }
    
    STDAPI GetText(BSTR *pbstrText) {
        // Return current keyboard name
        *pbstrText = SysAllocString(GetCurrentKeyboardName());
        return S_OK;
    }
    
    void UpdateKeyboardIcon(const std::vector<uint8_t>& iconData) {
        if (!iconData.empty()) {
            // Convert BMP data to HICON
            m_hIcon = CreateIconFromBMP(iconData.data(), iconData.size());
        }
    }
};
```

#### Key Event Processing
```cpp
HRESULT OnKeyDown(ITfContext *pContext, WPARAM wParam, LPARAM lParam, BOOL *pfEaten) {
    ProcessKeyOutput output = {0};
    
    // Convert Windows key to FFI format
    int keyCode = static_cast<int>(wParam);
    char character = MapVirtualKeyToChar(wParam, lParam);
    
    // Get modifiers
    int shift = (GetKeyState(VK_SHIFT) & 0x8000) ? 1 : 0;
    int ctrl = (GetKeyState(VK_CONTROL) & 0x8000) ? 1 : 0;
    int alt = (GetKeyState(VK_MENU) & 0x8000) ? 1 : 0;
    int capsLock = (GetKeyState(VK_CAPITAL) & 0x0001) ? 1 : 0;
    
    // Process key
    KeyMagicResult result = keymagic_engine_process_key(
        m_engineHandle, keyCode, character, 
        shift, ctrl, alt, capsLock, &output
    );
    
    if (result == KeyMagicResult_Success) {
        // Check if we should commit text
        bool shouldCommit = false;
        std::string textToCommit;
        
        if (output.composing_text) {
            std::string composing(output.composing_text);
            
            // Check if we should commit based on key
            switch (wParam) {
                case VK_SPACE:
                    if (output.is_processed) {
                        // Engine processed space, check if composing ends with space
                        if (!composing.empty() && composing.back() == ' ') {
                            shouldCommit = true;
                            textToCommit = composing;
                        }
                    } else {
                        // Engine didn't process space, commit current text + space
                        shouldCommit = true;
                        textToCommit = composing + " ";
                    }
                    break;
                    
                case VK_RETURN:  // Enter key - commit without adding newline
                case VK_TAB:     // Tab key - commit without adding tab
                    shouldCommit = true;
                    textToCommit = composing;
                    break;
                    
                case VK_ESCAPE:  // Escape - cancel composition
                    shouldCommit = false;
                    keymagic_engine_reset(m_engineHandle);
                    TerminateComposition(ec);
                    *pfEaten = TRUE;
                    return S_OK;
            }
            
            // Request edit session
            RequestEditSession(pContext, shouldCommit, textToCommit, composing);
        }
        
        *pfEaten = output.is_processed ? TRUE : FALSE;
    }
    
    // Cleanup
    if (output.text) keymagic_free_string(output.text);
    if (output.composing_text) keymagic_free_string(output.composing_text);
    
    return S_OK;
}
```

#### Composition Management
```cpp
class CEditSession : public ITfEditSession {
    bool m_shouldCommit;
    std::wstring m_textToCommit;
    std::wstring m_composingText;
    
public:
    STDAPI DoEditSession(TfEditCookie ec) {
        if (m_shouldCommit) {
            // Commit text and clear composition
            CommitText(ec, m_textToCommit);
            TerminateComposition(ec);
            
            // Reset engine after commit
            keymagic_engine_reset(m_pTextService->GetEngineHandle());
        } else {
            // Just update composition display
            if (!m_composingText.empty()) {
                UpdateCompositionString(ec, m_composingText);
            } else {
                TerminateComposition(ec);
            }
        }
        return S_OK;
    }
};

// Simplified composition update
void UpdateCompositionString(TfEditCookie ec, const std::wstring& text) {
    if (!m_pComposition) {
        StartComposition(ec);
    }
    
    ITfRange *pRange;
    if (SUCCEEDED(m_pComposition->GetRange(&pRange))) {
        // Set the composition text
        pRange->SetText(ec, 0, text.c_str(), text.length());
        
        // Apply underline display attribute
        ApplyDisplayAttributes(ec, pRange);
        
        pRange->Release();
    }
}

// Commit text to document
void CommitText(TfEditCookie ec, const std::wstring& text) {
    // Terminate any existing composition first
    if (m_pComposition) {
        TerminateComposition(ec);
    }
    
    // Insert the committed text
    ITfInsertAtSelection *pInsertAtSelection;
    ITfRange *pRange;
    
    if (SUCCEEDED(m_pContext->QueryInterface(IID_ITfInsertAtSelection, 
                                             (void **)&pInsertAtSelection))) {
        if (SUCCEEDED(pInsertAtSelection->InsertTextAtSelection(
            ec, 0, text.c_str(), text.length(), &pRange))) {
            pRange->Release();
        }
        pInsertAtSelection->Release();
    }
}
```

#### Thread Safety
```cpp
class CKeyMagicTextService {
private:
    CRITICAL_SECTION m_cs;
    void* m_engineHandle;
    
public:
    void ProcessKeyThreadSafe(...) {
        EnterCriticalSection(&m_cs);
        // Process key
        LeaveCriticalSection(&m_cs);
    }
};
```

### GUI Implementation Details

#### Main Window Structure
- **ListView**: Display installed keyboards with columns (Icon, Name, Description, Hotkey)
  - Icons extracted from .km2 files if available
  - Default icon for keyboards without embedded icons
- **Toolbar**: Add, Remove, Settings, Help buttons
- **Status Bar**: Active keyboard, TSF status
- **System Tray**: Quick switch menu with keyboard icons, settings access

#### Keyboard Manager (Rust)
```rust
use keymagic_core::ffi::*;
use keymagic_core::km2::Km2Loader;
use std::collections::HashMap;
use std::ffi::CString;

pub struct KeyboardInfo {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub description: String,
    pub icon_data: Option<Vec<u8>>,  // BMP data from .km2
    pub hotkey: Option<String>,
    pub engine: *mut EngineHandle,
}

pub struct KeyboardManager {
    keyboards: HashMap<String, KeyboardInfo>,
    active_keyboard: Option<String>,
}

impl KeyboardManager {
    pub fn load_keyboard(&mut self, path: &Path) -> Result<String> {
        // First, load the .km2 file to extract metadata
        let km2_data = std::fs::read(path)?;
        let km2_file = Km2Loader::load(&km2_data)?;
        
        // Extract keyboard info
        let name = km2_file.info.get("name")
            .and_then(|v| v.as_string())
            .unwrap_or("Unknown Keyboard")
            .to_string();
            
        let description = km2_file.info.get("desc")
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();
            
        // Extract icon if available
        let icon_data = km2_file.info.get("icon")
            .and_then(|v| v.as_binary())
            .map(|data| data.to_vec());
            
        // Create engine
        let engine = unsafe { keymagic_engine_new() };
        let c_path = CString::new(path.to_str().unwrap())?;
        
        let result = unsafe {
            keymagic_engine_load_keyboard(engine, c_path.as_ptr())
        };
        
        if result == KeyMagicResult::Success {
            let keyboard_id = self.generate_id(path);
            
            let info = KeyboardInfo {
                id: keyboard_id.clone(),
                path: path.to_path_buf(),
                name,
                description,
                icon_data,
                hotkey: None,  // Loaded from registry
                engine,
            };
            
            self.keyboards.insert(keyboard_id.clone(), info);
            self.save_to_registry(&keyboard_id, path)?;
            Ok(keyboard_id)
        } else {
            unsafe { keymagic_engine_free(engine) };
            Err(anyhow!("Failed to load keyboard"))
        }
    }
    
    pub fn get_keyboard_icon(&self, id: &str) -> Option<HICON> {
        self.keyboards.get(id)
            .and_then(|info| info.icon_data.as_ref())
            .and_then(|bmp_data| self.bmp_to_hicon(bmp_data))
    }
    
    fn bmp_to_hicon(&self, bmp_data: &[u8]) -> Option<HICON> {
        // Convert BMP data to Windows HICON
        // Implementation using Windows GDI APIs
    }
    
    pub fn test_keyboard(&self, id: &str, text: &str) -> Result<String> {
        // Use engine to test input for keyboard preview
    }
}
```

#### Icon Usage Throughout the Application

1. **ListView Icons**:
   - 16x16 or 32x32 icons extracted from .km2 files
   - Default KeyMagic icon for keyboards without embedded icons
   - High DPI scaling support

2. **System Tray Menu**:
   - Active keyboard icon shown in system tray
   - Quick switch menu shows all keyboard icons
   - Visual indication of active keyboard

3. **Language Bar**:
   - TSF can provide keyboard icon to Windows language bar
   - Icon changes when switching keyboards

4. **Icon Extraction and Caching**:
   ```rust
   impl KeyboardManager {
       pub fn cache_keyboard_icons(&mut self) {
           let cache_dir = self.get_icon_cache_dir();
           
           for (id, info) in &self.keyboards {
               if let Some(bmp_data) = &info.icon_data {
                   // Save to cache for faster loading
                   let icon_path = cache_dir.join(format!("{}.bmp", id));
                   std::fs::write(&icon_path, bmp_data).ok();
               }
           }
       }
   }
   ```

#### Registry Layout
```
HKEY_CURRENT_USER\Software\KeyMagic\
    Settings\
        StartWithWindows = 1
        ShowInSystemTray = 1
        DefaultKeyboard = "myanmar-unicode"
    Keyboards\
        myanmar-unicode\
            Path = "C:\Program Files\KeyMagic\keyboards\myanmar-unicode.km2"
            Name = "Myanmar Unicode"
            Description = "Standard Myanmar Unicode keyboard"
            Hotkey = "Ctrl+Shift+M"
            Enabled = 1
            HasIcon = 1  // Indicates embedded icon available
```

### Build System

#### TSF Build (CMake)
```cmake
cmake_minimum_required(VERSION 3.20)
project(KeyMagicTSF)

# Import Rust library
find_package(Corrosion REQUIRED)
corrosion_import_crate(MANIFEST_PATH ../Cargo.toml)

# TSF DLL
add_library(KeyMagicTSF SHARED
    src/KeyMagicTextService.cpp
    src/KeyEventSink.cpp
    src/Composition.cpp
    src/DisplayAttribute.cpp
    src/ClassFactory.cpp
    src/DllMain.cpp
    src/Registry.cpp
    src/Globals.cpp
)

target_link_libraries(KeyMagicTSF PRIVATE
    keymagic_core    # Rust static library
    msctf           # TSF
    uuid
    ole32
    advapi32
)

# Export definition
set_target_properties(KeyMagicTSF PROPERTIES
    LINK_FLAGS "/DEF:${CMAKE_CURRENT_SOURCE_DIR}/src/KeyMagicTSF.def"
)
```

#### Rust Build
```toml
# keymagic-windows/Cargo.toml
[workspace]
members = ["gui"]

[dependencies]
keymagic-core = { path = "../keymagic-core" }

# gui/Cargo.toml
[package]
name = "keymagic-config"
version = "0.1.0"

[dependencies]
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Controls",
    "Win32_System_Registry",
    "Win32_UI_Shell",
    "Win32_Graphics_Gdi",
]}
keymagic-core = { path = "../../keymagic-core" }
anyhow = "1.0"

[build-dependencies]
winres = "0.1"
```

## Implementation Phases

### Phase 5.1: Foundation Setup

**Objectives**:
- Create project structure
- Set up build systems
- Implement basic COM server

**Tasks**:
1. Create keymagic-windows directory structure
2. Set up CMake for TSF project
3. Create keymagic_ffi.h from Rust FFI
4. Implement COM server skeleton:
   - DllMain, DllGetClassObject, DllRegisterServer
   - Basic ITfTextInputProcessor
   - Registry registration
5. Set up Rust GUI project with windows-rs
6. Create basic Win32 window

**Deliverables**:
- Compilable TSF DLL (non-functional)
- Basic GUI window executable
- Build scripts for both components

### Phase 5.2: Core Functionality

**Objectives**:
- Implement key processing pipeline
- Basic keyboard management
- Initial TSF-GUI integration

**Tasks**:
1. Implement ITfKeyEventSink
2. Wire up keymagic_engine_process_key
3. Basic composition handling
4. Text insertion/deletion
5. GUI: Keyboard list view
6. GUI: Add/remove keyboards
7. Complete TSF composition management
8. Display attributes (underline)
9. Context focus handling
10. GUI: Registry persistence
11. GUI: System tray
12. TSF-GUI communication via registry

**Deliverables**:
- Functional TSF that can process keys
- GUI that can manage keyboards
- Basic integration working

### Phase 5.3: Advanced Features

**Objectives**:
- Polish TSF behavior
- Complete GUI features
- System integration

**Tasks**:
1. TSF: Smart backspace support
2. TSF: Proper composition termination
3. TSF: Multi-context support
4. GUI: Keyboard preview/test area
5. GUI: Hotkey configuration
6. GUI: Settings dialog
7. Language bar integration
8. Keyboard switching UI
9. Auto-start with Windows
10. High DPI support
11. Error handling improvements
12. Logging system

**Deliverables**:
- Feature-complete TSF
- Polished GUI with all features
- Smooth system integration

### Phase 5.4: Testing and Polish

**Objectives**:
- Comprehensive testing
- Performance optimization
- Bug fixes

**Tasks**:
1. Unit tests for TSF components
2. GUI automated tests
3. Integration testing suite
4. Performance profiling
5. Memory leak detection
6. User acceptance testing
7. Bug fixes based on testing

**Deliverables**:
- Test suite
- Performance metrics
- Bug-free implementation

### Phase 5.5: Deployment

**Objectives**:
- Create installer
- Documentation
- Release preparation

**Tasks**:
1. Inno Setup installer script
2. Code signing (if available)
3. User manual with screenshots
4. Developer documentation
5. Troubleshooting guide
6. Release build optimization
7. Final testing on clean systems

**Deliverables**:
- Signed installer
- Complete documentation
- Release-ready package

## Testing Strategy

### Unit Tests
- TSF: COM interface tests
- TSF: Key processing tests
- GUI: Keyboard manager tests
- FFI: Interface boundary tests

### Integration Tests
- TSF + keymagic-core integration
- GUI + Registry integration
- TSF + GUI communication
- Full system tests

### Test Applications
- Notepad (basic Win32)
- WordPad (RichEdit)
- Chrome/Firefox (web input)
- Microsoft Word (complex)
- Windows Terminal (console)

### Performance Targets
- Key processing latency: < 10ms
- Memory usage: < 50MB
- Startup time: < 500ms
- No memory leaks over 24h usage

## Risk Mitigation

### Technical Risks
1. **TSF Complexity**: Mitigate with incremental implementation
2. **Thread Safety**: Use CRITICAL_SECTION, thorough testing
3. **Memory Management**: Use RAII, smart pointers
4. **Compatibility**: Test on multiple Windows versions

### Schedule Risks
1. **TSF Learning Curve**: May require additional research during implementation
2. **GUI Complexity**: Consider simplified v1 features
3. **Testing Coverage**: Automated tests essential for comprehensive coverage

## Success Criteria

1. **Functional Requirements**:
   - Process all KeyMagic keyboard layouts correctly
   - Work in all standard Windows applications
   - GUI can manage multiple keyboards
   - System tray quick switching works

2. **Performance Requirements**:
   - Meet all performance targets
   - No noticeable input lag
   - Smooth composition display

3. **Quality Requirements**:
   - No crashes in 24-hour usage
   - No memory leaks
   - Proper error handling
   - Clean uninstall

## Resources Required

### Development Tools
- Visual Studio 2022 or later
- Windows SDK (latest)
- CMake 3.20+
- Rust toolchain (MSVC target)
- Inno Setup 6+

### Documentation
- TSF Documentation (MSDN)
- windows-rs documentation
- KeyMagic design documents

### Testing Resources
- Windows 10/11 test machines
- Various test applications
- Automated testing tools

## Conclusion

This plan provides a comprehensive roadmap for implementing a production-quality TSF-based IME for KeyMagic on Windows. The phased approach allows for incremental development and testing, reducing risk and ensuring quality. The use of existing keymagic-core FFI simplifies the implementation while maintaining consistency with other platforms.