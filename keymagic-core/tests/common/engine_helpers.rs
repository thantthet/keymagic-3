//! Helper functions for testing with the new engine API

use keymagic_core::{KeyMagicEngine, KeyInput, EngineOutput};
use keymagic_core::engine::{ModifierState, ActionType};
use keymagic_core::VirtualKey;
use keymagic_core::km2::Km2Loader;

/// Create an engine from KMS rules string
pub fn create_engine(kms_rules: &str) -> Result<KeyMagicEngine, Box<dyn std::error::Error>> {
    // Compile KMS to KM2
    let km2_file = kms2km2::compile_kms(kms_rules)?;
    
    // Convert to binary
    let mut buffer = Vec::new();
    let writer = kms2km2::binary::Km2Writer::new(&mut buffer);
    writer.write_km2_file(&km2_file)?;
    
    // Load the binary and create engine
    create_engine_from_binary(&buffer)
}

/// Create an engine from binary data
pub fn create_engine_from_binary(data: &[u8]) -> Result<KeyMagicEngine, Box<dyn std::error::Error>> {
    let km2 = Km2Loader::load(data)?;
    Ok(KeyMagicEngine::new(km2)?)
}

/// Helper to create a KeyInput from just a character
pub fn key_input_from_char(ch: char) -> KeyInput {
    KeyInput::from_char(ch)
}

/// Helper to create a KeyInput from a virtual key
pub fn key_input_from_vk(vk: VirtualKey) -> KeyInput {
    KeyInput::from_vk(vk as u16, ModifierState::default())
}

/// Helper to create a KeyInput with virtual key and character
pub fn key_input_vk_char(vk: VirtualKey, ch: char) -> KeyInput {
    KeyInput::new(vk as u16, ModifierState::default(), Some(ch))
}

/// Helper to create a KeyInput with modifiers
pub fn key_input_with_modifiers(vk: VirtualKey, ch: Option<char>, shift: bool, ctrl: bool, alt: bool) -> KeyInput {
    let modifiers = ModifierState::new(shift, ctrl, alt, false);
    KeyInput::new(vk as u16, modifiers, ch)
}

/// Process a key and return the output
pub fn process_key(engine: &mut KeyMagicEngine, input: KeyInput) -> Result<EngineOutput, Box<dyn std::error::Error>> {
    Ok(engine.process_key(input)?)
}

/// Process a character and return the output
pub fn process_char(engine: &mut KeyMagicEngine, ch: char) -> Result<EngineOutput, Box<dyn std::error::Error>> {
    process_key(engine, key_input_from_char(ch))
}

/// Process a string of characters one by one
pub fn process_string(engine: &mut KeyMagicEngine, s: &str) -> Result<Vec<EngineOutput>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    for ch in s.chars() {
        results.push(process_char(engine, ch)?);
    }
    Ok(results)
}

/// Check if the engine produced the expected output
pub fn assert_output_text(output: &EngineOutput, expected: &str) {
    match &output.action {
        ActionType::Insert(text) => {
            assert_eq!(text, expected, "Expected insert '{}', got '{}'", expected, text);
        }
        ActionType::BackspaceDeleteAndInsert(_, text) => {
            assert_eq!(text, expected, "Expected text '{}', got '{}'", expected, text);
        }
        _ => panic!("Expected text output '{}', but got {:?}", expected, output.action),
    }
}

/// Check if no action was taken
pub fn assert_no_action(output: &EngineOutput) {
    assert_eq!(output.action, ActionType::None, "Expected no action, got {:?}", output.action);
}

/// Get the current composing text
pub fn get_composing_text(engine: &KeyMagicEngine) -> String {
    engine.composing_text().to_string()
}