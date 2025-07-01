use crate::types::{Rule, RuleElement, Km2File, VirtualKey};
use super::{KeyInput, EngineState, ModifierState};

/// Rule matching result
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The matched rule
    pub rule: Rule,
    /// The index of the matched rule
    pub rule_index: usize,
    /// Captured segments for back-references
    pub captures: Vec<String>,
    /// Length of input consumed
    pub consumed_length: usize,
}

/// Rule matcher implementation
pub struct RuleMatcher<'a> {
    keyboard: &'a Km2File,
}

impl<'a> RuleMatcher<'a> {
    pub fn new(keyboard: &'a Km2File) -> Self {
        Self { keyboard }
    }

    /// Find the best matching rule for the current input
    pub fn find_match(
        &self, 
        input: &str, 
        key_input: Option<&KeyInput>,
        state: &EngineState
    ) -> Option<MatchResult> {
        let mut best_match: Option<MatchResult> = None;
        
        // Try to match each rule
        for (index, rule) in self.keyboard.rules.iter().enumerate() {
            if let Some(result) = self.try_match_rule(rule, index, input, key_input, state) {
                // Keep the match with longer consumed length (greedy matching)
                if best_match.as_ref().map_or(true, |m| result.consumed_length > m.consumed_length) {
                    best_match = Some(result);
                }
            }
        }
        
        best_match
    }

    /// Try to match a single rule
    fn try_match_rule(
        &self,
        rule: &Rule,
        rule_index: usize,
        input: &str,
        key_input: Option<&KeyInput>,
        state: &EngineState,
    ) -> Option<MatchResult> {
        let mut captures = Vec::new();
        let mut input_pos = 0;
        let mut element_index = 0;
        
        // Check all state requirements at the beginning of the rule
        while element_index < rule.lhs.len() {
            if let RuleElement::Switch(state_idx) = &rule.lhs[element_index] {
                if !state.is_state_active(*state_idx) {
                    return None;
                }
                element_index += 1;
            } else {
                break;
            }
        }
        
        // Match remaining LHS elements
        for element in &rule.lhs[element_index..] {
            match element {
                RuleElement::String(s) => {
                    if !input[input_pos..].starts_with(s) {
                        return None;
                    }
                    input_pos += s.len();
                }
                RuleElement::Predefined(vk) => {
                    // Virtual key rules only match on key input
                    if let Some(ki) = key_input {
                        // Check if this is part of a modifier combination
                        let mut check_index = element_index + 1;
                        let mut required_modifiers = ModifierState::new();
                        let mut target_vk = *vk;
                        
                        // Check for modifier combinations (VK_SHIFT & VK_KEY_A)
                        while check_index < rule.lhs.len() {
                            match &rule.lhs[check_index] {
                                RuleElement::And => {
                                    check_index += 1;
                                    if check_index < rule.lhs.len() {
                                        if let RuleElement::Predefined(next_vk) = &rule.lhs[check_index] {
                                            // Check if it's a modifier or a key
                                            // Since VirtualKey uses repr(u16), we can compare directly
                                            match *next_vk {
                                                x if x == VirtualKey::Shift as u16 => required_modifiers.shift = true,
                                                x if x == VirtualKey::Control as u16 => required_modifiers.ctrl = true,
                                                x if x == VirtualKey::Menu as u16 => required_modifiers.alt = true,
                                                x if x == VirtualKey::LShift as u16 => required_modifiers.shift = true,
                                                x if x == VirtualKey::RShift as u16 => required_modifiers.shift = true,
                                                x if x == VirtualKey::LControl as u16 => required_modifiers.ctrl = true,
                                                x if x == VirtualKey::RControl as u16 => required_modifiers.ctrl = true,
                                                x if x == VirtualKey::LMenu as u16 => required_modifiers.alt = true,
                                                x if x == VirtualKey::RMenu as u16 => required_modifiers.alt_gr = true,
                                                _ => target_vk = *next_vk,
                                            }
                                        }
                                    }
                                }
                                _ => break,
                            }
                            check_index += 1;
                        }
                        
                        // Now check if the input matches
                        if ki.vk_code as u16 != target_vk {
                            return None;
                        }
                        
                        // Check modifiers
                        if required_modifiers.shift && !ki.modifiers.shift {
                            return None;
                        }
                        if required_modifiers.ctrl && !ki.modifiers.ctrl {
                            return None;
                        }
                        if required_modifiers.alt && !ki.modifiers.alt {
                            return None;
                        }
                        
                        // Skip past the modifier combinations we've processed
                        element_index = check_index - 1;
                    } else {
                        return None;
                    }
                }
                RuleElement::Any => {
                    // Match any single character
                    if let Some(ch) = input[input_pos..].chars().next() {
                        captures.push(ch.to_string());
                        input_pos += ch.len_utf8();
                    } else {
                        return None;
                    }
                }
                RuleElement::AnyOf(var_idx) => {
                    // Match any character from the variable
                    if let Some(var_content) = self.get_string(*var_idx) {
                        if let Some(ch) = input[input_pos..].chars().next() {
                            if var_content.contains(ch) {
                                captures.push(ch.to_string());
                                input_pos += ch.len_utf8();
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                RuleElement::NotAnyOf(var_idx) => {
                    // Match any character NOT in the variable
                    if let Some(var_content) = self.get_string(*var_idx) {
                        if let Some(ch) = input[input_pos..].chars().next() {
                            if !var_content.contains(ch) {
                                captures.push(ch.to_string());
                                input_pos += ch.len_utf8();
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                RuleElement::Variable(var_idx) => {
                    // Match the entire variable content as a string
                    if let Some(var_content) = self.get_string(*var_idx) {
                        if !input[input_pos..].starts_with(&var_content) {
                            return None;
                        }
                        input_pos += var_content.len();
                    } else {
                        return None;
                    }
                }
                RuleElement::And => {
                    // AND is used for combining virtual keys, handled elsewhere
                    continue;
                }
                _ => {
                    // TODO: Handle other element types
                    return None;
                }
            }
        }
        
        Some(MatchResult {
            rule: rule.clone(),
            rule_index,
            captures,
            consumed_length: input_pos,
        })
    }

    /// Get a string from the string table by index (1-based)
    fn get_string(&self, index: usize) -> Option<String> {
        if index == 0 || index > self.keyboard.strings.len() {
            return None;
        }
        Some(self.keyboard.strings[index - 1].value.clone())
    }

    /// Apply the RHS of a matched rule to generate output
    pub fn apply_rule(&self, match_result: &MatchResult) -> String {
        let mut output = String::new();
        
        for element in &match_result.rule.rhs {
            match element {
                RuleElement::String(s) => {
                    output.push_str(s);
                }
                RuleElement::Variable(idx) => {
                    if let Some(var_value) = self.get_string(*idx) {
                        output.push_str(&var_value);
                    }
                }
                RuleElement::Reference(idx) => {
                    // Back-reference to captured segment
                    if *idx > 0 && *idx <= match_result.captures.len() {
                        output.push_str(&match_result.captures[*idx - 1]);
                    }
                }
                RuleElement::Switch(_) => {
                    // State switches don't produce output text
                }
                _ => {
                    // TODO: Handle other RHS elements
                }
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FileHeader, StringEntry};

    fn create_test_keyboard() -> Km2File {
        Km2File {
            header: FileHeader::new(),
            strings: vec![
                StringEntry { value: "abc".to_string() },
                StringEntry { value: "xyz".to_string() },
            ],
            info: vec![],
            rules: vec![
                Rule {
                    lhs: vec![RuleElement::String("ka".to_string())],
                    rhs: vec![RuleElement::String("က".to_string())],
                },
                Rule {
                    lhs: vec![RuleElement::String("k".to_string())],
                    rhs: vec![RuleElement::String("က်".to_string())],
                },
            ],
        }
    }

    #[test]
    fn test_greedy_matching() {
        let keyboard = create_test_keyboard();
        let matcher = RuleMatcher::new(&keyboard);
        let state = EngineState::new();
        
        // Should match "ka" => "က" instead of "k" => "က်"
        let result = matcher.find_match("ka", None, &state);
        assert!(result.is_some());
        
        let match_result = result.unwrap();
        assert_eq!(match_result.consumed_length, 2);
        assert_eq!(match_result.rule_index, 0);
    }
}