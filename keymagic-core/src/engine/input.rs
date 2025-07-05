//! Input representation for the KeyMagic engine

/// Represents a keyboard input event
#[derive(Debug, Clone, PartialEq)]
pub struct KeyInput {
    /// Virtual key code (internal key code)
    pub key_code: u16,
    /// Modifier keys state
    pub modifiers: ModifierState,
    /// Character representation (if any)
    pub character: Option<char>,
}

impl KeyInput {
    /// Creates a new keyboard input
    pub fn new(key_code: u16, modifiers: ModifierState, character: Option<char>) -> Self {
        Self {
            key_code,
            modifiers,
            character,
        }
    }

    /// Creates a simple character input without modifiers
    pub fn from_char(ch: char) -> Self {
        Self {
            key_code: 0, // No VK code for character-only input
            modifiers: ModifierState::default(),
            character: Some(ch),
        }
    }

    /// Creates a virtual key input
    pub fn from_vk(key_code: u16, modifiers: ModifierState) -> Self {
        Self {
            key_code,
            modifiers,
            character: None,
        }
    }
}

/// State of modifier keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub caps_lock: bool,
}

impl ModifierState {
    /// Creates a new modifier state
    pub fn new(shift: bool, ctrl: bool, alt: bool, caps_lock: bool) -> Self {
        Self {
            shift,
            ctrl,
            alt,
            caps_lock,
        }
    }

    /// Checks if any modifier is active
    pub fn any(&self) -> bool {
        self.shift || self.ctrl || self.alt
    }

    /// Checks if no modifiers are active
    pub fn none(&self) -> bool {
        !self.any()
    }
}