use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

use crate::error::Result;

pub(crate) fn compute_delta<T>(settings: &T) -> Option<Value>
where
    T: Serialize + Default + PartialEq,
{
    let defaults = T::default();

    // If equal to defaults, no need to store
    if settings == &defaults {
        return None;
    }

    // Serialize both to JSON values
    // TODO: Handle serialization errors gracefully?
    let settings_value = serde_json::to_value(settings).ok()?;
    let defaults_value = serde_json::to_value(&defaults).ok()?;

    // Compute delta recursively
    compute_value_delta(&settings_value, &defaults_value)
}

/// Recursively compute delta between two JSON values
fn compute_value_delta(current: &Value, default: &Value) -> Option<Value> {
    match (current, default) {
        (Value::Object(curr_map), Value::Object(def_map)) => {
            let mut delta_map = Map::new();

            for (key, curr_val) in curr_map {
                if let Some(def_val) = def_map.get(key) {
                    // Key exists in both, check if different
                    if curr_val != def_val {
                        // Try to compute nested delta for objects
                        if let Some(nested_delta) = compute_value_delta(curr_val, def_val) {
                            delta_map.insert(key.clone(), nested_delta);
                        }
                    }
                } else {
                    // Key only in current, include it
                    delta_map.insert(key.clone(), curr_val.clone());
                }
            }

            if delta_map.is_empty() {
                None
            } else {
                Some(Value::Object(delta_map))
            }
        }
        _ => {
            // For non-object values, include if different
            if current != default {
                Some(current.clone())
            } else {
                None
            }
        }
    }
}

/// Merge delta with defaults to get complete settings
pub(crate) fn merge_with_defaults<T>(delta: Option<&Value>) -> Result<T>
where
    T: DeserializeOwned + Default + Serialize,
{
    let defaults = T::default();

    // If no delta, return defaults
    let Some(delta) = delta else {
        return Ok(defaults);
    };

    // Serialize defaults to JSON
    let mut defaults_value = serde_json::to_value(&defaults)?;

    // Merge delta into defaults
    merge_values(&mut defaults_value, delta);

    // Deserialize back to T
    let result: T = serde_json::from_value(defaults_value)?;
    Ok(result)
}

/// Recursively merge source into target
fn merge_values(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(target_map), Value::Object(source_map)) => {
            for (key, source_val) in source_map {
                if let Some(target_val) = target_map.get_mut(key) {
                    // Recursively merge nested objects
                    merge_values(target_val, source_val);
                } else {
                    // Key doesn't exist in target, add it
                    target_map.insert(key.clone(), source_val.clone());
                }
            }
        }
        (target, source) => {
            // Replace target with source
            *target = source.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
    struct TestSettings {
        value: i32,
        name: String,
        nested: NestedSettings,
    }

    #[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
    struct NestedSettings {
        enabled: bool,
        count: u32,
    }

    #[test]
    fn test_compute_delta_no_changes() {
        let settings = TestSettings::default();
        let delta = compute_delta(&settings);
        assert!(delta.is_none());
    }

    #[test]
    fn test_compute_delta_with_changes() {
        let mut settings = TestSettings::default();
        settings.value = 42;

        let delta = compute_delta(&settings);
        assert!(delta.is_some());

        let delta_value = delta.unwrap();
        assert!(delta_value.get("value").is_some());
        assert_eq!(delta_value.get("value").unwrap(), &Value::Number(42.into()));

        // Unchanged fields should not be in delta
        assert!(delta_value.get("name").is_none());
    }

    #[test]
    fn test_merge_with_defaults() {
        let mut delta_map = Map::new();
        delta_map.insert("value".to_string(), Value::Number(100.into()));
        let delta = Value::Object(delta_map);

        let result: TestSettings = merge_with_defaults(Some(&delta)).unwrap();
        assert_eq!(result.value, 100);
        assert_eq!(result.name, String::default()); // Should use default
    }
}
