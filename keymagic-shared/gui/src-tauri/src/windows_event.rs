#[cfg(target_os = "windows")]
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{SECURITY_ATTRIBUTES, SECURITY_DESCRIPTOR, InitializeSecurityDescriptor, SetSecurityDescriptorDacl, PSECURITY_DESCRIPTOR};
use windows::Win32::System::Threading::{CreateEventW, SetEvent};
use std::iter::once;

const REGISTRY_EVENT_NAME: &str = "Global\\KeyMagicRegistryUpdate";

#[cfg(target_os = "windows")]
pub struct WindowsEvent {
    handle: HANDLE,
}

#[cfg(target_os = "windows")]
impl WindowsEvent {
    /// Creates or opens the global registry update event with NULL DACL for universal access
    pub fn create_or_open() -> Result<Self> {
        unsafe {
            // Create security descriptor on stack
            let mut sd = SECURITY_DESCRIPTOR::default();
            
            // Initialize security descriptor
            InitializeSecurityDescriptor(PSECURITY_DESCRIPTOR(&mut sd as *mut _ as *mut _), 1)?; // SECURITY_DESCRIPTOR_REVISION = 1
            
            // Set NULL DACL (allowing all access)
            SetSecurityDescriptorDacl(PSECURITY_DESCRIPTOR(&mut sd as *mut _ as *mut _), true, None, false)?;
            
            // Create security attributes
            let sa = SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: &mut sd as *mut _ as *mut _,
                bInheritHandle: false.into(),
            };
            
            // Create wide string for event name
            let event_name_wide: Vec<u16> = REGISTRY_EVENT_NAME
                .encode_utf16()
                .chain(once(0))
                .collect();
            
            // Create or open the event
            let handle = CreateEventW(
                Some(&sa as *const SECURITY_ATTRIBUTES),
                true,  // Manual reset
                false, // Initial state
                PCWSTR::from_raw(event_name_wide.as_ptr()),
            )?;
            
            if handle.is_invalid() {
                return Err(Error::from_win32());
            }
            
            Ok(WindowsEvent { handle })
        }
    }
    
    /// Signals the event to notify all waiting threads
    pub fn signal(&self) -> Result<()> {
        unsafe {
            SetEvent(self.handle)?;
            Ok(())
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for WindowsEvent {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_invalid() {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub struct WindowsEvent;

#[cfg(not(target_os = "windows"))]
impl WindowsEvent {
    pub fn create_or_open() -> anyhow::Result<Self> {
        Ok(WindowsEvent)
    }
    
    pub fn signal(&self) -> anyhow::Result<()> {
        Ok(())
    }
}