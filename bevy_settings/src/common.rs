use crate::{Settings, SettingsStorage};
use bevy::{
    ecs::{change_detection::DetectChanges, system::Resource},
    log::{error, info},
    prelude::Res,
};

/// Resource that manages settings persistence for a specific settings type
#[derive(Resource, Clone)]
pub(crate) struct SettingsManager<T: Settings> {
    pub name: String,
    pub storage: SettingsStorage,
    pub _phantom: std::marker::PhantomData<T>,
}

/// System that saves settings when they are modified
pub(crate) fn save_settings_on_change<T: Settings>(
    settings: Res<T>,
    manager: Res<SettingsManager<T>>,
) {
    if settings.is_changed() && !settings.is_added() {
        if let Err(e) = manager.storage.save(&manager.name, &*settings) {
            error!("Failed to save settings for {}: {}", T::type_name(), e);
        } else {
            info!("Settings saved for {}", T::type_name());
        }
    }
}
