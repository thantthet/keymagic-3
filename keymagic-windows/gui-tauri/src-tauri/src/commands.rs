use crate::keyboard_manager::{KeyboardManager, KeyboardInfo};
use crate::hotkey::HotkeyManager;
use crate::updater::{UpdateInfo, check_for_updates_async};
use crate::autostart;
use crate::registry;
use std::sync::Mutex;
use tauri::{State, Manager, AppHandle};
use std::path::PathBuf;

type KeyboardManagerState = Mutex<KeyboardManager>;

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

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
    app_handle: AppHandle,
    keyboard_id: String,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    let result = manager.set_active_keyboard(&keyboard_id).map_err(|e| e.to_string())?;
    
    // Update tray icon
    crate::tray::update_tray_icon(&app_handle, &manager);
    
    Ok(result)
}

#[tauri::command]
pub fn validate_keyboards(
    state: State<KeyboardManagerState>,
    app_handle: AppHandle,
) -> Result<usize, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    
    // Get initial count for reporting
    let initial_count = manager.get_keyboards().len();
    
    // Run validation and cleanup
    manager.validate_and_cleanup().map_err(|e| e.to_string())?;
    
    // Get final count
    let final_count = manager.get_keyboards().len();
    let removed_count = initial_count.saturating_sub(final_count);
    
    // Update tray icon if keyboards were removed
    if removed_count > 0 {
        crate::tray::update_tray_icon(&app_handle, &manager);
    }
    
    Ok(removed_count)
}

#[tauri::command]
pub fn add_keyboard(
    state: State<KeyboardManagerState>,
    hotkey_manager: State<HotkeyManager>,
    app_handle: AppHandle,
    path: PathBuf,
) -> Result<String, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    let result = manager.load_keyboard(&path).map_err(|e| e.to_string())?;
    
    // Refresh hotkeys
    if let Err(e) = hotkey_manager.refresh_hotkeys(&app_handle, &manager) {
        eprintln!("Failed to refresh hotkeys: {}", e);
    }
    
    Ok(result)
}

#[tauri::command]
pub fn remove_keyboard(
    state: State<KeyboardManagerState>,
    hotkey_manager: State<HotkeyManager>,
    app_handle: AppHandle,
    keyboard_id: String,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.remove_keyboard(&keyboard_id).map_err(|e| e.to_string())?;
    
    // Refresh hotkeys
    if let Err(e) = hotkey_manager.refresh_hotkeys(&app_handle, &manager) {
        eprintln!("Failed to refresh hotkeys: {}", e);
    }
    
    Ok(())
}

#[tauri::command]
pub fn is_key_processing_enabled(state: State<KeyboardManagerState>) -> Result<bool, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    Ok(manager.is_key_processing_enabled())
}

#[tauri::command]
pub fn set_key_processing_enabled(
    state: State<KeyboardManagerState>,
    app_handle: AppHandle,
    enabled: bool,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    let result = manager.set_key_processing_enabled(enabled).map_err(|e| e.to_string())?;
    
    // Update tray icon
    crate::tray::update_tray_icon(&app_handle, &manager);
    
    Ok(result)
}

// Settings commands
#[tauri::command]
pub fn get_setting(key: String) -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        registry::get_setting(&key).map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(None)
}

#[tauri::command]
pub fn set_setting(key: String, value: String) -> Result<(), String> {
    // First save the setting to registry
    #[cfg(target_os = "windows")]
    {
        registry::set_setting(&key, Some(&value)).map_err(|e| e.to_string())?;
    }
    
    // Handle special settings
    if key == "StartWithWindows" {
        let enabled = value == "1";
        autostart::set_autostart(enabled)?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn set_keyboard_hotkey(
    state: State<KeyboardManagerState>,
    hotkey_manager: State<HotkeyManager>,
    app_handle: AppHandle,
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
    manager.set_keyboard_hotkey(&keyboard_id, hotkey_value).map_err(|e| e.to_string())?;
    
    // Refresh hotkeys and propagate any errors
    hotkey_manager.refresh_hotkeys(&app_handle, &manager)
        .map_err(|e| format!("Failed to register hotkey: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub fn update_tray_menu(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<KeyboardManagerState>();
    let manager = state.lock().map_err(|e| e.to_string())?;
    crate::tray::update_tray_menu(&app_handle, &manager);
    Ok(())
}

#[tauri::command]
pub fn set_on_off_hotkey(
    hotkey_manager: State<HotkeyManager>,
    app_handle: AppHandle,
    hotkey: Option<String>,
) -> Result<(), String> {
    // Set the hotkey
    hotkey_manager.set_on_off_hotkey(&app_handle, hotkey.as_deref()).map_err(|e| e.to_string())?;
    
    // Save to registry
    #[cfg(target_os = "windows")]
    {
        registry::set_on_off_hotkey(hotkey.as_deref()).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn get_on_off_hotkey() -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        registry::get_on_off_hotkey().map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(None)
}

#[tauri::command]
pub async fn check_for_update() -> Result<UpdateInfo, String> {
    check_for_updates_async().await.map_err(|e| e.to_string())
}