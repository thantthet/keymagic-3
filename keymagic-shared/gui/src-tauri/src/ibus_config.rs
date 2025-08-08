use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct IBusConfig {
    pub symbol: String,
    pub config_exists: bool,
    pub config_path: String,
}

const IBUS_CONFIG_PATH: &str = "/usr/share/ibus/component/keymagic3.xml";

/// Get IBus configuration information
#[tauri::command]
pub fn get_ibus_config() -> Result<IBusConfig, String> {
    let config_path = Path::new(IBUS_CONFIG_PATH);
    
    if !config_path.exists() {
        return Ok(IBusConfig {
            symbol: "Not installed".to_string(),
            config_exists: false,
            config_path: IBUS_CONFIG_PATH.to_string(),
        });
    }
    
    // Read the XML file
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read IBus config: {}", e))?;
    
    // Parse symbol from XML (simple string search for now)
    let symbol = extract_symbol(&content).unwrap_or_else(|| "Unknown".to_string());
    
    Ok(IBusConfig {
        symbol,
        config_exists: true,
        config_path: IBUS_CONFIG_PATH.to_string(),
    })
}

/// Extract symbol value from XML content
fn extract_symbol(xml_content: &str) -> Option<String> {
    // Look for <symbol>...</symbol> tag
    let start_tag = "<symbol>";
    let end_tag = "</symbol>";
    
    if let Some(start_pos) = xml_content.find(start_tag) {
        let start = start_pos + start_tag.len();
        if let Some(end_pos) = xml_content[start..].find(end_tag) {
            let symbol = xml_content[start..start + end_pos].trim().to_string();
            return Some(symbol);
        }
    }
    
    None
}

/// Check if IBus is installed on the system
#[tauri::command]
pub fn check_ibus_installed() -> bool {
    // Check if ibus command exists
    std::process::Command::new("which")
        .arg("ibus")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}