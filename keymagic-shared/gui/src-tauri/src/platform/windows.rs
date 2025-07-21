use super::{
    CompositionModeConfig, Config, GeneralConfig, InstalledKeyboard, KeyboardsConfig,
    Platform, PlatformFeatures, PlatformInfo,
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

// Registry paths - matching original implementation
const KEYMAGIC_ROOT: &str = r"Software\KeyMagic";
const KEYBOARDS_KEY: &str = r"Software\KeyMagic\Keyboards";
const SETTINGS_KEY: &str = r"Software\KeyMagic\Settings";

// Registry value names
const DEFAULT_KEYBOARD_VALUE: &str = "DefaultKeyboard";
const KEY_PROCESSING_ENABLED_VALUE: &str = "KeyProcessingEnabled";

// Keyboard entry value names
const KEYBOARD_PATH_VALUE: &str = "Path";
const KEYBOARD_NAME_VALUE: &str = "Name";
const KEYBOARD_DESCRIPTION_VALUE: &str = "Description";
const KEYBOARD_HOTKEY_VALUE: &str = "Hotkey";
const KEYBOARD_ENABLED_VALUE: &str = "Enabled";
const KEYBOARD_HASH_VALUE: &str = "Hash";

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
            },
            keyboards: KeyboardsConfig {
                active: None,
                last_used: Vec::new(),
                installed: Vec::new(),
            },
            composition_mode: CompositionModeConfig {
                enabled_processes: vec![
                    "firefox.exe".to_string(),
                    "chrome.exe".to_string(),
                    "Code.exe".to_string(),
                ],
            },
        }
    }
}

/// Notify Windows TSF (Text Services Framework) about registry changes using events
#[cfg(target_os = "windows")]
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
            
            // Active keyboard is stored with DefaultKeyboard name
            if let Ok(active) = settings_key.get_value::<String, _>(DEFAULT_KEYBOARD_VALUE) {
                config.keyboards.active = Some(active);
            }
        }
        
        // Load installed keyboards from registry
        if let Ok(keyboards_key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(KEYBOARDS_KEY) {
            for name in keyboards_key.enum_keys().filter_map(Result::ok) {
                if let Ok(kb_key) = keyboards_key.open_subkey(&name) {
                    let keyboard = InstalledKeyboard {
                        id: name.clone(),
                        name: kb_key.get_value(KEYBOARD_NAME_VALUE).unwrap_or(name),
                        filename: kb_key.get_value(KEYBOARD_PATH_VALUE).unwrap_or_default(),
                        hotkey: kb_key.get_value(KEYBOARD_HOTKEY_VALUE).ok(),
                        hash: kb_key.get_value(KEYBOARD_HASH_VALUE).unwrap_or_default(),
                    };
                    config.keyboards.installed.push(keyboard);
                }
            }
        }
        
        // Load composition mode processes from Settings registry
        if let Ok(settings_key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(SETTINGS_KEY) {
            if let Ok(processes) = settings_key.get_value::<String, _>("CompositionModeProcesses") {
                // Split by semicolon or newline
                config.composition_mode.enabled_processes = processes
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
            kb_key.set_value(KEYBOARD_PATH_VALUE, &keyboard.filename)?;
            kb_key.set_value(KEYBOARD_HASH_VALUE, &keyboard.hash)?;
            kb_key.set_value(KEYBOARD_ENABLED_VALUE, &1u32)?; // Always enabled for now
            
            if let Some(ref hotkey) = keyboard.hotkey {
                kb_key.set_value(KEYBOARD_HOTKEY_VALUE, hotkey)?;
            }
        }
        
        // Save composition mode processes as multi-string
        if !config.composition_mode.enabled_processes.is_empty() {
            // Note: winreg doesn't support REG_MULTI_SZ directly, so we'll save as semicolon-delimited
            let processes_str = config.composition_mode.enabled_processes.join(";");
            settings_key.set_value("CompositionModeProcesses", &processes_str)?;
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
    
    fn supports_language_profiles(&self) -> bool {
        true // Windows supports TSF language profiles
    }
    
    fn supports_composition_mode(&self) -> bool {
        true // Windows supports composition mode
    }
    
    fn get_platform_name(&self) -> &'static str {
        "windows"
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
    
    fn should_scan_bundled_keyboards(&self) -> Result<bool> {
        let current_version = env!("CARGO_PKG_VERSION");
        
        // Check version-based approach
        if let Ok(settings_key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(SETTINGS_KEY) {
            if let Ok(last_version) = settings_key.get_value::<String, _>("LastScannedVersion") {
                // Compare versions - if current > last, should scan for new keyboards
                return Ok(super::compare_versions(&current_version, &last_version));
            }
        }
        
        // No version recorded = should scan
        Ok(true)
    }
    
    fn mark_bundled_keyboards_scanned(&self) -> Result<()> {
        // Update to use version-based tracking
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (settings_key, _) = hkcu
            .create_subkey(SETTINGS_KEY)
            .context("Failed to create Settings key")?;
        
        // Set current version
        settings_key.set_value("LastScannedVersion", &env!("CARGO_PKG_VERSION"))?;
        Ok(())
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