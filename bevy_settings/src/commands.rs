use crate::{manager, SettingsGroup};
use bevy::ecs::system::{Command, Commands}; // Explicit import
use bevy::prelude::*;
use std::marker::PhantomData;

/// Extension trait for `Commands` to load/save settings.
pub trait SettingsCommandsExt {
    fn load_settings<T: SettingsGroup + Send + Sync + 'static>(&mut self);
    fn save_settings<T: SettingsGroup + Send + Sync + 'static>(&mut self);
}

impl<'w, 's> SettingsCommandsExt for Commands<'w, 's> {
    fn load_settings<T: SettingsGroup + Send + Sync + 'static>(&mut self) {
        let command = LoadSettingsCommand::<T> {
            _phantom: PhantomData,
        };
        self.queue(command);
    }

    fn save_settings<T: SettingsGroup + Send + Sync + 'static>(&mut self) {
        let command = SaveSettingsCommand::<T> {
            _phantom: PhantomData,
        };
        self.queue(command);
    }
}

// --- Systems ---

pub fn auto_save_system<T: SettingsGroup + Send + Sync + 'static>(
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

impl<T: SettingsGroup + Send + Sync + 'static> Command for LoadSettingsCommand<T> {
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

impl<T: SettingsGroup + Send + Sync + 'static> Command for SaveSettingsCommand<T> {
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
