use windows::{
    core::*,
    Win32::{
        Foundation::*,
        System::Registry::*,
    },
};

const TSF_CLSID: &str = "{12345678-1234-1234-1234-123456789ABC}";

pub struct TsfStatus;

impl TsfStatus {
    /// Check if the TSF is registered in the registry
    pub fn is_registered() -> bool {
        // Check COM registration
        if !Self::check_com_registration() {
            return false;
        }
        
        // Check TSF registration
        Self::check_tsf_registration()
    }
    
    /// Check if the TSF COM server is registered
    fn check_com_registration() -> bool {
        unsafe {
            let key_path = format!("CLSID\\{}", TSF_CLSID);
            let key_path_w: Vec<u16> = key_path.encode_utf16().chain(std::iter::once(0)).collect();
            
            let mut hkey = HKEY::default();
            let result = RegOpenKeyExW(
                HKEY_CLASSES_ROOT,
                PCWSTR(key_path_w.as_ptr()),
                0,
                KEY_READ,
                &mut hkey,
            );
            
            if result.is_ok() {
                RegCloseKey(hkey);
                true
            } else {
                false
            }
        }
    }
    
    /// Check if the TSF is registered with the Text Services Framework
    fn check_tsf_registration() -> bool {
        unsafe {
            let key_path = format!("SOFTWARE\\Microsoft\\CTF\\TIP\\{}", TSF_CLSID);
            let key_path_w: Vec<u16> = key_path.encode_utf16().chain(std::iter::once(0)).collect();
            
            let mut hkey = HKEY::default();
            let result = RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                PCWSTR(key_path_w.as_ptr()),
                0,
                KEY_READ,
                &mut hkey,
            );
            
            if result.is_ok() {
                RegCloseKey(hkey);
                true
            } else {
                false
            }
        }
    }
    
    /// Check if the Text Services Framework is running (ctfmon.exe)
    pub fn is_ctfmon_running() -> bool {
        unsafe {
            use windows::Win32::System::Diagnostics::ToolHelp::*;
            
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot.is_err() {
                return false;
            }
            
            let snapshot = snapshot.unwrap();
            
            let mut process_entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            
            if Process32FirstW(snapshot, &mut process_entry).is_ok() {
                loop {
                    let process_name = String::from_utf16_lossy(&process_entry.szExeFile)
                        .trim_end_matches('\0')
                        .to_lowercase();
                    
                    if process_name == "ctfmon.exe" {
                        let _ = CloseHandle(snapshot);
                        return true;
                    }
                    
                    if Process32NextW(snapshot, &mut process_entry).is_err() {
                        break;
                    }
                }
            }
            
            let _ = CloseHandle(snapshot);
            false
        }
    }
    
    /// Get a user-friendly status message
    pub fn get_status_message() -> String {
        let com_registered = Self::check_com_registration();
        let tsf_registered = Self::check_tsf_registration();
        let ctfmon_running = Self::is_ctfmon_running();
        
        if !com_registered {
            "TSF not installed (COM not registered)".to_string()
        } else if !tsf_registered {
            "TSF not registered with Windows".to_string()
        } else if !ctfmon_running {
            "TSF registered but Text Services not running".to_string()
        } else {
            "TSF registered and ready".to_string()
        }
    }
    
    /// Start ctfmon.exe if it's not running
    pub fn start_ctfmon() -> Result<()> {
        if Self::is_ctfmon_running() {
            return Ok(());
        }
        
        // Use a simple approach - just try to start it
        // Windows will handle if it's already running
        unsafe {
            use windows::Win32::UI::Shell::*;
            use windows::Win32::UI::WindowsAndMessaging::*;
            
            let result = ShellExecuteW(
                None,
                w!("open"),
                w!("ctfmon.exe"),
                None,
                None,
                SW_HIDE,
            );
            
            if result.0 as usize > 32 {
                Ok(())
            } else {
                Err(Error::from_win32())
            }
        }
    }
}