//! Engine state management

use std::collections::HashSet;
use super::buffer::ComposingBuffer;

/// Manages the state of the KeyMagic engine
#[derive(Debug, Clone)]
pub struct EngineState {
    /// Composing text buffer
    composing_buffer: ComposingBuffer,
    /// Active states (integer indices)
    active_states: HashSet<usize>,
}

impl EngineState {
    /// Creates a new engine state
    pub fn new() -> Self {
        Self {
            composing_buffer: ComposingBuffer::new(),
            active_states: HashSet::new(),
        }
    }

    /// Resets the engine state completely
    pub fn reset(&mut self) {
        self.composing_buffer.clear();
        self.active_states.clear();
    }

    /// Sets the composing text and resets states
    /// Used for external synchronization
    pub fn set_composing_text(&mut self, text: String) {
        self.composing_buffer = ComposingBuffer::from(text);
        self.active_states.clear();
    }

    /// Gets the current composing text
    pub fn composing_text(&self) -> &str {
        self.composing_buffer.as_str()
    }

    /// Gets a mutable reference to the composing buffer
    pub fn composing_buffer_mut(&mut self) -> &mut ComposingBuffer {
        &mut self.composing_buffer
    }


    /// Activates a state immediately
    pub fn activate_state(&mut self, state_index: usize) {
        self.active_states.insert(state_index);
    }

    /// Clears all active states
    pub fn clear_states(&mut self) {
        self.active_states.clear();
    }

    /// Gets the active states
    pub fn active_states(&self) -> &HashSet<usize> {
        &self.active_states
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}