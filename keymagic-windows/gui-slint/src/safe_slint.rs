use anyhow::{Result, Context};
use log::{debug, warn, error};
use slint::ComponentHandle;

/// Validate Slint backend before starting
pub fn validate_slint_backend() -> Result<()> {
    debug!("Validating Slint backend...");
    
    // Check if we're forcing software rendering
    if let Ok(backend) = std::env::var("SLINT_BACKEND") {
        debug!("SLINT_BACKEND set to: {}", backend);
    }
    
    // Check for known problematic GPU drivers on Windows
    #[cfg(windows)]
    {
        if should_force_software_rendering() {
            warn!("Detected potential GPU compatibility issues, forcing software rendering");
            std::env::set_var("SLINT_BACKEND", "software");
        }
    }
    
    Ok(())
}

/// Check if we should force software rendering
#[cfg(all(windows, target_arch = "aarch64"))]
fn should_force_software_rendering() -> bool {
    // Check for specific conditions that might require software rendering
    // This is a conservative approach for ARM64
    if std::env::consts::ARCH == "aarch64" {
        // On ARM64, default to software rendering unless explicitly disabled
        if let Ok(val) = std::env::var("SLINT_ALLOW_HARDWARE") {
            return val != "1" && val.to_lowercase() != "true";
        }
        return true;
    }
    false
}

#[cfg(all(windows, not(target_arch = "aarch64")))]
fn should_force_software_rendering() -> bool {
    false
}

#[cfg(not(windows))]
fn should_force_software_rendering() -> bool {
    false
}

/// Safely show a window with error handling
pub fn safe_show_window<T: ComponentHandle>(window: &T, window_name: &str) -> Result<()> {
    debug!("Attempting to show {} window", window_name);
    
    // Verify window is valid before showing
    if window.as_weak().upgrade().is_none() {
        return Err(anyhow::anyhow!("Window handle is invalid"));
    }
    
    window.show()
        .with_context(|| format!("Failed to show {} window", window_name))?;
    
    debug!("{} window shown successfully", window_name);
    Ok(())
}

/// Monitor window health
pub fn monitor_rendering_health<T: ComponentHandle>(window: &T) -> bool {
    if window.as_weak().upgrade().is_none() {
        error!("Window handle became invalid!");
        return false;
    }
    
    // Additional health checks could go here
    true
}

/// Wrap operations that might panic due to Slint issues
pub fn safe_slint_operation<F, R>(operation: F, operation_name: &str) -> Result<R>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    debug!("Executing safe Slint operation: {}", operation_name);
    
    std::panic::catch_unwind(operation)
        .map_err(|e| {
            error!("Panic in Slint operation '{}': {:?}", operation_name, e);
            anyhow::anyhow!("Slint operation '{}' panicked", operation_name)
        })
}

/// Safe event loop operation with panic recovery
pub fn safe_event_loop_operation<F, R>(operation: F, operation_name: &str) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    debug!("Starting event loop operation: {}", operation_name);
    
    match operation() {
        Ok(result) => {
            debug!("Event loop operation '{}' completed successfully", operation_name);
            Ok(result)
        }
        Err(e) => {
            error!("Event loop operation '{}' failed: {}", operation_name, e);
            
            // Check for specific Slint errors
            let error_str = format!("{:?}", e);
            if error_str.contains("OpenGL") || error_str.contains("GL") {
                error!("Graphics-related error detected. Try setting SLINT_BACKEND=software");
            }
            if error_str.contains("window") || error_str.contains("surface") {
                error!("Window/surface error detected. This might be a display driver issue.");
            }
            
            Err(e)
        }
    }
}