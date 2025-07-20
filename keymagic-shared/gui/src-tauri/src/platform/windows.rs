use super::{
    CompositionModeConfig, Config, GeneralConfig, InstalledKeyboard, KeyboardsConfig, Language,
    Platform, PlatformFeatures, PlatformInfo,
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

const REGISTRY_PATH: &str = r"Software\KeyMagic";
const SETTINGS_PATH: &str = r"Software\KeyMagic\Settings";
const KEYBOARDS_PATH: &str = r"Software\KeyMagic\Keyboards";

pub struct WindowsBackend {
    registry_key: RegKey,
}

impl WindowsBackend {
    pub fn new() -> Result<Self> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let registry_key = hkcu
            .create_subkey(REGISTRY_PATH)
            .context("Failed to open/create registry key")?
            .0;
        
        Ok(Self { registry_key })
    }
    
    fn default_config() -> Config {
        Config {
            general: GeneralConfig {
                start_with_system: false,
                check_for_updates: true,
                last_update_check: None,
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

impl Platform for WindowsBackend {
    fn load_config(&self) -> Result<Config> {
        let mut config = Self::default_config();
        
        // Load from registry
        if let Ok(start_with_system) = self.registry_key.get_value::<u32, _>("StartWithSystem") {
            config.general.start_with_system = start_with_system != 0;
        }
        
        if let Ok(check_updates) = self.registry_key.get_value::<u32, _>("CheckForUpdates") {
            config.general.check_for_updates = check_updates != 0;
        }
        
        if let Ok(last_check) = self.registry_key.get_value::<String, _>("LastUpdateCheck") {
            config.general.last_update_check = Some(last_check);
        }
        
        if let Ok(active) = self.registry_key.get_value::<String, _>("ActiveKeyboard") {
            config.keyboards.active = Some(active);
        }
        
        // Load installed keyboards from registry
        if let Ok(keyboards_key) = self.registry_key.open_subkey("Keyboards") {
            for name in keyboards_key.enum_keys().filter_map(Result::ok) {
                if let Ok(kb_key) = keyboards_key.open_subkey(&name) {
                    let keyboard = InstalledKeyboard {
                        id: name.clone(),
                        name: kb_key.get_value("DisplayName").unwrap_or(name),
                        filename: kb_key.get_value("FileName").unwrap_or_default(),
                        hotkey: kb_key.get_value("Hotkey").ok(),
                        hash: kb_key.get_value("Hash").unwrap_or_default(),
                    };
                    config.keyboards.installed.push(keyboard);
                }
            }
        }
        
        // Load composition mode processes from registry
        if let Ok(settings_key) = self.registry_key.open_subkey("Settings") {
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
        // Save to registry
        self.registry_key.set_value(
            "StartWithSystem",
            &(config.general.start_with_system as u32),
        )?;
        
        self.registry_key.set_value(
            "CheckForUpdates",
            &(config.general.check_for_updates as u32),
        )?;
        
        if let Some(ref last_check) = config.general.last_update_check {
            self.registry_key.set_value("LastUpdateCheck", last_check)?;
        }
        
        if let Some(ref active) = config.keyboards.active {
            self.registry_key.set_value("ActiveKeyboard", active)?;
        }
        
        // Save keyboards
        let keyboards_key = self.registry_key.create_subkey("Keyboards")?.0;
        
        for keyboard in &config.keyboards.installed {
            let kb_key = keyboards_key.create_subkey(&keyboard.id)?.0;
            kb_key.set_value("DisplayName", &keyboard.name)?;
            kb_key.set_value("FileName", &keyboard.filename)?;
            kb_key.set_value("Hash", &keyboard.hash)?;
            
            if let Some(ref hotkey) = keyboard.hotkey {
                kb_key.set_value("Hotkey", hotkey)?;
            }
        }
        
        // Save composition mode processes
        if !config.composition_mode.enabled_processes.is_empty() {
            let processes_str = config.composition_mode.enabled_processes.join(";");
            let settings_key = self.registry_key.create_subkey("Settings")?.0;
            settings_key.set_value("CompositionModeProcesses", &processes_str)?;
        }
        
        Ok(())
    }
    
    fn get_keyboards_dir(&self) -> PathBuf {
        // Get from registry or use default
        if let Ok(path) = self.registry_key.get_value::<String, _>("KeyboardsPath") {
            PathBuf::from(path)
        } else {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
                .join("KeyMagic")
                .join("Keyboards")
        }
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
        // Send notification to Windows IME (TSF)
        // This would involve sending a message to the TSF text service
        // For now, just update the registry
        self.registry_key.set_value("ActiveKeyboard", &keyboard_id)?;
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
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"))
            .join("KeyMagic")
    }
    
    fn get_data_dir(&self) -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
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
    
    fn get_system_languages(&self) -> Result<Vec<Language>> {
        let mut languages = Vec::new();
        
        // TODO: Implement proper Windows language enumeration
        // For now, return a basic set
        languages.push(Language {
            id: "en".to_string(),
            name: "English".to_string(),
            code: "en".to_string(),
        });
        
        languages.push(Language {
            id: "my".to_string(),
            name: "Myanmar".to_string(),
            code: "my".to_string(),
        });
        
        Ok(languages)
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
        match key {
            "StartWithWindows" => {
                // Check autostart status using tauri plugin
                if let Ok(autostart_manager) = tauri_plugin_autostart::WindowsAutoLaunch::new("KeyMagic") {
                    match autostart_manager.is_enabled() {
                        Ok(enabled) => Ok(Some(if enabled { "1" } else { "0" }.to_string())),
                        Err(_) => Ok(Some("0".to_string())),
                    }
                } else {
                    Ok(Some("0".to_string()))
                }
            }
            _ => {
                // Read from registry
                if let Ok(value) = self.registry_key.get_value::<String, _>(key) {
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
        }
    }
    
    fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        match key {
            "StartWithWindows" => {
                // Let autostart plugin handle this
                // Don't save to registry
                Ok(())
            }
            _ => {
                // Save to registry
                self.registry_key.set_value(key, &value)?;
                Ok(())
            }
        }
    }
    
    fn is_first_run(&self) -> Result<bool> {
        // Check if FirstRunScanKeyboards flag is set
        match self.registry_key.get_value::<u32, _>("FirstRunScanKeyboards") {
            Ok(val) => Ok(val != 0),
            Err(_) => Ok(true), // If key doesn't exist, it's first run
        }
    }
    
    fn clear_first_run_flag(&self) -> Result<()> {
        // Clear the first run flag
        self.registry_key.set_value("FirstRunScanKeyboards", &0u32)?;
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
}