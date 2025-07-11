use std::env;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::*;

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
    unsafe {
        let mut hkey = HKEY::default();
        
        // Open the Run key
        let run_key_path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(run_key_path.as_ptr()),
            0,
            KEY_READ,
            &mut hkey
        ).is_err() {
            return false;
        }
        
        // Check if our app entry exists
        let mut buffer = vec![0u16; 512];
        let mut size = buffer.len() as u32 * 2;
        let mut data_type = REG_VALUE_TYPE::default();
        
        let app_name: Vec<u16> = "KeyMagic"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        
        let result = RegQueryValueExW(
            hkey,
            PCWSTR(app_name.as_ptr()),
            None,
            Some(&mut data_type),
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut size),
        );
        
        let _ = RegCloseKey(hkey);
        
        result.is_ok()
    }
    
    #[cfg(not(target_os = "windows"))]
    false
}

/// Enable or disable autostart
pub fn set_autostart(enabled: bool) -> std::result::Result<(), String> {
    #[cfg(target_os = "windows")]
    unsafe {
        let mut hkey = HKEY::default();
        
        // Open the Run key
        let run_key_path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
            
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(run_key_path.as_ptr()),
            0,
            KEY_WRITE,
            &mut hkey
        ).is_err() {
            return Err("Failed to open Windows Run registry key".to_string());
        }
        
        let app_name: Vec<u16> = "KeyMagic"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
            
        let result = if enabled {
            // Get the executable path
            let exe_path = get_executable_path()?;
            
            // Convert to wide string
            let value_w: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();
            let value_bytes = std::slice::from_raw_parts(
                value_w.as_ptr() as *const u8,
                value_w.len() * 2
            );
            
            // Add the autostart entry
            RegSetValueExW(
                hkey,
                PCWSTR(app_name.as_ptr()),
                0,
                REG_SZ,
                Some(value_bytes),
            )
        } else {
            // Remove the autostart entry
            RegDeleteValueW(hkey, PCWSTR(app_name.as_ptr()))
        };
        
        let _ = RegCloseKey(hkey);
        
        if result.is_err() {
            return Err(format!("Failed to {} autostart registry entry", 
                if enabled { "create" } else { "remove" }));
        }
        
        Ok(())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

/// Sync the autostart state with the saved preference
/// Returns true if autostart is enabled, false otherwise
pub fn sync_autostart_with_preference() -> std::result::Result<bool, String> {
    // Get the saved preference
    let saved_preference = crate::commands::get_setting("StartWithWindows".to_string())?;
    let should_be_enabled = saved_preference.as_deref() == Some("1");
    
    // Get the actual state
    let is_enabled = is_autostart_enabled();
    
    // Sync if they don't match
    if should_be_enabled != is_enabled {
        set_autostart(should_be_enabled)?;
    }
    
    Ok(should_be_enabled)
}