use anyhow::Result;

pub struct HotkeyManager;

impl HotkeyManager {
    pub fn new() -> Self {
        Self
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

    /// Parse a hotkey string to validate its format
    fn parse_hotkey(&self, hotkey_str: &str) -> Result<()> {
        let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();
        
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Hotkey cannot be empty"));
        }
        
        let mut has_modifier = false;
        let mut has_key = false;
        let mut non_modifier_keys = Vec::new();
        
        for part in parts {
            if part.is_empty() {
                return Err(anyhow::anyhow!("Invalid hotkey format: contains empty key (consecutive '+' signs?)"));
            }
            
            match part.to_uppercase().as_str() {
                "CTRL" | "CONTROL" | "ALT" | "SHIFT" | "META" | "WIN" | "SUPER" | "CMD" => {
                    has_modifier = true;
                }
                key => {
                    non_modifier_keys.push(part);
                    if has_key {
                        return Err(anyhow::anyhow!(
                            "Hotkey can only contain one non-modifier key. Found multiple: {}",
                            non_modifier_keys.join(", ")
                        ));
                    }
                    has_key = true;
                    self.validate_key_code(key)?;
                }
            }
        }
        
        if !has_key {
            return Err(anyhow::anyhow!("Hotkey must include at least one non-modifier key (not just Ctrl/Alt/Shift/Win)"));
        }
        
        Ok(())
    }

    /// Validate a key code
    fn validate_key_code(&self, key: &str) -> Result<()> {
        match key.to_uppercase().as_str() {
            // Letters
            "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" |
            "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" |
            
            // Numbers
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" |
            
            // Function keys
            "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9" | "F10" | "F11" | "F12" |
            
            // Special keys
            "SPACE" | "TAB" | "ENTER" | "RETURN" | "ESCAPE" | "ESC" | "BACKSPACE" | "DELETE" |
            "HOME" | "END" | "PAGEUP" | "PGUP" | "PAGEDOWN" | "PGDOWN" |
            "LEFT" | "RIGHT" | "UP" | "DOWN" | "INSERT" |
            
            // Punctuation and symbols
            "MINUS" | "-" | "EQUAL" | "=" | "BRACKET_LEFT" | "[" | "BRACKET_RIGHT" | "]" |
            "BACKSLASH" | "\\" | "SEMICOLON" | ";" | "QUOTE" | "'" | "BACKQUOTE" | "`" |
            "COMMA" | "," | "PERIOD" | "." | "SLASH" | "/" => Ok(()),
            
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
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}