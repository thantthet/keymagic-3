//! Foreign Function Interface for KeyMagic Core
//! 
//! This module provides a C-compatible API that can be used from any language
//! that supports C FFI (Python, C, C++, etc.) across all platforms.

use crate::{KeyInput, KeyMagicEngine, VirtualKey, Km2File};
use crate::engine::{ModifierState, ActionType};
use crate::km2::Km2Loader;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::Mutex;

/// Opaque handle to a KeyMagic engine instance
pub struct EngineHandle {
    engine: Mutex<Option<KeyMagicEngine>>,
}

/// Result codes for FFI functions
#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum KeyMagicResult {
    Success = 0,
    ErrorInvalidHandle = -1,
    ErrorInvalidParameter = -2,
    ErrorEngineFailure = -3,
    ErrorUtf8Conversion = -4,
    ErrorNoKeyboard = -5,
}

/// Output from processing a key event
#[repr(C)]
pub struct ProcessKeyOutput {
    /// Action type: 0=None, 1=Insert, 2=BackspaceDelete, 3=BackspaceDeleteAndInsert
    pub action_type: c_int,
    /// Text to insert (UTF-8 encoded, null-terminated)
    pub text: *mut c_char,
    /// Number of characters to delete
    pub delete_count: c_int,
    /// Current composing text
    pub composing_text: *mut c_char,
    /// Whether the key was processed by the engine (0=false, 1=true)
    pub is_processed: c_int,
}

/// Creates a new engine instance
#[no_mangle]
pub extern "C" fn keymagic_engine_new() -> *mut EngineHandle {
    let handle = Box::new(EngineHandle {
        engine: Mutex::new(None),
    });
    Box::into_raw(handle)
}

/// Frees an engine instance
#[no_mangle]
pub extern "C" fn keymagic_engine_free(handle: *mut EngineHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

/// Loads a KM2 keyboard layout file
#[no_mangle]
pub extern "C" fn keymagic_engine_load_keyboard(
    handle: *mut EngineHandle,
    km2_path: *const c_char,
) -> KeyMagicResult {
    if handle.is_null() || km2_path.is_null() {
        return KeyMagicResult::ErrorInvalidParameter;
    }

    let handle = unsafe { &*handle };
    let path_str = match unsafe { CStr::from_ptr(km2_path) }.to_str() {
        Ok(s) => s,
        Err(_) => return KeyMagicResult::ErrorUtf8Conversion,
    };

    let km2_data = match std::fs::read(path_str) {
        Ok(data) => data,
        Err(_) => return KeyMagicResult::ErrorEngineFailure,
    };

    let km2_file = match Km2Loader::load(&km2_data) {
        Ok(file) => file,
        Err(_) => return KeyMagicResult::ErrorEngineFailure,
    };

    match handle.engine.lock() {
        Ok(mut engine_opt) => {
            match KeyMagicEngine::new(km2_file) {
                Ok(engine) => {
                    *engine_opt = Some(engine);
                    KeyMagicResult::Success
                }
                Err(_) => KeyMagicResult::ErrorEngineFailure,
            }
        }
        Err(_) => KeyMagicResult::ErrorEngineFailure,
    }
}

/// Loads a keyboard from memory buffer
#[no_mangle]
pub extern "C" fn keymagic_engine_load_keyboard_from_memory(
    handle: *mut EngineHandle,
    km2_data: *const u8,
    data_len: usize,
) -> KeyMagicResult {
    if handle.is_null() || km2_data.is_null() || data_len == 0 {
        return KeyMagicResult::ErrorInvalidParameter;
    }

    let handle = unsafe { &*handle };
    let data_slice = unsafe { std::slice::from_raw_parts(km2_data, data_len) };
    
    let km2_file = match Km2Loader::load(data_slice) {
        Ok(file) => file,
        Err(_) => return KeyMagicResult::ErrorEngineFailure,
    };

    match handle.engine.lock() {
        Ok(mut engine_opt) => {
            match KeyMagicEngine::new(km2_file) {
                Ok(engine) => {
                    *engine_opt = Some(engine);
                    KeyMagicResult::Success
                }
                Err(_) => KeyMagicResult::ErrorEngineFailure,
            }
        }
        Err(_) => KeyMagicResult::ErrorEngineFailure,
    }
}

/// Internal function to process key events (shared by normal and dry-run)
fn process_key_internal(
    handle: &EngineHandle,
    key_input: KeyInput,
    dry_run: bool,
    output: &mut ProcessKeyOutput,
) -> KeyMagicResult {
    // Initialize output
    output.action_type = 0;
    output.text = ptr::null_mut();
    output.delete_count = 0;
    output.composing_text = ptr::null_mut();
    output.is_processed = 0;

    match handle.engine.lock() {
        Ok(mut engine_opt) => {
            if let Some(engine) = engine_opt.as_mut() {
                let result = if dry_run {
                    engine.process_key_test(key_input)
                } else {
                    engine.process_key(key_input)
                };

                match result {
                    Ok(result) => {
                        // Set composing text
                        if let Ok(c_string) = CString::new(result.composing_text.clone()) {
                            output.composing_text = c_string.into_raw();
                        }
                        
                        // Process action
                        match &result.action {
                            ActionType::None => {
                                output.action_type = 0;
                            }
                            ActionType::Insert(text) => {
                                output.action_type = 1;
                                if let Ok(c_string) = CString::new(text.clone()) {
                                    output.text = c_string.into_raw();
                                }
                            }
                            ActionType::BackspaceDelete(count) => {
                                output.action_type = 2;
                                output.delete_count = *count as c_int;
                            }
                            ActionType::BackspaceDeleteAndInsert(count, text) => {
                                output.action_type = 3;
                                output.delete_count = *count as c_int;
                                if let Ok(c_string) = CString::new(text.clone()) {
                                    output.text = c_string.into_raw();
                                }
                            }
                        }
                        
                        // Set the is_processed flag
                        output.is_processed = if result.is_processed { 1 } else { 0 };
                        
                        KeyMagicResult::Success
                    }
                    Err(_) => KeyMagicResult::ErrorEngineFailure,
                }
            } else {
                KeyMagicResult::ErrorNoKeyboard
            }
        }
        Err(_) => KeyMagicResult::ErrorEngineFailure,
    }
}

/// Processes a key event
#[no_mangle]
pub extern "C" fn keymagic_engine_process_key(
    handle: *mut EngineHandle,
    key_code: c_int,
    character: c_char,
    shift: c_int,
    ctrl: c_int,
    alt: c_int,
    caps_lock: c_int,
    output: *mut ProcessKeyOutput,
) -> KeyMagicResult {
    if handle.is_null() || output.is_null() {
        return KeyMagicResult::ErrorInvalidParameter;
    }

    let handle = unsafe { &*handle };
    let output_ref = unsafe { &mut *output };

    // Convert character from c_char to Option<char>
    let char_opt = if character == 0 {
        None
    } else {
        Some(character as u8 as char)
    };

    let key_input = KeyInput {
        key_code: key_code as u16,
        modifiers: ModifierState {
            shift: shift != 0,
            ctrl: ctrl != 0,
            alt: alt != 0,
            caps_lock: caps_lock != 0,
        },
        character: char_opt,
    };

    process_key_internal(handle, key_input, false, output_ref)
}

/// Frees a string allocated by the engine
#[no_mangle]
pub extern "C" fn keymagic_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

/// Resets the engine state
#[no_mangle]
pub extern "C" fn keymagic_engine_reset(handle: *mut EngineHandle) -> KeyMagicResult {
    if handle.is_null() {
        return KeyMagicResult::ErrorInvalidParameter;
    }

    let handle = unsafe { &*handle };
    match handle.engine.lock() {
        Ok(mut engine_opt) => {
            if let Some(engine) = engine_opt.as_mut() {
                engine.reset();
                KeyMagicResult::Success
            } else {
                KeyMagicResult::ErrorNoKeyboard
            }
        }
        Err(_) => KeyMagicResult::ErrorEngineFailure,
    }
}

/// Gets the current composition string
#[no_mangle]
pub extern "C" fn keymagic_engine_get_composition(
    handle: *mut EngineHandle,
) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }

    let handle = unsafe { &*handle };
    match handle.engine.lock() {
        Ok(engine_opt) => {
            if let Some(engine) = engine_opt.as_ref() {
                let composition = engine.composing_text();
                match CString::new(composition) {
                    Ok(c_string) => c_string.into_raw(),
                    Err(_) => ptr::null_mut(),
                }
            } else {
                ptr::null_mut()
            }
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Sets the composition string
#[no_mangle]
pub extern "C" fn keymagic_engine_set_composition(
    handle: *mut EngineHandle,
    text: *const c_char,
) -> KeyMagicResult {
    if handle.is_null() {
        return KeyMagicResult::ErrorInvalidParameter;
    }

    let handle = unsafe { &*handle };
    
    // Handle null or empty text
    let text_str = if text.is_null() {
        ""
    } else {
        match unsafe { CStr::from_ptr(text) }.to_str() {
            Ok(s) => s,
            Err(_) => return KeyMagicResult::ErrorUtf8Conversion,
        }
    };

    match handle.engine.lock() {
        Ok(mut engine_opt) => {
            if let Some(engine) = engine_opt.as_mut() {
                engine.set_composing_text(text_str.to_string());
                KeyMagicResult::Success
            } else {
                KeyMagicResult::ErrorNoKeyboard
            }
        }
        Err(_) => KeyMagicResult::ErrorEngineFailure,
    }
}

/// Get library version
#[no_mangle]
pub extern "C" fn keymagic_get_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Process a key event with Windows VK code
#[no_mangle]
pub extern "C" fn keymagic_engine_process_key_win(
    handle: *mut EngineHandle,
    vk_code: c_int,        // Windows VK code (e.g., 0x41 for VK_A)
    character: c_char,
    shift: c_int,
    ctrl: c_int,
    alt: c_int,
    caps_lock: c_int,
    output: *mut ProcessKeyOutput,
) -> KeyMagicResult {
    // Convert Windows VK code to VirtualKey, return error if unsupported
    let virtual_key = match VirtualKey::from_win_vk(vk_code as u16) {
        Some(vk) => vk,
        None => return KeyMagicResult::ErrorInvalidParameter,
    };
    
    // Forward the call to the regular process_key function
    keymagic_engine_process_key(
        handle,
        virtual_key as i32,
        character,
        shift,
        ctrl,
        alt,
        caps_lock,
        output,
    )
}

/// Process a key event in test mode (does not modify engine state)
#[no_mangle]
pub extern "C" fn keymagic_engine_process_key_test(
    handle: *mut EngineHandle,
    key_code: c_int,
    character: c_char,
    shift: c_int,
    ctrl: c_int,
    alt: c_int,
    caps_lock: c_int,
    output: *mut ProcessKeyOutput,
) -> KeyMagicResult {
    if handle.is_null() || output.is_null() {
        return KeyMagicResult::ErrorInvalidParameter;
    }

    let handle = unsafe { &*handle };
    let output_ref = unsafe { &mut *output };

    // Convert character from c_char to Option<char>
    let char_opt = if character == 0 {
        None
    } else {
        Some(character as u8 as char)
    };

    let key_input = KeyInput {
        key_code: key_code as u16,
        modifiers: ModifierState {
            shift: shift != 0,
            ctrl: ctrl != 0,
            alt: alt != 0,
            caps_lock: caps_lock != 0,
        },
        character: char_opt,
    };

    process_key_internal(handle, key_input, true, output_ref)
}

/// Process a key event in test mode with Windows VK code (does not modify engine state)
#[no_mangle]
pub extern "C" fn keymagic_engine_process_key_test_win(
    handle: *mut EngineHandle,
    vk_code: c_int,        // Windows VK code (e.g., 0x41 for VK_A)
    character: c_char,
    shift: c_int,
    ctrl: c_int,
    alt: c_int,
    caps_lock: c_int,
    output: *mut ProcessKeyOutput,
) -> KeyMagicResult {
    // Convert Windows VK code to VirtualKey, return error if unsupported
    let virtual_key = match VirtualKey::from_win_vk(vk_code as u16) {
        Some(vk) => vk,
        None => return KeyMagicResult::ErrorInvalidParameter,
    };
    
    // Forward the call to the regular test function
    keymagic_engine_process_key_test(
        handle,
        virtual_key as i32,
        character,
        shift,
        ctrl,
        alt,
        caps_lock,
        output,
    )
}

/// Get the current composing text from the engine
/// Returns a newly allocated C string that must be freed with keymagic_engine_free_string
#[no_mangle]
pub extern "C" fn keymagic_engine_get_composing_text(engine: *mut KeyMagicEngine) -> *mut c_char {
    if engine.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let engine_ref = &*engine;
        match CString::new(engine_ref.composing_text()) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }
}

/// Free a string allocated by the engine
#[no_mangle]
pub extern "C" fn keymagic_engine_free_string(str: *mut c_char) {
    if !str.is_null() {
        unsafe {
            let _ = CString::from_raw(str);
        }
    }
}

/// Opaque handle for loaded KM2 file
pub struct Km2FileHandle(Km2File);

/// Load a KM2 file from path
/// Returns NULL on failure
#[no_mangle]
pub extern "C" fn keymagic_km2_load(path: *const c_char) -> *mut Km2FileHandle {
    if path.is_null() {
        return std::ptr::null_mut();
    }

    let path_str = unsafe {
        match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };

    match std::fs::read(path_str) {
        Ok(data) => {
            match crate::km2::Km2Loader::load(&data) {
                Ok(km2) => Box::into_raw(Box::new(Km2FileHandle(km2))),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a loaded KM2 file
#[no_mangle]
pub extern "C" fn keymagic_km2_free(handle: *mut Km2FileHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

/// Get keyboard name from KM2 file
/// Returns a newly allocated C string that must be freed with keymagic_free_string
/// Returns NULL if no name is defined
#[no_mangle]
pub extern "C" fn keymagic_km2_get_name(handle: *mut Km2FileHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let km2 = &(*handle).0;
        let metadata = km2.metadata();
        
        match metadata.name() {
            Some(name) => match CString::new(name) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            },
            None => std::ptr::null_mut(),
        }
    }
}

/// Get keyboard description from KM2 file
/// Returns a newly allocated C string that must be freed with keymagic_free_string
/// Returns NULL if no description is defined
#[no_mangle]
pub extern "C" fn keymagic_km2_get_description(handle: *mut Km2FileHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let km2 = &(*handle).0;
        let metadata = km2.metadata();
        
        match metadata.description() {
            Some(desc) => match CString::new(desc) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            },
            None => std::ptr::null_mut(),
        }
    }
}

/// Get hotkey from KM2 file
/// Returns a newly allocated C string that must be freed with keymagic_free_string
/// Returns NULL if no hotkey is defined
#[no_mangle]
pub extern "C" fn keymagic_km2_get_hotkey(handle: *mut Km2FileHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let km2 = &(*handle).0;
        let metadata = km2.metadata();
        
        match metadata.hotkey() {
            Some(hotkey) => match CString::new(hotkey) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            },
            None => std::ptr::null_mut(),
        }
    }
}

/// Parsed hotkey information for FFI
#[repr(C)]
pub struct HotkeyInfo {
    pub key_code: c_int,       // VirtualKey as int
    pub ctrl: c_int,           // 0 or 1
    pub alt: c_int,            // 0 or 1
    pub shift: c_int,          // 0 or 1
    pub meta: c_int,           // 0 or 1
}

/// Parse a hotkey string and return hotkey info
/// Returns 1 on success, 0 on failure
#[no_mangle]
pub extern "C" fn keymagic_parse_hotkey(hotkey_str: *const c_char, info: *mut HotkeyInfo) -> c_int {
    if hotkey_str.is_null() || info.is_null() {
        return 0;
    }

    let hotkey_string = unsafe {
        match CStr::from_ptr(hotkey_str).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };

    match crate::hotkey::HotkeyBinding::parse(hotkey_string) {
        Ok(binding) => {
            unsafe {
                (*info).key_code = binding.key as c_int;
                (*info).ctrl = if binding.ctrl { 1 } else { 0 };
                (*info).alt = if binding.alt { 1 } else { 0 };
                (*info).shift = if binding.shift { 1 } else { 0 };
                (*info).meta = if binding.meta { 1 } else { 0 };
            }
            1
        }
        Err(_) => 0,
    }
}

/// Get icon data from KM2 file
/// If buffer is NULL, returns the required buffer size
/// If buffer is not NULL, copies icon data to buffer and returns actual size copied
/// Returns 0 if no icon is defined or on error
#[no_mangle]
pub extern "C" fn keymagic_km2_get_icon_data(
    handle: *mut Km2FileHandle, 
    buffer: *mut u8,
    buffer_size: usize
) -> usize {
    if handle.is_null() {
        return 0;
    }

    unsafe {
        let km2 = &(*handle).0;
        let metadata = km2.metadata();
        
        match metadata.icon() {
            Some(icon_data) => {
                let data_len = icon_data.len();
                
                // If buffer is NULL, just return the required size
                if buffer.is_null() {
                    return data_len;
                }
                
                // If buffer is provided, copy data up to buffer_size
                if buffer_size >= data_len {
                    std::ptr::copy_nonoverlapping(icon_data.as_ptr(), buffer, data_len);
                    data_len
                } else {
                    // Buffer too small, return 0 to indicate error
                    0
                }
            }
            None => 0,
        }
    }
}

/// Convert VirtualKey enum value to display string
/// Returns a newly allocated C string that must be freed with keymagic_free_string
/// Returns NULL if the key code is invalid
#[no_mangle]
pub extern "C" fn keymagic_virtual_key_to_string(key_code: c_int) -> *mut c_char {
    // Try to convert the key_code to VirtualKey enum
    // Since VirtualKey uses u16 internally, we need to check bounds
    if key_code < 0 || key_code > u16::MAX as c_int {
        return ptr::null_mut();
    }
    
    // Find matching VirtualKey by iterating through all values
    let virtual_key = match key_code as u16 {
        2 => VirtualKey::Back,
        3 => VirtualKey::Tab,
        4 => VirtualKey::Return,
        5 => VirtualKey::Shift,
        6 => VirtualKey::Control,
        7 => VirtualKey::Menu,
        8 => VirtualKey::Pause,
        9 => VirtualKey::Capital,
        10 => VirtualKey::Kanji,
        11 => VirtualKey::Escape,
        12 => VirtualKey::Space,
        13 => VirtualKey::Prior,
        14 => VirtualKey::Next,
        15 => VirtualKey::Delete,
        16 => VirtualKey::Key0,
        17 => VirtualKey::Key1,
        18 => VirtualKey::Key2,
        19 => VirtualKey::Key3,
        20 => VirtualKey::Key4,
        21 => VirtualKey::Key5,
        22 => VirtualKey::Key6,
        23 => VirtualKey::Key7,
        24 => VirtualKey::Key8,
        25 => VirtualKey::Key9,
        26 => VirtualKey::KeyA,
        27 => VirtualKey::KeyB,
        28 => VirtualKey::KeyC,
        29 => VirtualKey::KeyD,
        30 => VirtualKey::KeyE,
        31 => VirtualKey::KeyF,
        32 => VirtualKey::KeyG,
        33 => VirtualKey::KeyH,
        34 => VirtualKey::KeyI,
        35 => VirtualKey::KeyJ,
        36 => VirtualKey::KeyK,
        37 => VirtualKey::KeyL,
        38 => VirtualKey::KeyM,
        39 => VirtualKey::KeyN,
        40 => VirtualKey::KeyO,
        41 => VirtualKey::KeyP,
        42 => VirtualKey::KeyQ,
        43 => VirtualKey::KeyR,
        44 => VirtualKey::KeyS,
        45 => VirtualKey::KeyT,
        46 => VirtualKey::KeyU,
        47 => VirtualKey::KeyV,
        48 => VirtualKey::KeyW,
        49 => VirtualKey::KeyX,
        50 => VirtualKey::KeyY,
        51 => VirtualKey::KeyZ,
        52 => VirtualKey::Numpad0,
        53 => VirtualKey::Numpad1,
        54 => VirtualKey::Numpad2,
        55 => VirtualKey::Numpad3,
        56 => VirtualKey::Numpad4,
        57 => VirtualKey::Numpad5,
        58 => VirtualKey::Numpad6,
        59 => VirtualKey::Numpad7,
        60 => VirtualKey::Numpad8,
        61 => VirtualKey::Numpad9,
        62 => VirtualKey::Multiply,
        63 => VirtualKey::Add,
        64 => VirtualKey::Separator,
        65 => VirtualKey::Subtract,
        66 => VirtualKey::Decimal,
        67 => VirtualKey::Divide,
        68 => VirtualKey::F1,
        69 => VirtualKey::F2,
        70 => VirtualKey::F3,
        71 => VirtualKey::F4,
        72 => VirtualKey::F5,
        73 => VirtualKey::F6,
        74 => VirtualKey::F7,
        75 => VirtualKey::F8,
        76 => VirtualKey::F9,
        77 => VirtualKey::F10,
        78 => VirtualKey::F11,
        79 => VirtualKey::F12,
        80 => VirtualKey::LShift,
        81 => VirtualKey::RShift,
        82 => VirtualKey::LControl,
        83 => VirtualKey::RControl,
        84 => VirtualKey::LMenu,
        85 => VirtualKey::RMenu,
        86 => VirtualKey::Oem1,
        87 => VirtualKey::OemPlus,
        88 => VirtualKey::OemComma,
        89 => VirtualKey::OemMinus,
        90 => VirtualKey::OemPeriod,
        91 => VirtualKey::Oem2,
        92 => VirtualKey::Oem3,
        93 => VirtualKey::Oem4,
        94 => VirtualKey::Oem5,
        95 => VirtualKey::Oem6,
        96 => VirtualKey::Oem7,
        97 => VirtualKey::Oem8,
        98 => VirtualKey::OemAx,
        99 => VirtualKey::Oem102,
        100 => VirtualKey::IcoHelp,
        101 => VirtualKey::Ico00,
        _ => return ptr::null_mut(),
    };
    
    // Get display string from the enum method
    let display_name = virtual_key.to_display_string();
    
    match CString::new(display_name) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

