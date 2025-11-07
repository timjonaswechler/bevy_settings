use crate::{
    SerializationFormat, Settings, SettingsStorage,
    common::{SettingsManager, save_settings_on_change},
};
use bevy::prelude::*;
use std::marker::PhantomData;

/// Plugin for managing all settings in Bevy using a fluent builder API.
///
/// Usage:
/// ```no_run
/// App::new()
///     .add_plugins(
///         SettingsPlugin::new("GameSettings")
///             .format(SerializationFormat::Json)
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
    /// Create a new settings plugin bound to a store name.
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

    /// Set a version string (for future migrations / metadata)
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set base path for settings files
    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = Some(path.into());
        self
    }

    /// Register a settings type `T`. File name will be derived from store name + type as implemented.
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
        store_name: &str,
        format: SerializationFormat,
        base_path: &str,
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

    /// Generate settings file name based on store name (supports placeholder `[slot]` style)
    fn get_settings_name(store_name: &str) -> String {
        if store_name.starts_with('[') && store_name.ends_with(']') {
            format!(
                "{}_{}",
                &store_name[1..store_name.len() - 1],
                T::type_name()
            )
        } else {
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

        // Insert as resource and manager
        app.insert_resource(settings);
        app.insert_resource(SettingsManager::<T> {
            name: settings_name,
            storage,
            _phantom: PhantomData,
        });
    }

    fn register_save_system(&self, app: &mut App) {
        app.add_systems(PostUpdate, save_settings_on_change::<T>);
    }
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let base_path = self.base_path.as_deref().unwrap_or("settings");

        // Load and insert all registered settings
        for handler in &self.handlers {
            handler.load_and_insert(app, &self.name, self.format, base_path);
        }

        // Register save systems for all settings
        for handler in &self.handlers {
            handler.register_save_system(app);
        }
    }
}
