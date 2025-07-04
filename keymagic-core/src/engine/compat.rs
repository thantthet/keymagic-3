//! Compatibility layer for existing tests

use crate::engine::{KeyMagicEngine, KeyInput, ModifierState, EngineOutput};
use crate::types::{Km2File, VirtualKey};
use crate::km2::Km2Loader;
use crate::error::Result;

/// Extension trait to provide backward compatibility for tests
pub trait EngineCompat {
    fn load_keyboard(&mut self, data: &[u8]) -> Result<()>;
    fn process_key_event(&mut self, input: KeyInput) -> Result<EngineOutput>;
}

impl EngineCompat for KeyMagicEngine {
    fn load_keyboard(&mut self, data: &[u8]) -> Result<()> {
        let keyboard = Km2Loader::load(data)?;
        *self = Self::new(keyboard)?;
        Ok(())
    }
    
    fn process_key_event(&mut self, input: KeyInput) -> Result<EngineOutput> {
        self.process_key(input)
    }
}

/// Create an engine without a keyboard (for tests)
pub fn new_test_engine() -> Result<KeyMagicEngine> {
    KeyMagicEngine::new(Km2File::default())
}

/// Convert VirtualKey to KeyInput for tests
pub fn key_input_from_vk_char(vk: VirtualKey, ch: Option<char>) -> KeyInput {
    // Convert VirtualKey enum to Windows VK code
    let vk_code = vk.to_win_vk();
    KeyInput::new(vk_code, ModifierState::default(), ch)
}

/// Create KeyInput from just a character
pub fn key_input_from_char(ch: char) -> KeyInput {
    KeyInput::from_char(ch)
}

/// Create ModifierState for tests
pub fn modifiers(shift: bool, ctrl: bool, alt: bool) -> ModifierState {
    ModifierState::new(shift, ctrl, alt, false)
}