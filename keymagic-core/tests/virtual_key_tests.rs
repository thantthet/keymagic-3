mod common;

use common::*;
use keymagic_core::{BinaryFormatElement, KeyMagicEngine, VirtualKey, engine::ModifierState};

#[test]
fn test_virtual_key_with_modifiers() {
    // Test: <VK_SHIFT & VK_KEY_A> => "A"
    let mut km2 = create_basic_km2();
    
    // Rule with modifier combination: AND + VK_SHIFT + VK_KEY_A
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::Shift as u16),
            BinaryFormatElement::Predefined(VirtualKey::KeyA as u16)
        ],
        vec![BinaryFormatElement::String("A".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Press Shift+A
    let mut modifiers = ModifierState::new();
    modifiers.shift = true;
    let result = engine.process_key_event(keymagic_core::KeyInput {
        vk_code: VirtualKey::KeyA,
        modifiers,
        char_value: Some('A'), // Uppercase due to shift
    }).unwrap();
    
    assert_eq!(result.commit_text, Some("A".to_string()));
    assert!(result.consumed);
}

#[test]
fn test_modifier_only_virtual_key_ignored() {
    // Test that VirtualKey with only modifiers (no actual key) is ignored
    let mut km2 = create_basic_km2();
    
    // Rule: <VK_SHIFT & VK_CONTROL> => "INVALID" (no actual key, only modifiers)
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::Shift as u16),
            BinaryFormatElement::Predefined(VirtualKey::Control as u16)
        ],
        vec![BinaryFormatElement::String("INVALID".to_string())]
    );
    
    // Add a valid rule to ensure engine works
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("a".to_string())],
        vec![BinaryFormatElement::String("A".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Type 'a' - should match the valid rule
    let result = engine.process_key_event(key_input_from_char('a')).unwrap();
    assert_eq!(result.commit_text, Some("A".to_string()));
    
    // The invalid rule (modifiers only) should have been ignored during preprocessing
}

#[test]
fn test_complex_modifier_combination() {
    // Test: <VK_CTRL & VK_ALT & VK_KEY_K> => "က"
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::Control as u16),
            BinaryFormatElement::Predefined(VirtualKey::Menu as u16), // Alt
            BinaryFormatElement::Predefined(VirtualKey::KeyK as u16)
        ],
        vec![BinaryFormatElement::String("က".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Press Ctrl+Alt+K
    let mut modifiers = ModifierState::new();
    modifiers.ctrl = true;
    modifiers.alt = true;
    let result = engine.process_key_event(keymagic_core::KeyInput {
        vk_code: VirtualKey::KeyK,
        modifiers,
        char_value: None, // No char when Ctrl+Alt is pressed
    }).unwrap();
    
    assert_eq!(result.commit_text, Some("က".to_string()));
    assert!(result.consumed);
}
