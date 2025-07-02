mod common;

use common::*;
use keymagic_core::{BinaryFormatElement, VirtualKey, km2::{Km2Loader, Km2Error}};

#[test]
fn test_invalid_predefined_usage() {
    // Create a KM2 file with a rule that has standalone Predefined in LHS
    let mut km2 = create_basic_km2();
    
    // Rule with standalone Predefined (invalid)
    // LHS: Predefined(VK_SPACE)
    add_rule(&mut km2,
        vec![BinaryFormatElement::Predefined(VirtualKey::Space as u16)],
        vec![BinaryFormatElement::String("test".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let result = Km2Loader::load(&binary);
    
    // Should fail with InvalidRule error
    assert!(matches!(result, Err(Km2Error::InvalidRule(0))));
}

#[test]
fn test_valid_predefined_with_and() {
    // Create a KM2 file with a valid virtual key combination
    let mut km2 = create_basic_km2();
    
    // Rule with valid combination: Predefined + AND + Predefined
    // LHS: Predefined(VK_SHIFT) + And + Predefined(VK_KEY_A)
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Predefined(VirtualKey::Shift as u16),
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::KeyA as u16)
        ],
        vec![BinaryFormatElement::String("A".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let result = Km2Loader::load(&binary);
    
    // Should load successfully
    assert!(result.is_ok());
}

#[test]
fn test_multiple_standalone_predefined_invalid() {
    // Test multiple standalone Predefined elements in the same rule
    let mut km2 = create_basic_km2();
    
    // Rule with multiple invalid Predefined elements
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::String("test".to_string()),
            BinaryFormatElement::Predefined(VirtualKey::Space as u16),
            BinaryFormatElement::String("more".to_string())
        ],
        vec![BinaryFormatElement::String("output".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let result = Km2Loader::load(&binary);
    
    // Should fail validation
    assert!(matches!(result, Err(Km2Error::InvalidRule(0))));
}

#[test]
fn test_complex_valid_virtual_key_combination() {
    // Test a complex but valid virtual key combination
    let mut km2 = create_basic_km2();
    
    // LHS: Ctrl + Alt + Shift + K
    add_rule(&mut km2,
        vec![
            BinaryFormatElement::Predefined(VirtualKey::Control as u16),
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::Menu as u16), // Alt
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::Shift as u16),
            BinaryFormatElement::And,
            BinaryFormatElement::Predefined(VirtualKey::KeyK as u16)
        ],
        vec![BinaryFormatElement::String("Special K".to_string())]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let result = Km2Loader::load(&binary);
    
    // Should load successfully
    assert!(result.is_ok());
}

#[test]
fn test_predefined_in_rhs_allowed() {
    // Predefined elements should be allowed in RHS without restrictions
    let mut km2 = create_basic_km2();
    
    // Valid rule with Predefined in RHS
    add_rule(&mut km2,
        vec![BinaryFormatElement::String("test".to_string())],
        vec![
            BinaryFormatElement::String("output".to_string()),
            BinaryFormatElement::Predefined(VirtualKey::Space as u16)
        ]
    );
    
    let binary = create_km2_binary(&km2).unwrap();
    let result = Km2Loader::load(&binary);
    
    // Should load successfully (validation only applies to LHS)
    assert!(result.is_ok());
}