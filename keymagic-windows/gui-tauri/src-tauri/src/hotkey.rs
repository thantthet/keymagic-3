use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Emitter};
use anyhow::{Result, anyhow};

use crate::keyboard_manager::KeyboardManager;
use crate::keyboard_hook::KeyboardHook;

pub struct HotkeyManager {
    keyboard_hook: Arc<KeyboardHook>,
    registered_hotkeys: Mutex<HashMap<String, String>>, // hotkey -> keyboard_id
    on_off_hotkey: Mutex<Option<String>>, // Global on/off hotkey
}

impl HotkeyManager {
    pub fn new() -> Self {
        let hook = KeyboardHook::new();
        
        // Install the keyboard hook
        if let Err(e) = hook.install() {
            eprintln!("Failed to install keyboard hook: {}", e);
        }
        
        Self {
            keyboard_hook: hook,
            registered_hotkeys: Mutex::new(HashMap::new()),
            on_off_hotkey: Mutex::new(None),
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
                        eprintln!("Failed to register hotkey '{}' for keyboard '{}': {}", 
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
            // Toggle this keyboard
            let state = app_handle.state::<Mutex<KeyboardManager>>();
            let (action_result, keyboard_name, key_processing_changed) = if let Ok(mut manager) = state.lock() {
                let current_active = manager.get_active_keyboard();
                let is_key_processing_enabled = manager.is_key_processing_enabled();
                
                // Check if this keyboard is currently active
                let is_currently_active = current_active == Some(&keyboard_id_str);
                
                let (result, _action_name, key_processing_changed) = if is_currently_active && is_key_processing_enabled {
                    // This keyboard is active and processing is enabled - disable processing
                    let disable_result = manager.set_key_processing_enabled(false);
                    (disable_result, "disabled", true)
                } else {
                    // Either this keyboard is not active or processing is disabled - activate it
                    let switch_result = manager.set_active_keyboard(&keyboard_id_str);
                    if switch_result.is_ok() && !is_key_processing_enabled {
                        // Also enable key processing if it was disabled
                        let _ = manager.set_key_processing_enabled(true);
                        (switch_result, "activated", true)
                    } else {
                        (switch_result, "activated", false)
                    }
                };
                
                let name = manager.get_keyboards()
                    .iter()
                    .find(|k| k.id == keyboard_id_str)
                    .map(|k| k.name.clone())
                    .unwrap_or_else(|| keyboard_id_str.clone());
                
                (result, name, key_processing_changed)
            } else {
                (Err(anyhow::anyhow!("Failed to lock keyboard manager")), keyboard_id_str.clone(), false)
            };
            
            if let Err(e) = action_result {
                eprintln!("Failed to toggle keyboard: {}", e);
            } else {
                // Re-acquire lock for tray updates
                if let Ok(manager) = state.lock() {
                    // Update tray menu and icon
                    crate::tray::update_tray_menu(&app_handle, &manager);
                    crate::tray::update_tray_icon(&app_handle, &manager);
                }
                
                // Emit event to update UI
                let _ = app_handle.emit("keyboard-switched", &keyboard_id_str);
                
                // If key processing state changed, emit that event too
                if key_processing_changed {
                    if let Ok(manager) = state.lock() {
                        let is_enabled = manager.is_key_processing_enabled();
                        let _ = app_handle.emit("key_processing_changed", is_enabled);
                    }
                }
                
                // Show native HUD notification with appropriate status
                if let Ok(manager) = state.lock() {
                    let is_enabled = manager.is_key_processing_enabled();
                    if is_enabled {
                        // Show the keyboard name when enabled
                        if let Err(e) = crate::hud::show_keyboard_hud(&keyboard_name) {
                            eprintln!("Failed to show HUD: {}", e);
                        }
                    } else {
                        // Show "KeyMagic Disabled" when disabled
                        if let Err(e) = crate::hud::show_status_hud("KeyMagic Disabled") {
                            eprintln!("Failed to show HUD: {}", e);
                        }
                    }
                }
            }
        })?;

        // Store the registration
        let mut registered = self.registered_hotkeys.lock().unwrap();
        registered.insert(hotkey.to_string(), keyboard_id.to_string());

        Ok(())
    }

    /// Unregister all hotkeys
    pub fn unregister_all_hotkeys(&self, app: &AppHandle) -> Result<()> {
        // Save the on/off hotkey before clearing
        let on_off_key = self.on_off_hotkey.lock().unwrap().clone();
        
        // Unregister all hotkeys from keyboard hook
        self.keyboard_hook.unregister_all_hotkeys()?;

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
        let app_handle = app.clone();
        
        // Register the hotkey with our keyboard hook
        self.keyboard_hook.register_hotkey(hotkey, move || {
            // Toggle key processing
            let state = app_handle.state::<Mutex<KeyboardManager>>();
            if let Ok(mut manager) = state.lock() {
                let current_state = manager.is_key_processing_enabled();
                let new_state = !current_state;
                
                if let Err(e) = manager.set_key_processing_enabled(new_state) {
                    eprintln!("Failed to toggle key processing: {}", e);
                } else {
                    // Update tray menu and icon
                    crate::tray::update_tray_menu(&app_handle, &manager);
                    crate::tray::update_tray_icon(&app_handle, &manager);
                    
                    // Emit event to update UI
                    let _ = app_handle.emit("key_processing_changed", new_state);
                    
                    // Show HUD notification
                    let status = if new_state { "Enabled" } else { "Disabled" };
                    if let Err(e) = crate::hud::show_status_hud(&format!("KeyMagic {}", status)) {
                        eprintln!("Failed to show HUD: {}", e);
                    }
                }
            };
        })?;

        // Store the registration
        let mut on_off = self.on_off_hotkey.lock().unwrap();
        *on_off = Some(hotkey.to_string());

        Ok(())
    }

    /// Unregister the on/off hotkey
    pub fn unregister_on_off_hotkey(&self, _app: &AppHandle) -> Result<()> {
        let mut on_off = self.on_off_hotkey.lock().unwrap();
        if let Some(hotkey_str) = on_off.take() {
            self.keyboard_hook.unregister_hotkey(&hotkey_str)?;
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
            use crate::registry;
            
            if let Some(hotkey) = registry::get_on_off_hotkey()
                .map_err(|e| anyhow!("Failed to get on/off hotkey: {}", e))? {
                if !hotkey.is_empty() {
                    self.set_on_off_hotkey(app, Some(&hotkey))?;
                }
            }
        }
        
        Ok(())
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        // The keyboard hook will uninstall itself when dropped
    }
}