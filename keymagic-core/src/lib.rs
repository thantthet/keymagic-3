pub mod types;
pub mod error;
pub mod km2;
pub mod engine;
pub mod ffi;
pub mod hotkey;

pub use types::*;

// Re-export commonly used types
pub use types::km2::{Km2File, Rule, BinaryFormatElement, InfoEntry, FileHeader, LayoutOptions, StringEntry, Metadata};
pub use types::errors::KmsError;
pub use types::virtual_keys::VirtualKey;
pub use error::{Error, Result};
pub use engine::{KeyMagicEngine, KeyInput, EngineOutput};