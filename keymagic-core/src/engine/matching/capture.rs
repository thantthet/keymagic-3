//! Capture management for pattern matching

use std::collections::HashMap;

/// Represents a captured value with both content and optional index
#[derive(Debug, Clone)]
pub struct Capture {
    /// The captured content (e.g., matched string, character)
    pub content: String,
    /// Optional index (e.g., position in a variable for AnyOf matches)
    pub index: Option<usize>,
}

impl Capture {
    /// Creates a new capture with content only
    pub fn new(content: String) -> Self {
        Self { content, index: None }
    }
    
    /// Creates a new capture with content and index
    pub fn with_index(content: String, index: usize) -> Self {
        Self { content, index: Some(index) }
    }
}

/// Manages captured values during pattern matching
#[derive(Debug, Clone)]
pub struct CaptureManager {
    captures: HashMap<usize, Capture>,
}

impl CaptureManager {
    /// Creates a new capture manager
    pub fn new() -> Self {
        Self {
            captures: HashMap::new(),
        }
    }


    /// Stores a capture with content only
    pub fn set_capture(&mut self, index: usize, value: String) {
        self.captures.insert(index, Capture::new(value));
    }
    
    /// Stores a capture with content and index
    pub fn set_capture_with_index(&mut self, index: usize, content: String, position: usize) {
        self.captures.insert(index, Capture::with_index(content, position));
    }

    /// Gets a capture by index (returns the content)
    pub fn get_capture(&self, index: usize) -> Option<&str> {
        self.captures.get(&index).map(|c| c.content.as_str())
    }
    
    /// Gets the full capture structure by index
    pub fn get_full_capture(&self, index: usize) -> Option<&Capture> {
        self.captures.get(&index)
    }

    
    /// Gets the next capture index
    pub fn next_index(&self) -> usize {
        self.captures.len() + 1
    }
}

impl Default for CaptureManager {
    fn default() -> Self {
        Self::new()
    }
}