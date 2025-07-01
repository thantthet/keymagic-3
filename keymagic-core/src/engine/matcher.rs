use crate::types::{Rule, BinaryFormatElement, Km2File};
use super::{KeyInput, EngineState};
use super::pattern::{RuleElement, MatchRule};

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
    rules: Vec<MatchRule>,
}

impl<'a> RuleMatcher<'a> {
    pub fn new(keyboard: &'a Km2File) -> Self {
        // Preprocess all rules
        let rules = keyboard.rules.iter()
            .enumerate()
            .map(|(i, rule)| MatchRule::from_rule(rule, i))
            .collect();
            
        Self { keyboard, rules }
    }

    /// Find the best matching rule for the current input
    pub fn find_match(
        &self, 
        input: &str, 
        key_input: Option<&KeyInput>,
        state: &EngineState
    ) -> Option<MatchResult> {
        let mut best_match: Option<MatchResult> = None;
        
        // Try to match each preprocessed rule
        for match_rule in &self.rules {
            if let Some(result) = self.try_match_rule(match_rule, input, key_input, state) {
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
        match_rule: &MatchRule,
        input: &str,
        key_input: Option<&KeyInput>,
        state: &EngineState,
    ) -> Option<MatchResult> {
        let mut captures = Vec::new();
        let mut input_pos = 0;
        
        // Match each pattern element
        for element in &match_rule.lhs {
            match element {
                RuleElement::State(state_idx) => {
                    // Check state requirement
                    if !state.is_state_active(*state_idx) {
                        return None;
                    }
                }
                
                RuleElement::String(s) => {
                    if !input[input_pos..].starts_with(s) {
                        return None;
                    }
                    input_pos += s.len();
                }
                
                RuleElement::Variable(var_idx) => {
                    if let Some(var_content) = self.get_string(*var_idx) {
                        if !input[input_pos..].starts_with(&var_content) {
                            return None;
                        }
                        input_pos += var_content.len();
                    } else {
                        return None;
                    }
                }
                
                RuleElement::VariableAnyOf(var_idx) => {
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
                
                RuleElement::VariableNotAnyOf(var_idx) => {
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
                
                RuleElement::Any => {
                    if let Some(ch) = input[input_pos..].chars().next() {
                        captures.push(ch.to_string());
                        input_pos += ch.len_utf8();
                    } else {
                        return None;
                    }
                }
                
                RuleElement::VirtualKey { key, shift, ctrl, alt, alt_gr } => {
                    // Virtual key rules only match on key input
                    if let Some(ki) = key_input {
                        if ki.vk_code as u16 != *key {
                            return None;
                        }
                        
                        // Check modifiers
                        if *shift && !ki.modifiers.shift {
                            return None;
                        }
                        if *ctrl && !ki.modifiers.ctrl {
                            return None;
                        }
                        if *alt && !ki.modifiers.alt {
                            return None;
                        }
                        if *alt_gr && !ki.modifiers.alt_gr {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                
                _ => {
                    // Other elements don't affect matching
                }
            }
        }
        
        // Get the original rule for the result
        let original_rule = &self.keyboard.rules[match_rule.original_index];
        
        Some(MatchResult {
            rule: original_rule.clone(),
            rule_index: match_rule.original_index,
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
                BinaryFormatElement::String(s) => {
                    output.push_str(s);
                }
                BinaryFormatElement::Variable(idx) => {
                    if let Some(var_value) = self.get_string(*idx) {
                        output.push_str(&var_value);
                    }
                }
                BinaryFormatElement::Reference(idx) => {
                    // Back-reference to captured segment
                    if *idx > 0 && *idx <= match_result.captures.len() {
                        output.push_str(&match_result.captures[*idx - 1]);
                    }
                }
                BinaryFormatElement::Switch(_) => {
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
                    lhs: vec![BinaryFormatElement::String("ka".to_string())],
                    rhs: vec![BinaryFormatElement::String("က".to_string())],
                },
                Rule {
                    lhs: vec![BinaryFormatElement::String("k".to_string())],
                    rhs: vec![BinaryFormatElement::String("က်".to_string())],
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