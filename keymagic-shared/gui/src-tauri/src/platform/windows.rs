use super::{
    CompositionModeConfig, DirectModeConfig, Config, GeneralConfig, InstalledKeyboard, KeyboardsConfig,
    Platform, PlatformFeatures, PlatformInfo,
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

use windows::Win32::System::Registry::{
    RegQueryValueExW, RegSetValueExW, REG_MULTI_SZ, REG_VALUE_TYPE,
};
use windows::core::PCWSTR;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

// Registry paths - matching original implementation
const KEYMAGIC_ROOT: &str = r"Software\KeyMagic";
const KEYBOARDS_KEY: &str = r"Software\KeyMagic\Keyboards";
const SETTINGS_KEY: &str = r"Software\KeyMagic\Settings";

// Registry value names
const DEFAULT_KEYBOARD_VALUE: &str = "DefaultKeyboard";
const KEY_PROCESSING_ENABLED_VALUE: &str = "KeyProcessingEnabled";

// Keyboard entry value names
const KEYBOARD_PATH_VALUE: &str = "Path";  // Legacy name for backward compatibility
const KEYBOARD_FILENAME_VALUE: &str = "FileName";  // New name for filename storage
const KEYBOARD_NAME_VALUE: &str = "Name";
const KEYBOARD_DESCRIPTION_VALUE: &str = "Description";
const KEYBOARD_HOTKEY_VALUE: &str = "Hotkey";
const KEYBOARD_ENABLED_VALUE: &str = "Enabled";
const KEYBOARD_HASH_VALUE: &str = "Hash";

/// Helper function to read REG_MULTI_SZ values from registry
fn read_multi_string_value(key: &RegKey, value_name: &str) -> Result<Vec<String>> {
    use windows::Win32::Foundation::ERROR_MORE_DATA;
    
    // Get the raw handle from winreg
    let hkey = unsafe { std::mem::transmute::<isize, windows::Win32::System::Registry::HKEY>(key.raw_handle()) };
    
    // Convert value name to wide string
    let value_name_wide: Vec<u16> = OsStr::new(value_name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    // First, get the required buffer size
    let mut data_type: REG_VALUE_TYPE = REG_VALUE_TYPE(0);
    let mut data_size: u32 = 0;
    
    unsafe {
        let result = RegQueryValueExW(
            hkey,
            PCWSTR::from_raw(value_name_wide.as_ptr()),
            None,
            Some(&mut data_type),
            None,
            Some(&mut data_size),
        );
        
        if result != ERROR_MORE_DATA && result.is_err() {
            return Err(anyhow::anyhow!("Failed to query registry value size"));
        }
        
        if data_type != REG_MULTI_SZ {
            return Err(anyhow::anyhow!("Registry value is not REG_MULTI_SZ"));
        }
    }
    
    // Allocate buffer and read the data
    let mut buffer: Vec<u16> = vec![0; (data_size / 2) as usize];
    
    unsafe {
        let result = RegQueryValueExW(
            hkey,
            PCWSTR::from_raw(value_name_wide.as_ptr()),
            None,
            Some(&mut data_type),
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut data_size),
        );
        
        if result.is_err() {
            return Err(anyhow::anyhow!("Failed to read registry value"));
        }
    }
    
    // Parse the multi-string (double-null terminated)
    let mut strings = Vec::new();
    let mut start = 0;
    
    for i in 0..buffer.len() {
        if buffer[i] == 0 {
            if start < i {
                let s = String::from_utf16(&buffer[start..i])
                    .map_err(|_| anyhow::anyhow!("Invalid UTF-16 string"))?;
                if !s.is_empty() {
                    strings.push(s);
                }
            }
            start = i + 1;
            
            // Check for double null terminator
            if i + 1 < buffer.len() && buffer[i + 1] == 0 {
                break;
            }
        }
    }
    
    Ok(strings)
}

/// Helper function to write REG_MULTI_SZ values to registry
fn write_multi_string_value(key: &RegKey, value_name: &str, values: &[String]) -> Result<()> {
    // Get the raw handle from winreg
    let hkey = unsafe { std::mem::transmute::<isize, windows::Win32::System::Registry::HKEY>(key.raw_handle()) };
    
    // Convert value name to wide string
    let value_name_wide: Vec<u16> = OsStr::new(value_name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    // Build the multi-string buffer
    let mut buffer: Vec<u16> = Vec::new();
    
    for value in values {
        let wide: Vec<u16> = OsStr::new(value).encode_wide().collect();
        buffer.extend(wide);
        buffer.push(0); // Null terminator for each string
    }
    buffer.push(0); // Double null terminator
    
    // Write to registry
    unsafe {
        // Convert u16 buffer to u8 slice
        let byte_slice = std::slice::from_raw_parts(
            buffer.as_ptr() as *const u8,
            buffer.len() * 2
        );
        
        let result = RegSetValueExW(
            hkey,
            PCWSTR::from_raw(value_name_wide.as_ptr()),
            0,
            REG_MULTI_SZ,
            Some(byte_slice),
        );
        
        if result.is_err() {
            return Err(anyhow::anyhow!("Failed to write registry value"));
        }
    }
    
    Ok(())
}


pub struct WindowsBackend {
    registry_key: RegKey,
}

impl WindowsBackend {
    pub fn new() -> Result<Self> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        
        // Ensure registry structure exists
        let (registry_key, _) = hkcu
            .create_subkey(KEYMAGIC_ROOT)
            .context("Failed to create KeyMagic root key")?;
        
        // Create Keyboards subkey
        hkcu.create_subkey(KEYBOARDS_KEY)
            .context("Failed to create Keyboards key")?;
        
        // Create Settings subkey
        hkcu.create_subkey(SETTINGS_KEY)
            .context("Failed to create Settings key")?;
        
        Ok(Self { registry_key })
    }
    
    fn default_config() -> Config {
        Config {
            general: GeneralConfig {
                start_with_system: false,
                check_for_updates: true,
                last_update_check: None,
                last_scanned_version: None,
                update_remind_after: None,
            },
            keyboards: KeyboardsConfig {
                active: None,
                last_used: Vec::new(),
                installed: Vec::new(),
            },
            composition_mode: CompositionModeConfig {
                enabled_hosts: vec![
                    "ms-teams.exe".to_string(),
                ],
            },
            direct_mode: DirectModeConfig {
                enabled_hosts: vec![],
            },
        }
    }
}

/// Notify Windows TSF (Text Services Framework) about registry changes using events
fn notify_registry_change() -> Result<()> {
    use log::{debug, error};
    
    debug!("[Registry Notifier] Sending registry update notification to TSF instances via Windows Event");
    
    // Use the Windows Event approach for registry update notification
    match crate::windows_event::WindowsEvent::create_or_open() {
        Ok(event) => {
            // Signal the event to notify TSF service about registry changes
            event.signal()
                .map_err(|e| anyhow::anyhow!("Failed to signal registry update event: {:?}", e))?;
            debug!("[Registry Notifier] Registry update event signaled successfully");
        }
        Err(e) => {
            error!("[Registry Notifier] Failed to create/open registry update event: {:?}", e);
            return Err(anyhow::anyhow!("Failed to create/open registry update event: {:?}", e));
        }
    }
    
    Ok(())
}

impl Platform for WindowsBackend {
    fn load_config(&self) -> Result<Config> {
        let mut config = Self::default_config();
        
        // Load from Settings key
        if let Ok(settings_key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(SETTINGS_KEY) {
            // Load general settings
            if let Ok(start_with_system) = settings_key.get_value::<u32, _>("StartWithSystem") {
                config.general.start_with_system = start_with_system != 0;
            }
            
            if let Ok(check_updates) = settings_key.get_value::<u32, _>("CheckForUpdates") {
                config.general.check_for_updates = check_updates != 0;
            }
            
            if let Ok(last_check) = settings_key.get_value::<String, _>("LastUpdateCheck") {
                config.general.last_update_check = Some(last_check);
            }
            
            if let Ok(remind_after) = settings_key.get_value::<String, _>("UpdateRemindAfter") {
                config.general.update_remind_after = Some(remind_after);
            }
            
            if let Ok(last_scanned) = settings_key.get_value::<String, _>("LastScannedVersion") {
                config.general.last_scanned_version = Some(last_scanned);
            }
            
            // Active keyboard is stored with DefaultKeyboard name
            if let Ok(active) = settings_key.get_value::<String, _>(DEFAULT_KEYBOARD_VALUE) {
                config.keyboards.active = Some(active);
            }
        }
        
        // Load installed keyboards from registry
        if let Ok(keyboards_key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(KEYBOARDS_KEY) {
            for name in keyboards_key.enum_keys().filter_map(Result::ok) {
                if let Ok(kb_key) = keyboards_key.open_subkey(&name) {
                    // Try to read from new FileName value first, fall back to Path for compatibility
                    let filename = if let Ok(filename) = kb_key.get_value::<String, _>(KEYBOARD_FILENAME_VALUE) {
                        // Found the new FileName value
                        filename
                    } else if let Ok(path_value) = kb_key.get_value::<String, _>(KEYBOARD_PATH_VALUE) {
                        // Fall back to old Path value for backward compatibility
                        // Extract filename from path if it's a full path
                        if path_value.contains('\\') || path_value.contains('/') {
                            // It's a full path, extract filename
                            PathBuf::from(&path_value)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&path_value)
                                .to_string()
                        } else {
                            // It's already a filename
                            path_value
                        }
                    } else {
                        // No filename or path found, skip this keyboard
                        continue;
                    };
                    
                    let keyboard = InstalledKeyboard {
                        id: name.clone(),
                        name: kb_key.get_value(KEYBOARD_NAME_VALUE).unwrap_or(name),
                        filename,
                        hotkey: kb_key.get_value(KEYBOARD_HOTKEY_VALUE).ok(),
                        hash: kb_key.get_value(KEYBOARD_HASH_VALUE).unwrap_or_default(),
                    };
                    config.keyboards.installed.push(keyboard);
                }
            }
        }
        
        // Load composition mode hosts from Settings registry
        if let Ok(settings_key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(SETTINGS_KEY) {
            // Try to read as REG_MULTI_SZ first using Windows API
            if let Ok(hosts) = read_multi_string_value(&settings_key, "CompositionModeHosts") {
                config.composition_mode.enabled_hosts = hosts;
            } else if let Ok(hosts) = settings_key.get_value::<String, _>("CompositionModeHosts") {
                // Fallback to semicolon-delimited string for backward compatibility
                config.composition_mode.enabled_hosts = hosts
                    .split(|c| c == ';' || c == '\n')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().to_string())
                    .collect();
            }
        }
        
        Ok(config)
    }
    
    fn save_config(&self, config: &Config) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        
        // Save to Settings key
        let (settings_key, _) = hkcu
            .create_subkey(SETTINGS_KEY)
            .context("Failed to create Settings key")?;
        
        settings_key.set_value(
            "StartWithSystem",
            &(config.general.start_with_system as u32),
        )?;
        
        settings_key.set_value(
            "CheckForUpdates",
            &(config.general.check_for_updates as u32),
        )?;
        
        if let Some(ref last_check) = config.general.last_update_check {
            settings_key.set_value("LastUpdateCheck", last_check)?;
        }
        
        if let Some(ref remind_after) = config.general.update_remind_after {
            settings_key.set_value("UpdateRemindAfter", remind_after)?;
        }
        
        if let Some(ref last_scanned) = config.general.last_scanned_version {
            settings_key.set_value("LastScannedVersion", last_scanned)?;
        }
        
        // Active keyboard uses DefaultKeyboard name
        if let Some(ref active) = config.keyboards.active {
            settings_key.set_value(DEFAULT_KEYBOARD_VALUE, active)?;
        }
        
        // Save keyboards
        let (keyboards_key, _) = hkcu
            .create_subkey(KEYBOARDS_KEY)
            .context("Failed to create Keyboards key")?;
        
        for keyboard in &config.keyboards.installed {
            let (kb_key, _) = keyboards_key.create_subkey(&keyboard.id)?;
            kb_key.set_value(KEYBOARD_NAME_VALUE, &keyboard.name)?;
            
            // Always save filename only (not full path)
            // Extract filename if it happens to be a full path
            let filename_only = if keyboard.filename.contains('\\') || keyboard.filename.contains('/') {
                PathBuf::from(&keyboard.filename)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&keyboard.filename)
                    .to_string()
            } else {
                keyboard.filename.clone()
            };
            
            // Write to new FileName value
            kb_key.set_value(KEYBOARD_FILENAME_VALUE, &filename_only)?;
            
            // Delete old Path value if it exists (cleanup migration)
            let _ = kb_key.delete_value(KEYBOARD_PATH_VALUE);
            
            kb_key.set_value(KEYBOARD_HASH_VALUE, &keyboard.hash)?;
            kb_key.set_value(KEYBOARD_ENABLED_VALUE, &1u32)?; // Always enabled for now
            
            if let Some(ref hotkey) = keyboard.hotkey {
                kb_key.set_value(KEYBOARD_HOTKEY_VALUE, hotkey)?;
            }
        }
        
        // Save composition mode hosts as REG_MULTI_SZ
        if !config.composition_mode.enabled_hosts.is_empty() {
            write_multi_string_value(&settings_key, "CompositionModeHosts", &config.composition_mode.enabled_hosts)?;
        }
        
        Ok(())
    }
    
    fn get_keyboards_dir(&self) -> PathBuf {
        // Try to get from Settings registry first
        if let Ok(settings_key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(SETTINGS_KEY) {
            if let Ok(path) = settings_key.get_value::<String, _>("KeyboardsPath") {
                return PathBuf::from(path);
            }
        }
        
        // Default to %LOCALAPPDATA%\KeyMagic\Keyboards
        dirs::data_local_dir()
            .or_else(|| {
                // Fallback to environment variable if dirs crate fails
                std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
            })
            .unwrap_or_else(|| {
                // Last resort fallback
                PathBuf::from("C:\\Users\\Default\\AppData\\Local")
            })
            .join("KeyMagic")
            .join("Keyboards")
    }
    
    fn get_keyboard_files(&self) -> Result<Vec<PathBuf>> {
        let keyboards_dir = self.get_keyboards_dir();
        let mut keyboards = Vec::new();
        
        if keyboards_dir.exists() {
            for entry in std::fs::read_dir(&keyboards_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("km2") {
                    keyboards.push(path);
                }
            }
        }
        
        Ok(keyboards)
    }
    
    fn notify_ime_update(&self, keyboard_id: &str) -> Result<()> {
        // Update the active keyboard in Settings
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (settings_key, _) = hkcu
            .create_subkey(SETTINGS_KEY)
            .context("Failed to open Settings key")?;
        
        settings_key.set_value(DEFAULT_KEYBOARD_VALUE, &keyboard_id)?;
        
        // Send notification to TSF text service about registry changes
        notify_registry_change()?;
        
        Ok(())
    }
    
    fn is_ime_running(&self) -> bool {
        // Check if KeyMagic TSF service is running
        // This would involve checking if the TSF text service is loaded
        true // For now, assume it's running
    }
    
    fn switch_keyboard(&self, keyboard_id: &str) -> Result<()> {
        self.notify_ime_update(keyboard_id)
    }
    
    fn get_config_dir(&self) -> PathBuf {
        // Use %LOCALAPPDATA% for config as well, matching the original implementation
        dirs::data_local_dir()
            .or_else(|| {
                std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
            })
            .unwrap_or_else(|| {
                PathBuf::from("C:\\Users\\Default\\AppData\\Local")
            })
            .join("KeyMagic")
    }
    
    fn get_data_dir(&self) -> PathBuf {
        // Use %LOCALAPPDATA% for data directory as well
        dirs::data_local_dir()
            .or_else(|| {
                std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
            })
            .unwrap_or_else(|| {
                PathBuf::from("C:\\Users\\Default\\AppData\\Local")
            })
            .join("KeyMagic")
    }
    
    
    fn get_platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            os: "windows".to_string(),
            features: PlatformFeatures {
                language_profiles: true,
                composition_mode: true,
                global_hotkeys: true,
                system_tray: true,
            },
        }
    }
    
    fn register_language_profile(&self, _keyboard_id: &str) -> Result<()> {
        // Register with Windows TSF
        // This would involve registering the keyboard as a TSF text service
        Ok(())
    }
    
    fn unregister_language_profile(&self, _keyboard_id: &str) -> Result<()> {
        // Unregister from Windows TSF
        Ok(())
    }
    
    fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        
        match key {
            _ => {
                // Read from Settings key
                if let Ok(settings_key) = hkcu.open_subkey(SETTINGS_KEY) {
                    if let Ok(value) = settings_key.get_value::<String, _>(key) {
                        Ok(Some(value))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }
    
    fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (settings_key, _) = hkcu
            .create_subkey(SETTINGS_KEY)
            .context("Failed to create Settings key")?;
        
        match key {
            _ => {
                // Save to Settings key
                settings_key.set_value(key, &value)?;
                Ok(())
            }
        }
    }
    
    
    fn get_bundled_keyboards_path(&self) -> Option<PathBuf> {
        // Get the installation directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let bundled_path = parent.join("keyboards");
                if bundled_path.exists() {
                    return Some(bundled_path);
                }
            }
        }
        None
    }
    
    fn get_enabled_languages(&self) -> Result<Vec<String>> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        
        // Read EnabledLanguages from Settings key
        if let Ok(settings_key) = hkcu.open_subkey(SETTINGS_KEY) {
            if let Ok(languages_str) = settings_key.get_value::<String, _>("EnabledLanguages") {
                // Split by semicolon
                let languages: Vec<String> = languages_str
                    .split(';')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().to_string())
                    .collect();
                    
                if !languages.is_empty() {
                    return Ok(languages);
                }
            }
        }
        
        // Default to English if nothing is set
        Ok(vec!["en-US".to_string()])
    }
    
    fn set_enabled_languages(&self, languages: &[String]) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (settings_key, _) = hkcu
            .create_subkey(SETTINGS_KEY)
            .context("Failed to create Settings key")?;
        
        // Save as semicolon-delimited string
        let languages_str = languages.join(";");
        settings_key.set_value("EnabledLanguages", &languages_str)?;
        
        Ok(())
    }
}