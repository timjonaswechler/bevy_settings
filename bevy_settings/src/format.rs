/// Serialization format for settings storage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    /// JSON format - human readable, easy to edit
    Json,
    /// Binary format using bincode - compact and efficient
    Binary,
}
