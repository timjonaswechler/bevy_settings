use crate::SettingsGroupTrait;
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;

/// Plugin for managing settings groups in Bevy.
///
/// This plugin provides functionality for loading, saving, and auto-saving settings groups.
/// It registers settings groups as Bevy resources and sets up systems for automatic persistence.
///
/// # Examples
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_settings::{SettingsPlugin, SettingsGroup};
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
/// fn main() {
///     App::new()
///         .add_plugins((MinimalPlugins, SettingsPlugin::default().register::<GraphicsSettings>()));
/// }
/// ```
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
    pub fn register<T: SettingsGroupTrait + Reflect + GetTypeRegistration>(mut self) -> Self {
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
        for registration in &self.registrations {
            registration(app);
        }
    }
}
