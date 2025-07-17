use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Emitter};
use anyhow::{Result, anyhow};
use log::error;

use crate::keyboard_manager::KeyboardManager;
use crate::keyboard_hook::KeyboardHook;

pub struct HotkeyManager {
    keyboard_hook: Arc<KeyboardHook>,
    registered_hotkeys: Mutex<HashMap<String, String>>, // hotkey -> keyboard_id
}

impl HotkeyManager {
    pub fn new() -> Self {
        let hook = KeyboardHook::new();
        
        // Install the keyboard hook
        if let Err(e) = hook.install() {
            error!("Failed to install keyboard hook: {}", e);
        }
        
        Self {
            keyboard_hook: hook,
            registered_hotkeys: Mutex::new(HashMap::new()),
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

    /// Register a single hotkey for a keyboard
    pub fn register_hotkey(&self, app: &AppHandle, keyboard_id: &str, hotkey: &str) -> Result<()> {
        // Clone values for the closure
        let keyboard_id_str = keyboard_id.to_string();
        let app_handle = app.clone();
        
        // Register the hotkey with our keyboard hook
        self.keyboard_hook.register_hotkey(hotkey, move || {
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

        // Store the registration
        let mut registered = self.registered_hotkeys.lock().unwrap();
        registered.insert(hotkey.to_string(), keyboard_id.to_string());

        Ok(())
    }

    /// Unregister all hotkeys
    pub fn unregister_all_hotkeys(&self, _app: &AppHandle) -> Result<()> {
        // Unregister all hotkeys from keyboard hook
        self.keyboard_hook.unregister_all_hotkeys()?;

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

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        // The keyboard hook will uninstall itself when dropped
    }
}