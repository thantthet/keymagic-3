use tauri::{
    AppHandle, Manager, Emitter, 
    menu::{Menu, MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    image::Image,
};
use std::sync::Mutex;
use crate::keyboard_manager::KeyboardManager;
use log::error;

pub fn create_system_tray(app: &AppHandle) -> tauri::Result<()> {
    let keyboard_manager = app.state::<Mutex<KeyboardManager>>();
    let manager = keyboard_manager.lock().unwrap();
    
    let menu = create_tray_menu(app, &manager)?;
    
    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("KeyMagic")
        .menu(&menu)
        .on_menu_event(move |app, event| handle_menu_event(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { 
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;
    
    // Set initial icon based on current state
    update_tray_icon(app, &manager);
    
    Ok(())
}

pub fn create_tray_menu(app: &AppHandle, keyboard_manager: &KeyboardManager) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = MenuBuilder::new(app);
    
    // Get all keyboards
    let keyboards = keyboard_manager.get_keyboards();
    let active_keyboard_id = keyboard_manager.get_active_keyboard();
    
    // Add keyboard items
    let mut menu = menu;
    if !keyboards.is_empty() {
        for keyboard in keyboards {
            let is_active = active_keyboard_id.as_ref().map(|id| id == &keyboard.id).unwrap_or(false);
            let mut item_builder = MenuItemBuilder::new(&keyboard.name)
                .id(format!("keyboard_{}", keyboard.id));
            
            // For active keyboard, we'll add a checkmark in the label instead
            if is_active {
                item_builder = MenuItemBuilder::new(&format!("✓ {}", keyboard.name))
                    .id(format!("keyboard_{}", keyboard.id));
            }
            
            let item = item_builder.build(app)?;
            menu = menu.item(&item);
        }
        
        menu = menu.separator();
    }
    
    menu = menu.separator();
    
    // Add action items
    let open_item = MenuItemBuilder::new("Open")
        .id("open")
        .build(app)?;
    let settings_item = MenuItemBuilder::new("Settings")
        .id("settings")
        .build(app)?;
    let check_update_item = MenuItemBuilder::new("Check for Updates...")
        .id("check_update")
        .build(app)?;
    
    menu = menu.item(&open_item);
    menu = menu.item(&settings_item);
    menu = menu.item(&check_update_item);
    menu = menu.separator();
    
    let quit_item = MenuItemBuilder::new("Quit")
        .id("quit")
        .build(app)?;
    menu = menu.item(&quit_item);
    
    menu.build()
}

pub fn handle_menu_event(app: &AppHandle, menu_id: &str) {
    match menu_id {
        "open" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "settings" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                // Emit event to navigate to settings
                let _ = window.emit("navigate", "settings");
            }
        }
        "check_update" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                // Emit event to navigate to settings and trigger update check
                let _ = window.emit("navigate", "settings");
                let _ = window.emit("check_for_updates", ());
            }
        }
        "quit" => {
            app.exit(0);
        }
        id if id.starts_with("keyboard_") => {
            // Switch keyboard
            let keyboard_id = id.strip_prefix("keyboard_").unwrap();
            let keyboard_manager = app.state::<Mutex<KeyboardManager>>();
            let mut manager = keyboard_manager.lock().unwrap();
            
            if manager.set_active_keyboard(keyboard_id).is_ok() {
                // Get the keyboard name for HUD
                let keyboard_name = manager.get_keyboards()
                    .iter()
                    .find(|k| k.id == keyboard_id)
                    .map(|k| k.name.clone())
                    .unwrap_or_else(|| keyboard_id.to_string());
                
                // Update tray menu and icon
                update_tray_menu(app, &manager);
                update_tray_icon(app, &manager);
                
                // Emit event to update UI
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("active_keyboard_changed", keyboard_id);
                }
                
                // Show native HUD notification
                if let Err(e) = crate::hud::show_keyboard_hud(&keyboard_name) {
                    error!("Failed to show HUD: {}", e);
                }
            }
        }
        _ => {}
    }
}

pub fn update_tray_menu(app: &AppHandle, keyboard_manager: &KeyboardManager) {
    if let Ok(menu) = create_tray_menu(app, keyboard_manager) {
        if let Some(tray) = app.tray_by_id("main") {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

/// Updates the tray icon and tooltip to always show the active keyboard
/// - Shows active keyboard icon with "KeyMagic - {KEYBOARD-NAME}" tooltip
/// - Falls back to default app icon if no keyboard is active
pub fn update_tray_icon(app: &AppHandle, keyboard_manager: &KeyboardManager) {
    if let Some(tray) = app.tray_by_id("main") {
        let (icon, tooltip) = if let Some(keyboard_id) = keyboard_manager.get_active_keyboard() {
            // Try to use active keyboard icon and name
            if let Some(keyboard) = keyboard_manager.get_keyboards()
                .iter()
                .find(|k| k.id == keyboard_id) 
            {
                let icon = if let Some(icon_data) = &keyboard.icon_data {
                    // Try to create icon from keyboard data
                    if let Ok(icon) = create_icon_from_data(icon_data) {
                        icon
                    } else {
                        // Fall back to default icon
                        app.default_window_icon().unwrap().clone()
                    }
                } else if let Some(color) = &keyboard.color {
                    // No icon data but has color - create colored icon
                    if let Ok(icon) = create_icon_from_color(color, &keyboard.name) {
                        icon
                    } else {
                        // Fall back to default icon
                        app.default_window_icon().unwrap().clone()
                    }
                } else {
                    // No icon data or color, use default
                    app.default_window_icon().unwrap().clone()
                };
                
                let tooltip = format!("KeyMagic - {}", keyboard.name);
                (icon, tooltip)
            } else {
                // Keyboard not found, use default
                (app.default_window_icon().unwrap().clone(), "KeyMagic".to_string())
            }
        } else {
            // No active keyboard, use default
            (app.default_window_icon().unwrap().clone(), "KeyMagic".to_string())
        };
        
        let _ = tray.set_icon(Some(icon));
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

/// Creates a Tauri Icon from raw image data (supports PNG, JPG, BMP)
fn create_icon_from_data(data: &[u8]) -> Result<Image<'static>, Box<dyn std::error::Error>> {
    use image::io::Reader as ImageReader;
    use std::io::Cursor;
    
    // First try to detect the image format
    if data.len() < 8 {
        return Err("Image data too small".into());
    }
    
    // Load the image using the image crate
    let img = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?;
    
    // Convert to RGBA8
    let rgba_image = img.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    let rgba_data = rgba_image.into_raw();
    
    // Create Tauri Image
    Ok(Image::new_owned(rgba_data, width, height))
}

/// Creates a solid color icon with the first letter of the keyboard name
fn create_icon_from_color(color: &str, _name: &str) -> Result<Image<'static>, Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Rgba};
    
    const ICON_SIZE: u32 = 32;
    
    // Parse hex color
    let color_hex = color.trim_start_matches('#');
    let r = u8::from_str_radix(&color_hex[0..2], 16)?;
    let g = u8::from_str_radix(&color_hex[2..4], 16)?;
    let b = u8::from_str_radix(&color_hex[4..6], 16)?;
    
    // Create image buffer with transparent background
    let img = ImageBuffer::from_fn(ICON_SIZE, ICON_SIZE, |x, y| {
        // Create a rounded rectangle
        let center = ICON_SIZE as f32 / 2.0;
        let radius = (ICON_SIZE as f32 / 2.0) - 2.0;
        let dx = x as f32 - center + 0.5;
        let dy = y as f32 - center + 0.5;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance <= radius {
            Rgba([r, g, b, 255])
        } else {
            Rgba([0, 0, 0, 0])
        }
    });
    
    // TODO: Add text rendering for the first letter of the name
    // For now, just return the solid color circle
    
    let rgba_data = img.into_raw();
    Ok(Image::new_owned(rgba_data, ICON_SIZE, ICON_SIZE))
}