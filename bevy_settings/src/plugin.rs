use crate::{
    SerializationFormat, Settings,
    common::{UnifiedSettingsManager, save_unified_settings_on_change, get_type_key},
    unified_storage::{UnifiedStorage, merge_with_defaults},
};
use bevy::prelude::*;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Plugin for managing all settings in Bevy using a fluent builder API with unified storage.
///
/// This plugin stores all registered settings in a single file instead of separate files per type.
/// The file contains a JSON structure with optional version and all settings as sub-objects.
///
/// Usage:
/// ```no_run
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
    name: String,
    format: SerializationFormat,
    version: Option<String>,
    base_path: Option<String>,
    handlers: Vec<Box<dyn SettingsHandler>>,
}

impl SettingsPlugin {
    /// Create a new settings plugin with the given store name.
    /// The store name will be used as the filename for the unified settings file.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            format: SerializationFormat::Json,
            version: None,
            base_path: None,
            handlers: Vec::new(),
        }
    }

    /// Set serialization format for all registered settings
    pub fn format(mut self, format: SerializationFormat) -> Self {
        self.format = format;
        self
    }

    /// Set a version string that will be included in the settings file
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set base path for settings files
    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = Some(path.into());
        self
    }

    /// Register a settings type. All registered types will be saved in the unified file.
    pub fn register<T: Settings + 'static>(mut self) -> Self {
        let handler = Box::new(TypedSettingsHandler::<T>::new());
        self.handlers.push(handler);
        self
    }
}

impl Default for SettingsPlugin {
    fn default() -> Self {
        Self::new("GameSettings")
    }
}

/// Internal trait for type-erased settings operations
trait SettingsHandler: Send + Sync {
    fn load_and_insert(
        &self,
        app: &mut App,
        storage: &UnifiedStorage,
    );
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
    fn load_and_insert(
        &self,
        app: &mut App,
        storage: &UnifiedStorage,
    ) {
        let type_key = get_type_key::<T>();
        
        // Load all settings from unified file
        let all_settings = storage.load_all().unwrap_or_else(|e| {
            warn!("Failed to load unified settings: {}. Using defaults.", e);
            serde_json::Map::new()
        });
        
        // Get delta for this type and merge with defaults
        let delta = all_settings.get(&type_key);
        let settings = merge_with_defaults::<T>(delta).unwrap_or_else(|e| {
            warn!(
                "Failed to merge settings for {}: {}. Using defaults.",
                T::type_name(),
                e
            );
            T::default()
        });
        
        // Insert as resource
        app.insert_resource(settings);
    }

    fn register_save_system(&self, app: &mut App) {
        app.add_systems(PostUpdate, save_unified_settings_on_change::<T>);
    }
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let base_path = self.base_path.as_deref().unwrap_or("settings");
        
        // Create unified storage
        let mut storage = UnifiedStorage::new(&self.name, self.format);
        storage = storage.with_base_path(base_path);
        if let Some(ref version) = self.version {
            storage = storage.with_version(version);
        }
        
        // Load and insert all registered settings
        for handler in &self.handlers {
            handler.load_and_insert(app, &storage);
        }
        
        // Insert the unified settings manager
        app.insert_resource(UnifiedSettingsManager {
            storage,
            settings_map: Arc::new(Mutex::new(HashMap::new())),
        });
        
        // Register save systems for all settings
        for handler in &self.handlers {
            handler.register_save_system(app);
        }
    }
}
