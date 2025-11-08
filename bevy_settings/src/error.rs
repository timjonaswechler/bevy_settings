use thiserror::Error;

/// Errors that can occur when working with settings
#[derive(Error, Debug)]
pub enum SettingsError {
    /// Error during JSON serialization/deserialization
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error during binary serialization
    #[error("Binary serialization error: {0}")]
    BincodeEncode(#[from] bincode::error::EncodeError),

    /// Error during binary deserialization
    #[error("Binary deserialization error: {0}")]
    BincodeDecode(#[from] bincode::error::DecodeError),

    /// Error during file I/O operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SettingsError>;
