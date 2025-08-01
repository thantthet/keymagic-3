use kms2km2::compile_kms;
use keymagic_core::KmsError;

#[test]
fn test_compile_fails_on_bom_without_stripping() {
    // This tests that if someone bypasses the file reading (which strips BOM)
    // and directly compiles a string with BOM, it properly fails
    let bom_content = "\u{FEFF}\"ka\" => \"က\"";
    
    let result = compile_kms(bom_content);
    
    assert!(result.is_err(), "Compilation should fail on BOM");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { line, message } => {
                assert_eq!(line, 1);
                assert!(message.contains("Unexpected token"));
            }
            _ => panic!("Expected Parse error, got {:?}", e),
        }
    }
}

#[test]
fn test_compile_succeeds_on_normal_content() {
    let content = r#"
/*
@NAME = "Test Keyboard"
*/

"ka" => "က"
"#;
    
    let result = compile_kms(content);
    
    assert!(result.is_ok(), "Compilation should succeed on valid content");
    
    if let Ok(km2) = result {
        assert_eq!(km2.rules.len(), 1);
        let info_count = km2.header.info_count;
        assert_eq!(info_count, 1); // NAME info
    }
}