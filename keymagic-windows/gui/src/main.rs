use windows::{
    core::*,
    Win32::{
        UI::WindowsAndMessaging::*,
    },
};

mod window;
mod app;
mod keyboard_manager;
mod keyboard_list;
mod keyboard_preview;
mod tray;
mod tsf_status;
mod logger;

use window::MainWindow;
use app::App;

fn main() -> Result<()> {
    // Initialize logger first
    if let Err(e) = logger::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
    }
    
    // Set up panic handler to log crashes
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = format!("PANIC: {}", panic_info);
        log_error!("{}", msg);
        eprintln!("{}", msg);
    }));
    
    log_info!("Starting KeyMagic GUI");
    
    let result = unsafe {
        // Initialize the application
        log_info!("Initializing application");
        let app = match App::new() {
            Ok(app) => app,
            Err(e) => {
                log_error!("Failed to initialize app: {}", e);
                return Err(e);
            }
        };
        
        // Create and show the main window
        log_info!("Creating main window");
        let window = match MainWindow::new(&app) {
            Ok(window) => window,
            Err(e) => {
                log_error!("Failed to create main window: {}", e);
                return Err(e);
            }
        };
        
        log_info!("Showing main window");
        window.show();
        
        log_info!("Entering message loop");
        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        
        log_info!("Message loop ended");
        Ok(())
    };
    
    if let Err(e) = &result {
        log_error!("Application error: {}", e);
    }
    
    log_info!("KeyMagic GUI shutting down");
    result
}