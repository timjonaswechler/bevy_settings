use crate::{SerializationFormat, Settings, SettingsStorage};
use bevy::{
    app::{App, Plugin, PostUpdate},
    ecs::{change_detection::DetectChanges, system::Resource},
    log::{error, info, warn},
    prelude::Res,
};
use std::any::TypeId;
use std::collections::HashMap;

/// Configuration for a settings type
pub struct SettingsConfig {
    /// Name for the settings file (without extension)
    pub name: String,
    /// Serialization format
    pub format: SerializationFormat,
    /// Base path for settings storage
    pub base_path: Option<String>,
}

impl SettingsConfig {
    /// Create a new settings configuration
    pub fn new(name: impl Into<String>, format: SerializationFormat) -> Self {
        Self {
            name: name.into(),
            format,
            base_path: None,
        }
    }

    /// Set a custom base path for settings storage
    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = Some(path.into());
        self
    }
}

/// Internal trait for type-erased settings operations
trait SettingsHandler: Send + Sync {
    fn load_and_insert(&self, app: &mut App, config: &SettingsConfig);
    fn register_save_system(&self, app: &mut App, config: &SettingsConfig);
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
    fn load_and_insert(&self, app: &mut App, config: &SettingsConfig) {
        let mut storage = SettingsStorage::new(config.format);
        if let Some(ref base_path) = config.base_path {
            storage = storage.with_base_path(base_path.clone());
        }

        // Load settings or use defaults
        let settings = storage.load::<T>(&config.name).unwrap_or_else(|e| {
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
            name: config.name.clone(),
            storage,
            _phantom: std::marker::PhantomData,
        });
    }

    fn register_save_system(&self, app: &mut App, _config: &SettingsConfig) {
        app.add_systems(PostUpdate, save_settings_on_change::<T>);
    }
}

/// Resource that manages settings persistence
#[derive(Resource, Clone)]
struct SettingsManager<T: Settings> {
    name: String,
    storage: SettingsStorage,
    _phantom: std::marker::PhantomData<T>,
}

/// System that saves settings when they are modified
fn save_settings_on_change<T: Settings>(settings: Res<T>, manager: Res<SettingsManager<T>>) {
    if settings.is_changed() && !settings.is_added() {
        if let Err(e) = manager.storage.save(&manager.name, &*settings) {
            error!("Failed to save settings for {}: {}", T::type_name(), e);
        } else {
            info!("Settings saved for {}", T::type_name());
        }
    }
}

/// Plugin for managing all settings in Bevy
///
/// This plugin manages multiple settings types through a single plugin instance.
/// Register settings types before adding the plugin to your app.
///
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_settings::{Settings, SettingsPlugin, SettingsConfig, SerializationFormat};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
/// struct GameSettings {
///     volume: f32,
/// }
///
/// #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
/// struct GraphicsSettings {
///     quality: i32,
/// }
///
/// App::new()
///     .add_plugins(
///         SettingsPlugin::new()
///             .register::<GameSettings>(SettingsConfig::new("game", SerializationFormat::Json))
///             .register::<GraphicsSettings>(SettingsConfig::new("graphics", SerializationFormat::Json))
///     );
/// ```
pub struct SettingsPlugin {
    handlers: HashMap<TypeId, (Box<dyn SettingsHandler>, SettingsConfig)>,
}

impl SettingsPlugin {
    /// Create a new settings plugin
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a settings type with configuration
    pub fn register<T: Settings + 'static>(mut self, config: SettingsConfig) -> Self {
        let type_id = TypeId::of::<T>();
        let handler = Box::new(TypedSettingsHandler::<T>::new());
        self.handlers.insert(type_id, (handler, config));
        self
    }
}

impl Default for SettingsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        // Load and insert all registered settings
        for (handler, config) in self.handlers.values() {
            handler.load_and_insert(app, config);
        }

        // Register save systems for all settings
        for (handler, config) in self.handlers.values() {
            handler.register_save_system(app, config);
        }
    }
}
