mod commands;
mod keyboard_manager;
mod tray;
mod hotkey;
mod hud;
mod registry_notifier;

use std::sync::Mutex;
use keyboard_manager::KeyboardManager;
use hotkey::HotkeyManager;
use tauri::Manager;

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
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}