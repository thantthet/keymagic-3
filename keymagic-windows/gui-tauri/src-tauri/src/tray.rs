use tauri::{
    AppHandle, Manager, Emitter, 
    menu::{Menu, MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
};
use std::sync::Mutex;
use crate::keyboard_manager::KeyboardManager;

pub fn create_system_tray(app: &AppHandle) -> tauri::Result<()> {
    let keyboard_manager = app.state::<Mutex<KeyboardManager>>();
    let manager = keyboard_manager.lock().unwrap();
    
    let menu = create_tray_menu(app, &manager)?;
    
    let _ = TrayIconBuilder::with_id("main")
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
    
    // Add Enable/Disable toggle
    let is_enabled = keyboard_manager.is_key_processing_enabled();
    let toggle_text = if is_enabled { "Disable" } else { "Enable" };
    let toggle_item = MenuItemBuilder::new(toggle_text)
        .id("toggle_enable")
        .build(app)?;
    menu = menu.item(&toggle_item);
    
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
        "toggle_enable" => {
            let keyboard_manager = app.state::<Mutex<KeyboardManager>>();
            let mut manager = keyboard_manager.lock().unwrap();
            let current_state = manager.is_key_processing_enabled();
            let _ = manager.set_key_processing_enabled(!current_state);
            
            // Update tray menu
            update_tray_menu(app, &manager);
            
            // Emit event to update UI
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("key_processing_changed", !current_state);
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
                
                // Update tray menu
                update_tray_menu(app, &manager);
                
                // Emit event to update UI
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("active_keyboard_changed", keyboard_id);
                }
                
                // Show native HUD notification
                if let Err(e) = crate::hud::show_keyboard_hud(&keyboard_name) {
                    eprintln!("Failed to show HUD: {}", e);
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