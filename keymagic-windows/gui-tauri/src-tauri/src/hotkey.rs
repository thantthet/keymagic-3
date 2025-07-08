use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, ShortcutState, Shortcut};
use anyhow::{Result, anyhow};

use crate::keyboard_manager::KeyboardManager;

pub struct HotkeyManager {
    registered_hotkeys: Mutex<HashMap<String, String>>, // hotkey -> keyboard_id
    on_off_hotkey: Mutex<Option<String>>, // Global on/off hotkey
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            registered_hotkeys: Mutex::new(HashMap::new()),
            on_off_hotkey: Mutex::new(None),
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
        
        // Parse the hotkey string for validation
        let (_modifiers, _key_code) = parse_hotkey(hotkey)?;
        
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
        
        // Save the on/off hotkey before clearing
        let on_off_key = self.on_off_hotkey.lock().unwrap().clone();
        
        // Unregister all shortcuts
        shortcut.unregister_all()
            .map_err(|e| anyhow!("Failed to unregister hotkeys: {}", e))?;

        // Clear the registration map
        let mut registered = self.registered_hotkeys.lock().unwrap();
        registered.clear();
        
        // Re-register the on/off hotkey if it exists
        if let Some(hotkey) = on_off_key {
            self.register_on_off_hotkey(app, &hotkey)?;
        }

        Ok(())
    }

    /// Update hotkeys when keyboards change
    pub fn refresh_hotkeys(&self, app: &AppHandle, keyboard_manager: &KeyboardManager) -> Result<()> {
        self.register_all_hotkeys(app, keyboard_manager)
    }

    /// Register the global on/off hotkey
    pub fn register_on_off_hotkey(&self, app: &AppHandle, hotkey: &str) -> Result<()> {
        let shortcut = app.global_shortcut();
        
        // Parse the hotkey string for validation
        let (_modifiers, _key_code) = parse_hotkey(hotkey)?;
        
        let app_handle = app.clone();
        
        // Register the shortcut
        shortcut
            .on_shortcut(hotkey, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    // Toggle key processing
                    let state = app_handle.state::<Mutex<KeyboardManager>>();
                    if let Ok(mut manager) = state.lock() {
                        let current_state = manager.is_key_processing_enabled();
                        let new_state = !current_state;
                        
                        if let Err(e) = manager.set_key_processing_enabled(new_state) {
                            eprintln!("Failed to toggle key processing: {}", e);
                        } else {
                            // Update tray menu
                            crate::tray::update_tray_menu(&app_handle, &manager);
                            
                            // Emit event to update UI
                            let _ = app_handle.emit("key_processing_changed", new_state);
                            
                            // Show HUD notification
                            let status = if new_state { "Enabled" } else { "Disabled" };
                            if let Err(e) = crate::hud::show_status_hud(&format!("KeyMagic {}", status)) {
                                eprintln!("Failed to show HUD: {}", e);
                            }
                        }
                    };
                }
            })
            .map_err(|e| anyhow!("Failed to register on/off hotkey {}: {}", hotkey, e))?;

        // Store the registration
        let mut on_off = self.on_off_hotkey.lock().unwrap();
        *on_off = Some(hotkey.to_string());

        Ok(())
    }

    /// Unregister the on/off hotkey
    pub fn unregister_on_off_hotkey(&self, app: &AppHandle) -> Result<()> {
        let mut on_off = self.on_off_hotkey.lock().unwrap();
        if let Some(hotkey_str) = on_off.take() {
            let shortcut = app.global_shortcut();
            // Parse the hotkey string back to Shortcut
            match hotkey_str.parse::<Shortcut>() {
                Ok(hotkey) => {
                    shortcut.unregister(hotkey)
                        .map_err(|e| anyhow!("Failed to unregister on/off hotkey: {}", e))?;
                }
                Err(e) => {
                    eprintln!("Failed to parse hotkey '{}': {}", hotkey_str, e);
                }
            }
        }
        Ok(())
    }

    /// Set and register the on/off hotkey (unregisters old one if exists)
    pub fn set_on_off_hotkey(&self, app: &AppHandle, hotkey: Option<&str>) -> Result<()> {
        // First unregister the old hotkey
        self.unregister_on_off_hotkey(app)?;
        
        // Then register the new one if provided
        if let Some(hotkey) = hotkey {
            if !hotkey.is_empty() {
                self.register_on_off_hotkey(app, hotkey)?;
            }
        }
        
        Ok(())
    }

    /// Load and register on/off hotkey from settings
    pub fn load_on_off_hotkey(&self, app: &AppHandle) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use windows::core::*;
            use windows::Win32::System::Registry::*;
            
            unsafe {
                let mut hkey = HKEY::default();
                
                if RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    w!("Software\\KeyMagic\\Settings"),
                    0,
                    KEY_READ,
                    &mut hkey
                ).is_ok() {
                    let mut buffer = vec![0u16; 256];
                    let mut size = buffer.len() as u32 * 2;
                    let mut data_type = REG_VALUE_TYPE::default();
                    
                    let result = RegQueryValueExW(
                        hkey,
                        w!("OnOffHotkey"),
                        None,
                        Some(&mut data_type),
                        Some(buffer.as_mut_ptr() as *mut u8),
                        Some(&mut size),
                    );
                    
                    RegCloseKey(hkey);
                    
                    if result.is_ok() {
                        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
                        buffer.truncate(len);
                        let hotkey = String::from_utf16_lossy(&buffer);
                        if !hotkey.is_empty() {
                            self.set_on_off_hotkey(app, Some(&hotkey))?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Parse a hotkey string like "Ctrl+Shift+M" or "Ctrl+Shift" into Tauri modifiers and optional key code
fn parse_hotkey(hotkey: &str) -> Result<(Modifiers, Option<Code>)> {
    let parts: Vec<&str> = hotkey.split('+').collect();
    if parts.is_empty() {
        return Err(anyhow!("Invalid hotkey format"));
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code = None;

    // Process all parts to identify modifiers and regular keys
    for part in parts.iter() {
        let part = part.trim();
        
        // Try to parse as modifier first
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "cmd" | "win" | "super" | "meta" => modifiers |= Modifiers::META,
            _ => {
                // Not a modifier, try to parse as a regular key
                if key_code.is_some() {
                    return Err(anyhow!("Multiple non-modifier keys specified"));
                }
                
                // Parse the key code
                key_code = Some(match part.to_uppercase().as_str() {
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
                    _ => return Err(anyhow!("Unknown key: {}", part)),
                });
            }
        }
    }

    // Validate: must have at least one modifier
    if modifiers.is_empty() {
        return Err(anyhow!("Hotkey must include at least one modifier (Ctrl, Alt, Shift, or Win/Cmd)"));
    }

    // Warn about potential issues with modifier-only hotkeys
    if key_code.is_none() {
        // Note: We still return Ok, but the caller should be aware this might not work
        eprintln!("Warning: Modifier-only hotkey '{}' may not be supported by the system", hotkey);
    }

    // Both modifiers-only and modifiers+key are valid from parsing perspective
    Ok((modifiers, key_code))
}