// Keyboard Hook Implementation with Robust Error Handling
//
// This module implements a Windows low-level keyboard hook that is resilient to:
// 1. Panics in the hook procedure (which would cause Windows to unhook it)
// 2. Mutex poisoning from panics in other threads
// 3. Callback panics that could crash the hook
// 4. Performance issues that could cause Windows to remove the hook
// 5. Invalid memory access from null/invalid pointers
//
// Key safety features:
// - All panics are caught at the hook procedure boundary
// - Mutex poisoning is recovered from gracefully
// - Callbacks execute in separate threads with panic isolation
// - Processing time is monitored to detect performance issues
// - All pointers are validated before dereferencing
//
// Usage:
// The hook also provides health monitoring via is_hook_active() and check_health()
// which can be called periodically to ensure the hook is still functioning.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};
use windows::{
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        UI::Input::KeyboardAndMouse::*,
        System::LibraryLoader::GetModuleHandleW,
    },
};
use anyhow::{Result, anyhow};
use log::{error, warn};

static mut HOOK_INSTANCE: Option<Arc<KeyboardHook>> = None;

type HotkeyCallback = Arc<dyn Fn() + Send + Sync>;

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
    last_modifier_state: Arc<Mutex<u32>>,
    modifier_only_candidate: Arc<Mutex<Option<u32>>>,
    #[allow(dead_code)]
    last_health_check: Arc<Mutex<Instant>>,
    #[allow(dead_code)]
    hook_installed_at: Arc<Mutex<Option<Instant>>>,
}

impl KeyboardHook {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            hook: RwLock::new(None),
            hotkeys: Arc::new(Mutex::new(HashMap::new())),
            modifier_state: Arc::new(Mutex::new(0)),
            last_modifier_state: Arc::new(Mutex::new(0)),
            modifier_only_candidate: Arc::new(Mutex::new(None)),
            last_health_check: Arc::new(Mutex::new(Instant::now())),
            hook_installed_at: Arc::new(Mutex::new(None)),
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
            *self.hook_installed_at.lock().unwrap() = Some(Instant::now());
            Ok(())
        }
    }

    #[allow(dead_code)]
    pub fn is_hook_active(&self) -> bool {
        self.hook.read().unwrap().is_some()
    }

    #[allow(dead_code)]
    pub fn check_health(&self) -> bool {
        let mut last_check = self.last_health_check.lock().unwrap();
        let now = Instant::now();
        
        // Only check every second to avoid overhead
        if now.duration_since(*last_check) < Duration::from_secs(1) {
            return true;
        }
        
        *last_check = now;
        
        // Check if hook is still installed
        let is_active = self.is_hook_active();
        if !is_active {
            warn!("Keyboard hook is no longer active");
        }
        
        is_active
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
            callback: Arc::new(callback),
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

    fn process_key(&self, vk_code: u32, is_keydown: bool) -> bool {
        // Use catch_unwind to prevent panics from propagating
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.process_key_internal(vk_code, is_keydown)
        }));
        
        match result {
            Ok(should_eat) => should_eat,
            Err(e) => {
                error!("Panic in process_key: {:?}", e);
                false // Don't eat the key on error
            }
        }
    }
    
    fn process_key_internal(&self, vk_code: u32, is_keydown: bool) -> bool {
        // Start timing to ensure we don't take too long
        let start = Instant::now();
        
        // Handle mutex poisoning gracefully
        let mut modifier_state = match self.modifier_state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                error!("Modifier state mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };
        
        let mut last_modifier_state = match self.last_modifier_state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                error!("Last modifier state mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };
        
        let mut modifier_only_candidate = match self.modifier_only_candidate.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                error!("Modifier candidate mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };
        
        let is_modifier_key = match vk_code {
            vk if vk == VK_CONTROL.0 as u32 || vk == VK_LCONTROL.0 as u32 || vk == VK_RCONTROL.0 as u32 => true,
            vk if vk == VK_MENU.0 as u32 || vk == VK_LMENU.0 as u32 || vk == VK_RMENU.0 as u32 => true,
            vk if vk == VK_SHIFT.0 as u32 || vk == VK_LSHIFT.0 as u32 || vk == VK_RSHIFT.0 as u32 => true,
            vk if vk == VK_LWIN.0 as u32 || vk == VK_RWIN.0 as u32 => true,
            _ => false,
        };
        
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
        
        // Track if we should eat this key event
        let mut should_eat_key = false;

        // Handle modifier-only hotkey detection
        if is_modifier_key {
            if is_keydown {
                // When a modifier is pressed, set it as a candidate
                *modifier_only_candidate = Some(current_modifiers);
            } else {
                // When a modifier is released, check if we should trigger a modifier-only hotkey
                if let Some(candidate_modifiers) = *modifier_only_candidate {
                    // Only trigger if we're releasing from the candidate state without any regular keys pressed
                    if *last_modifier_state == candidate_modifiers && current_modifiers != candidate_modifiers {
                        // Check for matching modifier-only hotkeys
                        let mut hotkeys = match self.hotkeys.lock() {
                            Ok(guard) => guard,
                            Err(poisoned) => {
                                error!("Hotkeys mutex poisoned, recovering");
                                poisoned.into_inner()
                            }
                        };
                        for (_, state) in hotkeys.iter_mut() {
                            if state.hotkey.vk_code.is_none() && state.hotkey.modifiers == candidate_modifiers && !state.triggered {
                                state.triggered = true;
                                // Execute callback in a safe way
                                let callback = state.callback.clone();
                                drop(hotkeys); // Release lock before callback
                                self.execute_callback_safely(&callback);
                                break;
                            }
                        }
                    }
                }
                // Clear candidate when any modifier is released
                *modifier_only_candidate = None;
            }
        } else {
            // Non-modifier key pressed - clear modifier-only candidate
            if is_keydown {
                *modifier_only_candidate = None;
                
                // Check hotkeys with regular keys
                let mut hotkeys = match self.hotkeys.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        error!("Hotkeys mutex poisoned, recovering");
                        poisoned.into_inner()
                    }
                };
                let mut callback_to_execute = None;
                for (_, state) in hotkeys.iter_mut() {
                    if let Some(hotkey_vk) = state.hotkey.vk_code {
                        // For hotkeys with a regular key
                        if vk_code == hotkey_vk && current_modifiers == state.hotkey.modifiers && !state.triggered {
                            state.triggered = true;
                            callback_to_execute = Some(state.callback.clone());
                            // Eat the key event for hotkeys with regular keys
                            should_eat_key = true;
                            break;
                        }
                    }
                }
                drop(hotkeys); // Release lock before callback
                
                if let Some(callback) = callback_to_execute {
                    self.execute_callback_safely(&callback);
                }
            } else {
                // Reset triggered state for regular hotkeys when key is released
                let mut hotkeys = match self.hotkeys.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        error!("Hotkeys mutex poisoned, recovering");
                        poisoned.into_inner()
                    }
                };
                for (_, state) in hotkeys.iter_mut() {
                    if state.triggered {
                        if let Some(hotkey_vk) = state.hotkey.vk_code {
                            // For regular hotkeys, reset when the key is released
                            if vk_code == hotkey_vk {
                                state.triggered = false;
                            }
                        }
                    }
                }
            }
        }
        
        // Reset triggered state for modifier-only hotkeys when modifiers change
        if current_modifiers != *last_modifier_state {
            let mut hotkeys = match self.hotkeys.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    error!("Hotkeys mutex poisoned, recovering");
                    poisoned.into_inner()
                }
            };
            for (_, state) in hotkeys.iter_mut() {
                if state.triggered && state.hotkey.vk_code.is_none() {
                    if current_modifiers != state.hotkey.modifiers {
                        state.triggered = false;
                    }
                }
            }
        }
        
        *last_modifier_state = current_modifiers;
        
        // Check if we're taking too long
        let elapsed = start.elapsed();
        if elapsed > Duration::from_millis(10) {
            warn!("Keyboard hook processing took {:?}, this may cause issues", elapsed);
        }
        
        should_eat_key
    }
    
    fn execute_callback_safely(&self, callback: &HotkeyCallback) {
        // Execute callback in a separate thread to prevent blocking the hook
        let callback_clone = callback.clone();
        std::thread::spawn(move || {
            // Catch panics in callbacks
            let result = catch_unwind(AssertUnwindSafe(|| {
                callback_clone();
            }));
            
            if let Err(e) = result {
                error!("Hotkey callback panicked: {:?}", e);
            }
        });
    }
}

unsafe extern "system" fn low_level_keyboard_proc(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Catch ALL panics to prevent the hook from being removed
    let result = catch_unwind(AssertUnwindSafe(|| {
        if ncode < 0 {
            return CallNextHookEx(None, ncode, wparam, lparam);
        }
        
        // Validate lparam before dereferencing
        if lparam.0 == 0 {
            error!("Invalid lparam in keyboard hook");
            return CallNextHookEx(None, ncode, wparam, lparam);
        }
        
        #[allow(static_mut_refs)]
        let hook_ref = unsafe { HOOK_INSTANCE.as_ref() };
        if let Some(hook) = hook_ref {
            // Safely dereference the keyboard struct
            let kb_struct_ptr = lparam.0 as *const KBDLLHOOKSTRUCT;
            if kb_struct_ptr.is_null() {
                error!("Null KBDLLHOOKSTRUCT pointer");
                return CallNextHookEx(None, ncode, wparam, lparam);
            }
            
            let kb_struct = *kb_struct_ptr;
            let vk_code = kb_struct.vkCode;
            let is_keydown = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
            
            // If process_key returns true, eat the key event
            if hook.process_key(vk_code, is_keydown) {
                return LRESULT(1); // Non-zero return value prevents further processing
            }
        }
        
        CallNextHookEx(None, ncode, wparam, lparam)
    }));
    
    match result {
        Ok(lresult) => lresult,
        Err(e) => {
            error!("CRITICAL: Panic in keyboard hook procedure: {:?}", e);
            // Always pass the key through on error to prevent system issues
            CallNextHookEx(None, ncode, wparam, lparam)
        }
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        let _ = self.uninstall();
    }
}