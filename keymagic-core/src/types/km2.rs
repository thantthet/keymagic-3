
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

#[derive(Debug, Clone)]
pub struct Rule {
    pub lhs: Vec<RuleElement>,
    pub rhs: Vec<RuleElement>,
}

#[derive(Debug, Clone)]
pub enum RuleElement {
    String(String),
    Variable(usize),            // 1-based index
    Reference(usize),           // Back-reference ($1, $2, etc.)
    Predefined(u16),           // Virtual key code
    Modifier(u16),             // Modifier flags (actually an opcode like opANYOF)
    AnyOf(usize),              // Variable index for [*]
    And,                       // Logical AND
    NotAnyOf(usize),           // Variable index for [^]
    Any,                       // ANY keyword
    Switch(usize),             // State switch (index into strings)
}

#[derive(Debug)]
pub struct Km2File {
    pub header: FileHeader,
    pub strings: Vec<StringEntry>,
    pub info: Vec<InfoEntry>,
    pub rules: Vec<Rule>,
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