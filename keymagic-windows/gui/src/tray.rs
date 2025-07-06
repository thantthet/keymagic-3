use windows::{
    core::*,
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        UI::Shell::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::Input::KeyboardAndMouse::*,
    },
};
use std::sync::{Arc, Mutex};
use crate::keyboard_manager::KeyboardManager;
use crate::hud::HudNotification;

// Tray icon constants
const WM_TRAYICON: u32 = WM_USER + 100;
const TRAY_ICON_ID: u32 = 1;

// Hotkey constants
const HOTKEY_ID_TOGGLE: i32 = 1;

// Tray menu command IDs
const ID_TRAY_SHOW: u16 = 1001;
const ID_TRAY_EXIT: u16 = 1002;
const ID_TRAY_TOGGLE_KEY_PROCESSING: u16 = 1003;
const ID_TRAY_SETTINGS: u16 = 1004;
const ID_TRAY_KEYBOARD_BASE: u16 = 2000; // Base ID for dynamic keyboard menu items

// Export the tray message constant
pub fn tray_message() -> u32 {
    WM_TRAYICON
}

pub struct TrayIcon {
    hwnd: HWND,
    keyboard_manager: Arc<Mutex<KeyboardManager>>,
    icon_enabled: HICON,
    icon_disabled: HICON,
    hud: Arc<HudNotification>,
}

impl TrayIcon {
    pub fn new(hwnd: HWND, keyboard_manager: Arc<Mutex<KeyboardManager>>) -> Result<Self> {
        unsafe {
            // Load icons - use the same icon for now, can customize later
            let icon_enabled = LoadIconW(GetModuleHandleW(None)?, PCWSTR(1 as *const u16))?;
            let icon_disabled = LoadIconW(GetModuleHandleW(None)?, PCWSTR(1 as *const u16))?;
            
            // Create HUD notification
            let hud = HudNotification::new()?;
            
            let mut tray = TrayIcon {
                hwnd,
                keyboard_manager,
                icon_enabled,
                icon_disabled,
                hud,
            };
            
            tray.create_icon()?;
            tray.register_toggle_hotkey()?;
            Ok(tray)
        }
    }
    
    fn register_toggle_hotkey(&mut self) -> Result<()> {
        unsafe {
            // Read hotkey from registry (default: Ctrl+Shift+Space)
            let hotkey_str = self.keyboard_manager.lock().unwrap()
                .read_registry_value("Settings\\ToggleHotkey")
                .unwrap_or_else(|| "Ctrl+Shift+Space".to_string());
            
            println!("Registering hotkey: {}", hotkey_str);
            
            // Parse hotkey string
            let (vk, modifiers) = KeyboardManager::parse_hotkey_string(&hotkey_str)
                .ok_or_else(|| Error::from_win32())?;
            
            println!("Parsed hotkey - modifiers: {}, vk: {}", modifiers, vk);
            
            // Register with Windows
            println!("Calling RegisterHotKey with hwnd: {:?}, id: {}, modifiers: 0x{:X}, vk: 0x{:X}", 
                self.hwnd, HOTKEY_ID_TOGGLE, modifiers, vk);
            
            if RegisterHotKey(self.hwnd, HOTKEY_ID_TOGGLE, HOT_KEY_MODIFIERS(modifiers), vk).is_err() {
                let error = Error::from_win32();
                println!("Failed to register custom hotkey: {:?}", error);
                
                // Try with default if custom hotkey fails
                let default_modifiers = MOD_CONTROL.0 | MOD_SHIFT.0;
                let default_vk = VK_SPACE.0 as u32;
                println!("Trying default hotkey with modifiers: 0x{:X}, vk: 0x{:X}", 
                    default_modifiers, default_vk);
                
                if RegisterHotKey(self.hwnd, HOTKEY_ID_TOGGLE, 
                    HOT_KEY_MODIFIERS(default_modifiers), default_vk).is_err() {
                    let error = Error::from_win32();
                    println!("Failed to register default hotkey: {:?}", error);
                    return Err(error);
                }
                println!("Registered default hotkey: Ctrl+Shift+Space");
            } else {
                println!("Successfully registered hotkey");
            }
        }
        
        Ok(())
    }
    
    
    pub fn unregister_hotkey(&self) -> Result<()> {
        unsafe {
            UnregisterHotKey(self.hwnd, HOTKEY_ID_TOGGLE);
        }
        Ok(())
    }
    
    pub fn reregister_hotkey(&mut self) -> Result<()> {
        // Unregister the old hotkey first
        self.unregister_hotkey()?;
        
        // Register the new hotkey
        self.register_toggle_hotkey()
    }
    
    pub fn toggle_key_processing_enabled(&self) -> Result<()> {
        let manager = self.keyboard_manager.lock().unwrap();
        let enabled = manager.is_key_processing_enabled();
        if let Err(e) = manager.set_key_processing_enabled(!enabled) {
            drop(manager);
            return Err(Error::new(HRESULT(-1), format!("Failed to set key processing enabled: {}", e).into()));
        }
        
        // Update tray icon and tooltip
        drop(manager); // Release lock before calling update_icon
        self.update_icon(!enabled)?;
        
        // Show HUD notification
        self.hud.show(
            if !enabled { "KeyMagic enabled" } else { "KeyMagic disabled" },
            !enabled
        )?;
        
        Ok(())
    }
    
    fn show_notification(&self, title: &str, text: &str) -> Result<()> {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ICON_ID,
                uFlags: NIF_INFO,
                dwInfoFlags: NIIF_INFO,
                szInfoTitle: {
                    let mut buf = [0u16; 64];
                    let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
                    let len = wide.len().min(63);
                    buf[..len].copy_from_slice(&wide[..len]);
                    buf
                },
                szInfo: {
                    let mut buf = [0u16; 256];
                    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                    let len = wide.len().min(255);
                    buf[..len].copy_from_slice(&wide[..len]);
                    buf
                },
                ..Default::default()
            };
            
            Shell_NotifyIconW(NIM_MODIFY, &mut nid).ok();
        }
        
        Ok(())
    }
    
    pub fn update_icon(&self, enabled: bool) -> Result<()> {
        let manager = self.keyboard_manager.lock().unwrap();
        let tooltip = if enabled {
            "KeyMagic - Enabled"
        } else {
            "KeyMagic - Disabled"
        };
        
        drop(manager); // Release lock
        
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ICON_ID,
                uFlags: NIF_ICON | NIF_TIP,
                hIcon: if enabled { self.icon_enabled } else { self.icon_disabled },
                szTip: {
                    let mut tip = [0u16; 128];
                    let wide: Vec<u16> = tooltip.encode_utf16().chain(std::iter::once(0)).collect();
                    let len = wide.len().min(127);
                    tip[..len].copy_from_slice(&wide[..len]);
                    tip
                },
                ..Default::default()
            };
            
            Shell_NotifyIconW(NIM_MODIFY, &mut nid).ok();
        }
        
        Ok(())
    }
    
    fn create_icon(&self) -> Result<()> {
        let manager = self.keyboard_manager.lock().unwrap();
        let enabled = manager.is_key_processing_enabled();
        let tooltip = if enabled {
            "KeyMagic - Enabled"
        } else {
            "KeyMagic - Disabled"
        };
        
        drop(manager); // Release lock
        
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ICON_ID,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: if enabled { self.icon_enabled } else { self.icon_disabled },
                szTip: {
                    let mut tip = [0u16; 128];
                    let wide: Vec<u16> = tooltip.encode_utf16().chain(std::iter::once(0)).collect();
                    let len = wide.len().min(127);
                    tip[..len].copy_from_slice(&wide[..len]);
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
            let key_processing_enabled = manager.is_key_processing_enabled();
            
            // Add key processing toggle
            let toggle_flags = if key_processing_enabled { MF_STRING | MF_CHECKED } else { MF_STRING };
            AppendMenuW(menu, toggle_flags, ID_TRAY_TOGGLE_KEY_PROCESSING as usize, w!("Enable KeyMagic"))?;
            AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            
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
            ID_TRAY_TOGGLE_KEY_PROCESSING => {
                // Toggle key processing enabled state
                let manager = self.keyboard_manager.lock().unwrap();
                let current_state = manager.is_key_processing_enabled();
                if let Err(e) = manager.set_key_processing_enabled(!current_state) {
                    eprintln!("Failed to toggle key processing: {}", e);
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
        let _ = self.unregister_hotkey();
        let _ = self.remove();
    }
}

