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

pub fn convert_keyboard_info(info: &KeyboardInfo, active_id: Option<&str>, manager: &KeyboardManager) -> SlintKeyboardInfo {
    // Convert icon data to Slint Image if available (supports BMP, PNG, JPG, ICO)
    let icon = if let Some(icon_data) = &info.icon_data {
        image_data_to_slint_image(icon_data).unwrap_or_default()
    } else {
        Image::default()
    };
    
    // Get effective hotkey (custom or default)
    let effective_hotkey = manager.get_effective_hotkey(info).unwrap_or_default();
    
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
    }
}