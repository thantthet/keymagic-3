use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Context, Result};
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

use crate::core::KeyboardManager;

pub struct HotkeyManager {
    registered_shortcuts: Arc<Mutex<HashMap<String, String>>>, // keyboard_id -> shortcut_string
    shortcut_to_keyboard: Arc<Mutex<HashMap<String, String>>>, // shortcut_string -> keyboard_id
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            registered_shortcuts: Arc::new(Mutex::new(HashMap::new())),
            shortcut_to_keyboard: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initialize hotkeys for all keyboards
    pub fn initialize<R: Runtime>(&self, app: &AppHandle<R>, keyboard_manager: Arc<KeyboardManager>) -> Result<()> {
        self.register_all_hotkeys(app, keyboard_manager)
    }

    /// Register hotkeys for all keyboards
    pub fn register_all_hotkeys<R: Runtime>(&self, app: &AppHandle<R>, keyboard_manager: Arc<KeyboardManager>) -> Result<()> {
        // Clear existing hotkeys first
        self.unregister_all_hotkeys(app)?;
        
        let keyboards = keyboard_manager.get_keyboards();
        
        for keyboard in keyboards {
            // Check if hotkey is explicitly disabled (empty string)
            if keyboard.hotkey.as_ref() == Some(&String::new()) {
                // User explicitly disabled hotkey, skip registration
                continue;
            }
            
            // Use custom hotkey if set, otherwise use default hotkey
            let effective_hotkey = keyboard.hotkey.as_ref().or(keyboard.default_hotkey.as_ref());
            
            if let Some(hotkey_str) = effective_hotkey {
                if !hotkey_str.is_empty() {
                    if let Err(e) = self.register_hotkey(app, &keyboard.id, hotkey_str, keyboard_manager.clone()) {
                        log::warn!("Failed to register hotkey '{}' for keyboard '{}': {}", hotkey_str, keyboard.id, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Register a single hotkey for a keyboard
    pub fn register_hotkey<R: Runtime>(
        &self, 
        app: &AppHandle<R>, 
        keyboard_id: &str, 
        hotkey_str: &str,
        keyboard_manager: Arc<KeyboardManager>
    ) -> Result<()> {
        let shortcut = self.parse_hotkey(hotkey_str)
            .context(format!("Failed to parse hotkey: {}", hotkey_str))?;
        
        // For now, just register the shortcut - the handling will be done via events
        app.global_shortcut().register(shortcut.clone())
            .context(format!("Failed to register global shortcut: {}", hotkey_str))?;
        
        // Get normalized string representation of the shortcut
        let normalized_shortcut = self.shortcut_to_string(&shortcut);
        
        // Store the registered shortcut
        let mut registered = self.registered_shortcuts.lock().unwrap();
        registered.insert(keyboard_id.to_string(), hotkey_str.to_string());
        
        let mut shortcut_map = self.shortcut_to_keyboard.lock().unwrap();
        shortcut_map.insert(normalized_shortcut.clone(), keyboard_id.to_string());
        
        log::info!("Registered hotkey '{}' (normalized: '{}') for keyboard '{}'", hotkey_str, normalized_shortcut, keyboard_id);
        
        Ok(())
    }

    /// Unregister a specific hotkey
    pub fn unregister_hotkey<R: Runtime>(&self, app: &AppHandle<R>, keyboard_id: &str) -> Result<()> {
        let mut registered = self.registered_shortcuts.lock().unwrap();
        
        if let Some(hotkey_str) = registered.remove(keyboard_id) {
            if let Ok(shortcut) = self.parse_hotkey(&hotkey_str) {
                app.global_shortcut().unregister(shortcut)
                    .context(format!("Failed to unregister hotkey: {}", hotkey_str))?;
            }
        }
        
        Ok(())
    }

    /// Unregister all hotkeys
    pub fn unregister_all_hotkeys<R: Runtime>(&self, app: &AppHandle<R>) -> Result<()> {
        let mut registered = self.registered_shortcuts.lock().unwrap();
        let mut shortcut_map = self.shortcut_to_keyboard.lock().unwrap();
        
        for (_, hotkey_str) in registered.iter() {
            if let Ok(shortcut) = self.parse_hotkey(hotkey_str) {
                let _ = app.global_shortcut().unregister(shortcut);
            }
        }
        
        registered.clear();
        shortcut_map.clear();
        Ok(())
    }

    /// Refresh all hotkeys (useful when keyboards are updated)
    pub fn refresh_hotkeys<R: Runtime>(&self, app: &AppHandle<R>, keyboard_manager: Arc<KeyboardManager>) -> Result<()> {
        self.register_all_hotkeys(app, keyboard_manager)
    }

    /// Validate a hotkey string without registering it
    pub fn validate_hotkey(&self, hotkey_str: &str) -> Result<()> {
        // Empty hotkey is valid (removes hotkey)
        if hotkey_str.is_empty() {
            return Ok(());
        }
        
        // Try to parse the hotkey - if it fails, return the error
        self.parse_hotkey(hotkey_str)?;
        
        Ok(())
    }

    /// Parse a hotkey string into a Shortcut
    fn parse_hotkey(&self, hotkey_str: &str) -> Result<Shortcut> {
        let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();
        
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Hotkey cannot be empty"));
        }
        
        let mut modifiers = Modifiers::empty();
        let mut key_code: Option<Code> = None;
        let mut non_modifier_keys = Vec::new();
        
        for part in parts {
            if part.is_empty() {
                return Err(anyhow::anyhow!("Invalid hotkey format: contains empty key (consecutive '+' signs?)"));
            }
            
            match part.to_uppercase().as_str() {
                "CTRL" | "CONTROL" => modifiers |= Modifiers::CONTROL,
                "ALT" => modifiers |= Modifiers::ALT,
                "SHIFT" => modifiers |= Modifiers::SHIFT,
                "META" | "WIN" | "SUPER" | "CMD" => modifiers |= Modifiers::META,
                key => {
                    non_modifier_keys.push(part);
                    if key_code.is_some() {
                        return Err(anyhow::anyhow!(
                            "Hotkey can only contain one non-modifier key. Found multiple: {}",
                            non_modifier_keys.join(", ")
                        ));
                    }
                    key_code = Some(self.parse_key_code(key)?);
                }
            }
        }
        
        let key = key_code.ok_or_else(|| {
            anyhow::anyhow!("Hotkey must include at least one non-modifier key (not just Ctrl/Alt/Shift/Win)")
        })?;
        
        Ok(Shortcut::new(Some(modifiers), key))
    }

    /// Parse a key string into a Code
    fn parse_key_code(&self, key: &str) -> Result<Code> {
        match key.to_uppercase().as_str() {
            // Letters
            "A" => Ok(Code::KeyA),
            "B" => Ok(Code::KeyB),
            "C" => Ok(Code::KeyC),
            "D" => Ok(Code::KeyD),
            "E" => Ok(Code::KeyE),
            "F" => Ok(Code::KeyF),
            "G" => Ok(Code::KeyG),
            "H" => Ok(Code::KeyH),
            "I" => Ok(Code::KeyI),
            "J" => Ok(Code::KeyJ),
            "K" => Ok(Code::KeyK),
            "L" => Ok(Code::KeyL),
            "M" => Ok(Code::KeyM),
            "N" => Ok(Code::KeyN),
            "O" => Ok(Code::KeyO),
            "P" => Ok(Code::KeyP),
            "Q" => Ok(Code::KeyQ),
            "R" => Ok(Code::KeyR),
            "S" => Ok(Code::KeyS),
            "T" => Ok(Code::KeyT),
            "U" => Ok(Code::KeyU),
            "V" => Ok(Code::KeyV),
            "W" => Ok(Code::KeyW),
            "X" => Ok(Code::KeyX),
            "Y" => Ok(Code::KeyY),
            "Z" => Ok(Code::KeyZ),
            
            // Numbers
            "0" => Ok(Code::Digit0),
            "1" => Ok(Code::Digit1),
            "2" => Ok(Code::Digit2),
            "3" => Ok(Code::Digit3),
            "4" => Ok(Code::Digit4),
            "5" => Ok(Code::Digit5),
            "6" => Ok(Code::Digit6),
            "7" => Ok(Code::Digit7),
            "8" => Ok(Code::Digit8),
            "9" => Ok(Code::Digit9),
            
            // Function keys
            "F1" => Ok(Code::F1),
            "F2" => Ok(Code::F2),
            "F3" => Ok(Code::F3),
            "F4" => Ok(Code::F4),
            "F5" => Ok(Code::F5),
            "F6" => Ok(Code::F6),
            "F7" => Ok(Code::F7),
            "F8" => Ok(Code::F8),
            "F9" => Ok(Code::F9),
            "F10" => Ok(Code::F10),
            "F11" => Ok(Code::F11),
            "F12" => Ok(Code::F12),
            
            // Special keys
            "SPACE" => Ok(Code::Space),
            "TAB" => Ok(Code::Tab),
            "ENTER" | "RETURN" => Ok(Code::Enter),
            "ESCAPE" | "ESC" => Ok(Code::Escape),
            "BACKSPACE" => Ok(Code::Backspace),
            "DELETE" => Ok(Code::Delete),
            "HOME" => Ok(Code::Home),
            "END" => Ok(Code::End),
            "PAGEUP" | "PGUP" => Ok(Code::PageUp),
            "PAGEDOWN" | "PGDOWN" => Ok(Code::PageDown),
            "LEFT" => Ok(Code::ArrowLeft),
            "RIGHT" => Ok(Code::ArrowRight),
            "UP" => Ok(Code::ArrowUp),
            "DOWN" => Ok(Code::ArrowDown),
            "INSERT" => Ok(Code::Insert),
            
            // Punctuation and symbols
            "MINUS" | "-" => Ok(Code::Minus),
            "EQUAL" | "=" => Ok(Code::Equal),
            "BRACKET_LEFT" | "[" => Ok(Code::BracketLeft),
            "BRACKET_RIGHT" | "]" => Ok(Code::BracketRight),
            "BACKSLASH" | "\\" => Ok(Code::Backslash),
            "SEMICOLON" | ";" => Ok(Code::Semicolon),
            "QUOTE" | "'" => Ok(Code::Quote),
            "BACKQUOTE" | "`" => Ok(Code::Backquote),
            "COMMA" | "," => Ok(Code::Comma),
            "PERIOD" | "." => Ok(Code::Period),
            "SLASH" | "/" => Ok(Code::Slash),
            
            _ => {
                // Provide helpful suggestions for common mistakes
                let suggestions = match key {
                    "CTRL" | "CONTROL" | "ALT" | "SHIFT" | "META" | "WIN" | "SUPER" | "CMD" => {
                        format!("'{}' is a modifier key and should be used with another key (e.g., {}+A)", key, key)
                    }
                    "PLUS" | "+" => {
                        "To use the Plus/Add key, use 'Equal' or '=' in your hotkey".to_string()
                    }
                    "DASH" => {
                        "Use 'Minus' or '-' for the dash/hyphen key".to_string()
                    }
                    "TILDE" | "~" => {
                        "Use 'Backquote' or '`' for the tilde key".to_string()
                    }
                    "EXCLAMATION" | "!" => {
                        "Special characters like '!' cannot be used directly. Use the base key instead (e.g., '1' with Shift)".to_string()
                    }
                    "AT" | "@" => {
                        "Special characters like '@' cannot be used directly. Use the base key instead (e.g., '2' with Shift)".to_string()
                    }
                    "APOSTROPHE" => {
                        "Use 'Quote' or single quote (') for the apostrophe key".to_string()
                    }
                    "PAGEUP" | "PAGEDOWN" => {
                        format!("Use 'PgUp' or 'PgDown' instead of '{}'", key)
                    }
                    _ => {
                        format!("'{}' is not a recognized key. Common keys: A-Z, 0-9, F1-F12, Space, Tab, Enter, Escape, Arrow keys (Left/Right/Up/Down)", key)
                    }
                };
                
                Err(anyhow::anyhow!("Invalid key: {}. {}", key, suggestions))
            }
        }
    }

    /// Get all registered shortcuts
    pub fn get_registered_shortcuts(&self) -> HashMap<String, String> {
        self.registered_shortcuts.lock().unwrap().clone()
    }
    
    /// Get keyboard ID for a shortcut
    pub fn get_keyboard_for_shortcut(&self, shortcut: &Shortcut) -> Option<String> {
        // Convert shortcut back to string format to look up
        let shortcut_str = self.shortcut_to_string(shortcut);
        self.shortcut_to_keyboard.lock().unwrap().get(&shortcut_str).cloned()
    }
    
    /// Convert a Shortcut to its string representation
    fn shortcut_to_string(&self, shortcut: &Shortcut) -> String {
        let mut parts = Vec::new();
        
        if shortcut.mods.contains(Modifiers::CONTROL) {
            parts.push("Ctrl".to_string());
        }
        if shortcut.mods.contains(Modifiers::ALT) {
            parts.push("Alt".to_string());
        }
        if shortcut.mods.contains(Modifiers::SHIFT) {
            parts.push("Shift".to_string());
        }
        if shortcut.mods.contains(Modifiers::META) {
            parts.push("Meta".to_string());
        }
        
        // Add the key code
        parts.push(self.code_to_string(&shortcut.key));
        
        parts.join("+")
    }
    
    /// Convert a Code to its string representation
    fn code_to_string(&self, code: &Code) -> String {
        match code {
            Code::KeyA => "A",
            Code::KeyB => "B",
            Code::KeyC => "C",
            Code::KeyD => "D",
            Code::KeyE => "E",
            Code::KeyF => "F",
            Code::KeyG => "G",
            Code::KeyH => "H",
            Code::KeyI => "I",
            Code::KeyJ => "J",
            Code::KeyK => "K",
            Code::KeyL => "L",
            Code::KeyM => "M",
            Code::KeyN => "N",
            Code::KeyO => "O",
            Code::KeyP => "P",
            Code::KeyQ => "Q",
            Code::KeyR => "R",
            Code::KeyS => "S",
            Code::KeyT => "T",
            Code::KeyU => "U",
            Code::KeyV => "V",
            Code::KeyW => "W",
            Code::KeyX => "X",
            Code::KeyY => "Y",
            Code::KeyZ => "Z",
            Code::Digit0 => "0",
            Code::Digit1 => "1",
            Code::Digit2 => "2",
            Code::Digit3 => "3",
            Code::Digit4 => "4",
            Code::Digit5 => "5",
            Code::Digit6 => "6",
            Code::Digit7 => "7",
            Code::Digit8 => "8",
            Code::Digit9 => "9",
            Code::F1 => "F1",
            Code::F2 => "F2",
            Code::F3 => "F3",
            Code::F4 => "F4",
            Code::F5 => "F5",
            Code::F6 => "F6",
            Code::F7 => "F7",
            Code::F8 => "F8",
            Code::F9 => "F9",
            Code::F10 => "F10",
            Code::F11 => "F11",
            Code::F12 => "F12",
            Code::Space => "SPACE",
            Code::Tab => "TAB",
            Code::Enter => "ENTER",
            Code::Escape => "ESCAPE",
            Code::Backspace => "BACKSPACE",
            Code::Delete => "DELETE",
            Code::Home => "HOME",
            Code::End => "END",
            Code::PageUp => "PAGEUP",
            Code::PageDown => "PAGEDOWN",
            Code::ArrowLeft => "LEFT",
            Code::ArrowRight => "RIGHT",
            Code::ArrowUp => "UP",
            Code::ArrowDown => "DOWN",
            Code::Insert => "INSERT",
            Code::Minus => "MINUS",
            Code::Equal => "EQUAL",
            Code::BracketLeft => "BRACKET_LEFT",
            Code::BracketRight => "BRACKET_RIGHT",
            Code::Backslash => "BACKSLASH",
            Code::Semicolon => "SEMICOLON",
            Code::Quote => "QUOTE",
            Code::Backquote => "BACKQUOTE",
            Code::Comma => "COMMA",
            Code::Period => "PERIOD",
            Code::Slash => "SLASH",
            _ => "UNKNOWN",
        }.to_string()
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}