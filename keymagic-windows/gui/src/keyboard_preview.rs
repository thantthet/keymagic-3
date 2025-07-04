use windows::{
    core::*,
    Win32::{
        Foundation::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
        Graphics::Gdi::*,
        UI::Input::KeyboardAndMouse::*,
    },
};
use std::sync::{Arc, Mutex};
use std::ffi::CString;
use keymagic_core::ffi::*;
use anyhow::Result;

const IDC_TEST_INPUT: i32 = 2001;
const IDC_TEST_OUTPUT: i32 = 2002;
const IDC_CLEAR_BUTTON: i32 = 2003;
const IDC_COMPOSING_LABEL: i32 = 2004;

pub struct KeyboardPreview {
    hwnd: HWND,
    input_hwnd: HWND,
    output_hwnd: HWND,
    composing_hwnd: HWND,
    engine: Arc<Mutex<Option<*mut EngineHandle>>>,
    original_wndproc: isize,
}

impl KeyboardPreview {
    fn scale_for_dpi(value: i32, dpi: u32) -> i32 {
        (value as f32 * dpi as f32 / 96.0) as i32
    }
    
    pub fn new(parent: HWND, dpi: u32) -> Result<Arc<Self>> {
        unsafe {
            let instance = GetModuleHandleW(None)?;
            
            // Create container static control with DPI scaling
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!(""),
                WS_CHILD | WS_VISIBLE,
                Self::scale_for_dpi(10, dpi),
                Self::scale_for_dpi(250, dpi),
                Self::scale_for_dpi(860, dpi),
                Self::scale_for_dpi(350, dpi),
                parent,
                None,
                instance,
                None,
            );
            
            if hwnd.0 == 0 {
                return Err(Error::from_win32().into());
            }
            
            // Create groupbox
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("Keyboard Test Area"),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_GROUPBOX as u32),
                0,
                0,
                Self::scale_for_dpi(860, dpi),
                Self::scale_for_dpi(350, dpi),
                hwnd,
                None,
                instance,
                None,
            );
            
            // Create labels
            let label1 = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("Type here to test keyboard (press keys to see real-time processing):"),
                WS_CHILD | WS_VISIBLE,
                Self::scale_for_dpi(10, dpi),
                Self::scale_for_dpi(30, dpi),
                Self::scale_for_dpi(600, dpi),  // Increased width for better text display
                Self::scale_for_dpi(20, dpi),
                hwnd,
                None,
                instance,
                None,
            );
            
            let label2 = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("Composing:"),
                WS_CHILD | WS_VISIBLE,
                Self::scale_for_dpi(10, dpi),
                Self::scale_for_dpi(90, dpi),
                Self::scale_for_dpi(100, dpi),
                Self::scale_for_dpi(20, dpi),
                hwnd,
                None,
                instance,
                None,
            );
            
            let label3 = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("Output:"),
                WS_CHILD | WS_VISIBLE,
                Self::scale_for_dpi(10, dpi),
                Self::scale_for_dpi(150, dpi),
                Self::scale_for_dpi(100, dpi),
                Self::scale_for_dpi(20, dpi),
                hwnd,
                None,
                instance,
                None,
            );
            
            // Create test input edit control
            let input_hwnd = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                w!("EDIT"),
                w!(""),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE((ES_MULTILINE | ES_AUTOVSCROLL) as u32) | WS_VSCROLL,
                Self::scale_for_dpi(10, dpi),
                Self::scale_for_dpi(50, dpi),
                Self::scale_for_dpi(840, dpi),
                Self::scale_for_dpi(40, dpi),  // Increased height
                hwnd,
                HMENU(IDC_TEST_INPUT as _),
                instance,
                None,
            );
            
            // Create composing display (shows current composing text)
            let composing_hwnd = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                w!("EDIT"),
                w!(""),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE((ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL) as u32) | WS_VSCROLL,
                Self::scale_for_dpi(10, dpi),
                Self::scale_for_dpi(110, dpi),
                Self::scale_for_dpi(840, dpi),
                Self::scale_for_dpi(40, dpi),  // Increased height
                hwnd,
                HMENU(IDC_COMPOSING_LABEL as _),
                instance,
                None,
            );
            
            // Create output display (read-only)
            let output_hwnd = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                w!("EDIT"),
                w!(""),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE((ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL) as u32) | WS_VSCROLL,
                Self::scale_for_dpi(10, dpi),
                Self::scale_for_dpi(170, dpi),
                Self::scale_for_dpi(840, dpi),
                Self::scale_for_dpi(120, dpi),  // Increased height
                hwnd,
                HMENU(IDC_TEST_OUTPUT as _),
                instance,
                None,
            );
            
            // Create clear button
            let clear_button = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("Clear"),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                Self::scale_for_dpi(10, dpi),
                Self::scale_for_dpi(280, dpi),
                Self::scale_for_dpi(80, dpi),
                Self::scale_for_dpi(30, dpi),
                hwnd,
                HMENU(IDC_CLEAR_BUTTON as _),
                instance,
                None,
            );
            
            // Create note
            let note_label = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("Note: This preview shows how the engine processes each key. Composing text shows the engine's internal state."),
                WS_CHILD | WS_VISIBLE,
                Self::scale_for_dpi(100, dpi),
                Self::scale_for_dpi(285, dpi),
                Self::scale_for_dpi(700, dpi),
                Self::scale_for_dpi(20, dpi),
                hwnd,
                None,
                instance,
                None,
            );
            
            // Create info label
            let info_label = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("Press Space/Enter to commit, Escape to cancel composition. Backspace works with smart rules."),
                WS_CHILD | WS_VISIBLE,
                Self::scale_for_dpi(100, dpi),
                Self::scale_for_dpi(305, dpi),
                Self::scale_for_dpi(700, dpi),
                Self::scale_for_dpi(20, dpi),
                hwnd,
                None,
                instance,
                None,
            );
            
            // Set fonts with DPI scaling
            let edit_font_size = Self::scale_for_dpi(14, dpi);  // Font for edit controls
            let label_font_size = Self::scale_for_dpi(11, dpi); // Font for labels
            
            let edit_font = CreateFontW(
                edit_font_size,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                FALSE.0 as u32,
                FALSE.0 as u32,
                FALSE.0 as u32,
                DEFAULT_CHARSET.0 as u32,
                OUT_DEFAULT_PRECIS.0 as u32,
                CLIP_DEFAULT_PRECIS.0 as u32,
                CLEARTYPE_QUALITY.0 as u32,
                (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
                w!("Segoe UI"),
            );
            
            let label_font = CreateFontW(
                label_font_size,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                FALSE.0 as u32,
                FALSE.0 as u32,
                FALSE.0 as u32,
                DEFAULT_CHARSET.0 as u32,
                OUT_DEFAULT_PRECIS.0 as u32,
                CLIP_DEFAULT_PRECIS.0 as u32,
                CLEARTYPE_QUALITY.0 as u32,
                (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
                w!("Segoe UI"),
            );
            
            // Apply fonts to controls
            if label_font.0 != 0 {
                let _ = SendMessageW(label1, WM_SETFONT, WPARAM(label_font.0 as _), LPARAM(1));
                let _ = SendMessageW(label2, WM_SETFONT, WPARAM(label_font.0 as _), LPARAM(1));
                let _ = SendMessageW(label3, WM_SETFONT, WPARAM(label_font.0 as _), LPARAM(1));
                let _ = SendMessageW(clear_button, WM_SETFONT, WPARAM(label_font.0 as _), LPARAM(1));
                let _ = SendMessageW(note_label, WM_SETFONT, WPARAM(label_font.0 as _), LPARAM(1));
                let _ = SendMessageW(info_label, WM_SETFONT, WPARAM(label_font.0 as _), LPARAM(1));
            }
            
            if edit_font.0 != 0 {
                let _ = SendMessageW(input_hwnd, WM_SETFONT, WPARAM(edit_font.0 as _), LPARAM(1));
                let _ = SendMessageW(output_hwnd, WM_SETFONT, WPARAM(edit_font.0 as _), LPARAM(1));
                let _ = SendMessageW(composing_hwnd, WM_SETFONT, WPARAM(edit_font.0 as _), LPARAM(1));
            }
            
            let preview = Arc::new(Self {
                hwnd,
                input_hwnd,
                output_hwnd,
                composing_hwnd,
                engine: Arc::new(Mutex::new(None)),
                original_wndproc: 0,
            });
            
            // Store preview instance and subclass the input control
            let preview_ptr = Arc::as_ptr(&preview) as *mut std::ffi::c_void;
            SetWindowLongPtrW(input_hwnd, GWLP_USERDATA, preview_ptr as _);
            
            let original_wndproc = SetWindowLongPtrW(
                input_hwnd,
                GWLP_WNDPROC,
                Self::input_subclass_proc as usize as _,
            );
            
            // Update the original_wndproc in the struct
            // This is a bit tricky since we already created the Arc
            // We'll use unsafe to update it
            let preview_ref = Arc::as_ptr(&preview) as *mut Self;
            (*preview_ref).original_wndproc = original_wndproc;
            
            Ok(preview)
        }
    }
    
    pub fn load_keyboard(&self, keyboard_path: &str) -> Result<()> {
        unsafe {
            // Free existing engine if any
            let mut engine_guard = self.engine.lock().unwrap();
            if let Some(engine) = *engine_guard {
                keymagic_engine_free(engine);
            }
            
            // Create new engine
            let engine = keymagic_engine_new();
            if engine.is_null() {
                return Err(anyhow::anyhow!("Failed to create engine"));
            }
            
            // Load keyboard
            let c_path = CString::new(keyboard_path)?;
            let result = keymagic_engine_load_keyboard(engine, c_path.as_ptr());
            
            if result != KeyMagicResult::Success {
                keymagic_engine_free(engine);
                return Err(anyhow::anyhow!("Failed to load keyboard"));
            }
            
            *engine_guard = Some(engine);
            
            // Clear and update UI
            let _ = SetWindowTextW(self.input_hwnd, w!(""));
            let _ = SetWindowTextW(self.composing_hwnd, w!(""));
            let _ = SetWindowTextW(self.output_hwnd, w!("Keyboard loaded. Start typing to test...\r\n"));
        }
        Ok(())
    }
    
    pub fn handle_command(&self, cmd: u16) -> Result<()> {
        unsafe {
            match cmd as i32 {
                IDC_CLEAR_BUTTON => {
                    let _ = SetWindowTextW(self.input_hwnd, w!(""));
                    let _ = SetWindowTextW(self.composing_hwnd, w!(""));
                    let _ = SetWindowTextW(self.output_hwnd, w!(""));
                    
                    // Reset engine
                    let engine_guard = self.engine.lock().unwrap();
                    if let Some(engine) = *engine_guard {
                        keymagic_engine_reset(engine);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    // Compatibility method for window.rs
    pub fn update_output(&self) {
        // This is handled by the subclassed window procedure now
        // This method is kept for compatibility but does nothing
    }
    
    unsafe extern "system" fn input_subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let preview_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Self;
        if preview_ptr.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        
        let preview = &*preview_ptr;
        
        match msg {
            WM_KEYDOWN => {
                let vk_code = wparam.0 as u32;
                
                // Log key down event
                println!("\n=== WM_KEYDOWN Event ===");
                println!("VK Code: 0x{:X} ({})", vk_code, vk_code);
                println!("Key Name: {}", get_vk_name(vk_code));
                
                // Get current keyboard state
                let mut keyboard_state = [0u8; 256];
                let _ = GetKeyboardState(&mut keyboard_state);
                
                // Log modifier states
                let shift = keyboard_state[VK_SHIFT.0 as usize] & 0x80 != 0;
                let ctrl = keyboard_state[VK_CONTROL.0 as usize] & 0x80 != 0;
                let alt = keyboard_state[VK_MENU.0 as usize] & 0x80 != 0;
                let caps = keyboard_state[VK_CAPITAL.0 as usize] & 0x01 != 0;
                
                println!("Modifiers: Shift={}, Ctrl={}, Alt={}, CapsLock={}", 
                         shift, ctrl, alt, caps);
                
                // Convert to character
                let mut char_buffer = [0u16; 2];
                let scan_code = MapVirtualKeyW(vk_code, MAPVK_VK_TO_VSC);
                println!("Scan Code: 0x{:X}", scan_code);
                
                let chars_written = ToUnicode(
                    vk_code,
                    scan_code,
                    Some(&keyboard_state),
                    &mut char_buffer,
                    0,
                );
                
                let character = if chars_written > 0 {
                    char::from_u32(char_buffer[0] as u32).unwrap_or('\0')
                } else {
                    '\0'
                };
                
                println!("ToUnicode result: {} chars", chars_written);
                if chars_written > 0 {
                    println!("Character: '{}' (U+{:04X})", character, character as u32);
                } else {
                    println!("Character: none");
                }
                
                // Process with engine
                let engine_guard = preview.engine.lock().unwrap();
                if let Some(engine) = *engine_guard {
                    let mut output = ProcessKeyOutput {
                        action_type: 0,
                        text: std::ptr::null_mut(),
                        delete_count: 0,
                        composing_text: std::ptr::null_mut(),
                        is_processed: 0,
                    };
                    
                    // Get modifier states for engine
                    let shift = if keyboard_state[VK_SHIFT.0 as usize] & 0x80 != 0 { 1 } else { 0 };
                    let ctrl = if keyboard_state[VK_CONTROL.0 as usize] & 0x80 != 0 { 1 } else { 0 };
                    let alt = if keyboard_state[VK_MENU.0 as usize] & 0x80 != 0 { 1 } else { 0 };
                    let caps = if keyboard_state[VK_CAPITAL.0 as usize] & 0x01 != 0 { 1 } else { 0 };
                    
                    println!("\n--- Calling Engine ---");
                    println!("Input: vk_code={}, char='{}' ({}), shift={}, ctrl={}, alt={}, caps={}",
                             vk_code, character, character as i8, shift, ctrl, alt, caps);
                    
                    let result = keymagic_engine_process_key_win(
                        engine,
                        vk_code as i32,
                        character as i8,
                        shift,
                        ctrl,
                        alt,
                        caps,
                        &mut output,
                    );
                    
                    println!("\n--- Engine Output ---");
                    println!("Result: {:?}", result);
                    
                    if result == KeyMagicResult::Success {
                        let action_str = match output.action_type {
                            0 => "None",
                            1 => "Insert",
                            2 => "Delete",
                            3 => "DeleteAndInsert",
                            _ => "Unknown",
                        };
                        
                        println!("Action Type: {} ({})", output.action_type, action_str);
                        println!("Delete Count: {}", output.delete_count);
                        println!("Is Processed: {}", output.is_processed);
                        
                        if !output.text.is_null() {
                            let text = std::ffi::CStr::from_ptr(output.text).to_string_lossy();
                            println!("Text: '{}'", text);
                        } else {
                            println!("Text: null");
                        }
                        
                        if !output.composing_text.is_null() {
                            let composing = std::ffi::CStr::from_ptr(output.composing_text).to_string_lossy();
                            println!("Composing Text: '{}'", composing);
                        } else {
                            println!("Composing Text: null");
                        }
                        
                        preview.handle_engine_output(&output, vk_code, engine);
                        
                        // Clean up
                        if !output.text.is_null() {
                            keymagic_free_string(output.text);
                        }
                        if !output.composing_text.is_null() {
                            keymagic_free_string(output.composing_text);
                        }
                        
                        println!("\nPreventing default: {}", output.is_processed != 0);
                        
                        // If processed, prevent default handling
                        if output.is_processed != 0 {
                            return LRESULT(0);
                        }
                    } else {
                        println!("Engine error: {:?}", result);
                    }
                }
            }
            _ => {}
        }
        
        CallWindowProcW(
            Some(std::mem::transmute(preview.original_wndproc)),
            hwnd,
            msg,
            wparam,
            lparam,
        )
    }
    
    fn handle_engine_output(&self, output: &ProcessKeyOutput, vk_code: u32, engine: *mut EngineHandle) {
        unsafe {
            println!("\n--- Handle Engine Output ---");
            
            // Update composing text display
            if !output.composing_text.is_null() {
                let composing = std::ffi::CStr::from_ptr(output.composing_text)
                    .to_string_lossy();
                println!("Composing text to display: '{}'", composing);
                
                let composing_wide: Vec<u16> = composing.encode_utf16().chain(std::iter::once(0)).collect();
                let _ = SetWindowTextW(self.composing_hwnd, PCWSTR(composing_wide.as_ptr()));
                
                // Should we commit?
                let should_commit = if vk_code == VK_SPACE.0 as u32 {
                    // Commit if composing ends with space or engine didn't process
                    let commit = output.is_processed == 0 || composing.ends_with(' ');
                    println!("Space key: is_processed={}, ends_with_space={}, should_commit={}", 
                             output.is_processed, composing.ends_with(' '), commit);
                    commit
                } else if vk_code == VK_RETURN.0 as u32 || vk_code == VK_TAB.0 as u32 {
                    println!("Return/Tab key: always commit");
                    true
                } else if vk_code == VK_ESCAPE.0 as u32 {
                    println!("Escape key: cancel composition");
                    // Clear composing
                    let _ = SetWindowTextW(self.composing_hwnd, w!(""));
                    return;
                } else {
                    println!("Other key: no commit");
                    false
                };
                
                if should_commit && !composing.is_empty() {
                    println!("Committing text: '{}'", composing);
                    // Append to output
                    let mut current_output = vec![0u16; 4096];
                    let len = GetWindowTextW(self.output_hwnd, &mut current_output);
                    
                    let mut new_output = String::from_utf16_lossy(&current_output[..len as usize]);
                    new_output.push_str(&composing);
                    if vk_code == VK_RETURN.0 as u32 {
                        new_output.push_str("\r\n");
                    } else if vk_code == VK_SPACE.0 as u32 && output.is_processed == 0 {
                        new_output.push(' ');
                    }
                    
                    let output_wide: Vec<u16> = new_output.encode_utf16().chain(std::iter::once(0)).collect();
                    let _ = SetWindowTextW(self.output_hwnd, PCWSTR(output_wide.as_ptr()));
                    
                    // Clear composing and reset engine
                    println!("Clearing composing display and resetting engine");
                    let _ = SetWindowTextW(self.composing_hwnd, w!(""));
                    keymagic_engine_reset(engine);
                } else if should_commit && composing.is_empty() {
                    println!("Should commit but composing is empty - skipping");
                }
            } else {
                println!("No composing text from engine");
            }
            
            println!("=== End of Key Processing ===\n");
        }
    }
}

impl Drop for KeyboardPreview {
    fn drop(&mut self) {
        // Free engine when preview is dropped
        let engine_guard = self.engine.lock().unwrap();
        if let Some(engine) = *engine_guard {
            keymagic_engine_free(engine);
        }
    }
}

// Helper function to get VK key name
fn get_vk_name(vk_code: u32) -> &'static str {
    match vk_code {
        0x08 => "VK_BACK",
        0x09 => "VK_TAB",
        0x0D => "VK_RETURN",
        0x10 => "VK_SHIFT",
        0x11 => "VK_CONTROL",
        0x12 => "VK_MENU (Alt)",
        0x13 => "VK_PAUSE",
        0x14 => "VK_CAPITAL (CapsLock)",
        0x1B => "VK_ESCAPE",
        0x20 => "VK_SPACE",
        0x21 => "VK_PRIOR (PageUp)",
        0x22 => "VK_NEXT (PageDown)",
        0x23 => "VK_END",
        0x24 => "VK_HOME",
        0x25 => "VK_LEFT",
        0x26 => "VK_UP",
        0x27 => "VK_RIGHT",
        0x28 => "VK_DOWN",
        0x2D => "VK_INSERT",
        0x2E => "VK_DELETE",
        0x30..=0x39 => "VK_0-9",
        0x41..=0x5A => "VK_A-Z",
        0x60..=0x69 => "VK_NUMPAD0-9",
        0x70..=0x87 => "VK_F1-F24",
        0xA0 => "VK_LSHIFT",
        0xA1 => "VK_RSHIFT",
        0xA2 => "VK_LCONTROL",
        0xA3 => "VK_RCONTROL",
        0xA4 => "VK_LMENU (LAlt)",
        0xA5 => "VK_RMENU (RAlt)",
        0xBA => "VK_OEM_1 (;:)",
        0xBB => "VK_OEM_PLUS",
        0xBC => "VK_OEM_COMMA",
        0xBD => "VK_OEM_MINUS",
        0xBE => "VK_OEM_PERIOD",
        0xBF => "VK_OEM_2 (/?)",
        0xC0 => "VK_OEM_3 (`~)",
        0xDB => "VK_OEM_4 ([{)",
        0xDC => "VK_OEM_5 (\\|)",
        0xDD => "VK_OEM_6 (]})",
        0xDE => "VK_OEM_7 ('\")",
        _ => "Unknown",
    }
}