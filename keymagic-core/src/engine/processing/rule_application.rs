//! Rule application logic

use crate::types::Rule;
use crate::engine::types::Element;
use crate::engine::state::EngineState;
use crate::engine::matching::CaptureManager;
use crate::Result;

/// Handles applying matched rules to engine state
pub struct RuleProcessor;

impl RuleProcessor {
    /// Applies a matched rule to the engine state
    pub fn apply_rule(
        rule: &Rule,
        state: &mut EngineState,
        captures: &CaptureManager,
        strings: &[String],
    ) -> Result<String> {
        let mut output = String::new();
        
        // Convert binary elements to engine elements
        let rhs_elements: Vec<Element> = rule.rhs.iter()
            .map(|e| e.clone().into())
            .collect();

        // Process RHS elements
        let mut i = 0;
        while i < rhs_elements.len() {
            match &rhs_elements[i] {
                Element::Variable(var_idx) => {
                    // Check if next element is a modifier (index)
                    if i + 1 < rhs_elements.len() {
                        if let Element::Modifier(ref_idx) = &rhs_elements[i + 1] {
                            // This is Variable[index] pattern
                            if let Some(capture) = captures.get_full_capture(*ref_idx as usize) {
                                // Use the index from the capture if available, otherwise parse the content
                                let index = if let Some(idx) = capture.index {
                                    idx
                                } else if let Ok(idx) = capture.content.parse::<usize>() {
                                    idx
                                } else {
                                    i += 1; // Skip just the variable
                                    continue;
                                };
                                
                                let var_content = strings.get(*var_idx)
                                    .ok_or_else(|| crate::error::Error::InvalidVariableIndex(*var_idx))?;
                                
                                // Get character at index
                                if let Some(ch) = var_content.chars().nth(index) {
                                    output.push(ch);
                                }
                                
                                i += 2; // Skip both variable and modifier
                                continue;
                            }
                        }
                    }
                    
                    // Regular variable
                    let var_content = strings.get(*var_idx)
                        .ok_or_else(|| crate::error::Error::InvalidVariableIndex(*var_idx))?;
                    output.push_str(var_content);
                }
                Element::String(s) => {
                    output.push_str(s);
                }
                Element::Reference(ref_idx) => {
                    if let Some(captured) = captures.get_capture(*ref_idx) {
                        output.push_str(captured);
                    }
                }
                Element::Switch(state_idx) => {
                    state.activate_state(*state_idx);
                }
                _ => {}
            }
            i += 1;
        }

        Ok(output)
    }
}