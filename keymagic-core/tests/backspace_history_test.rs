//! Tests for backspace history functionality

use keymagic_core::{KeyMagicEngine, Km2File, VirtualKey};

mod common;
use common::*;

#[test]
fn test_backspace_history_with_auto_bksp() {
    // Create a keyboard with auto_bksp enabled
    let mut keyboard = Km2File::default();
    keyboard.header.layout_options.auto_bksp = 1;
    
    let mut engine = KeyMagicEngine::new(keyboard).unwrap();
    
    // Type some characters
    process_char(&mut engine, 'a').unwrap();
    assert_eq!(get_composing_text(&engine), "a");
    
    process_char(&mut engine, 'b').unwrap();
    assert_eq!(get_composing_text(&engine), "ab");
    
    process_char(&mut engine, 'c').unwrap();
    assert_eq!(get_composing_text(&engine), "abc");
    
    // Backspace should restore previous state
    let backspace_input = key_input_from_vk(VirtualKey::Back);
    process_key(&mut engine, backspace_input.clone()).unwrap();
    assert_eq!(get_composing_text(&engine), "ab");
    
    // Another backspace
    process_key(&mut engine, backspace_input.clone()).unwrap();
    assert_eq!(get_composing_text(&engine), "a");
    
    // Another backspace
    process_key(&mut engine, backspace_input).unwrap();
    assert_eq!(get_composing_text(&engine), "");
}

#[test]
fn test_backspace_without_auto_bksp() {
    // Create a keyboard with auto_bksp disabled (default)
    let keyboard = Km2File::default();
    let mut engine = KeyMagicEngine::new(keyboard).unwrap();
    
    // Type some characters
    process_char(&mut engine, 'a').unwrap();
    process_char(&mut engine, 'b').unwrap();
    process_char(&mut engine, 'c').unwrap();
    assert_eq!(get_composing_text(&engine), "abc");
    
    // Backspace with auto_bksp disabled should not be processed
    let backspace_input = key_input_from_vk(VirtualKey::Back);
    let output = process_key(&mut engine, backspace_input).unwrap();
    assert_eq!(get_composing_text(&engine), "abc"); // Text should remain unchanged
    assert!(!output.is_processed); // Backspace should not be processed
}

#[test]
fn test_history_cleared_on_reset() {
    let mut keyboard = Km2File::default();
    keyboard.header.layout_options.auto_bksp = 1;
    
    let mut engine = KeyMagicEngine::new(keyboard).unwrap();
    
    // Build some history
    process_char(&mut engine, 'a').unwrap();
    process_char(&mut engine, 'b').unwrap();
    
    // Reset should clear history
    engine.reset();
    assert_eq!(get_composing_text(&engine), "");
    
    // Type new text
    process_char(&mut engine, 'x').unwrap();
    assert_eq!(get_composing_text(&engine), "x");
    
    // Backspace should delete character, not restore history
    let backspace_input = key_input_from_vk(VirtualKey::Back);
    process_key(&mut engine, backspace_input).unwrap();
    assert_eq!(get_composing_text(&engine), "");
}

#[test]
fn test_history_cleared_on_set_composing_text() {
    let mut keyboard = Km2File::default();
    keyboard.header.layout_options.auto_bksp = 1;
    
    let mut engine = KeyMagicEngine::new(keyboard).unwrap();
    
    // Build some history
    process_char(&mut engine, 'a').unwrap();
    process_char(&mut engine, 'b').unwrap();
    
    // Set composing text should clear history
    engine.set_composing_text("xyz".to_string());
    assert_eq!(get_composing_text(&engine), "xyz");
    
    // Backspace should delete one character, not restore history
    let backspace_input = key_input_from_vk(VirtualKey::Back);
    process_key(&mut engine, backspace_input).unwrap();
    assert_eq!(get_composing_text(&engine), "xy");
}

#[test]
fn test_history_max_size() {
    let mut keyboard = Km2File::default();
    keyboard.header.layout_options.auto_bksp = 1;
    
    let mut engine = KeyMagicEngine::new(keyboard).unwrap();
    
    // Type more than max_history_size characters
    for i in 0..25 {
        process_char(&mut engine, (b'a' + (i % 26)) as char).unwrap();
    }
    
    // Should be able to backspace up to max_history_size times
    let backspace_input = key_input_from_vk(VirtualKey::Back);
    for _ in 0..20 {
        process_key(&mut engine, backspace_input.clone()).unwrap();
    }
    
    // After 20 backspaces, we should have 5 characters left (25 - 20)
    assert_eq!(get_composing_text(&engine).len(), 5);
    
    // Further backspaces should delete one character at a time
    process_key(&mut engine, backspace_input).unwrap();
    assert_eq!(get_composing_text(&engine).len(), 4);
}

#[test]
fn test_process_key_test_with_backspace_history() {
    let mut keyboard = Km2File::default();
    keyboard.header.layout_options.auto_bksp = 1;
    
    let mut engine = KeyMagicEngine::new(keyboard).unwrap();
    
    // Build some state
    process_char(&mut engine, 'a').unwrap();
    process_char(&mut engine, 'b').unwrap();
    
    // Test mode backspace should work with history
    let backspace_input = key_input_from_vk(VirtualKey::Back);
    let test_output = engine.process_key_test(backspace_input).unwrap();
    assert_eq!(test_output.composing_text, "a");
    
    // Original state should still be "ab"
    assert_eq!(get_composing_text(&engine), "ab");
}

#[test]
fn test_backspace_history_with_rules() {
    use keymagic_core::{Rule, BinaryFormatElement, StringEntry};
    
    // Create a keyboard with a rule "ka" => "က"
    let mut keyboard = Km2File::default();
    keyboard.header.layout_options.auto_bksp = 1;
    
    // Add strings
    keyboard.strings.push(StringEntry { value: "ka".to_string() });
    keyboard.strings.push(StringEntry { value: "က".to_string() });
    
    // Add rule: "ka" => "က" 
    let rule = Rule {
        lhs: vec![BinaryFormatElement::String("ka".to_string())],
        rhs: vec![BinaryFormatElement::String("က".to_string())],
    };
    keyboard.rules.push(rule);
    keyboard.header.rule_count = 1;
    keyboard.header.string_count = 2;
    
    let mut engine = KeyMagicEngine::new(keyboard).unwrap();
    
    // Type "k"
    process_char(&mut engine, 'k').unwrap();
    assert_eq!(get_composing_text(&engine), "k");
    
    // Type "a" - should transform to "က"
    process_char(&mut engine, 'a').unwrap();
    assert_eq!(get_composing_text(&engine), "က");
    
    // Backspace should restore to "k" (the state before the transformation)
    let backspace_input = key_input_from_vk(VirtualKey::Back);
    process_key(&mut engine, backspace_input.clone()).unwrap();
    assert_eq!(get_composing_text(&engine), "k");
    
    // Another backspace should clear everything
    process_key(&mut engine, backspace_input).unwrap();
    assert_eq!(get_composing_text(&engine), "");
}

#[test]
fn test_backspace_history_not_recorded_for_backspace() {
    let mut keyboard = Km2File::default();
    keyboard.header.layout_options.auto_bksp = 1;
    
    let mut engine = KeyMagicEngine::new(keyboard).unwrap();
    
    // Type some characters
    process_char(&mut engine, 'a').unwrap();
    process_char(&mut engine, 'b').unwrap();
    process_char(&mut engine, 'c').unwrap();
    assert_eq!(get_composing_text(&engine), "abc");
    
    // First backspace restores "ab"
    let backspace_input = key_input_from_vk(VirtualKey::Back);
    process_key(&mut engine, backspace_input.clone()).unwrap();
    assert_eq!(get_composing_text(&engine), "ab");
    
    // Type new character
    process_char(&mut engine, 'd').unwrap();
    assert_eq!(get_composing_text(&engine), "abd");
    
    // Backspace should restore to "ab" (not "abc" since that backspace wasn't recorded)
    process_key(&mut engine, backspace_input).unwrap();
    assert_eq!(get_composing_text(&engine), "ab");
}