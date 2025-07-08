use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::ffi::CString;
use anyhow::{Result, anyhow};

#[cfg(target_os = "windows")]
use windows::core::*;
#[cfg(target_os = "windows")]
use windows::Win32::System::Registry::*;

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
}

pub struct KeyboardManager {
    keyboards: HashMap<String, KeyboardInfo>,
    active_keyboard: Option<String>,
}

impl KeyboardManager {
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            keyboards: HashMap::new(),
            active_keyboard: None,
        };
        
        // Load keyboards from registry on Windows
        #[cfg(target_os = "windows")]
        manager.load_from_registry()?;
        
        Ok(manager)
    }
    
    pub fn load_keyboard(&mut self, path: &Path) -> Result<String> {
        // Read the .km2 file
        let km2_data = std::fs::read(path)?;
        
        // Parse the KM2 file to extract metadata
        let km2 = Km2Loader::load(&km2_data)?;
        let metadata = km2.metadata();
        
        // Also validate that it can be loaded by the engine
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
        let keyboard_id = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        let name = metadata.name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| keyboard_id.clone());
            
        let description = metadata.description()
            .map(|s| s.to_string())
            .unwrap_or_else(|| String::new());
            
        let default_hotkey = metadata.hotkey()
            .map(|s| s.to_string());
            
        let icon_data = metadata.icon()
            .map(|data| data.to_vec());
            
        let info = KeyboardInfo {
            id: keyboard_id.clone(),
            path: path.to_path_buf(),
            name,
            description,
            icon_data,
            default_hotkey: default_hotkey.clone(),
            hotkey: None,
            enabled: true,
        };
        
        self.keyboards.insert(keyboard_id.clone(), info);
        
        #[cfg(target_os = "windows")]
        self.save_to_registry(&keyboard_id)?;
        
        Ok(keyboard_id)
    }
    
    pub fn remove_keyboard(&mut self, id: &str) -> Result<()> {
        if self.keyboards.remove(id).is_some() {
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
    
    pub fn get_keyboard(&self, id: &str) -> Option<&KeyboardInfo> {
        self.keyboards.get(id)
    }
    
    pub fn set_active_keyboard(&mut self, id: &str) -> Result<()> {
        if self.keyboards.contains_key(id) {
            self.active_keyboard = Some(id.to_string());
            #[cfg(target_os = "windows")]
            self.save_active_keyboard()?;
        }
        Ok(())
    }
    
    pub fn get_active_keyboard(&self) -> Option<&str> {
        self.active_keyboard.as_deref()
    }
    
    pub fn is_key_processing_enabled(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                let mut hkey = HKEY::default();
                
                if RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    w!("Software\\KeyMagic\\Settings"),
                    0,
                    KEY_READ,
                    &mut hkey
                ).is_ok() {
                    let enabled = self.read_registry_dword(hkey, w!("KeyProcessingEnabled"))
                        .unwrap_or(1) != 0;
                    RegCloseKey(hkey);
                    enabled
                } else {
                    true
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            true
        }
    }
    
    pub fn set_key_processing_enabled(&mut self, enabled: bool) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                let mut hkey = HKEY::default();
                
                if RegCreateKeyW(
                    HKEY_CURRENT_USER,
                    w!("Software\\KeyMagic\\Settings"),
                    &mut hkey
                ).is_ok() {
                    self.write_registry_dword(hkey, w!("KeyProcessingEnabled"), if enabled { 1 } else { 0 })?;
                    RegCloseKey(hkey);
                    Ok(())
                } else {
                    Err(anyhow!("Failed to open registry key"))
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }
    
    pub fn get_setting(&self, key: &str) -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                let settings_path = format!("Software\\KeyMagic\\Settings\\{}", key);
                let wide_path: Vec<u16> = settings_path.encode_utf16().chain(std::iter::once(0)).collect();
                
                let mut hkey = HKEY::default();
                if RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    PCWSTR(wide_path.as_ptr()),
                    0,
                    KEY_READ,
                    &mut hkey
                ).is_ok() {
                    let value = self.read_registry_string(hkey, w!("")).unwrap_or_default();
                    RegCloseKey(hkey);
                    Ok(value)
                } else {
                    Err(anyhow!("Setting not found"))
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(anyhow!("Settings not supported on this platform"))
        }
    }
    
    // Windows-specific registry operations
    #[cfg(target_os = "windows")]
    fn load_from_registry(&mut self) -> Result<()> {
        unsafe {
            let key_path = w!("Software\\KeyMagic\\Keyboards");
            let mut hkey = HKEY::default();
            
            if RegOpenKeyExW(HKEY_CURRENT_USER, key_path, 0, KEY_READ, &mut hkey).is_ok() {
                let mut index = 0;
                let mut name_buffer = vec![0u16; 256];
                
                loop {
                    let mut name_len = name_buffer.len() as u32;
                    
                    let result = RegEnumKeyExW(
                        hkey,
                        index,
                        PWSTR(name_buffer.as_mut_ptr()),
                        &mut name_len,
                        None,
                        PWSTR::null(),
                        None,
                        None
                    );
                    
                    if result.is_err() {
                        break;
                    }
                    
                    let keyboard_id = String::from_utf16_lossy(&name_buffer[..name_len as usize]);
                    
                    // Load keyboard details
                    let mut kb_hkey = HKEY::default();
                    let kb_key_path = format!("Software\\KeyMagic\\Keyboards\\{}", keyboard_id);
                    let kb_key_path_w: Vec<u16> = kb_key_path.encode_utf16().chain(std::iter::once(0)).collect();
                    
                    if RegOpenKeyExW(HKEY_CURRENT_USER, PCWSTR::from_raw(kb_key_path_w.as_ptr()), 0, KEY_READ, &mut kb_hkey).is_ok() {
                        let path = self.read_registry_string(kb_hkey, w!("Path")).unwrap_or_default();
                        let name = self.read_registry_string(kb_hkey, w!("Name")).unwrap_or(keyboard_id.clone());
                        let description = self.read_registry_string(kb_hkey, w!("Description")).unwrap_or_default();
                        let hotkey = self.read_registry_string(kb_hkey, w!("Hotkey"));
                        let enabled = self.read_registry_dword(kb_hkey, w!("Enabled")).unwrap_or(1) != 0;
                        
                        // Try to load default hotkey and icon from .km2 file
                        let path_buf = PathBuf::from(&path);
                        let (default_hotkey, icon_data) = if path_buf.exists() {
                            if let Ok(km2_data) = std::fs::read(&path_buf) {
                                if let Ok(km2) = Km2Loader::load(&km2_data) {
                                    let metadata = km2.metadata();
                                    (
                                        metadata.hotkey().map(|s| s.to_string()),
                                        metadata.icon().map(|data| data.to_vec())
                                    )
                                } else {
                                    (None, None)
                                }
                            } else {
                                (None, None)
                            }
                        } else {
                            (None, None)
                        };
                        
                        let info = KeyboardInfo {
                            id: keyboard_id.clone(),
                            path: path_buf,
                            name,
                            description,
                            icon_data,
                            default_hotkey,
                            hotkey,
                            enabled,
                        };
                        
                        self.keyboards.insert(keyboard_id, info);
                        
                        RegCloseKey(kb_hkey);
                    }
                    
                    index += 1;
                }
                
                RegCloseKey(hkey);
            }
            
            // Load active keyboard
            let settings_key_path = w!("Software\\KeyMagic\\Settings");
            if RegOpenKeyExW(HKEY_CURRENT_USER, settings_key_path, 0, KEY_READ, &mut hkey).is_ok() {
                self.active_keyboard = self.read_registry_string(hkey, w!("DefaultKeyboard"));
                RegCloseKey(hkey);
            }
        }
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn save_to_registry(&self, keyboard_id: &str) -> Result<()> {
        let info = self.keyboards.get(keyboard_id)
            .ok_or_else(|| anyhow!("Keyboard not found"))?;
            
        unsafe {
            let key_path = format!("Software\\KeyMagic\\Keyboards\\{}", keyboard_id);
            let key_path_w: Vec<u16> = key_path.encode_utf16().chain(std::iter::once(0)).collect();
            let mut hkey = HKEY::default();
            
            if RegCreateKeyW(
                HKEY_CURRENT_USER,
                PCWSTR::from_raw(key_path_w.as_ptr()),
                &mut hkey
            ).is_ok() {
                self.write_registry_string(hkey, w!("Path"), &info.path.to_string_lossy())?;
                self.write_registry_string(hkey, w!("Name"), &info.name)?;
                self.write_registry_string(hkey, w!("Description"), &info.description)?;
                if let Some(hotkey) = &info.hotkey {
                    self.write_registry_string(hkey, w!("Hotkey"), hotkey)?;
                }
                self.write_registry_dword(hkey, w!("Enabled"), if info.enabled { 1 } else { 0 })?;
                
                RegCloseKey(hkey);
            }
        }
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn remove_from_registry(&self, keyboard_id: &str) -> Result<()> {
        unsafe {
            let key_path = format!("Software\\KeyMagic\\Keyboards\\{}", keyboard_id);
            let key_path_w: Vec<u16> = key_path.encode_utf16().chain(std::iter::once(0)).collect();
            
            RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR::from_raw(key_path_w.as_ptr()));
        }
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn save_active_keyboard(&self) -> Result<()> {
        unsafe {
            let mut hkey = HKEY::default();
            
            if RegCreateKeyW(
                HKEY_CURRENT_USER,
                w!("Software\\KeyMagic\\Settings"),
                &mut hkey
            ).is_ok() {
                if let Some(active) = &self.active_keyboard {
                    self.write_registry_string(hkey, w!("DefaultKeyboard"), active)?;
                }
                
                RegCloseKey(hkey);
            }
        }
        
        Ok(())
    }
    
    // Registry helper methods
    #[cfg(target_os = "windows")]
    unsafe fn read_registry_string(&self, hkey: HKEY, value_name: PCWSTR) -> Option<String> {
        let mut buffer = vec![0u16; 256];
        let mut size = buffer.len() as u32 * 2;
        let mut data_type = REG_VALUE_TYPE::default();
        
        let result = RegQueryValueExW(
            hkey,
            value_name,
            None,
            Some(&mut data_type),
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut size),
        );
        
        if result.is_ok() {
            let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
            buffer.truncate(len);
            Some(String::from_utf16_lossy(&buffer))
        } else {
            None
        }
    }
    
    #[cfg(target_os = "windows")]
    unsafe fn read_registry_dword(&self, hkey: HKEY, value_name: PCWSTR) -> Option<u32> {
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
    
    #[cfg(target_os = "windows")]
    unsafe fn write_registry_string(&self, hkey: HKEY, value_name: PCWSTR, value: &str) -> Result<()> {
        let value_w: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        let value_bytes = std::slice::from_raw_parts(
            value_w.as_ptr() as *const u8,
            value_w.len() * 2
        );
        
        if RegSetValueExW(
            hkey,
            value_name,
            0,
            REG_SZ,
            Some(value_bytes),
        ).is_err() {
            return Err(anyhow!("Failed to write registry value"));
        }
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    pub unsafe fn write_registry_dword(&self, hkey: HKEY, value_name: PCWSTR, value: u32) -> Result<()> {
        let value_bytes = std::slice::from_raw_parts(
            &value as *const u32 as *const u8,
            std::mem::size_of::<u32>()
        );
        
        if RegSetValueExW(
            hkey,
            value_name,
            0,
            REG_DWORD,
            Some(value_bytes),
        ).is_err() {
            return Err(anyhow!("Failed to write registry value"));
        }
        
        Ok(())
    }
    
    pub fn set_keyboard_hotkey(&mut self, id: &str, hotkey: Option<&str>) -> Result<()> {
        if let Some(keyboard) = self.keyboards.get_mut(id) {
            keyboard.hotkey = hotkey.map(|s| s.to_string());
            
            #[cfg(target_os = "windows")]
            self.save_to_registry(id)?;
        } else {
            return Err(anyhow!("Keyboard not found"));
        }
        
        Ok(())
    }
}