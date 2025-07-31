use super::{
    CompositionModeConfig, DirectModeConfig, Config, GeneralConfig, InstalledKeyboard, KeyboardsConfig,
    Platform, PlatformFeatures, PlatformInfo,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use keymagic_core::hotkey::HotkeyBinding;

pub struct LinuxBackend {
    config_dir: PathBuf,
    data_dir: PathBuf,
    keyboards_dir: PathBuf,
}

impl LinuxBackend {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("keymagic3");
        
        let data_dir = dirs::data_dir()
            .context("Failed to get data directory")?
            .join("keymagic3");
        
        // User keyboards directory within data directory
        let keyboards_dir = data_dir.join("keyboards");
        
        // Create directories if they don't exist
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(&keyboards_dir)?;
        
        Ok(Self {
            config_dir,
            data_dir,
            keyboards_dir,
        })
    }
    
    fn get_config_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
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
                enabled_hosts: vec![],
            },
            direct_mode: DirectModeConfig {
                enabled_hosts: vec![],
            },
        }
    }
}

impl Platform for LinuxBackend {
    fn load_config(&self) -> Result<Config> {
        let config_path = self.get_config_path();
        
        if !config_path.exists() {
            let config = Self::default_config();
            self.save_config(&config)?;
            return Ok(config);
        }
        
        let contents = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        toml::from_str(&contents).context("Failed to parse config file")
    }
    
    fn save_config(&self, config: &Config) -> Result<()> {
        let config_path = self.get_config_path();
        let contents = toml::to_string_pretty(config)
            .context("Failed to serialize config")?;
        
        fs::write(&config_path, contents)
            .context("Failed to write config file")?;
        
        Ok(())
    }
    
    fn get_keyboards_dir(&self) -> PathBuf {
        self.keyboards_dir.clone()
    }
    
    fn get_keyboard_files(&self) -> Result<Vec<PathBuf>> {
        let mut keyboards = Vec::new();
        
        // Check user keyboards directory
        if self.keyboards_dir.exists() {
            for entry in fs::read_dir(&self.keyboards_dir)? {
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
        // Send D-Bus notification to IBus engine
        // TODO: Implement D-Bus notification when needed
        log::info!("Notifying IME update for keyboard: {}", keyboard_id);
        
        Ok(())
    }
    
    fn is_ime_running(&self) -> bool {
        // Check if IBus KeyMagic engine is running
        // TODO: Implement proper IBus engine detection
        false
    }
    
    fn switch_keyboard(&self, keyboard_id: &str) -> Result<()> {
        self.notify_ime_update(keyboard_id)
    }
    
    fn get_config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }
    
    fn get_data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }
    
    
    fn get_platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            os: "linux".to_string(),
            features: PlatformFeatures {
                language_profiles: false,
                composition_mode: false,
                global_hotkeys: true,
                system_tray: true,
            },
        }
    }
    
    fn get_bundled_keyboards_path(&self) -> Option<PathBuf> {
        // Check system-wide bundled keyboards location
        let system_keyboards_path = PathBuf::from("/usr/share/keymagic3/keyboards");
        if system_keyboards_path.exists() {
            return Some(system_keyboards_path);
        }
        
        // Fallback: Check relative to executable (for development/testing)
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
    
    fn normalize_hotkey_for_display(&self, hotkey: &str) -> String {
        if hotkey.is_empty() {
            return String::new();
        }
        
        // Try to parse the hotkey
        match HotkeyBinding::parse(hotkey) {
            Ok(binding) => {
                let mut parts = Vec::new();
                
                // Add modifiers in Linux order: Super, Ctrl, Alt, Shift
                if binding.meta {
                    parts.push("Super");  // Super/Windows key
                }
                if binding.ctrl {
                    parts.push("Ctrl");
                }
                if binding.alt {
                    parts.push("Alt");
                }
                if binding.shift {
                    parts.push("Shift");
                }
                
                // Add the main key using the display string method
                parts.push(binding.key.to_display_string());
                
                parts.join("+")
            }
            Err(_) => {
                // If parsing fails, return the original
                hotkey.to_string()
            }
        }
    }
}