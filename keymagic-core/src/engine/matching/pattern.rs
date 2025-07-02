//! Pattern representation and preprocessing

use crate::engine::types::{Element, Predefined};
use crate::types::opcodes::{FLAG_ANYOF, FLAG_NANYOF};

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
    /// Virtual key
    VirtualKey(Predefined),
    /// Modifier key
    Modifier { shift: bool, ctrl: bool, alt: bool },
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
    pub fn from_elements(elements: &[Element]) -> Self {
        let mut pattern_elements = Vec::new();
        let mut char_length = 0;
        let mut state_count = 0;
        let mut vk_count = 0;
        let mut i = 0;

        while i < elements.len() {
            match &elements[i] {
                Element::String(s) => {
                    char_length += s.chars().count();
                    pattern_elements.push(PatternElement::String(s.clone()));
                }
                Element::Variable(idx) => {
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
                Element::Predefined(vk) => {
                    vk_count += 1;
                    pattern_elements.push(PatternElement::VirtualKey(*vk));
                }
                Element::Modifier(flags) => {
                    // Parse modifier flags
                    let shift = (*flags & 0x01) != 0;
                    let ctrl = (*flags & 0x02) != 0;
                    let alt = (*flags & 0x04) != 0;
                    pattern_elements.push(PatternElement::Modifier { shift, ctrl, alt });
                }
                Element::Any => {
                    char_length += 1;
                    pattern_elements.push(PatternElement::Any);
                }
                Element::Switch(state_idx) => {
                    state_count += 1;
                    pattern_elements.push(PatternElement::State(*state_idx));
                }
                Element::And => {
                    // AND is used to combine VK elements - skip it
                    // The actual VK element will follow
                }
                _ => {} // Skip other elements in LHS
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
                PatternElement::State(_) | PatternElement::VirtualKey(_) | PatternElement::Modifier { .. } => {
                    // These don't contribute to text length
                }
            }
        }

        Some(length)
    }
}