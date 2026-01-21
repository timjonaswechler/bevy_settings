use crate::{
    delta::{compute_delta, merge_with_defaults},
    SettingsGroupTrait,
};
use bevy::prelude::*;
use serde_json::Value;
use std::fs;

#[derive(thiserror::Error, Debug)]
pub enum ManagerError {
    /// Error during file I/O operations (e.g., reading or writing settings files).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Path resolution failed (e.g., missing path parameters or invalid paths).
    #[error("Path resolution failed: {0}")]
    Path(String),

    /// Error during serialization or deserialization.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// The file format is not supported.
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
}

type Result<T> = std::result::Result<T, ManagerError>;

/// Loads a settings group from disk and applies it to the ECS resource.
///
/// If the settings file does not exist, the resource is initialized with default values
/// while preserving path parameters (e.g., `{id}` in `"saves/{id}.json"`).
///
/// # Arguments
/// - `world`: The Bevy ECS world containing the settings resource.
///
/// # Returns
/// - `Ok(())`: Settings were loaded successfully or defaults were applied.
/// - `Err(ManagerError)`: An error occurred during loading (e.g., I/O, serialization, or path resolution).
///
/// # Type Parameters
/// - `T`: The settings group type (must implement `SettingsGroupTrait`).
pub(crate) fn load_settings_logic<T: SettingsGroupTrait>(world: &mut World) -> Result<()> {
    let current_resource = world.resource::<T>().clone();
    let path = current_resource
        .resolve()
        .map_err(|e| ManagerError::Path(e.to_string()))?;
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

/// Saves a settings group to disk using delta encoding.
///
/// Only fields that differ from their default values are saved. If the delta is empty,
/// the settings file is deleted (if it exists). Path parameters (e.g., `{id}`) are excluded
/// from the serialized payload.
///
/// # Arguments
/// - `world`: The Bevy ECS world containing the settings resource.
///
/// # Returns
/// - `Ok(())`: Settings were saved successfully or the file was deleted (if delta was empty).
/// - `Err(ManagerError)`: An error occurred during saving (e.g., I/O, serialization, or path resolution).
///
/// # Type Parameters
/// - `T`: The settings group type (must implement `SettingsGroupTrait`).
pub(crate) fn save_settings_logic<T: SettingsGroupTrait>(world: &mut World) -> Result<()> {
    let settings = world.resource::<T>();

    ensure_path_params_present(settings)?;

    let path = settings
        .resolve()
        .map_err(|e| ManagerError::Path(e.to_string()))?;
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

/// Validates that all path parameters (e.g., `{id}`) are present and non-empty.
///
/// # Arguments
/// - `settings`: The settings group to validate.
///
/// # Returns
/// - `Ok(())`: All path parameters are present and non-empty.
/// - `Err(ManagerError::Path)`: A path parameter is missing or empty.
///
/// # Type Parameters
/// - `T`: The settings group type (must implement `SettingsGroupTrait`).
fn ensure_path_params_present<T: SettingsGroupTrait>(settings: &T) -> Result<()> {
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

/// Decodes settings content from a byte slice into a `serde_json::Value`.
///
/// Supported formats: `json`, `toml`, `ron`, `bin`.
///
/// # Arguments
/// - `content`: The raw bytes of the settings file.
/// - `format`: The file format (e.g., `"json"`, `"toml"`).
///
/// # Returns
/// - `Ok(Value)`: The decoded settings as a `serde_json::Value`.
/// - `Err(ManagerError::UnsupportedFormat)`: The format is not supported.
/// - `Err(ManagerError::Serialization)`: Decoding failed (e.g., invalid content).
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

/// Encodes a `serde_json::Value` into a byte vector for saving to disk.
///
/// Supported formats: `json`, `toml`, `ron`, `bin`.
///
/// # Arguments
/// - `value`: The settings data to encode.
/// - `format`: The file format (e.g., `"json"`, `"toml"`).
///
/// # Returns
/// - `Ok(Vec<u8>)`: The encoded settings as bytes.
/// - `Err(ManagerError::UnsupportedFormat)`: The format is not supported.
/// - `Err(ManagerError::Serialization)`: Encoding failed.
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

/// Copies path parameter fields (e.g., `{id}`) from the source settings to the target.
///
/// This ensures that path parameters are preserved when reloading settings or applying defaults.
/// Only fields listed in `T::path_params()` are copied.
fn copy_params<T: SettingsGroupTrait>(source: &T, target: &mut T) {
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
    if let (Value::Object(s_map), Value::Object(t_map)) = (&source_val, &mut target_val) {
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
