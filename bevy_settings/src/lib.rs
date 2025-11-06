//! # Bevy Settings
//!
//! A settings management system for Bevy that:
//! - Manages settings as Bevy resources
//! - Persists only deviations from default values
//! - Supports JSON and binary serialization formats
//! - Provides a derive macro to reduce boilerplate
//!
//! ## Example
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_settings::{Settings, SettingsPlugin, SerializationFormat};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
//! struct GameSettings {
//!     volume: f32,
//!     resolution: (u32, u32),
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(SettingsPlugin::<GameSettings>::new(
//!             "game_settings",
//!             SerializationFormat::Json,
//!         ))
//!         .run();
//! }
//! ```

pub use bevy_settings_derive::Settings;

mod error;
mod format;
mod plugin;
mod storage;
mod trait_def;

pub use error::SettingsError;
pub use format::SerializationFormat;
pub use plugin::SettingsPlugin;
pub use storage::SettingsStorage;
pub use trait_def::Settings;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{Settings, SettingsError, SettingsPlugin, SettingsStorage, SerializationFormat};
}

