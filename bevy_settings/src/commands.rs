use crate::{SettingsGroupTrait, manager};
use bevy::ecs::system::{Command, Commands}; // Explicit import
use bevy::prelude::*;
use std::marker::PhantomData;

/// Extension trait for `bevy::prelude::Commands` to load and save settings.
///
/// Provides a convenient way to queue settings operations (e.g., loading or saving a settings group)
/// as commands in the Bevy ECS. This ensures that settings operations are executed in a deterministic
/// and thread-safe manner.
///
/// # Examples
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_settings::{SettingsCommandsExt, SettingsGroupTrait};
/// use bevy_settings_derive::SettingsGroup;
/// use bevy_paths::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(SettingsGroup, Serialize, Deserialize, Resource, Clone, PartialEq, Reflect, Default)]
/// #[serde(default)]
/// #[settings("settings/graphics.json")]
/// struct GraphicsSettings {
///     resolution: String,
///     vsync: bool,
/// }
///
/// fn load_graphics_settings(mut commands: Commands) {
///     commands.load_settings::<GraphicsSettings>();
/// }
/// ```
pub trait SettingsCommandsExt {
    /// Queues a command to load a settings group from disk.
    ///
    /// The settings group is loaded asynchronously and applied to the ECS as a resource.
    /// If the file does not exist, the settings group is initialized with default values.
    ///
    /// # Type Parameters
    /// - `T`: The settings group type (must implement `SettingsGroupTrait`, `Send`, `Sync`, and `'static`).
    ///
    /// # Examples
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_settings::{SettingsCommandsExt, SettingsGroupTrait};
    /// use bevy_settings_derive::SettingsGroup;
    /// use bevy_paths::prelude::*;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(SettingsGroup, Serialize, Deserialize, Resource, Clone, PartialEq, Reflect, Default)]
    /// #[serde(default)]
    /// #[settings("settings/audio.json")]
    /// struct AudioSettings {
    ///     volume: f32,
    /// }
    ///
    /// fn save_audio_settings(mut commands: Commands) {
    ///     commands.save_settings::<AudioSettings>();
    /// }
    /// ```
    fn load_settings<T: SettingsGroupTrait + Send + Sync + 'static>(&mut self);

    /// Queues a command to save a settings group to disk.
    ///
    /// The settings group is serialized and saved to disk asynchronously. Only fields that differ
    /// from their default values are saved (delta encoding).
    ///
    /// # Type Parameters
    /// - `T`: The settings group type (must implement `SettingsGroupTrait`, `Send`, `Sync`, and `'static`).
    ///
    /// # Examples
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_settings::{SettingsCommandsExt, SettingsGroupTrait};
    /// use bevy_settings_derive::SettingsGroup;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(SettingsGroup, Serialize, Deserialize, Resource, Clone, PartialEq, Reflect, Default)]
    /// #[settings("settings/audio.json")]
    /// struct AudioSettings {
    ///     volume: f32,
    /// }
    ///
    /// fn save_audio_settings(mut commands: Commands) {
    ///     commands.save_settings::<AudioSettings>();
    /// }
    /// ```
    fn save_settings<T: SettingsGroupTrait + Send + Sync + 'static>(&mut self);
}

impl<'w, 's> SettingsCommandsExt for Commands<'w, 's> {
    fn load_settings<T: SettingsGroupTrait + Send + Sync + 'static>(&mut self) {
        let command = LoadSettingsCommand::<T> {
            _phantom: PhantomData,
        };
        self.queue(command);
    }

    fn save_settings<T: SettingsGroupTrait + Send + Sync + 'static>(&mut self) {
        let command = SaveSettingsCommand::<T> {
            _phantom: PhantomData,
        };
        self.queue(command);
    }
}

// --- Systems ---

pub fn auto_save_system<T: SettingsGroupTrait + Send + Sync + 'static>(
    mut commands: Commands,
    settings: Res<T>,
) {
    if settings.is_changed() && !settings.is_added() {
        commands.save_settings::<T>();
    }
}

// --- Commands ---

struct LoadSettingsCommand<T> {
    _phantom: PhantomData<T>,
}

impl<T: SettingsGroupTrait + Send + Sync + 'static> Command for LoadSettingsCommand<T> {
    fn apply(self, world: &mut World) {
        if let Err(e) = manager::load_settings_logic::<T>(world) {
            error!(
                "Failed to load settings for {}: {}",
                std::any::type_name::<T>(),
                e
            );
        }
    }
}

struct SaveSettingsCommand<T> {
    _phantom: PhantomData<T>,
}

impl<T: SettingsGroupTrait + Send + Sync + 'static> Command for SaveSettingsCommand<T> {
    fn apply(self, world: &mut World) {
        if let Err(e) = manager::save_settings_logic::<T>(world) {
            error!(
                "Failed to save settings for {}: {}",
                std::any::type_name::<T>(),
                e
            );
        }
    }
}
