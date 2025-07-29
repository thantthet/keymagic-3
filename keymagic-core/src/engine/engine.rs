//! Main KeyMagic engine implementation

use crate::types::{Km2File, Rule};
use crate::engine::types::Element;
use crate::engine::{
    input::KeyInput,
    output::EngineOutput,
    state::EngineState,
    matching::{RuleMatcher, Pattern, MatchContext},
    processing::{RuleProcessor, RecursiveProcessor, ActionGenerator, should_stop_recursion},
};
use crate::error::Result;
use crate::VirtualKey;

/// Main KeyMagic engine for processing keyboard input
pub struct KeyMagicEngine {
    /// Loaded keyboard layout
    keyboard: Km2File,
    /// Engine state
    state: EngineState,
    /// Preprocessed rules with patterns
    rules: Vec<(Rule, Pattern)>,
    /// Extracted strings for faster access
    strings: Vec<String>,
    /// History of engine states for undo functionality
    state_history: Vec<EngineState>,
    /// Maximum number of states to keep in history
    max_history_size: usize,
}

impl KeyMagicEngine {
    /// Creates a new engine with the given keyboard layout
    pub fn new(keyboard: Km2File) -> Result<Self> {
        // Extract strings from StringEntry
        let strings: Vec<String> = keyboard.strings.iter()
            .map(|entry| entry.value.clone())
            .collect();
        
        // Preprocess and sort rules
        let mut rules = Self::preprocess_rules(&keyboard)?;
        Self::sort_rules(&mut rules);

        Ok(Self {
            keyboard,
            state: EngineState::new(),
            rules,
            strings,
            state_history: Vec::new(),
            max_history_size: 20,
        })
    }

    /// Processes a key input and returns the engine output
    pub fn process_key(&mut self, input: KeyInput) -> Result<EngineOutput> {
        Self::process_key_internal(&self.keyboard, &self.rules, &self.strings, input, &mut self.state, &mut self.state_history, self.max_history_size)
    }

    /// Processes a key input without modifying engine state (test/preview mode)
    pub fn process_key_test(&self, input: KeyInput) -> Result<EngineOutput> {
        let mut temp_state = self.state.clone();
        let mut temp_history = self.state_history.clone();
        Self::process_key_internal(&self.keyboard, &self.rules, &self.strings, input, &mut temp_state, &mut temp_history, self.max_history_size)
    }

    /// Internal key processing that works with a mutable state reference
    fn process_key_internal(keyboard: &Km2File, rules: &[(Rule, Pattern)], strings: &[String], input: KeyInput, state: &mut EngineState, state_history: &mut Vec<EngineState>, max_history_size: usize) -> Result<EngineOutput> {
        // Store initial state for action generation
        let before_text = state.composing_text().to_string();
        
        // Save the state BEFORE processing (for undo functionality)
        let state_before_processing = state.clone();

        // Create match context
        let context = MatchContext::for_key_input(
            state.composing_text(),
            &input,
            state.active_states(),
        );

        // Track whether a rule was matched (input was processed)
        let is_processed: bool;

        // Try to find a matching rule
        if let Some((rule, pattern, captures)) = RuleMatcher::find_match(rules, &context, strings) {
            // A rule matched, so the input was processed
            is_processed = true;

            // Calculate the matched length from the pattern
            let matched_len = pattern.calculate_match_length(strings).unwrap_or(0);

            // Clear active states
            state.clear_states();

            // Apply the matched rule
            let output = RuleProcessor::apply_rule(
                rule,
                state,
                &captures,
                strings,
            )?;

            // Append conetxt char to the composing buffer if rule has no VK
            if !pattern.has_vk() {
                if let Some(ch) = input.character {
                    state.composing_buffer_mut().append(&ch.to_string());
                }
            }

            // Update composing buffer by replacing only the matched portion
            state.composing_buffer_mut().replace_from_end(matched_len, &output);

            // Check if we should stop recursion based on the output
            if !should_stop_recursion(&output) {
                // Process recursive rules
                RecursiveProcessor::process_recursive(
                    state,
                    rules,
                    strings,
                )?;
            }
        } else {
            // No rule matched
            
            // Check if this is a backspace key
            if input.key_code == VirtualKey::Back as u16
                && !state.composing_text().is_empty() {
                // Backspace key pressed, and composing buffer is not empty
                if keyboard.header.layout_options.auto_bksp == 1 {
                    if !state_history.is_empty() {
                        // Restore from history (undo-like behavior)
                        *state = state_history.pop().unwrap();
                    } else {
                        // No history, delete one character backward
                        state.composing_buffer_mut().backspace();
                    }
                    is_processed = true;
                } else {
                    // auto_bksp is disabled, process backspace as simple delete
                    state.composing_buffer_mut().backspace();
                    is_processed = true;
                }
            } else if let Some(ch) = input.character {
                // if character is available, set is_processed to true
                is_processed = true;

                // append character if available & not eat_all_unused_keys
                if keyboard.header.layout_options.eat == 0 {
                    state.composing_buffer_mut().append(&ch.to_string());
                } else {
                    // key is processed and eaten
                }
            } else {
                // if no character, set is_processed to false
                is_processed = false;
            }

            // Clear active states
            state.clear_states();
        }

        // Generate output action
        let after_text = state.composing_text().to_string();
        let action = ActionGenerator::generate_action(&before_text, &after_text, true);

        // Record state in history (but not for backspace operations)
        if input.key_code != VirtualKey::Back as u16 && is_processed {
            state_history.push(state_before_processing);
            
            // Maintain max history size
            if state_history.len() > max_history_size {
                state_history.remove(0);
            }
        }

        Ok(EngineOutput::new(after_text, action, is_processed))
    }

    /// Resets the engine state
    pub fn reset(&mut self) {
        self.state.reset();
        self.state_history.clear();
    }

    /// Sets the composing text and resets states
    /// Used for external synchronization
    pub fn set_composing_text(&mut self, text: String) {
        self.state.set_composing_text(text);
        self.state_history.clear();
    }

    /// Gets the current composing text
    pub fn composing_text(&self) -> &str {
        self.state.composing_text()
    }

    /// Gets the loaded keyboard layout
    pub fn keyboard(&self) -> &Km2File {
        &self.keyboard
    }

    /// Preprocesses rules into patterns for efficient matching
    fn preprocess_rules(keyboard: &Km2File) -> Result<Vec<(Rule, Pattern)>> {
        keyboard.rules
            .iter()
            .map(|rule| {
                // Convert binary elements to engine elements
                let lhs_elements: Vec<Element> = rule.lhs.iter()
                    .map(|e| e.clone().into())
                    .collect();
                let pattern = Pattern::from_elements(&lhs_elements);
                Ok((rule.clone(), pattern))
            })
            .collect()
    }

    /// Sorts rules by priority (state > VK > length)
    fn sort_rules(rules: &mut [(Rule, Pattern)]) {
        rules.sort_by(|a, b| {
            if a.1.has_priority_over(&b.1) {
                std::cmp::Ordering::Less
            } else if b.1.has_priority_over(&a.1) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::output::ActionType;

    #[test]
    fn test_engine_creation() {
        let keyboard = Km2File::default();
        let engine = KeyMagicEngine::new(keyboard);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_simple_key_processing() {
        let keyboard = Km2File::default();
        let mut engine = KeyMagicEngine::new(keyboard).unwrap();
        
        let input = KeyInput::from_char('a');
        let output = engine.process_key(input).unwrap();
        
        assert_eq!(output.composing_text, "a");
        assert_eq!(output.action, ActionType::Insert("a".to_string()));
    }

    #[test]
    fn test_reset() {
        let keyboard = Km2File::default();
        let mut engine = KeyMagicEngine::new(keyboard).unwrap();
        
        // Add some text
        engine.process_key(KeyInput::from_char('a')).unwrap();
        assert_eq!(engine.composing_text(), "a");
        
        // Reset
        engine.reset();
        assert_eq!(engine.composing_text(), "");
    }

    #[test]
    fn test_set_composing_text() {
        let keyboard = Km2File::default();
        let mut engine = KeyMagicEngine::new(keyboard).unwrap();
        
        engine.set_composing_text("test".to_string());
        assert_eq!(engine.composing_text(), "test");
    }

    #[test]
    fn test_process_key_test_does_not_modify_state() {
        let keyboard = Km2File::default();
        let mut engine = KeyMagicEngine::new(keyboard).unwrap();
        
        // Set initial state
        engine.process_key(KeyInput::from_char('a')).unwrap();
        assert_eq!(engine.composing_text(), "a");
        
        // Test mode should not modify state
        let test_output = engine.process_key_test(KeyInput::from_char('b')).unwrap();
        assert_eq!(test_output.composing_text, "ab");
        
        // Original state should be unchanged
        assert_eq!(engine.composing_text(), "a");
        
        // Normal processing should still work
        let normal_output = engine.process_key(KeyInput::from_char('b')).unwrap();
        assert_eq!(normal_output.composing_text, "ab");
        assert_eq!(engine.composing_text(), "ab");
    }

    #[test]
    fn test_process_key_test_same_result_as_normal() {
        let keyboard = Km2File::default();
        let mut engine1 = KeyMagicEngine::new(keyboard.clone()).unwrap();
        let mut engine2 = KeyMagicEngine::new(keyboard).unwrap();
        
        // Set same initial state
        engine1.process_key(KeyInput::from_char('a')).unwrap();
        engine2.process_key(KeyInput::from_char('a')).unwrap();
        
        // Test key
        let input = KeyInput::from_char('b');
        
        // Get test result
        let test_output = engine1.process_key_test(input.clone()).unwrap();
        
        // Get normal result
        let normal_output = engine2.process_key(input).unwrap();
        
        // Results should be identical
        assert_eq!(test_output.composing_text, normal_output.composing_text);
        assert_eq!(test_output.action, normal_output.action);
        assert_eq!(test_output.is_processed, normal_output.is_processed);
    }

}