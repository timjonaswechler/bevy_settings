use crate::{
    SerializationFormat, Settings,
    common::{UnifiedSettingsManager, save_unified_settings_on_change, get_type_key},
    unified_storage::{UnifiedStorage, merge_with_defaults},
};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// A fluent API for managing settings in Bevy with unified storage
///
/// `SettingsStore` provides a builder-style API for configuring and registering
/// multiple settings types with shared configuration. All settings are stored in
/// a single file with optional versioning.
///
/// It can be used in two ways:
///
/// 1. As a Plugin (recommended): Automatically loads settings on startup and saves on changes
/// 2. As a Resource: For manual control over loading and saving
///
/// # Example as Plugin
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_settings::{Settings, SettingsStore, SerializationFormat};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
/// struct Input {
///     mouse_sensitivity: f32,
/// }
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(
///         SettingsStore::new("GameSettings")
///             .format(SerializationFormat::Json)
///             .version("0.1.0")
///             .with_base_path("settings")
///             .register::<Input>()
///     )
///     .run();
/// ```
///
/// # Example as Resource
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_settings::{Settings, SettingsStore, SerializationFormat};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
/// struct Input {
///     mouse_sensitivity: f32,
/// }
///
/// let settings_store = SettingsStore::new("GameSettings")
///     .format(SerializationFormat::Json)
///     .with_base_path("settings")
///     .register::<Input>();
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .insert_resource(settings_store)
///     .run();
/// ```
#[derive(Resource)]
pub struct SettingsStore {
    /// Name or identifier for this settings store (used as filename)
    name: String,
    /// Serialization format for all settings in this store
    format: SerializationFormat,
    /// Version string (included in the settings file)
    version: Option<String>,
    /// Base path for settings files
    base_path: Option<String>,
    /// Registered settings handlers
    handlers: Vec<Box<dyn SettingsHandler>>,
}

impl SettingsStore {
    /// Create a new settings store with the given name
    ///
    /// The name will be used as the filename for the unified settings file.
    ///
    /// # Arguments
    /// * `name` - Name for this settings store (e.g., "GameSettings" will create "GameSettings.json")
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            format: SerializationFormat::Json, // Default to JSON
            version: None,
            base_path: None,
            handlers: Vec::new(),
        }
    }

    /// Set the serialization format for all settings in this store
    ///
    /// # Arguments
    /// * `format` - Either `SerializationFormat::Json` or `SerializationFormat::Binary`
    pub fn format(mut self, format: SerializationFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the version for this settings store
    ///
    /// The version will be included in the settings file for future migration support.
    ///
    /// # Arguments
    /// * `version` - Version string (e.g., "0.1.0")
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the base path for settings file storage
    ///
    /// # Arguments
    /// * `path` - Base directory path for settings files
    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = Some(path.into());
        self
    }

    /// Register a settings type with this store
    ///
    /// All settings registered with this store will be saved in the unified file
    /// with their type name (lowercase) as the key.
    ///
    /// # Type Parameters
    /// * `T` - The settings type to register (must implement `Settings`)
    pub fn register<T: Settings + 'static>(mut self) -> Self {
        let handler = Box::new(TypedSettingsHandler::<T>::new());
        self.handlers.push(handler);
        self
    }

    /// Get the base path, using "settings" as default if not set
    fn get_base_path(&self) -> String {
        self.base_path.as_deref().unwrap_or("settings").to_string()
    }

    /// Get the name of this settings store
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the version of this settings store
    pub fn get_version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Get the serialization format
    pub fn get_format(&self) -> SerializationFormat {
        self.format
    }

    /// Get the base path
    pub fn get_base_path_option(&self) -> Option<&str> {
        self.base_path.as_deref()
    }
}

impl Plugin for SettingsStore {
    fn build(&self, app: &mut App) {
        let base_path = self.get_base_path();

        // Create unified storage
        let mut storage = UnifiedStorage::new(&self.name, self.format);
        storage = storage.with_base_path(&base_path);
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
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Settings> TypedSettingsHandler<T> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
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
