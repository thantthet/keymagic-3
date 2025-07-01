use crate::types::VirtualKey;

/// Represents a key input event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyInput {
    /// Virtual key code
    pub vk_code: VirtualKey,
    /// Modifier state
    pub modifiers: ModifierState,
    /// Character value (if available)
    pub char_value: Option<char>,
}

/// Modifier key state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub alt_gr: bool,
    pub caps_lock: bool,
}

impl ModifierState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_alt_gr(mut self) -> Self {
        self.alt_gr = true;
        self
    }

    pub fn with_caps_lock(mut self) -> Self {
        self.caps_lock = true;
        self
    }

    /// Check if Ctrl+Alt should be treated as AltGr
    pub fn is_alt_gr(&self, treat_ctrl_alt_as_ralt: bool) -> bool {
        self.alt_gr || (treat_ctrl_alt_as_ralt && self.ctrl && self.alt)
    }
}

impl KeyInput {
    pub fn new(vk_code: VirtualKey, modifiers: ModifierState) -> Self {
        Self {
            vk_code,
            modifiers,
            char_value: None,
        }
    }

    pub fn with_char(mut self, ch: char) -> Self {
        self.char_value = Some(ch);
        self
    }
}