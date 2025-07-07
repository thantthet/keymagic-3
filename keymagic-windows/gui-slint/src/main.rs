// Hide console window on Windows
#![windows_subsystem = "windows"]

use anyhow::Result;
use log::{info, error};

slint::include_modules!();

mod app;
mod keyboard_manager;
mod models;
mod file_dialog;
mod crash_handler;
mod safe_slint;

use app::App;

fn main() {
    // Initialize crash handler and logging first
    if let Err(e) = crash_handler::init() {
        eprintln!("Failed to initialize crash handler: {}", e);
    }
    
    info!("KeyMagic GUI starting up...");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Platform: Windows ARM64");
    
    // Run the actual application
    if let Err(e) = run_app() {
        error!("Application failed: {}", e);
        
        // Show error dialog to user
        #[cfg(windows)]
        show_error_dialog(&format!("KeyMagic GUI encountered an error:\n\n{}", e));
        
        std::process::exit(1);
    }
    
    info!("KeyMagic GUI shutting down normally");
}

fn run_app() -> Result<()> {
    use crate::safe_slint::*;
    
    info!("Initializing application...");
    
    // Validate Slint backend before starting
    validate_slint_backend()
        .map_err(|e| {
            error!("Backend validation failed: {}", e);
            e
        })?;
    
    // Initialize the application
    let app = App::new()
        .map_err(|e| {
            error!("Failed to create App: {}", e);
            e
        })?;
    
    info!("Creating main window...");
    
    // Create the main window
    let main_window = MainWindow::new()
        .map_err(|e| {
            error!("Failed to create MainWindow: {}", e);
            e
        })?;
    
    info!("Setting up UI...");
    
    // Set up initial data
    app.setup_ui(&main_window)
        .map_err(|e| {
            error!("Failed to setup UI: {}", e);
            e
        })?;
    
    info!("Connecting callbacks...");
    
    // Connect callbacks
    app.connect_callbacks(&main_window);
    
    info!("Showing window...");
    
    // Show the window with safety checks
    safe_show_window(&main_window, "main")
        .map_err(|e| {
            error!("Failed to show window: {}", e);
            e
        })?;
    
    info!("Starting event loop...");
    
    // Monitor window health periodically
    let window_clone = main_window.as_weak();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));
            if let Some(window) = window_clone.upgrade() {
                if !monitor_rendering_health(&window) {
                    error!("Window health check failed!");
                }
            } else {
                break;
            }
        }
    });
    
    // Run the event loop with monitoring
    safe_event_loop_operation(
        || main_window.run().map_err(|e| anyhow::anyhow!("Event loop error: {}", e)),
        "main_event_loop"
    )?;
    
    Ok(())
}

#[cfg(windows)]
fn show_error_dialog(message: &str) {
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONERROR};
    use windows::core::PCWSTR;
    
    unsafe {
        let title: Vec<u16> = "KeyMagic Error\0".encode_utf16().collect();
        let msg: Vec<u16> = format!("{}\0", message).encode_utf16().collect();
        
        MessageBoxW(
            None,
            PCWSTR(msg.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}