use crate::keyboard_manager::{KeyboardManager, KeyboardInfo, KeyboardComparison};
use crate::hotkey::HotkeyManager;
use crate::updater::{UpdateInfo, check_for_updates_async};
use crate::registry;
use std::sync::Mutex;
use tauri::{State, Manager, AppHandle};
use tauri_plugin_autostart::ManagerExt;
use std::path::PathBuf;
use log::error;
use std::process::Command;
use std::ffi::{CStr, CString};
use keymagic_core::ffi::*;

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
        // First update registry
        registry::set_enabled_languages(&languages).map_err(|e| e.to_string())?;
        
        // Try to update TSF language profiles directly first
        match crate::language_profiles::update_language_profiles(&languages) {
            Ok(_) => Ok(()),
            Err(e) => {
                // If it fails (likely due to permissions), return a special error
                Err(format!("ELEVATION_REQUIRED: {}", e))
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

#[tauri::command]
pub fn apply_language_changes_elevated(languages: Vec<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::env;
        
        // Get the path to our own executable
        let exe_path = env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;
        
        // Join languages with commas
        let languages_str = languages.join(",");
        
        // Launch elevated process with hidden window
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        let output = Command::new("powershell")
            .args(&[
                "-WindowStyle", "Hidden",
                "-Command",
                &format!(
                    "Start-Process '{}' -ArgumentList '--update-languages','{}' -Verb RunAs -Wait -WindowStyle Hidden",
                    exe_path.display(),
                    languages_str
                ),
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to launch elevated process: {}", e))?;
        
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("The operation was canceled by the user") {
                Err("ELEVATION_CANCELLED".to_string())
            } else {
                Err(format!("Failed to apply language changes: {}", stderr))
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

#[tauri::command]
pub fn get_supported_languages() -> Result<Vec<(String, String)>, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(crate::windows_languages::get_all_languages())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Fallback for non-Windows platforms
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
}

#[tauri::command]
pub fn search_languages(query: String) -> Result<Vec<(String, String)>, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(crate::windows_languages::search_languages(&query))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For non-Windows, just filter the default list
        let all_languages = get_supported_languages()?;
        let query_lower = query.to_lowercase();
        Ok(all_languages.into_iter()
            .filter(|(code, name)| {
                code.to_lowercase().contains(&query_lower) || 
                name.to_lowercase().contains(&query_lower)
            })
            .collect())
    }
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

#[tauri::command]
pub fn run_command(command: String, args: Vec<String>) -> Result<(), String> {
    Command::new(command)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to run command: {}", e))?;
    Ok(())
}

use std::collections::HashMap;

#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Debug, Clone, serde::Serialize)]
pub struct KeyboardLayoutData {
    pub keyboard_name: String,
    pub keys: HashMap<String, KeyData>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct KeyData {
    pub unshifted: String,
    pub shifted: String,
}

#[tauri::command]
pub fn get_keyboard_layout(
    state: State<KeyboardManagerState>,
    keyboard_id: String,
) -> Result<KeyboardLayoutData, String> {
    #[cfg(not(target_os = "windows"))]
    {
        return Err("Keyboard layout viewer is only available on Windows".to_string());
    }
    
    #[cfg(target_os = "windows")]
    {
    let manager = state.lock().map_err(|e| e.to_string())?;
    
    // Get keyboard info
    let keyboards = manager.get_keyboards();
    let keyboard = keyboards
        .iter()
        .find(|kb| kb.id == keyboard_id)
        .ok_or_else(|| "Keyboard not found".to_string())?;
    
    // Create a new engine instance for testing
    let engine = keymagic_engine_new();
    if engine.is_null() {
        return Err("Failed to create engine".to_string());
    }
    
    // Load the keyboard
    let c_path = CString::new(keyboard.path.to_str().unwrap()).map_err(|e| e.to_string())?;
    let load_result = keymagic_engine_load_keyboard(engine, c_path.as_ptr());
    
    if load_result != KeyMagicResult::Success {
        keymagic_engine_free(engine);
        return Err("Failed to load keyboard".to_string());
    }
    
    // Test all standard keys
    let mut keys = HashMap::new();
    
    // Define all standard US QWERTY keys with their VK codes and positions
    let standard_keys = vec![
        // Number row
        ("Backquote", VK_OEM_3.0 as i32, '`', '~'),
        ("Digit1", VK_1.0 as i32, '1', '!'),
        ("Digit2", VK_2.0 as i32, '2', '@'),
        ("Digit3", VK_3.0 as i32, '3', '#'),
        ("Digit4", VK_4.0 as i32, '4', '$'),
        ("Digit5", VK_5.0 as i32, '5', '%'),
        ("Digit6", VK_6.0 as i32, '6', '^'),
        ("Digit7", VK_7.0 as i32, '7', '&'),
        ("Digit8", VK_8.0 as i32, '8', '*'),
        ("Digit9", VK_9.0 as i32, '9', '('),
        ("Digit0", VK_0.0 as i32, '0', ')'),
        ("Minus", VK_OEM_MINUS.0 as i32, '-', '_'),
        ("Equal", VK_OEM_PLUS.0 as i32, '=', '+'),
        
        // Top row (QWERTY)
        ("KeyQ", VK_Q.0 as i32, 'q', 'Q'),
        ("KeyW", VK_W.0 as i32, 'w', 'W'),
        ("KeyE", VK_E.0 as i32, 'e', 'E'),
        ("KeyR", VK_R.0 as i32, 'r', 'R'),
        ("KeyT", VK_T.0 as i32, 't', 'T'),
        ("KeyY", VK_Y.0 as i32, 'y', 'Y'),
        ("KeyU", VK_U.0 as i32, 'u', 'U'),
        ("KeyI", VK_I.0 as i32, 'i', 'I'),
        ("KeyO", VK_O.0 as i32, 'o', 'O'),
        ("KeyP", VK_P.0 as i32, 'p', 'P'),
        ("BracketLeft", VK_OEM_4.0 as i32, '[', '{'),
        ("BracketRight", VK_OEM_6.0 as i32, ']', '}'),
        ("Backslash", VK_OEM_5.0 as i32, '\\', '|'),
        
        // Home row
        ("KeyA", VK_A.0 as i32, 'a', 'A'),
        ("KeyS", VK_S.0 as i32, 's', 'S'),
        ("KeyD", VK_D.0 as i32, 'd', 'D'),
        ("KeyF", VK_F.0 as i32, 'f', 'F'),
        ("KeyG", VK_G.0 as i32, 'g', 'G'),
        ("KeyH", VK_H.0 as i32, 'h', 'H'),
        ("KeyJ", VK_J.0 as i32, 'j', 'J'),
        ("KeyK", VK_K.0 as i32, 'k', 'K'),
        ("KeyL", VK_L.0 as i32, 'l', 'L'),
        ("Semicolon", VK_OEM_1.0 as i32, ';', ':'),
        ("Quote", VK_OEM_7.0 as i32, '\'', '"'),
        
        // Bottom row
        ("KeyZ", VK_Z.0 as i32, 'z', 'Z'),
        ("KeyX", VK_X.0 as i32, 'x', 'X'),
        ("KeyC", VK_C.0 as i32, 'c', 'C'),
        ("KeyV", VK_V.0 as i32, 'v', 'V'),
        ("KeyB", VK_B.0 as i32, 'b', 'B'),
        ("KeyN", VK_N.0 as i32, 'n', 'N'),
        ("KeyM", VK_M.0 as i32, 'm', 'M'),
        ("Comma", VK_OEM_COMMA.0 as i32, ',', '<'),
        ("Period", VK_OEM_PERIOD.0 as i32, '.', '>'),
        ("Slash", VK_OEM_2.0 as i32, '/', '?'),
        
        // Space bar
        ("Space", VK_SPACE.0 as i32, ' ', ' '),
    ];
    
    // Test each key
    for (key_id, vk_code, unshifted_char, shifted_char) in standard_keys {
        let mut key_data = KeyData {
            unshifted: String::new(),
            shifted: String::new(),
        };
        
        // Test unshifted state
        let mut output = ProcessKeyOutput {
            action_type: 0,
            text: std::ptr::null_mut(),
            delete_count: 0,
            composing_text: std::ptr::null_mut(),
            is_processed: 0,
        };
        
        let result = keymagic_engine_process_key_test_win(
            engine,
            vk_code,
            unshifted_char as i8,
            0, // shift
            0, // ctrl
            0, // alt
            0, // caps_lock
            &mut output,
        );
        
        if result == KeyMagicResult::Success {
            if output.is_processed != 0 && !output.composing_text.is_null() {
                let composing = unsafe { CStr::from_ptr(output.composing_text) }
                    .to_string_lossy()
                    .to_string();
                key_data.unshifted = composing;
            } else {
                // Key not processed, use default character
                key_data.unshifted = unshifted_char.to_string();
            }
            
            // Free strings
            if !output.text.is_null() {
                keymagic_free_string(output.text);
            }
            if !output.composing_text.is_null() {
                keymagic_free_string(output.composing_text);
            }
        }
        
        // Test shifted state
        let mut output = ProcessKeyOutput {
            action_type: 0,
            text: std::ptr::null_mut(),
            delete_count: 0,
            composing_text: std::ptr::null_mut(),
            is_processed: 0,
        };
        
        let result = keymagic_engine_process_key_test_win(
            engine,
            vk_code,
            shifted_char as i8,
            1, // shift
            0, // ctrl
            0, // alt
            0, // caps_lock
            &mut output,
        );
        
        if result == KeyMagicResult::Success {
            if output.is_processed != 0 && !output.composing_text.is_null() {
                let composing = unsafe { CStr::from_ptr(output.composing_text) }
                    .to_string_lossy()
                    .to_string();
                key_data.shifted = composing;
            } else {
                // Key not processed, use default character
                key_data.shifted = shifted_char.to_string();
            }
            
            // Free strings
            if !output.text.is_null() {
                keymagic_free_string(output.text);
            }
            if !output.composing_text.is_null() {
                keymagic_free_string(output.composing_text);
            }
        }
        
        keys.insert(key_id.to_string(), key_data);
    }
    
    // Clean up
    keymagic_engine_free(engine);
    
    Ok(KeyboardLayoutData {
        keyboard_name: keyboard.name.clone(),
        keys,
    })
    }
}
