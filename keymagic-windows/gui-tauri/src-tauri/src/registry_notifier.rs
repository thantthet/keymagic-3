#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{CreateEventW, SetEvent};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{CloseHandle, HANDLE, FALSE};
#[cfg(target_os = "windows")]
use windows::core::w;

use anyhow::Result;

pub struct RegistryNotifier {
    #[cfg(target_os = "windows")]
    reload_event: HANDLE,
}

impl RegistryNotifier {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                // Create/Open the event for notifying TSF instances
                let reload_event = CreateEventW(
                    None,
                    FALSE,  // Auto-reset
                    FALSE,  // Initial state
                    w!("Global\\KeyMagicReloadRegistry"),
                )?;
                
                Ok(Self { reload_event })
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Ok(Self {})
        }
    }
    
    pub fn notify_registry_changed(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                // Signal all TSF instances to reload
                SetEvent(self.reload_event)?;
                Ok(())
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for RegistryNotifier {
    fn drop(&mut self) {
        unsafe {
            if self.reload_event != HANDLE::default() {
                CloseHandle(self.reload_event);
            }
        }
    }
}

// Helper function to notify after registry changes
pub fn save_settings_and_notify<F>(save_fn: F) -> Result<()>
where
    F: FnOnce() -> Result<()>
{
    // 1. Write to registry
    save_fn()?;
    
    // 2. Signal all TSF instances
    let notifier = RegistryNotifier::new()?;
    notifier.notify_registry_changed()?;
    
    Ok(())
}