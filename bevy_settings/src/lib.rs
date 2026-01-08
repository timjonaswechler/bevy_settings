mod commands;
mod delta;
mod error;
mod group;
mod manager;
mod plugin;

pub use commands::SettingsCommandsExt;
pub use delta::{compute_delta, merge_with_defaults};
pub use error::SettingsError;
pub use group::SettingsGroup;
pub use plugin::SettingsPlugin;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{SettingsCommandsExt, SettingsError, SettingsGroup, SettingsPlugin};
    pub use bevy_paths::prelude::*;
    pub use bevy_settings_derive::SettingsGroup; // Re-export bevy_paths so user has everything
}

#[cfg(feature = "meta")]
pub use bevy_settings_meta::{
    LocalizedText, SettingDescriptor, SettingKind, SettingsError, UiHint,
};
