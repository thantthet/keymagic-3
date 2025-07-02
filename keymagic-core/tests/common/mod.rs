use keymagic_core::{Km2File, FileHeader, LayoutOptions, InfoEntry, StringEntry, Rule, BinaryFormatElement};

#[cfg(test)]
pub mod engine_helpers;

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

/// Adds a string to the strings table and returns its 1-based index
#[allow(dead_code)]
pub fn add_string(km2: &mut Km2File, value: &str) -> usize {
    km2.strings.push(StringEntry {
        value: value.to_string(),
    });
    km2.header.string_count = km2.strings.len() as u16;
    km2.strings.len() // Return 1-based index
}

/// Adds a rule to the Km2File
#[allow(dead_code)]
pub fn add_rule(km2: &mut Km2File, lhs: Vec<BinaryFormatElement>, rhs: Vec<BinaryFormatElement>) {
    km2.rules.push(Rule { lhs, rhs });
    km2.header.rule_count = km2.rules.len() as u16;
}

/// Decode UTF-8 text from bytes
pub fn decode_utf8_text(data: &[u8]) -> String {
    String::from_utf8_lossy(data).to_string()
}

// Re-export engine helpers
#[cfg(test)]
pub use engine_helpers::*;