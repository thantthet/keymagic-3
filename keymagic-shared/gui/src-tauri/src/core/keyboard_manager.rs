use anyhow::{Context, Result};
use keymagic_core::{KeyMagicEngine, Km2File, km2::Km2Loader};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::platform::{InstalledKeyboard, Platform};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardInfo {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub path: PathBuf,
    pub hotkey: Option<String>,
    pub hash: String,
    pub is_active: bool,
}

pub struct KeyboardManager {
    platform: Box<dyn Platform>,
    keyboards: Arc<Mutex<HashMap<String, KeyboardInfo>>>,
    active_keyboard: Arc<Mutex<Option<String>>>,
    engine: Arc<Mutex<Option<KeyMagicEngine>>>,
}

impl KeyboardManager {
    pub fn new(platform: Box<dyn Platform>) -> Self {
        Self {
            platform,
            keyboards: Arc::new(Mutex::new(HashMap::new())),
            active_keyboard: Arc::new(Mutex::new(None)),
            engine: Arc::new(Mutex::new(None)),
        }
    }
    
    pub fn get_platform(&self) -> &dyn Platform {
        &*self.platform
    }
    
    pub fn get_config(&self) -> crate::platform::Config {
        self.platform.load_config().unwrap_or_else(|_| {
            crate::platform::Config {
                general: crate::platform::GeneralConfig {
                    start_with_system: false,
                    check_for_updates: true,
                    last_update_check: None,
                },
                keyboards: crate::platform::KeyboardsConfig {
                    active: None,
                    last_used: vec![],
                    installed: vec![],
                },
                composition_mode: Default::default(),
            }
        })
    }
    
    pub fn save_config(&self, config: &crate::platform::Config) -> Result<()> {
        self.platform.save_config(config)
    }
    
    pub fn initialize(&self) -> Result<()> {
        // Load config
        let config = self.platform.load_config()?;
        
        // Load keyboards from config
        let mut keyboards = self.keyboards.lock().unwrap();
        for installed in &config.keyboards.installed {
            let path = self.platform.get_keyboards_dir().join(&installed.filename);
            if path.exists() {
                keyboards.insert(
                    installed.id.clone(),
                    KeyboardInfo {
                        id: installed.id.clone(),
                        name: installed.name.clone(),
                        filename: installed.filename.clone(),
                        path,
                        hotkey: installed.hotkey.clone(),
                        hash: installed.hash.clone(),
                        is_active: false,
                    },
                );
            }
        }
        
        // Set active keyboard
        if let Some(active_id) = config.keyboards.active {
            drop(keyboards); // Release lock before calling set_active_keyboard
            self.set_active_keyboard(&active_id)?;
        }
        
        Ok(())
    }
    
    pub fn scan_keyboards(&self) -> Result<Vec<KeyboardInfo>> {
        let keyboard_files = self.platform.get_keyboard_files()?;
        let mut found_keyboards = Vec::new();
        
        for path in keyboard_files {
            if let Ok(layout) = self.load_keyboard_file(&path) {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown.km2")
                    .to_string();
                
                let id = path.file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let name = layout.metadata().name().unwrap_or(id.clone());
                let hash = self.calculate_file_hash(&path)?;
                
                found_keyboards.push(KeyboardInfo {
                    id,
                    name,
                    filename,
                    path: path.clone(),
                    hotkey: None,
                    hash,
                    is_active: false,
                });
            }
        }
        
        Ok(found_keyboards)
    }
    
    pub fn add_keyboard(&self, keyboard_info: KeyboardInfo) -> Result<()> {
        let mut keyboards = self.keyboards.lock().unwrap();
        keyboards.insert(keyboard_info.id.clone(), keyboard_info.clone());
        drop(keyboards);
        
        // Update config
        self.save_keyboards_to_config()?;
        
        Ok(())
    }
    
    pub fn remove_keyboard(&self, keyboard_id: &str) -> Result<()> {
        let mut keyboards = self.keyboards.lock().unwrap();
        keyboards.remove(keyboard_id);
        drop(keyboards);
        
        // If this was the active keyboard, clear it
        let mut active = self.active_keyboard.lock().unwrap();
        if active.as_ref() == Some(&keyboard_id.to_string()) {
            *active = None;
            *self.engine.lock().unwrap() = None;
        }
        drop(active);
        
        // Update config
        self.save_keyboards_to_config()?;
        
        // Unregister from platform if supported
        if self.platform.supports_language_profiles() {
            self.platform.unregister_language_profile(keyboard_id)?;
        }
        
        Ok(())
    }
    
    pub fn set_active_keyboard(&self, keyboard_id: &str) -> Result<()> {
        let keyboards = self.keyboards.lock().unwrap();
        
        if let Some(keyboard_info) = keyboards.get(keyboard_id) {
            let layout = self.load_keyboard_file(&keyboard_info.path)?;
            
            // Update engine
            let mut engine_lock = self.engine.lock().unwrap();
            *engine_lock = Some(KeyMagicEngine::new(layout)?);
            drop(engine_lock);
            
            // Update active keyboard
            let mut active = self.active_keyboard.lock().unwrap();
            *active = Some(keyboard_id.to_string());
            drop(active);
            
            // Update is_active flag
            drop(keyboards);
            self.update_active_flags(keyboard_id)?;
            
            // Notify platform
            self.platform.switch_keyboard(keyboard_id)?;
            
            // Update config
            self.save_keyboards_to_config()?;
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Keyboard not found: {}", keyboard_id))
        }
    }
    
    pub fn get_active_keyboard(&self) -> Option<String> {
        self.active_keyboard.lock().unwrap().clone()
    }
    
    pub fn get_keyboards(&self) -> Vec<KeyboardInfo> {
        self.keyboards.lock().unwrap().values().cloned().collect()
    }
    
    pub fn get_keyboard(&self, keyboard_id: &str) -> Option<KeyboardInfo> {
        self.keyboards.lock().unwrap().get(keyboard_id).cloned()
    }
    
    pub fn get_engine(&self) -> Arc<Mutex<Option<KeyMagicEngine>>> {
        self.engine.clone()
    }
    
    pub fn update_hotkey(&self, keyboard_id: &str, hotkey: Option<String>) -> Result<()> {
        let mut keyboards = self.keyboards.lock().unwrap();
        if let Some(keyboard) = keyboards.get_mut(keyboard_id) {
            keyboard.hotkey = hotkey;
        }
        drop(keyboards);
        
        self.save_keyboards_to_config()?;
        Ok(())
    }
    
    pub fn import_keyboard(&self, file_path: &Path) -> Result<KeyboardInfo> {
        // Load the keyboard to validate it
        let layout = self.load_keyboard_file(file_path)?;
        
        // Generate keyboard info
        let filename = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.km2")
            .to_string();
        
        let id = file_path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let name = layout.metadata().name().unwrap_or(id.clone());
        let hash = self.calculate_file_hash(file_path)?;
        
        // Copy to keyboards directory
        let dest_path = self.platform.get_keyboards_dir().join(&filename);
        if dest_path != file_path {
            fs::create_dir_all(dest_path.parent().unwrap())?;
            fs::copy(file_path, &dest_path)?;
        }
        
        let keyboard_info = KeyboardInfo {
            id: id.clone(),
            name,
            filename,
            path: dest_path,
            hotkey: None,
            hash,
            is_active: false,
        };
        
        // Add to manager
        self.add_keyboard(keyboard_info.clone())?;
        
        // Register with platform if supported
        if self.platform.supports_language_profiles() {
            self.platform.register_language_profile(&id)?;
        }
        
        Ok(keyboard_info)
    }
    
    fn load_keyboard_file(&self, path: &Path) -> Result<Km2File> {
        let data = fs::read(path)
            .context("Failed to read keyboard file")?;
        
        Km2Loader::load(&data)
            .context("Failed to parse keyboard file")
    }
    
    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        use std::io::Read;
        use sha2::{Sha256, Digest};
        
        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];
        
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    fn update_active_flags(&self, active_id: &str) -> Result<()> {
        let mut keyboards = self.keyboards.lock().unwrap();
        for (id, keyboard) in keyboards.iter_mut() {
            keyboard.is_active = id == active_id;
        }
        Ok(())
    }
    
    fn save_keyboards_to_config(&self) -> Result<()> {
        let mut config = self.platform.load_config()?;
        
        // Update active keyboard
        config.keyboards.active = self.active_keyboard.lock().unwrap().clone();
        
        // Update installed keyboards
        let keyboards = self.keyboards.lock().unwrap();
        config.keyboards.installed = keyboards
            .values()
            .map(|kb| InstalledKeyboard {
                id: kb.id.clone(),
                name: kb.name.clone(),
                filename: kb.filename.clone(),
                hotkey: kb.hotkey.clone(),
                hash: kb.hash.clone(),
            })
            .collect();
        
        self.platform.save_config(&config)?;
        Ok(())
    }
}