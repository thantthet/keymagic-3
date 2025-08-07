mod commands;
mod core;
mod hotkey;
mod platform;
mod updater;
mod app_enumerator;

#[cfg(target_os = "macos")]
mod imk_installer;


#[cfg(target_os = "windows")]
mod keyboard_icon;

#[cfg(target_os = "windows")]
mod language_profiles;

#[cfg(target_os = "windows")]
mod windows_languages;

#[cfg(target_os = "windows")]
mod windows_event;

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
                        .level(log::LevelFilter::Debug)
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
            
            // Setup plugins
            app.handle().plugin(tauri_plugin_opener::init())?;
            app.handle().plugin(tauri_plugin_dialog::init())?;
            app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            ))?;
            
            // Check and disable autostart if it was previously enabled
            {
                use tauri_plugin_autostart::ManagerExt;
                let autolaunch_manager = app.handle().autolaunch();
                
                // Check if autostart is currently enabled
                match autolaunch_manager.is_enabled() {
                    Ok(true) => {
                        // Autostart is enabled, disable it
                        log::info!("Autostart was previously enabled, disabling it now");
                        if let Err(e) = autolaunch_manager.disable() {
                            log::error!("Failed to disable autostart: {}", e);
                        } else {
                            log::info!("Successfully disabled autostart");
                        }
                    }
                    Ok(false) => {
                        log::info!("Autostart is already disabled");
                    }
                    Err(e) => {
                        log::warn!("Failed to check autostart status: {}", e);
                    }
                }
                
                // Also update the config to reflect this change
                if let Ok(mut config) = keyboard_manager.get_platform().load_config() {
                    if config.general.start_with_system {
                        config.general.start_with_system = false;
                        let _ = keyboard_manager.get_platform().save_config(&config);
                    }
                }
            }
            
            app.handle().plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                // If another instance tries to start, focus our window
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }))?;
            
            
            // Check for updates on startup (async)
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Wait a bit for the app to fully initialize
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                
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
                        log::error!("Failed to check for updates on startup: {}", e);
                    }
                }
            });
            
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
            commands::validate_hotkey,
            commands::check_for_updates,
            commands::restart_app,
            commands::quit_app,
            commands::open_keyboards_folder,
            commands::get_composition_mode_hosts,
            commands::set_composition_mode_hosts,
            commands::get_app_version,
            commands::should_scan_bundled_keyboards,
            commands::get_bundled_keyboards,
            commands::import_bundled_keyboard,
            commands::mark_bundled_keyboards_scanned,
            commands::get_setting,
            commands::set_setting,
            commands::get_update_remind_after,
            commands::set_update_remind_after,
            commands::run_command,
            commands::add_composition_mode_host,
            commands::remove_composition_mode_host,
            commands::get_direct_mode_hosts,
            commands::add_direct_mode_host,
            commands::remove_direct_mode_host,
            commands::get_supported_languages,
            commands::get_enabled_languages,
            commands::search_languages,
            commands::set_enabled_languages,
            commands::apply_language_changes_elevated,
            commands::check_for_update,
            commands::convert_kms_to_km2,
            commands::validate_kms_file,
            commands::convert_kms_file,
            commands::get_running_apps,
            #[cfg(target_os = "macos")]
            imk_installer::check_imk_status,
            #[cfg(target_os = "macos")]
            imk_installer::install_imk_bundle,
            #[cfg(target_os = "macos")]
            imk_installer::uninstall_imk_bundle,
            #[cfg(target_os = "macos")]
            imk_installer::open_input_sources_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Update language profiles when running with elevated privileges
#[cfg(target_os = "windows")]
pub fn update_languages_elevated(languages_str: &str) -> anyhow::Result<()> {
    use anyhow::anyhow;
    
    // Parse comma-separated language codes
    let languages: Vec<String> = languages_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    if languages.is_empty() {
        return Err(anyhow!("No languages provided"));
    }
    
    // Update the language profiles
    crate::language_profiles::update_language_profiles(&languages)
        .map_err(|e| anyhow!("Failed to update language profiles: {}", e))
}

#[cfg(not(target_os = "windows"))]
pub fn update_languages_elevated(_languages_str: &str) -> anyhow::Result<()> {
    Ok(())
}

