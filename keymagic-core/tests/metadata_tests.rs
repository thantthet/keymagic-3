mod common;

use common::*;
use keymagic_core::{LayoutOptions, km2::Km2Loader};

#[test]
fn test_metadata_name() {
    // Test @NAME metadata option
    let mut km2 = create_basic_km2();
    add_info_text(&mut km2, "name", "Myanmar Unicode");
    
    let binary = create_km2_binary(&km2).unwrap();
    
    // Load the KM2 file
    let loaded = Km2Loader::load(&binary).unwrap();
    assert_eq!(loaded.info.len(), 1);
    
    let name_entry = &loaded.info[0];
    assert_eq!(&name_entry.id, b"name");
    let text = decode_utf8_text(&name_entry.data);
    assert_eq!(text, "Myanmar Unicode");
}

#[test]
fn test_metadata_description() {
    // Test @DESCRIPTION metadata option
    let mut km2 = create_basic_km2();
    add_info_text(&mut km2, "desc", "Test keyboard layout");
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let desc_entry = &loaded.info[0];
    assert_eq!(&desc_entry.id, b"desc");
    let text = decode_utf8_text(&desc_entry.data);
    assert_eq!(text, "Test keyboard layout");
}

#[test]
fn test_metadata_font_family() {
    // Test @FONTFAMILY metadata option
    let mut km2 = create_basic_km2();
    add_info_text(&mut km2, "font", "Myanmar3");
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    let font_entry = &loaded.info[0];
    assert_eq!(&font_entry.id, b"font");
    let text = decode_utf8_text(&font_entry.data);
    assert_eq!(text, "Myanmar3");
}

#[test]
fn test_layout_options_track_capslock() {
    // Test @TRACK_CAPSLOCK = "TRUE"
    let options = LayoutOptions {
        track_caps: 1,
        auto_bksp: 0,
        eat: 0,
        pos_based: 0,
        right_alt: 0,
    };
    
    let km2 = create_km2_with_options(options);
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    assert_eq!(loaded.header.layout_options.track_caps, 1);
    assert_eq!(loaded.header.layout_options.auto_bksp, 0);
}

#[test]
fn test_layout_options_smart_backspace() {
    // Test @SMART_BACKSPACE = "TRUE"
    let options = LayoutOptions {
        track_caps: 0,
        auto_bksp: 1,
        eat: 0,
        pos_based: 0,
        right_alt: 0,
    };
    
    let km2 = create_km2_with_options(options);
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    assert_eq!(loaded.header.layout_options.track_caps, 0);
    assert_eq!(loaded.header.layout_options.auto_bksp, 1);
}

#[test]
fn test_layout_options_eat_all_unused_keys() {
    // Test @EAT_ALL_UNUSED_KEYS = "TRUE"
    let options = LayoutOptions {
        track_caps: 0,
        auto_bksp: 0,
        eat: 1,
        pos_based: 0,
        right_alt: 0,
    };
    
    let km2 = create_km2_with_options(options);
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    assert_eq!(loaded.header.layout_options.eat, 1);
}

#[test]
fn test_layout_options_us_layout_based() {
    // Test @US_LAYOUT_BASED = "TRUE"
    let options = LayoutOptions {
        track_caps: 0,
        auto_bksp: 0,
        eat: 0,
        pos_based: 1,
        right_alt: 0,
    };
    
    let km2 = create_km2_with_options(options);
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    assert_eq!(loaded.header.layout_options.pos_based, 1);
}

#[test]
fn test_layout_options_treat_ctrl_alt_as_ralt() {
    // Test @TREAT_CTRL_ALT_AS_RALT = "TRUE"
    let options = LayoutOptions {
        track_caps: 0,
        auto_bksp: 0,
        eat: 0,
        pos_based: 0,
        right_alt: 1,
    };
    
    let km2 = create_km2_with_options(options);
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    assert_eq!(loaded.header.layout_options.right_alt, 1);
}

#[test]
fn test_multiple_metadata_entries() {
    // Test multiple metadata options together
    let mut km2 = create_basic_km2();
    add_info_text(&mut km2, "name", "Test Keyboard");
    add_info_text(&mut km2, "desc", "A test keyboard");
    add_info_text(&mut km2, "font", "Arial");
    
    km2.header.layout_options = LayoutOptions {
        track_caps: 1,
        auto_bksp: 1,
        eat: 0,
        pos_based: 1,
        right_alt: 0,
    };
    
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    assert_eq!(loaded.info.len(), 3);
    assert_eq!(loaded.header.layout_options.track_caps, 1);
    assert_eq!(loaded.header.layout_options.auto_bksp, 1);
    assert_eq!(loaded.header.layout_options.pos_based, 1);
}

#[test]
fn test_default_layout_options() {
    // Test default values when no options are specified
    let km2 = create_basic_km2();
    let binary = create_km2_binary(&km2).unwrap();
    let loaded = Km2Loader::load(&binary).unwrap();
    
    // Default values from LayoutOptions::default()
    assert_eq!(loaded.header.layout_options.track_caps, 1);  // Default: true
    assert_eq!(loaded.header.layout_options.auto_bksp, 0);   // Default: false
    assert_eq!(loaded.header.layout_options.eat, 0);         // Default: false
    assert_eq!(loaded.header.layout_options.pos_based, 0);   // Default: false
    assert_eq!(loaded.header.layout_options.right_alt, 1);   // Default: true
}