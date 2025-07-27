use crate::core::{KeyboardInfo, KeyboardManager};
use crate::hotkey::HotkeyManager;
use crate::platform::PlatformInfo;
use keymagic_core::{KeyInput, VirtualKey};
use keymagic_core::engine::ModifierState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

pub type AppState = Arc<KeyboardManager>;

// Re-export UpdateInfo from updater module
pub use crate::updater::UpdateInfo;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyMapping {
    pub shifted: Option<String>,
    pub unshifted: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyboardLayoutData {
    pub keyboard_name: String,
    pub keyboard_id: String,
    pub keys: HashMap<String, KeyMapping>,
}

#[tauri::command]
pub fn get_platform_info(state: State<AppState>) -> Result<PlatformInfo, String> {
    // Get platform info from the keyboard manager
    Ok(state.get_platform_info())
}

#[tauri::command]
pub fn get_keyboards(state: State<AppState>) -> Result<Vec<KeyboardInfo>, String> {
    Ok(state.get_keyboards())
}

#[tauri::command]
pub fn get_active_keyboard(state: State<AppState>) -> Result<Option<String>, String> {
    Ok(state.get_active_keyboard())
}

#[tauri::command]
pub fn set_active_keyboard(
    app: AppHandle,
    state: State<AppState>,
    keyboard_id: String,
) -> Result<(), String> {
    state
        .set_active_keyboard(&keyboard_id)
        .map_err(|e| e.to_string())?;
    
    
    // Emit event to notify all UI components
    let _ = app.emit("active_keyboard_changed", &keyboard_id);
    
    Ok(())
}

#[tauri::command]
pub fn scan_keyboards(state: State<AppState>) -> Result<Vec<KeyboardInfo>, String> {
    state.scan_keyboards().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_keyboard_layout(
    state: State<AppState>,
    keyboard_id: String,
) -> Result<KeyboardLayoutData, String> {
    let keyboards = state.get_keyboards();
    let keyboard = keyboards
        .iter()
        .find(|k| k.id == keyboard_id)
        .ok_or_else(|| format!("Keyboard not found: {}", keyboard_id))?;

    // Load the keyboard file to get the actual engine
    let layout = state.load_keyboard_file(&keyboard.path)
        .map_err(|e| format!("Failed to load keyboard file: {}", e))?;
    
    // Create a temporary engine for this keyboard
    let mut engine = keymagic_core::KeyMagicEngine::new(layout)
        .map_err(|e| format!("Failed to create engine: {}", e))?;
    
    let mut keys = HashMap::new();
    
    // Helper function to get output for a key with specific modifiers
    let mut get_key_output = |vk: VirtualKey, modifiers: ModifierState, character: Option<char>| -> Option<String> {
        // Reset engine state before each test
        engine.reset();
        
        let input = KeyInput::new(vk as u16, modifiers, character);
        match engine.process_key_test(input) {
            Ok(output) => {
                if output.composing_text.is_empty() {
                    None
                } else {
                    Some(output.composing_text)
                }
            }
            Err(_) => None,
        }
    };

    // Define key mappings from VirtualKey to DOM key names with their default characters
    let key_mappings = [
        // Number row (unshifted, shifted characters)
        (VirtualKey::Oem3, "Backquote", '`', '~'),
        (VirtualKey::Key1, "Digit1", '1', '!'),
        (VirtualKey::Key2, "Digit2", '2', '@'),
        (VirtualKey::Key3, "Digit3", '3', '#'),
        (VirtualKey::Key4, "Digit4", '4', '$'),
        (VirtualKey::Key5, "Digit5", '5', '%'),
        (VirtualKey::Key6, "Digit6", '6', '^'),
        (VirtualKey::Key7, "Digit7", '7', '&'),
        (VirtualKey::Key8, "Digit8", '8', '*'),
        (VirtualKey::Key9, "Digit9", '9', '('),
        (VirtualKey::Key0, "Digit0", '0', ')'),
        (VirtualKey::OemMinus, "Minus", '-', '_'),
        (VirtualKey::OemPlus, "Equal", '=', '+'),
        
        // Top row (QWERTY)
        (VirtualKey::KeyQ, "KeyQ", 'q', 'Q'),
        (VirtualKey::KeyW, "KeyW", 'w', 'W'),
        (VirtualKey::KeyE, "KeyE", 'e', 'E'),
        (VirtualKey::KeyR, "KeyR", 'r', 'R'),
        (VirtualKey::KeyT, "KeyT", 't', 'T'),
        (VirtualKey::KeyY, "KeyY", 'y', 'Y'),
        (VirtualKey::KeyU, "KeyU", 'u', 'U'),
        (VirtualKey::KeyI, "KeyI", 'i', 'I'),
        (VirtualKey::KeyO, "KeyO", 'o', 'O'),
        (VirtualKey::KeyP, "KeyP", 'p', 'P'),
        (VirtualKey::Oem4, "BracketLeft", '[', '{'),
        (VirtualKey::Oem6, "BracketRight", ']', '}'),
        (VirtualKey::Oem5, "Backslash", '\\', '|'),
        
        // Home row (ASDF)
        (VirtualKey::KeyA, "KeyA", 'a', 'A'),
        (VirtualKey::KeyS, "KeyS", 's', 'S'),
        (VirtualKey::KeyD, "KeyD", 'd', 'D'),
        (VirtualKey::KeyF, "KeyF", 'f', 'F'),
        (VirtualKey::KeyG, "KeyG", 'g', 'G'),
        (VirtualKey::KeyH, "KeyH", 'h', 'H'),
        (VirtualKey::KeyJ, "KeyJ", 'j', 'J'),
        (VirtualKey::KeyK, "KeyK", 'k', 'K'),
        (VirtualKey::KeyL, "KeyL", 'l', 'L'),
        (VirtualKey::Oem1, "Semicolon", ';', ':'),
        (VirtualKey::Oem7, "Quote", '\'', '"'),
        
        // Bottom row (ZXCV)
        (VirtualKey::KeyZ, "KeyZ", 'z', 'Z'),
        (VirtualKey::KeyX, "KeyX", 'x', 'X'),
        (VirtualKey::KeyC, "KeyC", 'c', 'C'),
        (VirtualKey::KeyV, "KeyV", 'v', 'V'),
        (VirtualKey::KeyB, "KeyB", 'b', 'B'),
        (VirtualKey::KeyN, "KeyN", 'n', 'N'),
        (VirtualKey::KeyM, "KeyM", 'm', 'M'),
        (VirtualKey::OemComma, "Comma", ',', '<'),
        (VirtualKey::OemPeriod, "Period", '.', '>'),
        (VirtualKey::Oem2, "Slash", '/', '?'),
        
        // Space
        (VirtualKey::Space, "Space", ' ', ' '),
    ];

    // Generate key mappings using the engine
    for (vk, dom_key, unshifted_char, shifted_char) in key_mappings.iter() {
        let unshifted = get_key_output(*vk, ModifierState::new(false, false, false, false), Some(*unshifted_char));
        let shifted = get_key_output(*vk, ModifierState::new(true, false, false, false), Some(*shifted_char));
        
        keys.insert(dom_key.to_string(), KeyMapping {
            shifted,
            unshifted,
        });
    }

    Ok(KeyboardLayoutData {
        keyboard_name: keyboard.name.clone(),
        keyboard_id: keyboard.id.clone(),
        keys,
    })
}

#[tauri::command]
pub fn import_keyboard(
    app: AppHandle,
    state: State<AppState>,
    file_path: PathBuf,
) -> Result<KeyboardInfo, String> {
    let keyboard_info = state
        .import_keyboard(&file_path)
        .map_err(|e| e.to_string())?;
    
    
    Ok(keyboard_info)
}

#[tauri::command]
pub fn remove_keyboard(
    app: AppHandle,
    state: State<AppState>,
    keyboard_id: String,
) -> Result<(), String> {
    
    state
        .remove_keyboard(&keyboard_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_hotkey(
    state: State<AppState>,
    keyboard_id: String,
    hotkey: Option<String>,
) -> Result<(), String> {
    // Update the keyboard hotkey
    state
        .update_hotkey(&keyboard_id, hotkey)
        .map_err(|e| e.to_string())
}


#[tauri::command]
pub fn validate_hotkey(app: AppHandle, hotkey: String) -> Result<(), String> {
    // Empty hotkey is always valid
    if hotkey.is_empty() {
        return Ok(());
    }
    
    if let Some(hotkey_manager) = app.try_state::<Arc<HotkeyManager>>() {
        hotkey_manager
            .validate_hotkey(&hotkey)
            .map_err(|e| e.to_string())
    } else {
        Err("Hotkey manager not available".to_string())
    }
}



#[tauri::command]
pub async fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
    match crate::updater::check_for_updates_async().await {
        Ok(update_info) => Ok(Some(update_info)),
        Err(e) => {
            log::error!("Failed to check for updates: {}", e);
            // Return None instead of error to allow graceful degradation
            Ok(None)
        }
    }
}

#[tauri::command]
pub fn restart_app(app_handle: tauri::AppHandle) -> Result<(), String> {
    app_handle.restart();
}

#[tauri::command]
pub fn quit_app(app_handle: tauri::AppHandle) -> Result<(), String> {
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
pub fn open_keyboards_folder(_state: State<AppState>) -> Result<(), String> {
    // This would need access to the platform instance to get keyboards_dir
    // For now, we'll use a placeholder
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("C:\\ProgramData\\KeyMagic\\Keyboards")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg("/usr/share/keymagic/keyboards")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("/Library/Application Support/KeyMagic/Keyboards")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn get_composition_mode_hosts(state: State<AppState>) -> Result<Vec<String>, String> {
    let config = state.get_config();
    Ok(config.composition_mode.enabled_hosts.clone())
}

#[tauri::command]
pub fn set_composition_mode_hosts(
    state: State<AppState>,
    hosts: Vec<String>,
) -> Result<(), String> {
    let mut config = state.get_config();
    config.composition_mode.enabled_hosts = hosts;
    state.save_config(&config).map_err(|e| e.to_string())
}



// Version info
#[tauri::command]
pub fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

// Import wizard commands
#[tauri::command]
pub fn should_scan_bundled_keyboards(state: State<AppState>) -> Result<bool, String> {
    // Check if we need to scan bundled keyboards based on version
    let current_version = env!("CARGO_PKG_VERSION");
    
    let config = state.get_platform()
        .load_config()
        .map_err(|e| e.to_string())?;
    
    match &config.general.last_scanned_version {
        Some(last_version) => {
            // Use the compare_versions function from platform module
            Ok(crate::platform::compare_versions(current_version, last_version))
        }
        None => {
            // First run, should scan
            Ok(true)
        }
    }
}

#[tauri::command]
pub fn get_bundled_keyboards(state: State<AppState>) -> Result<Vec<BundledKeyboard>, String> {
    let platform = state.get_platform();
    let bundled_path = match platform.get_bundled_keyboards_path() {
        Some(path) => path,
        None => return Ok(vec![]),
    };
    
    let mut bundled_keyboards = Vec::new();
    let installed_keyboards = state.get_keyboards();
    
    if bundled_path.exists() {
        match std::fs::read_dir(&bundled_path) {
            Ok(entries) => {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("km2") {
                        let id = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        // Try to load the keyboard to get proper name, icon, and check for updates
                        let (name, icon_data, bundled_hash) = match state.load_keyboard_file(&path) {
                            Ok(layout) => {
                                let metadata = layout.metadata();
                                let name = metadata.name().unwrap_or(id.clone());
                                let icon_data = metadata.icon().map(|data| data.to_vec());
                                let hash = state.calculate_file_hash(&path).unwrap_or_default();
                                (name, icon_data, hash)
                            }
                            Err(_) => (id.clone(), None, String::new())
                        };
                        
                        // Check if already installed and compare hashes
                        let status = if let Some(installed) = installed_keyboards.iter().find(|k| k.name == name) {
                            if bundled_hash.is_empty() {
                                "Installed"  // Can't compare, assume installed
                            } else if installed.hash == bundled_hash {
                                "Unchanged"  // Same hash, up to date
                            } else {
                                // Check if the installed file was modified after installation
                                // For now, just mark as "Updated" (in real implementation, 
                                // we'd check if user modified the file)
                                "Updated"  // Hash mismatch means bundled version is newer
                            }
                        } else {
                            "New"
                        }.to_string();
                        
                        bundled_keyboards.push(BundledKeyboard {
                            id: id.clone(),
                            name,
                            status,
                            icon_data,
                            bundled_path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
            Err(_) => {}
        }
    }
    
    Ok(bundled_keyboards)
}

#[tauri::command]
pub fn import_bundled_keyboard(
    state: State<AppState>,
    bundled_path: String,
    keyboard_status: String,
    app_handle: tauri::AppHandle,
) -> Result<KeyboardInfo, String> {
    let keyboard_file = std::path::PathBuf::from(&bundled_path);
    if !keyboard_file.exists() {
        return Err(format!("Bundled keyboard file not found: {}", bundled_path));
    }
    
    // Check if this is an update (keyboard with same name already exists)
    if keyboard_status == "Updated" {
        // First, read the bundled keyboard to get its name
        match state.load_keyboard_file(&keyboard_file) {
            Ok(layout) => {
                let metadata = layout.metadata();
                let keyboard_id = keyboard_file.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let name = metadata.name().unwrap_or(keyboard_id);
                
                // Find existing keyboard with the same name
                if let Some(existing_keyboard) = state.get_keyboard_by_name(&name) {
                    // Remove the old keyboard
                    state.remove_keyboard(&existing_keyboard.id)
                        .map_err(|e| format!("Failed to remove old keyboard: {}", e))?;
                }
            }
            Err(e) => {
                return Err(format!("Failed to read bundled keyboard: {}", e));
            }
        }
    }
    
    // Import the new/updated keyboard
    let keyboard_info = state.import_keyboard(&keyboard_file)
        .map_err(|e| format!("Failed to import keyboard: {}", e))?;
    
    
    
    Ok(keyboard_info)
}

#[tauri::command]
pub fn mark_bundled_keyboards_scanned(state: State<AppState>) -> Result<(), String> {
    // Update the last scanned version to current version
    let mut config = state.get_platform()
        .load_config()
        .map_err(|e| e.to_string())?;
    
    config.general.last_scanned_version = Some(env!("CARGO_PKG_VERSION").to_string());
    
    state.get_platform()
        .save_config(&config)
        .map_err(|e| e.to_string())
}

// Settings commands
#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> Result<String, String> {
    Ok(state.get_platform()
        .get_setting(&key)
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "".to_string()))
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    state.get_platform()
        .set_setting(&key, &value)
        .map_err(|e| e.to_string())
}

// Update reminder commands
#[tauri::command]
pub fn get_update_remind_after(state: State<AppState>) -> Result<Option<String>, String> {
    let config = state.get_platform()
        .load_config()
        .map_err(|e| e.to_string())?;
    Ok(config.general.update_remind_after)
}

#[tauri::command]
pub fn set_update_remind_after(state: State<AppState>, value: Option<String>) -> Result<(), String> {
    let mut config = state.get_platform()
        .load_config()
        .map_err(|e| e.to_string())?;
    config.general.update_remind_after = value;
    state.get_platform()
        .save_config(&config)
        .map_err(|e| e.to_string())
}

// Process management (Windows-specific, but we'll make it work cross-platform)
#[tauri::command]
pub fn run_command(command: String, args: Vec<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(command)
            .args(args)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let _ = command;
        let _ = args;
        Err("This command is only available on Windows".to_string())
    }
}

// Composition mode host management
#[tauri::command]
pub fn add_composition_mode_host(
    state: State<AppState>,
    host_name: String,
) -> Result<(), String> {
    let mut config = state.get_config();
    
    // Add host if not already in list
    if !config.composition_mode.enabled_hosts.contains(&host_name) {
        config.composition_mode.enabled_hosts.push(host_name);
        state.save_config(&config).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn remove_composition_mode_host(
    state: State<AppState>,
    host_name: String,
) -> Result<(), String> {
    let mut config = state.get_config();
    
    // Remove host from list
    config.composition_mode.enabled_hosts.retain(|h| h != &host_name);
    state.save_config(&config).map_err(|e| e.to_string())?;
    
    Ok(())
}

// Direct mode host management
#[tauri::command]
pub fn get_direct_mode_hosts(state: State<AppState>) -> Result<Vec<String>, String> {
    let config = state.get_config();
    Ok(config.direct_mode.enabled_hosts.clone())
}

#[tauri::command]
pub fn add_direct_mode_host(
    state: State<AppState>,
    host_name: String,
) -> Result<(), String> {
    let mut config = state.get_config();
    
    // Add host if not already in list
    if !config.direct_mode.enabled_hosts.contains(&host_name) {
        config.direct_mode.enabled_hosts.push(host_name);
        state.save_config(&config).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn remove_direct_mode_host(
    state: State<AppState>,
    host_name: String,
) -> Result<(), String> {
    let mut config = state.get_config();
    
    // Remove host from list
    config.direct_mode.enabled_hosts.retain(|h| h != &host_name);
    state.save_config(&config).map_err(|e| e.to_string())?;
    
    Ok(())
}

// Language profile commands (Windows-specific features)
#[tauri::command]
pub fn get_supported_languages(_state: State<AppState>) -> Result<Vec<(String, String)>, String> {
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
            ("id-ID".to_string(), "Indonesian".to_string()),
            ("ms-MY".to_string(), "Malay".to_string()),
            ("fil-PH".to_string(), "Filipino".to_string()),
            ("zh-CN".to_string(), "Chinese (Simplified)".to_string()),
            ("zh-TW".to_string(), "Chinese (Traditional)".to_string()),
            ("ja-JP".to_string(), "Japanese".to_string()),
            ("ko-KR".to_string(), "Korean".to_string()),
        ])
    }
}

#[tauri::command]
#[allow(unused_variables)]
pub fn get_enabled_languages(state: State<AppState>) -> Result<Vec<String>, String> {
    #[cfg(target_os = "windows")]
    {
        state.get_platform()
            .get_enabled_languages()
            .map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(vec!["en-US".to_string()])
}

#[tauri::command]
pub fn search_languages(_state: State<AppState>, query: String) -> Result<Vec<(String, String)>, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(crate::windows_languages::search_languages(&query))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For non-Windows, just filter the default list
        let all_languages = get_supported_languages(_state)?;
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
#[allow(unused_variables)]
pub fn set_enabled_languages(
    state: State<AppState>,
    languages: Vec<String>,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // First update platform storage
        state.get_platform()
            .set_enabled_languages(&languages)
            .map_err(|e| e.to_string())?;
        
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
pub fn apply_language_changes_elevated(
    _state: State<AppState>,
    languages: Vec<String>,
) -> Result<(), String> {
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
            if stderr.contains("canceled") {
                Err("ELEVATION_CANCELLED".to_string())
            } else {
                Err(format!("Failed to apply language changes: {}", stderr))
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let _ = languages;
        Err("This feature is only available on Windows".to_string())
    }
}

// Update checking - legacy alias for check_for_updates
#[tauri::command]
pub async fn check_for_update() -> Result<Option<UpdateInfo>, String> {
    check_for_updates().await
}

// KMS to KM2 converter commands
#[tauri::command]
pub fn convert_kms_to_km2(
    input_path: String,
    output_path: String,
) -> Result<(), String> {
    let input = std::path::PathBuf::from(&input_path);
    let output = std::path::PathBuf::from(&output_path);
    
    // Ensure input file exists
    if !input.exists() {
        return Err(format!("Input file not found: {}", input_path));
    }
    
    // Ensure it's a file, not a directory
    if input.is_dir() {
        return Err(format!("Input path is a directory, not a file: {}", input_path));
    }
    
    // Ensure input has .kms extension
    if input.extension().and_then(|s| s.to_str()) != Some("kms") {
        return Err("Input file must have .kms extension".to_string());
    }
    
    // Convert using kms2km2 crate
    kms2km2::convert_kms_to_km2(&input, &output)
        .map_err(|e| format!("Conversion failed: {}", e))
}

#[tauri::command]
pub fn validate_kms_file(
    file_path: String,
) -> Result<String, String> {
    use std::fs;
    
    let path = std::path::PathBuf::from(&file_path);
    
    // Debug: Log the path
    log::info!("Validating KMS file at path: {:?}", path);
    
    // Ensure file exists
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    // Get file metadata
    let metadata = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return Err(format!("Failed to get file metadata: {}", e))
    };
    
    // Log file type
    log::info!("File type - is_file: {}, is_dir: {}, is_symlink: {}", 
        metadata.is_file(), 
        metadata.is_dir(), 
        metadata.file_type().is_symlink()
    );
    
    // Ensure it's a file, not a directory
    if metadata.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", file_path));
    }
    
    // Ensure it has .kms extension
    if path.extension().and_then(|s| s.to_str()) != Some("kms") {
        return Err("File must have .kms extension".to_string());
    }
    
    // Try to read the file content first to debug
    match fs::read_to_string(&path) {
        Ok(content) => {
            log::info!("Successfully read file, content length: {} bytes", content.len());
        }
        Err(e) => {
            return Err(format!("Failed to read file: {} (os error: {:?})", e, e.raw_os_error()));
        }
    }
    
    // Try to compile the KMS file to validate it
    match kms2km2::compile_kms_file(&path) {
        Ok(km2_file) => {
            // Extract metadata for validation result
            let metadata = km2_file.metadata();
            let name = metadata.name().unwrap_or("Unnamed Keyboard".to_string());
            let description = metadata.description().unwrap_or("No description".to_string());
            
            Ok(format!("Valid KMS file\nName: {}\nDescription: {}", name, description))
        }
        Err(e) => Err(format!("Invalid KMS file: {}", e))
    }
}

#[tauri::command]
pub fn convert_kms_file(
    input_path: String,
    output_path: String,
) -> Result<(), String> {
    // Use the existing convert_kms_to_km2 function
    convert_kms_to_km2(input_path, output_path)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundledKeyboard {
    pub id: String,
    pub name: String,
    pub status: String, // "New", "Updated", "Installed", "Unchanged", "Modified"
    pub icon_data: Option<Vec<u8>>,
    pub bundled_path: String,
}