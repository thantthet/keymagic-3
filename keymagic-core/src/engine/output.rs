//! Output representation for the KeyMagic engine

/// Result of processing a key input
#[derive(Debug, Clone, PartialEq)]
pub struct EngineOutput {
    /// Current composing text buffer
    pub composing_text: String,
    /// Action to perform
    pub action: ActionType,
    /// Whether the input was processed by the engine (matched a rule)
    pub is_processed: bool,
}

/// Types of actions the engine can output
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    /// No action needed (e.g., state change only)
    None,
    /// Insert text at cursor
    Insert(String),
    /// Delete characters before cursor
    BackspaceDelete(usize),
    /// Delete characters then insert text
    BackspaceDeleteAndInsert(usize, String),
}

impl EngineOutput {
    /// Creates a new engine output
    pub fn new(composing_text: String, action: ActionType, is_processed: bool) -> Self {
        Self {
            composing_text,
            action,
            is_processed,
        }
    }

    /// Creates a no-action output
    pub fn none(composing_text: String) -> Self {
        Self {
            composing_text,
            action: ActionType::None,
            is_processed: false,
        }
    }

    /// Creates an insert action output
    pub fn insert(composing_text: String, text: String) -> Self {
        Self {
            composing_text,
            action: ActionType::Insert(text),
            is_processed: true,
        }
    }

    /// Creates a delete action output
    pub fn delete(composing_text: String, count: usize) -> Self {
        Self {
            composing_text,
            action: ActionType::BackspaceDelete(count),
            is_processed: true,
        }
    }

    /// Creates a delete-and-insert action output
    pub fn delete_and_insert(composing_text: String, delete_count: usize, insert_text: String) -> Self {
        Self {
            composing_text,
            action: ActionType::BackspaceDeleteAndInsert(delete_count, insert_text),
            is_processed: true,
        }
    }
}