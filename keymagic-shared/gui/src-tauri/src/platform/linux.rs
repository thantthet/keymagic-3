use super::{
    CompositionModeConfig, Config, GeneralConfig, InstalledKeyboard, KeyboardsConfig, Language,
    Platform, PlatformFeatures, PlatformInfo,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct LinuxBackend {
    config_dir: PathBuf,
    data_dir: PathBuf,
    keyboards_dir: PathBuf,
}

impl LinuxBackend {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("keymagic");
        
        let data_dir = dirs::data_dir()
            .context("Failed to get data directory")?
            .join("keymagic");
        
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
            },
            keyboards: KeyboardsConfig {
                active: None,
                last_used: Vec::new(),
                installed: Vec::new(),
            },
            composition_mode: CompositionModeConfig {
                enabled_processes: vec![
                    "firefox".to_string(),
                    "chromium".to_string(),
                    "chrome".to_string(),
                    "code".to_string(),
                    "gedit".to_string(),
                ],
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
    
    fn supports_language_profiles(&self) -> bool {
        false // Linux doesn't have language profiles like Windows
    }
    
    fn supports_composition_mode(&self) -> bool {
        true // Linux can support composition mode
    }
    
    fn get_platform_name(&self) -> &'static str {
        "linux"
    }
    
    fn get_platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            os: "linux".to_string(),
            features: PlatformFeatures {
                language_profiles: false,
                composition_mode: true,
                global_hotkeys: true,
                system_tray: true,
            },
        }
    }
    
    fn get_system_languages(&self) -> Result<Vec<Language>> {
        // Get system locales
        let mut languages = Vec::new();
        
        // Parse locale -a output
        if let Ok(output) = std::process::Command::new("locale")
            .arg("-a")
            .output()
        {
            if let Ok(locales) = String::from_utf8(output.stdout) {
                for locale in locales.lines() {
                    if let Some(lang_code) = locale.split('.').next() {
                        languages.push(Language {
                            id: lang_code.to_string(),
                            name: lang_code.to_string(), // TODO: Get proper display name
                            code: lang_code.to_string(),
                        });
                    }
                }
            }
        }
        
        // Always include Myanmar
        if !languages.iter().any(|l| l.code == "my" || l.code == "my_MM") {
            languages.push(Language {
                id: "my".to_string(),
                name: "Myanmar".to_string(),
                code: "my".to_string(),
            });
        }
        
        Ok(languages)
    }
}