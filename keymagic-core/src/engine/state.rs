use std::collections::HashSet;

/// Represents the current state of the KeyMagic engine
#[derive(Debug, Clone)]
pub struct EngineState {
    /// Current composing string buffer
    pub composing_buffer: String,
    /// Active state indices (for state-based rules)
    pub active_states: HashSet<usize>,
    /// History of recent inputs for backspace handling
    pub input_history: Vec<String>,
    /// Maximum history size
    max_history: usize,
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            composing_buffer: String::new(),
            active_states: HashSet::new(),
            input_history: Vec::new(),
            max_history: 20,
        }
    }

    /// Reset the engine state
    pub fn reset(&mut self) {
        self.composing_buffer.clear();
        self.active_states.clear();
        self.input_history.clear();
    }

    /// Toggle a state (enter if not active, exit if active)
    pub fn toggle_state(&mut self, state_index: usize) {
        if self.active_states.contains(&state_index) {
            self.active_states.remove(&state_index);
        } else {
            self.active_states.insert(state_index);
        }
    }

    /// Check if a state is active
    pub fn is_state_active(&self, state_index: usize) -> bool {
        self.active_states.contains(&state_index)
    }

    /// Add to input history
    pub fn add_to_history(&mut self, input: String) {
        self.input_history.push(input);
        if self.input_history.len() > self.max_history {
            self.input_history.remove(0);
        }
    }

    /// Get the last n items from history
    pub fn get_recent_history(&self, n: usize) -> Vec<&str> {
        let start = self.input_history.len().saturating_sub(n);
        self.input_history[start..].iter().map(|s| s.as_str()).collect()
    }

    /// Clear composing buffer
    pub fn clear_composing(&mut self) {
        self.composing_buffer.clear();
    }

    /// Append to composing buffer
    pub fn append_to_composing(&mut self, text: &str) {
        self.composing_buffer.push_str(text);
    }

    /// Remove last n characters from composing buffer
    pub fn backspace_composing(&mut self, n: usize) {
        let new_len = self.composing_buffer.len().saturating_sub(n);
        self.composing_buffer.truncate(new_len);
    }
}