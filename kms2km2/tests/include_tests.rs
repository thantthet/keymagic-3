use kms2km2::*;
use std::fs;
use std::env;

#[test]
fn test_nested_includes() {
    let temp_dir = env::temp_dir().join("kms2km2_nested_test");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create deeply nested include file
    let deep_content = r#"
// Deeply nested rules
"deep1" => "deep_output1"
"#;
    let deep_path = temp_dir.join("deep.kms");
    fs::write(&deep_path, deep_content).expect("Failed to write deep include file");
    
    // Create nested include file
    let nested_content = r#"
// Nested rules
"nested1" => "nested_output1"

include("deep.kms")

"nested2" => "nested_output2"
"#;
    let nested_path = temp_dir.join("nested.kms");
    fs::write(&nested_path, nested_content).expect("Failed to write nested include file");
    
    // Create main file with include
    let main_content = r#"
/*
@NAME = "Nested Include Test"
*/

"main1" => "main_output1"

include("nested.kms")

"main2" => "main_output2"
"#;
    
    let main_path = temp_dir.join("main.kms");
    fs::write(&main_path, main_content).expect("Failed to write main KMS");
    
    let km2 = compile_kms_file(&main_path).expect("Failed to compile KMS");
    
    // Should have rules from all files
    assert_eq!(km2.rules.len(), 5, "Expected 5 rules (2 main + 2 nested + 1 deep), got {}", km2.rules.len());
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_circular_include_detection() {
    let temp_dir = env::temp_dir().join("kms2km2_circular_test");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create file A that includes B
    let file_a_content = r#"
"rule_a" => "output_a"
include("file_b.kms")
"#;
    let file_a_path = temp_dir.join("file_a.kms");
    fs::write(&file_a_path, file_a_content).expect("Failed to write file A");
    
    // Create file B that includes A (circular)
    let file_b_content = r#"
"rule_b" => "output_b"
include("file_a.kms")
"#;
    let file_b_path = temp_dir.join("file_b.kms");
    fs::write(&file_b_path, file_b_content).expect("Failed to write file B");
    
    // Should fail with circular include error
    let result = compile_kms_file(&file_a_path);
    assert!(result.is_err(), "Expected circular include to fail");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { message, .. } => {
                assert!(message.contains("Circular include"), 
                    "Expected circular include error, got: {}", message);
            }
            _ => panic!("Expected Parse error, got: {:?}", e),
        }
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_missing_include_file() {
    let temp_dir = env::temp_dir().join("kms2km2_missing_test");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create main file that includes non-existent file
    let main_content = r#"
"main1" => "main_output1"
include("does_not_exist.kms")
"main2" => "main_output2"
"#;
    
    let main_path = temp_dir.join("main.kms");
    fs::write(&main_path, main_content).expect("Failed to write main KMS");
    
    // Should fail with IO error
    let result = compile_kms_file(&main_path);
    assert!(result.is_err(), "Expected missing include to fail");
    
    if let Err(e) = result {
        match e {
            KmsError::Io(_) => {
                // Expected error type
            }
            _ => panic!("Expected IO error, got: {:?}", e),
        }
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_include_with_variables() {
    let temp_dir = env::temp_dir().join("kms2km2_vars_test");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create include file with variables
    let include_content = r#"
// Variables in included file
$included_var = "abc"
$another_var = U1000 + U1001

// Rules using variables
$included_var => "included_output"
"#;
    let include_path = temp_dir.join("vars.kms");
    fs::write(&include_path, include_content).expect("Failed to write include file");
    
    // Create main file
    let main_content = r#"
/*
@NAME = "Variable Include Test"
*/

$main_var = "xyz"

include("vars.kms")

// Rule using variable from included file
$included_var + "test" => "combined_output"
"#;
    
    let main_path = temp_dir.join("main.kms");
    fs::write(&main_path, main_content).expect("Failed to write main KMS");
    
    let km2 = compile_kms_file(&main_path).expect("Failed to compile KMS");
    
    // Should have variables from both files
    assert!(km2.strings.len() >= 3, "Expected at least 3 strings/variables");
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_absolute_path_include() {
    let temp_dir = env::temp_dir().join("kms2km2_abs_test");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create include file
    let include_content = r#"
"abs_rule" => "abs_output"
"#;
    let include_path = temp_dir.join("absolute.kms");
    fs::write(&include_path, include_content).expect("Failed to write include file");
    
    // Create main file with absolute path include
    let main_content = format!(r#"
"main_rule" => "main_output"
include("{}")
"#, include_path.to_string_lossy());
    
    let main_path = temp_dir.join("main.kms");
    fs::write(&main_path, &main_content).expect("Failed to write main KMS");
    
    let km2 = compile_kms_file(&main_path).expect("Failed to compile KMS");
    
    // Should have both rules
    assert_eq!(km2.rules.len(), 2, "Expected 2 rules");
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}