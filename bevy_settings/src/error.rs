use thiserror::Error;

#[derive(Error, Debug)]
pub enum SettingsError {
    /// Error during JSON serialization or deserialization.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error during binary serialization.
    #[error("Binary serialization error: {0}")]
    BincodeEncode(#[from] bincode::error::EncodeError),

    /// Error during binary deserialization.
    #[error("Binary deserialization error: {0}")]
    BincodeDecode(#[from] bincode::error::DecodeError),

    /// Error during file I/O operations (e.g., reading or writing settings files).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The settings file was not found.
    ///
    /// This is not treated as an error; instead, the settings are initialized with defaults.
    #[error("Settings file not found, using defaults")]
    FileNotFound,

    /// Failed to compare settings with their defaults.
    ///
    /// This may occur if serialization fails during comparison.
    #[error("Failed to compare settings with defaults")]
    ComparisonFailed,
}

pub(crate) type Result<T> = std::result::Result<T, SettingsError>;
