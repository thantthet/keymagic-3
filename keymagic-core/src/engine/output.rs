/// Output from the KeyMagic engine after processing a key input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineOutput {
    /// Text to be committed to the application
    pub commit_text: Option<String>,
    /// Updated composing text to display
    pub composing_text: Option<String>,
    /// Number of characters to delete before inserting
    pub delete_count: usize,
    /// Whether the key event was consumed
    pub consumed: bool,
}

impl Default for EngineOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineOutput {
    pub fn new() -> Self {
        Self {
            commit_text: None,
            composing_text: None,
            delete_count: 0,
            consumed: false,
        }
    }
    
    /// Create an output that consumes the key but produces no visible output
    pub fn consume() -> Self {
        Self {
            commit_text: None,
            composing_text: None,
            delete_count: 0,
            consumed: true,
        }
    }

    /// Create an output that commits text
    pub fn commit(text: String) -> Self {
        Self {
            commit_text: Some(text.clone()),
            composing_text: Some(text),
            delete_count: 0,
            consumed: true,
        }
    }
    
    /// Create an output that commits text with custom composing text
    pub fn commit_with_composing(commit: String, composing: Option<String>) -> Self {
        Self {
            commit_text: Some(commit),
            composing_text: composing,
            delete_count: 0,
            consumed: true,
        }
    }

    /// Create an output that updates composing text
    pub fn composing(text: String) -> Self {
        Self {
            commit_text: None,
            composing_text: Some(text),
            delete_count: 0,
            consumed: true,
        }
    }

    /// Create an output that does nothing (key not consumed)
    pub fn pass_through() -> Self {
        Self {
            commit_text: None,
            composing_text: None,
            delete_count: 0,
            consumed: false,
        }
    }

    /// Set the number of characters to delete
    pub fn with_delete(mut self, count: usize) -> Self {
        self.delete_count = count;
        self
    }

    /// Check if this output has any effect
    pub fn has_effect(&self) -> bool {
        self.commit_text.is_some() || self.composing_text.is_some() || self.delete_count > 0
    }
}