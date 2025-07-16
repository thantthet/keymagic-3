use crate::keyboard_manager::{KeyboardManager, KeyboardInfo, KeyboardComparison};
use crate::hotkey::HotkeyManager;
use crate::updater::{UpdateInfo, check_for_updates_async};
use crate::registry;
use std::sync::Mutex;
use tauri::{State, Manager, AppHandle};
use tauri_plugin_autostart::ManagerExt;
use std::path::PathBuf;
use log::error;

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
        error!("Failed to refresh hotkeys: {}", e);
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
        error!("Failed to refresh hotkeys: {}", e);
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
pub fn set_setting(app_handle: AppHandle, key: String, value: String) -> Result<(), String> {
    // First save the setting to registry
    #[cfg(target_os = "windows")]
    {
        registry::set_setting(&key, Some(&value)).map_err(|e| e.to_string())?;
    }
    
    // Handle special settings
    if key == "StartWithWindows" {
        let enabled = value == "1";
        let autostart_manager = app_handle.autolaunch();
        if enabled {
            autostart_manager.enable()
                .map_err(|e| format!("Failed to enable autostart: {}", e))?;
        } else {
            autostart_manager.disable()
                .map_err(|e| format!("Failed to disable autostart: {}", e))?;
        }
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

#[tauri::command]
pub fn check_first_run_scan_keyboards() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        registry::get_setting_dword("FirstRunScanKeyboards")
            .map(|val| val.map(|v| v == 1).unwrap_or(false))
            .map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(false)
}

#[tauri::command]
pub fn clear_first_run_scan_keyboards() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        registry::set_setting_dword("FirstRunScanKeyboards", None)
            .map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

#[tauri::command]
pub fn get_enabled_languages() -> Result<Vec<String>, String> {
    #[cfg(target_os = "windows")]
    {
        registry::get_enabled_languages().map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(vec!["en-US".to_string()])
}

#[tauri::command]
pub fn set_enabled_languages(languages: Vec<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        registry::set_enabled_languages(&languages).map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

#[tauri::command]
pub fn get_supported_languages() -> Result<Vec<(String, String)>, String> {
    Ok(vec![
        ("en-US".to_string(), "English (United States)".to_string()),
        ("my-MM".to_string(), "Myanmar".to_string()),
        ("th-TH".to_string(), "Thai".to_string()),
        ("km-KH".to_string(), "Khmer (Cambodia)".to_string()),
        ("lo-LA".to_string(), "Lao".to_string()),
        ("vi-VN".to_string(), "Vietnamese".to_string()),
        ("zh-CN".to_string(), "Chinese (Simplified)".to_string()),
        ("zh-TW".to_string(), "Chinese (Traditional)".to_string()),
        ("ja-JP".to_string(), "Japanese".to_string()),
        ("ko-KR".to_string(), "Korean".to_string()),
    ])
}

#[tauri::command]
pub fn get_bundled_keyboards(state: State<KeyboardManagerState>) -> Result<Vec<KeyboardComparison>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    
    // Get list of bundled keyboards
    let bundled = manager.get_bundled_keyboards()
        .map_err(|e| e.to_string())?;
    
    // Compare with installed keyboards
    manager.compare_with_bundled(bundled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_bundled_keyboard(
    state: State<KeyboardManagerState>,
    hotkey_manager: State<HotkeyManager>,
    app_handle: AppHandle,
    bundled_path: PathBuf,
    keyboard_status: String,
) -> Result<String, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    
    // Check if this is an update (keyboard with same name already exists)
    if keyboard_status == "Updated" {
        // First, read the bundled keyboard to get its name
        let km2_data = std::fs::read(&bundled_path)
            .map_err(|e| format!("Failed to read keyboard file: {}", e))?;
        let km2 = keymagic_core::km2::Km2Loader::load(&km2_data)
            .map_err(|e| format!("Failed to parse keyboard file: {}", e))?;
        let metadata = km2.metadata();
        
        let name = metadata.name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| bundled_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string());
        
        // Find existing keyboard with the same name
        if let Some(existing_keyboard) = manager.get_keyboard_by_name(&name).cloned() {
            // Remove the old keyboard
            manager.remove_keyboard(&existing_keyboard.id)
                .map_err(|e| format!("Failed to remove old keyboard: {}", e))?;
        }
    }
    
    // Load the new/updated keyboard
    let keyboard_id = manager.load_keyboard(&bundled_path)
        .map_err(|e| e.to_string())?;
    
    // Refresh hotkeys
    if let Err(e) = hotkey_manager.refresh_hotkeys(&app_handle, &manager) {
        error!("Failed to refresh hotkeys: {}", e);
    }
    
    // Update tray
    crate::tray::update_tray_icon(&app_handle, &manager);
    
    Ok(keyboard_id)
}

// Composition mode process list commands
#[tauri::command]
pub fn get_composition_mode_processes() -> Result<Vec<String>, String> {
    #[cfg(target_os = "windows")]
    {
        registry::get_composition_mode_processes().map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(vec![])
}

#[tauri::command]
pub fn set_composition_mode_processes(processes: Vec<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        registry::set_composition_mode_processes(&processes).map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

#[tauri::command]
pub fn add_composition_mode_process(process_name: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        registry::add_composition_mode_process(&process_name).map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

#[tauri::command]
pub fn remove_composition_mode_process(process_name: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        registry::remove_composition_mode_process(&process_name).map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(())
}