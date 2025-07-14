#[cfg(target_os = "windows")]
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{SECURITY_ATTRIBUTES, SECURITY_DESCRIPTOR, InitializeSecurityDescriptor, SetSecurityDescriptorDacl, PSECURITY_DESCRIPTOR};
use windows::Win32::System::Threading::{CreateEventW, SetEvent};
use std::iter::once;
use std::sync::Mutex;

const GLOBAL_EVENT_NAME: &str = "Global\\KeyMagicRegistryUpdate";

#[cfg(target_os = "windows")]
pub struct WindowsEvent {
    handle: HANDLE,
}

#[cfg(target_os = "windows")]
impl WindowsEvent {
    /// Creates or opens the global event with NULL DACL for universal access
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
            let event_name_wide: Vec<u16> = GLOBAL_EVENT_NAME
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

#[cfg(target_os = "windows")]
static GLOBAL_EVENT: Mutex<Option<WindowsEvent>> = Mutex::new(None);

/// Initialize the global event at startup (creates it if it doesn't exist)
#[cfg(target_os = "windows")]
pub fn initialize_global_event() -> Result<()> {
    // Create the event at startup to ensure it exists for TSF
    let event = WindowsEvent::create_or_open()?;
    
    // Store the event handle globally to keep it alive
    if let Ok(mut global) = GLOBAL_EVENT.lock() {
        *global = Some(event);
    }
    
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn initialize_global_event() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub struct WindowsEvent;

#[cfg(not(target_os = "windows"))]
impl WindowsEvent {
    pub fn create_or_open() -> anyhow::Result<Self> {
        Ok(WindowsEvent)
    }
    
    pub fn open_existing() -> anyhow::Result<Self> {
        Ok(WindowsEvent)
    }
    
    pub fn signal(&self) -> anyhow::Result<()> {
        Ok(())
    }
}