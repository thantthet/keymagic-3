use thiserror::Error;

#[derive(Error, Debug)]
pub enum Km2Error {
    #[error("Invalid magic code: expected 'KMKL', got {0:?}")]
    InvalidMagicCode([u8; 4]),
    
    #[error("Unsupported version: {major}.{minor}")]
    UnsupportedVersion { major: u8, minor: u8 },
    
    #[error("File too small: {0} bytes")]
    FileTooSmall(usize),
    
    #[error("Invalid UTF-16 string at offset {0}")]
    InvalidUtf16(usize),
    
    #[error("Invalid string index: {0} (max: {1})")]
    InvalidStringIndex(usize, usize),
    
    #[error("Invalid opcode: {0:#06X}")]
    InvalidOpcode(u16),
    
    #[error("Truncated file: expected {expected} bytes, got {actual}")]
    TruncatedFile { expected: usize, actual: usize },
    
    #[error("Invalid rule structure at index {0}")]
    InvalidRule(usize),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Km2Error>;