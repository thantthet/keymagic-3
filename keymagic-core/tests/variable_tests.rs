mod common;

use common::*;
use keymagic_core::{BinaryFormatElement, km2::Km2Loader};
use keymagic_core::engine::ActionType;
use keymagic_core::types::opcodes::FLAG_ANYOF;

#[test]
fn test_variable_string_literals() {
    // Test: $consonants = "ကခဂဃင"
    let mut km2 = create_basic_km2();
    
    // Add the variable as a string
    let var_idx = add_string(&mut km2, "ကခဂဃင");
    
    // Create a rule that uses the variable: $consonants[*] => "consonants"
    add_rule(&mut km2, 
        vec![
            BinaryFormatElement::Variable(var_idx),
            BinaryFormatElement::Modifier(FLAG_ANYOF)
        ],
        vec![BinaryFormatElement::String("consonants".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    // Verify the variable was loaded
    assert_eq!(loaded.strings.len(), 1);
    assert_eq!(loaded.strings[0].value, "ကခဂဃင");
    
    // Test the rule works
    let mut engine = create_engine_from_binary(&binary).unwrap();
    let result = process_char(&mut engine, 'က').unwrap();
    assert_eq!(result.action, ActionType::Insert("consonants".to_string()));
}

#[test]
fn test_variable_unicode_concatenation() {
    // Test: $vowels = U1000 + U1001 + U1002
    let mut km2 = create_basic_km2();
    
    // Add variable with concatenated Unicode values
    let var_idx = add_string(&mut km2, "\u{1000}\u{1001}\u{1002}");
    
    // Create a rule: $vowels[*] => "vowels"
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Variable(var_idx),
            BinaryFormatElement::Modifier(FLAG_ANYOF)
        ],
        vec![BinaryFormatElement::String("vowels".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    assert_eq!(loaded.strings[0].value, "\u{1000}\u{1001}\u{1002}");
    
    // Test matching
    let mut engine = create_engine_from_binary(&binary).unwrap();
    let result = process_char(&mut engine, '\u{1000}').unwrap();
    assert_eq!(result.action, ActionType::Insert("vowels".to_string()));
}

#[test]
fn test_variable_concatenation() {
    // Test: $combined = $consonants + $vowels
    let mut km2 = create_basic_km2();
    
    // Add base variables
    let cons_idx = add_string(&mut km2, "ကခဂ");
    let vowels_idx = add_string(&mut km2, "ာိီ");
    
    // Combined variable would be preprocessed by KMS compiler
    // Here we simulate the result
    let combined_idx = add_string(&mut km2, "ကခဂာိီ");
    
    // Rule using combined variable
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Variable(combined_idx),
            BinaryFormatElement::Modifier(FLAG_ANYOF)
        ],
        vec![BinaryFormatElement::String("combined".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Test matching a character from the combined variable
    let result = process_char(&mut engine, 'ခ').unwrap();
    assert_eq!(result.action, ActionType::Insert("combined".to_string()));
    
    // Reset and test vowel
    engine = create_engine_from_binary(&binary).unwrap();
    let result = process_char(&mut engine, 'ိ').unwrap();
    assert_eq!(result.action, ActionType::Insert("combined".to_string()));
}

#[test]
fn test_variable_in_rule_output() {
    // Test using variables in rule output
    let mut km2 = create_basic_km2();
    
    // Add variables
    let cons_idx = add_string(&mut km2, "က");
    let vowel_idx = add_string(&mut km2, "ာ");
    
    // Rule: "a" => $cons + $vowel
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("a".to_string())],
        vec![
            BinaryFormatElement::Variable(cons_idx),
            BinaryFormatElement::Variable(vowel_idx)
        ]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let mut engine = create_engine_from_binary(&binary).unwrap();
    let result = process_char(&mut engine, 'a').unwrap();
    assert_eq!(result.action, ActionType::Insert("ကာ".to_string()));
}

#[test]
fn test_predefined_unicode_variables() {
    // Test predefined variables like $ZWS = U200B
    let mut km2 = create_basic_km2();
    
    // Add ZWS variable
    let zws_idx = add_string(&mut km2, "\u{200B}");
    
    // Rule: "zws" => $ZWS + "test"
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("zws".to_string())],
        vec![
            BinaryFormatElement::Variable(zws_idx),
            BinaryFormatElement::String("test".to_string())
        ]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Process each character
    process_char(&mut engine, 'z').unwrap();
    process_char(&mut engine, 'w').unwrap();
    let result = process_char(&mut engine, 's').unwrap();
    
    assert_eq!(result.action, ActionType::BackspaceDeleteAndInsert(2, "\u{200B}test".to_string()));
}

#[test]
fn test_variable_with_mixed_content() {
    // Test variable with mixed Unicode and regular characters
    let mut km2 = create_basic_km2();
    
    // Variable with mixed content
    let mixed_idx = add_string(&mut km2, "a\u{1000}b\u{1001}c");
    
    // Rule to match any character from the variable
    // Pattern: Variable + Modifier(OP_ANYOF)
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Variable(mixed_idx),
            BinaryFormatElement::Modifier(FLAG_ANYOF)
        ],
        vec![BinaryFormatElement::String("matched".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let mut engine = create_engine_from_binary(&binary).unwrap();
    
    // Test matching 'a'
    let result = process_char(&mut engine, 'a').unwrap();
    assert_eq!(result.action, ActionType::Insert("matched".to_string()));
    
    // Test matching Unicode character
    engine = create_engine_from_binary(&binary).unwrap();
    let result = process_char(&mut engine, '\u{1000}').unwrap();
    assert_eq!(result.action, ActionType::Insert("matched".to_string()));
}