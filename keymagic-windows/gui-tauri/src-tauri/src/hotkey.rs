use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, ShortcutState};
use anyhow::{Result, anyhow};

use crate::keyboard_manager::KeyboardManager;

pub struct HotkeyManager {
    registered_hotkeys: Mutex<HashMap<String, String>>, // hotkey -> keyboard_id
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            registered_hotkeys: Mutex::new(HashMap::new()),
        }
    }

    /// Register all keyboard hotkeys
    pub fn register_all_hotkeys(&self, app: &AppHandle, keyboard_manager: &KeyboardManager) -> Result<()> {
        // First unregister all existing hotkeys
        self.unregister_all_hotkeys(app)?;

        // Register new hotkeys
        for keyboard in keyboard_manager.get_keyboards() {
            if let Some(hotkey) = &keyboard.hotkey {
                if !hotkey.is_empty() {
                    self.register_hotkey(app, &keyboard.id, hotkey)?;
                }
            }
        }

        Ok(())
    }

    /// Register a single hotkey for a keyboard
    pub fn register_hotkey(&self, app: &AppHandle, keyboard_id: &str, hotkey: &str) -> Result<()> {
        let shortcut = app.global_shortcut();
        
        // Parse the hotkey string (currently unused but will be needed for validation)
        let (_modifiers, _key) = parse_hotkey(hotkey)?;
        
        // Clone values for the closure and storage
        let keyboard_id_str = keyboard_id.to_string();
        let keyboard_id_clone = keyboard_id_str.clone();
        let app_handle = app.clone();
        
        // Register the shortcut
        shortcut
            .on_shortcut(hotkey, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    // Switch to this keyboard
                    let state = app_handle.state::<Mutex<KeyboardManager>>();
                    if let Ok(mut manager) = state.lock() {
                        if let Err(e) = manager.set_active_keyboard(&keyboard_id_clone) {
                            eprintln!("Failed to switch keyboard: {}", e);
                        } else {
                            // Get the keyboard info for the name
                            let keyboard_name = manager.get_keyboards()
                                .iter()
                                .find(|k| k.id == keyboard_id_clone)
                                .map(|k| k.name.clone())
                                .unwrap_or_else(|| keyboard_id_clone.clone());
                            
                            // Update tray menu
                            crate::tray::update_tray_menu(&app_handle, &manager);
                            
                            // Emit event to update UI
                            let _ = app_handle.emit("keyboard-switched", &keyboard_id_clone);
                            
                            // Show native HUD notification
                            if let Err(e) = crate::hud::show_keyboard_hud(&keyboard_name) {
                                eprintln!("Failed to show HUD: {}", e);
                            }
                        }
                    };
                }
            })
            .map_err(|e| anyhow!("Failed to register hotkey {}: {}", hotkey, e))?;

        // Store the registration
        let mut registered = self.registered_hotkeys.lock().unwrap();
        registered.insert(hotkey.to_string(), keyboard_id_str);

        Ok(())
    }

    /// Unregister all hotkeys
    pub fn unregister_all_hotkeys(&self, app: &AppHandle) -> Result<()> {
        let shortcut = app.global_shortcut();
        
        // Unregister all shortcuts
        shortcut.unregister_all()
            .map_err(|e| anyhow!("Failed to unregister hotkeys: {}", e))?;

        // Clear the registration map
        let mut registered = self.registered_hotkeys.lock().unwrap();
        registered.clear();

        Ok(())
    }

    /// Update hotkeys when keyboards change
    pub fn refresh_hotkeys(&self, app: &AppHandle, keyboard_manager: &KeyboardManager) -> Result<()> {
        self.register_all_hotkeys(app, keyboard_manager)
    }
}

/// Parse a hotkey string like "Ctrl+Shift+M" into Tauri modifiers and key code
fn parse_hotkey(hotkey: &str) -> Result<(Modifiers, Code)> {
    let parts: Vec<&str> = hotkey.split('+').collect();
    if parts.is_empty() {
        return Err(anyhow!("Invalid hotkey format"));
    }

    let mut modifiers = Modifiers::empty();
    let mut key_part = "";

    for (i, part) in parts.iter().enumerate() {
        let part = part.trim();
        if i == parts.len() - 1 {
            // Last part is the key
            key_part = part;
        } else {
            // These are modifiers
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
                "alt" => modifiers |= Modifiers::ALT,
                "shift" => modifiers |= Modifiers::SHIFT,
                "cmd" | "win" | "super" | "meta" => modifiers |= Modifiers::META,
                _ => return Err(anyhow!("Unknown modifier: {}", part)),
            }
        }
    }

    // Parse the key code
    let code = match key_part.to_uppercase().as_str() {
        "A" => Code::KeyA,
        "B" => Code::KeyB,
        "C" => Code::KeyC,
        "D" => Code::KeyD,
        "E" => Code::KeyE,
        "F" => Code::KeyF,
        "G" => Code::KeyG,
        "H" => Code::KeyH,
        "I" => Code::KeyI,
        "J" => Code::KeyJ,
        "K" => Code::KeyK,
        "L" => Code::KeyL,
        "M" => Code::KeyM,
        "N" => Code::KeyN,
        "O" => Code::KeyO,
        "P" => Code::KeyP,
        "Q" => Code::KeyQ,
        "R" => Code::KeyR,
        "S" => Code::KeyS,
        "T" => Code::KeyT,
        "U" => Code::KeyU,
        "V" => Code::KeyV,
        "W" => Code::KeyW,
        "X" => Code::KeyX,
        "Y" => Code::KeyY,
        "Z" => Code::KeyZ,
        "0" => Code::Digit0,
        "1" => Code::Digit1,
        "2" => Code::Digit2,
        "3" => Code::Digit3,
        "4" => Code::Digit4,
        "5" => Code::Digit5,
        "6" => Code::Digit6,
        "7" => Code::Digit7,
        "8" => Code::Digit8,
        "9" => Code::Digit9,
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        "SPACE" => Code::Space,
        "TAB" => Code::Tab,
        "ENTER" | "RETURN" => Code::Enter,
        "ESC" | "ESCAPE" => Code::Escape,
        "BACKSPACE" => Code::Backspace,
        "DELETE" => Code::Delete,
        "INSERT" => Code::Insert,
        "HOME" => Code::Home,
        "END" => Code::End,
        "PAGEUP" => Code::PageUp,
        "PAGEDOWN" => Code::PageDown,
        "LEFT" => Code::ArrowLeft,
        "RIGHT" => Code::ArrowRight,
        "UP" => Code::ArrowUp,
        "DOWN" => Code::ArrowDown,
        _ => return Err(anyhow!("Unknown key: {}", key_part)),
    };

    Ok((modifiers, code))
}