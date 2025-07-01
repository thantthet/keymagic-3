use keymagic_core::{Km2File, FileHeader, LayoutOptions, InfoEntry, StringEntry, Rule, RuleElement};

/// Creates a KM2 binary file from a Km2File struct
pub fn create_km2_binary(km2: &Km2File) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Use kms2km2's Km2Writer for writing binary data
    let mut buffer = Vec::new();
    let writer = kms2km2::binary::Km2Writer::new(&mut buffer);
    writer.write_km2_file(km2).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(buffer)
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
    // Convert text to UTF-8 bytes (matching kms2km2 compiler behavior)
    let data = text.as_bytes().to_vec();
    
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

/// Decode UTF-8 text from bytes
pub fn decode_utf8_text(data: &[u8]) -> String {
    String::from_utf8_lossy(data).to_string()
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