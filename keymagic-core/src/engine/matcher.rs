use crate::types::Km2File;
use super::{KeyInput, EngineState};
use super::pattern::{RuleElement, MatchRule};

/// Information about a captured match
#[derive(Debug, Clone)]
pub struct CaptureInfo {
    /// The matched character or string
    pub text: String,
    /// For variable matches, the position in the variable (None for other captures)
    pub position: Option<usize>,
}

impl CaptureInfo {
    fn new_text(text: String) -> Self {
        Self { text, position: None }
    }
    
    fn new_with_position(text: String, position: usize) -> Self {
        Self { text, position: Some(position) }
    }
}

/// Rule matching result
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The matched preprocessed rule
    pub rule: MatchRule,
    /// The index of the matched rule
    pub rule_index: usize,
    /// Captured segments for back-references
    pub captures: Vec<CaptureInfo>,
    /// Length of input consumed
    pub consumed_length: usize,
}

/// Rule matcher implementation
pub struct RuleMatcher<'a> {
    keyboard: &'a Km2File,
    pub(super) rules: Vec<MatchRule>,
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
            // Check if this rule has VirtualKey in LHS
            let has_virtual_key = match_rule.lhs.iter()
                .any(|e| matches!(e, RuleElement::VirtualKey { .. }));
            
            // Determine the input to use based on rule type
            let effective_input = if has_virtual_key {
                // For VK rules, use input as-is (don't append char)
                input.to_string()
            } else if let Some(ki) = key_input {
                // For non-VK rules, append char if available
                if let Some(ch) = ki.char_value {
                    format!("{}{}", input, ch)
                } else {
                    // No char available for non-VK rule
                    continue;
                }
            } else {
                // No key input for matching
                input.to_string()
            };
            
            if let Some(result) = self.try_match_rule(match_rule, &effective_input, key_input, state) {
                // Keep the match with longer consumed length (greedy matching)
                if best_match.as_ref().map_or(true, |m| result.consumed_length > m.consumed_length) {
                    best_match = Some(result);
                }
            }
        }
        
        best_match
    }
    
    /// Find the best matching rule and determine if char should be appended to composing buffer
    pub fn find_best_match(
        &self,
        composing: &str,
        key_input: Option<&KeyInput>,
        state: &EngineState
    ) -> Option<(MatchResult, bool)> {
        // find_match now handles the logic of whether to use char or VK
        if let Some(match_result) = self.find_match(composing, key_input, state) {
            // Check if this rule has VirtualKey in LHS
            let has_virtual_key = match_result.rule.lhs.iter()
                .any(|e| matches!(e, RuleElement::VirtualKey { .. }));
            
            // For VK rules, we don't append char to composing buffer
            // For non-VK rules, we do append char (already handled in find_match)
            let should_append_char = !has_virtual_key && key_input.and_then(|ki| ki.char_value).is_some();
            
            Some((match_result, should_append_char))
        } else {
            None
        }
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
                    // Capture the matched string
                    captures.push(CaptureInfo::new_text(s.clone()));
                    input_pos += s.len();
                }
                
                RuleElement::Variable(var_idx) => {
                    if let Some(var_content) = self.get_string(*var_idx) {
                        if !input[input_pos..].starts_with(&var_content) {
                            return None;
                        }
                        // Capture the entire variable content
                        captures.push(CaptureInfo::new_text(var_content.clone()));
                        input_pos += var_content.len();
                    } else {
                        return None;
                    }
                }
                
                RuleElement::VariableAnyOf(var_idx) => {
                    if let Some(var_content) = self.get_string(*var_idx) {
                        if let Some(ch) = input[input_pos..].chars().next() {
                            if let Some(position) = var_content.chars().position(|c| c == ch) {
                                // Capture both the character and its position
                                captures.push(CaptureInfo::new_with_position(ch.to_string(), position));
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
                                // For NOT matching, just capture the character
                                captures.push(CaptureInfo::new_text(ch.to_string()));
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
                        captures.push(CaptureInfo::new_text(ch.to_string()));
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
        
        Some(MatchResult {
            rule: match_rule.clone(),
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
                RuleElement::String(s) => {
                    output.push_str(s);
                }
                RuleElement::Variable(idx) => {
                    if let Some(var_value) = self.get_string(*idx) {
                        output.push_str(&var_value);
                    }
                }
                RuleElement::VariableWithBackRef(var_idx, backref_idx) => {
                    if let Some(var_value) = self.get_string(*var_idx) {
                        // Get the capture info from the back-reference
                        if *backref_idx > 0 && (*backref_idx as usize) <= match_result.captures.len() {
                            let capture = &match_result.captures[*backref_idx as usize - 1];
                            
                            // If the capture has a position, use it to index into the variable
                            if let Some(position) = capture.position {
                                if let Some(ch) = var_value.chars().nth(position) {
                                    output.push(ch);
                                }
                            } else {
                                // Otherwise, just use the captured text
                                output.push_str(&capture.text);
                            }
                        }
                    }
                }
                RuleElement::Reference(idx) => {
                    // Back-reference to captured segment
                    if *idx > 0 && *idx <= match_result.captures.len() {
                        output.push_str(&match_result.captures[*idx - 1].text);
                    }
                }
                RuleElement::Switch(_) => {
                    // State switches don't produce output text
                }
                _ => {
                    // Other elements don't produce output
                }
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FileHeader, StringEntry, BinaryFormatElement, Rule};

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