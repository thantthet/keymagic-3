use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::ffi::CString;
use anyhow::{Result, anyhow};
use crate::registry_notifier::RegistryNotifier;
use crate::app_paths::AppPaths;
use crate::registry;
use crate::hotkey::normalize_hotkey;
use sha2::{Sha256, Digest};
use std::io::Read;
use log::{info, debug, warn};

// Import FFI types from keymagic-core
use keymagic_core::ffi::*;
use keymagic_core::km2::Km2Loader;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyboardInfo {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub description: String,
    pub icon_data: Option<Vec<u8>>,
    pub default_hotkey: Option<String>,
    pub hotkey: Option<String>,
    pub enabled: bool,
    pub color: Option<String>,  // Hex color for keyboards without icons
    pub hash: Option<String>,    // SHA-256 hash of the keyboard file
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum KeyboardStatus {
    New,              // Not installed
    Unchanged,        // Same hash - no update needed
    Updated,          // Different hash - update available
    Modified,         // Installed but file missing/corrupted
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyboardComparison {
    pub id: String,
    pub name: String,
    pub bundled_path: PathBuf,
    pub bundled_hash: String,
    pub installed_hash: Option<String>,
    pub status: KeyboardStatus,
    pub icon_data: Option<Vec<u8>>,
}

pub struct KeyboardManager {
    keyboards: HashMap<String, KeyboardInfo>,
    active_keyboard: Option<String>,
    app_paths: AppPaths,
}


/// Generate a color for a keyboard based on its name
fn generate_keyboard_color(name: &str) -> String {
    // Predefined palette of distinct colors
    let colors = [
        "#2196F3", // Blue
        "#4CAF50", // Green
        "#FF9800", // Orange
        "#9C27B0", // Purple
        "#F44336", // Red
        "#00BCD4", // Cyan
        "#795548", // Brown
        "#607D8B", // Blue Grey
        "#E91E63", // Pink
        "#009688", // Teal
        "#FFC107", // Amber
        "#3F51B5", // Indigo
        "#8BC34A", // Light Green
        "#FF5722", // Deep Orange
        "#673AB7", // Deep Purple
    ];
    
    // Generate a simple hash from the name
    let mut hash = 0u32;
    for byte in name.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    
    // Select a color based on the hash
    let index = (hash as usize) % colors.len();
    colors[index].to_string()
}


impl KeyboardManager {
    pub fn new() -> Result<Self> {
        let app_paths = AppPaths::new()?;
        
        let mut manager = Self {
            keyboards: HashMap::new(),
            active_keyboard: None,
            app_paths,
        };
        
        // Load keyboards from registry on Windows
        #[cfg(target_os = "windows")]
        manager.load_from_registry()?;
        
        // Validate and clean up any missing keyboard files
        manager.validate_and_cleanup()?;
        
        Ok(manager)
    }
    
    pub fn load_keyboard(&mut self, path: &Path) -> Result<String> {
        // Read the .km2 file
        let km2_data = std::fs::read(path)?;
        
        // Parse the KM2 file to extract metadata
        let km2 = Km2Loader::load(&km2_data)?;
        let metadata = km2.metadata();
        
        // Extract metadata from km2 file
        let base_keyboard_id = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        // Copy the keyboard file to app data directory
        let (managed_path, keyboard_id) = self.app_paths.install_keyboard(path, &base_keyboard_id)?;
        
        // Validate that it can be loaded by the engine using the managed path
        let engine = keymagic_engine_new();
        if engine.is_null() {
            return Err(anyhow!("Failed to create engine"));
        }
        
        let c_path = CString::new(managed_path.to_str().unwrap())?;
        let result = keymagic_engine_load_keyboard(engine, c_path.as_ptr());
        
        // Clean up engine
        keymagic_engine_free(engine);
        
        if result != KeyMagicResult::Success {
            // Clean up the copied file on failure
            let _ = self.app_paths.uninstall_keyboard(&keyboard_id);
            return Err(anyhow!("Failed to load keyboard"));
        }
            
        let name = metadata.name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| keyboard_id.clone());
            
        let description = metadata.description()
            .map(|s| s.to_string())
            .unwrap_or_else(|| String::new());
            
        let default_hotkey = metadata.hotkey()
            .map(|s| normalize_hotkey(&s));
            
        let icon_data = metadata.icon()
            .map(|data| data.to_vec());
            
        // Generate a color if there's no icon
        let color = if icon_data.is_none() {
            Some(generate_keyboard_color(&name))
        } else {
            None
        };
        
        // Calculate hash of the keyboard file
        let hash = Self::calculate_hash(&managed_path).ok();
        
        let info = KeyboardInfo {
            id: keyboard_id.clone(),
            path: managed_path,  // Use the managed path, not the original
            name,
            description,
            icon_data,
            default_hotkey: default_hotkey.clone(),
            hotkey: None,
            enabled: true,
            color,
            hash,
        };
        
        self.keyboards.insert(keyboard_id.clone(), info);
        
        #[cfg(target_os = "windows")]
        self.save_to_registry(&keyboard_id)?;
        
        Ok(keyboard_id)
    }
    
    pub fn remove_keyboard(&mut self, id: &str) -> Result<()> {
        if self.keyboards.remove(id).is_some() {
            // Remove the keyboard file from app data directory
            self.app_paths.uninstall_keyboard(id)?;
            
            #[cfg(target_os = "windows")]
            self.remove_from_registry(id)?;
            
            // If this was the active keyboard, clear it
            if self.active_keyboard.as_ref() == Some(&id.to_string()) {
                self.active_keyboard = None;
                #[cfg(target_os = "windows")]
                self.save_active_keyboard()?;
            }
        }
        
        Ok(())
    }
    
    pub fn get_keyboards(&self) -> Vec<&KeyboardInfo> {
        self.keyboards.values().collect()
    }
    
    
    pub fn set_active_keyboard(&mut self, id: &str) -> Result<()> {
        if self.keyboards.contains_key(id) {
            self.active_keyboard = Some(id.to_string());
            #[cfg(target_os = "windows")]
            {
                debug!("[KeyboardManager] Setting active keyboard to: {}", id);
                self.save_active_keyboard()?;
                debug!("[KeyboardManager] Active keyboard saved to registry");
                
                // Notify TSF instances to reload via SendInput
                debug!("[KeyboardManager] Notifying TSF instances of keyboard change");
                RegistryNotifier::notify_registry_changed()?;
            }
        }
        Ok(())
    }
    
    pub fn get_active_keyboard(&self) -> Option<&str> {
        self.active_keyboard.as_deref()
    }
    
    pub fn is_key_processing_enabled(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            registry::get_key_processing_enabled().unwrap_or(false)
        }
        #[cfg(not(target_os = "windows"))]
        {
            true
        }
    }
    
    pub fn set_key_processing_enabled(&mut self, enabled: bool) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            debug!("[KeyboardManager] Setting key processing enabled: {}", enabled);
            registry::set_key_processing_enabled(enabled)
                .map_err(|e| anyhow!("Failed to set key processing enabled: {}", e))?;
            debug!("[KeyboardManager] Key processing enabled setting saved to registry");
            
            // Notify TSF instances to reload via SendInput
            debug!("[KeyboardManager] Notifying TSF instances of enabled state change");
            RegistryNotifier::notify_registry_changed()?;
            
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }
    
    
    // Windows-specific registry operations
    #[cfg(target_os = "windows")]
    fn load_from_registry(&mut self) -> Result<()> {
        // Load keyboards from registry
        let registry_keyboards = registry::load_keyboards()?;
        
        for reg_kb in registry_keyboards {
            // Skip keyboards with missing files
            let path = PathBuf::from(&reg_kb.path);
            if !path.exists() {
                warn!("[KeyboardManager] Skipping keyboard {} - file not found: {}", reg_kb.id, path.display());
                continue;
            }
            
            // Try to load default hotkey and icon from .km2 file
            let (default_hotkey, icon_data) = if let Ok(km2_data) = std::fs::read(&path) {
                if let Ok(km2) = Km2Loader::load(&km2_data) {
                    let metadata = km2.metadata();
                    (
                        metadata.hotkey().map(|s| normalize_hotkey(&s)),
                        metadata.icon().map(|data| data.to_vec())
                    )
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };
            
            // Generate a color if there's no icon and no color in registry
            let color = if icon_data.is_none() && reg_kb.color.is_none() {
                Some(generate_keyboard_color(&reg_kb.name))
            } else {
                reg_kb.color
            };
            
            // Calculate hash if not stored in registry
            let hash = reg_kb.hash.or_else(|| Self::calculate_hash(&path).ok());
            
            let info = KeyboardInfo {
                id: reg_kb.id.clone(),
                path,
                name: reg_kb.name,
                description: reg_kb.description,
                icon_data,
                default_hotkey,
                hotkey: reg_kb.hotkey.map(|h| normalize_hotkey(&h)),
                enabled: reg_kb.enabled,
                color,
                hash,
            };
            
            self.keyboards.insert(reg_kb.id, info);
        }
        
        // Load active keyboard
        self.active_keyboard = registry::get_active_keyboard()?;
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn save_to_registry(&self, keyboard_id: &str) -> Result<()> {
        let info = self.keyboards.get(keyboard_id)
            .ok_or_else(|| anyhow!("Keyboard not found"))?;
        
        let reg_kb = registry::RegistryKeyboard {
            id: keyboard_id.to_string(),
            path: info.path.to_string_lossy().to_string(),
            name: info.name.clone(),
            description: info.description.clone(),
            hotkey: info.hotkey.clone(),
            color: info.color.clone(),
            enabled: info.enabled,
            hash: info.hash.clone(),
        };
        
        registry::save_keyboard(&reg_kb)?;
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn remove_from_registry(&self, keyboard_id: &str) -> Result<()> {
        registry::remove_keyboard(keyboard_id)?;
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn save_active_keyboard(&self) -> Result<()> {
        if let Some(active) = &self.active_keyboard {
            registry::set_active_keyboard(Some(active.as_str()))
                .map_err(|e| anyhow!("Failed to save active keyboard: {}", e))?;
        } else {
            registry::set_active_keyboard(None)
                .map_err(|e| anyhow!("Failed to clear active keyboard: {}", e))?;
        }
        Ok(())
    }
    
    pub fn set_keyboard_hotkey(&mut self, id: &str, hotkey: Option<&str>) -> Result<()> {
        if let Some(keyboard) = self.keyboards.get_mut(id) {
            keyboard.hotkey = hotkey.map(|s| normalize_hotkey(s));
            
            #[cfg(target_os = "windows")]
            self.save_to_registry(id)?;
        } else {
            return Err(anyhow!("Keyboard not found"));
        }
        
        Ok(())
    }
    
    /// Validates that all registered keyboards have their files present
    /// and removes any keyboards whose files are missing
    pub fn validate_and_cleanup(&mut self) -> Result<()> {
        let mut missing_keyboards = Vec::new();
        
        // Check each keyboard for missing files
        for (id, info) in &self.keyboards {
            if !info.path.exists() {
                warn!("[KeyboardManager] Keyboard file missing: {} at {}", id, info.path.display());
                missing_keyboards.push(id.clone());
            }
        }
        
        // Store count before consuming the vector
        let removed_count = missing_keyboards.len();
        
        // Remove missing keyboards
        for id in missing_keyboards {
            warn!("[KeyboardManager] Removing missing keyboard from registry: {}", id);
            self.keyboards.remove(&id);
            
            #[cfg(target_os = "windows")]
            self.remove_from_registry(&id)?;
            
            // If this was the active keyboard, clear it
            if self.active_keyboard.as_ref() == Some(&id) {
                self.active_keyboard = None;
                #[cfg(target_os = "windows")]
                self.save_active_keyboard()?;
            }
        }
        
        if removed_count > 0 {
            info!("[KeyboardManager] Cleaned up {} missing keyboards", removed_count);
        }
        
        Ok(())
    }
    
    /// Calculate SHA-256 hash of a file
    pub fn calculate_hash(file_path: &Path) -> Result<String> {
        let mut file = std::fs::File::open(file_path)?;
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
    
    /// Compare bundled keyboards with installed keyboards
    pub fn compare_with_bundled(&self, bundled_keyboards: Vec<(String, PathBuf)>) -> Result<Vec<KeyboardComparison>> {
        let mut comparisons = Vec::new();
        
        for (keyboard_id, bundled_path) in bundled_keyboards {
            // Read the bundled keyboard file to get metadata
            let km2_data = std::fs::read(&bundled_path)?;
            let km2 = Km2Loader::load(&km2_data)?;
            let metadata = km2.metadata();
            
            let name = metadata.name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| keyboard_id.clone());
                
            let icon_data = metadata.icon()
                .map(|data| data.to_vec());
            
            // Calculate hash of bundled file
            let bundled_hash = Self::calculate_hash(&bundled_path)?;
            
            // Check if keyboard is installed by name
            let installed_info = self.get_keyboard_by_name(&name);
            
            let (installed_hash, status) = match installed_info {
                Some(info) => {
                    // Check if file still exists
                    if info.path.exists() {
                        // Get or calculate current hash
                        let current_hash = if let Some(stored_hash) = &info.hash {
                            stored_hash.clone()
                        } else {
                            Self::calculate_hash(&info.path)?
                        };
                        
                        // Compare hashes
                        let status = if current_hash == bundled_hash {
                            KeyboardStatus::Unchanged
                        } else {
                            KeyboardStatus::Updated
                        };
                        
                        (Some(current_hash), status)
                    } else {
                        // File is missing
                        (info.hash.clone(), KeyboardStatus::Modified)
                    }
                }
                None => (None, KeyboardStatus::New),
            };
            
            comparisons.push(KeyboardComparison {
                id: keyboard_id,
                name,
                bundled_path,
                bundled_hash,
                installed_hash,
                status,
                icon_data,
            });
        }
        
        Ok(comparisons)
    }
    
    /// Get keyboard by name (case-insensitive)
    pub fn get_keyboard_by_name(&self, name: &str) -> Option<&KeyboardInfo> {
        let name_lower = name.to_lowercase();
        self.keyboards.values()
            .find(|kb| kb.name.to_lowercase() == name_lower)
    }
    
    /// Get list of bundled keyboards from app installation directory
    pub fn get_bundled_keyboards(&self) -> Result<Vec<(String, PathBuf)>> {
        let app_dir = self.app_paths.get_app_install_dir()?;
        let keyboards_dir = app_dir.join("keyboards");
        
        let mut bundled_keyboards = Vec::new();
        
        if keyboards_dir.exists() {
            for entry in std::fs::read_dir(&keyboards_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|e| e.to_str()) == Some("km2") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        bundled_keyboards.push((stem.to_string(), path));
                    }
                }
            }
        }
        
        Ok(bundled_keyboards)
    }
}