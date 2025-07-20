use crate::core::KeyboardManager;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

pub fn create_tray_icon<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let tray_menu = build_tray_menu(app)?;
    
    let _ = TrayIconBuilder::with_id("main")
        .tooltip("KeyMagic")
        .icon(app.default_window_icon().unwrap().clone())
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

pub fn update_tray_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("main") {
        let new_menu = build_tray_menu(app)?;
        tray.set_menu(Some(new_menu))?;
        
        // Update tooltip with active keyboard name
        let keyboard_manager = app.state::<Arc<KeyboardManager>>();
        if let Some(active_id) = keyboard_manager.get_active_keyboard() {
            if let Some(keyboard) = keyboard_manager.get_keyboards()
                .iter()
                .find(|k| k.id == active_id) 
            {
                tray.set_tooltip(Some(&format!("KeyMagic - {}", keyboard.name)))?;
            }
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