use crate::{SerializationFormat, Settings, error::Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Unified storage that saves multiple settings types to a single file
#[derive(Clone)]
pub struct UnifiedStorage {
    pub(crate) format: SerializationFormat,
    pub(crate) base_path: PathBuf,
    pub(crate) filename: String,
    pub(crate) version: Option<String>,
}

impl UnifiedStorage {
    /// Create a new unified storage with the specified format
    pub fn new(filename: impl Into<String>, format: SerializationFormat) -> Self {
        Self {
            format,
            base_path: PathBuf::from("settings"),
            filename: filename.into(),
            version: None,
        }
    }

    /// Set the base path for settings files
    pub fn with_base_path(mut self, path: impl AsRef<Path>) -> Self {
        self.base_path = path.as_ref().to_path_buf();
        self
    }

    /// Set the version string
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Get the full path for the unified settings file
    fn get_path(&self) -> PathBuf {
        self.base_path
            .join(format!("{}.{}", self.filename, self.format.extension()))
    }

    /// Load all settings from the unified file
    pub fn load_all(&self) -> Result<Map<String, Value>> {
        let path = self.get_path();

        // If file doesn't exist, return empty map
        if !path.exists() {
            return Ok(Map::new());
        }

        let content = fs::read(&path)?;
        
        // Deserialize based on format
        let root: Value = match self.format {
            SerializationFormat::Json => serde_json::from_slice(&content)?,
            SerializationFormat::Binary => {
                let config = bincode::config::standard();
                bincode::serde::decode_from_slice(&content, config)
                    .map_err(|e| crate::error::SettingsError::BincodeDecode(e))?
                    .0
            }
        };

        // Extract the settings map (skip version field)
        if let Value::Object(mut map) = root {
            // Remove version from the map (it's metadata, not settings)
            map.remove("version");
            Ok(map)
        } else {
            Ok(Map::new())
        }
    }

    /// Load a specific settings type from the unified file (available for manual usage)
    #[allow(dead_code)]
    pub fn load<T: Settings>(&self, type_key: &str) -> Result<T> {
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

    /// Save multiple settings types to the unified file
    pub fn save_all(&self, settings_map: &HashMap<String, Value>) -> Result<()> {
        let path = self.get_path();
        
        // If all settings are empty (equal to defaults), delete the file
        if settings_map.is_empty() {
            if path.exists() {
                fs::remove_file(&path)?;
            }
            return Ok(());
        }

        // Build the root object with version and all settings
        let mut root = Map::new();
        
        // Add version if present
        if let Some(ref version) = self.version {
            root.insert("version".to_string(), Value::String(version.clone()));
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
                let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
                let size = bincode::serde::encode_into_slice(&root_value, &mut buffer, config)
                    .map_err(|e| crate::error::SettingsError::BincodeEncode(e))?;
                buffer.truncate(size);
                buffer
            }
        };

        fs::write(&path, content)?;
        Ok(())
    }

    /// Delete the unified settings file (available for manual usage)
    #[allow(dead_code)]
    pub fn delete(&self) -> Result<()> {
        let path = self.get_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

/// Compute delta between current settings and defaults
/// Returns None if settings equal defaults, otherwise returns a Value with only changed fields
pub fn compute_delta<T: Settings>(settings: &T) -> Option<Value> {
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
pub fn merge_with_defaults<T: Settings>(delta: Option<&Value>) -> Result<T> {
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
