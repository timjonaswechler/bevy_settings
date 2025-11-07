use crate::{Settings, SettingsStorage, unified_storage::UnifiedStorage};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use serde_json::Value;
use std::collections::HashMap;

/// Resource that manages settings persistence for a specific settings type
/// 
/// **LEGACY**: This is kept for backwards compatibility with the old storage system
/// where each settings type had its own file. New code should use the unified storage
/// system via `SettingsPlugin` or `SettingsStore`.
/// 
/// This will be removed in a future version.
#[allow(dead_code)]
#[deprecated(since = "0.2.0", note = "Use unified storage via SettingsPlugin or SettingsStore")]
#[derive(Resource, Clone)]
pub(crate) struct SettingsManager<T: Settings> {
    pub name: String,
    pub storage: SettingsStorage,
    pub _phantom: std::marker::PhantomData<T>,
}

/// System that saves settings when they are modified
/// 
/// **LEGACY**: This is kept for backwards compatibility with the old storage system.
/// New code should use `save_unified_settings_on_change` instead.
/// 
/// This will be removed in a future version.
#[allow(dead_code)]
#[deprecated(since = "0.2.0", note = "Use save_unified_settings_on_change")]
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

/// Shared resource for unified settings storage
#[derive(Resource, Clone)]
pub(crate) struct UnifiedSettingsManager {
    pub storage: UnifiedStorage,
    /// Shared map of all settings values (type_key -> JSON value)
    /// Using Arc<Mutex<>> to allow multiple systems to update the same map
    pub settings_map: Arc<Mutex<HashMap<String, Value>>>,
}

/// System that saves a specific settings type to the unified storage
pub(crate) fn save_unified_settings_on_change<T: Settings>(
    settings: Res<T>,
    manager: Res<UnifiedSettingsManager>,
) {
    if settings.is_changed() && !settings.is_added() {
        let type_key = get_type_key::<T>();
        
        // Compute delta (only changed fields)
        let delta = crate::unified_storage::compute_delta(&*settings);
        
        // Update the shared settings map
        let mut map = manager.settings_map.lock().unwrap();
        
        if let Some(delta_value) = delta {
            map.insert(type_key.clone(), delta_value);
        } else {
            // Settings equal defaults, remove from map
            map.remove(&type_key);
        }
        
        // Save all settings to disk
        if let Err(e) = manager.storage.save_all(&map) {
            error!("Failed to save unified settings: {}", e);
        } else {
            info!("Unified settings saved");
        }
    }
}

/// Get the type key for a settings type (lowercase type name)
pub(crate) fn get_type_key<T: Settings>() -> String {
    T::type_name().to_lowercase()
}
