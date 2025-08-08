use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub message: String,
    pub requires_logout: bool,
    pub already_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IMKInfo {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub needs_update: bool,
    pub enabled_in_system: bool,
}

/// Get the path to the IMK bundle in the user's Input Methods directory
fn get_user_imk_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    PathBuf::from(home)
        .join("Library")
        .join("Input Methods")
        .join("KeyMagic3-Server.app")
}

/// Get the path to the embedded IMK bundle in the app's resources
fn get_embedded_imk_path(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_path = app
        .path()
        .resolve("KeyMagic3-Server.app", tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve IMK bundle resource: {}", e))?;
    
    Ok(resource_path)
}

/// Read version from IMK bundle's Info.plist
fn read_bundle_version(bundle_path: &Path) -> Option<String> {
    let plist_path = bundle_path.join("Contents").join("Info.plist");
    
    // Use PlistBuddy to read version
    let output = Command::new("/usr/libexec/PlistBuddy")
        .args(&[
            "-c",
            "Print :CFBundleShortVersionString",
            plist_path.to_str()?,
        ])
        .output()
        .ok()?;
    
    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

/// Check if IMK bundle is installed and get its info
#[tauri::command]
pub fn check_imk_status(app: AppHandle) -> Result<IMKInfo, String> {
    let user_imk_path = get_user_imk_path();
    let embedded_imk_path = get_embedded_imk_path(&app)?;
    
    if !user_imk_path.exists() {
        return Ok(IMKInfo {
            installed: false,
            path: None,
            version: None,
            needs_update: false,
            enabled_in_system: false,
        });
    }
    
    let installed_version = read_bundle_version(&user_imk_path);
    let embedded_version = read_bundle_version(&embedded_imk_path)
        .ok_or_else(|| "Failed to read embedded IMK version".to_string())?;
    
    let needs_update = match &installed_version {
        Some(installed) => compare_versions(installed, &embedded_version) < 0,
        None => true, // If we can't read version, assume update is needed
    };
    
    let enabled_in_system = is_keymagic_enabled();
    
    Ok(IMKInfo {
        installed: true,
        path: Some(user_imk_path.to_string_lossy().to_string()),
        version: installed_version,
        needs_update,
        enabled_in_system,
    })
}

/// Compare semantic versions (returns -1 if v1 < v2, 0 if equal, 1 if v1 > v2)
fn compare_versions(v1: &str, v2: &str) -> i32 {
    let v1_parts: Vec<u32> = v1
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let v2_parts: Vec<u32> = v2
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    
    for i in 0..std::cmp::max(v1_parts.len(), v2_parts.len()) {
        let p1 = v1_parts.get(i).unwrap_or(&0);
        let p2 = v2_parts.get(i).unwrap_or(&0);
        
        if p1 < p2 {
            return -1;
        } else if p1 > p2 {
            return 1;
        }
    }
    
    0
}

/// Check if KeyMagic is enabled in system input sources
fn is_keymagic_enabled() -> bool {
    // Check com.apple.inputsources (where custom input methods are tracked)
    if let Ok(output) = Command::new("defaults")
        .args(&["read", "com.apple.inputsources"])
        .output()
    {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            // Check for KeyMagic3 bundle identifier
            return content.contains("org.keymagic.inputmethod.KeyMagic3");
        }
    }
    
    false
}

/// Open System Settings to Input Sources panel
#[tauri::command]
pub fn open_input_sources_settings() -> Result<(), String> {
    // Try modern System Settings first (macOS 13+)
    let result = Command::new("open")
        .args(&[
            "x-apple.systempreferences:com.apple.Keyboard-Settings.extension?Text%20Input"
        ])
        .output();
    
    if result.is_err() || !result.unwrap().status.success() {
        // Fall back to older System Preferences
        Command::new("open")
            .args(&[
                "/System/Library/PreferencePanes/Keyboard.prefPane",
            ])
            .output()
            .map_err(|e| format!("Failed to open System Preferences: {}", e))?;
    }
    
    Ok(())
}

/// Install or update the IMK bundle
#[tauri::command]
pub async fn install_imk_bundle(app: AppHandle) -> Result<InstallResult, String> {
    let user_imk_path = get_user_imk_path();
    let embedded_imk_path = get_embedded_imk_path(&app)?;
    
    // Create Input Methods directory if it doesn't exist
    let input_methods_dir = user_imk_path.parent().unwrap();
    if !input_methods_dir.exists() {
        fs::create_dir_all(input_methods_dir)
            .map_err(|e| format!("Failed to create Input Methods directory: {}", e))?;
    }
    
    // Check if we need to remove old version
    if user_imk_path.exists() {
        // First, try to quit the running input method
        let _ = Command::new("osascript")
            .args(&[
                "-e",
                "tell application \"KeyMagic3-Server\" to quit"
            ])
            .output();
        
        // Give it a moment to quit
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Remove old bundle
        fs::remove_dir_all(&user_imk_path)
            .map_err(|e| format!("Failed to remove old IMK bundle: {}", e))?;
    }
    
    // Copy new bundle using ditto (preserves all attributes and structure)
    let output = Command::new("ditto")
        .args(&[
            embedded_imk_path.to_str().unwrap(),
            user_imk_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to copy IMK bundle: {}", e))?;
    
    if !output.status.success() {
        return Ok(InstallResult {
            success: false,
            message: format!(
                "Failed to copy IMK bundle: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
            requires_logout: false,
            already_enabled: false,
        });
    }
    
    // Set proper permissions
    let _ = Command::new("chmod")
        .args(&["-R", "755", user_imk_path.to_str().unwrap()])
        .output();
    
    // Check if KeyMagic is already enabled in system
    let already_enabled = is_keymagic_enabled();
    
    let message = if already_enabled {
        "KeyMagic input method updated successfully! The update is complete and KeyMagic is ready to use.\n\nYou can continue using <strong>Control+Space</strong> to switch between input methods.".to_string()
    } else {
        "KeyMagic input method installed successfully! To complete the setup:\n\n1. Click <strong>Open System Settings</strong> below\n2. Click the <strong>+</strong> button in the Input Sources panel\n3. Type <strong>KeyMagic</strong> in the search bar\n4. Select <strong>KeyMagic3</strong> from the search results and click <strong>Add</strong>\n5. Use <strong>Control+Space</strong> to switch between input methods\n   (or click the input method icon in the menu bar)".to_string()
    };
    
    Ok(InstallResult {
        success: true,
        message,
        requires_logout: false,
        already_enabled,
    })
}


/// Uninstall the IMK bundle
#[tauri::command]
pub async fn uninstall_imk_bundle() -> Result<InstallResult, String> {
    let user_imk_path = get_user_imk_path();
    
    if !user_imk_path.exists() {
        return Ok(InstallResult {
            success: true,
            message: "KeyMagic input method is not installed.".to_string(),
            requires_logout: false,
            already_enabled: false,
        });
    }
    
    // First, try to quit the running input method
    let _ = Command::new("osascript")
        .args(&[
            "-e",
            "tell application \"KeyMagic3-Server\" to quit"
        ])
        .output();
    
    // Give it a moment to quit
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Remove the bundle
    fs::remove_dir_all(&user_imk_path)
        .map_err(|e| format!("Failed to remove IMK bundle: {}", e))?;
    
    Ok(InstallResult {
        success: true,
        message: "KeyMagic input method uninstalled successfully. You may need to log out and log back in for changes to take full effect.".to_string(),
        requires_logout: true,
        already_enabled: false,
    })
}