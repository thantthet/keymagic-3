use kms2km2::*;
use std::fs;
use std::env;

#[test]
fn test_icon_file_not_found() {
    let temp_dir = env::temp_dir().join("kms2km2_icon_test1");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create KMS file with non-existent icon
    let kms_content = r#"
/*
@NAME = "Icon Test"
@ICON = "does_not_exist.png"
*/

"test" => "output"
"#;
    
    let kms_path = temp_dir.join("test.kms");
    fs::write(&kms_path, kms_content).expect("Failed to write KMS");
    
    // Should fail with icon not found error
    let result = compile_kms_file(&kms_path);
    assert!(result.is_err(), "Expected missing icon to fail");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { message, .. } => {
                assert!(message.contains("Icon file not found"), 
                    "Expected 'Icon file not found' error, got: {}", message);
            }
            _ => panic!("Expected Parse error, got: {:?}", e),
        }
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_invalid_icon_format() {
    let temp_dir = env::temp_dir().join("kms2km2_icon_test2");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create a text file pretending to be an icon
    let fake_icon_path = temp_dir.join("fake_icon.png");
    fs::write(&fake_icon_path, "This is not an image file").expect("Failed to write fake icon");
    
    // Create KMS file
    let kms_content = r#"
/*
@NAME = "Invalid Icon Test"
@ICON = "fake_icon.png"
*/

"test" => "output"
"#;
    
    let kms_path = temp_dir.join("test.kms");
    fs::write(&kms_path, kms_content).expect("Failed to write KMS");
    
    // Should fail with invalid format error
    let result = compile_kms_file(&kms_path);
    assert!(result.is_err(), "Expected invalid icon format to fail");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { message, .. } => {
                assert!(message.contains("Invalid image format") && message.contains("PNG, BMP, or JPG/JPEG"), 
                    "Expected 'Invalid image format' error mentioning PNG, BMP, or JPG/JPEG, got: {}", message);
            }
            _ => panic!("Expected Parse error, got: {:?}", e),
        }
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_valid_png_icon() {
    let temp_dir = env::temp_dir().join("kms2km2_icon_test3");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create a minimal valid PNG file (1x1 red pixel)
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,  // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,  // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41,  // IDAT chunk
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
        0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
        0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,  // IEND chunk
        0x44, 0xAE, 0x42, 0x60, 0x82
    ];
    
    let icon_path = temp_dir.join("icon.png");
    fs::write(&icon_path, png_data).expect("Failed to write PNG icon");
    
    // Create KMS file
    let kms_content = r#"
/*
@NAME = "Valid PNG Icon Test"
@ICON = "icon.png"
*/

"test" => "output"
"#;
    
    let kms_path = temp_dir.join("test.kms");
    fs::write(&kms_path, kms_content).expect("Failed to write KMS");
    
    // Should compile successfully
    let result = compile_kms_file(&kms_path);
    assert!(result.is_ok(), "Expected valid PNG icon to succeed: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_valid_bmp_icon() {
    let temp_dir = env::temp_dir().join("kms2km2_icon_test4");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create a minimal valid BMP file header
    let bmp_data = vec![
        0x42, 0x4D,             // BM signature
        0x3A, 0x00, 0x00, 0x00, // File size
        0x00, 0x00, 0x00, 0x00, // Reserved
        0x36, 0x00, 0x00, 0x00, // Data offset
        0x28, 0x00, 0x00, 0x00, // DIB header size
        0x01, 0x00, 0x00, 0x00, // Width
        0x01, 0x00, 0x00, 0x00, // Height
        0x01, 0x00,             // Planes
        0x18, 0x00,             // Bits per pixel
        0x00, 0x00, 0x00, 0x00, // Compression
        0x04, 0x00, 0x00, 0x00, // Image size
        0x00, 0x00, 0x00, 0x00, // X pixels per meter
        0x00, 0x00, 0x00, 0x00, // Y pixels per meter
        0x00, 0x00, 0x00, 0x00, // Colors used
        0x00, 0x00, 0x00, 0x00, // Important colors
        0xFF, 0x00, 0x00, 0x00  // Pixel data (blue)
    ];
    
    let icon_path = temp_dir.join("icon.bmp");
    fs::write(&icon_path, bmp_data).expect("Failed to write BMP icon");
    
    // Create KMS file
    let kms_content = r#"
/*
@NAME = "Valid BMP Icon Test"
@ICON = "icon.bmp"
*/

"test" => "output"
"#;
    
    let kms_path = temp_dir.join("test.kms");
    fs::write(&kms_path, kms_content).expect("Failed to write KMS");
    
    // Should compile successfully
    let result = compile_kms_file(&kms_path);
    assert!(result.is_ok(), "Expected valid BMP icon to succeed: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_valid_jpeg_icon() {
    let temp_dir = env::temp_dir().join("kms2km2_icon_test5");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create a minimal valid JPEG file (very small 1x1)
    let jpeg_data = vec![
        0xFF, 0xD8, 0xFF, 0xE0, // SOI + APP0 marker
        0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, // JFIF header
        0x01, 0x01, 0x00, 0x48, 0x00, 0x48, 0x00, 0x00,
        0xFF, 0xDB, 0x00, 0x43, // DQT marker
        0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08, // Quantization table
        0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C,
        0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
        0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D,
        0x1A, 0x1C, 0x1C, 0x20, 0x24, 0x2E, 0x27, 0x20,
        0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29,
        0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27,
        0x39, 0x3D, 0x38, 0x32, 0x3C, 0x2E, 0x33, 0x34,
        0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, // SOF0 marker
        0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, // DHT marker
        0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x09, 0xFF, 0xC4, 0x00, 0x14,
        0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, // SOS marker
        0x00, 0x00, 0x3F, 0x00, 0x7F, 0x35, 0xFF, 0xD9  // EOI marker
    ];
    
    let icon_path = temp_dir.join("icon.jpg");
    fs::write(&icon_path, jpeg_data).expect("Failed to write JPEG icon");
    
    // Create KMS file
    let kms_content = r#"
/*
@NAME = "Valid JPEG Icon Test"
@ICON = "icon.jpg"
*/

"test" => "output"
"#;
    
    let kms_path = temp_dir.join("test.kms");
    fs::write(&kms_path, kms_content).expect("Failed to write KMS");
    
    // Should compile successfully
    let result = compile_kms_file(&kms_path);
    assert!(result.is_ok(), "Expected valid JPEG icon to succeed: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_icon_too_small() {
    let temp_dir = env::temp_dir().join("kms2km2_icon_test6");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create a file that's too small to be a valid image
    let icon_path = temp_dir.join("tiny.png");
    fs::write(&icon_path, vec![0x89, 0x50]).expect("Failed to write tiny file");
    
    // Create KMS file
    let kms_content = r#"
/*
@NAME = "Tiny Icon Test"
@ICON = "tiny.png"
*/

"test" => "output"
"#;
    
    let kms_path = temp_dir.join("test.kms");
    fs::write(&kms_path, kms_content).expect("Failed to write KMS");
    
    // Should fail with too small error
    let result = compile_kms_file(&kms_path);
    assert!(result.is_err(), "Expected tiny icon to fail");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { message, .. } => {
                assert!(message.contains("too small"), 
                    "Expected 'too small' error, got: {}", message);
            }
            _ => panic!("Expected Parse error, got: {:?}", e),
        }
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_icon_too_large() {
    let temp_dir = env::temp_dir().join("kms2km2_icon_test7");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create a large fake PNG file (over 1MB)
    let mut large_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,  // PNG signature
    ];
    large_data.resize(1024 * 1024 + 1000, 0); // Just over 1MB
    
    let icon_path = temp_dir.join("large.png");
    fs::write(&icon_path, large_data).expect("Failed to write large icon");
    
    // Create KMS file
    let kms_content = r#"
/*
@NAME = "Large Icon Test"
@ICON = "large.png"
*/

"test" => "output"
"#;
    
    let kms_path = temp_dir.join("test.kms");
    fs::write(&kms_path, kms_content).expect("Failed to write KMS");
    
    // Should fail with too large error
    let result = compile_kms_file(&kms_path);
    assert!(result.is_err(), "Expected large icon to fail");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { message, .. } => {
                assert!(message.contains("too large"), 
                    "Expected 'too large' error, got: {}", message);
            }
            _ => panic!("Expected Parse error, got: {:?}", e),
        }
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ico_format_not_supported() {
    let temp_dir = env::temp_dir().join("kms2km2_icon_test8");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create a valid ICO file header
    let ico_data = vec![
        0x00, 0x00, 0x01, 0x00,  // ICO header
        0x01, 0x00,              // 1 image
        0x10, 0x10,              // 16x16
        0x00, 0x00,              // No palette
        0x01, 0x00,              // 1 plane
        0x08, 0x00,              // 8 bits per pixel
        0x68, 0x00, 0x00, 0x00,  // Size
        0x16, 0x00, 0x00, 0x00,  // Offset
    ];
    
    let icon_path = temp_dir.join("icon.ico");
    fs::write(&icon_path, ico_data).expect("Failed to write ICO icon");
    
    // Create KMS file
    let kms_content = r#"
/*
@NAME = "ICO Format Test"
@ICON = "icon.ico"
*/

"test" => "output"
"#;
    
    let kms_path = temp_dir.join("test.kms");
    fs::write(&kms_path, kms_content).expect("Failed to write KMS");
    
    // Should fail since ICO is not supported
    let result = compile_kms_file(&kms_path);
    assert!(result.is_err(), "Expected ICO format to fail");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { message, .. } => {
                assert!(message.contains("Invalid image format") && message.contains("PNG, BMP, or JPG/JPEG"), 
                    "Expected error about unsupported ICO format, got: {}", message);
            }
            _ => panic!("Expected Parse error, got: {:?}", e),
        }
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_empty_icon_path() {
    let temp_dir = env::temp_dir().join("kms2km2_icon_test9");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create KMS file with empty icon path
    let kms_content = r#"
/*
@NAME = "Empty Icon Path Test"
@ICON = ""
*/

"test" => "output"
"#;
    
    let kms_path = temp_dir.join("test.kms");
    fs::write(&kms_path, kms_content).expect("Failed to write KMS");
    
    // Should fail with empty path error
    let result = compile_kms_file(&kms_path);
    assert!(result.is_err(), "Expected empty icon path to fail");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { message, .. } => {
                assert!(message.contains("Icon path cannot be empty"), 
                    "Expected 'Icon path cannot be empty' error, got: {}", message);
            }
            _ => panic!("Expected Parse error, got: {:?}", e),
        }
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_whitespace_only_icon_path() {
    let temp_dir = env::temp_dir().join("kms2km2_icon_test10");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create KMS file with whitespace-only icon path
    let kms_content = r#"
/*
@NAME = "Whitespace Icon Path Test"
@ICON = "   "
*/

"test" => "output"
"#;
    
    let kms_path = temp_dir.join("test.kms");
    fs::write(&kms_path, kms_content).expect("Failed to write KMS");
    
    // Should fail with empty path error
    let result = compile_kms_file(&kms_path);
    assert!(result.is_err(), "Expected whitespace-only icon path to fail");
    
    if let Err(e) = result {
        match e {
            KmsError::Parse { message, .. } => {
                assert!(message.contains("Icon path cannot be empty"), 
                    "Expected 'Icon path cannot be empty' error, got: {}", message);
            }
            _ => panic!("Expected Parse error, got: {:?}", e),
        }
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}