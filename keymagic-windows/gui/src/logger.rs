use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use std::path::PathBuf;

lazy_static::lazy_static! {
    static ref LOG_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);
}

pub fn init_logger() -> Result<(), std::io::Error> {
    let log_path = std::env::current_exe()?
        .parent()
        .unwrap()
        .join("keymagic-gui.log");
    
    *LOG_FILE.lock().unwrap() = Some(log_path.clone());
    
    // Clear existing log
    std::fs::write(&log_path, "")?;
    
    log_info("KeyMagic GUI started");
    log_info(&format!("Log file: {}", log_path.display()));
    
    Ok(())
}

pub fn log_info(message: &str) {
    log_message("INFO", message);
}

pub fn log_error(message: &str) {
    log_message("ERROR", message);
}

pub fn log_debug(message: &str) {
    #[cfg(debug_assertions)]
    log_message("DEBUG", message);
    #[cfg(not(debug_assertions))]
    let _ = message; // Suppress unused warning in release builds
}

fn log_message(level: &str, message: &str) {
    if let Some(log_path) = LOG_FILE.lock().unwrap().as_ref() {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_line = format!("[{}] [{}] {}\n", timestamp, level, message);
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
        {
            let _ = file.write_all(log_line.as_bytes());
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::log_info(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::log_error(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::log_debug(&format!($($arg)*));
    };
}