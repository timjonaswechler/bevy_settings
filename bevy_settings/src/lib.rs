
mod error;
mod format;
mod plugin;
mod storage;
mod trait_def;

pub use error::SettingsError;
pub use format::SerializationFormat;
pub use plugin::SettingsPlugin;
pub use trait_def::Settings;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{SerializationFormat, Settings, SettingsError, SettingsPlugin};
}

#[cfg(feature = "meta")]
pub use bevy_settings_meta::{
    LocalizedText, SettingDescriptor, SettingKind, SettingsError, UiHint,
};
