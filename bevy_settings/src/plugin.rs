use crate::{
    storage::{
        get_type_key, merge_with_defaults, save_settings_on_change, SettingsManager, Storage,
    },
    SerializationFormat, Settings,
};
use bevy::prelude::*;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

/// Plugin for managing all settings in Bevy using a fluent builder API with storage.
///
/// This plugin stores all registered settings in a single file instead of separate files per type.
/// The file contains a JSON structure with optional version and all settings as sub-objects.
///
/// Usage:
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_settings::{Settings, SettingsPlugin, SerializationFormat};
/// # use serde::{Deserialize, Serialize};
/// #
/// # #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
/// # struct GameSettings { volume: f32 }
/// #
/// # #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
/// # struct AudioSettings { master: f32 }
/// #
/// App::new()
///     .add_plugins(
///         SettingsPlugin::new("GameSettings")
///             .format(SerializationFormat::Json)
///             .version("0.1.0")
///             .with_base_path("settings")
///             .register::<GameSettings>()
///             .register::<AudioSettings>()
///     );
/// ```
pub struct SettingsPlugin {
    storage: Storage,
    handlers: Vec<Box<dyn SettingsHandler>>,
}

impl SettingsPlugin {
    pub fn new(name: impl Into<String>) -> Self {
        // Default to package version
        let storage = Storage::new(name.into(), SerializationFormat::Json)
            .with_version(env!("CARGO_PKG_VERSION"));
        Self {
            storage,
            handlers: Vec::new(),
        }
    }

    pub fn format(mut self, format: SerializationFormat) -> Self {
        self.storage.format = format;
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.storage = self.storage.with_version(version);
        self
    }

    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.storage = self.storage.with_base_path(path.into());
        self
    }

    pub fn register<T: Settings + 'static>(mut self) -> Self {
        let handler = Box::new(TypedSettingsHandler::<T>::new());
        self.handlers.push(handler);
        self
    }
}

impl Default for SettingsPlugin {
    fn default() -> Self {
        Self::new("Settings")
    }
}

/// Internal trait for type-erased settings operations
trait SettingsHandler: Send + Sync {
    fn load_and_insert(&self, app: &mut App, storage: &Storage, versions: &mut HashMap<String, String>);
    fn register_save_system(&self, app: &mut App);
}

/// Concrete implementation of SettingsHandler for a specific type
struct TypedSettingsHandler<T: Settings> {
    _phantom: PhantomData<T>,
}

impl<T: Settings> TypedSettingsHandler<T> {
    fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: Settings> SettingsHandler for TypedSettingsHandler<T> {
    fn load_and_insert(&self, app: &mut App, storage: &Storage, versions: &mut HashMap<String, String>) {
        let type_key = get_type_key::<T>();

        // Load all settings and version info from file
        let (all_settings, file_versions) = storage.load_all_with_versions().unwrap_or_else(|e| {
            warn!("Failed to load settings: {}. Using defaults.", e);
            (serde_json::Map::new(), serde_json::Map::new())
        });

        // Get delta for this type
        let delta = all_settings.get(&type_key);

        // Parse versions - use the storage version as target version
        let file_version = file_versions
            .get(&type_key)
            .and_then(|v| v.as_str())
            .and_then(|s| semver::Version::parse(s).ok());

        let target_version = storage
            .version
            .as_ref()
            .and_then(|s| semver::Version::parse(s).ok());

        // Apply migration if needed
        let migrated_delta = if let Some(delta_value) = delta {
            match T::migrate(file_version.as_ref(), target_version.as_ref().unwrap_or(&semver::Version::new(0, 0, 0)), delta_value.clone()) {
                Ok((migrated, changed)) => {
                    if changed {
                        info!("Migrated settings for {} from {:?} to {:?}", T::type_name(), file_version, target_version);
                    }
                    Some(migrated)
                }
                Err(e) => {
                    warn!("Failed to migrate settings for {}: {}. Using delta as-is.", T::type_name(), e);
                    Some(delta_value.clone())
                }
            }
        } else {
            None
        };

        // Merge with defaults
        let settings = merge_with_defaults::<T>(migrated_delta.as_ref()).unwrap_or_else(|e| {
            warn!(
                "Failed to merge settings for {}: {}. Using defaults.",
                T::type_name(),
                e
            );
            T::default()
        });

        // Store version for this section from storage
        if let Some(ref version_str) = storage.version {
            versions.insert(type_key.clone(), version_str.clone());
        }

        // Insert as resource
        app.insert_resource(settings);
    }

    fn register_save_system(&self, app: &mut App) {
        app.add_systems(PostUpdate, save_settings_on_change::<T>);
    }
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let storage = self.storage.clone();
        let mut versions = HashMap::new();

        for handler in &self.handlers {
            handler.load_and_insert(app, &storage, &mut versions);
        }

        app.insert_resource(SettingsManager {
            storage,
            settings_map: Arc::new(Mutex::new(HashMap::new())),
            versions: Arc::new(Mutex::new(versions)),
        });

        for handler in &self.handlers {
            handler.register_save_system(app);
        }
    }
}
