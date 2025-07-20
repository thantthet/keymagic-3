use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsBackend as PlatformBackend;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxBackend as PlatformBackend;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacOSBackend as PlatformBackend;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub keyboards: KeyboardsConfig,
    #[serde(default)]
    pub composition_mode: CompositionModeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub start_with_system: bool,
    pub check_for_updates: bool,
    pub last_update_check: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardsConfig {
    pub active: Option<String>,
    pub last_used: Vec<String>,
    pub installed: Vec<InstalledKeyboard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledKeyboard {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub hotkey: Option<String>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompositionModeConfig {
    pub enabled_processes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub features: PlatformFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformFeatures {
    pub language_profiles: bool,
    pub composition_mode: bool,
    pub global_hotkeys: bool,
    pub system_tray: bool,
}

pub trait Platform: Send + Sync {
    // Configuration storage
    fn load_config(&self) -> Result<Config>;
    fn save_config(&self, config: &Config) -> Result<()>;
    
    // Keyboard management
    fn get_keyboards_dir(&self) -> PathBuf;
    fn get_keyboard_files(&self) -> Result<Vec<PathBuf>>;
    
    // IME integration
    fn notify_ime_update(&self, keyboard_id: &str) -> Result<()>;
    fn is_ime_running(&self) -> bool;
    fn switch_keyboard(&self, keyboard_id: &str) -> Result<()>;
    
    // System integration
    fn get_config_dir(&self) -> PathBuf;
    fn get_data_dir(&self) -> PathBuf;
    fn supports_language_profiles(&self) -> bool;
    fn supports_composition_mode(&self) -> bool;
    
    // Platform info
    fn get_platform_name(&self) -> &'static str;
    fn get_platform_info(&self) -> PlatformInfo;
    
    // Optional platform-specific methods with default implementations
    fn register_language_profile(&self, _keyboard_id: &str) -> Result<()> {
        Ok(())
    }
    
    fn unregister_language_profile(&self, _keyboard_id: &str) -> Result<()> {
        Ok(())
    }
    
    // Language profile management
    fn get_enabled_languages(&self) -> Result<Vec<String>> {
        Ok(vec!["en-US".to_string()]) // Default implementation
    }
    
    fn set_enabled_languages(&self, _languages: &[String]) -> Result<()> {
        Ok(()) // Default implementation - no-op
    }
    
    // Settings management
    fn get_setting(&self, key: &str) -> Result<Option<String>> {
        // Default implementation - can be overridden by platforms
        match key {
            "StartWithWindows" => {
                let config = self.load_config()?;
                Ok(Some(if config.general.start_with_system { "1" } else { "0" }.to_string()))
            }
            _ => Ok(None),
        }
    }
    
    fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        // Default implementation - can be overridden by platforms
        if key == "StartWithWindows" {
            let mut config = self.load_config()?;
            config.general.start_with_system = value == "1";
            self.save_config(&config)?;
        }
        Ok(())
    }
    
    // First run checks
    fn is_first_run(&self) -> Result<bool> {
        Ok(false) // Default: not first run
    }
    
    fn clear_first_run_flag(&self) -> Result<()> {
        Ok(())
    }
    
    // Bundled keyboards
    fn get_bundled_keyboards_path(&self) -> Option<PathBuf> {
        None // Default: no bundled keyboards
    }
}

pub fn create_platform() -> Result<Box<dyn Platform>> {
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(windows::WindowsBackend::new()?))
    }
    
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::LinuxBackend::new()?))
    }
    
    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(macos::MacOSBackend::new()?))
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(anyhow::anyhow!("Unsupported platform"))
    }
}