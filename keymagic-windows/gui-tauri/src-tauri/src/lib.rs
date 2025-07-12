mod commands;
mod keyboard_manager;
mod tray;
mod hotkey;
mod hud;
mod registry_notifier;
mod updater;
mod autostart;
mod app_paths;

use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use keyboard_manager::KeyboardManager;
use hotkey::HotkeyManager;
use tauri::{Manager, Emitter};

// Cleanup handler that disables key processing on drop
struct CleanupHandler {
    app_handle: tauri::AppHandle,
}

impl Drop for CleanupHandler {
    fn drop(&mut self) {
        eprintln!("KeyMagic GUI exiting - disabling key processing");
        if let Some(keyboard_manager) = self.app_handle.try_state::<Mutex<KeyboardManager>>() {
            if let Ok(mut manager) = keyboard_manager.lock() {
                let _ = manager.set_key_processing_enabled(false);
                eprintln!("Key processing disabled on exit");
            }
        }
    }
}

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
            commands::get_app_version,
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
            
            // Sync autostart setting with actual Windows Run registry
            #[cfg(target_os = "windows")]
            {
                if let Err(e) = autostart::sync_autostart_with_preference() {
                    eprintln!("Failed to sync autostart setting: {}", e);
                }
            }
            
            // Setup cleanup handler
            let cleanup_handler = CleanupHandler {
                app_handle: app.app_handle().clone(),
            };
            app.manage(cleanup_handler);
            
            // Setup system tray
            #[cfg(desktop)]
            {
                let _ = tray::create_system_tray(&app.app_handle());
                
                let window = app.get_webview_window("main").unwrap();
                
                let keyboard_manager = app.state::<Mutex<KeyboardManager>>();
                let manager = keyboard_manager.lock().unwrap();
                
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
                
                // Track if this is the first minimize in this session
                let first_minimize = Arc::new(AtomicBool::new(true));
                
                // Always hide window instead of closing when close button is clicked
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            // Hide window instead of closing
                            api.prevent_close();
                            let _ = window_clone.hide();
                            
                            // Show notification on first minimize of this session
                            if first_minimize.load(Ordering::Relaxed) {
                                first_minimize.store(false, Ordering::Relaxed);
                                
                                // Show the notification using HUD
                                if let Err(e) = crate::hud::show_tray_minimize_notification() {
                                    eprintln!("Failed to show minimize notification: {}", e);
                                }
                            }
                        }
                        _ => {}
                    }
                });
                
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