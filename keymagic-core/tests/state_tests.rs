mod common;

use common::*;
use keymagic_core::{BinaryFormatElement, VirtualKey};
use keymagic_core::engine::ActionType;

#[test]
fn test_basic_state_toggle() {
    // Test state toggle: < VK_CFLEX > => ('zawgyi')
    let mut km2 = create_basic_km2();
    
    // Add state name to strings
    let state_idx = 0;
    
    // Rule to enter state - add And to make it valid
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::Oem3 as u16)
        ],
        vec![BinaryFormatElement::Switch(state_idx)]
    );
    
    // Rule that only works in state: ('zawgyi') + '1' => "၁"
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Switch(state_idx),
            BinaryFormatElement::String("1".to_string())
        ],
        vec![BinaryFormatElement::String("၁".to_string())]
    );
    
    // Normal rule: '1' => "1"
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("1".to_string())],
        vec![BinaryFormatElement::String("1".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type '1' before entering state - should output "1"
    let result = process_char(&mut engine, '1').unwrap();
    assert_eq!(result.composing_text, "1");
    assert_eq!(result.action, ActionType::Insert("1".to_string()));
    
    // Press Cflex to enter state
    let result = process_key(&mut engine, key_input_from_vk(VirtualKey::Oem3)).unwrap();
    assert_eq!(result.composing_text, "1");
    assert_eq!(result.action, ActionType::None); // State switch doesn't produce output
    
    // Type '1' in state - should output "၁"
    let result = process_char(&mut engine, '1').unwrap();
    assert_eq!(result.composing_text, "1၁");
    assert_eq!(result.action, ActionType::Insert("၁".to_string()));
    
    // Type '1' again - state should be cleared, so output "1"
    let result = process_char(&mut engine, '1').unwrap();
    assert_eq!(result.composing_text, "1၁1");
    assert_eq!(result.action, ActionType::Insert("1".to_string()));
}

#[test]
fn test_multiple_states() {
    // Test multiple states can be active simultaneously
    let mut km2 = create_basic_km2();
    
    let state1_idx = 0;
    let state2_idx = 1;
    
    // Rules to enter states - add And to make valid
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::F1 as u16)
        ],
        vec![BinaryFormatElement::Switch(state1_idx)]
    );
    
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::F2 as u16)
        ],
        vec![BinaryFormatElement::Switch(state2_idx)]
    );
    
    // Rule that works in state1: ('state1') + 'a' => "A1"
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Switch(state1_idx),
            BinaryFormatElement::String("a".to_string())
        ],
        vec![BinaryFormatElement::String("A1".to_string())]
    );
    
    // Rule that works in state2: ('state2') + 'a' => "A2"
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Switch(state2_idx),
            BinaryFormatElement::String("a".to_string())
        ],
        vec![BinaryFormatElement::String("A2".to_string())]
    );
    
    // Default rule: 'a' => "a"
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("a".to_string())],
        vec![BinaryFormatElement::String("a".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Enter state1
    process_key(&mut engine, key_input_from_vk(VirtualKey::F1)).unwrap();
    
    // Type 'a' in state1 - should output "A1"
    let result = process_char(&mut engine, 'a').unwrap();
    assert_eq!(result.composing_text, "A1");
    assert_eq!(result.action, ActionType::Insert("A1".to_string()));
    
    // State1 is cleared after the input, type 'a' again - should output "a"
    let result = process_char(&mut engine, 'a').unwrap();
    assert_eq!(result.composing_text, "A1a");
    assert_eq!(result.action, ActionType::Insert("a".to_string()));
    
    // Enter state2
    process_key(&mut engine, key_input_from_vk(VirtualKey::F2)).unwrap();
    
    // Type 'a' in state2 - should output "A2"
    let result = process_char(&mut engine, 'a').unwrap();
    assert_eq!(result.composing_text, "A1aA2");
    assert_eq!(result.action, ActionType::Insert("A2".to_string()));
}

#[test]
fn test_state_with_any_wildcard() {
    // Test: ('state') + ANY => $1 + $1
    let mut km2 = create_basic_km2();
    
    let state_idx = 0;
    
    // Rule to enter state - add And to make valid
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::F3 as u16)
        ],
        vec![BinaryFormatElement::Switch(state_idx)]
    );
    
    // Rule in state that matches ANY and maintains state
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Switch(state_idx),
            BinaryFormatElement::Any
        ],
        vec![
            BinaryFormatElement::Reference(1), // $1 - the matched character
            BinaryFormatElement::Reference(1)  // $1 - the matched character
        ]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Enter state
    process_key(&mut engine, key_input_from_vk(VirtualKey::F3)).unwrap();
    
    // Type 'x' - should pass through and maintain state
    let result = process_char(&mut engine, 'x').unwrap();
    assert_eq!(result.composing_text, "xx");
    assert_eq!(result.action, ActionType::Insert("xx".to_string()));
}

#[test]
fn test_state_based_digit_conversion() {
    // Test Zawgyi-style digit conversion in state
    let mut km2 = create_basic_km2();
    
    let zg_state_idx = 0;
    
    // Rule to enter Zawgyi mode - add And to make valid
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::Oem3 as u16)
        ],
        vec![BinaryFormatElement::Switch(zg_state_idx)]
    );
    
    // Zawgyi digit rules in state
    // ('zg_key') + '1' => U100D + U1039 + U100D
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Switch(zg_state_idx),
            BinaryFormatElement::String("1".to_string())
        ],
        vec![BinaryFormatElement::String("\u{100D}\u{1039}\u{100D}".to_string())]
    );
    
    // ('zg_key') + '2' => U100E + U1039 + U100E
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Switch(zg_state_idx),
            BinaryFormatElement::String("2".to_string())
        ],
        vec![BinaryFormatElement::String("\u{100E}\u{1039}\u{100E}".to_string())]
    );
    
    // Normal digit rules
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("1".to_string())],
        vec![BinaryFormatElement::String("၁".to_string())]
    );
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("2".to_string())],
        vec![BinaryFormatElement::String("၂".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type digits normally
    let result = process_char(&mut engine, '1').unwrap();
    assert_eq!(result.action, ActionType::Insert("၁".to_string()));
    
    let result = process_char(&mut engine, '2').unwrap();
    assert_eq!(result.action, ActionType::Insert("၂".to_string()));
    assert_eq!(result.composing_text, "၁၂");
    
    // Enter Zawgyi mode
    process_key(&mut engine, key_input_from_vk(VirtualKey::Oem3)).unwrap();
    
    // Type digit '1' in Zawgyi mode
    let result = process_char(&mut engine, '1').unwrap();
    assert_eq!(result.action, ActionType::Insert("\u{100D}\u{1039}\u{100D}".to_string()));
    assert_eq!(result.composing_text, "၁၂\u{100D}\u{1039}\u{100D}");
    
    // State is cleared after input, so type '2' normally (not in zawgyi mode)
    let result = process_char(&mut engine, '2').unwrap();
    assert_eq!(result.action, ActionType::Insert("၂".to_string()));
    assert_eq!(result.composing_text, "၁၂\u{100D}\u{1039}\u{100D}၂");
    
    // Enter Zawgyi mode again
    process_key(&mut engine, key_input_from_vk(VirtualKey::Oem3)).unwrap();
    
    // Type digit '2' in Zawgyi mode
    let result = process_char(&mut engine, '2').unwrap();
    assert_eq!(result.action, ActionType::Insert("\u{100E}\u{1039}\u{100E}".to_string()));
    assert_eq!(result.composing_text, "၁၂\u{100D}\u{1039}\u{100D}၂\u{100E}\u{1039}\u{100E}");
}

#[test]
fn test_multiple_active_states() {
    // Test that multiple states can be active simultaneously
    let mut km2 = create_basic_km2();
    
    let state1_idx = 0;
    let state2_idx = 1;
    
    // Rule to enter both states at once - add And to make valid
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::F5 as u16)
        ],
        vec![
            BinaryFormatElement::Switch(state1_idx),
            BinaryFormatElement::Switch(state2_idx)
        ]
    );
    
    // Rule that only works when both states are active
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Switch(state1_idx),
            BinaryFormatElement::Switch(state2_idx),
            BinaryFormatElement::String("x".to_string())
        ],
        vec![BinaryFormatElement::String("BOTH".to_string())]
    );
    
    // Rule for state1 only
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Switch(state1_idx),
            BinaryFormatElement::String("x".to_string())
        ],
        vec![BinaryFormatElement::String("S1".to_string())]
    );
    
    // Default rule
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("x".to_string())],
        vec![BinaryFormatElement::String("x".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type 'x' without states - should output "x"
    let result = process_char(&mut engine, 'x').unwrap();
    assert_eq!(result.action, ActionType::Insert("x".to_string()));
    
    // Enter both states
    process_key(&mut engine, key_input_from_vk(VirtualKey::F5)).unwrap();
    
    // Type 'x' with both states active - should output "BOTH"
    let result = process_char(&mut engine, 'x').unwrap();
    assert_eq!(result.action, ActionType::Insert("BOTH".to_string()));
    assert_eq!(result.composing_text, "xBOTH");
}

#[test]
fn test_state_priority_in_rule_sorting() {
    // Test that state rules have priority over non-state rules
    let mut km2 = create_basic_km2();
    
    let state_idx = 0;
    
    // Rule to enter state - add And to make valid
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::F4 as u16)
        ],
        vec![BinaryFormatElement::Switch(state_idx)]
    );
    
    // Long non-state rule: "test" => "normal"
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("test".to_string())],
        vec![BinaryFormatElement::String("normal".to_string())]
    );
    
    // Short state rule: ('priority_test') + 't' => "state"
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Switch(state_idx),
            BinaryFormatElement::String("t".to_string())
        ],
        vec![BinaryFormatElement::String("state".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Enter state
    process_key(&mut engine, key_input_from_vk(VirtualKey::F4)).unwrap();
    
    // Type 't' - should match state rule even though it's shorter
    let result = process_char(&mut engine, 't').unwrap();
    assert_eq!(result.action, ActionType::Insert("state".to_string()));
}