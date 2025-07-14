use anyhow::Result;
use crate::windows_event::WindowsEvent;

pub struct RegistryNotifier;

impl RegistryNotifier {
    /// Notify all TSF instances to reload registry settings using Windows Event
    pub fn notify_registry_changed() -> Result<()> {
        println!("[RegistryNotifier] Sending registry reload notification to TSF instances via Windows Event");
        
        #[cfg(target_os = "windows")]
        {
            // Create or open the global event
            match WindowsEvent::create_or_open() {
                Ok(event) => {
                    // Signal the event
                    event.signal()
                        .map_err(|e| anyhow::anyhow!("Failed to signal event: {:?}", e))?;
                    println!("[RegistryNotifier] Event signaled successfully");
                }
                Err(e) => {
                    eprintln!("[RegistryNotifier] Failed to create/open event: {:?}", e);
                    return Err(anyhow::anyhow!("Failed to create/open event: {:?}", e));
                }
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            println!("[RegistryNotifier] Registry notification skipped (not Windows)");
        }
        
        Ok(())
    }
}

// Helper function to notify after registry changes
pub fn save_settings_and_notify<F>(save_fn: F) -> Result<()>
where
    F: FnOnce() -> Result<()>
{
    println!("[RegistryNotifier] save_settings_and_notify: Starting registry update");
    
    // 1. Write to registry
    save_fn()?;
    println!("[RegistryNotifier] save_settings_and_notify: Registry write completed");
    
    // 2. Notify all TSF instances via SendInput
    RegistryNotifier::notify_registry_changed()?;
    println!("[RegistryNotifier] save_settings_and_notify: Notification sent");
    
    Ok(())
}