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
        
        while i < elements.len() {
            match &elements[i] {
                BinaryFormatElement::String(s) => {
                    result.push(RuleElement::String(s.clone()));
                }
                
                BinaryFormatElement::Variable(idx) => {
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
                
                BinaryFormatElement::Predefined(vk) => {
                    // Check if this is preceded by AND (part of a virtual key combination)
                    let preceded_by_and = i > 0 && matches!(elements[i - 1], BinaryFormatElement::And);
                    
                    if preceded_by_and {
                        // This is part of a combination already being processed, skip it
                    } else {
                        // Check if this starts a virtual key combination (followed by AND)
                        let starts_combination = i + 1 < elements.len() && matches!(elements[i + 1], BinaryFormatElement::And);
                        
                        if starts_combination {
                            // Process the entire virtual key combination
                            let mut shift = false;
                            let mut ctrl = false;
                            let mut alt = false;
                            let mut alt_gr = false;
                            let mut target_vk = None;
                            
                            // Process first key
                            use crate::types::VirtualKey;
                            match *vk {
                                x if x == VirtualKey::Shift as u16 => shift = true,
                                x if x == VirtualKey::Control as u16 => ctrl = true,
                                x if x == VirtualKey::Menu as u16 => alt = true,
                                x if x == VirtualKey::LShift as u16 => shift = true,
                                x if x == VirtualKey::RShift as u16 => shift = true,
                                x if x == VirtualKey::LControl as u16 => ctrl = true,
                                x if x == VirtualKey::RControl as u16 => ctrl = true,
                                x if x == VirtualKey::LMenu as u16 => alt = true,
                                x if x == VirtualKey::RMenu as u16 => alt_gr = true,
                                _ => target_vk = Some(*vk),
                            }
                            
                            // Look ahead for AND + more keys
                            let mut j = i + 1;
                            while j < elements.len() && matches!(elements[j], BinaryFormatElement::And) {
                                if j + 1 < elements.len() {
                                    if let BinaryFormatElement::Predefined(key) = &elements[j + 1] {
                                        match *key {
                                            x if x == VirtualKey::Shift as u16 => shift = true,
                                            x if x == VirtualKey::Control as u16 => ctrl = true,
                                            x if x == VirtualKey::Menu as u16 => alt = true,
                                            x if x == VirtualKey::LShift as u16 => shift = true,
                                            x if x == VirtualKey::RShift as u16 => shift = true,
                                            x if x == VirtualKey::LControl as u16 => ctrl = true,
                                            x if x == VirtualKey::RControl as u16 => ctrl = true,
                                            x if x == VirtualKey::LMenu as u16 => alt = true,
                                            x if x == VirtualKey::RMenu as u16 => alt_gr = true,
                                            _ => target_vk = Some(*key),
                                        }
                                        j += 2;
                                        i = j - 1; // Skip processed elements
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            
                            // Only create VirtualKey if we have a non-modifier key
                            if let Some(key) = target_vk {
                                result.push(RuleElement::VirtualKey {
                                    key,
                                    shift,
                                    ctrl,
                                    alt,
                                    alt_gr,
                                });
                            }
                            // If no target key (only modifiers), skip creating VirtualKey
                        } else {
                            // Standalone Predefined - treat as string
                            // Convert the virtual key code to a string
                            result.push(RuleElement::String((*vk as u8 as char).to_string()));
                        }
                    }
                }
                
                BinaryFormatElement::Any => {
                    result.push(RuleElement::Any);
                }
                
                BinaryFormatElement::Switch(idx) => {
                    result.push(RuleElement::State(*idx));
                }
                
                BinaryFormatElement::And => {
                    // Check if this is the start of a virtual key combination
                    if i + 1 < elements.len() {
                        if let BinaryFormatElement::Predefined(_vk) = &elements[i + 1] {
                            // This And starts a virtual key combination
                            // Process the entire combination starting from here
                            let mut shift = false;
                            let mut ctrl = false;
                            let mut alt = false;
                            let mut alt_gr = false;
                            let mut target_vk = None;
                            
                            // Skip the And we're currently at
                            i += 1;
                            
                            // Process all And + Predefined pairs
                            while i < elements.len() {
                                if let BinaryFormatElement::Predefined(key) = &elements[i] {
                                    use crate::types::VirtualKey;
                                    match *key {
                                        x if x == VirtualKey::Shift as u16 => shift = true,
                                        x if x == VirtualKey::Control as u16 => ctrl = true,
                                        x if x == VirtualKey::Menu as u16 => alt = true,
                                        x if x == VirtualKey::LShift as u16 => shift = true,
                                        x if x == VirtualKey::RShift as u16 => shift = true,
                                        x if x == VirtualKey::LControl as u16 => ctrl = true,
                                        x if x == VirtualKey::RControl as u16 => ctrl = true,
                                        x if x == VirtualKey::LMenu as u16 => alt = true,
                                        x if x == VirtualKey::RMenu as u16 => alt_gr = true,
                                        _ => target_vk = Some(*key),
                                    }
                                    
                                    // Check if there's another And following
                                    if i + 1 < elements.len() && matches!(elements[i + 1], BinaryFormatElement::And) {
                                        i += 2; // Skip both Predefined and And
                                        continue;
                                    } else {
                                        break; // End of combination
                                    }
                                } else {
                                    break; // Not a Predefined, end of combination
                                }
                            }
                            
                            // Only create VirtualKey if we have a non-modifier key
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
                    }
                    // If not followed by Predefined, just skip the And
                }
                
                _ => {
                    // Skip other elements in LHS
                }
            }
            
            i += 1;
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