use super::{
    CompositionModeConfig, DirectModeConfig, Config, GeneralConfig, KeyboardsConfig,
    Platform, PlatformFeatures, PlatformInfo,
};
use anyhow::{Context, Result};
use plist;
use std::fs;
use std::path::PathBuf;
use keymagic_core::hotkey::HotkeyBinding;

pub struct MacOSBackend {
    config_dir: PathBuf,
    data_dir: PathBuf,
    keyboards_dir: PathBuf,
}

impl MacOSBackend {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::preference_dir()
            .context("Failed to get preferences directory")?
            .join("net.keymagic");
        
        let data_dir = dirs::data_dir()
            .context("Failed to get data directory")?
            .join("KeyMagic");
        
        // User-specific keyboards directory (primary location)
        let keyboards_dir = data_dir.join("Keyboards");
        
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
        self.config_dir.join("config.plist")
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
                enabled_hosts: vec![
                    "com.apple.Spotlight".to_string(),
                    "com.apple.finder".to_string(),
                    "com.apple.TextEdit".to_string(),
                    "com.microsoft.Word".to_string(),
                    "com.apple.Dictionary".to_string(),
                    "ru.keepcoder.Telegram".to_string(),
                    "com.tencent.xinWeChat".to_string(),
                    "com.tinyspeck.slackmacgap".to_string(),
                    "com.apple.Safari".to_string(),
                    "us.zoom.xos".to_string(),
                    "com.apple.dt.Xcode".to_string(),
                    "com.apple.AppStore".to_string(),
                ],
            },
        }
    }
}

impl Platform for MacOSBackend {
    fn load_config(&self) -> Result<Config> {
        let config_path = self.get_config_path();
        
        if !config_path.exists() {
            let config = Self::default_config();
            self.save_config(&config)?;
            return Ok(config);
        }
        
        // Load from plist
        let value = plist::from_file(&config_path)
            .context("Failed to read plist config file")?;
        
        plist::from_value(&value).context("Failed to parse plist config")
    }
    
    fn save_config(&self, config: &Config) -> Result<()> {
        let config_path = self.get_config_path();
        
        // Convert to plist Value
        let value = plist::to_value(config)
            .context("Failed to serialize config to plist value")?;
        
        // Write as binary plist for better performance and smaller size
        plist::to_file_binary(&config_path, &value)
            .context("Failed to write plist config file")?;
        
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
    
    fn notify_ime_update(&self, _keyboard_id: &str) -> Result<()> {
        // TODO: Notify macOS IMK engine
        // This would involve sending a notification to the Input Method Kit
        Ok(())
    }
    
    fn is_ime_running(&self) -> bool {
        // TODO: Check if KeyMagic IMK is running
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
            os: "macos".to_string(),
            features: PlatformFeatures {
                language_profiles: false,
                composition_mode: true,
                global_hotkeys: true,
                system_tray: true,
            },
        }
    }
    
    fn get_bundled_keyboards_path(&self) -> Option<PathBuf> {
        // For macOS, bundled keyboards are inside the app bundle
        if let Ok(exe_path) = std::env::current_exe() {
            // Navigate from executable to Resources directory in app bundle
            // Typical structure: MyApp.app/Contents/MacOS/executable
            // We need: MyApp.app/Contents/Resources/keyboards
            if let Some(macos_dir) = exe_path.parent() {
                if let Some(contents_dir) = macos_dir.parent() {
                    let resources_keyboards_path = contents_dir.join("Resources").join("keyboards");
                    if resources_keyboards_path.exists() {
                        return Some(resources_keyboards_path);
                    }
                }
            }
            
            // Fallback for development: check relative to executable
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
                
                // Add modifiers in macOS order: Cmd, Ctrl, Opt, Shift
                if binding.meta {
                    parts.push("⌘");  // Command symbol
                }
                if binding.ctrl {
                    parts.push("⌃");  // Control symbol
                }
                if binding.alt {
                    parts.push("⌥");  // Option symbol
                }
                if binding.shift {
                    parts.push("⇧");  // Shift symbol
                }
                
                // Add the main key using the display string method
                parts.push(binding.key.to_display_string());
                
                // Use thin space separator for macOS style
                parts.join("\u{2009}")
            }
            Err(_) => {
                // If parsing fails, return the original
                hotkey.to_string()
            }
        }
    }
}