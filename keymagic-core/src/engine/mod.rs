//! KeyMagic Engine - Core key processing engine
//! 
//! This module provides the main engine for processing keyboard input
//! according to KeyMagic keyboard layout rules.

mod engine;
mod input;
mod output;
mod state;
mod matching;
mod processing;
mod types;
mod utils;

#[cfg(test)]
mod compat;

pub use engine::KeyMagicEngine;
pub use input::{KeyInput, ModifierState};
pub use output::{EngineOutput, ActionType};
pub use types::{Element, Predefined};

// Re-export error types
pub use crate::error::{Error, Result};