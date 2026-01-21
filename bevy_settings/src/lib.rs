#![warn(missing_docs)]
//! A Bevy plugin for managing persistent settings with delta encoding and path-based storage.
//!
//! `bevy_settings` provides a flexible and efficient way to manage game or application settings
//! as Bevy ECS resources. It supports:
//! - **Delta encoding**: Only saves fields that differ from defaults, reducing file size.
//! - **Path-based storage**: Settings are saved to files based on path templates (e.g., `saves/{id}.json`).
//! - **Auto-saving**: Changes to settings are automatically persisted to disk.
//! - **Multiple formats**: Supports JSON, TOML, RON, and binary formats.
//!
//! # Usage
//! 1. Define a settings group using `#[derive(SettingsGroup)]` and the `#[settings("...")]` attribute.
//! 2. Register the settings group with `SettingsPlugin`.
//! 3. Use `Commands` extensions (`load_settings`/`save_settings`) to manage settings at runtime.
//!
//! # Examples
//! ```
//! use bevy::prelude::*;
//! use bevy_settings::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(SettingsGroup, Serialize, Deserialize, Resource, Clone, PartialEq, Reflect, Default)]
//! #[settings("settings/graphics.json")]
//! struct GraphicsSettings {
//!     resolution: String,
//!     vsync: bool,
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins((
//!             MinimalPlugins,
//!             SettingsPlugin::default()
//!                 .register::<GraphicsSettings>(),
//!         ));
//! }
//! ```
//!
//! # Design Decisions
//! - **Delta Encoding**: Reduces file size and avoids overwriting user-modified files with defaults.
//! - **Path Parameters**: Fields like `{id}` in path templates are excluded from serialization.
//! - **Auto-Save**: Runs in `PostUpdate` to ensure changes are persisted without manual intervention.
//!
//! # Feature Flags
//! - `meta`: Enables metadata support for settings (e.g., UI hints, localization).

mod commands;
mod delta;
mod error;
mod group;
mod manager;
mod plugin;

pub use bevy_settings_derive::SettingsGroup;
pub use commands::SettingsCommandsExt;
pub use error::SettingsError;
pub use group::SettingsGroupTrait;
pub use manager::ManagerError;
pub use plugin::SettingsPlugin;

#[cfg(feature = "meta")]
pub use bevy_settings_meta::{
    LocalizedText, SettingDescriptor, SettingKind, SettingsError, UiHint,
};
