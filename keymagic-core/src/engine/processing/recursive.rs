//! Recursive rule processing

use crate::types::Rule;
use crate::engine::state::EngineState;
use crate::engine::matching::{RuleMatcher, Pattern, MatchContext};
use crate::engine::processing::RuleProcessor;
use crate::Result;

const MAX_RECURSION_DEPTH: usize = 10;

/// Handles recursive rule matching and application
pub struct RecursiveProcessor;

impl RecursiveProcessor {
    /// Recursively applies rules until no more matches or stop condition is met
    pub fn process_recursive(
        state: &mut EngineState,
        rules: &[(Rule, Pattern)],
        strings: &[String],
    ) -> Result<()> {
        let mut depth = 0;

        loop {
            // Check recursion depth
            if depth >= MAX_RECURSION_DEPTH {
                break;
            }

            // Create context for recursive matching
            let context = MatchContext::for_recursive(
                state.composing_text(),
                state.active_states(),
            );

            // Try to find a matching rule
            if let Some((rule, pattern, captures)) = RuleMatcher::find_match(rules, &context, strings) {
                // Get the pattern length
                let matched_len = pattern.calculate_match_length(strings).unwrap_or(0);
                // Apply the rule
                let output = RuleProcessor::apply_rule(rule, state, &captures, strings)?;

                // log rule
                println!("Recursive Rule: {:?}", rule);
                // log matched length
                println!("Recursive Matched length: {}", matched_len);
                // log pattern
                println!("Recursive Pattern: {:?}", pattern);
                // log captures
                println!("Recursive Captures: {:?}", captures);
                // log output
                println!("Recursive Output: {}", output);
                // log composing buffer
                println!("Recursive Composing buffer: {}", state.composing_text());
                
                // Update composing buffer by replacing only the matched portion
                state.composing_buffer_mut().replace_from_end(matched_len, &output);
                // log composing buffer
                println!("Recursive Composing buffer after: {}", state.composing_text());
                
                // Check stop conditions based on the output
                if should_stop_recursion(&output) {
                    break;
                }
                
                depth += 1;
            } else {
                // No more matches
                break;
            }
        }

        Ok(())
    }
}

/// Checks if recursion should stop based on the rule output
pub fn should_stop_recursion(output: &str) -> bool {
    // Stop if empty
    if output.is_empty() {
        return true;
    }

    // Stop if single printable ASCII character (excluding space)
    let chars: Vec<char> = output.chars().collect();
    if chars.len() == 1 {
        let ch = chars[0];
        // Printable ASCII excluding space: '!' through '~'
        if matches!(ch, '!'..='~') {
            return true;
        }
    }

    false
}