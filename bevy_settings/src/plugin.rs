use crate::{SerializationFormat, Settings, SettingsStorage};
use bevy::prelude::*;
use std::marker::PhantomData;

/// Legacy plugin for managing a single settings type in Bevy (deprecated)
///
/// **Note:** This is the legacy API. For new code, use the new `SettingsPlugin`
/// which allows registering multiple settings types with a single plugin.
///
/// This plugin:
/// - Loads settings from disk on startup
/// - Saves settings to disk when they change
/// - Manages settings as Bevy resources
///
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_settings::{Settings, TypedSettingsPlugin, SerializationFormat};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
/// struct GameSettings {
///     volume: f32,
/// }
///
/// App::new()
///     .add_plugins(TypedSettingsPlugin::<GameSettings>::new(
///         "game_settings",
///         SerializationFormat::Json,
///     ));
/// ```
pub struct SettingsPlugin<T: Settings> {
    name: String,
    storage: SettingsStorage,
    _phantom: PhantomData<T>,
}

impl<T: Settings> SettingsPlugin<T> {
    /// Create a new settings plugin
    ///
    /// # Arguments
    /// * `name` - Name for the settings file (without extension)
    /// * `format` - Serialization format (JSON or Binary)
    pub fn new(name: impl Into<String>, format: SerializationFormat) -> Self {
        Self {
            name: name.into(),
            storage: SettingsStorage::new(format),
            _phantom: PhantomData,
        }
    }

    /// Set a custom base path for settings storage
    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.storage = self.storage.with_base_path(path.into());
        self
    }
}

impl<T: Settings> Plugin for SettingsPlugin<T> {
    fn build(&self, app: &mut App) {
        // Load settings or use defaults
        let settings = self.storage.load::<T>(&self.name).unwrap_or_else(|e| {
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
            name: self.name.clone(),
            storage: self.storage.clone(),
            _phantom: PhantomData,
        });

        // Add system to save settings when they change
        app.add_systems(PostUpdate, save_settings_on_change::<T>);
    }
}

/// Resource that manages settings persistence
#[derive(Resource, Clone)]
struct SettingsManager<T: Settings> {
    name: String,
    storage: SettingsStorage,
    _phantom: PhantomData<T>,
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

impl Clone for SettingsStorage {
    fn clone(&self) -> Self {
        Self {
            format: self.format,
            base_path: self.base_path.clone(),
        }
    }
}
