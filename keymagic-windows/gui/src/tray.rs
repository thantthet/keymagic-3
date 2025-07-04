use windows::{
    core::*,
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        UI::Shell::*,
        System::LibraryLoader::GetModuleHandleW,
    },
};
use std::sync::{Arc, Mutex};
use crate::keyboard_manager::KeyboardManager;

// Tray icon constants
const WM_TRAYICON: u32 = WM_USER + 100;
const TRAY_ICON_ID: u32 = 1;

// Tray menu command IDs
const ID_TRAY_SHOW: u16 = 1001;
const ID_TRAY_EXIT: u16 = 1002;
const ID_TRAY_KEYBOARD_BASE: u16 = 2000; // Base ID for dynamic keyboard menu items

pub struct TrayIcon {
    hwnd: HWND,
    keyboard_manager: Arc<Mutex<KeyboardManager>>,
}

impl TrayIcon {
    pub fn new(hwnd: HWND, keyboard_manager: Arc<Mutex<KeyboardManager>>) -> Result<Self> {
        let tray = TrayIcon {
            hwnd,
            keyboard_manager,
        };
        
        tray.create_icon()?;
        Ok(tray)
    }
    
    fn create_icon(&self) -> Result<()> {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ICON_ID,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: LoadIconW(GetModuleHandleW(None)?, PCWSTR(1 as *const u16))?, // IDI_KEYMAGIC
                szTip: {
                    let mut tip = [0u16; 128];
                    let tip_text = w!("KeyMagic Configuration Manager");
                    let len = tip_text.as_wide().len().min(127);
                    tip[..len].copy_from_slice(&tip_text.as_wide()[..len]);
                    tip
                },
                ..Default::default()
            };
            
            Shell_NotifyIconW(NIM_ADD, &mut nid).ok();
        }
        
        Ok(())
    }
    
    pub fn update_tooltip(&self, text: &str) -> Result<()> {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ICON_ID,
                uFlags: NIF_TIP,
                szTip: {
                    let mut tip = [0u16; 128];
                    let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                    let len = wide_text.len().min(127);
                    tip[..len].copy_from_slice(&wide_text[..len]);
                    tip
                },
                ..Default::default()
            };
            
            Shell_NotifyIconW(NIM_MODIFY, &mut nid).ok();
        }
        
        Ok(())
    }
    
    pub fn show_context_menu(&self, x: i32, y: i32) -> Result<()> {
        unsafe {
            // Create popup menu
            let menu = CreatePopupMenu()?;
            
            // Add static menu items
            AppendMenuW(menu, MF_STRING, ID_TRAY_SHOW as usize, w!("Show KeyMagic"))?;
            AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            
            // Add keyboard items
            let manager = self.keyboard_manager.lock().unwrap();
            let keyboards = manager.get_keyboards();
            let active_id = manager.get_active_keyboard();
            
            if !keyboards.is_empty() {
                // Add keyboard section header
                AppendMenuW(menu, MF_STRING | MF_DISABLED, 0, w!("Keyboards:"))?;
                
                // Add each keyboard
                for (index, info) in keyboards.iter().enumerate() {
                    let menu_id = ID_TRAY_KEYBOARD_BASE + index as u16;
                    let flags = if Some(info.id.as_str()) == active_id {
                        MF_STRING | MF_CHECKED
                    } else {
                        MF_STRING
                    };
                    
                    let name_wide: Vec<u16> = info.name.encode_utf16().chain(std::iter::once(0)).collect();
                    AppendMenuW(menu, flags, menu_id as usize, PCWSTR(name_wide.as_ptr()))?;
                }
                
                AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            }
            
            // Add exit item
            AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT as usize, w!("Exit"))?;
            
            // Show the menu
            let _ = SetForegroundWindow(self.hwnd);
            TrackPopupMenuEx(
                menu,
                (TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON).0,
                x,
                y,
                self.hwnd,
                None,
            );
            
            DestroyMenu(menu)?;
        }
        
        Ok(())
    }
    
    pub fn handle_menu_command(&self, cmd: u16) -> Result<()> {
        match cmd {
            ID_TRAY_SHOW => {
                unsafe {
                    ShowWindow(self.hwnd, SW_SHOW);
                    SetForegroundWindow(self.hwnd).ok();
                }
            }
            ID_TRAY_EXIT => {
                unsafe {
                    PostMessageW(self.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0))?;
                }
            }
            cmd if cmd >= ID_TRAY_KEYBOARD_BASE => {
                // Handle keyboard selection
                let index = (cmd - ID_TRAY_KEYBOARD_BASE) as usize;
                let mut manager = self.keyboard_manager.lock().unwrap();
                let keyboards = manager.get_keyboards();
                
                if index < keyboards.len() {
                    let keyboard_id = keyboards[index].id.clone();
                    drop(keyboards);
                    if let Err(e) = manager.set_active_keyboard(&keyboard_id) {
                        // Log error or show notification
                        eprintln!("Failed to activate keyboard: {}", e);
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    pub fn remove(&self) -> Result<()> {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ICON_ID,
                ..Default::default()
            };
            
            Shell_NotifyIconW(NIM_DELETE, &mut nid).ok();
        }
        
        Ok(())
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        let _ = self.remove();
    }
}

// Export the tray message constant for use in window procedure
pub const fn tray_message() -> u32 {
    WM_TRAYICON
}