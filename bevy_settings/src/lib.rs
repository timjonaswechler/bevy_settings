//! # Bevy Settings
//!
//! A settings management system for Bevy that:
//! - Manages settings as Bevy resources
//! - Persists only deviations from default values
//! - Supports JSON and binary serialization formats
//! - Provides a derive macro to reduce boilerplate
//! - Offers multiple APIs: fluent `SettingsStore`, unified `SettingsPlugin`, and legacy `TypedSettingsPlugin`
//!
//! ## Quick Start with SettingsStore (Recommended)
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_settings::{Settings, SettingsStore, SerializationFormat};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
//! struct GameSettings {
//!     volume: f32,
//!     resolution: (u32, u32),
//! }
//!
//! #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
//! struct AudioSettings {
//!     master_volume: f32,
//! }
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(
//!         SettingsStore::new("GameSettings")
//!             .format(SerializationFormat::Json)
//!             .version("0.1.0")
//!             .with_base_path("settings")
//!             .register::<GameSettings>()
//!             .register::<AudioSettings>()
//!     )
//!     .run();
//! ```
//!
//! ## Using SettingsStore as a Resource
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_settings::{Settings, SettingsStore, SerializationFormat};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
//! struct GameSettings {
//!     volume: f32,
//! }
//!
//! let settings_store = SettingsStore::new("GameSettings")
//!     .format(SerializationFormat::Json)
//!     .with_base_path("settings")
//!     .register::<GameSettings>();
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .insert_resource(settings_store)
//!     .run();
//! ```
//!
//! ## Alternative API with SettingsPlugin
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_settings::{Settings, SettingsPlugin, SettingsConfig, SerializationFormat};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
//! struct GameSettings {
//!     volume: f32,
//!     resolution: (u32, u32),
//! }
//!
//! #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
//! struct AudioSettings {
//!     master_volume: f32,
//! }
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(
//!         SettingsPlugin::new()
//!             .register::<GameSettings>(SettingsConfig::new("game", SerializationFormat::Json))
//!             .register::<AudioSettings>(SettingsConfig::new("audio", SerializationFormat::Json))
//!     )
//!     .run();
//! ```

pub use bevy_settings_derive::Settings;

mod common;
mod error;
mod format;
mod plugin;
mod plugin_v2;
mod settings_store;
mod storage;
mod trait_def;

pub use error::SettingsError;
pub use format::SerializationFormat;
pub use plugin_v2::{SettingsConfig, SettingsPlugin};
pub use settings_store::SettingsStore;
pub use storage::SettingsStorage;
pub use trait_def::Settings;

// Re-export the old plugin for backward compatibility
pub use plugin::SettingsPlugin as TypedSettingsPlugin;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        SerializationFormat, Settings, SettingsConfig, SettingsError, SettingsPlugin,
        SettingsStorage, SettingsStore, TypedSettingsPlugin,
    };
}
