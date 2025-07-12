use std::env;

#[cfg(target_os = "windows")]
use crate::registry;

/// Get the current executable path
pub fn get_executable_path() -> std::result::Result<String, String> {
    env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))
        .and_then(|path| {
            path.to_str()
                .ok_or_else(|| "Failed to convert path to string".to_string())
                .map(|s| s.to_string())
        })
}

/// Check if autostart is currently enabled
pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        registry::is_autostart_enabled().unwrap_or(false)
    }
    
    #[cfg(not(target_os = "windows"))]
    false
}

/// Enable or disable autostart
pub fn set_autostart(enabled: bool) -> std::result::Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let exe_path = if enabled {
            Some(env::current_exe()
                .map_err(|e| format!("Failed to get executable path: {}", e))?)
        } else {
            None
        };
        
        registry::set_autostart_enabled(enabled, exe_path.as_ref())
            .map_err(|e| format!("Failed to set autostart: {}", e))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

/// Sync the autostart state with the saved preference
/// Returns true if autostart is enabled, false otherwise
pub fn sync_autostart_with_preference() -> std::result::Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        // Get the saved preference
        let should_be_enabled = registry::get_setting("StartWithWindows")
            .map_err(|e| format!("Failed to get setting: {}", e))?
            .as_deref() == Some("1");
        
        // Get the actual state
        let is_enabled = registry::is_autostart_enabled()
            .map_err(|e| format!("Failed to check autostart: {}", e))?;
        
        // Sync if they don't match
        if should_be_enabled != is_enabled {
            set_autostart(should_be_enabled)?;
        }
        
        Ok(should_be_enabled)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok(false)
    }
}