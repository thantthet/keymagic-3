use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use anyhow::{Result, anyhow};
use log::error;

use crate::keyboard_manager::KeyboardManager;

/// Normalize hotkey string to consistent format
/// Examples: "ctrl+space" -> "Ctrl+Space", "CTRL + SHIFT + A" -> "Ctrl+Shift+A"
pub fn normalize_hotkey(hotkey: &str) -> String {
    // Split by common separators and filter out empty parts
    let parts: Vec<&str> = hotkey
        .split(|c| c == '+' || c == '-' || c == ' ')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    if parts.is_empty() {
        return String::new();
    }
    
    // Sort modifiers in consistent order: Ctrl, Shift, Alt, Win
    let mut modifiers = Vec::new();
    let mut main_keys = Vec::new();
    
    for part in parts {
        let normalized = normalize_key_part(part);
        match normalized.as_str() {
            "Ctrl" | "Shift" | "Alt" | "Win" => modifiers.push(normalized),
            _ => main_keys.push(normalized),
        }
    }
    
    // Sort modifiers in canonical order
    modifiers.sort_by_key(|m| match m.as_str() {
        "Ctrl" => 0,
        "Shift" => 1,
        "Alt" => 2,
        "Win" => 3,
        _ => 4,
    });
    
    // Combine modifiers and main keys
    let mut result = modifiers;
    result.extend(main_keys);
    
    result.join("+")
}

/// Normalize individual key part
fn normalize_key_part(part: &str) -> String {
    let lower = part.to_lowercase();
    
    // Common key mappings
    match lower.as_str() {
        // Modifiers
        "ctrl" | "control" | "ctl" => "Ctrl".to_string(),
        "shift" | "shft" => "Shift".to_string(),
        "alt" | "option" | "opt" => "Alt".to_string(),
        "cmd" | "command" | "win" | "windows" | "super" | "meta" => "Win".to_string(),
        
        // Special keys
        "space" | "spacebar" | "spc" => "Space".to_string(),
        "tab" => "Tab".to_string(),
        "enter" | "return" | "ret" => "Enter".to_string(),
        "esc" | "escape" => "Escape".to_string(),
        "backspace" | "back" | "bksp" => "Backspace".to_string(),
        "delete" | "del" => "Delete".to_string(),
        "insert" | "ins" => "Insert".to_string(),
        "home" => "Home".to_string(),
        "end" => "End".to_string(),
        "pageup" | "pgup" | "page_up" | "prior" => "PageUp".to_string(),
        "pagedown" | "pgdown" | "pgdn" | "page_down" | "next" => "PageDown".to_string(),
        
        // Arrow keys
        "left" | "arrowleft" | "arrow_left" | "leftarrow" => "Left".to_string(),
        "right" | "arrowright" | "arrow_right" | "rightarrow" => "Right".to_string(),
        "up" | "arrowup" | "arrow_up" | "uparrow" => "Up".to_string(),
        "down" | "arrowdown" | "arrow_down" | "downarrow" => "Down".to_string(),
        
        // Numpad
        "num0" | "numpad0" | "numpad_0" => "Numpad0".to_string(),
        "num1" | "numpad1" | "numpad_1" => "Numpad1".to_string(),
        "num2" | "numpad2" | "numpad_2" => "Numpad2".to_string(),
        "num3" | "numpad3" | "numpad_3" => "Numpad3".to_string(),
        "num4" | "numpad4" | "numpad_4" => "Numpad4".to_string(),
        "num5" | "numpad5" | "numpad_5" => "Numpad5".to_string(),
        "num6" | "numpad6" | "numpad_6" => "Numpad6".to_string(),
        "num7" | "numpad7" | "numpad_7" => "Numpad7".to_string(),
        "num8" | "numpad8" | "numpad_8" => "Numpad8".to_string(),
        "num9" | "numpad9" | "numpad_9" => "Numpad9".to_string(),
        
        // Function keys with various formats
        _ => {
            // Check for function keys (F1-F24)
            if let Some(num) = parse_function_key(&lower) {
                format!("F{}", num)
            }
            // Single character - uppercase it
            else if part.len() == 1 && part.chars().all(|c| c.is_alphabetic()) {
                part.to_uppercase()
            }
            // Digit keys
            else if part.len() == 1 && part.chars().all(|c| c.is_numeric()) {
                part.to_string()
            }
            // For anything else, use title case
            else {
                // First letter uppercase, rest lowercase
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            }
        }
    }
}

/// Parse function key from various formats (f1, F1, func1, function1, etc.)
fn parse_function_key(s: &str) -> Option<u8> {
    // Remove common prefixes
    let num_part = s
        .strip_prefix("f")
        .or_else(|| s.strip_prefix("func"))
        .or_else(|| s.strip_prefix("function"))
        .or_else(|| s.strip_prefix("fn"))
        .unwrap_or(s);
    
    // Try to parse the number
    num_part.parse::<u8>().ok().filter(|&n| n >= 1 && n <= 24)
}

pub struct HotkeyManager {
    registered_hotkeys: Arc<Mutex<HashMap<String, String>>>, // hotkey -> keyboard_id
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            registered_hotkeys: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register all keyboard hotkeys
    pub fn register_all_hotkeys(&self, app: &AppHandle, keyboard_manager: &KeyboardManager) -> Result<()> {
        // First unregister all existing hotkeys
        self.unregister_all_hotkeys(app)?;

        let mut failed_hotkeys = Vec::new();

        // Register new hotkeys, collecting failures
        for keyboard in keyboard_manager.get_keyboards() {
            // Determine which hotkey to use: custom hotkey takes precedence over default
            let hotkey_to_register = match &keyboard.hotkey {
                Some(custom_hotkey) if !custom_hotkey.is_empty() => {
                    // Use custom hotkey if it's set and not empty
                    Some(custom_hotkey.as_str())
                }
                _ => {
                    // Otherwise, use default hotkey if available
                    keyboard.default_hotkey.as_deref()
                }
            };
            
            if let Some(hotkey) = hotkey_to_register {
                if !hotkey.is_empty() {
                    if let Err(e) = self.register_hotkey(app, &keyboard.id, hotkey) {
                        error!("Failed to register hotkey '{}' for keyboard '{}': {}", 
                                 hotkey, keyboard.name, e);
                        failed_hotkeys.push(format!("{} ({})", keyboard.name, hotkey));
                    }
                }
            }
        }

        // If any hotkeys failed, return an error with details
        if !failed_hotkeys.is_empty() {
            return Err(anyhow!(
                "Failed to register hotkeys for: {}",
                failed_hotkeys.join(", ")
            ));
        }

        Ok(())
    }

    /// Validate and prepare hotkey for Tauri global shortcut
    /// Ensures the hotkey has at least one modifier and exactly one normal key
    fn validate_and_prepare_hotkey(hotkey: &str) -> Result<String> {
        // First normalize using the existing function
        let normalized = normalize_hotkey(hotkey);
        
        // Split into parts
        let parts: Vec<&str> = normalized.split('+').collect();
        
        if parts.is_empty() {
            return Err(anyhow!("Empty hotkey"));
        }
        
        let mut has_modifier = false;
        let mut normal_key_count = 0;
        
        // Check each part
        for part in &parts {
            match part.as_ref() {
                "Ctrl" | "Alt" | "Shift" | "Win" => has_modifier = true,
                _ => normal_key_count += 1,
            }
        }
        
        // Validate structure
        if !has_modifier {
            return Err(anyhow!("Hotkey must include at least one modifier (Ctrl, Alt, Shift, or Win)"));
        }
        
        if normal_key_count == 0 {
            return Err(anyhow!("Hotkey must include a normal key (letter, number, or special key)"));
        }
        
        if normal_key_count > 1 {
            return Err(anyhow!("Hotkey can only have one normal key"));
        }
        
        // Convert Win to Super for Tauri
        let tauri_format = normalized.replace("Win", "Super");
        
        Ok(tauri_format)
    }

    /// Register a single hotkey for a keyboard
    pub fn register_hotkey(&self, app: &AppHandle, keyboard_id: &str, hotkey: &str) -> Result<()> {
        // Validate and prepare the hotkey
        let prepared_hotkey = Self::validate_and_prepare_hotkey(hotkey)?;
        
        // Check if this hotkey is already registered
        {
            let registered = self.registered_hotkeys.lock().unwrap();
            if let Some(existing_keyboard_id) = registered.get(&prepared_hotkey) {
                if existing_keyboard_id != keyboard_id {
                    return Err(anyhow!(
                        "Hotkey '{}' is already registered for another keyboard", 
                        hotkey
                    ));
                }
                // If it's already registered for the same keyboard, just return success
                return Ok(());
            }
        }
        
        // Parse the shortcut string
        let shortcut = Shortcut::try_from(prepared_hotkey.as_str())
            .map_err(|e| anyhow!("Invalid hotkey format '{}': {:?}", hotkey, e))?;

        // Clone values for the closure
        let keyboard_id_str = keyboard_id.to_string();
        let app_handle = app.clone();
        
        // Register the global shortcut
        app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
            // Switch to this keyboard
            let state = app_handle.state::<Mutex<KeyboardManager>>();
            let (action_result, keyboard_name) = if let Ok(mut manager) = state.lock() {
                // Switch to the keyboard
                let switch_result = manager.set_active_keyboard(&keyboard_id_str);
                
                let name = manager.get_keyboards()
                    .iter()
                    .find(|k| k.id == keyboard_id_str)
                    .map(|k| k.name.clone())
                    .unwrap_or_else(|| keyboard_id_str.clone());
                
                (switch_result, name)
            } else {
                (Err(anyhow::anyhow!("Failed to lock keyboard manager")), keyboard_id_str.clone())
            };
            
            if let Err(e) = action_result {
                error!("Failed to switch keyboard: {}", e);
            } else {
                // Re-acquire lock for tray updates
                if let Ok(manager) = state.lock() {
                    // Update tray menu and icon
                    crate::tray::update_tray_menu(&app_handle, &manager);
                    crate::tray::update_tray_icon(&app_handle, &manager);
                }
                
                // Emit event to update UI
                let _ = app_handle.emit("keyboard-switched", &keyboard_id_str);
                
                // Show HUD notification with keyboard name
                if let Err(e) = crate::hud::show_keyboard_hud(&keyboard_name) {
                    error!("Failed to show HUD: {}", e);
                }
            }
        })?;

        // Store the registration with prepared key
        let mut registered = self.registered_hotkeys.lock().unwrap();
        registered.insert(prepared_hotkey, keyboard_id.to_string());

        Ok(())
    }

    /// Unregister all hotkeys
    pub fn unregister_all_hotkeys(&self, app: &AppHandle) -> Result<()> {
        // Get all registered hotkeys
        let registered = self.registered_hotkeys.lock().unwrap();
        
        // Unregister each shortcut
        for hotkey_str in registered.keys() {
            if let Ok(shortcut) = Shortcut::try_from(hotkey_str.as_str()) {
                if let Err(e) = app.global_shortcut().unregister(shortcut) {
                    error!("Failed to unregister shortcut '{}': {}", hotkey_str, e);
                }
            }
        }
        
        // Clear the registration map
        drop(registered);
        self.registered_hotkeys.lock().unwrap().clear();

        Ok(())
    }

    /// Update hotkeys when keyboards change
    pub fn refresh_hotkeys(&self, app: &AppHandle, keyboard_manager: &KeyboardManager) -> Result<()> {
        self.register_all_hotkeys(app, keyboard_manager)
    }
}