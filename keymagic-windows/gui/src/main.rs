use windows::{
    core::*,
    Win32::{
        UI::WindowsAndMessaging::*,
    },
};

mod window;
mod app;
mod keyboard_manager_simple;
mod keyboard_list;

use window::MainWindow;
use app::App;

fn main() -> Result<()> {
    unsafe {
        // Initialize the application
        let app = App::new()?;
        
        // Create and show the main window
        let window = MainWindow::new(&app)?;
        window.show();
        
        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        
        Ok(())
    }
}