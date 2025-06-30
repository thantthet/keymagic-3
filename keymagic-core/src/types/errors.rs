use thiserror::Error;

#[derive(Error, Debug)]
pub enum KmsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },
    
    #[error("Invalid Unicode escape: {0}")]
    InvalidUnicode(String),
    
    #[error("Undefined variable: ${0}")]
    UndefinedVariable(String),
    
    #[error("Invalid virtual key: {0}")]
    InvalidVirtualKey(String),
    
    #[error("Invalid rule: {0}")]
    InvalidRule(String),
    
    #[error("Circular variable reference: ${0}")]
    CircularReference(String),
    
    #[error("Include file not found: {0}")]
    IncludeNotFound(String),
    
    #[error("Binary write error: {0}")]
    BinaryWrite(String),
}