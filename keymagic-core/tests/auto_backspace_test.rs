//! Tests for auto backspace functionality

mod common;
use common::*;

use keymagic_core::BinaryFormatElement;
use keymagic_core::engine::ActionType;
use keymagic_core::types::km2::{Km2File, LayoutOptions};
use kms2km2::VirtualKey;

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
    let input_k = key_input_vk_char(VirtualKey::KeyK, 'k');
    
    let output_k = engine.process_key(input_k).unwrap();
    assert_eq!(output_k.composing_text, "k");
    assert_eq!(output_k.action, ActionType::Insert("k".to_string()));
    assert!(output_k.is_processed);
    
    // Type 'a' - matches "ka" => "က"
    let input_a = key_input_vk_char(VirtualKey::KeyA, 'a');
    
    let output_a = engine.process_key(input_a).unwrap();
    assert_eq!(output_a.composing_text, "က");
    assert_eq!(output_a.action, ActionType::BackspaceDeleteAndInsert(1, "က".to_string()));
    assert!(output_a.is_processed);
    
    // Press backspace - auto_bksp should act like undo, restoring to "k"
    let input_backspace = key_input_from_vk(VirtualKey::Back);
    
    let output_backspace = engine.process_key(input_backspace).unwrap();
    assert_eq!(output_backspace.composing_text, "k", "Should restore to previous state 'k'");
    assert_eq!(output_backspace.action, ActionType::BackspaceDeleteAndInsert(1, "k".to_string()), "Should replace 'က' with 'k'");
    assert!(output_backspace.is_processed);
    
    // Press backspace again - should restore to empty state
    let input_backspace2 = key_input_from_vk(VirtualKey::Back);
    
    let output_backspace2 = engine.process_key(input_backspace2).unwrap();
    assert_eq!(output_backspace2.composing_text, "", "Should restore to empty state");
    assert_eq!(output_backspace2.action, ActionType::BackspaceDelete(1), "Should delete 'k'");
    assert!(output_backspace2.is_processed);
}

#[test]
fn test_auto_backspace_disabled() {
    let keyboard = create_test_keyboard_without_auto_bksp();
    let binary = create_km2_binary(&keyboard).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type 'k' - should show in composing
    let input_k = key_input_vk_char(VirtualKey::KeyK, 'k');
    
    let output_k = engine.process_key(input_k).unwrap();
    assert_eq!(output_k.composing_text, "k");
    assert_eq!(output_k.action, ActionType::Insert("k".to_string()));
    assert!(output_k.is_processed);
    
    // Type 'a' - matches "ka" => "က"
    let input_a = key_input_vk_char(VirtualKey::KeyA, 'a');
    
    let output_a = engine.process_key(input_a).unwrap();
    assert_eq!(output_a.composing_text, "က");
    assert_eq!(output_a.action, ActionType::BackspaceDeleteAndInsert(1, "က".to_string()));
    assert!(output_a.is_processed);
    
    // Press backspace - auto_bksp disabled, should simply delete last character
    let input_backspace = key_input_from_vk(VirtualKey::Back);
    
    let output_backspace = engine.process_key(input_backspace).unwrap();
    assert_eq!(output_backspace.composing_text, "", "Should delete last character, leaving empty");
    assert_eq!(output_backspace.action, ActionType::BackspaceDelete(1), "Should delete 1 character");
    assert!(output_backspace.is_processed, "Backspace should be processed when buffer is not empty");
}

#[test]
fn test_auto_backspace_empty_buffer() {
    let keyboard = create_test_keyboard_with_auto_bksp();
    let binary = create_km2_binary(&keyboard).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Press backspace with empty buffer - should not delete anything
    let input_backspace = key_input_from_vk(VirtualKey::Back);
    
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
    let input_k = key_input_vk_char(VirtualKey::KeyK, 'k');
    
    let output_k = engine.process_key(input_k).unwrap();
    assert_eq!(output_k.composing_text, "k");
    
    // Type 'a' - matches "ka" => "က", auto_bksp should not trigger
    let input_a = key_input_vk_char(VirtualKey::KeyA, 'a');
    
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
    let input_k = key_input_vk_char(VirtualKey::KeyK, 'k');
    
    engine.process_key(input_k).unwrap();
    
    // Type 'x' - should append normally (auto_bksp only works with backspace key)
    let input_x = key_input_vk_char(VirtualKey::KeyX, 'x');
    
    let output_x = engine.process_key(input_x).unwrap();
    assert_eq!(output_x.composing_text, "kx", "Should append 'x' normally");
    assert_eq!(output_x.action, ActionType::Insert("x".to_string()));
    assert!(output_x.is_processed, "Character input should be processed");
}

#[test]
fn test_auto_backspace_disabled_simple_delete() {
    let keyboard = create_test_keyboard_without_auto_bksp();
    let binary = create_km2_binary(&keyboard).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type "test" - no rules match, just appending
    for ch in "test".chars() {
        let input = key_input_from_char(ch);
        engine.process_key(input).unwrap();
    }
    assert_eq!(engine.composing_text(), "test");
    
    // Press backspace - should delete 't'
    let input_backspace = key_input_from_vk(VirtualKey::Back);
    let output = engine.process_key(input_backspace).unwrap();
    assert_eq!(output.composing_text, "tes", "Should delete last character");
    assert_eq!(output.action, ActionType::BackspaceDelete(1));
    assert!(output.is_processed);
    
    // Press backspace again - should delete 's'
    let input_backspace2 = key_input_from_vk(VirtualKey::Back);
    let output2 = engine.process_key(input_backspace2).unwrap();
    assert_eq!(output2.composing_text, "te", "Should delete last character");
    assert_eq!(output2.action, ActionType::BackspaceDelete(1));
    assert!(output2.is_processed);
}

#[test]
fn test_auto_backspace_enabled_without_history() {
    let keyboard = create_test_keyboard_with_auto_bksp();
    let binary = create_km2_binary(&keyboard).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type "test" - no rules match, just appending
    for ch in "test".chars() {
        let input = key_input_from_char(ch);
        engine.process_key(input).unwrap();
    }
    assert_eq!(engine.composing_text(), "test");
    
    // Press backspace - no history to restore, should delete 't'
    let input_backspace = key_input_from_vk(VirtualKey::Back);
    let output = engine.process_key(input_backspace).unwrap();
    assert_eq!(output.composing_text, "tes", "Should delete last character when no history");
    assert_eq!(output.action, ActionType::BackspaceDelete(1));
    assert!(output.is_processed);
}