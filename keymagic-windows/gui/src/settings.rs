use windows::{
    core::*,
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        System::LibraryLoader::GetModuleHandleW,
        Graphics::Gdi::*,
    },
};
use std::sync::{Arc, Mutex};
use std::ffi::c_void;
use crate::keyboard_manager::KeyboardManager;

const IDC_HOTKEY_EDIT: u16 = 2001;
const IDC_HOTKEY_SET: u16 = 2002;
const IDC_OK: u16 = 2003;
const IDC_CANCEL: u16 = 2004;
const IDC_DEFAULT: u16 = 2005;

pub struct SettingsDialog {
    keyboard_manager: Arc<Mutex<KeyboardManager>>,
    hwnd: HWND,
    hotkey_edit: HWND,
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
                hotkey_edit: HWND::default(),
            });
            
            let dialog_ptr = dialog.as_mut() as *mut SettingsDialog;
            
            // Create dialog window with dialog pointer
            let hwnd = CreateWindowExW(
                WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE,
                class_name,
                w!("KeyMagic Settings"),
                WS_POPUP | WS_CAPTION | WS_SYSMENU,
                CW_USEDEFAULT, CW_USEDEFAULT, 400, 200,
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
        
        // Label
        let label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("Toggle Hotkey:"),
            WS_CHILD | WS_VISIBLE,
            20, 20, 100, 20,
            self.hwnd,
            None,
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(label, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // Hotkey edit control
        self.hotkey_edit = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w!("EDIT"),
            w!(""),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE((ES_AUTOHSCROLL | ES_READONLY) as u32),
            130, 20, 200, 24,
            self.hwnd,
            HMENU(IDC_HOTKEY_EDIT as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(self.hotkey_edit, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // Load current hotkey
        let current_hotkey = self.keyboard_manager.lock().unwrap()
            .read_registry_value("Settings\\ToggleHotkey")
            .unwrap_or_else(|| "Ctrl+Shift+Space".to_string());
        
        let hotkey_wide: Vec<u16> = current_hotkey.encode_utf16().chain(std::iter::once(0)).collect();
        SetWindowTextW(self.hotkey_edit, PCWSTR(hotkey_wide.as_ptr()));
        
        // Set button
        let set_btn = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("Set"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            340, 20, 40, 24,
            self.hwnd,
            HMENU(IDC_HOTKEY_SET as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(set_btn, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
        
        // Instructions
        let instructions = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("Click 'Set' and press your desired key combination"),
            WS_CHILD | WS_VISIBLE,
            20, 55, 360, 20,
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
            20, 100, 120, 25,
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
            210, 140, 80, 25,
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
            300, 140, 80, 25,
            self.hwnd,
            HMENU(IDC_CANCEL as isize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(cancel_btn, WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1));
    }
    
    unsafe fn handle_command(&mut self, cmd: u16) {
        match cmd {
            IDC_HOTKEY_SET => {
                // Start hotkey capture
                MessageBoxW(
                    self.hwnd,
                    w!("Press your desired hotkey combination now..."),
                    w!("Set Hotkey"),
                    MB_OK | MB_ICONINFORMATION,
                );
                // TODO: Implement actual hotkey capture
            }
            IDC_DEFAULT => {
                // Set default hotkey
                let default_hotkey = w!("Ctrl+Shift+Space");
                SetWindowTextW(self.hotkey_edit, default_hotkey);
            }
            IDC_OK => {
                // Save hotkey
                let mut buffer = vec![0u16; 256];
                let len = GetWindowTextW(self.hotkey_edit, &mut buffer);
                if len > 0 {
                    let hotkey = String::from_utf16_lossy(&buffer[..len as usize]);
                    let _ = self.keyboard_manager.lock().unwrap()
                        .write_registry_value("Settings\\ToggleHotkey", &hotkey);
                }
                PostMessageW(self.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
            }
            IDC_CANCEL => {
                PostMessageW(self.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
            }
            _ => {}
        }
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