pub use bevy_settings_derive::Settings;

mod error;
mod format;
mod plugin;
mod storage;
mod trait_def;

pub use error::SettingsError;
pub use format::SerializationFormat;
pub use plugin::SettingsPlugin;
pub use trait_def::Settings;

// Re-export semver for use in migrate implementations
pub use semver;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{SerializationFormat, Settings, SettingsError, SettingsPlugin};
}
