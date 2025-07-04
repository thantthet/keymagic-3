//! Core rule matching logic

use crate::types::Rule;
use super::{Pattern, PatternElement, MatchContext, CaptureManager};
use super::pattern::VariableMatch;

/// Handles rule matching
pub struct RuleMatcher;

impl RuleMatcher {
    /// Finds the best matching rule for the given context
    /// Returns the matched rule, pattern, and captures
    pub fn find_match<'a>(
        rules: &'a [(Rule, Pattern)],
        context: &MatchContext,
        strings: &[String],
    ) -> Option<(&'a Rule, &'a Pattern, CaptureManager)> {
        for (rule, pattern) in rules {
            if let Some(captures) = Self::try_match_pattern(pattern, context, strings) {
                return Some((rule, pattern, captures));
            }
        }
        None
    }
    
    /// Tries to match a pattern against the context
    /// Returns captures
    fn try_match_pattern(
        pattern: &Pattern,
        context: &MatchContext,
        strings: &[String],
    ) -> Option<CaptureManager> {
        // Calculate the exact pattern length first
        let pattern_len = pattern.calculate_match_length(strings)?;
        
        // Determine what text to match against based on pattern type and context
        let text_to_match = if !context.is_recursive {
            if pattern.has_vk() {
                // Pattern has VK - match against composing text only, VK will be checked separately
                context.composing_text.to_string()
            } else {
                // Pattern has no VK - match against composing text + character
                let mut text = context.composing_text.to_string();
                if let Some(input) = context.key_input {
                    if let Some(ch) = input.character {
                        text.push(ch);
                    }
                }
                text
            }
        } else {
            // Recursive matching - only use composing text
            context.composing_text.to_string()
        };
        
        let text_chars: Vec<char> = text_to_match.chars().collect();
        
        // Early return if text is shorter than pattern
        if text_chars.len() < pattern_len {
            return None;
        }
        
        // Match pattern from the end of the text
        let start_pos = text_chars.len().saturating_sub(pattern_len);
        let mut captures = CaptureManager::new();
        let mut text_pos = start_pos;
        let mut match_success = true;
        
        for element in &pattern.elements {
            match element {
                PatternElement::State(state_idx) => {
                    // State must be active
                    if !context.active_states.contains(state_idx) {
                        match_success = false;
                        break;
                    }
                }
                PatternElement::VirtualKey(vk) => {
                    // Virtual keys only match in non-recursive context
                    if context.is_recursive {
                        match_success = false;
                        break;
                    }
                    // Convert VK code to Predefined for comparison
                    if let Some(vk_code) = context.vk_code() {
                        if vk_code != vk.raw() {
                            match_success = false;
                            break;
                        }
                    } else {
                        // No VK code in context
                        match_success = false;
                        break;
                    }
                }
                PatternElement::Modifier { shift, ctrl, alt } => {
                    // Modifiers only match in non-recursive context
                    if context.is_recursive {
                        match_success = false;
                        break;
                    }
                    if !context.modifiers_match(*shift, *ctrl, *alt) {
                        match_success = false;
                        break;
                    }
                }
                PatternElement::String(s) => {
                    // Match string from composing text
                    let s_chars: Vec<char> = s.chars().collect();
                    if text_pos + s_chars.len() > text_chars.len() {
                        match_success = false;
                        break;
                    }
                    
                    // Check if substring matches
                    let matched_str: String = text_chars[text_pos..text_pos + s_chars.len()].iter().collect();
                    if matched_str != *s {
                        match_success = false;
                        break;
                    }
                    
                    // Capture the matched string
                    captures.set_capture(captures.next_index(), matched_str);
                    text_pos += s_chars.len();
                }
                PatternElement::Variable(var_idx, var_match) => {
                    // Get variable content
                    let default_string = String::new();
                    let var_content = strings.get(*var_idx).unwrap_or(&default_string);
                    
                    match var_match {
                        VariableMatch::Exact => {
                            // Match entire variable content
                            let var_chars: Vec<char> = var_content.chars().collect();
                            if text_pos + var_chars.len() > text_chars.len() {
                                match_success = false;
                                break;
                            }
                            
                            // Check if the text matches the variable content
                            let matched_str: String = text_chars[text_pos..text_pos + var_chars.len()].iter().collect();
                            if matched_str != *var_content {
                                match_success = false;
                                break;
                            }
                            
                            // Capture the matched content
                            captures.set_capture(captures.next_index(), matched_str);
                            text_pos += var_chars.len();
                        }
                        VariableMatch::AnyOf => {
                            // Match one character from variable
                            if text_pos >= text_chars.len() {
                                match_success = false;
                                break;
                            }
                            
                            let ch = text_chars[text_pos];
                            if let Some(position) = var_content.chars().position(|c| c == ch) {
                                // Capture both the character and its position for Variable[$1] references
                                captures.set_capture_with_index(captures.next_index(), ch.to_string(), position);
                                text_pos += 1;
                            } else {
                                match_success = false;
                                break;
                            }
                        }
                        VariableMatch::NotAnyOf => {
                            // Match one character NOT in variable
                            if text_pos >= text_chars.len() {
                                match_success = false;
                                break;
                            }
                            
                            let ch = text_chars[text_pos];
                            if var_content.chars().any(|c| c == ch) {
                                match_success = false;
                                break;
                            }
                            
                            // Capture the character
                            captures.set_capture(captures.next_index(), ch.to_string());
                            text_pos += 1;
                        }
                    }
                }
                PatternElement::Any => {
                    // Match any printable ASCII character
                    if text_pos >= text_chars.len() {
                        match_success = false;
                        break;
                    }
                    
                    let ch = text_chars[text_pos];
                    if !is_printable_ascii(ch) {
                        match_success = false;
                        break;
                    }
                    
                    // Capture the character
                    captures.set_capture(captures.next_index(), ch.to_string());
                    text_pos += 1;
                }
            }
        }
        
        if match_success && text_pos == start_pos + pattern_len {
            // We found a match!
            return Some(captures);
        }
        
        None
    }
    
}

/// Checks if a character is printable ASCII (0x20-0x7E excluding space)
fn is_printable_ascii(ch: char) -> bool {
    matches!(ch, '!'..='~')
}