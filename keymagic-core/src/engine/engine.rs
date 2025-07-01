use crate::{
    Error, Result,
    km2::Km2Loader,
    types::{Km2File, RuleElement, VirtualKey},
};
use super::{EngineState, KeyInput, EngineOutput, matcher::RuleMatcher};

/// The main KeyMagic engine
pub struct KeyMagicEngine {
    /// Loaded keyboard layout
    keyboard: Option<Km2File>,
    /// Current engine state
    state: EngineState,
}

impl Default for KeyMagicEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyMagicEngine {
    /// Create a new engine instance
    pub fn new() -> Self {
        Self {
            keyboard: None,
            state: EngineState::new(),
        }
    }

    /// Load a keyboard layout from KM2 binary data
    pub fn load_keyboard(&mut self, data: &[u8]) -> Result<()> {
        let mut keyboard = Km2Loader::load(data)?;
        
        // Sort rules by priority for proper matching order
        self.sort_rules_by_priority(&mut keyboard);
        
        self.keyboard = Some(keyboard);
        self.state.reset();
        Ok(())
    }
    
    /// Sort rules by priority: state count, vk count, then total length
    fn sort_rules_by_priority(&self, keyboard: &mut Km2File) {
        keyboard.rules.sort_by(|a, b| {
            // First compare state count (rules with states have priority)
            let a_state_count = Self::count_states(&a.lhs);
            let b_state_count = Self::count_states(&b.lhs);
            if a_state_count != b_state_count {
                return b_state_count.cmp(&a_state_count);
            }
            
            // Then compare virtual key count
            let a_vk_count = Self::count_virtual_keys(&a.lhs);
            let b_vk_count = Self::count_virtual_keys(&b.lhs);
            if a_vk_count != b_vk_count {
                return b_vk_count.cmp(&a_vk_count);
            }
            
            // Finally compare total character length
            let a_length = Self::calculate_rule_length(&a.lhs, &keyboard.strings);
            let b_length = Self::calculate_rule_length(&b.lhs, &keyboard.strings);
            b_length.cmp(&a_length)
        });
    }
    
    /// Count state elements in a rule
    fn count_states(elements: &[RuleElement]) -> usize {
        elements.iter().filter(|e| matches!(e, RuleElement::Switch(_))).count()
    }
    
    /// Count virtual key elements in a rule
    fn count_virtual_keys(elements: &[RuleElement]) -> usize {
        elements.iter().filter(|e| matches!(e, RuleElement::Predefined(_))).count()
    }
    
    /// Calculate the total matching character length of a rule
    fn calculate_rule_length(elements: &[RuleElement], strings: &[crate::types::StringEntry]) -> usize {
        let mut length = 0;
        let mut i = 0;
        
        while i < elements.len() {
            match &elements[i] {
                RuleElement::String(s) => {
                    length += s.chars().count();
                }
                RuleElement::Variable(idx) => {
                    // Get variable string length
                    if *idx > 0 && *idx <= strings.len() {
                        length += strings[*idx - 1].value.chars().count();
                    }
                }
                RuleElement::AnyOf(_) | RuleElement::NotAnyOf(_) | RuleElement::Any => {
                    length += 1;
                }
                RuleElement::Predefined(_) => {
                    // Count (AND + Predefined)+ as 1
                    length += 1;
                    // Skip any following AND + Predefined combinations
                    while i + 2 < elements.len() {
                        if matches!(elements[i + 1], RuleElement::And) 
                            && matches!(elements[i + 2], RuleElement::Predefined(_)) {
                            i += 2;
                        } else {
                            break;
                        }
                    }
                }
                RuleElement::Switch(_) => {
                    // States don't contribute to length
                }
                _ => {}
            }
            i += 1;
        }
        
        length
    }

    /// Check if a keyboard is loaded
    pub fn has_keyboard(&self) -> bool {
        self.keyboard.is_some()
    }

    /// Get keyboard info (name, description, etc.)
    pub fn get_keyboard_info(&self) -> Option<KeyboardInfo> {
        self.keyboard.as_ref().map(|kb| {
            let mut info = KeyboardInfo::default();
            
            // Extract info from the info section
            for entry in &kb.info {
                match &entry.id {
                    b"eman" => { // "name" in little-endian
                        if let Ok(name) = String::from_utf8(entry.data.clone()) {
                            info.name = Some(name);
                        }
                    }
                    b"csed" => { // "desc" in little-endian
                        if let Ok(desc) = String::from_utf8(entry.data.clone()) {
                            info.description = Some(desc);
                        }
                    }
                    b"tnof" => { // "font" in little-endian
                        if let Ok(font) = String::from_utf8(entry.data.clone()) {
                            info.font_family = Some(font);
                        }
                    }
                    _ => {}
                }
            }
            
            info
        })
    }

    /// Process a key input event
    pub fn process_key_event(&mut self, input: KeyInput) -> Result<EngineOutput> {
        // Check if keyboard is loaded
        if self.keyboard.is_none() {
            return Err(Error::Engine("No keyboard loaded".into()));
        }

        // Handle special keys
        if input.vk_code == VirtualKey::Back {
            return self.handle_backspace();
        }

        // Convert key input to string if possible
        let input_str = if let Some(ch) = input.char_value {
            ch.to_string()
        } else {
            // For virtual keys without char value, try to match directly
            return self.process_virtual_key(&input);
        };

        // Append to composing buffer
        self.state.append_to_composing(&input_str);

        // Try to match rules
        let keyboard = self.keyboard.as_ref().unwrap();
        let matcher = RuleMatcher::new(keyboard);
        
        // Try matching with full composing buffer
        if let Some(match_result) = matcher.find_match(&self.state.composing_buffer, Some(&input), &self.state) {
            // Apply the matched rule
            let output = matcher.apply_rule(&match_result);
            
            // Clear the consumed input from composing buffer
            let remaining = self.state.composing_buffer[match_result.consumed_length..].to_string();
            self.state.composing_buffer = remaining;
            
            // Check for state switch in RHS
            if let Some(RuleElement::Switch(state_idx)) = match_result.rule.rhs.first() {
                self.state.toggle_state(*state_idx);
                return Ok(EngineOutput::pass_through());
            }
            
            // Apply recursive matching if needed
            let final_output = self.apply_recursive_matching(output)?;
            
            // Return the output
            if self.state.composing_buffer.is_empty() {
                Ok(EngineOutput::commit(final_output))
            } else {
                Ok(EngineOutput::commit(final_output)
                    .with_delete(match_result.consumed_length))
            }
        } else if keyboard.header.layout_options.eat != 0 {
            // Eat the key if no match and eat option is enabled
            self.state.clear_composing();
            Ok(EngineOutput::pass_through().with_delete(1))
        } else {
            // Update composing display
            Ok(EngineOutput::composing(self.state.composing_buffer.clone()))
        }
    }

    /// Handle backspace key
    fn handle_backspace(&mut self) -> Result<EngineOutput> {
        if self.state.composing_buffer.is_empty() {
            // Nothing to delete
            Ok(EngineOutput::pass_through())
        } else {
            // Remove last character from composing buffer
            self.state.backspace_composing(1);
            
            if self.state.composing_buffer.is_empty() {
                Ok(EngineOutput::pass_through().with_delete(1))
            } else {
                Ok(EngineOutput::composing(self.state.composing_buffer.clone()))
            }
        }
    }

    /// Process virtual key input
    fn process_virtual_key(&mut self, input: &KeyInput) -> Result<EngineOutput> {
        let keyboard = self.keyboard.as_ref().unwrap();
        let matcher = RuleMatcher::new(keyboard);
        
        // Try to match virtual key rules
        if let Some(match_result) = matcher.find_match("", Some(input), &self.state) {
            let output = matcher.apply_rule(&match_result);
            
            // Check for state switch
            if let Some(RuleElement::Switch(state_idx)) = match_result.rule.rhs.first() {
                self.state.toggle_state(*state_idx);
                return Ok(EngineOutput::pass_through());
            }
            
            Ok(EngineOutput::commit(output))
        } else if keyboard.header.layout_options.eat != 0 {
            Ok(EngineOutput::pass_through())
        } else {
            Ok(EngineOutput::pass_through())
        }
    }

    /// Apply recursive rule matching
    fn apply_recursive_matching(&self, mut output: String) -> Result<String> {
        // Check stop conditions
        if output.is_empty() {
            return Ok(output);
        }
        
        // Single ASCII printable character (excluding space)
        if output.len() == 1 {
            if let Some(ch) = output.chars().next() {
                if ch.is_ascii() && ch != ' ' && !ch.is_control() {
                    return Ok(output);
                }
            }
        }
        
        // Apply rules recursively
        let keyboard = self.keyboard.as_ref().unwrap();
        let matcher = RuleMatcher::new(keyboard);
        let mut max_iterations = 100; // Prevent infinite loops
        
        loop {
            if max_iterations == 0 {
                break;
            }
            max_iterations -= 1;
            
            if let Some(match_result) = matcher.find_match(&output, None, &self.state) {
                let new_output = matcher.apply_rule(&match_result);
                let remaining = output[match_result.consumed_length..].to_string();
                output = new_output + &remaining;
                
                // Check stop conditions again
                if output.is_empty() {
                    break;
                }
                if output.len() == 1 {
                    if let Some(ch) = output.chars().next() {
                        if ch.is_ascii() && ch != ' ' && !ch.is_control() {
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }
        
        Ok(output)
    }


    /// Get the current composing text
    pub fn get_composing(&self) -> &str {
        &self.state.composing_buffer
    }

    /// Reset the engine state
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Get the current engine state (for debugging)
    pub fn get_state(&self) -> &EngineState {
        &self.state
    }
}

/// Information about a loaded keyboard
#[derive(Debug, Clone, Default)]
pub struct KeyboardInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    pub font_family: Option<String>,
}