use crate::{error::Result, SerializationFormat, Settings};
use std::fs;
use std::path::{Path, PathBuf};

/// Handles storage and retrieval of settings
pub struct SettingsStorage {
    pub(crate) format: SerializationFormat,
    pub(crate) base_path: PathBuf,
}

impl SettingsStorage {
    /// Create a new settings storage with the specified format
    pub fn new(format: SerializationFormat) -> Self {
        Self {
            format,
            base_path: PathBuf::from("settings"),
        }
    }

    /// Set the base path for settings files
    pub fn with_base_path(mut self, path: impl AsRef<Path>) -> Self {
        self.base_path = path.as_ref().to_path_buf();
        self
    }

    /// Get the full path for a settings file
    fn get_path(&self, name: &str) -> PathBuf {
        self.base_path
            .join(format!("{}.{}", name, self.format.extension()))
    }

    /// Load settings from disk, merging with defaults
    /// 
    /// Only the values that differ from defaults are stored, so this
    /// loads the stored values and merges them with defaults.
    pub fn load<T: Settings>(&self, name: &str) -> Result<T> {
        let path = self.get_path(name);
        
        // If file doesn't exist, return defaults
        if !path.exists() {
            return Ok(T::default());
        }

        let content = fs::read(&path)?;
        
        // Deserialize based on format
        let settings = match self.format {
            SerializationFormat::Json => serde_json::from_slice(&content)?,
            SerializationFormat::Binary => bincode::deserialize(&content)?,
        };

        Ok(settings)
    }

    /// Save settings to disk, storing only values that differ from defaults
    /// 
    /// This compares the current settings with defaults. If they are identical,
    /// the settings file is deleted (no need to store defaults). Otherwise,
    /// the entire settings object is saved.
    pub fn save<T: Settings>(&self, name: &str, settings: &T) -> Result<()> {
        let path = self.get_path(name);
        
        // If settings equal defaults, delete the file (no need to store defaults)
        if settings == &T::default() {
            if path.exists() {
                fs::remove_file(&path)?;
            }
            return Ok(());
        }

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize based on format
        let content = match self.format {
            SerializationFormat::Json => serde_json::to_vec_pretty(settings)?,
            SerializationFormat::Binary => bincode::serialize(settings)?,
        };

        fs::write(&path, content)?;
        Ok(())
    }

    /// Delete settings file
    pub fn delete(&self, name: &str) -> Result<()> {
        let path = self.get_path(name);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
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
    }

    // Manual implementation for testing without the derive macro
    impl bevy::prelude::Resource for TestSettings {}
    
    impl Settings for TestSettings {
        fn type_name() -> &'static str {
            "TestSettings"
        }
    }

    #[test]
    fn test_json_storage() {
        let storage = SettingsStorage::new(SerializationFormat::Json)
            .with_base_path("/tmp/bevy_settings_test");

        let settings = TestSettings {
            value: 42,
            name: "test".to_string(),
        };

        // Save and load
        storage.save("test", &settings).unwrap();
        let loaded: TestSettings = storage.load("test").unwrap();
        assert_eq!(settings, loaded);

        // Clean up
        storage.delete("test").unwrap();
    }

    #[test]
    fn test_binary_storage() {
        let storage = SettingsStorage::new(SerializationFormat::Binary)
            .with_base_path("/tmp/bevy_settings_test");

        let settings = TestSettings {
            value: 100,
            name: "binary_test".to_string(),
        };

        // Save and load
        storage.save("test_binary", &settings).unwrap();
        let loaded: TestSettings = storage.load("test_binary").unwrap();
        assert_eq!(settings, loaded);

        // Clean up
        storage.delete("test_binary").unwrap();
    }

    #[test]
    fn test_default_settings() {
        let storage = SettingsStorage::new(SerializationFormat::Json)
            .with_base_path("/tmp/bevy_settings_test");

        // Saving default settings should not create a file
        let defaults = TestSettings::default();
        storage.save("defaults", &defaults).unwrap();
        
        let path = storage.get_path("defaults");
        assert!(!path.exists());

        // Loading non-existent file should return defaults
        let loaded: TestSettings = storage.load("nonexistent").unwrap();
        assert_eq!(loaded, TestSettings::default());
    }
}
