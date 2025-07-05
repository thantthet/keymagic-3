mod common;

use common::*;
use keymagic_core::engine::{ModifierState, KeyInput};
use keymagic_core::VirtualKey;

#[test]
fn test_exact_modifier_matching() {
    let rules = r#"
        < VK_SHIFT & VK_KEY_A > => "Shift+A"
        < VK_CTRL & VK_KEY_A > => "Ctrl+A"
        < VK_ALT & VK_KEY_A > => "Alt+A"
        < VK_SHIFT & VK_CTRL & VK_KEY_A > => "Shift+Ctrl+A"
        < VK_KEY_A > => "Just A"
    "#;
    
    let mut engine = create_engine(rules).unwrap();
    
    // Test 1: Shift+A should only match when ONLY shift is pressed
    let input = KeyInput::new(
        VirtualKey::KeyA as u16,
        ModifierState { shift: true, ctrl: false, alt: false, caps_lock: false },
        Some('A')
    );
    let result = engine.process_key(input).unwrap();
    assert_eq!(result.composing_text, "Shift+A");
    engine.reset();
    
    // Test 2: Ctrl+A should only match when ONLY ctrl is pressed
    let input = KeyInput::new(
        VirtualKey::KeyA as u16,
        ModifierState { shift: false, ctrl: true, alt: false, caps_lock: false },
        None
    );
    let result = engine.process_key(input).unwrap();
    assert_eq!(result.composing_text, "Ctrl+A");
    engine.reset();
    
    // Test 3: Shift+Ctrl+A should NOT match Shift+A rule even though shift is present
    let input = KeyInput::new(
        VirtualKey::KeyA as u16,
        ModifierState { shift: true, ctrl: true, alt: false, caps_lock: false },
        None
    );
    let result = engine.process_key(input).unwrap();
    assert_eq!(result.composing_text, "Shift+Ctrl+A");
    engine.reset();
    
    // Test 4: Shift+Alt+A should not match any rule (no exact match)
    let input = KeyInput::new(
        VirtualKey::KeyA as u16,
        ModifierState { shift: true, ctrl: false, alt: true, caps_lock: false },
        None
    );
    let result = engine.process_key(input).unwrap();
    // Should not match any rule
    assert_eq!(result.composing_text, "");
    assert!(!result.is_processed); // Key should not be consumed
    engine.reset();
    
    // Test 5: Just 'A' without modifiers
    let input = KeyInput::new(
        VirtualKey::KeyA as u16,
        ModifierState { shift: false, ctrl: false, alt: false, caps_lock: false },
        None
    );
    let result = engine.process_key(input).unwrap();
    assert_eq!(result.composing_text, "Just A");
    engine.reset();
    
    // Test 6: Extra modifier (all three) should not match any specific rule
    let input = KeyInput::new(
        VirtualKey::KeyA as u16,
        ModifierState { shift: true, ctrl: true, alt: true, caps_lock: false },
        None
    );
    let result = engine.process_key(input).unwrap();
    assert_eq!(result.composing_text, "");
    assert!(!result.is_processed);
}

#[test]
fn test_caps_lock_independent() {
    // Caps Lock state should not affect modifier matching
    let rules = r#"
        < VK_SHIFT & VK_KEY_A > => "Shift+A"
    "#;
    
    let mut engine = create_engine(rules).unwrap();
    
    // Shift+A with Caps Lock ON should still match
    let input = KeyInput::new(
        VirtualKey::KeyA as u16,
        ModifierState { shift: true, ctrl: false, alt: false, caps_lock: true },
        Some('a') // lowercase because shift+caps cancel out
    );
    let result = engine.process_key(input).unwrap();
    assert_eq!(result.composing_text, "Shift+A");
}