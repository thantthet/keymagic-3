mod commands;
mod keyboard_manager;
mod tray;
mod hotkey;
mod hud;
mod registry_notifier;
mod updater;

use std::sync::Mutex;
use keyboard_manager::KeyboardManager;
use hotkey::HotkeyManager;
use tauri::{Manager, Emitter};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize keyboard manager
    let keyboard_manager = KeyboardManager::new()
        .expect("Failed to initialize keyboard manager");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(Mutex::new(keyboard_manager))
        .manage(HotkeyManager::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_keyboards,
            commands::get_active_keyboard,
            commands::set_active_keyboard,
            commands::add_keyboard,
            commands::remove_keyboard,
            commands::is_key_processing_enabled,
            commands::set_key_processing_enabled,
            commands::get_setting,
            commands::set_setting,
            commands::set_keyboard_hotkey,
            commands::update_tray_menu,
            commands::set_on_off_hotkey,
            commands::get_on_off_hotkey,
            commands::check_for_update,
        ])
        .setup(|app| {
            // Initialize native HUD window
            #[cfg(target_os = "windows")]
            {
                if let Err(e) = hud::initialize_hud() {
                    eprintln!("Failed to initialize HUD: {}", e);
                }
            }
            
            // Setup system tray
            #[cfg(desktop)]
            {
                let _ = tray::create_system_tray(&app.app_handle());
                
                let window = app.get_webview_window("main").unwrap();
                
                // Check if should minimize to tray
                let keyboard_manager = app.state::<Mutex<KeyboardManager>>();
                let manager = keyboard_manager.lock().unwrap();
                let minimize_to_tray = manager.get_setting("minimize_to_tray")
                    .unwrap_or_else(|_| "true".to_string()) == "true";
                
                // Register all keyboard hotkeys
                let hotkey_manager = app.state::<HotkeyManager>();
                if let Err(e) = hotkey_manager.register_all_hotkeys(&app.app_handle(), &manager) {
                    eprintln!("Failed to register hotkeys: {}", e);
                }
                
                // Load and register on/off hotkey
                if let Err(e) = hotkey_manager.load_on_off_hotkey(&app.app_handle()) {
                    eprintln!("Failed to load on/off hotkey: {}", e);
                }
                
                drop(manager);
                
                if minimize_to_tray {
                    // Hide window instead of closing when close button is clicked
                    let window_clone = window.clone();
                    window.on_window_event(move |event| {
                        match event {
                            tauri::WindowEvent::CloseRequested { api, .. } => {
                                // Hide window instead of closing
                                api.prevent_close();
                                let _ = window_clone.hide();
                            }
                            _ => {}
                        }
                    });
                }
                
                // Check for updates on startup (async)
                let app_handle = app.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    // Wait a bit for the app to fully initialize
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    
                    // Check for updates silently
                    match crate::updater::check_for_updates_async().await {
                        Ok(update_info) => {
                            if update_info.update_available {
                                // Emit event to notify UI about available update
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    let _ = window.emit("update_available", update_info);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to check for updates on startup: {}", e);
                        }
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}