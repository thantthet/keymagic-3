use std::panic;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;
use std::env;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging()?;
    setup_crash_handler();
    Ok(())
}

fn setup_crash_handler() {
    // Set custom panic hook
    panic::set_hook(Box::new(|panic_info| {
        let crash_report = generate_crash_report(panic_info);
        
        // Try to save crash report to file
        let crash_path = save_crash_report(&crash_report).ok();
        
        if let Some(ref path) = crash_path {
            eprintln!("Crash report saved to: {}", path.display());
        }
        
        // Also print to stderr
        eprintln!("{}", crash_report);
        
        // Show error dialog if possible
        #[cfg(windows)]
        {
            use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONERROR};
            use windows::core::w;
            
            unsafe {
                let title = w!("KeyMagic Configuration Manager - Crash");
                let message = if let Some(path) = crash_path {
                    format!("The application has crashed.\n\nCrash report saved to:\n{}", path.display())
                } else {
                    format!("The application has crashed.\n\nCould not save crash report to disk.")
                };
                let message_w: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
                
                MessageBoxW(
                    None,
                    windows::core::PCWSTR::from_raw(message_w.as_ptr()),
                    title,
                    MB_OK | MB_ICONERROR,
                );
            }
        }
    }));
}

fn generate_crash_report(panic_info: &panic::PanicInfo) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("=== KeyMagic GUI Crash Report ===\n"));
    report.push_str(&format!("Time: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
    report.push_str(&format!("Version: {}\n", env!("CARGO_PKG_VERSION")));
    
    // Panic location
    if let Some(location) = panic_info.location() {
        report.push_str(&format!("\nPanic Location:\n"));
        report.push_str(&format!("  File: {}\n", location.file()));
        report.push_str(&format!("  Line: {}\n", location.line()));
        report.push_str(&format!("  Column: {}\n", location.column()));
    }
    
    // Panic message
    report.push_str(&format!("\nPanic Message:\n"));
    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        report.push_str(&format!("  {}\n", s));
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
        report.push_str(&format!("  {}\n", s));
    } else {
        report.push_str("  (Unknown panic payload)\n");
    }
    
    // Backtrace
    report.push_str(&format!("\nBacktrace:\n"));
    let backtrace = std::backtrace::Backtrace::force_capture();
    report.push_str(&format!("{}\n", backtrace));
    
    // System info
    report.push_str(&format!("\nSystem Information:\n"));
    report.push_str(&format!("  OS: {}\n", std::env::consts::OS));
    report.push_str(&format!("  Architecture: {}\n", std::env::consts::ARCH));
    
    // Environment variables that might be relevant
    report.push_str(&format!("\nRelevant Environment:\n"));
    for (key, value) in env::vars() {
        if key.starts_with("SLINT_") || key.starts_with("RUST_") {
            report.push_str(&format!("  {}={}\n", key, value));
        }
    }
    
    report
}

fn save_crash_report(report: &str) -> std::io::Result<PathBuf> {
    let crash_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("KeyMagic")
        .join("crashes");
    
    create_dir_all(&crash_dir)?;
    
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let crash_file = crash_dir.join(format!("crash_{}.txt", timestamp));
    
    let mut file = File::create(&crash_file)?;
    file.write_all(report.as_bytes())?;
    
    Ok(crash_file)
}

// Setup logging to file
pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    use env_logger::Builder;
    use std::io::Write;
    
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("KeyMagic")
        .join("logs");
    
    create_dir_all(&log_dir)?;
    
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let log_file = log_dir.join(format!("keymagic_gui_{}.log", timestamp));
    
    let log_file = File::create(log_file)?;
    
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {} - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Debug)
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();
    
    Ok(())
}