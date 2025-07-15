use anyhow::Result;
use crate::windows_event::WindowsEvent;
use log::{debug, error};

pub struct RegistryNotifier;

impl RegistryNotifier {
    /// Notify all TSF instances to reload registry settings using Windows Event
    pub fn notify_registry_changed() -> Result<()> {
        debug!("[RegistryNotifier] Sending registry reload notification to TSF instances via Windows Event");
        
        #[cfg(target_os = "windows")]
        {
            // Create or open the global event
            match WindowsEvent::create_or_open() {
                Ok(event) => {
                    // Signal the event
                    event.signal()
                        .map_err(|e| anyhow::anyhow!("Failed to signal event: {:?}", e))?;
                    debug!("[RegistryNotifier] Event signaled successfully");
                }
                Err(e) => {
                    error!("[RegistryNotifier] Failed to create/open event: {:?}", e);
                    return Err(anyhow::anyhow!("Failed to create/open event: {:?}", e));
                }
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            debug!("[RegistryNotifier] Registry notification skipped (not Windows)");
        }
        
        Ok(())
    }
}

