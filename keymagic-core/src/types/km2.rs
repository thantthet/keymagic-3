
use std::collections::HashMap;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct FileHeader {
    pub magic_code: [u8; 4],    // "KMKL"
    pub major_version: u8,      // 1
    pub minor_version: u8,      // 5
    pub string_count: u16,
    pub info_count: u16,
    pub rule_count: u16,
    pub layout_options: LayoutOptions,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct LayoutOptions {
    pub track_caps: u8,         // 0 or 1
    pub auto_bksp: u8,          // 0 or 1
    pub eat: u8,                // 0 or 1
    pub pos_based: u8,          // 0 or 1
    pub right_alt: u8,          // 0 or 1 (v1.5+)
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            track_caps: 1,  // true
            auto_bksp: 0,   // false
            eat: 0,         // false
            pos_based: 0,   // false
            right_alt: 1,   // true
        }
    }
}

#[derive(Debug, Clone)]
pub struct StringEntry {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct InfoEntry {
    pub id: [u8; 4],
    pub data: Vec<u8>,
}

/// A wrapper around keyboard metadata info entries that provides convenient access methods
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    entries: HashMap<[u8; 4], Vec<u8>>,
}

impl Metadata {
    /// Create a new Metadata instance from a vector of InfoEntry
    pub fn new(entries: Vec<InfoEntry>) -> Self {
        let mut map = HashMap::new();
        for entry in entries {
            map.insert(entry.id, entry.data);
        }
        Self { entries: map }
    }
    
    /// Get an info entry's data by its ID
    pub fn get(&self, id: &[u8; 4]) -> Option<&Vec<u8>> {
        self.entries.get(id)
    }
    
    /// Get an info entry's data as UTF-8 string
    pub fn get_string(&self, id: &[u8; 4]) -> Option<String> {
        self.get(id)
            .map(|data| String::from_utf8_lossy(data).into_owned())
    }
    
    /// Get the keyboard name
    pub fn name(&self) -> Option<String> {
        self.get_string(INFO_NAME)
    }
    
    /// Get the keyboard description
    pub fn description(&self) -> Option<String> {
        self.get_string(INFO_DESC)
    }
    
    /// Get the font family
    pub fn font_family(&self) -> Option<String> {
        self.get_string(INFO_FONT)
    }
    
    /// Get the hotkey string
    pub fn hotkey(&self) -> Option<String> {
        self.get_string(INFO_HTKY)
    }
    
    /// Get the icon data
    pub fn icon(&self) -> Option<&[u8]> {
        self.get(INFO_ICON)
            .map(|data| data.as_slice())
    }
    
    /// Check if a specific info entry exists
    pub fn has(&self, id: &[u8; 4]) -> bool {
        self.entries.contains_key(id)
    }
    
    /// Get the number of info entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if metadata is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Iterate over all entries
    pub fn iter(&self) -> impl Iterator<Item = (&[u8; 4], &Vec<u8>)> {
        self.entries.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub lhs: Vec<BinaryFormatElement>,
    pub rhs: Vec<BinaryFormatElement>,
}

/// Binary format element from KM2 file
/// These directly represent the opcodes and data from the compiled KM2 format
#[derive(Debug, Clone)]
pub enum BinaryFormatElement {
    String(String),
    Variable(usize),            // 1-based index into strings table
    Reference(usize),           // Back-reference ($1, $2, etc.)
    Predefined(u16),           // Virtual key code
    Modifier(u16),             // Modifier flags (FLAG_ANYOF, FLAG_NANYOF, or numeric index)
    And,                       // Logical AND for combining keys
    Any,                       // ANY keyword - matches any character
    Switch(usize),             // State switch (0-based integer ID)
}

#[derive(Debug)]
pub struct Km2File {
    pub header: FileHeader,
    pub strings: Vec<StringEntry>,
    pub info: Vec<InfoEntry>,
    pub rules: Vec<Rule>,
}

impl Km2File {
    /// Get a Metadata wrapper for convenient access to info entries
    pub fn metadata(&self) -> Metadata {
        Metadata::new(self.info.clone())
    }
}

impl FileHeader {
    pub fn new() -> Self {
        FileHeader {
            magic_code: *b"KMKL",
            major_version: 1,
            minor_version: 5,
            string_count: 0,
            info_count: 0,
            rule_count: 0,
            layout_options: LayoutOptions::default(),
        }
    }
}

// Standard info IDs (stored as little-endian multi-char constants)
pub const INFO_NAME: &[u8; 4] = b"eman"; // 'name' in little-endian
pub const INFO_DESC: &[u8; 4] = b"csed"; // 'desc' in little-endian
pub const INFO_FONT: &[u8; 4] = b"tnof"; // 'font' in little-endian
pub const INFO_ICON: &[u8; 4] = b"noci"; // 'icon' in little-endian
pub const INFO_HTKY: &[u8; 4] = b"ykth"; // 'htky' in little-endian