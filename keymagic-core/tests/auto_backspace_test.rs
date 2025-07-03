//! Tests for auto backspace functionality

mod common;
use common::*;

use keymagic_core::BinaryFormatElement;
use keymagic_core::engine::{KeyInput, ModifierState, Predefined, ActionType};
use keymagic_core::types::km2::{Km2File, LayoutOptions};

/// Create a simple test keyboard with auto_bksp enabled
fn create_test_keyboard_with_auto_bksp() -> Km2File {
    let mut options = LayoutOptions::default();
    options.auto_bksp = 1;
    
    let mut keyboard = create_basic_km2();
    keyboard.header.layout_options = options;
    
    // Rule: "ka" => "က"
    add_rule(&mut keyboard,
        vec![BinaryFormatElement::String("ka".to_string())],
        vec![BinaryFormatElement::String("က".to_string())]
    );
    
    keyboard
}

/// Create a simple test keyboard with auto_bksp disabled (default)
fn create_test_keyboard_without_auto_bksp() -> Km2File {
    let mut keyboard = create_basic_km2();
    
    // auto_bksp is 0 by default
    assert_eq!(keyboard.header.layout_options.auto_bksp, 0);
    
    // Rule: "ka" => "က"
    add_rule(&mut keyboard,
        vec![BinaryFormatElement::String("ka".to_string())],
        vec![BinaryFormatElement::String("က".to_string())]
    );
    
    keyboard
}

#[test]
fn test_auto_backspace_enabled() {
    let keyboard = create_test_keyboard_with_auto_bksp();
    let binary = create_km2_binary(&keyboard).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type 'k' - should show in composing
    let input_k = KeyInput {
        key_code: Predefined::from_raw('K' as u16),
        modifiers: ModifierState::default(),
        character: Some('k'),
    };
    
    let output_k = engine.process_key(input_k).unwrap();
    assert_eq!(output_k.composing_text, "k");
    assert_eq!(output_k.action, ActionType::Insert("k".to_string()));
    assert!(output_k.is_processed);
    
    // Press backspace - no rule matches, auto_bksp should delete 'k'
    let input_backspace = KeyInput {
        key_code: Predefined::from_raw(0x08), // VK_BACK
        modifiers: ModifierState::default(),
        character: None, // Backspace has no character
    };
    
    let output_backspace = engine.process_key(input_backspace).unwrap();
    assert_eq!(output_backspace.composing_text, "", "Should have backspaced the 'k'");
    assert_eq!(output_backspace.action, ActionType::BackspaceDelete(1), "Should delete 1 character");
    assert!(output_backspace.is_processed);
}

#[test]
fn test_auto_backspace_disabled() {
    let keyboard = create_test_keyboard_without_auto_bksp();
    let binary = create_km2_binary(&keyboard).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type 'k' - should show in composing
    let input_k = KeyInput {
        key_code: Predefined::from_raw('K' as u16),
        modifiers: ModifierState::default(),
        character: Some('k'),
    };
    
    let output_k = engine.process_key(input_k).unwrap();
    assert_eq!(output_k.composing_text, "k");
    assert_eq!(output_k.action, ActionType::Insert("k".to_string()));
    assert!(output_k.is_processed);
    
    // Press backspace - no rule matches, auto_bksp disabled, should NOT delete
    let input_backspace = KeyInput {
        key_code: Predefined::from_raw(0x08), // VK_BACK
        modifiers: ModifierState::default(),
        character: None, // Backspace has no character
    };
    
    let output_backspace = engine.process_key(input_backspace).unwrap();
    assert_eq!(output_backspace.composing_text, "k", "Should keep 'k' as backspace has no character");
    assert_eq!(output_backspace.action, ActionType::None);
    assert!(!output_backspace.is_processed, "Backspace without auto_bksp should not be processed");
}

#[test]
fn test_auto_backspace_empty_buffer() {
    let keyboard = create_test_keyboard_with_auto_bksp();
    let binary = create_km2_binary(&keyboard).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Press backspace with empty buffer - should not delete anything
    let input_backspace = KeyInput {
        key_code: Predefined::from_raw(0x08), // VK_BACK
        modifiers: ModifierState::default(),
        character: None,
    };
    
    let output_backspace = engine.process_key(input_backspace).unwrap();
    assert_eq!(output_backspace.composing_text, "", "Buffer should remain empty");
    assert_eq!(output_backspace.action, ActionType::None);
    assert!(!output_backspace.is_processed, "Backspace on empty buffer should not be processed");
}

#[test]
fn test_auto_backspace_with_successful_match() {
    let keyboard = create_test_keyboard_with_auto_bksp();
    let binary = create_km2_binary(&keyboard).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type 'k'
    let input_k = KeyInput {
        key_code: Predefined::from_raw('K' as u16),
        modifiers: ModifierState::default(),
        character: Some('k'),
    };
    
    let output_k = engine.process_key(input_k).unwrap();
    assert_eq!(output_k.composing_text, "k");
    
    // Type 'a' - matches "ka" => "က", auto_bksp should not trigger
    let input_a = KeyInput {
        key_code: Predefined::from_raw('A' as u16),
        modifiers: ModifierState::default(),
        character: Some('a'),
    };
    
    let output_a = engine.process_key(input_a).unwrap();
    assert_eq!(output_a.composing_text, "က");
    assert_eq!(output_a.action, ActionType::BackspaceDeleteAndInsert(1, "က".to_string()));
    assert!(output_a.is_processed);
}

#[test]
fn test_auto_backspace_with_normal_character() {
    let keyboard = create_test_keyboard_with_auto_bksp();
    let binary = create_km2_binary(&keyboard).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type 'k'
    let input_k = KeyInput {
        key_code: Predefined::from_raw('K' as u16),
        modifiers: ModifierState::default(),
        character: Some('k'),
    };
    
    engine.process_key(input_k).unwrap();
    
    // Type 'x' - should append normally (auto_bksp only works with backspace key)
    let input_x = KeyInput {
        key_code: Predefined::from_raw('X' as u16),
        modifiers: ModifierState::default(),
        character: Some('x'),
    };
    
    let output_x = engine.process_key(input_x).unwrap();
    assert_eq!(output_x.composing_text, "kx", "Should append 'x' normally");
    assert_eq!(output_x.action, ActionType::Insert("x".to_string()));
    assert!(output_x.is_processed, "Character input should be processed");
}