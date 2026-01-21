use bevy::prelude::Resource;
use bevy_paths::TypedPath;
use serde::{Serialize, de::DeserializeOwned};

/// A grouping of settings that corresponds to a single file.
///
/// This trait is used to define settings groups that can be loaded, saved, and managed
/// as Bevy resources. It is typically derived using `#[derive(SettingsGroup)]` and requires
/// a `#[settings("...")]` attribute to specify the file path template.
///
/// # Type Parameters
/// Settings groups must implement the following traits:
/// - `Resource`: To be used as a Bevy ECS resource.
/// - `Serialize` and `DeserializeOwned`: For serialization and deserialization.
/// - `Default`: To provide default values.
/// - `Clone`: For copying settings.
/// - `PartialEq`: For comparing settings.
/// - `TypedPath`: For resolving file paths.
///
/// # Examples
/// ```
/// use bevy::reflect::Reflect;
/// use bevy::prelude::*;
/// use bevy_paths::prelude::*;
/// use bevy_settings::*;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(SettingsGroup, Resource, Serialize, Deserialize, Clone, PartialEq, Default, Reflect, Debug)]
/// #[settings("settings/config.toml")]
/// struct SettingsFile;
///
/// #[derive(SettingsGroup, Resource, Serialize, Deserialize, Clone, PartialEq, Default, Reflect, Debug)]
/// #[settings("levels/{name}/map.dat")]
/// struct Level {
///     name: String,
/// }
///
/// fn load_settings() {
///     let settings = SettingsFile;
///     let settings_path = settings.resolve().expect("Failed to resolve path");
///     println!("Settings path: {:?}", settings_path);
/// }
///
/// fn load_level() {
///     let dungeon = Level {
///         name: "dungeon_1".to_string(),
///     };
///     let dungeon_path = dungeon.resolve().expect("Failed to resolve path");
///     println!("Dungeon path: {:?}", dungeon_path);
/// }
/// ```
///
/// Returns a list of field names that are used as path parameters.
///
/// These fields are automatically detected by the `#[derive(SettingsGroup)]` macro
/// from the `#[settings("...")]` attribute (e.g., `"saves/{id}.json"` -> `["id"]`).
/// They are removed from the serialized payload before saving, as they are part of the file path.
///
/// # Returns
/// A static slice of field names used as path parameters.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use bevy_settings::SettingsGroup;
/// use crate::bevy_settings::SettingsGroupTrait;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(SettingsGroup, Serialize, Reflect, Deserialize, Resource, Clone, PartialEq, Default)]
/// #[serde(default)]
/// #[settings("saves/{id}.json")]
/// struct SaveSettings {
///     id: String,
///     volume: f32,
/// }
///
/// assert_eq!(SaveSettings::path_params(), ["id"]);
/// ```
pub trait SettingsGroupTrait:
    Resource + Serialize + DeserializeOwned + Default + Clone + PartialEq + TypedPath
{
    /// Returns a list of field names that are used as path parameters.
    ///
    /// These fields are automatically detected by the `#[derive(SettingsGroup)]` macro
    /// from the `#[settings("...")]` attribute (e.g., `"saves/{id}.json"` -> `["id"]`).
    /// They are removed from the serialized payload before saving, as they are part of the file path.
    ///
    /// # Returns
    /// A static slice of field names used as path parameters.
    fn path_params() -> &'static [&'static str];
}
