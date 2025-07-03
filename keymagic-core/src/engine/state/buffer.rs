//! Composing buffer management

/// Manages the composing text buffer
#[derive(Debug, Clone)]
pub struct ComposingBuffer {
    content: String,
}

impl ComposingBuffer {
    /// Creates a new empty buffer
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    /// Creates a buffer from existing text
    pub fn from(text: String) -> Self {
        Self { content: text }
    }

    /// Clears the buffer
    pub fn clear(&mut self) {
        self.content.clear();
    }

    /// Sets the buffer content
    pub fn set(&mut self, text: String) {
        self.content = text;
    }

    /// Appends text to the buffer
    pub fn append(&mut self, text: &str) {
        self.content.push_str(text);
    }

    /// Gets the buffer content as a string slice
    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// Gets the buffer length in characters
    pub fn char_count(&self) -> usize {
        self.content.chars().count()
    }

    /// Checks if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Takes the content, leaving an empty buffer
    pub fn take(&mut self) -> String {
        std::mem::take(&mut self.content)
    }

    /// Replaces characters from the end of the buffer
    /// 
    /// # Arguments
    /// * `char_count` - Number of characters to remove from the end
    /// * `replacement` - Text to append after removing
    pub fn replace_from_end(&mut self, char_count: usize, replacement: &str) {
        // Convert to char indices for proper Unicode handling
        let chars: Vec<char> = self.content.chars().collect();
        let total_chars = chars.len();
        
        if char_count >= total_chars {
            // Replace entire content
            self.content = replacement.to_string();
        } else {
            // Keep the prefix and replace the suffix
            let keep_chars = total_chars - char_count;
            self.content = chars.into_iter()
                .take(keep_chars)
                .collect::<String>() + replacement;
        }
    }
    
    /// Removes one character from the end of the buffer (backspace)
    pub fn backspace(&mut self) {
        if !self.content.is_empty() {
            // Remove the last character, handling Unicode properly
            let chars: Vec<char> = self.content.chars().collect();
            let char_count = chars.len();
            if char_count > 0 {
                self.content = chars.into_iter()
                    .take(char_count - 1)
                    .collect();
            }
        }
    }
}

impl Default for ComposingBuffer {
    fn default() -> Self {
        Self::new()
    }
}