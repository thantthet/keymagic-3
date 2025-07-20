mod commands;
mod core;
mod platform;
mod tray;

use commands::AppState;
use core::KeyboardManager;
use platform::create_platform;
use std::sync::Arc;
use tauri::Manager;

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
            
            // Store in app state
            app.manage(keyboard_manager.clone() as AppState);
            
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}