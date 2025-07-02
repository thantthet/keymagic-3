use crate::types::{Rule, BinaryFormatElement, FLAG_ANYOF, FLAG_NANYOF};

/// Intermediate representation for pattern matching
/// This simplifies the matching logic by pre-processing complex patterns
#[derive(Debug, Clone)]
pub enum RuleElement {
    /// Match exact string
    String(String),
    
    /// Match entire variable content
    Variable(usize),
    
    /// Match any single character from variable
    VariableAnyOf(usize),
    
    /// Match any single character NOT in variable
    VariableNotAnyOf(usize),
    
    /// Variable with back-reference index (for output)
    /// The u16 is a back-reference index (e.g., $1) that provides the character index
    VariableWithBackRef(usize, u16),
    
    /// Virtual key with modifiers
    VirtualKey {
        key: u16,
        shift: bool,
        ctrl: bool,
        alt: bool,
        alt_gr: bool,
    },
    
    /// Match any single character
    Any,
    
    /// State requirement
    State(usize),
    
    /// Back-reference (for output)
    Reference(usize),
    
    /// State switch (for output)
    Switch(usize),
}

/// Preprocessed rule for efficient matching
#[derive(Debug, Clone)]
pub struct MatchRule {
    pub lhs: Vec<RuleElement>,
    pub rhs: Vec<RuleElement>,
    pub original_index: usize,
}

impl MatchRule {
    /// Convert a Rule to MatchRule by preprocessing complex patterns
    pub fn from_rule(rule: &Rule, index: usize) -> Self {
        let lhs = Self::preprocess_lhs(&rule.lhs);
        let rhs = Self::preprocess_rhs(&rule.rhs);
        
        Self {
            lhs,
            rhs,
            original_index: index,
        }
    }
    
    /// Preprocess LHS elements for pattern matching
    fn preprocess_lhs(elements: &[BinaryFormatElement]) -> Vec<RuleElement> {
        let mut result = Vec::new();
        let mut i = 0;
        
        // State for processing virtual key combinations
        enum State {
            Normal,
            InVirtualKey {
                shift: bool,
                ctrl: bool,
                alt: bool,
                alt_gr: bool,
                target_vk: Option<u16>,
            },
        }
        
        let mut state = State::Normal;
        
        while i < elements.len() {
            match (&state, &elements[i]) {
                // Normal state processing
                (State::Normal, BinaryFormatElement::String(s)) => {
                    result.push(RuleElement::String(s.clone()));
                }
                
                (State::Normal, BinaryFormatElement::Variable(idx)) => {
                    // Check if followed by modifier
                    if i + 1 < elements.len() {
                        if let BinaryFormatElement::Modifier(flags) = &elements[i + 1] {
                            match *flags {
                                FLAG_ANYOF => {
                                    result.push(RuleElement::VariableAnyOf(*idx));
                                    i += 1; // Skip the modifier
                                }
                                FLAG_NANYOF => {
                                    result.push(RuleElement::VariableNotAnyOf(*idx));
                                    i += 1; // Skip the modifier
                                }
                                _ => {
                                    // Other modifier - just add variable
                                    result.push(RuleElement::Variable(*idx));
                                }
                            }
                        } else {
                            result.push(RuleElement::Variable(*idx));
                        }
                    } else {
                        result.push(RuleElement::Variable(*idx));
                    }
                }
                
                (State::Normal, BinaryFormatElement::And) => {
                    // AND marks the start of a virtual key combination
                    // Next elements should be Predefined (VK)
                    state = State::InVirtualKey {
                        shift: false,
                        ctrl: false,
                        alt: false,
                        alt_gr: false,
                        target_vk: None,
                    };
                }
                
                (State::Normal, BinaryFormatElement::Any) => {
                    result.push(RuleElement::Any);
                }
                
                (State::Normal, BinaryFormatElement::Switch(idx)) => {
                    result.push(RuleElement::State(*idx));
                }
                
                // In virtual key combination state
                (State::InVirtualKey { shift, ctrl, alt, alt_gr, target_vk }, BinaryFormatElement::Predefined(vk)) => {
                    use crate::types::VirtualKey;
                    
                    // Update state based on the key
                    let mut new_shift = *shift;
                    let mut new_ctrl = *ctrl;
                    let mut new_alt = *alt;
                    let mut new_alt_gr = *alt_gr;
                    let mut new_target_vk = *target_vk;
                    
                    match *vk {
                        x if x == VirtualKey::Shift as u16 => new_shift = true,
                        x if x == VirtualKey::Control as u16 => new_ctrl = true,
                        x if x == VirtualKey::Menu as u16 => new_alt = true,
                        x if x == VirtualKey::LShift as u16 => new_shift = true,
                        x if x == VirtualKey::RShift as u16 => new_shift = true,
                        x if x == VirtualKey::LControl as u16 => new_ctrl = true,
                        x if x == VirtualKey::RControl as u16 => new_ctrl = true,
                        x if x == VirtualKey::LMenu as u16 => new_alt = true,
                        x if x == VirtualKey::RMenu as u16 => new_alt_gr = true,
                        _ => new_target_vk = Some(*vk),
                    }
                    
                    state = State::InVirtualKey {
                        shift: new_shift,
                        ctrl: new_ctrl,
                        alt: new_alt,
                        alt_gr: new_alt_gr,
                        target_vk: new_target_vk,
                    };
                }
                
                (State::InVirtualKey { .. }, BinaryFormatElement::And) => {
                    // AND in the middle of VK sequence should not happen
                    // End the current VK sequence and start a new one
                    if let State::InVirtualKey { shift, ctrl, alt, alt_gr, target_vk } = &state {
                        if let Some(key) = target_vk {
                            result.push(RuleElement::VirtualKey {
                                key: *key,
                                shift: *shift,
                                ctrl: *ctrl,
                                alt: *alt,
                                alt_gr: *alt_gr,
                            });
                        }
                    }
                    
                    // Start new VK sequence
                    state = State::InVirtualKey {
                        shift: false,
                        ctrl: false,
                        alt: false,
                        alt_gr: false,
                        target_vk: None,
                    };
                }
                
                // End of virtual key combination
                (State::InVirtualKey { shift, ctrl, alt, alt_gr, target_vk }, _) => {
                    // End of virtual key combination, emit VirtualKey if valid
                    if let Some(key) = target_vk {
                        result.push(RuleElement::VirtualKey {
                            key: *key,
                            shift: *shift,
                            ctrl: *ctrl,
                            alt: *alt,
                            alt_gr: *alt_gr,
                        });
                    }
                    
                    // Reset to normal state and reprocess current element
                    state = State::Normal;
                    continue;
                }
                
                // Standalone Predefined should not occur due to validation
                (State::Normal, BinaryFormatElement::Predefined(_)) => {
                    // This should not happen after validation
                    // Skip it
                }
                
                _ => {
                    // Skip other elements in LHS
                }
            }
            
            i += 1;
        }
        
        // Handle case where we end in InVirtualKey state
        if let State::InVirtualKey { shift, ctrl, alt, alt_gr, target_vk } = state {
            if let Some(key) = target_vk {
                result.push(RuleElement::VirtualKey {
                    key,
                    shift,
                    ctrl,
                    alt,
                    alt_gr,
                });
            }
        }
        
        result
    }
    
    /// Preprocess RHS elements
    fn preprocess_rhs(elements: &[BinaryFormatElement]) -> Vec<RuleElement> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < elements.len() {
            match &elements[i] {
                BinaryFormatElement::String(s) => {
                    result.push(RuleElement::String(s.clone()));
                }
                
                BinaryFormatElement::Variable(idx) => {
                    // Check if followed by modifier with back-reference index
                    if i + 1 < elements.len() {
                        if let BinaryFormatElement::Modifier(backref_idx) = &elements[i + 1] {
                            // In RHS, modifier after variable is a back-reference index
                            // It's not a flag like ANYOF/NANYOF
                            result.push(RuleElement::VariableWithBackRef(*idx, *backref_idx));
                            i += 1; // Skip the modifier
                        } else {
                            result.push(RuleElement::Variable(*idx));
                        }
                    } else {
                        result.push(RuleElement::Variable(*idx));
                    }
                }
                
                BinaryFormatElement::Reference(idx) => {
                    result.push(RuleElement::Reference(*idx));
                }
                
                BinaryFormatElement::Switch(idx) => {
                    result.push(RuleElement::Switch(*idx));
                }
                
                _ => {
                    // Skip other elements in RHS
                }
            }
            
            i += 1;
        }
        
        result
    }
}