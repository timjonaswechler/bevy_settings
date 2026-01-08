use bevy::prelude::Resource;
use bevy_paths::TypedPath;
use serde::{de::DeserializeOwned, Serialize};

/// A grouping of settings that corresponds to a single file.
///
/// This trait is typically derived via `#[derive(SettingsGroup)]`.
pub trait SettingsGroup:
    Resource + Serialize + DeserializeOwned + Default + Clone + PartialEq + TypedPath
{
    /// Returns a list of field names that are used as path parameters.
    ///
    /// These fields are automatically detected by the macro (e.g. "saves/{id}.json" -> ["id"]).
    /// They should be removed from the data payload before saving, as they are part of the key.
    fn path_params() -> &'static [&'static str];
}
