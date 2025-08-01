use std::fs;
use std::path::Path;
use kms2km2::compile_kms_file;

#[test]
fn test_utf8_with_bom() {
    // UTF-8 BOM is the byte sequence EF BB BF
    let utf8_bom = b"\xEF\xBB\xBF";
    
    let kms_content = r#"/*
@NAME = "UTF-8 BOM Test"
@DESCRIPTION = "Test UTF-8 file with BOM"
*/

// Test with Myanmar text
$myanmar = "မြန်မာ"

// Simple rule
"ka" => "က"
"ma" => "မ"

// Rule with Myanmar variable
$myanmar => "Myanmar text found"
"#;

    // Create content with BOM
    let mut content_with_bom = Vec::from(utf8_bom);
    content_with_bom.extend_from_slice(kms_content.as_bytes());
    
    let input_path = Path::new("target/utf8_bom_test.kms");
    
    // Write file with BOM
    fs::write(input_path, &content_with_bom).expect("Failed to write test file");
    
    // Try to compile the file
    let result = compile_kms_file(input_path);
    
    // Should compile successfully
    assert!(result.is_ok(), "Failed to compile UTF-8 file with BOM: {:?}", result.err());
    
    let km2 = result.unwrap();
    
    // Verify the content was parsed correctly
    let info_count = km2.header.info_count;
    assert_eq!(info_count, 2, "Expected 2 info entries");
    assert!(km2.rules.len() >= 3, "Expected at least 3 rules");
    
    // Find the Myanmar variable in strings
    let myanmar_string = km2.strings.iter()
        .find(|s| s.value == "မြန်မာ")
        .expect("Myanmar variable string not found");
    
    assert_eq!(myanmar_string.value, "မြန်မာ", "Myanmar text not correctly parsed");
    
    // Clean up
    let _ = fs::remove_file(input_path);
}

#[test]
fn test_utf8_without_bom() {
    let kms_content = r#"/*
@NAME = "UTF-8 No BOM Test"
@DESCRIPTION = "Test UTF-8 file without BOM"
*/

// Test with Myanmar text
$myanmar = "မြန်မာ"

// Simple rule
"ka" => "က"
"#;

    let input_path = Path::new("target/utf8_no_bom_test.kms");
    
    // Write file without BOM
    fs::write(input_path, kms_content).expect("Failed to write test file");
    
    // Try to compile the file
    let result = compile_kms_file(input_path);
    
    // Should compile successfully
    assert!(result.is_ok(), "Failed to compile UTF-8 file without BOM: {:?}", result.err());
    
    let km2 = result.unwrap();
    
    // Find the Myanmar variable in strings
    let myanmar_string = km2.strings.iter()
        .find(|s| s.value == "မြန်မာ")
        .expect("Myanmar variable string not found");
    
    assert_eq!(myanmar_string.value, "မြန်မာ", "Myanmar text not correctly parsed");
    
    // Clean up
    let _ = fs::remove_file(input_path);
}

#[test]
fn test_utf8_bom_with_include() {
    // Create an include file with BOM
    let utf8_bom = b"\xEF\xBB\xBF";
    let include_content = r#"// Included file with BOM
"include_rule" => "included_output"
"#;
    
    let mut include_with_bom = Vec::from(utf8_bom);
    include_with_bom.extend_from_slice(include_content.as_bytes());
    
    let include_path = Path::new("target/include_with_bom.kms");
    fs::write(include_path, &include_with_bom).expect("Failed to write include file");
    
    // Create main file with BOM that includes the other file
    let main_content = r#"/*
@NAME = "Main with BOM"
*/

"main_rule" => "main_output"

include("include_with_bom.kms")
"#;
    
    let mut main_with_bom = Vec::from(utf8_bom);
    main_with_bom.extend_from_slice(main_content.as_bytes());
    
    let main_path = Path::new("target/main_with_bom.kms");
    fs::write(main_path, &main_with_bom).expect("Failed to write main file");
    
    // Try to compile
    let result = compile_kms_file(main_path);
    
    // Should compile successfully
    assert!(result.is_ok(), "Failed to compile files with BOM: {:?}", result.err());
    
    let km2 = result.unwrap();
    
    // Should have both rules
    assert!(km2.rules.len() >= 2, "Expected at least 2 rules");
    
    // Clean up
    let _ = fs::remove_file(main_path);
    let _ = fs::remove_file(include_path);
}