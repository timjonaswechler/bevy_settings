use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Trait for settings that can be managed by the settings system
/// 
/// This trait is typically derived using the `#[derive(Settings)]` macro.
/// 
/// # Requirements
/// Types implementing this trait must also implement:
/// - `Resource` - to be used as a Bevy resource
/// - `Serialize` + `Deserialize` - for persistence
/// - `Default` - to provide default values
/// - `Clone` - for creating copies
/// - `PartialEq` - for detecting changes from defaults
pub trait Settings: Resource + Serialize + for<'de> Deserialize<'de> + Default + Clone + PartialEq {
    /// Get the type name of the settings struct
    fn type_name() -> &'static str;
}
