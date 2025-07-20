mod commands;
mod core;
mod hotkey;
mod platform;
mod tray;

use commands::AppState;
use core::KeyboardManager;
use hotkey::HotkeyManager;
use platform::create_platform;
use std::sync::Arc;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Setup logging
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            
            // Initialize platform
            let platform = create_platform()
                .expect("Failed to create platform backend");
            
            // Create keyboard manager
            let keyboard_manager = Arc::new(KeyboardManager::new(platform));
            keyboard_manager.initialize()
                .expect("Failed to initialize keyboard manager");
            
            // Create hotkey manager
            let hotkey_manager = Arc::new(HotkeyManager::new());
            
            // Store in app state
            app.manage(keyboard_manager.clone() as AppState);
            app.manage(hotkey_manager.clone());
            
            // Create system tray
            tray::create_tray_icon(app.handle())?;
            
            // Setup plugins
            app.handle().plugin(tauri_plugin_opener::init())?;
            app.handle().plugin(tauri_plugin_dialog::init())?;
            app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            ))?;
            app.handle().plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                // If another instance tries to start, focus our window
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }))?;
            
            // Set up global shortcut plugin with handler
            {
                use tauri_plugin_global_shortcut::ShortcutState;
                
                let km_for_handler = keyboard_manager.clone();
                let hm_for_handler = hotkey_manager.clone();
                let app_handle = app.handle().clone();
                
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |_app, shortcut, event| {
                            if let ShortcutState::Pressed = event.state() {
                                // Get the keyboard ID for this shortcut
                                if let Some(keyboard_id) = hm_for_handler.get_keyboard_for_shortcut(shortcut) {
                                    log::info!("Hotkey pressed for keyboard: {}", keyboard_id);
                                    
                                    // Switch to the keyboard
                                    if let Err(e) = km_for_handler.set_active_keyboard(&keyboard_id) {
                                        log::error!("Failed to switch to keyboard '{}': {}", keyboard_id, e);
                                        return;
                                    }
                                    
                                    // Update tray menu
                                    if let Err(e) = crate::tray::update_tray_menu(&app_handle) {
                                        log::error!("Failed to update tray menu: {}", e);
                                    }
                                    
                                    // Emit event to notify all UI components
                                    let _ = app_handle.emit("active_keyboard_changed", &keyboard_id);
                                }
                            }
                        })
                        .build()
                )?;
            }
            
            // Initialize hotkeys after all plugins are loaded
            hotkey_manager.initialize(app.handle(), keyboard_manager.clone())
                .expect("Failed to initialize hotkey manager");
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_platform_info,
            commands::get_keyboards,
            commands::get_active_keyboard,
            commands::set_active_keyboard,
            commands::get_keyboard_layout,
            commands::scan_keyboards,
            commands::import_keyboard,
            commands::remove_keyboard,
            commands::update_hotkey,
            commands::get_system_languages,
            commands::check_for_updates,
            commands::restart_app,
            commands::quit_app,
            commands::open_keyboards_folder,
            commands::get_composition_mode_processes,
            commands::set_composition_mode_processes,
            commands::set_start_with_system,
            commands::get_start_with_system,
            commands::update_tray_menu,
            commands::get_app_version,
            commands::check_first_run_scan_keyboards,
            commands::get_bundled_keyboards,
            commands::clear_first_run_scan_keyboards,
            commands::get_setting,
            commands::set_setting,
            commands::run_command,
            commands::add_composition_mode_process,
            commands::remove_composition_mode_process,
            commands::get_supported_languages,
            commands::get_enabled_languages,
            commands::search_languages,
            commands::set_enabled_languages,
            commands::apply_language_changes_elevated,
            commands::check_for_update,
            commands::get_registered_hotkeys,
            commands::refresh_hotkeys,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}