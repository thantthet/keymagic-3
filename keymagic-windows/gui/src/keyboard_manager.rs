use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::ffi::CString;
use windows::core::*;
use windows::Win32::System::Registry::*;
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
        let mut manager = Self {
            keyboards: HashMap::new(),
            active_keyboard: None,
        };
        
        // Load keyboards from registry
        manager.load_from_registry()?;
        
        Ok(manager)
    }
    
    pub fn load_keyboard(&mut self, path: &Path) -> Result<String> {
        // Read the .km2 file
        let km2_data = std::fs::read(path)?;
        
        // Use keymagic-core to parse the file and extract metadata
        // For now, we'll create a temporary engine to load and validate the keyboard
        let engine = unsafe { keymagic_engine_new() };
        if engine.is_null() {
            return Err(anyhow!("Failed to create engine"));
        }
        
        let c_path = CString::new(path.to_str().unwrap())?;
        let result = unsafe {
            keymagic_engine_load_keyboard(engine, c_path.as_ptr())
        };
        
        // Clean up engine
        unsafe { keymagic_engine_free(engine) };
        
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
        self.save_to_registry(&keyboard_id)?;
        
        Ok(keyboard_id)
    }
    
    pub fn remove_keyboard(&mut self, id: &str) -> Result<()> {
        if self.keyboards.remove(id).is_some() {
            self.remove_from_registry(id)?;
            
            // If this was the active keyboard, clear it
            if self.active_keyboard.as_ref() == Some(&id.to_string()) {
                self.active_keyboard = None;
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
            self.save_active_keyboard()?;
        }
        Ok(())
    }
    
    pub fn get_active_keyboard(&self) -> Option<&str> {
        self.active_keyboard.as_deref()
    }
    
    // Registry operations
    fn load_from_registry(&mut self) -> Result<()> {
        unsafe {
            let key_path = w!("Software\\KeyMagic\\Keyboards");
            let mut hkey = HKEY::default();
            
            if RegOpenKeyExW(HKEY_CURRENT_USER, key_path, 0, KEY_READ, &mut hkey) .is_ok() {
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
                    
                    if result .is_err() {
                        break;
                    }
                    
                    let keyboard_id = String::from_utf16_lossy(&name_buffer[..name_len as usize]);
                    
                    // Load keyboard details
                    let mut kb_hkey = HKEY::default();
                    let kb_key_path = format!("Software\\KeyMagic\\Keyboards\\{}", keyboard_id);
                    let kb_key_path_w: Vec<u16> = kb_key_path.encode_utf16().chain(std::iter::once(0)).collect();
                    
                    if RegOpenKeyExW(HKEY_CURRENT_USER, PCWSTR::from_raw(kb_key_path_w.as_ptr()), 0, KEY_READ, &mut kb_hkey) .is_ok() {
                        let path = self.read_registry_string(kb_hkey, w!("Path")).unwrap_or_default();
                        let name = self.read_registry_string(kb_hkey, w!("Name")).unwrap_or(keyboard_id.clone());
                        let description = self.read_registry_string(kb_hkey, w!("Description")).unwrap_or_default();
                        let hotkey = self.read_registry_string(kb_hkey, w!("Hotkey"));
                        let enabled = self.read_registry_dword(kb_hkey, w!("Enabled")).unwrap_or(1) != 0;
                        
                        let info = KeyboardInfo {
                            id: keyboard_id.clone(),
                            path: PathBuf::from(path),
                            name,
                            description,
                            icon_data: None, // TODO: Load from cache
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
            if RegOpenKeyExW(HKEY_CURRENT_USER, settings_key_path, 0, KEY_READ, &mut hkey) .is_ok() {
                self.active_keyboard = self.read_registry_string(hkey, w!("DefaultKeyboard"));
                RegCloseKey(hkey);
            }
        }
        
        Ok(())
    }
    
    fn save_to_registry(&self, keyboard_id: &str) -> Result<()> {
        let info = self.keyboards.get(keyboard_id)
            .ok_or_else(|| anyhow!("Keyboard not found"))?;
            
        unsafe {
            let key_path = format!("Software\\KeyMagic\\Keyboards\\{}", keyboard_id);
            let key_path_w: Vec<u16> = key_path.encode_utf16().chain(std::iter::once(0)).collect();
            let mut hkey = HKEY::default();
            
            // Use simplified registry function
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
    
    fn remove_from_registry(&self, keyboard_id: &str) -> Result<()> {
        unsafe {
            let key_path = format!("Software\\KeyMagic\\Keyboards\\{}", keyboard_id);
            let key_path_w: Vec<u16> = key_path.encode_utf16().chain(std::iter::once(0)).collect();
            
            RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR::from_raw(key_path_w.as_ptr()));
        }
        
        Ok(())
    }
    
    fn save_active_keyboard(&self) -> Result<()> {
        unsafe {
            let mut hkey = HKEY::default();
            
            // Use simplified registry function
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
    unsafe fn read_registry_string(&self, hkey: HKEY, value_name: PCWSTR) -> Option<String> {
        let mut data_type = REG_VALUE_TYPE::default();
        let mut data_size = 0u32;
        
        if RegQueryValueExW(hkey, value_name, None, Some(&mut data_type), None, Some(&mut data_size)) .is_ok() {
            let mut buffer = vec![0u8; data_size as usize];
            
            if RegQueryValueExW(hkey, value_name, None, None, Some(buffer.as_mut_ptr()), Some(&mut data_size)) .is_ok() {
                // Convert from UTF-16
                let u16_buffer: Vec<u16> = buffer.chunks_exact(2)
                    .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                    .take_while(|&c| c != 0)
                    .collect();
                    
                return Some(String::from_utf16_lossy(&u16_buffer));
            }
        }
        
        None
    }
    
    unsafe fn read_registry_dword(&self, hkey: HKEY, value_name: PCWSTR) -> Option<u32> {
        let mut data_type = REG_VALUE_TYPE::default();
        let mut data = 0u32;
        let mut data_size = std::mem::size_of::<u32>() as u32;
        
        if RegQueryValueExW(
            hkey,
            value_name,
            None,
            Some(&mut data_type),
            Some(std::slice::from_raw_parts_mut(&mut data as *mut u32 as *mut u8, std::mem::size_of::<u32>())),
            Some(&mut data_size)
        ) .is_ok() {
            Some(data)
        } else {
            None
        }
    }
    
    unsafe fn write_registry_string(&self, hkey: HKEY, value_name: PCWSTR, value: &str) -> Result<()> {
        let value_w: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        
        if RegSetValueExW(
            hkey,
            value_name,
            0,
            REG_SZ,
            Some(std::slice::from_raw_parts(value_w.as_ptr() as *const u8, value_w.len() * 2))
        ) .is_err() {
            return Err(anyhow!("Failed to write registry value"));
        }
        
        Ok(())
    }
    
    unsafe fn write_registry_dword(&self, hkey: HKEY, value_name: PCWSTR, value: u32) -> Result<()> {
        if RegSetValueExW(
            hkey,
            value_name,
            0,
            REG_DWORD,
            Some(std::slice::from_raw_parts(&value as *const u32 as *const u8, std::mem::size_of::<u32>()))
        ) .is_err() {
            return Err(anyhow!("Failed to write registry value"));
        }
        
        Ok(())
    }
}