use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::ffi::CString;
use anyhow::{Result, anyhow};

// Import FFI types from keymagic-core
use keymagic_core::ffi::*;

pub struct KeyboardInfo {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub description: String,
    pub icon_data: Option<Vec<u8>>,
    pub hotkey: Option<String>,
    pub enabled: bool,
}

pub struct KeyboardManager {
    keyboards: HashMap<String, KeyboardInfo>,
    active_keyboard: Option<String>,
}

impl KeyboardManager {
    pub fn new() -> Result<Self> {
        let manager = Self {
            keyboards: HashMap::new(),
            active_keyboard: None,
        };
        
        // For now, skip registry loading
        Ok(manager)
    }
    
    pub fn load_keyboard(&mut self, path: &Path) -> Result<String> {
        // Validate the keyboard file by loading it
        let engine = keymagic_engine_new();
        if engine.is_null() {
            return Err(anyhow!("Failed to create engine"));
        }
        
        let c_path = CString::new(path.to_str().unwrap())?;
        let result = keymagic_engine_load_keyboard(engine, c_path.as_ptr());
        
        // Clean up engine
        keymagic_engine_free(engine);
        
        if result != KeyMagicResult::Success {
            return Err(anyhow!("Failed to load keyboard"));
        }
        
        // Extract metadata from km2 file
        // For now, use simple parsing to get the name
        let keyboard_id = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        let info = KeyboardInfo {
            id: keyboard_id.clone(),
            path: path.to_path_buf(),
            name: keyboard_id.clone(), // TODO: Extract from km2
            description: String::new(), // TODO: Extract from km2
            icon_data: None, // TODO: Extract from km2
            hotkey: None,
            enabled: true,
        };
        
        self.keyboards.insert(keyboard_id.clone(), info);
        
        Ok(keyboard_id)
    }
    
    pub fn remove_keyboard(&mut self, id: &str) -> Result<()> {
        if self.keyboards.remove(id).is_some() {
            // If this was the active keyboard, clear it
            if self.active_keyboard.as_ref() == Some(&id.to_string()) {
                self.active_keyboard = None;
            }
        }
        
        Ok(())
    }
    
    pub fn get_keyboards(&self) -> Vec<&KeyboardInfo> {
        self.keyboards.values().collect()
    }
    
    pub fn get_keyboard(&self, id: &str) -> Option<&KeyboardInfo> {
        self.keyboards.get(id)
    }
    
    pub fn set_active_keyboard(&mut self, id: &str) -> Result<()> {
        if self.keyboards.contains_key(id) {
            self.active_keyboard = Some(id.to_string());
        }
        Ok(())
    }
    
    pub fn get_active_keyboard(&self) -> Option<&str> {
        self.active_keyboard.as_deref()
    }
}