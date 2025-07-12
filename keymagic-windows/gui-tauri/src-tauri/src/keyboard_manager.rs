use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::ffi::CString;
use anyhow::{Result, anyhow};
use crate::registry_notifier::RegistryNotifier;
use crate::app_paths::AppPaths;

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
    app_paths: AppPaths,
}

/// Normalize hotkey string to consistent format
/// Examples: "ctrl+space" -> "Ctrl+Space", "CTRL + SHIFT + A" -> "Ctrl+Shift+A"
fn normalize_hotkey(hotkey: &str) -> String {
    // Split by common separators and filter out empty parts
    let parts: Vec<&str> = hotkey
        .split(|c| c == '+' || c == '-' || c == ' ')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    if parts.is_empty() {
        return String::new();
    }
    
    // Sort modifiers in consistent order: Ctrl, Shift, Alt, Win
    let mut modifiers = Vec::new();
    let mut main_keys = Vec::new();
    
    for part in parts {
        let normalized = normalize_key_part(part);
        match normalized.as_str() {
            "Ctrl" | "Shift" | "Alt" | "Win" => modifiers.push(normalized),
            _ => main_keys.push(normalized),
        }
    }
    
    // Sort modifiers in canonical order
    modifiers.sort_by_key(|m| match m.as_str() {
        "Ctrl" => 0,
        "Shift" => 1,
        "Alt" => 2,
        "Win" => 3,
        _ => 4,
    });
    
    // Combine modifiers and main keys
    let mut result = modifiers;
    result.extend(main_keys);
    
    result.join("+")
}

/// Normalize individual key part
fn normalize_key_part(part: &str) -> String {
    let lower = part.to_lowercase();
    
    // Common key mappings
    match lower.as_str() {
        // Modifiers
        "ctrl" | "control" | "ctl" => "Ctrl".to_string(),
        "shift" | "shft" => "Shift".to_string(),
        "alt" | "option" | "opt" => "Alt".to_string(),
        "cmd" | "command" | "win" | "windows" | "super" | "meta" => "Win".to_string(),
        
        // Special keys
        "space" | "spacebar" | "spc" => "Space".to_string(),
        "tab" => "Tab".to_string(),
        "enter" | "return" | "ret" => "Enter".to_string(),
        "esc" | "escape" => "Escape".to_string(),
        "backspace" | "back" | "bksp" => "Backspace".to_string(),
        "delete" | "del" => "Delete".to_string(),
        "insert" | "ins" => "Insert".to_string(),
        "home" => "Home".to_string(),
        "end" => "End".to_string(),
        "pageup" | "pgup" | "page_up" | "prior" => "PageUp".to_string(),
        "pagedown" | "pgdown" | "pgdn" | "page_down" | "next" => "PageDown".to_string(),
        
        // Arrow keys
        "left" | "arrowleft" | "arrow_left" | "leftarrow" => "Left".to_string(),
        "right" | "arrowright" | "arrow_right" | "rightarrow" => "Right".to_string(),
        "up" | "arrowup" | "arrow_up" | "uparrow" => "Up".to_string(),
        "down" | "arrowdown" | "arrow_down" | "downarrow" => "Down".to_string(),
        
        // Numpad
        "num0" | "numpad0" | "numpad_0" => "Numpad0".to_string(),
        "num1" | "numpad1" | "numpad_1" => "Numpad1".to_string(),
        "num2" | "numpad2" | "numpad_2" => "Numpad2".to_string(),
        "num3" | "numpad3" | "numpad_3" => "Numpad3".to_string(),
        "num4" | "numpad4" | "numpad_4" => "Numpad4".to_string(),
        "num5" | "numpad5" | "numpad_5" => "Numpad5".to_string(),
        "num6" | "numpad6" | "numpad_6" => "Numpad6".to_string(),
        "num7" | "numpad7" | "numpad_7" => "Numpad7".to_string(),
        "num8" | "numpad8" | "numpad_8" => "Numpad8".to_string(),
        "num9" | "numpad9" | "numpad_9" => "Numpad9".to_string(),
        
        // Function keys with various formats
        _ => {
            // Check for function keys (F1-F24)
            if let Some(num) = parse_function_key(&lower) {
                format!("F{}", num)
            }
            // Single character - uppercase it
            else if part.len() == 1 && part.chars().all(|c| c.is_alphabetic()) {
                part.to_uppercase()
            }
            // Digit keys
            else if part.len() == 1 && part.chars().all(|c| c.is_numeric()) {
                part.to_string()
            }
            // For anything else, use title case
            else {
                // First letter uppercase, rest lowercase
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            }
        }
    }
}

/// Parse function key from various formats (f1, F1, func1, function1, etc.)
fn parse_function_key(s: &str) -> Option<u8> {
    // Remove common prefixes
    let num_part = s
        .strip_prefix("f")
        .or_else(|| s.strip_prefix("func"))
        .or_else(|| s.strip_prefix("function"))
        .or_else(|| s.strip_prefix("fn"))
        .unwrap_or(s);
    
    // Try to parse the number
    num_part.parse::<u8>().ok().filter(|&n| n >= 1 && n <= 24)
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
            
        let info = KeyboardInfo {
            id: keyboard_id.clone(),
            path: managed_path,  // Use the managed path, not the original
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
    
    pub fn get_keyboard(&self, id: &str) -> Option<&KeyboardInfo> {
        self.keyboards.get(id)
    }
    
    pub fn set_active_keyboard(&mut self, id: &str) -> Result<()> {
        if self.keyboards.contains_key(id) {
            self.active_keyboard = Some(id.to_string());
            #[cfg(target_os = "windows")]
            {
                println!("[KeyboardManager] Setting active keyboard to: {}", id);
                self.save_active_keyboard()?;
                println!("[KeyboardManager] Active keyboard saved to registry");
                
                // Notify TSF instances to reload via SendInput
                println!("[KeyboardManager] Notifying TSF instances of keyboard change");
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
                    println!("[KeyboardManager] Setting key processing enabled: {}", enabled);
                    self.write_registry_dword(hkey, w!("KeyProcessingEnabled"), if enabled { 1 } else { 0 })?;
                    RegCloseKey(hkey);
                    println!("[KeyboardManager] Key processing enabled setting saved to registry");
                    
                    // Notify TSF instances to reload via SendInput
                    println!("[KeyboardManager] Notifying TSF instances of enabled state change");
                    RegistryNotifier::notify_registry_changed()?;
                    
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
                        let hotkey = self.read_registry_string(kb_hkey, w!("Hotkey"))
                            .map(|h| normalize_hotkey(&h));
                        let enabled = self.read_registry_dword(kb_hkey, w!("Enabled")).unwrap_or(1) != 0;
                        
                        // Try to load default hotkey and icon from .km2 file
                        let path_buf = PathBuf::from(&path);
                        let (default_hotkey, icon_data) = if path_buf.exists() {
                            if let Ok(km2_data) = std::fs::read(&path_buf) {
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
            keyboard.hotkey = hotkey.map(|s| normalize_hotkey(s));
            
            #[cfg(target_os = "windows")]
            self.save_to_registry(id)?;
        } else {
            return Err(anyhow!("Keyboard not found"));
        }
        
        Ok(())
    }
}