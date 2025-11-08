use crate::{error::Result, SerializationFormat, Settings};
use bevy::prelude::*;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Buffer size for binary serialization (1 MB)
const BINARY_BUFFER_SIZE: usize = 1024 * 1024;

/// Storage that saves multiple settings types to a single file
#[derive(Clone)]
pub(crate) struct Storage {
    pub(crate) format: SerializationFormat,
    pub(crate) base_path: PathBuf,
    pub(crate) filename: String,
    pub(crate) version: Option<String>,
}

impl Storage {
    /// Create a new storage with the specified format
    pub(crate) fn new(filename: impl Into<String>, format: SerializationFormat) -> Self {
        Self {
            format,
            base_path: PathBuf::from("settings"),
            filename: filename.into(),
            version: None,
        }
    }

    /// Set the base path for settings files
    pub(crate) fn with_base_path(mut self, path: impl AsRef<Path>) -> Self {
        self.base_path = path.as_ref().to_path_buf();
        self
    }

    /// Set the version string
    pub(crate) fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Get the full path for the settings file
    fn get_path(&self) -> PathBuf {
        self.base_path
            .join(format!("{}.{}", self.filename, self.format.extension()))
    }

    /// Load all settings from the file, returning both settings and version info
    pub(crate) fn load_all_with_versions(&self) -> Result<(Map<String, Value>, Map<String, Value>)> {
        let path = self.get_path();

        // If file doesn't exist, return empty maps
        if !path.exists() {
            return Ok((Map::new(), Map::new()));
        }

        let content = fs::read(&path)?;

        // Deserialize based on format
        let root: Value = match self.format {
            SerializationFormat::Json => serde_json::from_slice(&content)?,
            SerializationFormat::Binary => {
                let config = bincode::config::standard();
                bincode::serde::decode_from_slice(&content, config)
                    .map_err(crate::error::SettingsError::BincodeDecode)?
                    .0
            }
        };

        // Extract the settings map and versions
        if let Value::Object(mut map) = root {
            // Extract version info (per-section versions)
            let versions = if let Some(Value::Object(versions_obj)) = map.remove("_versions") {
                versions_obj
            } else {
                Map::new()
            };

            Ok((map, versions))
        } else {
            Ok((Map::new(), Map::new()))
        }
    }

    /// Load all settings from the file
    pub(crate) fn load_all(&self) -> Result<Map<String, Value>> {
        let (settings, _versions) = self.load_all_with_versions()?;
        Ok(settings)
    }

    /// Load a specific settings type from the file
    ///
    /// This method is provided for manual control over loading. When using the plugin system,
    /// settings are loaded automatically.
    ///
    /// # Arguments
    /// * `type_key` - The lowercase type name (e.g., "audiosettings" for AudioSettings)
    ///
    /// # Returns
    /// Returns the merged settings (defaults + saved delta) or defaults if not found
    #[allow(dead_code)]
    pub(crate) fn load<T: Settings>(&self, type_key: &str) -> Result<T> {
        let all_settings = self.load_all()?;

        // Try to find settings for this type
        if let Some(value) = all_settings.get(type_key) {
            let settings: T = serde_json::from_value(value.clone())?;
            Ok(settings)
        } else {
            // Not found, return defaults
            Ok(T::default())
        }
    }

    /// Save multiple settings types to the file with version information
    pub(crate) fn save_all_with_versions(
        &self,
        settings_map: &HashMap<String, Value>,
        versions: &HashMap<String, String>,
    ) -> Result<()> {
        let path = self.get_path();

        // If all settings are empty (equal to defaults), delete the file
        if settings_map.is_empty() {
            if path.exists() {
                fs::remove_file(&path)?;
            }
            return Ok(());
        }

        // Build the root object with version info and all settings
        let mut root = Map::new();

        // Add version information per section
        if !versions.is_empty() {
            let mut versions_obj = Map::new();
            for (section, version) in versions {
                versions_obj.insert(section.clone(), Value::String(version.clone()));
            }
            root.insert("_versions".to_string(), Value::Object(versions_obj));
        }

        // Add all settings
        for (key, value) in settings_map {
            root.insert(key.clone(), value.clone());
        }

        let root_value = Value::Object(root);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize based on format
        let content = match self.format {
            SerializationFormat::Json => serde_json::to_vec_pretty(&root_value)?,
            SerializationFormat::Binary => {
                let config = bincode::config::standard();
                let mut buffer = vec![0u8; BINARY_BUFFER_SIZE];
                let size = bincode::serde::encode_into_slice(&root_value, &mut buffer, config)
                    .map_err(crate::error::SettingsError::BincodeEncode)?;
                buffer.truncate(size);
                buffer
            }
        };

        fs::write(&path, content)?;
        Ok(())
    }

    /// Save multiple settings types to the file
    pub(crate) fn save_all(&self, settings_map: &HashMap<String, Value>) -> Result<()> {
        self.save_all_with_versions(settings_map, &HashMap::new())
    }

    /// Delete the settings file
    ///
    /// This method is provided for manual control. When using the plugin system,
    /// files are automatically deleted when all settings return to their defaults.
    #[allow(dead_code)]
    pub(crate) fn delete(&self) -> Result<()> {
        let path = self.get_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

/// Compute delta between current settings and defaults
/// Returns None if settings equal defaults, otherwise returns a Value with only changed fields
pub(crate) fn compute_delta<T: Settings>(settings: &T) -> Option<Value> {
    let defaults = T::default();

    // If equal to defaults, no need to store
    if settings == &defaults {
        return None;
    }

    // Serialize both to JSON values
    let settings_value = serde_json::to_value(settings).ok()?;
    let defaults_value = serde_json::to_value(&defaults).ok()?;

    // Compute delta recursively
    compute_value_delta(&settings_value, &defaults_value)
}

/// Recursively compute delta between two JSON values
fn compute_value_delta(current: &Value, default: &Value) -> Option<Value> {
    match (current, default) {
        (Value::Object(curr_map), Value::Object(def_map)) => {
            let mut delta_map = Map::new();

            for (key, curr_val) in curr_map {
                if let Some(def_val) = def_map.get(key) {
                    // Key exists in both, check if different
                    if curr_val != def_val {
                        // Try to compute nested delta for objects
                        if let Some(nested_delta) = compute_value_delta(curr_val, def_val) {
                            delta_map.insert(key.clone(), nested_delta);
                        }
                    }
                } else {
                    // Key only in current, include it
                    delta_map.insert(key.clone(), curr_val.clone());
                }
            }

            if delta_map.is_empty() {
                None
            } else {
                Some(Value::Object(delta_map))
            }
        }
        _ => {
            // For non-object values, include if different
            if current != default {
                Some(current.clone())
            } else {
                None
            }
        }
    }
}

/// Merge delta with defaults to get complete settings
pub(crate) fn merge_with_defaults<T: Settings>(delta: Option<&Value>) -> Result<T> {
    let defaults = T::default();

    // If no delta, return defaults
    let Some(delta) = delta else {
        return Ok(defaults);
    };

    // Serialize defaults to JSON
    let mut defaults_value = serde_json::to_value(&defaults)?;

    // Merge delta into defaults
    merge_values(&mut defaults_value, delta);

    // Deserialize back to T
    let result: T = serde_json::from_value(defaults_value)?;
    Ok(result)
}

/// Recursively merge source into target
fn merge_values(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(target_map), Value::Object(source_map)) => {
            for (key, source_val) in source_map {
                if let Some(target_val) = target_map.get_mut(key) {
                    // Recursively merge nested objects
                    merge_values(target_val, source_val);
                } else {
                    // Key doesn't exist in target, add it
                    target_map.insert(key.clone(), source_val.clone());
                }
            }
        }
        (target, source) => {
            // Replace target with source
            *target = source.clone();
        }
    }
}

/// System that saves a specific settings type to the storage
pub(crate) fn save_settings_on_change<T: Settings>(
    settings: Res<T>,
    manager: Res<SettingsManager>,
) {
    if settings.is_changed() && !settings.is_added() {
        let type_key = get_type_key::<T>();

        // Compute delta (only changed fields)
        let delta = crate::storage::compute_delta(&*settings);

        // Update the shared settings map
        let mut map = manager.settings_map.lock().unwrap();

        if let Some(delta_value) = delta {
            map.insert(type_key.clone(), delta_value);
        } else {
            // Settings equal defaults, remove from map
            map.remove(&type_key);
        }

        // Get versions
        let versions = manager.versions.lock().unwrap();

        // Save all settings to disk
        if let Err(e) = manager.storage.save_all_with_versions(&map, &versions) {
            error!("Failed to save settings: {}", e);
        } else {
            info!("Settings saved");
        }
    }
}
#[derive(Resource, Clone)]
pub(crate) struct SettingsManager {
    pub storage: Storage,
    /// Shared map of all settings values (type_key -> JSON value)
    /// Using Arc<Mutex<>> to allow multiple systems to update the same map
    pub settings_map: Arc<Mutex<HashMap<String, Value>>>,
    /// Shared map of version information per section (section_name -> version string)
    pub versions: Arc<Mutex<HashMap<String, String>>>,
}

/// Get the type key for a settings type (uses SECTION constant)
pub(crate) fn get_type_key<T: Settings>() -> String {
    T::SECTION.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
    struct TestSettings {
        value: i32,
        name: String,
        nested: NestedSettings,
    }

    #[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
    struct NestedSettings {
        enabled: bool,
        count: u32,
    }

    impl bevy::prelude::Resource for TestSettings {}
    impl Settings for TestSettings {
        fn type_name() -> &'static str {
            "TestSettings"
        }

        const SECTION: &'static str = "testsettings";
    }

    #[test]
    fn test_compute_delta_no_changes() {
        let settings = TestSettings::default();
        let delta = compute_delta(&settings);
        assert!(delta.is_none());
    }

    #[test]
    fn test_compute_delta_with_changes() {
        let mut settings = TestSettings::default();
        settings.value = 42;

        let delta = compute_delta(&settings);
        assert!(delta.is_some());

        let delta_value = delta.unwrap();
        assert!(delta_value.get("value").is_some());
        assert_eq!(delta_value.get("value").unwrap(), &Value::Number(42.into()));

        // Unchanged fields should not be in delta
        assert!(delta_value.get("name").is_none());
    }

    #[test]
    fn test_merge_with_defaults() {
        let mut delta_map = Map::new();
        delta_map.insert("value".to_string(), Value::Number(100.into()));
        let delta = Value::Object(delta_map);

        let result: TestSettings = merge_with_defaults(Some(&delta)).unwrap();
        assert_eq!(result.value, 100);
        assert_eq!(result.name, String::default()); // Should use default
    }
}
