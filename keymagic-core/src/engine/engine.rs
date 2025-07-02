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
        })
    }

    /// Processes a key input and returns the engine output
    pub fn process_key(&mut self, input: KeyInput) -> Result<EngineOutput> {
        // Store initial state for action generation
        let before_text = self.state.composing_text().to_string();

        // Create match context
        let context = MatchContext::for_key_input(
            self.state.composing_text(),
            &input,
            self.state.active_states(),
        );

        // Try to find a matching rule
        if let Some((rule, pattern, captures)) = RuleMatcher::find_match(&self.rules, &context, &self.strings) {
            // Calculate the matched length from the pattern
            let matched_len = pattern.calculate_match_length(&self.strings).unwrap_or(0);

            // Clear active states
            self.state.clear_states();

            // Apply the matched rule
            let output = RuleProcessor::apply_rule(
                rule,
                &mut self.state,
                &captures,
                &self.strings,
            )?;

            // Append conetxt char to the composing buffer if rule has no VK
            if !pattern.has_vk() {
                if let Some(ch) = input.character {
                    self.state.composing_buffer_mut().append(&ch.to_string());
                }
            }

            // Update composing buffer by replacing only the matched portion
            self.state.composing_buffer_mut().replace_from_end(matched_len, &output);

            // Check if we should stop recursion based on the output
            if !should_stop_recursion(&output) {
                // Process recursive rules
                RecursiveProcessor::process_recursive(
                    &mut self.state,
                    &self.rules,
                    &self.strings,
                )?;
            }
        } else {
            // No rule matched

            // append character if available & not eat_all_unused_keys
            if self.keyboard.header.layout_options.eat == 0 {
                if let Some(ch) = input.character {
                    self.state.composing_buffer_mut().append(&ch.to_string());
                }
            }

            // Clear active states
            self.state.clear_states();
        }

        // Generate output action
        let after_text = self.state.composing_text().to_string();
        let action = ActionGenerator::generate_action(&before_text, &after_text, true);

        Ok(EngineOutput::new(after_text, action))
    }

    /// Resets the engine state
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Sets the composing text and resets states
    /// Used for external synchronization
    pub fn set_composing_text(&mut self, text: String) {
        self.state.set_composing_text(text);
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
}