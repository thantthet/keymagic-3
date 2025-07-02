mod common;

use common::*;
use keymagic_core::engine::ActionType;
use std::path::PathBuf;

/// Get the path to the fixtures directory
fn fixtures_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path
}

#[test]
fn test_vowel_e_reordering_basic() {
    // Test basic vowel_e reordering from MyanSan.kms
    // Line 102: $vowelEK => $filler + $vowelEU
    // This converts 'a' to U200A + U1031
    
    let kms_path = fixtures_dir().join("MyanSan.kms");
    let km2_file = kms2km2::compile_kms_file(&kms_path)
        .expect("Failed to compile MyanSan.kms");
    
    let km2_binary = create_km2_binary(&km2_file)
        .expect("Failed to create KM2 binary");
    
    let mut engine = create_engine_from_binary(&km2_binary).expect("Failed to load keyboard");
    
    // Test 1: Type 'a' - should produce filler + vowel_e
    let result = process_char(&mut engine, 'a').unwrap();
    assert_eq!(result.action, ActionType::Insert("\u{200A}\u{1031}".to_string()));
    assert_eq!(result.composing_text, "\u{200A}\u{1031}");
}

#[test]
fn test_vowel_e_reordering_with_consonant() {
    // Test vowel_e reordering with consonant
    // Line 104: $filler + u1031 + $consU[*] => $3 + u1031
    // This reorders filler+vowel_e+consonant to consonant+vowel_e
    
    let kms_path = fixtures_dir().join("MyanSan.kms");
    let km2_file = kms2km2::compile_kms_file(&kms_path)
        .expect("Failed to compile MyanSan.kms");
    
    let km2_binary = create_km2_binary(&km2_file)
        .expect("Failed to create KM2 binary");
    
    let mut engine = create_engine_from_binary(&km2_binary).expect("Failed to load keyboard");
    
    // Type 'a' then 'u' (which maps to က - U1000)
    // Should reorder to က + vowel_e
    let result = process_char(&mut engine, 'a').unwrap();
    assert_eq!(result.composing_text, "\u{200A}\u{1031}");
    
    let result = process_char(&mut engine, 'u').unwrap();

    assert_eq!(result.action, ActionType::BackspaceDeleteAndInsert(2, "\u{1000}\u{1031}".to_string()));
    assert_eq!(result.composing_text, "\u{1000}\u{1031}");
}

#[test]
fn test_vowel_e_reordering_with_stacked_consonant() {
    // Test vowel_e reordering with stacked consonants
    // Line 105: u1031 + u1039 + $consU[*] => u1039 + $3 + u1031
    // This reorders vowel_e + stack + consonant to stack + consonant + vowel_e
    
    let kms_path = fixtures_dir().join("MyanSan.kms");
    let km2_file = kms2km2::compile_kms_file(&kms_path)
        .expect("Failed to compile MyanSan.kms");
    
    let km2_binary = create_km2_binary(&km2_file)
        .expect("Failed to create KM2 binary");
    
    let mut engine = create_engine_from_binary(&km2_binary).expect("Failed to load keyboard");
    
    // First get vowel_e + consonant in the buffer
    process_char(&mut engine, 'a').unwrap();
    process_char(&mut engine, 'u').unwrap();
    
    // Type 'F' for stack (U1039)
    let result = process_char(&mut engine, 'F').unwrap();
    assert_eq!(result.composing_text, "\u{1000}\u{1031}\u{1039}");
    
    // Type 'u' for က (U1000) - should reorder
    let result = process_char(&mut engine, 'u').unwrap();
    assert_eq!(result.composing_text, "\u{1000}\u{1039}\u{1000}\u{1031}");
    assert_eq!(result.action, ActionType::BackspaceDeleteAndInsert(2, "\u{1039}\u{1000}\u{1031}".to_string()));
}

#[test]
fn test_vowel_e_reordering_with_medial() {
    // Test vowel_e reordering with medial characters
    // Line 106: u1031 + $mediaK[*] => $mediaU[$2] + u1031
    // This reorders vowel_e + medial to medial + vowel_e
    
    let kms_path = fixtures_dir().join("MyanSan.kms");
    let km2_file = kms2km2::compile_kms_file(&kms_path)
        .expect("Failed to compile MyanSan.kms");
    
    let km2_binary = create_km2_binary(&km2_file)
        .expect("Failed to create KM2 binary");
    
    let mut engine = create_engine_from_binary(&km2_binary).expect("Failed to load keyboard");
    
    // Get filler + vowel_e first
    process_char(&mut engine, 'a').unwrap();
    
    // Type 's' for ya-yit medial (U103B)
    let result = process_char(&mut engine, 's').unwrap();
    assert_eq!(result.action, ActionType::BackspaceDeleteAndInsert(1, "\u{103B}\u{1031}".to_string()));
}

#[test]
fn test_vowel_e_complex_reordering() {
    // Test a more complex case: vowel_e with consonant and medial
    // Should apply multiple reordering rules
    
    let kms_path = fixtures_dir().join("MyanSan.kms");
    let km2_file = kms2km2::compile_kms_file(&kms_path)
        .expect("Failed to compile MyanSan.kms");
    
    let km2_binary = create_km2_binary(&km2_file)
        .expect("Failed to create KM2 binary");
    
    let mut engine = create_engine_from_binary(&km2_binary).expect("Failed to load keyboard");
    
    // Type 'a' (vowel_e), 'u' (က), 's' (ya-yit)
    // Should produce က + ya-yit + vowel_e
    process_char(&mut engine, 'a').unwrap();
    process_char(&mut engine, 'u').unwrap();
    let result = process_char(&mut engine, 's').unwrap();
    
    // The final result should be consonant + medial + vowel_e
    assert_eq!(result.action, ActionType::BackspaceDeleteAndInsert(1, "\u{103B}\u{1031}".to_string()));
}

#[test]
fn test_vowel_e_with_multiple_consonants() {
    // Test typing multiple consonants after vowel_e
    
    let kms_path = fixtures_dir().join("MyanSan.kms");
    let km2_file = kms2km2::compile_kms_file(&kms_path)
        .expect("Failed to compile MyanSan.kms");
    
    let km2_binary = create_km2_binary(&km2_file)
        .expect("Failed to create KM2 binary");
    
    let mut engine = create_engine_from_binary(&km2_binary).expect("Failed to load keyboard");
    
    // Type 'au' - should produce က + vowel_e
    process_char(&mut engine, 'a').unwrap();
    process_char(&mut engine, 'u').unwrap();
    assert_eq!(engine.composing_text(), "\u{1000}\u{1031}");
    
    // Type another 'i' (င - U1004) - should just append
    let result = process_char(&mut engine, 'i').unwrap();
    assert_eq!(result.action, ActionType::Insert("\u{1004}".to_string()));
}

#[test]
fn test_filler_removal_at_end() {
    // Test the rule at line 210: $filler + U1031 + U103F => $3 + $2
    // This removes filler when followed by U103F
    
    let kms_path = fixtures_dir().join("MyanSan.kms");
    let km2_file = kms2km2::compile_kms_file(&kms_path)
        .expect("Failed to compile MyanSan.kms");
    
    let km2_binary = create_km2_binary(&km2_file)
        .expect("Failed to create KM2 binary");
    
    let mut engine = create_engine_from_binary(&km2_binary).expect("Failed to load keyboard");
    
    // Type 'a' for filler + vowel_e
    process_char(&mut engine, 'a').unwrap();
    assert_eq!(engine.composing_text(), "\u{200A}\u{1031}");
    
    // Type 'O' which maps to U103F
    let result = process_char(&mut engine, 'O').unwrap();
    // Should remove filler and reorder
    assert_eq!(result.action, ActionType::BackspaceDeleteAndInsert(2, "\u{103F}\u{1031}".to_string()));
}