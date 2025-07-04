use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
        UI::Controls::*,
        UI::Controls::Dialogs::*,
    },
};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::ffi::c_void;
use crate::app::App;
use crate::keyboard_manager::KeyboardManager;
use crate::keyboard_list::KeyboardListView;
use crate::keyboard_preview::KeyboardPreview;

const WINDOW_CLASS_NAME: PCWSTR = w!("KeyMagicConfigWindow");
const WINDOW_TITLE: PCWSTR = w!("KeyMagic Configuration Manager");

// Menu command IDs
const ID_FILE_ADD_KEYBOARD: u16 = 101;
const ID_FILE_REMOVE_KEYBOARD: u16 = 102;
const ID_FILE_EXIT: u16 = 103;
const ID_KEYBOARD_ACTIVATE: u16 = 201;
const ID_KEYBOARD_CONFIGURE: u16 = 202;
const ID_HELP_ABOUT: u16 = 301;

// Custom window messages
const WM_UPDATE_OUTPUT: u32 = WM_USER + 1;

pub struct MainWindow {
    hwnd: HWND,
    app: Arc<App>,
    keyboard_manager: Arc<Mutex<KeyboardManager>>,
    list_view: RefCell<Option<KeyboardListView>>,
    preview: RefCell<Option<Arc<KeyboardPreview>>>,
}

impl MainWindow {
    pub fn new(app: &Arc<App>) -> Result<Arc<Self>> {
        unsafe {
            // Create keyboard manager
            let keyboard_manager = Arc::new(Mutex::new(KeyboardManager::new().map_err(|e| Error::new(HRESULT(-1), HSTRING::from(e.to_string())))?));
            
            // Register window class
            let instance = GetModuleHandleW(None)?;
            
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::window_proc),
                cbClsExtra: 0,
                cbWndExtra: std::mem::size_of::<*const MainWindow>() as i32,
                hInstance: instance.into(),
                hIcon: LoadIconW(None, IDI_APPLICATION)?,
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
            });
            
            // Store window pointer for WM_CREATE
            let window_ptr = Arc::as_ptr(&window) as *const c_void;
            
            // Create window
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                WINDOW_CLASS_NAME,
                WINDOW_TITLE,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                900,
                700,
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
                
                // Create the ListView
                if !window_ptr.is_null() {
                    let window = &*window_ptr;
                    let list_view = KeyboardListView::new(hwnd, window.keyboard_manager.clone());
                    if let Ok(lv) = list_view {
                        *window.list_view.borrow_mut() = Some(lv);
                    }
                    
                    // Create the preview area
                    let preview = KeyboardPreview::new(hwnd);
                    if let Ok(pv) = preview {
                        *window.preview.borrow_mut() = Some(pv);
                    }
                }
                
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
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
                        ID_FILE_EXIT => {
                            PostQuitMessage(0);
                        }
                        ID_KEYBOARD_ACTIVATE => {
                            window.activate_keyboard();
                        }
                        _ => {
                            // Check if it's from the preview area
                            if let Some(preview) = window.preview.borrow().as_ref() {
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
                    if let Some(_list_view) = window.list_view.borrow().as_ref() {
                        SetWindowPos(
                            HWND(SendMessageW(hwnd, WM_USER, WPARAM(1001), LPARAM(0)).0 as _),
                            None,
                            10,
                            10,
                            width - 20,
                            200,  // Fixed height for ListView
                            SWP_NOZORDER,
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
                }
            }
        }
    }
}