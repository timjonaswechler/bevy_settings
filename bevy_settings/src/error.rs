use thiserror::Error;

/// Errors that can occur when working with settings
#[derive(Error, Debug)]
pub enum SettingsError {
    /// Error during JSON serialization/deserialization
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error during binary serialization/deserialization
    #[error("Binary serialization error: {0}")]
    Bincode(#[from] bincode::Error),

    /// Error during file I/O operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Settings file not found (not an error, just means using defaults)
    #[error("Settings file not found, using defaults")]
    FileNotFound,

    /// Error comparing settings with defaults
    #[error("Failed to compare settings with defaults")]
    ComparisonFailed,
}

pub type Result<T> = std::result::Result<T, SettingsError>;
