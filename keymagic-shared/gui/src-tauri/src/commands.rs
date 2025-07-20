use crate::core::{KeyboardInfo, KeyboardManager};
use crate::platform::{Language, PlatformInfo};
use keymagic_core::{KeyInput, VirtualKey};
use keymagic_core::engine::ModifierState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

pub type AppState = Arc<KeyboardManager>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub release_notes: String,
}

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
pub fn get_platform_info(_state: State<AppState>) -> Result<PlatformInfo, String> {
    // Access platform through a temporary manager instance
    // In a real implementation, we'd store the platform reference separately
    Ok(PlatformInfo {
        os: std::env::consts::OS.to_string(),
        features: crate::platform::PlatformFeatures {
            language_profiles: cfg!(windows),
            composition_mode: true,
            global_hotkeys: true,
            system_tray: true,
        },
    })
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
    state: State<AppState>,
    keyboard_id: String,
) -> Result<(), String> {
    state
        .set_active_keyboard(&keyboard_id)
        .map_err(|e| e.to_string())
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
    state: State<AppState>,
    file_path: PathBuf,
) -> Result<KeyboardInfo, String> {
    state
        .import_keyboard(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_keyboard(
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
    state
        .update_hotkey(&keyboard_id, hotkey)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_system_languages(_state: State<AppState>) -> Result<Vec<Language>, String> {
    // This would need access to the platform instance
    // For now, return a basic set
    Ok(vec![
        Language {
            id: "en".to_string(),
            name: "English".to_string(),
            code: "en".to_string(),
        },
        Language {
            id: "my".to_string(),
            name: "Myanmar".to_string(),
            code: "my".to_string(),
        },
    ])
}

#[tauri::command]
pub fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
    // TODO: Implement update checking
    Ok(None)
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
pub fn get_composition_mode_processes(_state: State<AppState>) -> Result<Vec<String>, String> {
    // This would need access to the platform config
    // For now, return a default set
    Ok(vec![
        "firefox".to_string(),
        "chrome".to_string(),
        "code".to_string(),
    ])
}

#[tauri::command]
pub fn set_composition_mode_processes(
    _state: State<AppState>,
    _processes: Vec<String>,
) -> Result<(), String> {
    // This would update the platform config
    // TODO: Implement when we have proper config access
    Ok(())
}

#[tauri::command]
pub fn set_start_with_system(
    _state: State<AppState>,
    _enabled: bool,
) -> Result<(), String> {
    // This would update the platform config and autostart
    // TODO: Implement when we have proper config access
    Ok(())
}

#[tauri::command]
pub fn get_start_with_system(_state: State<AppState>) -> Result<bool, String> {
    // This would read from platform config
    // TODO: Implement when we have proper config access
    Ok(false)
}

#[tauri::command]
pub fn update_tray_menu(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::tray::update_tray_menu(&app_handle)
        .map_err(|e| e.to_string())
}

// Version info
#[tauri::command]
pub fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

// Import wizard commands
#[tauri::command]
pub fn check_first_run_scan_keyboards(state: State<AppState>) -> Result<bool, String> {
    state.get_platform()
        .is_first_run()
        .map_err(|e| e.to_string())
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
                        
                        // Check if already installed
                        let status = if installed_keyboards.iter().any(|k| k.id == id) {
                            "Installed"
                        } else {
                            "New"
                        }.to_string();
                        
                        bundled_keyboards.push(BundledKeyboard {
                            id: id.clone(),
                            name: id, // Will be updated when loaded
                            status,
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
pub fn clear_first_run_scan_keyboards(state: State<AppState>) -> Result<(), String> {
    state.get_platform()
        .clear_first_run_flag()
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

// Composition mode process management
#[tauri::command]
pub fn add_composition_mode_process(
    state: State<AppState>,
    process_name: String,
) -> Result<(), String> {
    let mut config = state.get_config();
    
    // Add process if not already in list
    if !config.composition_mode.enabled_processes.contains(&process_name) {
        config.composition_mode.enabled_processes.push(process_name);
        state.save_config(&config).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn remove_composition_mode_process(
    state: State<AppState>,
    process_name: String,
) -> Result<(), String> {
    let mut config = state.get_config();
    
    // Remove process from list
    config.composition_mode.enabled_processes.retain(|p| p != &process_name);
    state.save_config(&config).map_err(|e| e.to_string())?;
    
    Ok(())
}

// Language profile commands (Windows-specific features)
#[tauri::command]
pub fn get_supported_languages(_state: State<AppState>) -> Result<Vec<(String, String)>, String> {
    // TODO: Return list of supported languages
    // For now, return a basic set
    Ok(vec![
        ("en".to_string(), "English".to_string()),
        ("my".to_string(), "Myanmar (Burmese)".to_string()),
        ("zh".to_string(), "Chinese".to_string()),
        ("ja".to_string(), "Japanese".to_string()),
        ("ko".to_string(), "Korean".to_string()),
        ("th".to_string(), "Thai".to_string()),
    ])
}

#[tauri::command]
pub fn get_enabled_languages(_state: State<AppState>) -> Result<Vec<String>, String> {
    // TODO: Return list of currently enabled language codes
    Ok(vec!["en".to_string()])
}

#[tauri::command]
pub fn search_languages(_state: State<AppState>, query: String) -> Result<Vec<(String, String)>, String> {
    // TODO: Implement language search
    // For now, just filter the basic set
    let all_languages = vec![
        ("en", "English"),
        ("my", "Myanmar (Burmese)"),
        ("zh", "Chinese"),
        ("ja", "Japanese"),
        ("ko", "Korean"),
        ("th", "Thai"),
    ];
    
    let query_lower = query.to_lowercase();
    let results: Vec<(String, String)> = all_languages
        .into_iter()
        .filter(|(code, name)| {
            code.to_lowercase().contains(&query_lower) || 
            name.to_lowercase().contains(&query_lower)
        })
        .map(|(code, name)| (code.to_string(), name.to_string()))
        .collect();
    
    Ok(results)
}

#[tauri::command]
pub fn set_enabled_languages(
    _state: State<AppState>,
    _languages: Vec<String>,
) -> Result<(), String> {
    // TODO: Update enabled languages in system
    // This might require elevation on Windows
    Ok(())
}

#[tauri::command]
pub fn apply_language_changes_elevated(
    _state: State<AppState>,
    _languages: Vec<String>,
) -> Result<(), String> {
    // TODO: Apply language changes with elevation
    // This is Windows-specific
    #[cfg(target_os = "windows")]
    {
        // Would need to launch elevated process
        return Err("ELEVATION_REQUIRED".to_string());
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        return Err("This feature is only available on Windows".to_string());
    }
}

// Update checking - return the existing UpdateInfo type
#[tauri::command]
pub fn check_for_update() -> Result<Option<UpdateInfo>, String> {
    // TODO: Implement actual update checking
    // For now, return None (no update available)
    Ok(None)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundledKeyboard {
    pub id: String,
    pub name: String,
    pub status: String, // "New", "Updated", "Installed"
}