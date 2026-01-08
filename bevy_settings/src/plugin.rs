use crate::SettingsGroup;
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;

/// Plugin for managing settings groups.
pub struct SettingsPlugin {
    registrations: Vec<Box<dyn Fn(&mut App) + Send + Sync>>,
}

impl Default for SettingsPlugin {
    fn default() -> Self {
        Self {
            registrations: Vec::new(),
        }
    }
}

impl SettingsPlugin {
    /// Registers a settings group type.
    pub fn register<T: SettingsGroup + Reflect + GetTypeRegistration>(mut self) -> Self {
        self.registrations.push(Box::new(|app| {
            app.register_type::<T>();
            app.init_resource::<T>();

            // Register auto-save system
            app.add_systems(PostUpdate, crate::commands::auto_save_system::<T>);
        }));
        self
    }
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_paths::PathRegistryPlugin>() {
            warn!("PathRegistryPlugin is not added. Settings paths might not resolve correctly.");
        }

        for registration in &self.registrations {
            registration(app);
        }
    }
}
