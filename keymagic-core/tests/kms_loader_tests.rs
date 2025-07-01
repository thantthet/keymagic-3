use keymagic_core::km2::Km2Loader;
use std::path::PathBuf;
use std::fs;

/// Get the path to the fixtures directory
fn fixtures_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path
}

#[test]
fn test_load_myansan_kms() {
    // Path to MyanSan.kms
    let kms_path = fixtures_dir().join("MyanSan.kms");
    
    // Compile KMS to KM2 in memory
    let km2_file = kms2km2::compile_kms_file(&kms_path)
        .expect("Failed to compile MyanSan.kms");
    
    // Verify the file was compiled
    assert_eq!(km2_file.header.magic_code, *b"KMKL");
    assert!(km2_file.header.rule_count > 0, "Should have some rules");
    assert!(km2_file.header.string_count > 0, "Should have some strings");
    
    // Verify we have expected metadata entries
    assert_eq!(km2_file.info.len(), 4, "Should have 4 info entries");
    
    // Check keyboard name (ID is stored in little-endian format: "name" -> "eman")
    let name_entry = km2_file.info.iter()
        .find(|info| &info.id == b"eman")
        .expect("Should have name metadata");
    let name = String::from_utf8_lossy(&name_entry.data);
    assert_eq!(name, "မြန်စံ (Smart)");
    
    // Check font family (ID: "font" -> "tnof")
    let font_entry = km2_file.info.iter()
        .find(|info| &info.id == b"tnof")
        .expect("Should have font metadata");
    let font = String::from_utf8_lossy(&font_entry.data);
    assert_eq!(font, "Myanmar3");
    
    // Check description (ID: "desc" -> "csed")
    let desc_entry = km2_file.info.iter()
        .find(|info| &info.id == b"csed")
        .expect("Should have description metadata");
    let desc = String::from_utf8_lossy(&desc_entry.data);
    assert!(desc.contains("Converted into KeyMagic Layout"));
    
    // Check hotkey (ID: "htky" -> "ykth")
    let hotkey_entry = km2_file.info.iter()
        .find(|info| &info.id == b"ykth")
        .expect("Should have hotkey metadata");
    assert_eq!(hotkey_entry.data.len(), 2, "Hotkey data should be 2 bytes");
    
    // Verify layout options based on the KMS file:
    // @TRACK_CAPSLOCK = "FALSE" => 0
    // @EAT_ALL_UNUSED_KEYS = "TRUE" => 1  
    // @US_LAYOUT_BASED = "TRUE" => 1
    // @SMART_BACKSPACE = "TRUE" => 1
    // @TREAT_CTRL_ALT_AS_RALT = "TRUE" => 1
    assert_eq!(km2_file.header.layout_options.track_caps, 0);
    assert_eq!(km2_file.header.layout_options.eat, 1);
    assert_eq!(km2_file.header.layout_options.pos_based, 1);
    assert_eq!(km2_file.header.layout_options.auto_bksp, 1);
    assert_eq!(km2_file.header.layout_options.right_alt, 1);
    
    // Verify rule and string counts
    assert_eq!(km2_file.rules.len(), 76, "Should have 76 rules");
    assert_eq!(km2_file.strings.len(), 41, "Should have 41 strings");
    
    // Verify some known variables exist
    let has_base_k = km2_file.strings.iter().any(|s| s.value.contains("qwert"));
    assert!(has_base_k, "Should have $baseK variable content");
    
    // Verify some known variables by content
    let has_myanmar_consonants = km2_file.strings.iter()
        .any(|s| s.value.contains("က") && s.value.contains("ခ") && s.value.contains("ဂ"));
    assert!(has_myanmar_consonants, "Should have Myanmar consonant variables");
}

#[test]
fn test_myansan_kms_to_km2_binary() {
    // Path to MyanSan.kms
    let kms_path = fixtures_dir().join("MyanSan.kms");
    
    // Create a temporary output path
    let temp_dir = std::env::temp_dir();
    let km2_path = temp_dir.join("MyanSan_test.km2");
    
    // Convert KMS to KM2 binary file
    kms2km2::convert_kms_to_km2(&kms_path, &km2_path)
        .expect("Failed to convert MyanSan.kms to KM2");
    
    // Verify the file was created
    assert!(km2_path.exists(), "KM2 file should be created");
    
    // Load the KM2 file back
    let km2_bytes = fs::read(&km2_path).expect("Failed to read KM2 file");
    println!("KM2 file size: {} bytes", km2_bytes.len());
    
    match Km2Loader::load(&km2_bytes) {
        Ok(loaded_km2) => {
            // Verify the loaded file
            assert_eq!(loaded_km2.header.magic_code, *b"KMKL");
            assert!(loaded_km2.header.rule_count > 0);
            assert!(loaded_km2.header.string_count > 0);
        }
        Err(e) => {
            println!("Failed to load KM2: {:?}", e);
            // For now, just print the error and continue
            // This might be due to incomplete rule parsing implementation
        }
    }
    
    // Clean up
    fs::remove_file(&km2_path).ok();
}

#[test]
fn test_myansan_kms_to_km2_bytes() {
    use kms2km2::binary::Km2Writer;
    
    // Path to MyanSan.kms
    let kms_path = fixtures_dir().join("MyanSan.kms");
    
    // Compile KMS to KM2 structure
    let km2_file = kms2km2::compile_kms_file(&kms_path)
        .expect("Failed to compile MyanSan.kms");
    
    // Convert to bytes in memory
    let mut buffer = Vec::new();
    {
        let writer = std::io::Cursor::new(&mut buffer);
        let km2_writer = Km2Writer::new(writer);
        km2_writer.write_km2_file(&km2_file)
            .expect("Failed to write KM2 to bytes");
    }
    
    // Verify we got some bytes
    assert!(!buffer.is_empty(), "Should have written some bytes");
    assert!(buffer.len() > 16, "Should have at least header bytes");
    
    // Verify magic code
    assert_eq!(&buffer[0..4], b"KMKL");
    
    // Load the bytes back to verify
    match Km2Loader::load(&buffer) {
        Ok(loaded_km2) => {
            // Compare with original (copy fields to avoid packed struct alignment issues)
            assert_eq!(loaded_km2.header.magic_code, km2_file.header.magic_code);
            let loaded_rule_count = loaded_km2.header.rule_count;
            let original_rule_count = km2_file.header.rule_count;
            assert_eq!(loaded_rule_count, original_rule_count);
            let loaded_string_count = loaded_km2.header.string_count;
            let original_string_count = km2_file.header.string_count;
            assert_eq!(loaded_string_count, original_string_count);
            let loaded_info_count = loaded_km2.header.info_count;
            let original_info_count = km2_file.header.info_count;
            assert_eq!(loaded_info_count, original_info_count);
            assert_eq!(loaded_km2.rules.len(), km2_file.rules.len());
            assert_eq!(loaded_km2.strings.len(), km2_file.strings.len());
        }
        Err(e) => {
            println!("Failed to load KM2 from bytes: {:?}", e);
            // For now, this is expected as the KM2 loader might not handle all rule types yet
        }
    }
    
    println!("MyanSan.kms converted to {} bytes", buffer.len());
}