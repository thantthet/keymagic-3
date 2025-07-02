use std::path::Path;
use std::fs;
use kms2km2::{convert_kms_to_km2, compile_kms_file};
use keymagic_core::*;

#[test]
fn test_simple_conversion() {
    let input_path = Path::new("tests/fixtures/simple_test.kms");
    let output_path = Path::new("target/simple_test.km2");
    
    // Ensure output doesn't exist
    let _ = fs::remove_file(output_path);
    
    // Convert
    let result = convert_kms_to_km2(input_path, output_path);
    assert!(result.is_ok(), "Conversion failed: {:?}", result);
    
    // Check output exists
    assert!(output_path.exists(), "Output file not created");
    
    // Read and verify basic structure
    let data = fs::read(output_path).expect("Failed to read output");
    
    // Check magic code
    assert_eq!(&data[0..4], b"KMKL", "Invalid magic code");
    
    // Check version
    assert_eq!(data[4], 1, "Invalid major version");
    assert_eq!(data[5], 5, "Invalid minor version");
    
    // Clean up
    let _ = fs::remove_file(output_path);
}

#[test]
fn test_state_handling() {
    // Create a KMS file with multiple states
    let kms_content = r#"
/*
@NAME = "State Test"
@DESCRIPTION = "Testing state handling"
*/

// Define three different states
<VK_KEY_1> => ('state_one')
<VK_KEY_2> => ('state_two') 
<VK_KEY_3> => ('state_three')

// Rules using states
('state_one') + "a" => "output1"
('state_two') + "b" => "output2"
('state_three') + "c" => "output3"

// State maintaining itself
('state_one') + ANY => $1 + ('state_one')
"#;
    
    let input_path = Path::new("target/state_test.kms");
    let output_path = Path::new("target/state_test.km2");
    
    // Write test file
    fs::write(input_path, kms_content).expect("Failed to write test KMS");
    
    // Convert
    let result = convert_kms_to_km2(input_path, output_path);
    assert!(result.is_ok(), "Conversion failed: {:?}", result);
    
    // Compile and check the KM2 structure
    let km2_result = compile_kms_file(input_path);
    assert!(km2_result.is_ok());
    let km2 = km2_result.unwrap();
    
    // Verify we have rules
    assert!(km2.rules.len() >= 7, "Expected at least 7 rules, got {}", km2.rules.len());
    
    // Check that state indices are unique by examining the compiled rules
    let mut state_indices = std::collections::HashSet::new();
    for rule in &km2.rules {
        for element in &rule.rhs {
            if let BinaryFormatElement::Switch(idx) = element {
                state_indices.insert(*idx);
            }
        }
    }
    
    // We should have 3 unique state indices
    assert_eq!(state_indices.len(), 3, "Expected 3 unique state indices, got {:?}", state_indices);
    
    // Verify state indices start from 0 (our implementation)
    let mut sorted_indices: Vec<_> = state_indices.iter().cloned().collect();
    sorted_indices.sort();
    assert_eq!(sorted_indices, vec![0, 1, 2], "State indices should be 0, 1, 2");
    
    // Clean up
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn test_comprehensive_conversion() {
    let input_path = Path::new("tests/fixtures/comprehensive_test.kms");
    let output_path = Path::new("target/comprehensive_test.km2");
    
    // Convert
    let result = convert_kms_to_km2(input_path, output_path);
    assert!(result.is_ok(), "Conversion failed: {:?}", result);
    
    // Compile to check structure
    let km2 = compile_kms_file(input_path).expect("Failed to compile KMS");
    
    // Verify strings were added
    assert!(km2.strings.len() > 0, "No strings in output");
    
    // Verify rules were compiled
    assert!(km2.rules.len() > 0, "No rules in output");
    
    // Check for various rule types
    let mut has_string_rule = false;
    let mut has_variable_rule = false;
    let mut has_vk_rule = false;
    let mut has_state_rule = false;
    let mut has_anyof_pattern = false;
    let mut has_reference_rule = false;
    
    for rule in &km2.rules {
        // Check for ANYOF pattern (Variable + Modifier(OP_ANYOF))
        let lhs_vec: Vec<_> = rule.lhs.iter().collect();
        for i in 0..lhs_vec.len() {
            if let BinaryFormatElement::Variable(_) = lhs_vec[i] {
                if i + 1 < lhs_vec.len() {
                    if let BinaryFormatElement::Modifier(op) = lhs_vec[i + 1] {
                        if *op == FLAG_ANYOF {
                            has_anyof_pattern = true;
                        }
                    }
                }
            }
            
            match lhs_vec[i] {
                BinaryFormatElement::String(_) => has_string_rule = true,
                BinaryFormatElement::Variable(_) => has_variable_rule = true,
                BinaryFormatElement::Predefined(_) => has_vk_rule = true,
                BinaryFormatElement::Switch(_) => has_state_rule = true,
                _ => {}
            }
        }
        
        for element in &rule.rhs {
            if let BinaryFormatElement::Reference(_) = element {
                has_reference_rule = true;
            }
        }
    }
    
    assert!(has_string_rule, "No string rules found");
    assert!(has_variable_rule, "No variable rules found");
    assert!(has_vk_rule, "No virtual key rules found");
    assert!(has_state_rule, "No state rules found");
    assert!(has_anyof_pattern, "No wildcard (ANYOF) patterns found");
    assert!(has_reference_rule, "No reference rules found");
    
    // Clean up
    let _ = fs::remove_file(output_path);
}

#[test]
fn test_options_parsing() {
    let kms_content = r#"
/*
@NAME = "Options Test"
@FONTFAMILY = "Myanmar3"
@DESCRIPTION = "Test all options"
@HOTKEY = "CTRL+SHIFT+M"
@TRACK_CAPSLOCK = "TRUE"
@EAT_ALL_UNUSED_KEYS = "FALSE"
@US_LAYOUT_BASED = "TRUE"
@SMART_BACKSPACE = "TRUE"
@TREAT_CTRL_ALT_AS_RALT = "FALSE"
*/

"ka" => "က"
"#;
    
    let input_path = Path::new("target/options_test.kms");
    fs::write(input_path, kms_content).expect("Failed to write test KMS");
    
    let km2 = compile_kms_file(input_path).expect("Failed to compile KMS");
    
    // Check options
    assert_eq!(km2.header.layout_options.track_caps, 1);
    assert_eq!(km2.header.layout_options.eat, 0);
    assert_eq!(km2.header.layout_options.pos_based, 1);
    assert_eq!(km2.header.layout_options.auto_bksp, 1);
    assert_eq!(km2.header.layout_options.right_alt, 0);
    
    // Check info
    assert!(km2.info.iter().any(|i| &i.id == b"eman"), "No name info found");
    assert!(km2.info.iter().any(|i| &i.id == b"tnof"), "No font info found");
    assert!(km2.info.iter().any(|i| &i.id == b"csed"), "No description info found");
    assert!(km2.info.iter().any(|i| &i.id == b"ykth"), "No hotkey info found");
    
    // Clean up
    let _ = fs::remove_file(input_path);
}

#[test]
fn test_unicode_handling() {
    let kms_content = r#"
/*
@NAME = "Unicode Test"
*/

// Various Unicode notations
U1000 => "က"
u1001 => "ခ"
"\u1002" => "ဂ"
"\x1003" => "ဃ"

// Unicode in variables
$test = U1000 + u1001 + u1002
$test => "matched"
"#;
    
    let input_path = Path::new("target/unicode_test.kms");
    fs::write(input_path, kms_content).expect("Failed to write test KMS");
    
    let km2 = compile_kms_file(input_path).expect("Failed to compile KMS");
    
    // Should have rules for each unicode notation
    assert!(km2.rules.len() >= 5, "Expected at least 5 rules");
    
    // Clean up
    let _ = fs::remove_file(input_path);
}

#[test]
fn test_error_handling() {
    // Test invalid syntax
    let kms_content = r#"
/*
@NAME = "Error Test"
*/

// Missing output
"test" =>

// Invalid syntax
=> "output"
"#;
    
    let input_path = Path::new("target/error_test.kms");
    fs::write(input_path, kms_content).expect("Failed to write test KMS");
    
    let result = compile_kms_file(input_path);
    assert!(result.is_err(), "Should fail on invalid syntax");
    
    // Clean up
    let _ = fs::remove_file(input_path);
}

#[test]
fn test_complex_rules() {
    let kms_content = r#"
/*  
@NAME = "Complex Rules Test"
*/

$consonants = "ကခဂဃ"
$vowels = "ါာိီ"

// Wildcard with back-reference
$consonants[*] + "a" => $1 + "ာ"

// Multiple wildcards
$consonants[*] + $vowels[*] => $2 + $1

// Negation
$consonants[^] + "x" => "not_consonant"

// ANY keyword
ANY + "test" => $1 + "_test"

// NULL output
"delete" + <VK_BACK> => NULL

// Complex key combinations
<VK_CTRL & VK_ALT & VK_KEY_A> => "special"
"#;
    
    let input_path = Path::new("target/complex_test.kms");
    fs::write(input_path, kms_content).expect("Failed to write test KMS");
    
    let km2 = compile_kms_file(input_path).expect("Failed to compile KMS");
    
    // Verify various rule elements exist
    // Note: AnyOf and NotAnyOf are represented as Variable + Modifier(OP_ANYOF/OP_NANYOF)
    let mut has_anyof_pattern = false;
    let mut has_nanyof_pattern = false;
    let mut has_any = false;
    let mut has_predefined = false;
    let mut has_modifier = false;
    let mut has_reference = false;
    
    for rule in &km2.rules {
        // Check for ANYOF pattern: Variable followed by Modifier(OP_ANYOF)
        let lhs_vec: Vec<_> = rule.lhs.iter().collect();
        let _rhs_vec: Vec<_> = rule.rhs.iter().collect();
        
        // Check LHS for patterns
        for i in 0..lhs_vec.len() {
            if let BinaryFormatElement::Variable(_) = lhs_vec[i] {
                if i + 1 < lhs_vec.len() {
                    if let BinaryFormatElement::Modifier(op) = lhs_vec[i + 1] {
                        match *op {
                            FLAG_ANYOF => has_anyof_pattern = true,
                            FLAG_NANYOF => has_nanyof_pattern = true,
                            _ => {}
                        }
                    }
                }
            }
            
            match lhs_vec[i] {
                BinaryFormatElement::Any => has_any = true,
                BinaryFormatElement::Predefined(_) => has_predefined = true,
                BinaryFormatElement::Modifier(_) => has_modifier = true,
                _ => {}
            }
        }
        
        // Check RHS for references
        for elem in &rule.rhs {
            if matches!(elem, BinaryFormatElement::Reference(_)) {
                has_reference = true;
            }
        }
    }
    
    assert!(has_anyof_pattern, "No ANYOF pattern (Variable + Modifier(OP_ANYOF)) found");
    assert!(has_nanyof_pattern, "No NANYOF pattern (Variable + Modifier(OP_NANYOF)) found");
    assert!(has_any, "No ANY rules found");
    assert!(has_predefined, "No predefined (VK) rules found");
    assert!(has_modifier, "No modifier rules found");
    assert!(has_reference, "No back-references found");
    
    // Clean up
    let _ = fs::remove_file(input_path);
}

#[test]
fn test_variable_expansion() {
    let kms_content = r#"
/*
@NAME = "Variable Test"
*/

// Test variable references and expansion
$var1 = "abc"
$var2 = $var1 + "def"
$var3 = U1000 + U1001
$var4 = $var3 + $var1

// Rules using variables
$var1 => "found var1"
$var2 => "found var2"
$var3 => "found var3"
$var4 => "found var4"
"#;
    
    let input_path = Path::new("target/variable_test.kms");
    fs::write(input_path, kms_content).expect("Failed to write test KMS");
    
    let km2 = compile_kms_file(input_path).expect("Failed to compile KMS");
    
    // Should have created the variables
    assert!(km2.strings.len() >= 4, "Expected at least 4 variable strings");
    
    // Should have rules for each variable
    assert!(km2.rules.len() >= 4, "Expected at least 4 rules");
    
    // Clean up
    let _ = fs::remove_file(input_path);
}

#[test]
fn test_state_transitions() {
    let kms_content = r#"
/*
@NAME = "State Transition Test"
*/

// Initial state transitions
<VK_KEY_1> => ('mode1')
<VK_KEY_2> => ('mode2')

// State to state transitions
('mode1') + <VK_KEY_2> => ('mode2')
('mode2') + <VK_KEY_1> => ('mode1')

// Exit state
('mode1') + <VK_ESCAPE> => "exit"
('mode2') + <VK_ESCAPE> => "exit"

// State-specific behavior
('mode1') + "a" => "A"
('mode2') + "a" => "α"
"#;
    
    let input_path = Path::new("target/state_transition_test.kms");
    fs::write(input_path, kms_content).expect("Failed to write test KMS");
    
    let km2 = compile_kms_file(input_path).expect("Failed to compile KMS");
    
    // Count state switches in patterns and outputs
    let mut pattern_switches = 0;
    let mut output_switches = 0;
    
    for rule in &km2.rules {
        for element in &rule.lhs {
            if matches!(element, BinaryFormatElement::Switch(_)) {
                pattern_switches += 1;
            }
        }
        for element in &rule.rhs {
            if matches!(element, BinaryFormatElement::Switch(_)) {
                output_switches += 1;
            }
        }
    }
    
    assert!(pattern_switches >= 4, "Expected at least 4 pattern switches");
    assert!(output_switches >= 4, "Expected at least 4 output switches");
    
    // Clean up
    let _ = fs::remove_file(input_path);
}

#[test]
#[ignore = "Include directive not yet implemented"]
fn test_include_directive() {
    // Create an include file
    let include_content = r#"
// Included rules
"include1" => "included_output1"
"include2" => "included_output2"
"#;
    
    let include_path = Path::new("target/included.kms");
    fs::write(include_path, include_content).expect("Failed to write include file");
    
    // Create main file with include
    let kms_content = r#"
/*
@NAME = "Include Test"
*/

"main1" => "main_output1"

include("target/included.kms")

"main2" => "main_output2"
"#;
    
    let input_path = Path::new("target/include_test.kms");
    fs::write(input_path, kms_content).expect("Failed to write test KMS");
    
    let km2 = compile_kms_file(input_path).expect("Failed to compile KMS");
    
    // Should have rules from both main and included files
    assert!(km2.rules.len() >= 4, "Expected at least 4 rules (2 main + 2 included)");
    
    // Clean up
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(include_path);
}

#[test]
fn test_modifier_combinations() {
    let kms_content = r#"
/*
@NAME = "Modifier Test"
*/

// Single modifiers
<VK_SHIFT & VK_KEY_A> => "Shift+A"
<VK_CTRL & VK_KEY_B> => "Ctrl+B"
<VK_ALT & VK_KEY_C> => "Alt+C"

// Multiple modifiers
<VK_CTRL & VK_SHIFT & VK_KEY_D> => "Ctrl+Shift+D"
<VK_CTRL & VK_ALT & VK_KEY_E> => "Ctrl+Alt+E"
<VK_SHIFT & VK_ALT & VK_KEY_F> => "Shift+Alt+F"

// All modifiers
<VK_CTRL & VK_SHIFT & VK_ALT & VK_KEY_G> => "Ctrl+Shift+Alt+G"

// Alt Gr
<VK_ALT_GR & VK_KEY_H> => "AltGr+H"
"#;
    
    let input_path = Path::new("target/modifier_test.kms");
    fs::write(input_path, kms_content).expect("Failed to write test KMS");
    
    let km2 = compile_kms_file(input_path).expect("Failed to compile KMS");
    
    // Count key combinations and AND operators
    // Note: Modifier keys (Shift, Ctrl, Alt) are represented as Predefined values
    let mut and_count = 0;
    let mut predefined_count = 0;
    let mut rules_with_modifiers = 0;
    
    for rule in &km2.rules {
        let has_and = rule.lhs.iter().any(|e| matches!(e, BinaryFormatElement::And));
        if has_and {
            rules_with_modifiers += 1;
        }
        
        for element in &rule.lhs {
            match element {
                BinaryFormatElement::And => and_count += 1,
                BinaryFormatElement::Predefined(_) => predefined_count += 1,
                _ => {}
            }
        }
    }
    
    // We have 8 rules with key combinations
    assert!(rules_with_modifiers >= 8, "Expected at least 8 rules with modifiers, got {}", rules_with_modifiers);
    assert!(and_count >= 7, "Expected at least 7 AND operators for key combinations, got {}", and_count);
    assert!(predefined_count >= 16, "Expected at least 16 predefined keys (modifiers + keys), got {}", predefined_count);
    
    // Clean up
    let _ = fs::remove_file(input_path);
}