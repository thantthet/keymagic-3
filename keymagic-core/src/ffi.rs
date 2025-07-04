//! Foreign Function Interface for KeyMagic Core
//! 
//! This module provides a C-compatible API that can be used from any language
//! that supports C FFI (Python, C, C++, etc.) across all platforms.

use crate::{KeyMagicEngine, KeyInput};
use crate::engine::{ModifierState, ActionType, Predefined};
use crate::km2::Km2Loader;
use crate::types::VirtualKey;
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

    // Initialize output
    output_ref.action_type = 0;
    output_ref.text = ptr::null_mut();
    output_ref.delete_count = 0;
    output_ref.composing_text = ptr::null_mut();
    output_ref.is_processed = 0;

    // Convert character from c_char to Option<char>
    let char_opt = if character == 0 {
        None
    } else {
        Some(character as u8 as char)
    };

    let key_input = KeyInput {
        key_code: Predefined::from_raw(key_code as u16),
        modifiers: ModifierState {
            shift: shift != 0,
            ctrl: ctrl != 0,
            alt: alt != 0,
            caps_lock: caps_lock != 0,
        },
        character: char_opt,
    };

    match handle.engine.lock() {
        Ok(mut engine_opt) => {
            if let Some(engine) = engine_opt.as_mut() {
                match engine.process_key(key_input) {
                    Ok(result) => {
                        // Set composing text
                        if let Ok(c_string) = CString::new(result.composing_text.clone()) {
                            output_ref.composing_text = c_string.into_raw();
                        }
                        
                        // Process action
                        match &result.action {
                            ActionType::None => {
                                output_ref.action_type = 0;
                            }
                            ActionType::Insert(text) => {
                                output_ref.action_type = 1;
                                if let Ok(c_string) = CString::new(text.clone()) {
                                    output_ref.text = c_string.into_raw();
                                }
                            }
                            ActionType::BackspaceDelete(count) => {
                                output_ref.action_type = 2;
                                output_ref.delete_count = *count as c_int;
                            }
                            ActionType::BackspaceDeleteAndInsert(count, text) => {
                                output_ref.action_type = 3;
                                output_ref.delete_count = *count as c_int;
                                if let Ok(c_string) = CString::new(text.clone()) {
                                    output_ref.text = c_string.into_raw();
                                }
                            }
                        }
                        
                        // Set the is_processed flag
                        output_ref.is_processed = if result.is_processed { 1 } else { 0 };
                        
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

/// Get library version
#[no_mangle]
pub extern "C" fn keymagic_get_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Process a key event with Windows VK code
/// 
/// This is a Windows-specific variant that accepts Windows Virtual Key codes
/// and converts them to KeyMagic predefined values before processing.
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
    if handle.is_null() || output.is_null() {
        return KeyMagicResult::ErrorInvalidParameter;
    }

    // Convert Windows VK code to KeyMagic predefined value
    let predefined_code = if let Some(vk) = VirtualKey::from_win_vk(vk_code as u16) {
        vk as u16
    } else {
        // If we don't recognize the VK code, pass it through as-is
        // This allows for potential custom handling
        vk_code as u16
    };

    // Call the standard process_key function with the converted code
    keymagic_engine_process_key(
        handle,
        predefined_code as c_int,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_lifecycle() {
        unsafe {
            let engine = keymagic_engine_new();
            assert!(!engine.is_null());
            keymagic_engine_free(engine);
        }
    }

    #[test]
    fn test_version() {
        unsafe {
            let version = keymagic_get_version();
            assert!(!version.is_null());
            let version_str = CStr::from_ptr(version).to_str().unwrap();
            assert!(!version_str.is_empty());
        }
    }
}