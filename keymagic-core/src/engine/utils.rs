//! Utility functions and implementations for the engine

use crate::types::{Km2File, FileHeader};

impl Default for Km2File {
    fn default() -> Self {
        Self {
            header: FileHeader::new(),
            strings: Vec::new(),
            info: Vec::new(),
            rules: Vec::new(),
        }
    }
}