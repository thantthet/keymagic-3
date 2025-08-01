use kms2km2::parser::Parser;
use keymagic_core::KmsError;

#[test]
fn test_parser_fails_on_invalid_first_token() {
    // Test 1: BOM at the beginning should cause parse error, not silent failure
    let bom_content = "\u{FEFF}\"ka\" => \"က\"";
    let mut parser = Parser::new(bom_content);
    let result = parser.parse();
    
    assert!(result.is_err(), "Parser should fail when encountering BOM");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { line, message } => {
                assert_eq!(line, 1, "Error should be on line 1");
                assert!(message.contains("Unexpected token"), "Error should mention unexpected token");
                assert!(message.contains('\u{feff}') || message.contains("feff"), 
                    "Error should mention the BOM character");
            }
            _ => panic!("Expected Parse error, got {:?}", e),
        }
    }
}

#[test]
fn test_parser_fails_on_invalid_character() {
    // Test 2: Other invalid characters should also fail
    let invalid_content = "§invalid => \"output\"";
    let mut parser = Parser::new(invalid_content);
    let result = parser.parse();
    
    assert!(result.is_err(), "Parser should fail on invalid character");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { line, message } => {
                assert_eq!(line, 1);
                assert!(message.contains("Unexpected token") || message.contains("§"));
            }
            _ => panic!("Expected Parse error, got {:?}", e),
        }
    }
}

#[test]
fn test_parser_succeeds_on_valid_content() {
    // Test 3: Valid content should parse successfully
    let valid_content = r#"
/* @NAME = "Test" */
"ka" => "က"
"kha" => "ခ"
"#;
    
    let mut parser = Parser::new(valid_content);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should succeed on valid content");
    
    if let Ok(ast) = result {
        assert_eq!(ast.rules.len(), 2, "Should have 2 rules");
        assert_eq!(ast.options.get("NAME"), Some(&"Test".to_string()));
    }
}

#[test]
fn test_parser_empty_input() {
    // Test 4: Empty input should parse successfully (empty keyboard)
    let empty_content = "";
    let mut parser = Parser::new(empty_content);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle empty input");
    
    if let Ok(ast) = result {
        assert_eq!(ast.rules.len(), 0);
        assert_eq!(ast.variables.len(), 0);
    }
}

#[test]
fn test_parser_whitespace_only() {
    // Test 5: Whitespace-only input should parse successfully
    let whitespace_content = "   \n\t\r\n   ";
    let mut parser = Parser::new(whitespace_content);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle whitespace-only input");
}

#[test]
fn test_parser_invalid_after_valid() {
    // Test 6: Invalid token after valid content should fail at the right place
    let mixed_content = r#"
"ka" => "က"
§invalid
"#;
    
    let mut parser = Parser::new(mixed_content);
    let result = parser.parse();
    
    assert!(result.is_err(), "Parser should fail on invalid token");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { line, .. } => {
                assert!(line >= 2, "Error should be after line 1");
            }
            _ => panic!("Expected Parse error"),
        }
    }
}