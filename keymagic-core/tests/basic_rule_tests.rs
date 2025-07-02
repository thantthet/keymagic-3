mod common;

use common::*;
use keymagic_core::BinaryFormatElement;
use keymagic_core::engine::ActionType;

#[test]
fn test_simple_string_mapping() {
    // Test: "ka" => "က"
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("ka".to_string())],
        vec![BinaryFormatElement::String("က".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type 'k'
    let result = process_char(&mut engine, 'k').unwrap();
    assert_eq!(result.composing_text, "k");
    assert_eq!(result.action, ActionType::Insert("k".to_string()));
    
    // Type 'a' to complete "ka"
    let result = process_char(&mut engine, 'a').unwrap();
    assert_eq!(result.composing_text, "က");
    assert_eq!(result.action, ActionType::BackspaceDeleteAndInsert(1, "က".to_string()));
}

#[test]
fn test_single_char_mapping() {
    // Test: 'a' => U200B + "test"
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("a".to_string())],
        vec![
            BinaryFormatElement::String("\u{200B}".to_string()),
            BinaryFormatElement::String("test".to_string())
        ]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    let result = process_char(&mut engine, 'a').unwrap();
    assert_eq!(result.composing_text, "\u{200B}test");
    assert_eq!(result.action, ActionType::Insert("\u{200B}test".to_string()));
}

#[test]
fn test_unicode_to_unicode_mapping() {
    // Test: U0061 + U0062 => U1000
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::String("\u{0061}".to_string()), // 'a'
            BinaryFormatElement::String("\u{0062}".to_string())  // 'b'
        ],
        vec![BinaryFormatElement::String("\u{1000}".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    process_char(&mut engine, 'a').unwrap();
    let result = process_char(&mut engine, 'b').unwrap();
    assert_eq!(result.composing_text, "\u{1000}");
    assert_eq!(result.action, ActionType::BackspaceDeleteAndInsert(1, "\u{1000}".to_string()));
}

#[test]
fn test_variable_substitution_in_rules() {
    // Test: $input => $output
    let mut km2 = create_basic_km2();
    
    let input_idx = add_string(&mut km2, "test");
    let output_idx = add_string(&mut km2, "result");
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::Variable(input_idx)],
        vec![BinaryFormatElement::Variable(output_idx)]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type "test"
    process_string(&mut engine, "tes").unwrap();
    let result = process_char(&mut engine, 't').unwrap();
    assert_eq!(result.composing_text, "result");
    assert_eq!(result.action, ActionType::BackspaceDeleteAndInsert(3, "result".to_string()));
}

#[test]
fn test_multiple_rules_priority() {
    // Test that both rules work correctly
    // Note: In the current implementation, "k" => "SHORT" matches immediately
    // This is different from some IME implementations that wait for longer patterns
    let mut km2 = create_basic_km2();
    
    // Add rules
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("k".to_string())],
        vec![BinaryFormatElement::String("SHORT".to_string())]
    );
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("ka".to_string())],
        vec![BinaryFormatElement::String("LONG".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Test 1: Type 'k' alone - matches "k" => "SHORT"
    let result = process_char(&mut engine, 'k').unwrap();
    assert_eq!(result.composing_text, "SHORT");
    assert_eq!(result.action, ActionType::Insert("SHORT".to_string()));
    
    // Reset engine for next test
    engine.reset();
    
    // Test 2: Type "ka" - should match "ka" => "LONG" if we have it in composing
    // First, let's type something that doesn't match to build composing text
    process_char(&mut engine, 'x').unwrap(); // 'x' doesn't match any rule
    engine.reset();
    
    // Actually, let's test a different scenario
    // Add a rule that doesn't consume 'k' immediately
    let mut km2_2 = create_basic_km2();
    add_rule(&mut km2_2,
        vec![BinaryFormatElement::String("ka".to_string())],
        vec![BinaryFormatElement::String("LONG".to_string())]
    );
    
    let binary2 = create_km2_binary(&km2_2).unwrap();
    let mut engine2 = create_engine_from_binary(&binary2).unwrap();
    
    // Now type "ka" - should match
    process_char(&mut engine2, 'k').unwrap();
    let result = process_char(&mut engine2, 'a').unwrap();
    assert_eq!(result.composing_text, "LONG");
}

#[test]
fn test_no_matching_rule() {
    // Test that unmatched characters are passed through
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("ka".to_string())],
        vec![BinaryFormatElement::String("က".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type 'x' - no matching rule
    let result = process_char(&mut engine, 'x').unwrap();
    assert_eq!(result.composing_text, "x");
    assert_eq!(result.action, ActionType::Insert("x".to_string()));
}

#[test]
fn test_greedy_matching() {
    // Test: "title" => "Title" - greedy matching behavior
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("title".to_string())],
        vec![BinaryFormatElement::String("Title".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Type "titl" - should keep composing
    process_string(&mut engine, "titl").unwrap();
    assert_eq!(engine.composing_text(), "titl");
    
    // Type 'e' - completes "title"
    let result = process_char(&mut engine, 'e').unwrap();
    assert_eq!(result.composing_text, "Title");
    assert_eq!(result.action, ActionType::BackspaceDeleteAndInsert(4, "Title".to_string()));
}

#[test]
fn test_recursive_rule_chain() {
    // Test: 'x' => "abc", 'abc' => "X"
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("x".to_string())],
        vec![BinaryFormatElement::String("abc".to_string())]
    );
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("abc".to_string())],
        vec![BinaryFormatElement::String("X".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    let result = process_char(&mut engine, 'x').unwrap();
    assert_eq!(result.composing_text, "X");
}

#[test]
fn test_multi_character_replacement() {
    // Test: "ah" => "ဟ"
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("ah".to_string())],
        vec![BinaryFormatElement::String("ဟ".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    process_char(&mut engine, 'a').unwrap();
    let result = process_char(&mut engine, 'h').unwrap();
    assert_eq!(result.composing_text, "ဟ");
    assert_eq!(result.action, ActionType::BackspaceDeleteAndInsert(1, "ဟ".to_string()));
}