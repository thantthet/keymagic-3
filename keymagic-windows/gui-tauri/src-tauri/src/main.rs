// Prevents additional console window on Windows, DO NOT REMOVE!!
#![windows_subsystem = "windows"]

use std::env;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check if we're running in elevated mode for language updates
    if args.len() >= 3 && args[1] == "--update-languages" {
        // Running elevated to update language profiles
        match gui_tauri_lib::update_languages_elevated(&args[2]) {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("Failed to update language profiles: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Normal GUI execution
        gui_tauri_lib::run()
    }
}
