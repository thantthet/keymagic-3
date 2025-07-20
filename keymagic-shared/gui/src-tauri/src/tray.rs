use crate::core::KeyboardManager;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

pub fn create_tray_icon<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let tray_menu = build_tray_menu(app)?;
    
    // Get the initial icon and tooltip based on active keyboard
    let keyboard_manager = app.state::<Arc<KeyboardManager>>();
    let (icon, tooltip) = if let Some(active_id) = keyboard_manager.get_active_keyboard() {
        if let Some(keyboard) = keyboard_manager.get_keyboards()
            .iter()
            .find(|k| k.id == active_id) 
        {
            (
                get_keyboard_icon(app, keyboard.icon_data.as_ref()),
                format!("KeyMagic - {}", keyboard.name)
            )
        } else {
            let default_icon = app.default_window_icon().unwrap();
            let (width, height) = (default_icon.width(), default_icon.height());
            let rgba_data = default_icon.rgba().to_vec();
            (
                Image::new_owned(rgba_data, width, height),
                "KeyMagic".to_string()
            )
        }
    } else {
        let default_icon = app.default_window_icon().unwrap();
        let (width, height) = (default_icon.width(), default_icon.height());
        let rgba_data = default_icon.rgba().to_vec();
        (
            Image::new_owned(rgba_data, width, height),
            "KeyMagic".to_string()
        )
    };
    
    let _ = TrayIconBuilder::with_id("main")
        .tooltip(&tooltip)
        .icon(icon)
        .menu(&tray_menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| handle_tray_event(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;
    
    Ok(())
}

pub fn build_tray_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let keyboard_manager = app.state::<Arc<KeyboardManager>>();
    let keyboards = keyboard_manager.get_keyboards();
    let active_keyboard = keyboard_manager.get_active_keyboard();
    
    let tray_menu = Menu::new(app)?;
    
    // Add keyboard items
    if !keyboards.is_empty() {
        for keyboard in keyboards {
            let menu_item = if Some(&keyboard.id) == active_keyboard.as_ref() {
                MenuItemBuilder::with_id(&keyboard.id, format!("✓ {}", keyboard.name))
                    .build(app)?
            } else {
                MenuItemBuilder::with_id(&keyboard.id, &keyboard.name)
                    .build(app)?
            };
            tray_menu.append(&menu_item)?;
        }
        
        // Add separator after keyboards
        tray_menu.append(&PredefinedMenuItem::separator(app)?)?;
    }
    
    // Add static menu items
    let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    
    tray_menu.append(&show_item)?;
    tray_menu.append(&settings_item)?;
    tray_menu.append(&PredefinedMenuItem::separator(app)?)?;
    tray_menu.append(&quit_item)?;
    
    Ok(tray_menu)
}

fn get_keyboard_icon<R: Runtime>(app: &AppHandle<R>, icon_data: Option<&Vec<u8>>) -> Image<'static> {
    // Try to use the keyboard's icon data
    if let Some(data) = icon_data {
        // Try to create icon from keyboard data
        if let Ok(icon) = create_icon_from_data(data) {
            return icon;
        }
    }
    
    // Fall back to the default app icon
    // We need to extract the raw data and recreate the image to satisfy the lifetime requirements
    let default_icon = app.default_window_icon().unwrap();
    let (width, height) = (default_icon.width(), default_icon.height());
    let rgba_data = default_icon.rgba().to_vec();
    Image::new_owned(rgba_data, width, height)
}

/// Creates a Tauri Icon from raw image data (supports PNG, JPG, BMP)
fn create_icon_from_data(data: &[u8]) -> Result<Image<'static>, Box<dyn std::error::Error>> {
    use image::ImageReader;
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

pub fn update_tray_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("main") {
        let new_menu = build_tray_menu(app)?;
        tray.set_menu(Some(new_menu))?;
        
        // Update tooltip and icon with active keyboard info
        let keyboard_manager = app.state::<Arc<KeyboardManager>>();
        if let Some(active_id) = keyboard_manager.get_active_keyboard() {
            if let Some(keyboard) = keyboard_manager.get_keyboards()
                .iter()
                .find(|k| k.id == active_id) 
            {
                // Update tooltip
                tray.set_tooltip(Some(&format!("KeyMagic - {}", keyboard.name)))?;
                
                // Update icon to show the active keyboard's icon
                let icon = get_keyboard_icon(app, keyboard.icon_data.as_ref());
                tray.set_icon(Some(icon))?;
            }
        } else {
            // No active keyboard, use default icon and tooltip
            tray.set_tooltip(Some("KeyMagic"))?;
            let default_icon = app.default_window_icon().unwrap();
            let (width, height) = (default_icon.width(), default_icon.height());
            let rgba_data = default_icon.rgba().to_vec();
            let icon = Image::new_owned(rgba_data, width, height);
            tray.set_icon(Some(icon))?;
        }
    }
    
    Ok(())
}

fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, menu_id: &str) {
    match menu_id {
        "quit" => {
            app.exit(0);
        }
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "settings" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.eval("window.location.hash = '#settings'");
            }
        }
        keyboard_id => {
            // Try to switch keyboard
            let keyboard_manager = app.state::<Arc<KeyboardManager>>();
            if let Ok(_) = keyboard_manager.set_active_keyboard(keyboard_id) {
                // Update tray menu to reflect the change
                let _ = update_tray_menu(app);
                
                // Notify frontend about the change
                let _ = app.emit("active_keyboard_changed", keyboard_id);
            }
        }
    }
}