use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::fs;
use std::sync::{OnceLock, Mutex};

// Embed the keyboard icon at compile time
const KEYBOARD_ICON_BYTES: &[u8] = include_bytes!("../resources/keymagic-keyboard.ico");

// Thread-safe one-time initialization using OnceLock
static ICON_PATH: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

/// Get the path to the keyboard icon file, extracting it if necessary
pub fn get_keyboard_icon_path() -> Result<PathBuf> {
    let icon_path_mutex = ICON_PATH.get_or_init(|| {
        let path = extract_keyboard_icon().ok();
        Mutex::new(path)
    });
    
    let icon_path = icon_path_mutex.lock().unwrap();
    icon_path.as_ref()
        .cloned()
        .ok_or_else(|| anyhow!("Failed to extract keyboard icon"))
}

/// Extract the embedded keyboard icon to app data directory
fn extract_keyboard_icon() -> Result<PathBuf> {
    // Get the app data directory for KeyMagic
    let app_data = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("APPDATA"))
        .map_err(|_| anyhow!("Failed to get app data directory"))?;
    
    let keymagic_dir = PathBuf::from(app_data).join("KeyMagic");
    
    // Create the directory if it doesn't exist
    fs::create_dir_all(&keymagic_dir)?;
    
    // Write the icon file
    let icon_path = keymagic_dir.join("keymagic-keyboard.ico");
    
    // Only write if the file doesn't exist or is different
    if !icon_path.exists() || should_update_icon(&icon_path)? {
        fs::write(&icon_path, KEYBOARD_ICON_BYTES)?;
    }
    
    Ok(icon_path)
}

/// Check if the existing icon file needs to be updated
fn should_update_icon(existing_path: &PathBuf) -> Result<bool> {
    // Read existing file
    let existing_bytes = fs::read(existing_path)?;
    
    // Compare with embedded bytes
    Ok(existing_bytes != KEYBOARD_ICON_BYTES)
}