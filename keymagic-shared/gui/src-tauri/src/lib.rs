mod commands;
mod core;
mod hotkey;
mod notification;
mod platform;
mod tray;
mod updater;

#[cfg(target_os = "windows")]
mod hud_win32;

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
use notification::NotificationManager;
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
            
            // Initialize Windows HUD if on Windows
            #[cfg(target_os = "windows")]
            {
                if let Err(e) = crate::hud_win32::initialize_hud() {
                    log::warn!("Failed to initialize Windows HUD: {}", e);
                }
            }
            
            // Create notification manager
            let notification_manager = Arc::new(NotificationManager::new(app.handle().clone()));
            
            // Store in app state
            app.manage(keyboard_manager.clone() as AppState);
            app.manage(hotkey_manager.clone());
            app.manage(notification_manager.clone());
            app.manage(FirstMinimizeState::new());
            
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
                let nm_for_handler = notification_manager.clone();
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
                                    
                                    // Get keyboard info for the notification
                                    if let Some(keyboard) = km_for_handler.get_keyboard(&keyboard_id) {
                                        // Show HUD notification
                                        let nm = nm_for_handler.clone();
                                        let kb_name = keyboard.name.clone();
                                        let icon_data = keyboard.icon_data.clone();
                                        std::thread::spawn(move || {
                                            if let Err(e) = futures::executor::block_on(
                                                nm.show_keyboard_switch(&kb_name, icon_data)
                                            ) {
                                                log::error!("Failed to show HUD notification: {}", e);
                                            }
                                        });
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
            commands::check_for_updates,
            commands::restart_app,
            commands::quit_app,
            commands::open_keyboards_folder,
            commands::get_composition_mode_hosts,
            commands::set_composition_mode_hosts,
            commands::set_start_with_system,
            commands::get_start_with_system,
            commands::update_tray_menu,
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
            commands::get_registered_hotkeys,
            commands::refresh_hotkeys,
        ])
        .on_window_event(|window, event| {
            use tauri::WindowEvent;
            
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    // Hide window instead of closing
                    api.prevent_close();
                    let _ = window.hide();
                    
                    // Check if this is the first minimize
                    let app_handle = window.app_handle();
                    if app_handle.state::<FirstMinimizeState>().is_first() {
                        // Show notification
                        if let Some(nm) = app_handle.try_state::<Arc<NotificationManager>>() {
                            let nm = nm.inner().clone();
                            std::thread::spawn(move || {
                                if let Err(e) = futures::executor::block_on(
                                    nm.show_tray_notification()
                                ) {
                                    log::error!("Failed to show tray notification: {}", e);
                                }
                            });
                        }
                    }
                }
                _ => {}
            }
        })
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

// State to track first minimize
struct FirstMinimizeState(std::sync::Mutex<bool>);

impl FirstMinimizeState {
    fn new() -> Self {
        Self(std::sync::Mutex::new(true))
    }
    
    fn is_first(&self) -> bool {
        let mut first = self.0.lock().unwrap();
        let was_first = *first;
        *first = false;
        was_first
    }
}