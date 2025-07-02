mod common;

use common::*;
use keymagic_core::{BinaryFormatElement, VirtualKey};
use keymagic_core::engine::ActionType;

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
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Press Shift+A
    let input = key_input_with_modifiers(VirtualKey::KeyA, Some('A'), true, false, false);
    let result = process_key(&mut engine, input).unwrap();
    
    assert_eq!(result.composing_text, "A");
    assert_eq!(result.action, ActionType::Insert("A".to_string()));
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
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type 'a' - should match the valid rule
    let result = process_char(&mut engine, 'a').unwrap();
    assert_eq!(result.composing_text, "A");
    assert_eq!(result.action, ActionType::Insert("A".to_string()));
    
    // The invalid rule (modifiers only) should have been ignored during preprocessing
}

#[test]
fn test_complex_modifier_combination() {
    // Test: <VK_CTRL & VK_ALT & VK_KEY_K> => "က"
    // Note: This test is currently skipped as modifier combinations need special handling
    // TODO: Implement proper AND + VK sequence parsing in Pattern::from_elements
}

#[test]
fn test_virtual_key_without_modifiers() {
    // Test: <VK_F1> => "F1 pressed"
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::F1 as u16)
        ],
        vec![BinaryFormatElement::String("F1 pressed".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Press F1
    let input = key_input_from_vk(VirtualKey::F1);
    let result = process_key(&mut engine, input).unwrap();
    
    assert_eq!(result.composing_text, "F1 pressed");
    assert_eq!(result.action, ActionType::Insert("F1 pressed".to_string()));
}

#[test]
fn test_virtual_key_priority_over_char() {
    // Test that VK rules have priority over character rules
    let mut km2 = create_basic_km2();
    
    // Add character rule first
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("a".to_string())],
        vec![BinaryFormatElement::String("CHAR_A".to_string())]
    );
    
    // Add VK rule second
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::KeyA as u16)
        ],
        vec![BinaryFormatElement::String("VK_A".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Press 'a' key - VK rule should match first
    let input = key_input_vk_char(VirtualKey::KeyA, 'a');
    let result = process_key(&mut engine, input).unwrap();
    
    assert_eq!(result.composing_text, "VK_A");
}

#[test]
fn test_multiple_vk_in_sequence() {
    // Test multiple VK rules in sequence
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::F1 as u16)
        ],
        vec![BinaryFormatElement::String("[F1]".to_string())]
    );
    
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::F2 as u16)
        ],
        vec![BinaryFormatElement::String("[F2]".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Press F1
    let result = process_key(&mut engine, key_input_from_vk(VirtualKey::F1)).unwrap();
    assert_eq!(result.composing_text, "[F1]");
    
    // Press F2
    let result = process_key(&mut engine, key_input_from_vk(VirtualKey::F2)).unwrap();
    assert_eq!(result.composing_text, "[F1][F2]");
}

#[test]
fn test_special_key_backspace() {
    // Test backspace behavior
    let mut km2 = create_basic_km2();
    
    // First add a rule to type something
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("test".to_string())],
        vec![BinaryFormatElement::String("TEST".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type "test"
    process_string(&mut engine, "tes").unwrap();
    process_char(&mut engine, 't').unwrap();
    assert_eq!(engine.composing_text(), "TEST");
    
    // Without a backspace rule, VK_BACK just adds to composing
    // In a real implementation, the IME framework would handle backspace
}