#[cfg(test)]
mod tests {
    use keymagic_core::{KeyMagicEngine, KeyInput};
    use keymagic_core::engine::{ModifierState, Predefined};
    use keymagic_core::types::km2::{Km2File, FileHeader, LayoutOptions};

    fn create_empty_km2() -> Km2File {
        Km2File {
            header: FileHeader {
                magic_code: [0x4B, 0x4D, 0x4B, 0x4C], // "KMKL"
                major_version: 1,
                minor_version: 5,
                string_count: 0,
                info_count: 0,
                rule_count: 0,
                layout_options: LayoutOptions {
                    track_caps: 1,  // true
                    auto_bksp: 0,   // false
                    eat: 0,         // false
                    pos_based: 0,   // false
                    right_alt: 1,   // true
                },
            },
            strings: vec![],
            info: vec![],
            rules: vec![],
        }
    }

    #[test]
    fn test_engine_creation() {
        let km2 = create_empty_km2();
        let engine = KeyMagicEngine::new(km2).unwrap();
        assert_eq!(engine.composing_text(), "");
    }

    #[test]
    fn test_key_processing_without_rules() {
        let km2 = create_empty_km2();
        let mut engine = KeyMagicEngine::new(km2).unwrap();
        
        let input = KeyInput {
            key_code: Predefined::from_raw(65), // 'A'
            modifiers: ModifierState {
                shift: false,
                ctrl: false,
                alt: false,
                caps_lock: false,
            },
            character: Some('A'),
        };
        
        let output = engine.process_key(input).unwrap();
        // Without rules, the character should pass through
        assert_eq!(output.composing_text, "A");
    }
}