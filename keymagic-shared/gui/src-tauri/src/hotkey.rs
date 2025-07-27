use anyhow::Result;
use keymagic_core::hotkey::HotkeyBinding;

pub struct HotkeyManager;

impl HotkeyManager {
    pub fn new() -> Self {
        Self
    }

    /// Validate a hotkey string without registering it
    pub fn validate_hotkey(&self, hotkey_str: &str) -> Result<()> {
        // Empty hotkey is valid (removes hotkey)
        if hotkey_str.is_empty() {
            return Ok(());
        }
        
        // Try to parse the hotkey using keymagic-core's parser
        let hotkey = HotkeyBinding::parse(hotkey_str)
            .map_err(|e| anyhow::anyhow!("Invalid hotkey: {}", e))?;
        
        // On Windows, disallow Win/Meta modifier
        #[cfg(target_os = "windows")]
        if hotkey.meta {
            return Err(anyhow::anyhow!("The hotkey cannot contain the Win key"));
        }
        
        Ok(())
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}