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
pub trait Settings:
    Resource + Serialize + for<'de> Deserialize<'de> + Default + Clone + PartialEq
{
    /// Get the type name of the settings struct
    fn type_name() -> &'static str;

    /// The section name for this settings type in the settings file
    /// 
    /// By default, this is the lowercase version of the type name.
    /// Can be overridden by implementing this constant manually.
    const SECTION: &'static str;

    /// Migrate settings data from one version to another
    ///
    /// This method is called when loading settings if the file version differs
    /// from the target version. It allows you to transform the settings data
    /// to handle breaking changes between versions.
    ///
    /// # Arguments
    /// * `file_version` - The version stored in the settings file (None if not present)
    /// * `target_version` - The current version of the application
    /// * `data` - The raw JSON value read from the file
    ///
    /// # Returns
    /// Returns a tuple of (migrated_data, changed) where:
    /// - `migrated_data` is the transformed settings data
    /// - `changed` is true if migration modified the data
    ///
    /// # Default Implementation
    /// The default implementation performs no migration and returns the data unchanged.
    fn migrate(
        _file_version: Option<&semver::Version>,
        _target_version: &semver::Version,
        data: serde_json::Value,
    ) -> Result<(serde_json::Value, bool), crate::SettingsError> {
        Ok((data, false))
    }
}
