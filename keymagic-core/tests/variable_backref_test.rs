use keymagic_core::KeyMagicEngine;
use keymagic_core::types::*;
use kms2km2::binary::Km2Writer;

mod common;
use common::*;

#[test]
fn test_variable_positional_matching() {
    let mut km2 = create_basic_km2();
    
    // Add variables
    let row1k_idx = add_string(&mut km2, "qwerty");
    let row1u_idx = add_string(&mut km2, "ဆတနမအပ");
    
    // Add rule: $row1K[*] => $row1U[$1]
    // This should map q->ဆ, w->တ, e->န, r->မ, t->အ, y->ပ
    add_rule(&mut km2, 
        vec![
            BinaryFormatElement::Variable(row1k_idx),
            BinaryFormatElement::Modifier(FLAG_ANYOF),
        ],
        vec![
            BinaryFormatElement::Variable(row1u_idx),
            BinaryFormatElement::Modifier(1), // Back-reference $1
        ]
    );
    
    // Convert to binary
    let binary = create_km2_binary(&km2).unwrap();
    
    // Test the engine
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Test 'q' -> 'ဆ' (position 0)
    let result = engine.process_key_event(key_input_from_char('q')).unwrap();
    assert_eq!(result.commit_text, Some("ဆ".to_string()));
    
    // Test 'w' -> 'တ' (position 1)
    let result = engine.process_key_event(key_input_from_char('w')).unwrap();
    assert_eq!(result.commit_text, Some("တ".to_string()));
    
    // Test 'e' -> 'န' (position 2)
    let result = engine.process_key_event(key_input_from_char('e')).unwrap();
    assert_eq!(result.commit_text, Some("န".to_string()));
    
    // Test 'y' -> 'ပ' (position 5)
    let result = engine.process_key_event(key_input_from_char('y')).unwrap();
    assert_eq!(result.commit_text, Some("ပ".to_string()));
}

#[test]
fn test_multiple_variable_references() {
    let mut km2 = create_basic_km2();
    
    // Add variables
    let cons_idx = add_string(&mut km2, "kg");
    let vowels_idx = add_string(&mut km2, "ai");
    let cons_out_idx = add_string(&mut km2, "ကဂ");
    let vowels_out_idx = add_string(&mut km2, "ာိ");
    
    // Add rule: $cons[*] + $vowels[*] => $cons_out[$1] + $vowels_out[$2]
    add_rule(&mut km2, 
        vec![
            BinaryFormatElement::Variable(cons_idx),
            BinaryFormatElement::Modifier(FLAG_ANYOF),
            BinaryFormatElement::Variable(vowels_idx),
            BinaryFormatElement::Modifier(FLAG_ANYOF),
        ],
        vec![
            BinaryFormatElement::Variable(cons_out_idx),
            BinaryFormatElement::Modifier(1), // $1
            BinaryFormatElement::Variable(vowels_out_idx),
            BinaryFormatElement::Modifier(2), // $2
        ]
    );
    
    // Convert to binary
    let binary = create_km2_binary(&km2).unwrap();
    
    // Test the engine
    let mut engine = KeyMagicEngine::new();
    engine.load_keyboard(&binary).unwrap();
    
    // Test 'ka' -> 'ကာ' (positions 0,0)
    let result = engine.process_key_event(key_input_from_char('k')).unwrap();
    assert_eq!(result.commit_text, None); // Composing
    let result = engine.process_key_event(key_input_from_char('a')).unwrap();
    assert_eq!(result.commit_text, Some("ကာ".to_string()));
    
    // Test 'gi' -> 'ဂိ' (positions 1,1)
    let result = engine.process_key_event(key_input_from_char('g')).unwrap();
    assert_eq!(result.commit_text, None); // Composing
    let result = engine.process_key_event(key_input_from_char('i')).unwrap();
    assert_eq!(result.commit_text, Some("ဂိ".to_string()));
}