//! Error types for the KeyMagic engine

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid variable index: {0}")]
    InvalidVariableIndex(usize),
    
    #[error("Invalid reference index: {0}")]
    InvalidReferenceIndex(usize),
    
    #[error("Recursion depth exceeded")]
    RecursionDepthExceeded,
    
    #[error("Invalid state index: {0}")]
    InvalidStateIndex(usize),
    
    #[error("KM2 format error: {0}")]
    Km2Error(#[from] crate::km2::error::Km2Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;