//! Hotkey parsing and representation

use crate::VirtualKey;
use crate::error::{Error, Result};

/// Represents a parsed hotkey with key and modifier flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyBinding {
    /// The main key
    pub key: VirtualKey,
    /// Ctrl/Control modifier
    pub ctrl: bool,
    /// Alt/Option modifier
    pub alt: bool,
    /// Shift modifier
    pub shift: bool,
    /// Meta/Command/Win/Super modifier
    pub meta: bool,
}

impl HotkeyBinding {
    /// Parse a hotkey string like "CTRL+SHIFT+A" or "ctrl shift a"
    /// 
    /// # Examples
    /// ```
    /// use keymagic_core::hotkey::HotkeyBinding;
    /// 
    /// let hotkey = HotkeyBinding::parse("CTRL+SHIFT+K").unwrap();
    /// assert_eq!(hotkey.ctrl, true);
    /// assert_eq!(hotkey.shift, true);
    /// assert_eq!(hotkey.key, VirtualKey::KeyK);
    /// ```
    pub fn parse(hotkey_str: &str) -> Result<Self> {
        if hotkey_str.trim().is_empty() {
            return Err(Error::ParseError("Empty hotkey string".to_string()));
        }

        // Split by + or space, trim each part, convert to uppercase
        let parts: Vec<String> = hotkey_str
            .split(|c| c == '+' || c == ' ')
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .collect();

        if parts.is_empty() {
            return Err(Error::ParseError("No valid components in hotkey string".to_string()));
        }

        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut meta = false;
        let mut key: Option<VirtualKey> = None;

        for part in parts {
            match part.as_str() {
                // Ctrl variants
                "CTRL" | "CONTROL" => ctrl = true,
                
                // Alt variants
                "ALT" | "OPTION" => alt = true,
                
                // Shift
                "SHIFT" => shift = true,
                
                // Meta variants
                "META" | "CMD" | "COMMAND" | "WIN" | "SUPER" => meta = true,
                
                // Otherwise, try to parse as a key
                _ => {
                    if key.is_some() {
                        return Err(Error::ParseError(format!("Multiple keys specified: {:?}", part)));
                    }
                    key = Some(parse_key(&part)?);
                }
            }
        }

        match key {
            Some(k) => Ok(HotkeyBinding {
                key: k,
                ctrl,
                alt,
                shift,
                meta,
            }),
            None => Err(Error::ParseError("No key specified in hotkey".to_string())),
        }
    }
}

/// Parse a key string to VirtualKey
fn parse_key(key_str: &str) -> Result<VirtualKey> {
    match key_str {
        // Single letter keys
        s if s.len() == 1 => {
            let ch = s.chars().next().unwrap();
            match ch {
                'A' => Ok(VirtualKey::KeyA),
                'B' => Ok(VirtualKey::KeyB),
                'C' => Ok(VirtualKey::KeyC),
                'D' => Ok(VirtualKey::KeyD),
                'E' => Ok(VirtualKey::KeyE),
                'F' => Ok(VirtualKey::KeyF),
                'G' => Ok(VirtualKey::KeyG),
                'H' => Ok(VirtualKey::KeyH),
                'I' => Ok(VirtualKey::KeyI),
                'J' => Ok(VirtualKey::KeyJ),
                'K' => Ok(VirtualKey::KeyK),
                'L' => Ok(VirtualKey::KeyL),
                'M' => Ok(VirtualKey::KeyM),
                'N' => Ok(VirtualKey::KeyN),
                'O' => Ok(VirtualKey::KeyO),
                'P' => Ok(VirtualKey::KeyP),
                'Q' => Ok(VirtualKey::KeyQ),
                'R' => Ok(VirtualKey::KeyR),
                'S' => Ok(VirtualKey::KeyS),
                'T' => Ok(VirtualKey::KeyT),
                'U' => Ok(VirtualKey::KeyU),
                'V' => Ok(VirtualKey::KeyV),
                'W' => Ok(VirtualKey::KeyW),
                'X' => Ok(VirtualKey::KeyX),
                'Y' => Ok(VirtualKey::KeyY),
                'Z' => Ok(VirtualKey::KeyZ),
                '0' => Ok(VirtualKey::Key0),
                '1' => Ok(VirtualKey::Key1),
                '2' => Ok(VirtualKey::Key2),
                '3' => Ok(VirtualKey::Key3),
                '4' => Ok(VirtualKey::Key4),
                '5' => Ok(VirtualKey::Key5),
                '6' => Ok(VirtualKey::Key6),
                '7' => Ok(VirtualKey::Key7),
                '8' => Ok(VirtualKey::Key8),
                '9' => Ok(VirtualKey::Key9),
                _ => Err(Error::ParseError(format!("Unknown key: {}", s))),
            }
        }
        
        // Special keys
        "SPACE" => Ok(VirtualKey::Space),
        "ENTER" | "RETURN" => Ok(VirtualKey::Return),
        "TAB" => Ok(VirtualKey::Tab),
        "BACKSPACE" | "BACK" | "DELETE" => Ok(VirtualKey::Back),
        "ESCAPE" | "ESC" => Ok(VirtualKey::Escape),
        "CAPSLOCK" | "CAPS" => Ok(VirtualKey::Capital),
        
        // Function keys
        "F1" => Ok(VirtualKey::F1),
        "F2" => Ok(VirtualKey::F2),
        "F3" => Ok(VirtualKey::F3),
        "F4" => Ok(VirtualKey::F4),
        "F5" => Ok(VirtualKey::F5),
        "F6" => Ok(VirtualKey::F6),
        "F7" => Ok(VirtualKey::F7),
        "F8" => Ok(VirtualKey::F8),
        "F9" => Ok(VirtualKey::F9),
        "F10" => Ok(VirtualKey::F10),
        "F11" => Ok(VirtualKey::F11),
        "F12" => Ok(VirtualKey::F12),
        
        // OEM keys
        "PLUS" | "=" => Ok(VirtualKey::OemPlus),
        "MINUS" | "-" => Ok(VirtualKey::OemMinus),
        "COMMA" | "," => Ok(VirtualKey::OemComma),
        "PERIOD" | "." => Ok(VirtualKey::OemPeriod),
        "SEMICOLON" | ";" => Ok(VirtualKey::Oem1),
        "SLASH" | "/" => Ok(VirtualKey::Oem2),
        "GRAVE" | "`" => Ok(VirtualKey::Oem3),
        "LEFTBRACKET" | "[" => Ok(VirtualKey::Oem4),
        "BACKSLASH" | "\\" => Ok(VirtualKey::Oem5),
        "RIGHTBRACKET" | "]" => Ok(VirtualKey::Oem6),
        "QUOTE" | "'" => Ok(VirtualKey::Oem7),
        
        // Unknown key
        _ => Err(Error::ParseError(format!("Unknown key: {}", key_str))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_hotkey() {
        let hotkey = HotkeyBinding::parse("ctrl+a").unwrap();
        assert_eq!(hotkey.key, VirtualKey::KeyA);
        assert_eq!(hotkey.ctrl, true);
        assert_eq!(hotkey.alt, false);
        assert_eq!(hotkey.shift, false);
        assert_eq!(hotkey.meta, false);
    }

    #[test]
    fn test_parse_multiple_modifiers() {
        let hotkey = HotkeyBinding::parse("CTRL+SHIFT+ALT+K").unwrap();
        assert_eq!(hotkey.key, VirtualKey::KeyK);
        assert_eq!(hotkey.ctrl, true);
        assert_eq!(hotkey.alt, true);
        assert_eq!(hotkey.shift, true);
        assert_eq!(hotkey.meta, false);
    }

    #[test]
    fn test_parse_space_separated() {
        let hotkey = HotkeyBinding::parse("ctrl shift k").unwrap();
        assert_eq!(hotkey.key, VirtualKey::KeyK);
        assert_eq!(hotkey.ctrl, true);
        assert_eq!(hotkey.shift, true);
    }

    #[test]
    fn test_parse_mixed_separators() {
        let hotkey = HotkeyBinding::parse("ctrl+shift k").unwrap();
        assert_eq!(hotkey.key, VirtualKey::KeyK);
        assert_eq!(hotkey.ctrl, true);
        assert_eq!(hotkey.shift, true);
    }

    #[test]
    fn test_parse_meta_variants() {
        let keys = vec!["meta+k", "cmd+k", "command+k", "win+k", "super+k"];
        for key_str in keys {
            let hotkey = HotkeyBinding::parse(key_str).unwrap();
            assert_eq!(hotkey.key, VirtualKey::KeyK);
            assert_eq!(hotkey.meta, true);
        }
    }

    #[test]
    fn test_parse_special_keys() {
        let hotkey = HotkeyBinding::parse("ctrl+space").unwrap();
        assert_eq!(hotkey.key, VirtualKey::Space);
        
        let hotkey = HotkeyBinding::parse("ctrl+enter").unwrap();
        assert_eq!(hotkey.key, VirtualKey::Return);
        
        let hotkey = HotkeyBinding::parse("ctrl+f1").unwrap();
        assert_eq!(hotkey.key, VirtualKey::F1);
    }

    #[test]
    fn test_parse_case_insensitive() {
        let hotkey1 = HotkeyBinding::parse("CTRL+SHIFT+A").unwrap();
        let hotkey2 = HotkeyBinding::parse("ctrl+shift+a").unwrap();
        let hotkey3 = HotkeyBinding::parse("Ctrl+Shift+A").unwrap();
        
        assert_eq!(hotkey1, hotkey2);
        assert_eq!(hotkey2, hotkey3);
    }

    #[test]
    fn test_parse_errors() {
        assert!(HotkeyBinding::parse("").is_err());
        assert!(HotkeyBinding::parse("ctrl+").is_err());
        assert!(HotkeyBinding::parse("ctrl+shift").is_err());
        assert!(HotkeyBinding::parse("ctrl+unknown").is_err());
        assert!(HotkeyBinding::parse("ctrl+a+b").is_err());
    }
}