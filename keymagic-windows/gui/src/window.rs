use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
        UI::Controls::*,
        UI::Controls::Dialogs::*,
        UI::HiDpi::*,
    },
};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::ffi::c_void;
use crate::app::App;
use crate::keyboard_manager::KeyboardManager;
use crate::keyboard_list::KeyboardListView;
use crate::keyboard_preview::KeyboardPreview;
use crate::tray::{TrayIcon, tray_message};
use crate::tsf_status::TsfStatus;
use crate::{log_info, log_error};

const WINDOW_CLASS_NAME: PCWSTR = w!("KeyMagicConfigWindow");
const WINDOW_TITLE: PCWSTR = w!("KeyMagic Configuration Manager");

// Custom messages
const WM_HOTKEY_CHANGED: u32 = WM_USER + 200;

// Menu command IDs
const ID_FILE_ADD_KEYBOARD: u16 = 101;
const ID_FILE_REMOVE_KEYBOARD: u16 = 102;
const ID_FILE_EXIT: u16 = 103;
const ID_FILE_SETTINGS: u16 = 104;
const ID_KEYBOARD_ACTIVATE: u16 = 201;
const ID_KEYBOARD_CONFIGURE: u16 = 202;
const ID_HELP_ABOUT: u16 = 301;

// Custom window messages
const WM_UPDATE_OUTPUT: u32 = WM_USER + 1;

// Control IDs
const ID_STATUSBAR: u16 = 1100;

// Status bar constants
const SB_SETTEXT: u32 = WM_USER + 1;
const SBARS_SIZEGRIP: u32 = 0x0100;

pub struct MainWindow {
    hwnd: HWND,
    app: Arc<App>,
    keyboard_manager: Arc<Mutex<KeyboardManager>>,
    list_view: RefCell<Option<KeyboardListView>>,
    preview: RefCell<Option<Arc<KeyboardPreview>>>,
    tray_icon: RefCell<Option<TrayIcon>>,
    status_bar: RefCell<Option<HWND>>,
    dpi: RefCell<u32>,
}

impl MainWindow {
    fn scale_for_dpi(value: i32, dpi: u32) -> i32 {
        (value as f32 * dpi as f32 / 96.0) as i32
    }
    
    fn get_dpi_for_system() -> u32 {
        unsafe { GetDpiForSystem() }
    }
    
    pub fn new(app: &Arc<App>) -> Result<Arc<Self>> {
        unsafe {
            log_info!("MainWindow::new() started");
            
            // Get system DPI
            let system_dpi = Self::get_dpi_for_system();
            log_info!("System DPI: {}", system_dpi);
            
            // Create keyboard manager
            log_info!("Creating keyboard manager");
            let keyboard_manager = Arc::new(Mutex::new(
                KeyboardManager::new().map_err(|e| {
                    log_error!("Failed to create keyboard manager: {}", e);
                    Error::new(HRESULT(-1), HSTRING::from(e.to_string()))
                })?
            ));
            
            // Register window class
            log_info!("Getting module handle");
            let instance = GetModuleHandleW(None)?;
            
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::window_proc),
                cbClsExtra: 0,
                cbWndExtra: std::mem::size_of::<*const MainWindow>() as i32,
                hInstance: instance.into(),
                hIcon: {
                    log_info!("Loading window icon");
                    match LoadIconW(instance, PCWSTR(1 as *const u16)) {
                        Ok(icon) => icon,
                        Err(e) => {
                            log_error!("Failed to load icon, using default: {}", e);
                            LoadIconW(None, IDI_APPLICATION)?
                        }
                    }
                },
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as _),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: WINDOW_CLASS_NAME,
                hIconSm: HICON::default(),
            };
            
            let atom = RegisterClassExW(&wc);
            if atom == 0 {
                return Err(Error::from_win32());
            }
            
            // Create main window instance
            let window = Arc::new(MainWindow {
                hwnd: HWND::default(),
                app: app.clone(),
                keyboard_manager,
                list_view: RefCell::new(None),
                preview: RefCell::new(None),
                tray_icon: RefCell::new(None),
                status_bar: RefCell::new(None),
                dpi: RefCell::new(system_dpi),
            });
            
            // Store window pointer for WM_CREATE
            let window_ptr = Arc::as_ptr(&window) as *const c_void;
            
            // Create window with DPI-scaled dimensions
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                WINDOW_CLASS_NAME,
                WINDOW_TITLE,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                Self::scale_for_dpi(900, system_dpi),
                Self::scale_for_dpi(700, system_dpi),
                None,
                Self::create_menu()?,
                instance,
                Some(window_ptr),
            );
            
            if hwnd.0 == 0 {
                return Err(Error::from_win32());
            }
            
            // We need to update the hwnd after creation
            // Use a different approach - store window in user data and set hwnd during WM_CREATE
            
            Ok(window)
        }
    }
    
    pub fn show(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
            UpdateWindow(self.hwnd);
        }
    }
    
    unsafe fn create_menu() -> Result<HMENU> {
        let menu_bar = CreateMenu()?;
        
        // File menu
        let file_menu = CreatePopupMenu()?;
        AppendMenuW(file_menu, MF_STRING, ID_FILE_ADD_KEYBOARD as usize, w!("&Add Keyboard..."))?;
        AppendMenuW(file_menu, MF_STRING, ID_FILE_REMOVE_KEYBOARD as usize, w!("&Remove Keyboard"))?;
        AppendMenuW(file_menu, MF_SEPARATOR, 0, PCWSTR::null())?;
        AppendMenuW(file_menu, MF_STRING, ID_FILE_SETTINGS as usize, w!("&Settings..."))?;
        AppendMenuW(file_menu, MF_SEPARATOR, 0, PCWSTR::null())?;
        AppendMenuW(file_menu, MF_STRING, ID_FILE_EXIT as usize, w!("E&xit"))?;
        AppendMenuW(menu_bar, MF_POPUP, file_menu.0 as usize, w!("&File"))?;
        
        // Keyboard menu
        let keyboard_menu = CreatePopupMenu()?;
        AppendMenuW(keyboard_menu, MF_STRING, ID_KEYBOARD_ACTIVATE as usize, w!("&Activate"))?;
        AppendMenuW(keyboard_menu, MF_STRING, ID_KEYBOARD_CONFIGURE as usize, w!("&Configure..."))?;
        AppendMenuW(menu_bar, MF_POPUP, keyboard_menu.0 as usize, w!("&Keyboard"))?;
        
        // Help menu
        let help_menu = CreatePopupMenu()?;
        AppendMenuW(help_menu, MF_STRING, ID_HELP_ABOUT as usize, w!("&About..."))?;
        AppendMenuW(menu_bar, MF_POPUP, help_menu.0 as usize, w!("&Help"))?;
        
        Ok(menu_bar)
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
                let window_ptr = (*create_struct).lpCreateParams as *const MainWindow;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_ptr as isize);
                
                // Set the hwnd in the window struct
                if !window_ptr.is_null() {
                    let window = &*window_ptr;
                    // Need to cast away const to set hwnd - this is safe during window creation
                    let window_mut = window_ptr as *mut MainWindow;
                    (*window_mut).hwnd = hwnd;
                    let list_view = KeyboardListView::new(hwnd, window.keyboard_manager.clone(), *window.dpi.borrow());
                    if let Ok(lv) = list_view {
                        *window.list_view.borrow_mut() = Some(lv);
                    }
                    
                    // Create the preview area
                    let preview = KeyboardPreview::new(hwnd, *window.dpi.borrow());
                    if let Ok(pv) = preview {
                        *window.preview.borrow_mut() = Some(pv);
                    }
                    
                    // Create tray icon
                    if let Ok(tray) = TrayIcon::new(hwnd, window.keyboard_manager.clone()) {
                        *window.tray_icon.borrow_mut() = Some(tray);
                    }
                    
                    // Create status bar
                    let status_bar = CreateWindowExW(
                        WINDOW_EX_STYLE::default(),
                        w!("msctls_statusbar32"),
                        PCWSTR::null(),
                        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(SBARS_SIZEGRIP),
                        0, 0, 0, 0,
                        hwnd,
                        HMENU(ID_STATUSBAR as isize),
                        GetModuleHandleW(None).unwrap(),
                        None,
                    );
                    
                    if status_bar != HWND::default() {
                        *window.status_bar.borrow_mut() = Some(status_bar);
                        
                        // Set initial status
                        let status_msg = TsfStatus::get_status_message();
                        let status_w: Vec<u16> = status_msg.encode_utf16().chain(std::iter::once(0)).collect();
                        SendMessageW(status_bar, SB_SETTEXT, WPARAM(0), LPARAM(status_w.as_ptr() as isize));
                    }
                }
                
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_HOTKEY => {
                let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const MainWindow;
                if !window_ptr.is_null() {
                    let window = &*window_ptr;
                    let hotkey_id = wparam.0 as i32;
                    
                    log_info!("WM_HOTKEY received with id: {}", hotkey_id);
                    
                    if hotkey_id == 1 { // HOTKEY_ID_TOGGLE
                        log_info!("Toggling key processing enabled state");
                        if let Some(tray) = window.tray_icon.borrow().as_ref() {
                            match tray.toggle_key_processing_enabled() {
                                Ok(()) => log_info!("Key processing toggle successful"),
                                Err(e) => log_error!("Key processing toggle failed: {}", e),
                            }
                        } else {
                            log_error!("No tray icon available");
                        }
                    }
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const MainWindow;
                if !window_ptr.is_null() {
                    let window = &*window_ptr;
                    let cmd_id = (wparam.0 & 0xFFFF) as u16;
                    
                    match cmd_id {
                        ID_FILE_ADD_KEYBOARD => {
                            window.add_keyboard();
                        }
                        ID_FILE_REMOVE_KEYBOARD => {
                            window.remove_keyboard();
                        }
                        ID_FILE_SETTINGS => {
                            window.show_settings();
                        }
                        ID_FILE_EXIT => {
                            PostQuitMessage(0);
                        }
                        ID_KEYBOARD_ACTIVATE => {
                            window.activate_keyboard();
                        }
                        _ => {
                            // Check if it's from the tray icon
                            if let Some(tray) = window.tray_icon.borrow().as_ref() {
                                let _ = tray.handle_menu_command(cmd_id);
                            }
                            // Check if it's from the preview area
                            else if let Some(preview) = window.preview.borrow().as_ref() {
                                let _ = preview.handle_command(cmd_id);
                            }
                        }
                    }
                }
                LRESULT(0)
            }
            WM_NOTIFY => {
                let nmhdr = lparam.0 as *const NMHDR;
                if !nmhdr.is_null() && (*nmhdr).idFrom == 1001 {
                    let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const MainWindow;
                    if !window_ptr.is_null() {
                        let window = &*window_ptr;
                        if let Some(list_view) = window.list_view.borrow().as_ref() {
                            let _ = list_view.handle_command((*nmhdr).code as u16);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_SIZE => {
                // Resize ListView to fill client area
                let width = (lparam.0 & 0xFFFF) as i32;
                let _height = ((lparam.0 >> 16) & 0xFFFF) as i32;
                
                let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const MainWindow;
                if !window_ptr.is_null() {
                    let window = &*window_ptr;
                    let dpi = *window.dpi.borrow();
                    if let Some(_list_view) = window.list_view.borrow().as_ref() {
                        let margin = MainWindow::scale_for_dpi(10, dpi);
                        let list_height = MainWindow::scale_for_dpi(200, dpi);
                        SetWindowPos(
                            HWND(SendMessageW(hwnd, WM_USER, WPARAM(1001), LPARAM(0)).0 as _),
                            None,
                            margin,
                            margin,
                            width - margin * 2,
                            list_height,
                            SWP_NOZORDER,
                        );
                    }
                    
                    // Resize status bar
                    if let Some(status_bar) = window.status_bar.borrow().as_ref() {
                        SendMessageW(*status_bar, WM_SIZE, wparam, lparam);
                    }
                }
                LRESULT(0)
            }
            WM_DPICHANGED => {
                let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const MainWindow;
                if !window_ptr.is_null() {
                    let window = &*window_ptr;
                    let new_dpi = (wparam.0 & 0xFFFF) as u32;
                    *window.dpi.borrow_mut() = new_dpi;
                    
                    // Suggested new window size is in lparam
                    let suggested_rect = lparam.0 as *const RECT;
                    if !suggested_rect.is_null() {
                        let rect = &*suggested_rect;
                        SetWindowPos(
                            hwnd,
                            None,
                            rect.left,
                            rect.top,
                            rect.right - rect.left,
                            rect.bottom - rect.top,
                            SWP_NOZORDER | SWP_NOACTIVATE,
                        );
                    }
                }
                LRESULT(0)
            }
            WM_UPDATE_OUTPUT => {
                let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const MainWindow;
                if !window_ptr.is_null() {
                    let window = &*window_ptr;
                    if let Some(preview) = window.preview.borrow().as_ref() {
                        preview.update_output();
                    }
                }
                LRESULT(0)
            }
            WM_SYSCOMMAND => {
                // Handle minimize to tray
                if wparam.0 as u32 == SC_MINIMIZE {
                    ShowWindow(hwnd, SW_HIDE);
                    LRESULT(0)
                } else {
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
            }
            msg if msg == tray_message() => {
                let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const MainWindow;
                if !window_ptr.is_null() {
                    let window = &*window_ptr;
                    let icon_msg = lparam.0 as u32;
                    
                    match icon_msg {
                        WM_LBUTTONDBLCLK => {
                            // Double-click - show window
                            ShowWindow(hwnd, SW_SHOW);
                            SetForegroundWindow(hwnd);
                        }
                        WM_RBUTTONUP => {
                            // Right-click - show context menu
                            let mut pt = POINT::default();
                            GetCursorPos(&mut pt);
                            
                            if let Some(tray) = window.tray_icon.borrow().as_ref() {
                                let _ = tray.show_context_menu(pt.x, pt.y);
                            }
                        }
                        _ => {}
                    }
                }
                LRESULT(0)
            }
            WM_HOTKEY_CHANGED => {
                let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const MainWindow;
                if !window_ptr.is_null() {
                    let window = &*window_ptr;
                    
                    log_info!("Hotkey changed, re-registering...");
                    
                    // Re-register the hotkey with the tray icon
                    if let Some(tray) = window.tray_icon.borrow_mut().as_mut() {
                        match tray.reregister_hotkey() {
                            Ok(()) => log_info!("Hotkey re-registered successfully"),
                            Err(e) => log_error!("Failed to re-register hotkey: {}", e),
                        }
                    }
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    
    fn add_keyboard(&self) {
        unsafe {
            // Use a simple file open dialog approach
            let filter = "KeyMagic Keyboard Files (*.km2)\0*.km2\0All Files (*.*)\0*.*\0\0";
            let filter_w: Vec<u16> = filter.encode_utf16().collect();
            let mut file_path_buffer = vec![0u16; 260];
            
            let mut ofn = std::mem::zeroed::<OPENFILENAMEW>();
            ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
            ofn.hwndOwner = self.hwnd;
            ofn.lpstrFilter = PCWSTR(filter_w.as_ptr());
            ofn.lpstrFile = PWSTR(file_path_buffer.as_mut_ptr());
            ofn.nMaxFile = 260;
            ofn.lpstrTitle = w!("Select KeyMagic Keyboard");
            ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY;
            
            if GetOpenFileNameW(&mut ofn).as_bool() {
                let path = std::path::PathBuf::from(ofn.lpstrFile.to_string().unwrap());
                let mut manager = self.keyboard_manager.lock().unwrap();
                
                match manager.load_keyboard(&path) {
                    Ok(_) => {
                        drop(manager);
                        if let Some(list_view) = self.list_view.borrow().as_ref() {
                            let _ = list_view.refresh();
                        }
                        self.update_tray_tooltip();
                    }
                    Err(e) => {
                        let msg = format!("Failed to load keyboard: {}", e);
                        let msg_w: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
                        MessageBoxW(
                            self.hwnd,
                            PCWSTR(msg_w.as_ptr()),
                            w!("Error"),
                            MB_OK | MB_ICONERROR,
                        );
                    }
                }
            }
        }
    }
    
    fn remove_keyboard(&self) {
        if let Some(list_view) = self.list_view.borrow().as_ref() {
            if let Some(id) = list_view.get_selected_keyboard_id() {
                unsafe {
                    let result = MessageBoxW(
                        self.hwnd,
                        w!("Are you sure you want to remove this keyboard?"),
                        w!("Confirm Remove"),
                        MB_YESNO | MB_ICONQUESTION,
                    );
                    
                    if result == MESSAGEBOX_RESULT(6) { // IDYES
                        let mut manager = self.keyboard_manager.lock().unwrap();
                        if manager.remove_keyboard(&id).is_ok() {
                            drop(manager);
                            let _ = list_view.refresh();
                            self.update_tray_tooltip();
                        }
                    }
                }
            }
        }
    }
    
    fn activate_keyboard(&self) {
        if let Some(list_view) = self.list_view.borrow().as_ref() {
            if let Some(id) = list_view.get_selected_keyboard_id() {
                let mut manager = self.keyboard_manager.lock().unwrap();
                if manager.set_active_keyboard(&id).is_ok() {
                    // Load keyboard in preview if available
                    if let Some(keyboard_info) = manager.get_keyboard(&id) {
                        if let Some(preview) = self.preview.borrow().as_ref() {
                            let _ = preview.load_keyboard(keyboard_info.path.to_str().unwrap_or(""));
                        }
                    }
                    drop(manager);
                    let _ = list_view.refresh();
                    self.update_tray_tooltip();
                }
            }
        }
    }
    
    fn update_tray_tooltip(&self) {
        let manager = self.keyboard_manager.lock().unwrap();
        let tooltip = if let Some(active_id) = manager.get_active_keyboard() {
            if let Some(keyboard) = manager.get_keyboard(&active_id) {
                format!("KeyMagic - {}", keyboard.name)
            } else {
                "KeyMagic - No active keyboard".to_string()
            }
        } else {
            "KeyMagic - No active keyboard".to_string()
        };
        
        drop(manager);
        
        if let Some(tray) = self.tray_icon.borrow().as_ref() {
            let _ = tray.update_tooltip(&tooltip);
        }
    }
    
    fn show_settings(&self) {
        let _ = crate::settings::SettingsDialog::show(self.hwnd, self.keyboard_manager.clone());
    }
}