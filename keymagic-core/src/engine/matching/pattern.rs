//! Pattern representation and preprocessing

use crate::engine::types::Element;
use crate::types::opcodes::{FLAG_ANYOF, FLAG_NANYOF};
use crate::VirtualKey;

/// Preprocessed pattern for efficient matching
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Pattern elements
    pub elements: Vec<PatternElement>,
    /// Total character length (for sorting)
    pub char_length: usize,
    /// Number of state conditions
    pub state_count: usize,
    /// Number of virtual key conditions
    pub vk_count: usize,
}

/// Individual pattern element
#[derive(Debug, Clone)]
pub enum PatternElement {
    /// Literal string to match
    String(String),
    /// Variable content
    Variable(usize, VariableMatch),
    /// Virtual key(s) - can be a combination like Shift+A
    VirtualKey(Vec<VirtualKey>),
    /// State condition
    State(usize),
    /// Match any printable ASCII character
    Any,
}

/// Variable matching type
#[derive(Debug, Clone)]
pub enum VariableMatch {
    /// Match entire variable content
    Exact,
    /// Match any character in variable
    AnyOf,
    /// Match any character NOT in variable
    NotAnyOf,
}

impl Pattern {
    /// Creates a pattern from rule elements
    /// Validates that Element::Predefined only appears after Element::And
    pub fn from_elements(elements: &[Element]) -> Self {
        let mut pattern_elements = Vec::new();
        let mut char_length = 0;
        let mut state_count = 0;
        let mut vk_count = 0;
        let mut i = 0;
        let mut expecting_vk_after_and = false;

        while i < elements.len() {
            match &elements[i] {
                Element::String(s) => {
                    expecting_vk_after_and = false;
                    char_length += s.chars().count();
                    pattern_elements.push(PatternElement::String(s.clone()));
                }
                Element::Variable(idx) => {
                    expecting_vk_after_and = false;
                    // Check for modifiers after variable
                    let var_match = if i + 1 < elements.len() {
                        match &elements[i + 1] {
                            Element::Modifier(m) if *m == FLAG_ANYOF => {
                                i += 1; // Skip modifier
                                VariableMatch::AnyOf
                            }
                            Element::Modifier(m) if *m == FLAG_NANYOF => {
                                i += 1; // Skip modifier
                                VariableMatch::NotAnyOf
                            }
                            _ => VariableMatch::Exact,
                        }
                    } else {
                        VariableMatch::Exact
                    };
                    
                    // For wildcards, count as 1 character
                    if matches!(var_match, VariableMatch::AnyOf | VariableMatch::NotAnyOf) {
                        char_length += 1;
                    }
                    
                    pattern_elements.push(PatternElement::Variable(*idx, var_match));
                }
                Element::Predefined(_) => {
                    // Validate that Predefined only appears after And
                    if !expecting_vk_after_and {
                        // Invalid: Predefined without preceding And
                        // Skip this element as it's invalid in LHS
                        i += 1;
                        continue;
                    }
                    // Don't process here - will be handled by AND case
                    i += 1;
                    continue;
                }
                Element::Any => {
                    expecting_vk_after_and = false;
                    char_length += 1;
                    pattern_elements.push(PatternElement::Any);
                }
                Element::Switch(state_idx) => {
                    expecting_vk_after_and = false;
                    state_count += 1;
                    pattern_elements.push(PatternElement::State(*state_idx));
                }
                Element::And => {
                    // AND is used to combine VK elements
                    // Collect all subsequent Predefined elements
                    let mut vks = Vec::new();
                    let mut j = i + 1;
                    
                    while j < elements.len() {
                        if let Element::Predefined(vk) = &elements[j] {
                            vks.push(*vk);
                            j += 1;
                        } else {
                            break;
                        }
                    }
                    
                    if !vks.is_empty() {
                        // Convert Predefined to VirtualKey and validate
                        let mut virtual_keys = Vec::new();
                        
                        for vk in &vks {
                            if let Some(virtual_key) = VirtualKey::from_raw(vk.raw()) {
                                virtual_keys.push(virtual_key);
                            }
                        }
                        
                        vk_count += virtual_keys.len();
                        pattern_elements.push(PatternElement::VirtualKey(virtual_keys));
                        
                        i = j - 1; // Skip all processed VK elements
                    }
                    
                    expecting_vk_after_and = true;
                }
                Element::Modifier(_) => {
                    // invalid: modifier should be preceded by a variable
                    i += 1;
                    continue;
                }
                _ => {
                    expecting_vk_after_and = false;
                    // Skip other elements in LHS
                }
            }
            i += 1;
        }

        Self {
            elements: pattern_elements,
            char_length,
            state_count,
            vk_count,
        }
    }

    /// Checks if this pattern should be checked before another
    /// Returns true if this pattern has higher priority
    pub fn has_priority_over(&self, other: &Self) -> bool {
        // 1. State-specific rules have priority
        if self.state_count != other.state_count {
            return self.state_count > other.state_count;
        }

        // 2. Virtual key rules have priority
        if self.vk_count != other.vk_count {
            return self.vk_count > other.vk_count;
        }

        // 3. Longer patterns have priority
        self.char_length > other.char_length
    }

    /// Returns true if this pattern contains any virtual key elements
    pub fn has_vk(&self) -> bool {
        self.vk_count > 0
    }

    /// Calculates the exact character length this pattern will match
    pub fn calculate_match_length(&self, strings: &[String]) -> Option<usize> {
        let mut length = 0;

        for element in &self.elements {
            match element {
                PatternElement::String(s) => {
                    length += s.chars().count();
                }
                PatternElement::Variable(var_idx, var_match) => {
                    match var_match {
                        VariableMatch::Exact => {
                            // Get variable content length
                            let var_content = strings.get(*var_idx)?;
                            length += var_content.chars().count();
                        }
                        VariableMatch::AnyOf | VariableMatch::NotAnyOf => {
                            // These match exactly one character
                            length += 1;
                        }
                    }
                }
                PatternElement::Any => {
                    // Matches exactly one character
                    length += 1;
                }
                PatternElement::State(_) | PatternElement::VirtualKey(_) => {
                    // These don't contribute to text length
                }
            }
        }

        Some(length)
    }
}