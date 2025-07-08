use crate::keyboard_manager::{KeyboardManager, KeyboardInfo};
use std::sync::Mutex;
use tauri::{State, Manager, AppHandle};
use std::path::PathBuf;

type KeyboardManagerState = Mutex<KeyboardManager>;

#[tauri::command]
pub fn get_keyboards(state: State<KeyboardManagerState>) -> Result<Vec<KeyboardInfo>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_keyboards().into_iter().cloned().collect())
}

#[tauri::command]
pub fn get_active_keyboard(state: State<KeyboardManagerState>) -> Result<Option<String>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_active_keyboard().map(|s| s.to_string()))
}

#[tauri::command]
pub fn set_active_keyboard(
    state: State<KeyboardManagerState>,
    keyboard_id: String,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.set_active_keyboard(&keyboard_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_keyboard(
    state: State<KeyboardManagerState>,
    path: PathBuf,
) -> Result<String, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.load_keyboard(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_keyboard(
    state: State<KeyboardManagerState>,
    keyboard_id: String,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.remove_keyboard(&keyboard_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_key_processing_enabled(state: State<KeyboardManagerState>) -> Result<bool, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.is_key_processing_enabled())
}

#[tauri::command]
pub fn set_key_processing_enabled(
    state: State<KeyboardManagerState>,
    enabled: bool,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.set_key_processing_enabled(enabled).map_err(|e| e.to_string())
}

// Settings commands
#[tauri::command]
pub fn get_setting(key: String) -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        use windows::core::*;
        use windows::Win32::System::Registry::*;
        
        unsafe {
            let settings_path = format!("Software\\KeyMagic\\Settings\\{}", key);
            let wide_path: Vec<u16> = settings_path.encode_utf16().chain(std::iter::once(0)).collect();
            
            let mut hkey = HKEY::default();
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(wide_path.as_ptr()),
                0,
                KEY_READ,
                &mut hkey
            ).is_ok() {
                let mut buffer = vec![0u16; 256];
                let mut size = buffer.len() as u32 * 2;
                let mut data_type = REG_VALUE_TYPE::default();
                
                let result = RegQueryValueExW(
                    hkey,
                    w!(""),
                    None,
                    Some(&mut data_type),
                    Some(buffer.as_mut_ptr() as *mut u8),
                    Some(&mut size),
                );
                
                RegCloseKey(hkey);
                
                if result.is_ok() {
                    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
                    buffer.truncate(len);
                    return Ok(Some(String::from_utf16_lossy(&buffer)));
                }
            }
        }
    }
    
    Ok(None)
}

#[tauri::command]
pub fn set_setting(key: String, value: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::core::*;
        use windows::Win32::System::Registry::*;
        
        unsafe {
            let settings_path = format!("Software\\KeyMagic\\Settings\\{}", key);
            let wide_path: Vec<u16> = settings_path.encode_utf16().chain(std::iter::once(0)).collect();
            
            let mut hkey = HKEY::default();
            if RegCreateKeyW(
                HKEY_CURRENT_USER,
                PCWSTR(wide_path.as_ptr()),
                &mut hkey
            ).is_ok() {
                let value_w: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
                let value_bytes = std::slice::from_raw_parts(
                    value_w.as_ptr() as *const u8,
                    value_w.len() * 2
                );
                
                let result = RegSetValueExW(
                    hkey,
                    w!(""),
                    0,
                    REG_SZ,
                    Some(value_bytes),
                );
                
                RegCloseKey(hkey);
                
                if result.is_err() {
                    return Err("Failed to write registry value".to_string());
                }
            } else {
                return Err("Failed to create registry key".to_string());
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn set_keyboard_hotkey(
    state: State<KeyboardManagerState>,
    keyboard_id: String,
    hotkey: Option<String>,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    // Convert empty string to Some("") to distinguish from None
    let hotkey_value = match hotkey.as_deref() {
        Some("") => Some(""),       // Explicitly no hotkey
        Some(s) => Some(s),         // Custom hotkey
        None => None,               // Use default
    };
    manager.set_keyboard_hotkey(&keyboard_id, hotkey_value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tray_menu(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<KeyboardManagerState>();
    let manager = state.lock().map_err(|e| e.to_string())?;
    crate::tray::update_tray_menu(&app_handle, &manager);
    Ok(())
}