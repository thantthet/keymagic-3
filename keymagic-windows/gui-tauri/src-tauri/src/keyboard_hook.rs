use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use windows::{
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        UI::Input::KeyboardAndMouse::*,
        System::LibraryLoader::GetModuleHandleW,
    },
};
use anyhow::{Result, anyhow};

static mut HOOK_INSTANCE: Option<Arc<KeyboardHook>> = None;

type HotkeyCallback = Box<dyn Fn() + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hotkey {
    modifiers: u32,
    vk_code: Option<u32>,
}

impl Hotkey {
    pub fn from_string(hotkey: &str) -> Result<Self> {
        let parts: Vec<&str> = hotkey.split('+').collect();
        if parts.is_empty() {
            return Err(anyhow!("Invalid hotkey format"));
        }

        let mut modifiers = 0u32;
        let mut vk_code = None;

        for part in parts.iter() {
            let part = part.trim();
            
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= MOD_CONTROL,
                "alt" => modifiers |= MOD_ALT,
                "shift" => modifiers |= MOD_SHIFT,
                "win" | "super" | "meta" => modifiers |= MOD_WIN,
                _ => {
                    if vk_code.is_some() {
                        return Err(anyhow!("Multiple non-modifier keys specified"));
                    }
                    
                    vk_code = Some(match part.to_uppercase().as_str() {
                        "A" => VK_A.0 as u32,
                        "B" => VK_B.0 as u32,
                        "C" => VK_C.0 as u32,
                        "D" => VK_D.0 as u32,
                        "E" => VK_E.0 as u32,
                        "F" => VK_F.0 as u32,
                        "G" => VK_G.0 as u32,
                        "H" => VK_H.0 as u32,
                        "I" => VK_I.0 as u32,
                        "J" => VK_J.0 as u32,
                        "K" => VK_K.0 as u32,
                        "L" => VK_L.0 as u32,
                        "M" => VK_M.0 as u32,
                        "N" => VK_N.0 as u32,
                        "O" => VK_O.0 as u32,
                        "P" => VK_P.0 as u32,
                        "Q" => VK_Q.0 as u32,
                        "R" => VK_R.0 as u32,
                        "S" => VK_S.0 as u32,
                        "T" => VK_T.0 as u32,
                        "U" => VK_U.0 as u32,
                        "V" => VK_V.0 as u32,
                        "W" => VK_W.0 as u32,
                        "X" => VK_X.0 as u32,
                        "Y" => VK_Y.0 as u32,
                        "Z" => VK_Z.0 as u32,
                        "0" => VK_0.0 as u32,
                        "1" => VK_1.0 as u32,
                        "2" => VK_2.0 as u32,
                        "3" => VK_3.0 as u32,
                        "4" => VK_4.0 as u32,
                        "5" => VK_5.0 as u32,
                        "6" => VK_6.0 as u32,
                        "7" => VK_7.0 as u32,
                        "8" => VK_8.0 as u32,
                        "9" => VK_9.0 as u32,
                        "F1" => VK_F1.0 as u32,
                        "F2" => VK_F2.0 as u32,
                        "F3" => VK_F3.0 as u32,
                        "F4" => VK_F4.0 as u32,
                        "F5" => VK_F5.0 as u32,
                        "F6" => VK_F6.0 as u32,
                        "F7" => VK_F7.0 as u32,
                        "F8" => VK_F8.0 as u32,
                        "F9" => VK_F9.0 as u32,
                        "F10" => VK_F10.0 as u32,
                        "F11" => VK_F11.0 as u32,
                        "F12" => VK_F12.0 as u32,
                        "SPACE" => VK_SPACE.0 as u32,
                        "TAB" => VK_TAB.0 as u32,
                        "ENTER" | "RETURN" => VK_RETURN.0 as u32,
                        "ESC" | "ESCAPE" => VK_ESCAPE.0 as u32,
                        "BACKSPACE" => VK_BACK.0 as u32,
                        "DELETE" => VK_DELETE.0 as u32,
                        "INSERT" => VK_INSERT.0 as u32,
                        "HOME" => VK_HOME.0 as u32,
                        "END" => VK_END.0 as u32,
                        "PAGEUP" => VK_PRIOR.0 as u32,
                        "PAGEDOWN" => VK_NEXT.0 as u32,
                        "LEFT" => VK_LEFT.0 as u32,
                        "RIGHT" => VK_RIGHT.0 as u32,
                        "UP" => VK_UP.0 as u32,
                        "DOWN" => VK_DOWN.0 as u32,
                        _ => return Err(anyhow!("Unknown key: {}", part)),
                    });
                }
            }
        }

        if modifiers == 0 {
            return Err(anyhow!("Hotkey must include at least one modifier (Ctrl, Alt, Shift, or Win)"));
        }

        Ok(Hotkey { modifiers, vk_code })
    }
}

// Modifier constants
const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;
const MOD_WIN: u32 = 0x0008;

struct HotkeyState {
    hotkey: Hotkey,
    callback: HotkeyCallback,
    triggered: bool,
}

pub struct KeyboardHook {
    hook: RwLock<Option<HHOOK>>,
    hotkeys: Arc<Mutex<HashMap<String, HotkeyState>>>,
    modifier_state: Arc<Mutex<u32>>,
}

impl KeyboardHook {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            hook: RwLock::new(None),
            hotkeys: Arc::new(Mutex::new(HashMap::new())),
            modifier_state: Arc::new(Mutex::new(0)),
        })
    }

    pub fn install(self: &Arc<Self>) -> Result<()> {
        unsafe {
            // Store the instance globally
            HOOK_INSTANCE = Some(self.clone());

            let module = GetModuleHandleW(None)?;
            let hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_proc),
                module,
                0,
            )?;

            *self.hook.write().unwrap() = Some(hook);
            Ok(())
        }
    }

    pub fn uninstall(&self) -> Result<()> {
        if let Some(hook) = *self.hook.write().unwrap() {
            unsafe {
                UnhookWindowsHookEx(hook)?;
                HOOK_INSTANCE = None;
            }
        }
        Ok(())
    }

    pub fn register_hotkey<F>(&self, hotkey_str: &str, callback: F) -> Result<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let hotkey = Hotkey::from_string(hotkey_str)?;
        let mut hotkeys = self.hotkeys.lock().unwrap();
        
        hotkeys.insert(hotkey_str.to_string(), HotkeyState {
            hotkey,
            callback: Box::new(callback),
            triggered: false,
        });
        
        Ok(())
    }

    pub fn unregister_hotkey(&self, hotkey_str: &str) -> Result<()> {
        let mut hotkeys = self.hotkeys.lock().unwrap();
        hotkeys.remove(hotkey_str);
        Ok(())
    }

    pub fn unregister_all_hotkeys(&self) -> Result<()> {
        let mut hotkeys = self.hotkeys.lock().unwrap();
        hotkeys.clear();
        Ok(())
    }

    fn process_key(&self, vk_code: u32, is_keydown: bool) {
        let mut modifier_state = self.modifier_state.lock().unwrap();
        
        // Update modifier state
        match vk_code {
            vk if vk == VK_CONTROL.0 as u32 || vk == VK_LCONTROL.0 as u32 || vk == VK_RCONTROL.0 as u32 => {
                if is_keydown {
                    *modifier_state |= MOD_CONTROL;
                } else {
                    *modifier_state &= !MOD_CONTROL;
                }
            }
            vk if vk == VK_MENU.0 as u32 || vk == VK_LMENU.0 as u32 || vk == VK_RMENU.0 as u32 => {
                if is_keydown {
                    *modifier_state |= MOD_ALT;
                } else {
                    *modifier_state &= !MOD_ALT;
                }
            }
            vk if vk == VK_SHIFT.0 as u32 || vk == VK_LSHIFT.0 as u32 || vk == VK_RSHIFT.0 as u32 => {
                if is_keydown {
                    *modifier_state |= MOD_SHIFT;
                } else {
                    *modifier_state &= !MOD_SHIFT;
                }
            }
            vk if vk == VK_LWIN.0 as u32 || vk == VK_RWIN.0 as u32 => {
                if is_keydown {
                    *modifier_state |= MOD_WIN;
                } else {
                    *modifier_state &= !MOD_WIN;
                }
            }
            _ => {}
        }

        let current_modifiers = *modifier_state;
        drop(modifier_state);

        // Check hotkeys
        let mut hotkeys = self.hotkeys.lock().unwrap();
        
        for (_, state) in hotkeys.iter_mut() {
            if is_keydown {
                // For modifier-only hotkeys
                if state.hotkey.vk_code.is_none() {
                    // Check if exact modifiers are pressed
                    if current_modifiers == state.hotkey.modifiers && !state.triggered {
                        state.triggered = true;
                        (state.callback)();
                    }
                } else if let Some(hotkey_vk) = state.hotkey.vk_code {
                    // For hotkeys with a regular key
                    if vk_code == hotkey_vk && current_modifiers == state.hotkey.modifiers && !state.triggered {
                        state.triggered = true;
                        (state.callback)();
                    }
                }
            } else {
                // Reset triggered state when any key is released
                if state.triggered {
                    // For modifier-only hotkeys, reset when any modifier is released
                    if state.hotkey.vk_code.is_none() {
                        if current_modifiers != state.hotkey.modifiers {
                            state.triggered = false;
                        }
                    } else if let Some(hotkey_vk) = state.hotkey.vk_code {
                        // For regular hotkeys, reset when the key or modifiers are released
                        if vk_code == hotkey_vk || current_modifiers != state.hotkey.modifiers {
                            state.triggered = false;
                        }
                    }
                }
            }
        }
    }
}

unsafe extern "system" fn low_level_keyboard_proc(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if ncode >= 0 {
        if let Some(hook) = &HOOK_INSTANCE {
            let kb_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kb_struct.vkCode;
            let is_keydown = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
            
            hook.process_key(vk_code, is_keydown);
        }
    }
    
    CallNextHookEx(None, ncode, wparam, lparam)
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        let _ = self.uninstall();
    }
}