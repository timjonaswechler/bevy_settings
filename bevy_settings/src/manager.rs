use crate::{compute_delta, merge_with_defaults, SettingsGroup};
use bevy::prelude::*;
use bevy_paths::PathRegistry;
use serde_json::Value;
use std::fs;

/// Errors that can occur during settings operations
#[derive(thiserror::Error, Debug)]
pub enum ManagerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path resolution failed: {0}")]
    Path(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
}

type Result<T> = std::result::Result<T, ManagerError>;

/// Loads a settings group from disk and applies it to the resource
pub fn load_settings_logic<T: SettingsGroup>(world: &mut World) -> Result<()> {
    let registry = world.resource::<PathRegistry>().clone();
    let current_resource = world.resource::<T>().clone();

    let path = registry.resolve(&current_resource);
    debug!("Loading settings from: {:?}", path);

    if !path.exists() {
        debug!("File not found, resetting to defaults with preserved params.");
        let mut new_settings = T::default();
        copy_params(&current_resource, &mut new_settings);
        world.insert_resource(new_settings);
        return Ok(());
    }

    let content = fs::read(&path)?;
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("json");
    let delta_value: Value = decode_content(&content, ext)?;

    let mut new_settings: T = merge_with_defaults(Some(&delta_value))
        .map_err(|e| ManagerError::Serialization(e.to_string()))?;

    copy_params(&current_resource, &mut new_settings);
    world.insert_resource(new_settings);

    info!("Loaded settings for {}", std::any::type_name::<T>());
    Ok(())
}

/// Saves a settings group to disk
pub fn save_settings_logic<T: SettingsGroup>(world: &mut World) -> Result<()> {
    let registry = world.resource::<PathRegistry>();
    let settings = world.resource::<T>();

    ensure_path_params_present(settings)?;

    let path = registry.resolve(settings);
    let delta = compute_delta(settings);

    let value_to_save = if let Some(mut val) = delta {
        if let Value::Object(ref mut map) = val {
            for param in T::path_params() {
                map.remove(*param);
            }
            if map.is_empty() {
                None
            } else {
                Some(val)
            }
        } else {
            Some(val)
        }
    } else {
        None
    };

    if let Some(root) = value_to_save {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("json");
        let content = encode_content(&root, ext)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, content)?;
        debug!("Saved settings to {:?}", path);
    } else {
        if path.exists() {
            fs::remove_file(&path)?;
            debug!("Deleted empty settings file at {:?}", path);
        }
    }

    Ok(())
}

fn ensure_path_params_present<T: SettingsGroup>(settings: &T) -> Result<()> {
    let params = T::path_params();
    if params.is_empty() {
        return Ok(());
    }

    let value =
        serde_json::to_value(settings).map_err(|e| ManagerError::Serialization(e.to_string()))?;

    let map = value
        .as_object()
        .ok_or_else(|| ManagerError::Path("settings structure is not an object".to_string()))?;

    for param in params {
        let Some(val) = map.get(*param) else {
            return Err(ManagerError::Path(format!("missing path param '{param}'")));
        };

        if is_empty_path_value(val) {
            return Err(ManagerError::Path(format!(
                "path param '{param}' must not be empty"
            )));
        }
    }

    Ok(())
}

fn is_empty_path_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => s.trim().is_empty(),
        _ => false,
    }
}

fn decode_content(content: &[u8], format: &str) -> Result<Value> {
    match format {
        "json" => {
            serde_json::from_slice(content).map_err(|e| ManagerError::Serialization(e.to_string()))
        }
        "toml" => {
            let s = std::str::from_utf8(content)
                .map_err(|e| ManagerError::Serialization(e.to_string()))?;
            toml::from_str(s).map_err(|e| ManagerError::Serialization(e.to_string()))
        }
        "ron" => {
            let s = std::str::from_utf8(content)
                .map_err(|e| ManagerError::Serialization(e.to_string()))?;
            ron::from_str(s).map_err(|e| ManagerError::Serialization(e.to_string()))
        }
        "bin" => {
            let config = bincode::config::standard();
            let (val, _): (Value, usize) = bincode::serde::decode_from_slice(content, config)
                .map_err(|e| ManagerError::Serialization(e.to_string()))?;
            Ok(val)
        }
        _ => Err(ManagerError::UnsupportedFormat(format.to_string())),
    }
}

fn encode_content(value: &Value, format: &str) -> Result<Vec<u8>> {
    match format {
        "json" => {
            serde_json::to_vec_pretty(value).map_err(|e| ManagerError::Serialization(e.to_string()))
        }
        "toml" => {
            let s = toml::to_string_pretty(value)
                .map_err(|e| ManagerError::Serialization(e.to_string()))?;
            Ok(s.into_bytes())
        }
        "ron" => {
            let s = ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default())
                .map_err(|e| ManagerError::Serialization(e.to_string()))?;
            Ok(s.into_bytes())
        }
        "bin" => {
            let config = bincode::config::standard();
            bincode::serde::encode_to_vec(value, config)
                .map_err(|e| ManagerError::Serialization(e.to_string()))
        }
        _ => Err(ManagerError::UnsupportedFormat(format.to_string())),
    }
}

/// Copies parameter fields from source to target using Serde.
/// This avoids complexity with Reflection (clone_value methods) and guarantees type safety via Serde.
fn copy_params<T: SettingsGroup>(source: &T, target: &mut T) {
    let params = T::path_params();
    if params.is_empty() {
        return;
    }

    // 1. Source -> Value
    let source_val = match serde_json::to_value(source) {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to serialize source for params copy: {}", e);
            return;
        }
    };

    // 2. Target -> Value
    let mut target_val = match serde_json::to_value(&*target) {
        Ok(v) => v,

        Err(e) => {
            warn!("Failed to serialize target for params copy: {}", e);
            return;
        }
    };

    // 3. Copy fields in JSON
    let mut modified = false;
    if let (Value::Object(s_map), Value::Object(ref mut t_map)) = (&source_val, &mut target_val) {
        for param in params {
            if let Some(val) = s_map.get(*param) {
                // Only insert if different to avoid unnecessary clone?
                // No, we must overwrite target defaults with source param.
                t_map.insert(param.to_string(), val.clone());
                modified = true;
            }
        }
    }

    if modified {
        // 4. Value -> Target
        match serde_json::from_value(target_val) {
            Ok(v) => *target = v,
            Err(e) => warn!("Failed to apply params via Serde: {}", e),
        }
    }
}
