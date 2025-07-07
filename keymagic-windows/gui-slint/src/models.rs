use crate::KeyboardInfo as SlintKeyboardInfo;
use crate::keyboard_manager::{KeyboardInfo, KeyboardManager};
use slint::{SharedString, Image, SharedPixelBuffer, Rgba8Pixel};

/// Convert image data (BMP, PNG, JPG, ICO) to Slint Image
fn image_data_to_slint_image(image_data: &[u8]) -> Option<Image> {
    // Use the image crate to decode image data
    // It will automatically detect the format (BMP, PNG, JPG, etc.)
    use image::io::Reader as ImageReader;
    use std::io::Cursor;
    
    match ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()
    {
        Some(img) => {
            let rgba_image = img.to_rgba8();
            let width = rgba_image.width();
            let height = rgba_image.height();
            
            // Create SharedPixelBuffer from image data
            let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                rgba_image.as_raw(),
                width,
                height,
            );
            
            Some(Image::from_rgba8(buffer))
        }
        None => None,
    }
}

/// Parse a hotkey string into its components
fn parse_hotkey(hotkey: &str) -> (bool, bool, bool, String) {
    let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();
    
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut key = String::new();
    
    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" => ctrl = true,
            "alt" => alt = true,
            "shift" => shift = true,
            _ => {
                if !part.is_empty() {
                    key = part.to_string();
                }
            }
        }
    }
    
    (ctrl, alt, shift, key)
}

pub fn convert_keyboard_info(info: &KeyboardInfo, active_id: Option<&str>, manager: &KeyboardManager) -> SlintKeyboardInfo {
    // Convert icon data to Slint Image if available (supports BMP, PNG, JPG, ICO)
    let icon = if let Some(icon_data) = &info.icon_data {
        image_data_to_slint_image(icon_data).unwrap_or_default()
    } else {
        Image::default()
    };
    
    // Get effective hotkey (custom or default)
    let effective_hotkey = manager.get_effective_hotkey(info).unwrap_or_default();
    
    // Parse the hotkey
    let (hotkey_ctrl, hotkey_alt, hotkey_shift, hotkey_key) = parse_hotkey(&effective_hotkey);
    
    SlintKeyboardInfo {
        id: SharedString::from(&info.id),
        name: SharedString::from(&info.name),
        description: SharedString::from(&info.description),
        hotkey: SharedString::from(&effective_hotkey),
        default_hotkey: SharedString::from(info.default_hotkey.as_deref().unwrap_or("")),
        custom_hotkey: SharedString::from(info.hotkey.as_deref().unwrap_or("")),
        active: active_id.map(|id| id == &info.id).unwrap_or(false),
        enabled: info.enabled,
        has_icon: info.icon_data.is_some(),
        icon,
        hotkey_ctrl,
        hotkey_alt,
        hotkey_shift,
        hotkey_key: SharedString::from(&hotkey_key),
    }
}