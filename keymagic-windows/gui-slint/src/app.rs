use std::sync::{Arc, Mutex};
use anyhow::Result;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{MainWindow, KeyboardInfo as SlintKeyboardInfo};
use crate::keyboard_manager::KeyboardManager;
use crate::models::convert_keyboard_info;
use crate::file_dialog::show_open_file_dialog;

pub struct App {
    keyboard_manager: Arc<Mutex<KeyboardManager>>,
}

impl App {
    pub fn new() -> Result<Self> {
        let keyboard_manager = Arc::new(Mutex::new(KeyboardManager::new()?));
        
        Ok(Self {
            keyboard_manager,
        })
    }
    
    pub fn setup_ui(&self, window: &MainWindow) -> Result<()> {
        // Load keyboards and update UI
        self.refresh_keyboards(window)?;
        
        // Set initial KeyMagic enabled state
        let enabled = self.keyboard_manager.lock().unwrap().is_key_processing_enabled();
        window.set_keymagic_enabled(enabled);
        
        // Load settings from registry
        self.load_settings(window)?;
        
        Ok(())
    }
    
    fn load_settings(&self, window: &MainWindow) -> Result<()> {
        use windows::Win32::System::Registry::*;
        use windows::core::w;
        
        unsafe {
            let mut hkey = HKEY::default();
            
            // Load general settings
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                w!("Software\\KeyMagic\\Settings"),
                0,
                KEY_READ,
                &mut hkey
            ).is_ok() {
                // Start with Windows
                if let Some(value) = self.read_registry_dword(hkey, w!("StartWithWindows")) {
                    window.set_start_with_windows(value != 0);
                }
                
                // Other settings (with defaults if not found)
                if let Some(value) = self.read_registry_dword(hkey, w!("ShowInTray")) {
                    window.set_show_in_tray(value != 0);
                } else {
                    window.set_show_in_tray(true); // Default
                }
                
                if let Some(value) = self.read_registry_dword(hkey, w!("MinimizeToTray")) {
                    window.set_minimize_to_tray(value != 0);
                } else {
                    window.set_minimize_to_tray(true); // Default
                }
                
                if let Some(value) = self.read_registry_dword(hkey, w!("ShowNotifications")) {
                    window.set_show_notifications(value != 0);
                } else {
                    window.set_show_notifications(true); // Default
                }
                
                RegCloseKey(hkey);
            }
        }
        
        Ok(())
    }
    
    unsafe fn read_registry_dword(&self, hkey: windows::Win32::System::Registry::HKEY, value_name: windows::core::PCWSTR) -> Option<u32> {
        use windows::Win32::System::Registry::*;
        
        let mut data_type = REG_VALUE_TYPE::default();
        let mut data = 0u32;
        let mut data_size = std::mem::size_of::<u32>() as u32;
        
        let result = RegQueryValueExW(
            hkey,
            value_name,
            None,
            Some(&mut data_type),
            Some(&mut data as *mut u32 as *mut u8),
            Some(&mut data_size),
        );
        
        if result.is_ok() {
            Some(data)
        } else {
            None
        }
    }
    
    pub fn connect_callbacks(&self, window: &MainWindow) {
        // Page changed callback
        window.on_page_changed(move |page| {
            println!("Page changed to: {}", page);
        });
        
        // Add keyboard callback
        let window_weak_add = window.as_weak();
        let manager_add = self.keyboard_manager.clone();
        window.on_add_keyboard(move || {
            let window = window_weak_add.upgrade().unwrap();
            Self::handle_add_keyboard(&window, &manager_add);
        });
        
        // Remove keyboard callback - This is called when Remove button is clicked
        let window_weak_remove = window.as_weak();
        let manager_remove = self.keyboard_manager.clone();
        window.on_remove_keyboard(move || {
            let window = window_weak_remove.upgrade().unwrap();
            // Check if this is from the dialog confirmation
            let keyboard_to_remove = window.get_keyboard_to_remove();
            if !keyboard_to_remove.is_empty() {
                // This is from dialog confirmation, perform the removal
                Self::perform_keyboard_removal(&window, &manager_remove);
            } else {
                // This is from the Remove button, show the dialog
                Self::handle_remove_keyboard(&window, &manager_remove);
            }
        });
        
        // Activate keyboard callback
        let window_weak_activate = window.as_weak();
        let manager_activate = self.keyboard_manager.clone();
        window.on_activate_keyboard(move |id| {
            let window = window_weak_activate.upgrade().unwrap();
            Self::handle_activate_keyboard(&window, &manager_activate, &id);
        });
        
        // Toggle KeyMagic callback
        let window_weak_toggle = window.as_weak();
        let manager_toggle = self.keyboard_manager.clone();
        window.on_toggle_keymagic(move || {
            let window = window_weak_toggle.upgrade().unwrap();
            Self::handle_toggle_keymagic(&window, &manager_toggle);
        });
        
        // Save settings callback
        let window_weak_save = window.as_weak();
        let manager_save = self.keyboard_manager.clone();
        window.on_save_settings(move || {
            let window = window_weak_save.upgrade().unwrap();
            Self::handle_save_settings(&window, &manager_save);
        });
        
        // Reset settings callback
        let window_weak_reset = window.as_weak();
        let manager_reset = self.keyboard_manager.clone();
        window.on_reset_settings(move || {
            let window = window_weak_reset.upgrade().unwrap();
            Self::handle_reset_settings(&window, &manager_reset);
        });
        
        // Keyboard hotkey changed callback
        let window_weak_hotkey = window.as_weak();
        let manager_hotkey = self.keyboard_manager.clone();
        window.on_keyboard_hotkey_changed(move |id, hotkey| {
            let window = window_weak_hotkey.upgrade().unwrap();
            Self::handle_keyboard_hotkey_changed(&window, &manager_hotkey, &id, &hotkey);
        });
    }
    
    fn refresh_keyboards(&self, window: &MainWindow) -> Result<()> {
        let manager = self.keyboard_manager.lock().unwrap();
        let keyboards = manager.get_keyboards();
        let active_id = manager.get_active_keyboard();
        
        // Convert to Slint model
        let slint_keyboards: Vec<SlintKeyboardInfo> = keyboards.iter()
            .map(|kb| convert_keyboard_info(kb, active_id, &manager))
            .collect();
        
        let model = VecModel::from(slint_keyboards);
        window.set_keyboards(ModelRc::new(model));
        
        Ok(())
    }
    
    fn handle_add_keyboard(window: &MainWindow, manager: &Arc<Mutex<KeyboardManager>>) {
        println!("Add keyboard clicked");
        
        // Show file dialog
        if let Some(path) = show_open_file_dialog(None) {
            println!("Selected file: {:?}", path);
            
            // Load the keyboard
            let mut mgr = manager.lock().unwrap();
            match mgr.load_keyboard(&path) {
                Ok(keyboard_id) => {
                    println!("Successfully loaded keyboard: {}", keyboard_id);
                    drop(mgr);
                    
                    // Refresh the keyboard list
                    let mgr = manager.lock().unwrap();
                    let keyboards = mgr.get_keyboards();
                    let active_id = mgr.get_active_keyboard();
                    
                    let slint_keyboards: Vec<SlintKeyboardInfo> = keyboards.iter()
                        .map(|kb| convert_keyboard_info(kb, active_id, &mgr))
                        .collect();
                    
                    let model = VecModel::from(slint_keyboards);
                    window.set_keyboards(ModelRc::new(model));
                }
                Err(e) => {
                    eprintln!("Failed to load keyboard: {}", e);
                    drop(mgr);
                    
                    // Show error dialog
                    let error_msg = format!("Failed to load keyboard: {}", e);
                    window.invoke_show_error(slint::SharedString::from(error_msg));
                }
            }
        }
    }
    
    fn handle_remove_keyboard(window: &MainWindow, _manager: &Arc<Mutex<KeyboardManager>>) {
        let selected_id = window.get_selected_keyboard_id();
        if selected_id.is_empty() {
            return;
        }
        
        println!("Remove keyboard: {}", selected_id);
        
        // Show confirmation dialog
        window.invoke_show_remove_confirmation(selected_id.clone());
    }
    
    fn perform_keyboard_removal(window: &MainWindow, manager: &Arc<Mutex<KeyboardManager>>) {
        let keyboard_to_remove = window.get_keyboard_to_remove();
        if keyboard_to_remove.is_empty() {
            return;
        }
        
        let mut mgr = manager.lock().unwrap();
        if let Err(e) = mgr.remove_keyboard(&keyboard_to_remove) {
            eprintln!("Failed to remove keyboard: {}", e);
            drop(mgr);
            
            // Show error dialog
            let error_msg = format!("Failed to remove keyboard: {}", e);
            window.invoke_show_error(slint::SharedString::from(error_msg));
        } else {
            drop(mgr);
            
            // Clear selection if we removed the selected keyboard
            if window.get_selected_keyboard_id() == keyboard_to_remove {
                window.set_selected_keyboard_id(slint::SharedString::new());
            }
            
            // Clear the keyboard-to-remove property
            window.set_keyboard_to_remove(slint::SharedString::new());
            
            // Refresh the keyboard list
            let mgr = manager.lock().unwrap();
            let keyboards = mgr.get_keyboards();
            let active_id = mgr.get_active_keyboard();
            
            let slint_keyboards: Vec<SlintKeyboardInfo> = keyboards.iter()
                .map(|kb| convert_keyboard_info(kb, active_id, &mgr))
                .collect();
            
            let model = VecModel::from(slint_keyboards);
            window.set_keyboards(ModelRc::new(model));
        }
    }
    
    fn handle_activate_keyboard(window: &MainWindow, manager: &Arc<Mutex<KeyboardManager>>, id: &str) {
        println!("Activate keyboard: {}", id);
        
        let mut mgr = manager.lock().unwrap();
        if let Err(e) = mgr.set_active_keyboard(id) {
            eprintln!("Failed to activate keyboard: {}", e);
        } else {
            drop(mgr);
            // Refresh the keyboard list
            let mgr = manager.lock().unwrap();
            let keyboards = mgr.get_keyboards();
            let active_id = mgr.get_active_keyboard();
            
            let slint_keyboards: Vec<SlintKeyboardInfo> = keyboards.iter()
                .map(|kb| convert_keyboard_info(kb, active_id, &mgr))
                .collect();
            
            let model = VecModel::from(slint_keyboards);
            window.set_keyboards(ModelRc::new(model));
        }
    }
    
    fn handle_toggle_keymagic(window: &MainWindow, manager: &Arc<Mutex<KeyboardManager>>) {
        let mgr = manager.lock().unwrap();
        let current = mgr.is_key_processing_enabled();
        
        if let Err(e) = mgr.set_key_processing_enabled(!current) {
            eprintln!("Failed to toggle KeyMagic: {}", e);
        } else {
            window.set_keymagic_enabled(!current);
            println!("KeyMagic toggled to: {}", !current);
        }
    }
    
    fn handle_save_settings(window: &MainWindow, manager: &Arc<Mutex<KeyboardManager>>) {
        println!("Saving settings...");
        
        // Save general settings
        let mgr = manager.lock().unwrap();
        
        // Save start with Windows setting
        if let Err(e) = mgr.set_start_with_windows(window.get_start_with_windows()) {
            eprintln!("Failed to save start with Windows setting: {}", e);
        }
        
        drop(mgr);
        
        // Save other settings to registry
        use windows::Win32::System::Registry::*;
        use windows::core::w;
        
        unsafe {
            let mut hkey = HKEY::default();
            
            if RegCreateKeyW(
                HKEY_CURRENT_USER,
                w!("Software\\KeyMagic\\Settings"),
                &mut hkey
            ).is_ok() {
                let mgr = manager.lock().unwrap();
                
                // Save all settings
                mgr.write_registry_dword(hkey, w!("ShowInTray"), if window.get_show_in_tray() { 1 } else { 0 }).ok();
                mgr.write_registry_dword(hkey, w!("MinimizeToTray"), if window.get_minimize_to_tray() { 1 } else { 0 }).ok();
                mgr.write_registry_dword(hkey, w!("ShowNotifications"), if window.get_show_notifications() { 1 } else { 0 }).ok();
                
                RegCloseKey(hkey);
            }
        }
        
        println!("Settings saved successfully");
    }
    
    fn handle_reset_settings(window: &MainWindow, _manager: &Arc<Mutex<KeyboardManager>>) {
        println!("Resetting settings to defaults...");
        
        // Reset UI to default values
        window.set_start_with_windows(false);
        window.set_show_in_tray(true);
        window.set_minimize_to_tray(true);
        window.set_show_notifications(true);
        
        // Reset hotkeys would be handled per keyboard
        println!("Settings reset to defaults");
    }
    
    fn handle_keyboard_hotkey_changed(window: &MainWindow, manager: &Arc<Mutex<KeyboardManager>>, id: &str, hotkey: &str) {
        println!("Keyboard hotkey changed: {} -> {}", id, hotkey);
        
        let mut mgr = manager.lock().unwrap();
        
        // Validate hotkey format
        if !hotkey.is_empty() && !Self::is_valid_hotkey(hotkey) {
            eprintln!("Invalid hotkey format: {}", hotkey);
            return;
        }
        
        // Update the hotkey
        if let Err(e) = mgr.set_keyboard_hotkey(id, hotkey) {
            eprintln!("Failed to set keyboard hotkey: {}", e);
            drop(mgr);
            
            // Show error dialog
            let error_msg = format!("Failed to set hotkey: {}", e);
            window.invoke_show_error(slint::SharedString::from(error_msg));
        } else {
            drop(mgr);
            
            // Refresh the keyboard list to update the hotkey display
            let mgr = manager.lock().unwrap();
            let keyboards = mgr.get_keyboards();
            let active_id = mgr.get_active_keyboard();
            
            let slint_keyboards: Vec<SlintKeyboardInfo> = keyboards.iter()
                .map(|kb| convert_keyboard_info(kb, active_id, &mgr))
                .collect();
            
            let model = VecModel::from(slint_keyboards);
            window.set_keyboards(ModelRc::new(model));
        }
    }
    
    fn is_valid_hotkey(hotkey: &str) -> bool {
        // Basic validation: must contain at least two modifiers, or one modifier + a key
        let parts: Vec<&str> = hotkey.split('+').collect();
        if parts.is_empty() {
            return false;
        }
        
        let modifiers = ["Ctrl", "Alt", "Shift"];
        let modifier_count = parts.iter().filter(|p| modifiers.contains(&p.trim())).count();
        let has_key = parts.iter().any(|p| {
            let trimmed = p.trim();
            !modifiers.contains(&trimmed) && !trimmed.is_empty()
        });
        
        // Valid if at least 2 modifiers, or 1 modifier + a key
        modifier_count >= 2 || (modifier_count >= 1 && has_key)
    }
}