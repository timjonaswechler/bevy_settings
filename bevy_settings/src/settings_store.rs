use crate::{
    SerializationFormat, Settings, SettingsStorage,
    common::{SettingsManager, save_settings_on_change},
};
use bevy::{
    app::{App, Plugin, PostUpdate},
    ecs::resource::Resource,
    log::warn,
};

/// A fluent API for managing settings in Bevy
///
/// `SettingsStore` provides a builder-style API for configuring and registering
/// multiple settings types with shared configuration. It can be used in two ways:
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
    /// Name or identifier for this settings store
    name: String,
    /// Serialization format for all settings in this store
    format: SerializationFormat,
    /// Version string (currently for documentation, future versioning support)
    version: Option<String>,
    /// Base path for settings files
    base_path: Option<String>,
    /// Registered settings handlers
    handlers: Vec<Box<dyn SettingsHandler>>,
}

impl SettingsStore {
    /// Create a new settings store with the given name
    ///
    /// The name can be a simple identifier like "GameSettings" or a placeholder
    /// like "[gameplay]" for dynamic paths (e.g., save game slots).
    ///
    /// # Arguments
    /// * `name` - Name or identifier for this settings store
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
    /// This is currently used for documentation and can be used in the future
    /// for settings migration between versions.
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
    /// All settings registered with this store will use the same format and base path
    /// configured for the store. The settings file name will be derived from the type name.
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

        // Load and insert all registered settings
        for handler in &self.handlers {
            handler.load_and_insert(app, &self.name, self.format, &base_path);
        }

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
        store_name: &str,
        format: SerializationFormat,
        base_path: &str,
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

    /// Generate settings file name based on store name
    fn get_settings_name(store_name: &str) -> String {
        if store_name.starts_with('[') && store_name.ends_with(']') {
            // Placeholder: combine placeholder and type name
            format!(
                "{}_{}",
                &store_name[1..store_name.len() - 1],
                T::type_name()
            )
        } else {
            // Regular name: just use type name
            T::type_name().to_string()
        }
    }
}

impl<T: Settings> SettingsHandler for TypedSettingsHandler<T> {
    fn load_and_insert(
        &self,
        app: &mut App,
        store_name: &str,
        format: SerializationFormat,
        base_path: &str,
    ) {
        let mut storage = SettingsStorage::new(format);
        storage = storage.with_base_path(base_path);

        let settings_name = Self::get_settings_name(store_name);

        // Load settings or use defaults
        let settings = storage.load::<T>(&settings_name).unwrap_or_else(|e| {
            warn!(
                "Failed to load settings for {}: {}. Using defaults.",
                T::type_name(),
                e
            );
            T::default()
        });

        // Insert as resource
        app.insert_resource(settings);
        app.insert_resource(SettingsManager::<T> {
            name: settings_name,
            storage,
            _phantom: std::marker::PhantomData,
        });
    }

    fn register_save_system(&self, app: &mut App) {
        app.add_systems(PostUpdate, save_settings_on_change::<T>);
    }
}
