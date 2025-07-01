use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("KM2 loading error: {0}")]
    Km2(#[from] crate::km2::Km2Error),
    
    #[error("Engine error: {0}")]
    Engine(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("State error: {0}")]
    State(String),
}

pub type Result<T> = std::result::Result<T, Error>;