//! Windows Registry operations for KeyMagic
//! 
//! This module consolidates all Windows registry read/write operations
//! for better maintainability and scalability.

#![cfg(target_os = "windows")]

use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyW, RegDeleteKeyW, RegDeleteValueW, RegEnumKeyExW,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_READ, REG_DWORD, REG_SZ,
};

// Registry key paths
const KEYMAGIC_ROOT: &str = "Software\\KeyMagic";
const KEYBOARDS_KEY: &str = "Software\\KeyMagic\\Keyboards";
const SETTINGS_KEY: &str = "Software\\KeyMagic\\Settings";
const WINDOWS_RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

// Registry value names
const DEFAULT_KEYBOARD_VALUE: &str = "DefaultKeyboard";
const KEY_PROCESSING_ENABLED_VALUE: &str = "KeyProcessingEnabled";
const ON_OFF_HOTKEY_VALUE: &str = "OnOffHotkey";
const KEYMAGIC_AUTOSTART_VALUE: &str = "KeyMagic";

// Keyboard entry value names
const KEYBOARD_PATH_VALUE: &str = "Path";
const KEYBOARD_NAME_VALUE: &str = "Name";
const KEYBOARD_DESCRIPTION_VALUE: &str = "Description";
const KEYBOARD_HOTKEY_VALUE: &str = "Hotkey";
const KEYBOARD_COLOR_VALUE: &str = "Color";
const KEYBOARD_ENABLED_VALUE: &str = "Enabled";
const KEYBOARD_HASH_VALUE: &str = "Hash";

/// Represents a keyboard entry in the registry
#[derive(Debug, Clone)]
pub struct RegistryKeyboard {
    pub id: String,
    pub path: String,
    pub name: String,
    pub description: String,
    pub hotkey: Option<String>,
    pub color: Option<String>,
    pub enabled: bool,
    pub hash: Option<String>,
}

/// Registry operation errors
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Failed to open registry key: {0}")]
    OpenKeyFailed(String),
    
    #[error("Failed to create registry key: {0}")]
    CreateKeyFailed(String),
    
    #[error("Failed to read registry value: {0}")]
    ReadValueFailed(String),
    
    #[error("Failed to write registry value: {0}")]
    WriteValueFailed(String),
    
    #[error("Registry key not found")]
    KeyNotFound,
    
    #[error("Registry value not found")]
    ValueNotFound,
    
    #[error("Invalid UTF-16 string")]
    InvalidUtf16,
    
    #[error("Windows API error: {0}")]
    WindowsApi(String),
}

impl From<windows::core::Error> for RegistryError {
    fn from(err: windows::core::Error) -> Self {
        RegistryError::WindowsApi(err.to_string())
    }
}

/// Helper function to convert string to wide string
fn to_wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Helper function to convert wide string to string
fn from_wide_string(wide: &[u16]) -> Result<String, RegistryError> {
    let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16(&wide[..len]).map_err(|_| RegistryError::InvalidUtf16)
}

// ===== Low-level Registry Operations =====

/// Opens a registry key for reading
fn open_registry_key(key_path: &str) -> Result<HKEY, RegistryError> {
    let wide_key = to_wide_string(key_path);
    let mut hkey = HKEY::default();
    
    unsafe {
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(wide_key.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        );
        
        match result {
            Ok(()) => Ok(hkey),
            Err(e) if e.code() == ERROR_FILE_NOT_FOUND.to_hresult() => Err(RegistryError::KeyNotFound),
            Err(e) => Err(RegistryError::OpenKeyFailed(format!("Error: {:?}", e))),
        }
    }
}

/// Opens a registry key for writing (creates if doesn't exist)
fn create_or_open_registry_key(key_path: &str) -> Result<HKEY, RegistryError> {
    let wide_key = to_wide_string(key_path);
    let mut hkey = HKEY::default();
    
    unsafe {
        let result = RegCreateKeyW(
            HKEY_CURRENT_USER,
            PCWSTR(wide_key.as_ptr()),
            &mut hkey,
        );
        
        if result.is_ok() {
            Ok(hkey)
        } else {
            Err(RegistryError::CreateKeyFailed(format!("Error: {:?}", result)))
        }
    }
}

/// Reads a string value from registry
fn read_registry_string(hkey: HKEY, value_name: &str) -> Result<String, RegistryError> {
    let wide_name = to_wide_string(value_name);
    let mut buffer = vec![0u16; 256];
    let mut size = (buffer.len() * 2) as u32;
    
    unsafe {
        let result = RegQueryValueExW(
            hkey,
            PCWSTR(wide_name.as_ptr()),
            None,
            None,
            Some(buffer.as_mut_ptr() as *mut u8 as *mut _),
            Some(&mut size),
        );
        
        match result {
            Ok(()) => from_wide_string(&buffer),
            Err(e) if e.code() == ERROR_FILE_NOT_FOUND.to_hresult() => Err(RegistryError::ValueNotFound),
            Err(e) => Err(RegistryError::ReadValueFailed(format!("Error: {:?}", e))),
        }
    }
}

/// Writes a string value to registry
fn write_registry_string(hkey: HKEY, value_name: &str, value: &str) -> Result<(), RegistryError> {
    let wide_name = to_wide_string(value_name);
    let wide_value = to_wide_string(value);
    
    unsafe {
        let value_bytes = std::slice::from_raw_parts(
            wide_value.as_ptr() as *const u8,
            wide_value.len() * 2
        );
        
        let result = RegSetValueExW(
            hkey,
            PCWSTR(wide_name.as_ptr()),
            0,
            REG_SZ,
            Some(value_bytes),
        );
        
        if result.is_err() {
            Err(RegistryError::WriteValueFailed(format!("Failed to set value: {:?}", result)))
        } else {
            Ok(())
        }
    }
}

/// Reads a DWORD value from registry
fn read_registry_dword(hkey: HKEY, value_name: &str) -> Result<u32, RegistryError> {
    let wide_name = to_wide_string(value_name);
    let mut value: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    
    unsafe {
        let result = RegQueryValueExW(
            hkey,
            PCWSTR(wide_name.as_ptr()),
            None,
            None,
            Some(&mut value as *mut u32 as *mut u8 as *mut _),
            Some(&mut size),
        );
        
        match result {
            Ok(()) => Ok(value),
            Err(e) if e.code() == ERROR_FILE_NOT_FOUND.to_hresult() => Err(RegistryError::ValueNotFound),
            Err(e) => Err(RegistryError::ReadValueFailed(format!("Error: {:?}", e))),
        }
    }
}

/// Writes a DWORD value to registry
fn write_registry_dword(hkey: HKEY, value_name: &str, value: u32) -> Result<(), RegistryError> {
    let wide_name = to_wide_string(value_name);
    
    unsafe {
        let value_bytes = std::slice::from_raw_parts(
            &value as *const u32 as *const u8,
            std::mem::size_of::<u32>()
        );
        
        let result = RegSetValueExW(
            hkey,
            PCWSTR(wide_name.as_ptr()),
            0,
            REG_DWORD,
            Some(value_bytes),
        );
        
        if result.is_err() {
            Err(RegistryError::WriteValueFailed(format!("Failed to set value: {:?}", result)))
        } else {
            Ok(())
        }
    }
}

/// Deletes a registry value
fn delete_registry_value(hkey: HKEY, value_name: &str) -> Result<(), RegistryError> {
    let wide_name = to_wide_string(value_name);
    
    unsafe {
        let result = RegDeleteValueW(hkey, PCWSTR(wide_name.as_ptr()));
        
        match result {
            Ok(()) => Ok(()),
            Err(e) if e.code() == ERROR_FILE_NOT_FOUND.to_hresult() => Ok(()),
            Err(e) => Err(RegistryError::WindowsApi(format!("Failed to delete value: {:?}", e))),
        }
    }
}

/// Deletes a registry key
fn delete_registry_key(parent_key: HKEY, key_name: &str) -> Result<(), RegistryError> {
    let wide_name = to_wide_string(key_name);
    
    unsafe {
        let result = RegDeleteKeyW(parent_key, PCWSTR(wide_name.as_ptr()));
        
        match result {
            Ok(()) => Ok(()),
            Err(e) if e.code() == ERROR_FILE_NOT_FOUND.to_hresult() => Ok(()),
            Err(e) => Err(RegistryError::WindowsApi(format!("Failed to delete key: {:?}", e))),
        }
    }
}

/// Enumerates subkeys of a registry key
fn enumerate_subkeys(hkey: HKEY) -> Result<Vec<String>, RegistryError> {
    let mut subkeys = Vec::new();
    let mut index = 0;
    
    loop {
        let mut name_buffer = vec![0u16; 256];
        let mut name_size = name_buffer.len() as u32;
        
        unsafe {
            let result = RegEnumKeyExW(
                hkey,
                index,
                windows::core::PWSTR(name_buffer.as_mut_ptr()),
                &mut name_size,
                None,
                windows::core::PWSTR::null(),
                None,
                None,
            );
            
            if result.is_ok() {
                if let Ok(name) = from_wide_string(&name_buffer[..name_size as usize]) {
                    subkeys.push(name);
                }
                index += 1;
            } else {
                break;
            }
        }
    }
    
    Ok(subkeys)
}

// ===== High-level Operations =====

/// Ensures the KeyMagic registry structure exists
pub fn ensure_registry_structure() -> Result<(), RegistryError> {
    // Create root key
    let root_key = create_or_open_registry_key(KEYMAGIC_ROOT)?;
    unsafe { let _ = RegCloseKey(root_key); }
    
    // Create keyboards key
    let keyboards_key = create_or_open_registry_key(KEYBOARDS_KEY)?;
    unsafe { let _ = RegCloseKey(keyboards_key); }
    
    // Create settings key
    let settings_key = create_or_open_registry_key(SETTINGS_KEY)?;
    unsafe { let _ = RegCloseKey(settings_key); }
    
    Ok(())
}

// ===== Keyboard Management =====

/// Loads all keyboards from registry
pub fn load_keyboards() -> Result<Vec<RegistryKeyboard>, RegistryError> {
    let mut keyboards = Vec::new();
    
    let keyboards_key = match open_registry_key(KEYBOARDS_KEY) {
        Ok(key) => key,
        Err(RegistryError::KeyNotFound) => return Ok(keyboards),
        Err(e) => return Err(e),
    };
    
    let subkeys = enumerate_subkeys(keyboards_key)?;
    
    for keyboard_id in subkeys {
        let keyboard_key_path = format!("{}\\{}", KEYBOARDS_KEY, keyboard_id);
        if let Ok(keyboard_key) = open_registry_key(&keyboard_key_path) {
            let keyboard = RegistryKeyboard {
                id: keyboard_id,
                path: read_registry_string(keyboard_key, KEYBOARD_PATH_VALUE).unwrap_or_default(),
                name: read_registry_string(keyboard_key, KEYBOARD_NAME_VALUE).unwrap_or_default(),
                description: read_registry_string(keyboard_key, KEYBOARD_DESCRIPTION_VALUE).unwrap_or_default(),
                hotkey: read_registry_string(keyboard_key, KEYBOARD_HOTKEY_VALUE).ok(),
                color: read_registry_string(keyboard_key, KEYBOARD_COLOR_VALUE).ok(),
                enabled: read_registry_dword(keyboard_key, KEYBOARD_ENABLED_VALUE).unwrap_or(1) != 0,
                hash: read_registry_string(keyboard_key, KEYBOARD_HASH_VALUE).ok(),
            };
            
            keyboards.push(keyboard);
            unsafe { let _ = RegCloseKey(keyboard_key); }
        }
    }
    
    unsafe { let _ = RegCloseKey(keyboards_key); }
    Ok(keyboards)
}

/// Saves a keyboard to registry
pub fn save_keyboard(keyboard: &RegistryKeyboard) -> Result<(), RegistryError> {
    let keyboard_key_path = format!("{}\\{}", KEYBOARDS_KEY, keyboard.id);
    let keyboard_key = create_or_open_registry_key(&keyboard_key_path)?;
    
    write_registry_string(keyboard_key, KEYBOARD_PATH_VALUE, &keyboard.path)?;
    write_registry_string(keyboard_key, KEYBOARD_NAME_VALUE, &keyboard.name)?;
    write_registry_string(keyboard_key, KEYBOARD_DESCRIPTION_VALUE, &keyboard.description)?;
    
    if let Some(hotkey) = &keyboard.hotkey {
        write_registry_string(keyboard_key, KEYBOARD_HOTKEY_VALUE, hotkey)?;
    }
    
    if let Some(color) = &keyboard.color {
        write_registry_string(keyboard_key, KEYBOARD_COLOR_VALUE, color)?;
    }
    
    if let Some(hash) = &keyboard.hash {
        write_registry_string(keyboard_key, KEYBOARD_HASH_VALUE, hash)?;
    }
    
    write_registry_dword(keyboard_key, KEYBOARD_ENABLED_VALUE, if keyboard.enabled { 1 } else { 0 })?;
    
    unsafe { let _ = RegCloseKey(keyboard_key); }
    Ok(())
}

/// Removes a keyboard from registry
pub fn remove_keyboard(keyboard_id: &str) -> Result<(), RegistryError> {
    let keyboards_key = open_registry_key(KEYBOARDS_KEY)?;
    delete_registry_key(keyboards_key, keyboard_id)?;
    unsafe { let _ = RegCloseKey(keyboards_key); }
    Ok(())
}

// ===== Settings Management =====

/// Gets the active keyboard ID
pub fn get_active_keyboard() -> Result<Option<String>, RegistryError> {
    match open_registry_key(SETTINGS_KEY) {
        Ok(settings_key) => {
            let result = read_registry_string(settings_key, DEFAULT_KEYBOARD_VALUE).ok();
            unsafe { let _ = RegCloseKey(settings_key); }
            Ok(result)
        }
        Err(RegistryError::KeyNotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Sets the active keyboard ID
pub fn set_active_keyboard(keyboard_id: Option<&str>) -> Result<(), RegistryError> {
    let settings_key = create_or_open_registry_key(SETTINGS_KEY)?;
    
    if let Some(id) = keyboard_id {
        write_registry_string(settings_key, DEFAULT_KEYBOARD_VALUE, id)?;
    } else {
        delete_registry_value(settings_key, DEFAULT_KEYBOARD_VALUE)?;
    }
    
    unsafe { let _ = RegCloseKey(settings_key); }
    Ok(())
}

/// Gets whether key processing is enabled
pub fn get_key_processing_enabled() -> Result<bool, RegistryError> {
    match open_registry_key(SETTINGS_KEY) {
        Ok(settings_key) => {
            let enabled = read_registry_dword(settings_key, KEY_PROCESSING_ENABLED_VALUE)
                .unwrap_or(0) != 0;  // Default to disabled
            unsafe { let _ = RegCloseKey(settings_key); }
            Ok(enabled)
        }
        Err(RegistryError::KeyNotFound) => Ok(false), // Default to disabled
        Err(e) => Err(e),
    }
}

/// Sets whether key processing is enabled
pub fn set_key_processing_enabled(enabled: bool) -> Result<(), RegistryError> {
    let settings_key = create_or_open_registry_key(SETTINGS_KEY)?;
    write_registry_dword(settings_key, KEY_PROCESSING_ENABLED_VALUE, if enabled { 1 } else { 0 })?;
    unsafe { let _ = RegCloseKey(settings_key); }
    Ok(())
}

/// Gets the on/off hotkey
pub fn get_on_off_hotkey() -> Result<Option<String>, RegistryError> {
    match open_registry_key(SETTINGS_KEY) {
        Ok(settings_key) => {
            let result = read_registry_string(settings_key, ON_OFF_HOTKEY_VALUE).ok();
            unsafe { let _ = RegCloseKey(settings_key); }
            Ok(result)
        }
        Err(RegistryError::KeyNotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Sets the on/off hotkey
pub fn set_on_off_hotkey(hotkey: Option<&str>) -> Result<(), RegistryError> {
    let settings_key = create_or_open_registry_key(SETTINGS_KEY)?;
    
    if let Some(hk) = hotkey {
        write_registry_string(settings_key, ON_OFF_HOTKEY_VALUE, hk)?;
    } else {
        delete_registry_value(settings_key, ON_OFF_HOTKEY_VALUE)?;
    }
    
    unsafe { let _ = RegCloseKey(settings_key); }
    Ok(())
}

/// Gets a generic setting value
pub fn get_setting(key: &str) -> Result<Option<String>, RegistryError> {
    match open_registry_key(SETTINGS_KEY) {
        Ok(settings_key) => {
            let result = read_registry_string(settings_key, key).ok();
            unsafe { let _ = RegCloseKey(settings_key); }
            Ok(result)
        }
        Err(RegistryError::KeyNotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Sets a generic setting value
pub fn set_setting(key: &str, value: Option<&str>) -> Result<(), RegistryError> {
    let settings_key = create_or_open_registry_key(SETTINGS_KEY)?;
    
    if let Some(val) = value {
        write_registry_string(settings_key, key, val)?;
    } else {
        delete_registry_value(settings_key, key)?;
    }
    
    unsafe { let _ = RegCloseKey(settings_key); }
    Ok(())
}

/// Gets a generic DWORD setting value
pub fn get_setting_dword(key: &str) -> Result<Option<u32>, RegistryError> {
    match open_registry_key(SETTINGS_KEY) {
        Ok(settings_key) => {
            let result = read_registry_dword(settings_key, key).ok();
            unsafe { let _ = RegCloseKey(settings_key); }
            Ok(result)
        }
        Err(RegistryError::KeyNotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Sets a generic DWORD setting value
pub fn set_setting_dword(key: &str, value: Option<u32>) -> Result<(), RegistryError> {
    let settings_key = create_or_open_registry_key(SETTINGS_KEY)?;
    
    if let Some(val) = value {
        write_registry_dword(settings_key, key, val)?;
    } else {
        delete_registry_value(settings_key, key)?;
    }
    
    unsafe { let _ = RegCloseKey(settings_key); }
    Ok(())
}

// ===== Autostart Management =====

/// Checks if autostart is enabled
pub fn is_autostart_enabled() -> Result<bool, RegistryError> {
    match open_registry_key(WINDOWS_RUN_KEY) {
        Ok(run_key) => {
            let result = read_registry_string(run_key, KEYMAGIC_AUTOSTART_VALUE).is_ok();
            unsafe { let _ = RegCloseKey(run_key); }
            Ok(result)
        }
        Err(_) => Ok(false),
    }
}

/// Sets autostart enabled/disabled
pub fn set_autostart_enabled(enabled: bool, exe_path: Option<&PathBuf>) -> Result<(), RegistryError> {
    let run_key = match create_or_open_registry_key(WINDOWS_RUN_KEY) {
        Ok(key) => key,
        Err(_) => return Err(RegistryError::OpenKeyFailed(WINDOWS_RUN_KEY.to_string())),
    };
    
    if enabled {
        if let Some(path) = exe_path {
            let path_str = path.to_string_lossy();
            write_registry_string(run_key, KEYMAGIC_AUTOSTART_VALUE, &path_str)?;
        } else {
            return Err(RegistryError::WriteValueFailed("No executable path provided".to_string()));
        }
    } else {
        delete_registry_value(run_key, KEYMAGIC_AUTOSTART_VALUE)?;
    }
    
    unsafe { let _ = RegCloseKey(run_key); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_structure() {
        // Test creating registry structure
        assert!(ensure_registry_structure().is_ok());
    }

    #[test]
    fn test_keyboard_operations() {
        // Ensure structure exists
        ensure_registry_structure().unwrap();

        // Test saving a keyboard
        let keyboard = RegistryKeyboard {
            id: "test-keyboard".to_string(),
            path: "C:\\test\\keyboard.km2".to_string(),
            name: "Test Keyboard".to_string(),
            description: "A test keyboard".to_string(),
            hotkey: Some("Ctrl+Shift+T".to_string()),
            color: Some("#FF0000".to_string()),
            enabled: true,
        };

        assert!(save_keyboard(&keyboard).is_ok());

        // Test loading keyboards
        let keyboards = load_keyboards().unwrap();
        assert!(keyboards.iter().any(|k| k.id == "test-keyboard"));

        // Test removing keyboard
        assert!(remove_keyboard("test-keyboard").is_ok());
    }

    #[test]
    fn test_settings_operations() {
        // Ensure structure exists
        ensure_registry_structure().unwrap();

        // Test active keyboard
        assert!(set_active_keyboard(Some("test-kb")).is_ok());
        assert_eq!(get_active_keyboard().unwrap(), Some("test-kb".to_string()));

        // Test key processing enabled
        assert!(set_key_processing_enabled(false).is_ok());
        assert_eq!(get_key_processing_enabled().unwrap(), false);

        // Test on/off hotkey
        assert!(set_on_off_hotkey(Some("Ctrl+Space")).is_ok());
        assert_eq!(get_on_off_hotkey().unwrap(), Some("Ctrl+Space".to_string()));

        // Test generic settings
        assert!(set_setting("TestSetting", Some("TestValue")).is_ok());
        assert_eq!(get_setting("TestSetting").unwrap(), Some("TestValue".to_string()));
    }
}