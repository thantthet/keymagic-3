use windows::{
    core::*,
    Win32::{
        Foundation::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
        UI::Controls::*,
    },
};
use std::sync::{Arc, Mutex};
use crate::keyboard_manager_simple::{KeyboardManager, KeyboardInfo};

const IDC_KEYBOARD_LIST: i32 = 1001;

pub struct KeyboardListView {
    hwnd: HWND,
    keyboard_manager: Arc<Mutex<KeyboardManager>>,
}

impl KeyboardListView {
    pub fn new(parent: HWND, keyboard_manager: Arc<Mutex<KeyboardManager>>) -> Result<Self> {
        unsafe {
            // Initialize common controls
            let icc = INITCOMMONCONTROLSEX {
                dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_LISTVIEW_CLASSES,
            };
            InitCommonControlsEx(&icc);
            
            // Create ListView
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                WC_LISTVIEW,
                w!(""),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS),
                10,
                10,
                760,
                400,
                parent,
                HMENU(IDC_KEYBOARD_LIST as _),
                GetModuleHandleW(None)?,
                None,
            );
            
            if hwnd.0 == 0 {
                return Err(Error::from_win32());
            }
            
            // Set extended styles
            SendMessageW(hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, 
                WPARAM(0), 
                LPARAM((LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES | LVS_EX_DOUBLEBUFFER) as isize));
            
            // Add columns
            // Column 1: Name
            let name_text: Vec<u16> = "Name".encode_utf16().chain(std::iter::once(0)).collect();
            let mut lvc = LVCOLUMNW {
                mask: LVCF_TEXT | LVCF_WIDTH | LVCF_SUBITEM,
                cx: 200,
                pszText: PWSTR(name_text.as_ptr() as *mut _),
                iSubItem: 0,
                ..Default::default()
            };
            SendMessageW(hwnd, LVM_INSERTCOLUMNW, WPARAM(0), LPARAM(&lvc as *const _ as _));
            
            // Column 2: Description
            let desc_text: Vec<u16> = "Description".encode_utf16().chain(std::iter::once(0)).collect();
            lvc.cx = 300;
            lvc.pszText = PWSTR(desc_text.as_ptr() as *mut _);
            lvc.iSubItem = 1;
            SendMessageW(hwnd, LVM_INSERTCOLUMNW, WPARAM(1), LPARAM(&lvc as *const _ as _));
            
            // Column 3: Hotkey
            let hotkey_text: Vec<u16> = "Hotkey".encode_utf16().chain(std::iter::once(0)).collect();
            lvc.cx = 150;
            lvc.pszText = PWSTR(hotkey_text.as_ptr() as *mut _);
            lvc.iSubItem = 2;
            SendMessageW(hwnd, LVM_INSERTCOLUMNW, WPARAM(2), LPARAM(&lvc as *const _ as _));
            
            // Column 4: Status
            let status_text: Vec<u16> = "Status".encode_utf16().chain(std::iter::once(0)).collect();
            lvc.cx = 100;
            lvc.pszText = PWSTR(status_text.as_ptr() as *mut _);
            lvc.iSubItem = 3;
            SendMessageW(hwnd, LVM_INSERTCOLUMNW, WPARAM(3), LPARAM(&lvc as *const _ as _));
            
            let list_view = Self {
                hwnd,
                keyboard_manager: keyboard_manager.clone(),
            };
            
            // Populate the list
            list_view.refresh()?;
            
            Ok(list_view)
        }
    }
    
    pub fn refresh(&self) -> Result<()> {
        unsafe {
            // Clear existing items
            SendMessageW(self.hwnd, LVM_DELETEALLITEMS, WPARAM(0), LPARAM(0));
            
            // Add keyboards
            let manager = self.keyboard_manager.lock().unwrap();
            let keyboards = manager.get_keyboards();
            let active_id = manager.get_active_keyboard();
            
            for (index, keyboard) in keyboards.iter().enumerate() {
                self.add_keyboard_item(index as i32, keyboard, active_id)?;
            }
        }
        
        Ok(())
    }
    
    unsafe fn add_keyboard_item(&self, index: i32, keyboard: &KeyboardInfo, active_id: Option<&str>) -> Result<()> {
        // Insert item
        let name_w: Vec<u16> = keyboard.name.encode_utf16().chain(std::iter::once(0)).collect();
        let mut lvi = LVITEMW {
            mask: LVIF_TEXT | LVIF_PARAM,
            iItem: index,
            iSubItem: 0,
            pszText: PWSTR(name_w.as_ptr() as *mut _),
            lParam: LPARAM(keyboard as *const _ as isize),
            ..Default::default()
        };
        
        let item_index = SendMessageW(self.hwnd, LVM_INSERTITEMW, WPARAM(0), LPARAM(&lvi as *const _ as _));
        
        // Set subitems
        let desc_w: Vec<u16> = keyboard.description.encode_utf16().chain(std::iter::once(0)).collect();
        lvi.mask = LVIF_TEXT;
        lvi.iItem = item_index.0 as i32;
        lvi.iSubItem = 1;
        lvi.pszText = PWSTR(desc_w.as_ptr() as *mut _);
        SendMessageW(self.hwnd, LVM_SETITEMW, WPARAM(0), LPARAM(&lvi as *const _ as _));
        
        let hotkey_w: Vec<u16> = keyboard.hotkey.as_deref().unwrap_or("")
            .encode_utf16().chain(std::iter::once(0)).collect();
        lvi.iSubItem = 2;
        lvi.pszText = PWSTR(hotkey_w.as_ptr() as *mut _);
        SendMessageW(self.hwnd, LVM_SETITEMW, WPARAM(0), LPARAM(&lvi as *const _ as _));
        
        let status = if active_id == Some(&keyboard.id) {
            "Active"
        } else if keyboard.enabled {
            "Enabled"
        } else {
            "Disabled"
        };
        let status_w: Vec<u16> = status.encode_utf16().chain(std::iter::once(0)).collect();
        lvi.iSubItem = 3;
        lvi.pszText = PWSTR(status_w.as_ptr() as *mut _);
        SendMessageW(self.hwnd, LVM_SETITEMW, WPARAM(0), LPARAM(&lvi as *const _ as _));
        
        Ok(())
    }
    
    pub fn get_selected_keyboard_id(&self) -> Option<String> {
        unsafe {
            let selected = SendMessageW(self.hwnd, LVM_GETNEXTITEM, WPARAM(-1i32 as usize), LPARAM(LVNI_SELECTED as isize));
            if selected.0 >= 0 {
                // Get the keyboard ID from the item
                let mut buffer = vec![0u16; 256];
                let lvi = LVITEMW {
                    mask: LVIF_TEXT,
                    iItem: selected.0 as i32,
                    iSubItem: 0,
                    pszText: PWSTR(buffer.as_mut_ptr()),
                    cchTextMax: buffer.len() as i32,
                    ..Default::default()
                };
                
                if SendMessageW(self.hwnd, LVM_GETITEMW, WPARAM(0), LPARAM(&lvi as *const _ as _)).0 != 0 {
                    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
                    let name = String::from_utf16_lossy(&buffer[..len]);
                    
                    // Find keyboard by name
                    let manager = self.keyboard_manager.lock().unwrap();
                    for kb in manager.get_keyboards() {
                        if kb.name == name {
                            return Some(kb.id.clone());
                        }
                    }
                }
            }
        }
        
        None
    }
    
    pub fn handle_command(&self, code: u16) -> Result<()> {
        match code as u32 {
            NM_DBLCLK => {
                // Double-click to activate keyboard
                if let Some(id) = self.get_selected_keyboard_id() {
                    let mut manager = self.keyboard_manager.lock().unwrap();
                    let _ = manager.set_active_keyboard(&id);
                    drop(manager);
                    self.refresh()?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }
}