//! Action generation from state changes

use crate::engine::output::ActionType;

/// Generates output actions based on state changes
pub struct ActionGenerator;

impl ActionGenerator {
    /// Generates an action based on before/after composing text
    pub fn generate_action(
        before: &str,
        after: &str,
        _had_input: bool,
    ) -> ActionType {
        // If no change, no action needed
        if before == after {
            return ActionType::None;
        }

        // log before and after
        println!("Before: {}", before);
        println!("After: {}", after);

        let before_chars: Vec<char> = before.chars().collect();
        let after_chars: Vec<char> = after.chars().collect();

        // Find common prefix length
        let common_prefix_len = before_chars.iter()
            .zip(after_chars.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // Calculate what changed
        let chars_to_delete = before_chars.len() - common_prefix_len;
        let chars_to_insert: String = after_chars[common_prefix_len..].iter().collect();

        match (chars_to_delete, chars_to_insert.is_empty()) {
            (0, false) => {
                // Just insertion
                ActionType::Insert(chars_to_insert)
            }
            (n, true) if n > 0 => {
                // Just deletion
                ActionType::BackspaceDelete(n)
            }
            (n, false) if n > 0 => {
                // Delete and insert
                ActionType::BackspaceDeleteAndInsert(n, chars_to_insert)
            }
            _ => ActionType::None,
        }
    }
}