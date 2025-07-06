use windows::{
    core::*,
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        System::LibraryLoader::GetModuleHandleW,
        Graphics::Gdi::*,
    },
};

// Button state constants
const BST_UNCHECKED: i32 = 0;
const BST_CHECKED: i32 = 1;

// Custom messages
const WM_HOTKEY_CHANGED: u32 = WM_USER + 200;

use std::sync::{Arc, Mutex};
use std::ffi::c_void;
use crate::keyboard_manager::KeyboardManager;

const IDC_CHECK_CTRL: u16 = 2001;
const IDC_CHECK_ALT: u16 = 2002;
const IDC_CHECK_SHIFT: u16 = 2003;
const IDC_CHECK_WIN: u16 = 2004;
const IDC_KEY_EDIT: u16 = 2005;
const IDC_OK: u16 = 2006;
const IDC_CANCEL: u16 = 2007;
const IDC_DEFAULT: u16 = 2008;
const IDC_CURRENT_HOTKEY: u16 = 2009;

pub struct SettingsDialog {
    keyboard_manager: Arc<Mutex<KeyboardManager>>,
    hwnd: HWND,
    ctrl_check: HWND,
    alt_check: HWND,
    shift_check: HWND,
    win_check: HWND,
    key_edit: HWND,
    current_hotkey_label: HWND,
    parent_hwnd: HWND,
}

impl SettingsDialog {
    pub fn show(parent: HWND, keyboard_manager: Arc<Mutex<KeyboardManager>>) -> Result<()> {
        unsafe {
            // Register window class
            let class_name = w!("KeyMagicSettingsDialog");
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::dialog_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleW(None)?.into(),
                hIcon: HICON::default(),
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hbrBackground: HBRUSH((COLOR_BTNFACE.0 + 1) as isize),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: class_name,
                hIconSm: HICON::default(),
            };
            
            RegisterClassExW(&wc);
            
            // Create dialog struct first
            let mut dialog = Box::new(SettingsDialog {
                keyboard_manager,
                hwnd: HWND::default(),
                ctrl_check: HWND::default(),
                alt_check: HWND::default(),
                shift_check: HWND::default(),
                win_check: HWND::default(),
                key_edit: HWND::default(),
                current_hotkey_label: HWND::default(),
                parent_hwnd: parent,
            });
            
            let dialog_ptr = dialog.as_mut() as *mut SettingsDialog;
            
            // Create dialog window with dialog pointer
            let hwnd = CreateWindowExW(
                WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE,
                class_name,
                w!("KeyMagic Settings"),
                WS_POPUP | WS_CAPTION | WS_SYSMENU,
                CW_USEDEFAULT, CW_USEDEFAULT, 500, 400,
                parent,
                None,
                GetModuleHandleW(None)?,
                Some(dialog_ptr as *mut c_void),
            );
            
            if hwnd == HWND::default() {
                return Err(Error::from_win32());
            }
            
            // Don't drop the box, it will be freed in WM_CLOSE
            std::mem::forget(dialog);
            
            // Center window
            let mut rect = RECT::default();
            GetWindowRect(hwnd, &mut rect);
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            
            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);
            
            SetWindowPos(
                hwnd,
                HWND_TOP,
                (screen_width - width) / 2,
                (screen_height - height) / 2,
                0, 0,
                SWP_NOSIZE | SWP_NOZORDER,
            );
            
            ShowWindow(hwnd, SW_SHOW);
            
            // Message loop
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                if !IsDialogMessageW(hwnd, &msg).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
        
        Ok(())
    }
    
    unsafe fn create_controls(&mut self) {
        let hfont = HFONT(GetStockObject(DEFAULT_GUI_FONT).0);
        
        // Current hotkey label
        let current_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("Current Hotkey:"),
            WS_CHILD | WS_VISIBLE,
            20, 20, 120, 25,
            self.hwnd,
            None,
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(current_label, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // Load current hotkey
        let current_hotkey = self.keyboard_manager.lock().unwrap()
            .read_registry_value("Settings\\ToggleHotkey")
            .unwrap_or_else(|| "Ctrl+Shift+Space".to_string());
        
        self.current_hotkey_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!(""),
            WS_CHILD | WS_VISIBLE,
            150, 20, 320, 25,
            self.hwnd,
            HMENU(IDC_CURRENT_HOTKEY as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(self.current_hotkey_label, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        let hotkey_wide: Vec<u16> = current_hotkey.encode_utf16().chain(std::iter::once(0)).collect();
        SetWindowTextW(self.current_hotkey_label, PCWSTR(hotkey_wide.as_ptr()));
        
        // New hotkey section label
        let new_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("New Hotkey:"),
            WS_CHILD | WS_VISIBLE,
            20, 65, 120, 25,
            self.hwnd,
            None,
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(new_label, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // Modifier checkboxes
        self.ctrl_check = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("Ctrl"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            20, 100, 70, 25,
            self.hwnd,
            HMENU(IDC_CHECK_CTRL as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(self.ctrl_check, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        self.alt_check = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("Alt"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            100, 100, 70, 25,
            self.hwnd,
            HMENU(IDC_CHECK_ALT as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(self.alt_check, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        self.shift_check = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("Shift"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            180, 100, 70, 25,
            self.hwnd,
            HMENU(IDC_CHECK_SHIFT as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(self.shift_check, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        self.win_check = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("Win"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            260, 100, 70, 25,
            self.hwnd,
            HMENU(IDC_CHECK_WIN as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(self.win_check, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // Plus label
        let plus_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("+"),
            WS_CHILD | WS_VISIBLE,
            345, 100, 20, 25,
            self.hwnd,
            None,
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(plus_label, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // Key edit control
        self.key_edit = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w!("EDIT"),
            w!(""),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
            370, 100, 100, 28,
            self.hwnd,
            HMENU(IDC_KEY_EDIT as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(self.key_edit, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // Instructions
        let instructions = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("Examples: Space, Tab, Enter, Esc, A-Z, 0-9, F1-F12, Home, End, Delete"),
            WS_CHILD | WS_VISIBLE,
            20, 145, 450, 25,
            self.hwnd,
            None,
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(instructions, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // Default button
        let default_btn = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("Restore Default"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            20, 190, 140, 30,
            self.hwnd,
            HMENU(IDC_DEFAULT as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(default_btn, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // OK button
        let ok_btn = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("OK"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
            280, 240, 90, 30,
            self.hwnd,
            HMENU(IDC_OK as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(ok_btn, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // Cancel button
        let cancel_btn = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("Cancel"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            380, 240, 90, 30,
            self.hwnd,
            HMENU(IDC_CANCEL as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(cancel_btn, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // Parse current hotkey and set controls
        self.parse_and_set_hotkey(&current_hotkey);
    }
    
    unsafe fn handle_command(&mut self, cmd: u16) {
        match cmd {
            IDC_DEFAULT => {
                // Set default hotkey
                self.parse_and_set_hotkey("Ctrl+Shift+Space");
            }
            IDC_OK => {
                // Build hotkey string from controls
                let hotkey = self.build_hotkey_string();
                
                if hotkey.is_empty() {
                    MessageBoxW(
                        self.hwnd,
                        w!("Please specify at least one key"),
                        w!("Invalid Hotkey"),
                        MB_OK | MB_ICONWARNING,
                    );
                    return;
                }
                
                // Save hotkey
                let _ = self.keyboard_manager.lock().unwrap()
                    .write_registry_value("Settings\\ToggleHotkey", &hotkey);
                
                // Notify parent window that hotkey has changed
                PostMessageW(self.parent_hwnd, WM_HOTKEY_CHANGED, WPARAM(0), LPARAM(0));
                    
                PostMessageW(self.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
            }
            IDC_CANCEL => {
                PostMessageW(self.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
            }
            _ => {}
        }
    }
    
    unsafe fn parse_and_set_hotkey(&mut self, hotkey: &str) {
        // Reset all checkboxes
        SendMessageW(self.ctrl_check, BM_SETCHECK, WPARAM(BST_UNCHECKED as usize), LPARAM(0));
        SendMessageW(self.alt_check, BM_SETCHECK, WPARAM(BST_UNCHECKED as usize), LPARAM(0));
        SendMessageW(self.shift_check, BM_SETCHECK, WPARAM(BST_UNCHECKED as usize), LPARAM(0));
        SendMessageW(self.win_check, BM_SETCHECK, WPARAM(BST_UNCHECKED as usize), LPARAM(0));
        
        // Clear key edit
        SetWindowTextW(self.key_edit, w!(""));
        
        // Parse hotkey string
        let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();
        let mut key_part = "";
        
        for part in &parts {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => {
                    SendMessageW(self.ctrl_check, BM_SETCHECK, WPARAM(BST_CHECKED as usize), LPARAM(0));
                }
                "alt" | "menu" => {
                    SendMessageW(self.alt_check, BM_SETCHECK, WPARAM(BST_CHECKED as usize), LPARAM(0));
                }
                "shift" => {
                    SendMessageW(self.shift_check, BM_SETCHECK, WPARAM(BST_CHECKED as usize), LPARAM(0));
                }
                "win" | "windows" | "super" => {
                    SendMessageW(self.win_check, BM_SETCHECK, WPARAM(BST_CHECKED as usize), LPARAM(0));
                }
                _ => {
                    key_part = part;
                }
            }
        }
        
        // Set the key part
        if !key_part.is_empty() {
            let key_wide: Vec<u16> = key_part.encode_utf16().chain(std::iter::once(0)).collect();
            SetWindowTextW(self.key_edit, PCWSTR(key_wide.as_ptr()));
        }
    }
    
    unsafe fn build_hotkey_string(&self) -> String {
        let mut parts = Vec::new();
        
        // Check modifiers
        if SendMessageW(self.ctrl_check, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == BST_CHECKED as isize {
            parts.push("Ctrl".to_string());
        }
        if SendMessageW(self.alt_check, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == BST_CHECKED as isize {
            parts.push("Alt".to_string());
        }
        if SendMessageW(self.shift_check, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == BST_CHECKED as isize {
            parts.push("Shift".to_string());
        }
        if SendMessageW(self.win_check, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == BST_CHECKED as isize {
            parts.push("Win".to_string());
        }
        
        // Get key text
        let mut buffer = vec![0u16; 256];
        let len = GetWindowTextW(self.key_edit, &mut buffer);
        if len > 0 {
            let key = String::from_utf16_lossy(&buffer[..len as usize]).trim().to_string();
            if !key.is_empty() {
                parts.push(key);
            }
        }
        
        parts.join("+")
    }
    
    unsafe extern "system" fn dialog_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                let create_struct = lparam.0 as *const CREATESTRUCTW;
                if !create_struct.is_null() {
                    let dialog_ptr = (*create_struct).lpCreateParams as *mut SettingsDialog;
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, dialog_ptr as isize);
                    
                    if !dialog_ptr.is_null() {
                        let dialog = &mut *dialog_ptr;
                        dialog.hwnd = hwnd;
                        dialog.create_controls();
                    }
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                let dialog_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsDialog;
                if !dialog_ptr.is_null() {
                    let dialog = &mut *dialog_ptr;
                    let cmd = (wparam.0 & 0xFFFF) as u16;
                    dialog.handle_command(cmd);
                }
                LRESULT(0)
            }
            WM_CLOSE => {
                let dialog_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsDialog;
                if !dialog_ptr.is_null() {
                    let _ = Box::from_raw(dialog_ptr);
                }
                DestroyWindow(hwnd);
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}