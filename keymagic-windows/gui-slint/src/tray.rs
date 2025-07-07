use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use anyhow::Result;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        System::LibraryLoader::*,
        UI::Shell::*,
        UI::WindowsAndMessaging::*,
    },
};

const WM_USER_TRAY: u32 = WM_USER + 1;
const TRAY_ICON_ID: u32 = 1;

pub struct TrayIcon {
    hwnd: HWND,
    icon: HICON,
    tip: String,
    menu: HMENU,
    visible: bool,
}

impl TrayIcon {
    fn load_icon_from_resource() -> Result<HICON> {
        unsafe {
            // Get the module handle for the current executable
            let hinstance = GetModuleHandleW(None)?;
            
            // Load the icon from embedded resource
            // Resource ID 1 is set in build.rs: "1 ICON ..."
            let icon = LoadImageW(
                hinstance,
                PCWSTR(1 as *const u16), // Resource ID 1
                IMAGE_ICON,
                16, // Width for system tray (small icon)
                16, // Height for system tray (small icon)
                LR_DEFAULTCOLOR,
            )?;
            
            if !icon.is_invalid() {
                Ok(HICON(icon.0))
            } else {
                // Fallback to default application icon if custom icon not found
                let icon = LoadIconW(hinstance, IDI_APPLICATION)?;
                Ok(icon)
            }
        }
    }
    pub fn new(hwnd: HWND) -> Result<Self> {
        unsafe {
            // Load icon from embedded resource
            let icon = Self::load_icon_from_resource()?;
            
            // Create popup menu for tray
            let menu = CreatePopupMenu()?;
            
            Ok(Self {
                hwnd,
                icon,
                tip: "KeyMagic".to_string(),
                menu,
                visible: false,
            })
        }
    }
    
    pub fn show(&mut self) -> Result<()> {
        if self.visible {
            return Ok(());
        }
        
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ICON_ID,
                uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                uCallbackMessage: WM_USER_TRAY,
                hIcon: self.icon,
                ..Default::default()
            };
            
            // Set tooltip
            let tip_wide: Vec<u16> = self.tip.encode_utf16().chain(std::iter::once(0)).collect();
            nid.szTip[..tip_wide.len().min(128)].copy_from_slice(&tip_wide[..tip_wide.len().min(128)]);
            
            if Shell_NotifyIconW(NIM_ADD, &nid).as_bool() {
                self.visible = true;
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to add tray icon"))
            }
        }
    }
    
    pub fn hide(&mut self) -> Result<()> {
        if !self.visible {
            return Ok(());
        }
        
        unsafe {
            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ICON_ID,
                ..Default::default()
            };
            
            if Shell_NotifyIconW(NIM_DELETE, &nid).as_bool() {
                self.visible = false;
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to remove tray icon"))
            }
        }
    }
    
    pub fn update_tooltip(&mut self, tip: &str) -> Result<()> {
        self.tip = tip.to_string();
        
        if !self.visible {
            return Ok(());
        }
        
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ICON_ID,
                uFlags: NIF_TIP,
                ..Default::default()
            };
            
            // Set tooltip
            let tip_wide: Vec<u16> = self.tip.encode_utf16().chain(std::iter::once(0)).collect();
            nid.szTip[..tip_wide.len().min(128)].copy_from_slice(&tip_wide[..tip_wide.len().min(128)]);
            
            if Shell_NotifyIconW(NIM_MODIFY, &nid).as_bool() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to update tray tooltip"))
            }
        }
    }
    
    pub fn update_icon(&mut self, icon: HICON) -> Result<()> {
        self.icon = icon;
        
        if !self.visible {
            return Ok(());
        }
        
        unsafe {
            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ICON_ID,
                uFlags: NIF_ICON,
                hIcon: self.icon,
                ..Default::default()
            };
            
            if Shell_NotifyIconW(NIM_MODIFY, &nid).as_bool() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to update tray icon"))
            }
        }
    }
    
    pub fn build_menu(&mut self, keyboards: &[(String, String, bool)], active_id: Option<&str>) -> Result<()> {
        unsafe {
            // Clear existing menu
            let count = GetMenuItemCount(self.menu);
            for _ in 0..count {
                RemoveMenu(self.menu, 0, MF_BYPOSITION);
            }
            
            // Add keyboards section
            let mut index = 0;
            for (id, name, _enabled) in keyboards {
                let menu_text = if Some(id.as_str()) == active_id {
                    format!("✓ {}", name)
                } else {
                    name.clone()
                };
                
                let text_wide: Vec<u16> = menu_text.encode_utf16().chain(std::iter::once(0)).collect();
                
                AppendMenuW(
                    self.menu,
                    MF_STRING,
                    (1000 + index) as usize,
                    PCWSTR(text_wide.as_ptr()),
                )?;
                
                index += 1;
            }
            
            // Add separator
            if !keyboards.is_empty() {
                AppendMenuW(self.menu, MF_SEPARATOR, 0, None)?;
            }
            
            // Add static menu items
            AppendMenuW(
                self.menu,
                MF_STRING,
                2000,
                w!("&Open KeyMagic"),
            )?;
            
            AppendMenuW(
                self.menu,
                MF_STRING,
                2001,
                w!("&Exit"),
            )?;
            
            Ok(())
        }
    }
    
    pub fn show_menu(&self) -> Result<u32> {
        unsafe {
            // Get cursor position
            let mut pt = POINT::default();
            GetCursorPos(&mut pt)?;
            
            // Required to make menu disappear when clicking outside
            SetForegroundWindow(self.hwnd);
            
            // Show menu and get selection
            let cmd = TrackPopupMenu(
                self.menu,
                TPM_RETURNCMD | TPM_NONOTIFY | TPM_LEFTALIGN | TPM_BOTTOMALIGN,
                pt.x,
                pt.y,
                0,
                self.hwnd,
                None,
            );
            
            // Required to make menu disappear
            PostMessageW(self.hwnd, WM_NULL, WPARAM(0), LPARAM(0))?;
            
            Ok(cmd.0 as u32)
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        let _ = self.hide();
        unsafe {
            if !self.menu.is_invalid() {
                DestroyMenu(self.menu);
            }
        }
    }
}

pub struct TrayManager {
    hwnd: HWND,
    tray_icon: RefCell<Option<TrayIcon>>,
    keyboard_ids: RefCell<Vec<String>>,
    on_show_window: Arc<Mutex<dyn Fn() + Send + Sync>>,
    on_exit: Arc<Mutex<dyn Fn() + Send + Sync>>,
    on_keyboard_selected: Arc<Mutex<dyn Fn(&str) + Send + Sync>>,
}

impl TrayManager {
    pub fn new(
        on_show_window: impl Fn() + Send + Sync + 'static,
        on_exit: impl Fn() + Send + Sync + 'static,
        on_keyboard_selected: impl Fn(&str) + Send + Sync + 'static,
    ) -> Result<Arc<Self>> {
        let manager = Arc::new(Self {
            hwnd: HWND::default(),
            tray_icon: RefCell::new(None),
            keyboard_ids: RefCell::new(Vec::new()),
            on_show_window: Arc::new(Mutex::new(on_show_window)),
            on_exit: Arc::new(Mutex::new(on_exit)),
            on_keyboard_selected: Arc::new(Mutex::new(on_keyboard_selected)),
        });
        
        // Create hidden window for tray messages
        let hwnd = Self::create_tray_window(manager.clone())?;
        
        // Update hwnd in manager (this is a bit of a hack but works)
        unsafe {
            let manager_ptr = Arc::as_ptr(&manager) as *mut Self;
            (*manager_ptr).hwnd = hwnd;
        }
        
        Ok(manager)
    }
    
    fn create_tray_window(manager: Arc<Self>) -> Result<HWND> {
        unsafe {
            let instance = GetModuleHandleW(None)?;
            
            let class_name = w!("KeyMagicTrayWindow");
            
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(Self::window_proc),
                hInstance: instance.into(),
                lpszClassName: class_name,
                ..Default::default()
            };
            
            RegisterClassExW(&wc);
            
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                w!("KeyMagic Tray Window"),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                instance,
                Some(Arc::into_raw(manager) as *const _),
            );
            
            if hwnd.0 == 0 {
                Err(anyhow::anyhow!("Failed to create tray window"))
            } else {
                Ok(hwnd)
            }
        }
    }
    
    pub fn show(&self) -> Result<()> {
        let mut tray_opt = self.tray_icon.borrow_mut();
        if tray_opt.is_none() {
            let mut tray = TrayIcon::new(self.hwnd)?;
            tray.show()?;
            *tray_opt = Some(tray);
        }
        Ok(())
    }
    
    pub fn hide(&self) -> Result<()> {
        if let Some(mut tray) = self.tray_icon.borrow_mut().take() {
            tray.hide()?;
        }
        Ok(())
    }
    
    pub fn update_menu(&self, keyboards: &[(String, String, bool)], active_id: Option<&str>) -> Result<()> {
        // Store keyboard IDs for mapping menu commands
        self.keyboard_ids.replace(keyboards.iter().map(|(id, _, _)| id.clone()).collect());
        
        if let Some(tray) = self.tray_icon.borrow_mut().as_mut() {
            tray.build_menu(keyboards, active_id)?;
        }
        Ok(())
    }
    
    pub fn update_tooltip(&self, tip: &str) -> Result<()> {
        if let Some(tray) = self.tray_icon.borrow_mut().as_mut() {
            tray.update_tooltip(tip)?;
        }
        Ok(())
    }
    
    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                let create_struct = lparam.0 as *const CREATESTRUCTW;
                let manager = (*create_struct).lpCreateParams as *const Self;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, manager as _);
                LRESULT(0)
            }
            WM_USER_TRAY => {
                let manager = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Self;
                if !manager.is_null() {
                    let manager = Arc::from_raw(manager);
                    
                    match lparam.0 as u32 {
                        WM_LBUTTONDBLCLK => {
                            if let Ok(callback) = manager.on_show_window.lock() {
                                callback();
                            }
                        }
                        WM_RBUTTONUP => {
                            if let Some(tray) = manager.tray_icon.borrow().as_ref() {
                                if let Ok(cmd) = tray.show_menu() {
                                    manager.handle_menu_command(cmd);
                                }
                            }
                        }
                        _ => {}
                    }
                    
                    // Don't drop the Arc, just leak it back
                    Arc::into_raw(manager);
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                let manager = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Self;
                if !manager.is_null() {
                    // Take ownership of the Arc to properly drop it
                    Arc::from_raw(manager);
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    
    fn handle_menu_command(&self, cmd: u32) {
        match cmd {
            2000 => {
                // Open KeyMagic
                if let Ok(callback) = self.on_show_window.lock() {
                    callback();
                }
            }
            2001 => {
                // Exit
                if let Ok(callback) = self.on_exit.lock() {
                    callback();
                }
            }
            1000..=1999 => {
                // Keyboard selection
                let index = (cmd - 1000) as usize;
                let keyboard_ids = self.keyboard_ids.borrow();
                if let Some(keyboard_id) = keyboard_ids.get(index) {
                    if let Ok(callback) = self.on_keyboard_selected.lock() {
                        callback(keyboard_id);
                    }
                }
            }
            _ => {}
        }
    }
}

// Safe wrapper for public API
unsafe impl Send for TrayManager {}
unsafe impl Sync for TrayManager {}