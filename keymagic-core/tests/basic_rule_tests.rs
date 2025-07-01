mod common;

use common::*;
use keymagic_core::{BinaryFormatElement, km2::Km2Loader, KeyMagicEngine};

#[test]
fn test_simple_string_mapping() {
    // Test: "ka" => "က"
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("ka".to_string())],
        vec![BinaryFormatElement::String("က".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Type 'k'
    let result = engine.process_key_event(key_input_from_char('k')).unwrap();
    assert!(result.commit_text.is_none());
    assert_eq!(result.composing_text, Some("k".to_string()));
    
    // Type 'a' to complete "ka"
    let result = engine.process_key_event(key_input_from_char('a')).unwrap();
    assert_eq!(result.commit_text, Some("က".to_string()));
    assert_eq!(result.composing_text, None);
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
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    let result = engine.process_key_event(key_input_from_char('a')).unwrap();
    assert_eq!(result.commit_text, Some("\u{200B}test".to_string()));
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
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    engine.process_key_event(key_input_from_char('a'));
    let result = engine.process_key_event(key_input_from_char('b')).unwrap();
    assert_eq!(result.commit_text, Some("\u{1000}".to_string()));
}

#[test]
fn test_variable_substitution_in_rules() {
    // Test: $input => $output
    let mut km2 = create_basic_km2();
    
    let input_idx = add_string(&mut km2, "test");
    let output_idx = add_string(&mut km2, "result");
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::Variable(input_idx + 1)],
        vec![BinaryFormatElement::Variable(output_idx + 1)]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Type "test"
    engine.process_key_event(key_input_from_char('t')).unwrap();
    engine.process_key_event(key_input_from_char('e')).unwrap();
    engine.process_key_event(key_input_from_char('s')).unwrap();
    let result = engine.process_key_event(key_input_from_char('t')).unwrap();
    
    assert_eq!(result.commit_text, Some("result".to_string()));
}

#[test]
fn test_multiple_rules() {
    // Test multiple rules with different patterns
    let mut km2 = create_basic_km2();
    
    // Rule 1: "ka" => "က"
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("ka".to_string())],
        vec![BinaryFormatElement::String("က".to_string())]
    );
    
    // Rule 2: "kha" => "ခ"
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("kha".to_string())],
        vec![BinaryFormatElement::String("ခ".to_string())]
    );
    
    // Rule 3: "ga" => "ဂ"
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("ga".to_string())],
        vec![BinaryFormatElement::String("ဂ".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    // Test "ka"
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    engine.process_key_event(key_input_from_char('k'));
    let result = engine.process_key_event(key_input_from_char('a')).unwrap();
    assert_eq!(result.commit_text, Some("က".to_string()));
    
    // Test "kha"
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    engine.process_key_event(key_input_from_char('k'));
    engine.process_key_event(key_input_from_char('h'));
    let result = engine.process_key_event(key_input_from_char('a')).unwrap();
    assert_eq!(result.commit_text, Some("ခ".to_string()));
    
    // Test "ga"
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    engine.process_key_event(key_input_from_char('g'));
    let result = engine.process_key_event(key_input_from_char('a')).unwrap();
    assert_eq!(result.commit_text, Some("ဂ".to_string()));
}

#[test]
fn test_null_output() {
    // Test: "delete" => NULL (empty output)
    let mut km2 = create_basic_km2();
    
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("delete".to_string())],
        vec![] // Empty output represents NULL
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Type "delete"
    engine.process_key_event(key_input_from_char('d')).unwrap();
    engine.process_key_event(key_input_from_char('e')).unwrap();
    engine.process_key_event(key_input_from_char('l')).unwrap();
    engine.process_key_event(key_input_from_char('e')).unwrap();
    engine.process_key_event(key_input_from_char('t')).unwrap();
    let result = engine.process_key_event(key_input_from_char('e')).unwrap();
    
    // Should commit empty string (NULL output)
    assert_eq!(result.commit_text, Some("".to_string()));
    assert_eq!(result.composing_text, None);
}

#[test]
fn test_complex_pattern() {
    // Test complex pattern with multiple parts
    let mut km2 = create_basic_km2();
    
    // Create variables
    let prefix_idx = add_string(&mut km2, "pre");
    let suffix_idx = add_string(&mut km2, "fix");
    
    // Rule: $prefix + "test" + $suffix => "result"
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Variable(prefix_idx + 1),
            BinaryFormatElement::String("test".to_string()),
            BinaryFormatElement::Variable(suffix_idx + 1)
        ],
        vec![BinaryFormatElement::String("result".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Type "pretestfix"
    for ch in "pretestfi".chars() {
        engine.process_key_event(key_input_from_char(ch)).unwrap();
    }
    let result = engine.process_key_event(key_input_from_char('x')).unwrap();
    
    assert_eq!(result.commit_text, Some("result".to_string()));
}

#[test]
fn test_overlapping_patterns() {
    // Test that longer patterns take precedence
    let mut km2 = create_basic_km2();
    
    // Rule 1: "ah" => "အ"
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("ah".to_string())],
        vec![BinaryFormatElement::String("အ".to_string())]
    );
    
    // Rule 2: "aww" => "ဪ" (should take precedence when typing "aww")
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("aww".to_string())],
        vec![BinaryFormatElement::String("ဪ".to_string())]
    );

    // Rule 3: "h" => "ဟ"
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("h".to_string())],
        vec![BinaryFormatElement::String("ဟ".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Type 'a'
    engine.process_key_event(key_input_from_char('a')).unwrap();
    let result = engine.process_key_event(key_input_from_char('h')).unwrap();
    
    // Should match "ah" => "အ" not "h" => "ဟ"
    assert_eq!(result.commit_text, Some("အ".to_string()));
}