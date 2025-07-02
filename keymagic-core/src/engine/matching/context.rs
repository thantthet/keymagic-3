//! Matching context for rule evaluation

use crate::engine::input::KeyInput;
use crate::engine::types::Predefined;

/// Context for rule matching
#[derive(Debug, Clone)]
pub struct MatchContext<'a> {
    /// Current composing text
    pub composing_text: &'a str,
    /// Current key input (if any)
    pub key_input: Option<&'a KeyInput>,
    /// Active state indices
    pub active_states: &'a std::collections::HashSet<usize>,
    /// Whether this is a recursive match (no key input)
    pub is_recursive: bool,
}

impl<'a> MatchContext<'a> {
    /// Creates a context for initial key processing
    pub fn for_key_input(
        composing_text: &'a str,
        key_input: &'a KeyInput,
        active_states: &'a std::collections::HashSet<usize>,
    ) -> Self {
        Self {
            composing_text,
            key_input: Some(key_input),
            active_states,
            is_recursive: false,
        }
    }

    /// Creates a context for recursive matching
    pub fn for_recursive(
        composing_text: &'a str,
        active_states: &'a std::collections::HashSet<usize>,
    ) -> Self {
        Self {
            composing_text,
            key_input: None,
            active_states,
            is_recursive: true,
        }
    }

    /// Gets the virtual key code if available
    pub fn vk_code(&self) -> Option<Predefined> {
        self.key_input.map(|input| input.key_code)
    }

    /// Gets the character if available
    pub fn character(&self) -> Option<char> {
        self.key_input.and_then(|input| input.character)
    }

    /// Checks if modifiers match the input
    pub fn modifiers_match(&self, shift: bool, ctrl: bool, alt: bool) -> bool {
        if let Some(input) = self.key_input {
            input.modifiers.shift == shift
                && input.modifiers.ctrl == ctrl
                && input.modifiers.alt == alt
        } else {
            // No modifiers in recursive matching
            !shift && !ctrl && !alt
        }
    }
}