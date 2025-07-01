mod common;

use common::*;
use keymagic_core::{RuleElement, KeyMagicEngine, VirtualKey};

#[test]
fn test_basic_state_toggle() {
    // Test state toggle: < VK_CFLEX > => ('zawgyi')
    let mut km2 = create_basic_km2();
    
    // Add state name to strings
    let state_idx = add_string(&mut km2, "zawgyi");
    
    // Rule to enter state
    add_rule(&mut km2,
        vec![RuleElement::Predefined(VirtualKey::Oem3 as u16)],
        vec![RuleElement::Switch(state_idx + 1)]
    );
    
    // Rule that only works in state: ('zawgyi') + '1' => "၁"
    add_rule(&mut km2,
        vec![
            RuleElement::Switch(state_idx + 1),
            RuleElement::String("1".to_string())
        ],
        vec![RuleElement::String("၁".to_string())]
    );
    
    // Normal rule: '1' => "1"
    add_rule(&mut km2,
        vec![RuleElement::String("1".to_string())],
        vec![RuleElement::String("1".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Type '1' before entering state - should output "1"
    let result = engine.process_key_event(key_input_from_char('1')).unwrap();
    assert_eq!(result.commit_text, Some("1".to_string()));
    
    // Press Cflex to enter state
    let result = engine.process_key_event(keymagic_core::KeyInput::new(
        VirtualKey::Oem3, 
        keymagic_core::engine::ModifierState::new()
    )).unwrap();
    assert_eq!(result.commit_text, None); // State switch doesn't produce output
    
    // Type '1' in state - should output "၁"
    let result = engine.process_key_event(key_input_from_char('1')).unwrap();
    assert_eq!(result.commit_text, Some("၁".to_string()));
    
    // Press Cflex again to exit state
    engine.process_key_event(keymagic_core::KeyInput::new(
        VirtualKey::Oem3, 
        keymagic_core::engine::ModifierState::new()
    )).unwrap();
    
    // Type '1' after exiting state - should output "1" again
    let result = engine.process_key_event(key_input_from_char('1')).unwrap();
    assert_eq!(result.commit_text, Some("1".to_string()));
}

#[test]
fn test_multiple_states() {
    // Test multiple states can be active simultaneously
    let mut km2 = create_basic_km2();
    
    let state1_idx = add_string(&mut km2, "state1");
    let state2_idx = add_string(&mut km2, "state2");
    
    // Rules to enter states
    add_rule(&mut km2,
        vec![RuleElement::Predefined(VirtualKey::F1 as u16)],
        vec![RuleElement::Switch(state1_idx + 1)]
    );
    
    add_rule(&mut km2,
        vec![RuleElement::Predefined(VirtualKey::F2 as u16)],
        vec![RuleElement::Switch(state2_idx + 1)]
    );
    
    // Rule that works in state1: ('state1') + 'a' => "A1"
    add_rule(&mut km2,
        vec![
            RuleElement::Switch(state1_idx + 1),
            RuleElement::String("a".to_string())
        ],
        vec![RuleElement::String("A1".to_string())]
    );
    
    // Rule that works in state2: ('state2') + 'a' => "A2"
    add_rule(&mut km2,
        vec![
            RuleElement::Switch(state2_idx + 1),
            RuleElement::String("a".to_string())
        ],
        vec![RuleElement::String("A2".to_string())]
    );
    
    // Default rule: 'a' => "a"
    add_rule(&mut km2,
        vec![RuleElement::String("a".to_string())],
        vec![RuleElement::String("a".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Enter state1
    engine.process_key_event(keymagic_core::KeyInput::new(
        VirtualKey::F1, 
        keymagic_core::engine::ModifierState::new()
    )).unwrap();
    
    // Type 'a' in state1 - should output "A1"
    let result = engine.process_key_event(key_input_from_char('a')).unwrap();
    assert_eq!(result.commit_text, Some("A1".to_string()));
    
    // Enter state2 (state1 still active)
    engine.process_key_event(keymagic_core::KeyInput::new(
        VirtualKey::F2, 
        keymagic_core::engine::ModifierState::new()
    )).unwrap();
    
    // Type 'a' with both states active - state1 rule should take precedence (first in rules)
    let result = engine.process_key_event(key_input_from_char('a')).unwrap();
    assert_eq!(result.commit_text, Some("A1".to_string()));
}

#[test]
fn test_state_with_any_wildcard() {
    // Test: ('state') + ANY => $1 + ('state')
    let mut km2 = create_basic_km2();
    
    let state_idx = add_string(&mut km2, "special");
    
    // Rule to enter state
    add_rule(&mut km2,
        vec![RuleElement::Predefined(VirtualKey::F3 as u16)],
        vec![RuleElement::Switch(state_idx + 1)]
    );
    
    // Rule in state that matches ANY and maintains state
    add_rule(&mut km2,
        vec![
            RuleElement::Switch(state_idx + 1),
            RuleElement::Any
        ],
        vec![
            RuleElement::Reference(1), // $1 - the matched character
            RuleElement::Switch(state_idx + 1) // Maintain state
        ]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Enter state
    engine.process_key_event(keymagic_core::KeyInput::new(
        VirtualKey::F3, 
        keymagic_core::engine::ModifierState::new()
    )).unwrap();
    
    // Type 'x' - should pass through and maintain state
    let result = engine.process_key_event(key_input_from_char('x')).unwrap();
    assert_eq!(result.commit_text, Some("x".to_string()));
    
    // Type 'y' - should still be in state
    let result = engine.process_key_event(key_input_from_char('y')).unwrap();
    assert_eq!(result.commit_text, Some("y".to_string()));
    
    // Test that state is maintained - type another character
    let result = engine.process_key_event(key_input_from_char('z')).unwrap();
    assert_eq!(result.commit_text, Some("z".to_string()));
    
    // Exit state
    engine.process_key_event(keymagic_core::KeyInput::new(
        VirtualKey::F3, 
        keymagic_core::engine::ModifierState::new()
    )).unwrap();
    
    // Type 'a' - should not match the state rule anymore
    let result = engine.process_key_event(key_input_from_char('a')).unwrap();
    // With state exited, the ANY rule shouldn't match
    // Since there's no other rule, the behavior depends on layout options
    // Let's just verify we got out of the state by checking that it's not
    // producing the same output as when in state
    assert!(result.commit_text != Some("a".to_string()) || !result.consumed);
}

#[test]
fn test_state_based_digit_conversion() {
    // Test Zawgyi-style digit conversion in state
    let mut km2 = create_basic_km2();
    
    let zg_state_idx = add_string(&mut km2, "zg_key");
    
    // Rule to enter Zawgyi mode
    add_rule(&mut km2,
        vec![RuleElement::Predefined(VirtualKey::Oem3 as u16)],
        vec![RuleElement::Switch(zg_state_idx + 1)]
    );
    
    // Zawgyi digit rules in state
    // ('zg_key') + '1' => U100D + U1039 + U100D
    add_rule(&mut km2,
        vec![
            RuleElement::Switch(zg_state_idx + 1),
            RuleElement::String("1".to_string())
        ],
        vec![RuleElement::String("\u{100D}\u{1039}\u{100D}".to_string())]
    );
    
    // ('zg_key') + '2' => U100E + U1039 + U100E
    add_rule(&mut km2,
        vec![
            RuleElement::Switch(zg_state_idx + 1),
            RuleElement::String("2".to_string())
        ],
        vec![RuleElement::String("\u{100E}\u{1039}\u{100E}".to_string())]
    );
    
    // Normal digit rules
    add_rule(&mut km2,
        vec![RuleElement::String("1".to_string())],
        vec![RuleElement::String("၁".to_string())]
    );
    
    add_rule(&mut km2,
        vec![RuleElement::String("2".to_string())],
        vec![RuleElement::String("၂".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Type digits normally
    let result = engine.process_key_event(key_input_from_char('1')).unwrap();
    assert_eq!(result.commit_text, Some("၁".to_string()));
    
    let result = engine.process_key_event(key_input_from_char('2')).unwrap();
    assert_eq!(result.commit_text, Some("၂".to_string()));
    
    // Enter Zawgyi mode
    engine.process_key_event(keymagic_core::KeyInput::new(
        VirtualKey::Oem3, 
        keymagic_core::engine::ModifierState::new()
    )).unwrap();
    
    // Type digits in Zawgyi mode
    let result = engine.process_key_event(key_input_from_char('1')).unwrap();
    assert_eq!(result.commit_text, Some("\u{100D}\u{1039}\u{100D}".to_string()));
    
    let result = engine.process_key_event(key_input_from_char('2')).unwrap();
    assert_eq!(result.commit_text, Some("\u{100E}\u{1039}\u{100E}".to_string()));
}

#[test]
fn test_state_priority_in_rule_sorting() {
    // Test that state rules have priority over non-state rules
    let mut km2 = create_basic_km2();
    
    let state_idx = add_string(&mut km2, "priority_test");
    
    // Rule to enter state
    add_rule(&mut km2,
        vec![RuleElement::Predefined(VirtualKey::F4 as u16)],
        vec![RuleElement::Switch(state_idx + 1)]
    );
    
    // Long non-state rule: "test" => "normal"
    add_rule(&mut km2,
        vec![RuleElement::String("test".to_string())],
        vec![RuleElement::String("normal".to_string())]
    );
    
    // Short state rule: ('priority_test') + 't' => "state"
    add_rule(&mut km2,
        vec![
            RuleElement::Switch(state_idx + 1),
            RuleElement::String("t".to_string())
        ],
        vec![RuleElement::String("state".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Enter state
    engine.process_key_event(keymagic_core::KeyInput::new(
        VirtualKey::F4, 
        keymagic_core::engine::ModifierState::new()
    )).unwrap();
    
    // Type 't' - should match state rule even though it's shorter
    let result = engine.process_key_event(key_input_from_char('t')).unwrap();
    assert_eq!(result.commit_text, Some("state".to_string()));
}