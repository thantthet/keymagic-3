// Prevents additional console window on Windows, DO NOT REMOVE!!
#![windows_subsystem = "windows"]

use std::env;
use std::path::PathBuf;

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
        // Check if a .km2 file was passed as argument
        let km2_file = args.get(1)
            .filter(|arg| !arg.starts_with("--"))
            .and_then(|arg| {
                let path = PathBuf::from(arg);
                if path.extension().and_then(|e| e.to_str()) == Some("km2") && path.exists() {
                    Some(path)
                } else {
                    None
                }
            });
        
        // Normal GUI execution with optional file to open
        gui_tauri_lib::run_with_file(km2_file)
    }
}
