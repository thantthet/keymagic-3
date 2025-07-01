use keymagic_core::{Km2File, FileHeader, LayoutOptions, InfoEntry, StringEntry, Rule, RuleElement};
use std::io::Write;
use byteorder::{LittleEndian, WriteBytesExt, ByteOrder};

/// Creates a KM2 binary file from a Km2File struct
pub fn create_km2_binary(km2: &Km2File) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    
    // Write header
    buffer.write_all(&km2.header.magic_code)?;
    buffer.write_u8(km2.header.major_version)?;
    buffer.write_u8(km2.header.minor_version)?;
    buffer.write_u16::<LittleEndian>(km2.header.string_count)?;
    buffer.write_u16::<LittleEndian>(km2.header.info_count)?;
    buffer.write_u16::<LittleEndian>(km2.header.rule_count)?;
    buffer.write_u8(km2.header.layout_options.track_caps)?;
    buffer.write_u8(km2.header.layout_options.auto_bksp)?;
    buffer.write_u8(km2.header.layout_options.eat)?;
    buffer.write_u8(km2.header.layout_options.pos_based)?;
    buffer.write_u8(km2.header.layout_options.right_alt)?;
    
    // Write strings
    for string in &km2.strings {
        let utf16: Vec<u16> = string.value.encode_utf16().collect();
        buffer.write_u16::<LittleEndian>(utf16.len() as u16)?;
        for ch in utf16 {
            buffer.write_u16::<LittleEndian>(ch)?;
        }
    }
    
    // Write info entries
    for info in &km2.info {
        buffer.write_all(&info.id)?;
        buffer.write_u16::<LittleEndian>(info.data.len() as u16)?;
        buffer.write_all(&info.data)?;
    }
    
    // Write rules
    for rule in &km2.rules {
        // Write LHS length and elements
        let lhs_size = calculate_rule_elements_size(&rule.lhs);
        buffer.write_u16::<LittleEndian>(lhs_size as u16)?;
        write_rule_elements(&mut buffer, &rule.lhs)?;
        
        // Write RHS length and elements
        let rhs_size = calculate_rule_elements_size(&rule.rhs);
        buffer.write_u16::<LittleEndian>(rhs_size as u16)?;
        write_rule_elements(&mut buffer, &rule.rhs)?;
    }
    
    Ok(buffer)
}

fn calculate_rule_elements_size(elements: &[RuleElement]) -> usize {
    let mut size = 0;
    for element in elements {
        size += 2; // opcode
        match element {
            RuleElement::String(s) => {
                size += 2; // string length
                size += s.encode_utf16().count() * 2; // UTF-16 chars
            }
            RuleElement::Variable(_) | RuleElement::Reference(_) | 
            RuleElement::Predefined(_) | RuleElement::Modifier(_) |
            RuleElement::AnyOf(_) | RuleElement::NotAnyOf(_) | 
            RuleElement::Switch(_) => {
                size += 2; // index/value
            }
            RuleElement::And | RuleElement::Any => {
                // No additional data
            }
        }
    }
    size
}

fn write_rule_elements(buffer: &mut Vec<u8>, elements: &[RuleElement]) -> Result<(), Box<dyn std::error::Error>> {
    use keymagic_core::*;
    
    for element in elements {
        match element {
            RuleElement::String(s) => {
                buffer.write_u16::<LittleEndian>(OP_STRING)?;
                let utf16: Vec<u16> = s.encode_utf16().collect();
                buffer.write_u16::<LittleEndian>(utf16.len() as u16)?;
                for ch in utf16 {
                    buffer.write_u16::<LittleEndian>(ch)?;
                }
            }
            RuleElement::Variable(idx) => {
                buffer.write_u16::<LittleEndian>(OP_VARIABLE)?;
                buffer.write_u16::<LittleEndian>(*idx as u16)?;
            }
            RuleElement::Reference(idx) => {
                buffer.write_u16::<LittleEndian>(OP_REFERENCE)?;
                buffer.write_u16::<LittleEndian>(*idx as u16)?;
            }
            RuleElement::Predefined(vk) => {
                buffer.write_u16::<LittleEndian>(OP_PREDEFINED)?;
                buffer.write_u16::<LittleEndian>(*vk)?;
            }
            RuleElement::Modifier(flags) => {
                buffer.write_u16::<LittleEndian>(OP_MODIFIER)?;
                buffer.write_u16::<LittleEndian>(*flags)?;
            }
            RuleElement::AnyOf(idx) => {
                buffer.write_u16::<LittleEndian>(OP_ANYOF)?;
                buffer.write_u16::<LittleEndian>(*idx as u16)?;
            }
            RuleElement::NotAnyOf(idx) => {
                buffer.write_u16::<LittleEndian>(OP_NANYOF)?;
                buffer.write_u16::<LittleEndian>(*idx as u16)?;
            }
            RuleElement::Any => {
                buffer.write_u16::<LittleEndian>(OP_ANY)?;
            }
            RuleElement::And => {
                buffer.write_u16::<LittleEndian>(OP_AND)?;
            }
            RuleElement::Switch(idx) => {
                buffer.write_u16::<LittleEndian>(OP_SWITCH)?;
                buffer.write_u16::<LittleEndian>(*idx as u16)?;
            }
        }
    }
    Ok(())
}

/// Creates a basic Km2File with default header
pub fn create_basic_km2() -> Km2File {
    Km2File {
        header: FileHeader::new(),
        strings: vec![],
        info: vec![],
        rules: vec![],
    }
}

/// Creates a Km2File with specified layout options
pub fn create_km2_with_options(options: LayoutOptions) -> Km2File {
    let mut km2 = create_basic_km2();
    km2.header.layout_options = options;
    km2
}

/// Adds an info entry to a Km2File with text data
pub fn add_info_text(km2: &mut Km2File, id: &str, text: &str) {
    // Convert text to UTF-16LE bytes
    let utf16: Vec<u16> = text.encode_utf16().collect();
    let mut data = Vec::new();
    for ch in utf16 {
        data.write_u16::<LittleEndian>(ch).unwrap();
    }
    
    km2.info.push(InfoEntry {
        id: id.as_bytes().try_into().unwrap_or([0; 4]),
        data,
    });
    km2.header.info_count = km2.info.len() as u16;
}

/// Adds a string to the strings table and returns its index
#[allow(dead_code)]
pub fn add_string(km2: &mut Km2File, value: &str) -> usize {
    let index = km2.strings.len();
    km2.strings.push(StringEntry {
        value: value.to_string(),
    });
    km2.header.string_count = km2.strings.len() as u16;
    index
}

/// Adds a rule to the Km2File
#[allow(dead_code)]
pub fn add_rule(km2: &mut Km2File, lhs: Vec<RuleElement>, rhs: Vec<RuleElement>) {
    km2.rules.push(Rule { lhs, rhs });
    km2.header.rule_count = km2.rules.len() as u16;
}

/// Decode UTF-16LE text from bytes
pub fn decode_utf16le_text(data: &[u8]) -> String {
    let mut u16_vec = Vec::new();
    for chunk in data.chunks_exact(2) {
        u16_vec.push(LittleEndian::read_u16(chunk));
    }
    String::from_utf16_lossy(&u16_vec)
}

/// Helper to create a KeyInput from a character
pub fn key_input_from_char(ch: char) -> keymagic_core::KeyInput {
    use keymagic_core::{KeyInput, VirtualKey, engine::ModifierState};
    
    // Map common chars to virtual keys
    let vk = match ch {
        'a' => VirtualKey::KeyA,
        'b' => VirtualKey::KeyB,
        'c' => VirtualKey::KeyC,
        'd' => VirtualKey::KeyD,
        'e' => VirtualKey::KeyE,
        'f' => VirtualKey::KeyF,
        'g' => VirtualKey::KeyG,
        'h' => VirtualKey::KeyH,
        'i' => VirtualKey::KeyI,
        'j' => VirtualKey::KeyJ,
        'k' => VirtualKey::KeyK,
        'l' => VirtualKey::KeyL,
        'm' => VirtualKey::KeyM,
        'n' => VirtualKey::KeyN,
        'o' => VirtualKey::KeyO,
        'p' => VirtualKey::KeyP,
        'q' => VirtualKey::KeyQ,
        'r' => VirtualKey::KeyR,
        's' => VirtualKey::KeyS,
        't' => VirtualKey::KeyT,
        'u' => VirtualKey::KeyU,
        'v' => VirtualKey::KeyV,
        'w' => VirtualKey::KeyW,
        'x' => VirtualKey::KeyX,
        'y' => VirtualKey::KeyY,
        'z' => VirtualKey::KeyZ,
        'A'..='Z' => {
            // For uppercase, use same key but we'll handle shift separately
            match ch.to_ascii_lowercase() {
                'a' => VirtualKey::KeyA,
                'b' => VirtualKey::KeyB,
                'c' => VirtualKey::KeyC,
                'd' => VirtualKey::KeyD,
                'e' => VirtualKey::KeyE,
                'f' => VirtualKey::KeyF,
                'g' => VirtualKey::KeyG,
                'h' => VirtualKey::KeyH,
                'i' => VirtualKey::KeyI,
                'j' => VirtualKey::KeyJ,
                'k' => VirtualKey::KeyK,
                'l' => VirtualKey::KeyL,
                'm' => VirtualKey::KeyM,
                'n' => VirtualKey::KeyN,
                'o' => VirtualKey::KeyO,
                'p' => VirtualKey::KeyP,
                'q' => VirtualKey::KeyQ,
                'r' => VirtualKey::KeyR,
                's' => VirtualKey::KeyS,
                't' => VirtualKey::KeyT,
                'u' => VirtualKey::KeyU,
                'v' => VirtualKey::KeyV,
                'w' => VirtualKey::KeyW,
                'x' => VirtualKey::KeyX,
                'y' => VirtualKey::KeyY,
                'z' => VirtualKey::KeyZ,
                _ => VirtualKey::Space,
            }
        }
        '0' => VirtualKey::Key0,
        '1' => VirtualKey::Key1,
        '2' => VirtualKey::Key2,
        '3' => VirtualKey::Key3,
        '4' => VirtualKey::Key4,
        '5' => VirtualKey::Key5,
        '6' => VirtualKey::Key6,
        '7' => VirtualKey::Key7,
        '8' => VirtualKey::Key8,
        '9' => VirtualKey::Key9,
        ' ' => VirtualKey::Space,
        _ => VirtualKey::Space, // Default for non-ASCII chars
    };
    
    KeyInput::new(vk, ModifierState::new()).with_char(ch)
}