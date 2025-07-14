use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};

#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::{SHGetKnownFolderPath, FOLDERID_LocalAppData, KF_FLAG_CREATE};
#[cfg(target_os = "windows")]
use windows::Win32::System::Com::CoTaskMemFree;

/// Manages application data directories for KeyMagic
pub struct AppPaths {
    /// Root application data directory (e.g., %LOCALAPPDATA%\KeyMagic)
    #[allow(dead_code)]
    app_data_dir: PathBuf,
    /// Directory for keyboard files (e.g., %LOCALAPPDATA%\KeyMagic\Keyboards)
    keyboards_dir: PathBuf,
}

impl AppPaths {
    /// Creates a new AppPaths instance and ensures directories exist
    pub fn new() -> Result<Self> {
        let app_data_dir = Self::get_app_data_dir()?;
        let keyboards_dir = app_data_dir.join("Keyboards");
        
        // Ensure directories exist
        std::fs::create_dir_all(&app_data_dir)?;
        std::fs::create_dir_all(&keyboards_dir)?;
        
        Ok(Self {
            app_data_dir,
            keyboards_dir,
        })
    }
    
    /// Gets the application data directory path
    #[cfg(target_os = "windows")]
    fn get_app_data_dir() -> Result<PathBuf> {
        unsafe {
            let path = SHGetKnownFolderPath(&FOLDERID_LocalAppData, KF_FLAG_CREATE, None)?;
            let path_str = path.to_string()?;
            CoTaskMemFree(Some(path.0 as _));
            
            Ok(PathBuf::from(path_str).join("KeyMagic"))
        }
    }
    
    /// Gets the application data directory path (non-Windows fallback)
    #[cfg(not(target_os = "windows"))]
    fn get_app_data_dir() -> Result<PathBuf> {
        // For non-Windows, use the home directory approach
        let home = std::env::var("HOME")
            .map_err(|_| anyhow!("HOME environment variable not set"))?;
        Ok(PathBuf::from(home).join(".keymagic"))
    }
    
    
    /// Generates a managed path for a keyboard file based on its ID
    pub fn keyboard_file_path(&self, keyboard_id: &str) -> PathBuf {
        self.keyboards_dir.join(format!("{}.km2", keyboard_id))
    }
    
    /// Copies a keyboard file to the managed location
    /// If a file with the same name exists, prepends a unique identifier
    pub fn install_keyboard(&self, source_path: &Path, keyboard_id: &str) -> Result<(PathBuf, String)> {
        let mut final_keyboard_id = keyboard_id.to_string();
        let mut dest_path = self.keyboard_file_path(&final_keyboard_id);
        
        // Check if file already exists and generate unique name if needed
        if dest_path.exists() {
            // Generate a unique ID using timestamp and random component
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            
            // Simple random number (not cryptographically secure, but sufficient for this use case)
            let random_part = std::process::id() ^ (timestamp as u32);
            
            final_keyboard_id = format!("{}_{:x}_{}", keyboard_id, random_part, timestamp % 1000);
            dest_path = self.keyboard_file_path(&final_keyboard_id);
            
            // In the unlikely event this still exists, add a counter
            let mut counter = 1;
            while dest_path.exists() && counter < 100 {
                final_keyboard_id = format!("{}_{}_{:x}_{}", keyboard_id, counter, random_part, timestamp % 1000);
                dest_path = self.keyboard_file_path(&final_keyboard_id);
                counter += 1;
            }
            
            if dest_path.exists() {
                return Err(anyhow!("Unable to generate unique keyboard filename"));
            }
        }
        
        // Copy the file
        std::fs::copy(source_path, &dest_path)
            .map_err(|e| anyhow!("Failed to copy keyboard file: {}", e))?;
        
        Ok((dest_path, final_keyboard_id))
    }
    
    /// Removes a keyboard file from the managed location
    pub fn uninstall_keyboard(&self, keyboard_id: &str) -> Result<()> {
        let file_path = self.keyboard_file_path(keyboard_id);
        
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| anyhow!("Failed to remove keyboard file: {}", e))?;
        }
        
        Ok(())
    }
    
    
    /// Gets the application installation directory (where the exe is located)
    pub fn get_app_install_dir(&self) -> Result<PathBuf> {
        let exe_path = std::env::current_exe()
            .map_err(|e| anyhow!("Failed to get current exe path: {}", e))?;
        
        exe_path.parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| anyhow!("Failed to get parent directory of exe"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_paths_creation() {
        let app_paths = AppPaths::new();
        assert!(app_paths.is_ok());
        
        let paths = app_paths.unwrap();
        assert!(paths.app_data_dir.exists());
        assert!(paths.keyboards_dir.exists());
    }
    
    #[test]
    fn test_keyboard_file_path() {
        let app_paths = AppPaths::new().unwrap();
        let path = app_paths.keyboard_file_path("myanmar-unicode");
        
        assert!(path.to_string_lossy().contains("myanmar-unicode.km2"));
        assert!(path.to_string_lossy().contains("Keyboards"));
    }
}