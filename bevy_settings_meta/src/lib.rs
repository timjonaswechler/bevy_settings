use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
/// A localized text entry for settings metadata.
///
/// Used to provide human-readable labels, descriptions, and tooltips for settings.
/// Supports fallback values for cases where the localization key is not available.
///
/// # Examples
/// ```
/// use bevy_settings_meta::LocalizedText;
///
/// let label = LocalizedText {
///     key: "settings.server.port.label".to_string(),
///     fallback: Some("Port".to_string()),
/// };
/// ```
pub struct LocalizedText {
    /// The localization key for this text entry.
    ///
    /// Example: `"settings.server.port.label"`.
    pub key: String,
    /// A fallback value to use if the localization key is not found.
    ///
    /// If `None`, the key itself may be displayed as a fallback.
    pub fallback: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SettingKind {
    Integer {
        min: Option<i64>,
        max: Option<i64>,
        step: Option<i64>,
    },
    Float {
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
    },
    Boolean,
    Text {
        multiline: bool,
        max_len: Option<u32>,
    },
    Enum {
        variants: Vec<(String, Value)>,
    },
}

impl SettingKind {
    /// Prüft, ob `val` zum Typ passt (optional: strings parsen).
    pub fn validate_value(&self, val: &serde_json::Value) -> Result<(), SettingsError> {
        match self {
            SettingKind::Integer { min, max, .. } => {
                let n_opt = val
                    .as_i64()
                    .or_else(|| val.as_u64().map(|u| u as i64))
                    .or_else(|| val.as_str().and_then(|s| s.parse::<i64>().ok()));
                let n = n_opt.ok_or(SettingsError::TypeMismatch)?;
                if let Some(min) = min {
                    if n < *min {
                        return Err(SettingsError::ValidationFailed(format!(
                            "{} < min {}",
                            n, min
                        )));
                    }
                }
                if let Some(max) = max {
                    if n > *max {
                        return Err(SettingsError::ValidationFailed(format!(
                            "{} > max {}",
                            n, max
                        )));
                    }
                }
                Ok(())
            }
            SettingKind::Float { min, max, .. } => {
                let f_opt = val
                    .as_f64()
                    .or_else(|| val.as_str().and_then(|s| s.parse::<f64>().ok()));
                let f = f_opt.ok_or(SettingsError::TypeMismatch)?;
                if let Some(min) = min {
                    if f < *min {
                        return Err(SettingsError::ValidationFailed(format!(
                            "{} < min {}",
                            f, min
                        )));
                    }
                }
                if let Some(max) = max {
                    if f > *max {
                        return Err(SettingsError::ValidationFailed(format!(
                            "{} > max {}",
                            f, max
                        )));
                    }
                }
                Ok(())
            }
            SettingKind::Boolean => {
                if val.is_boolean()
                    || (val.is_string()
                        && matches!(
                            val.as_str().unwrap().to_lowercase().as_str(),
                            "true" | "false"
                        ))
                {
                    Ok(())
                } else {
                    Err(SettingsError::TypeMismatch)
                }
            }
            SettingKind::Text { max_len, .. } => {
                let s = val.as_str().ok_or(SettingsError::TypeMismatch)?;
                if let Some(max) = max_len {
                    if s.len() > *max as usize {
                        return Err(SettingsError::ValidationFailed("text too long".into()));
                    }
                }
                Ok(())
            }
            SettingKind::Enum { variants } => {
                // akzeptiere Entweder: Value ungleich String -> direkte Vergleich mit variant.value
                if variants.iter().any(|(_, v)| v == val) {
                    return Ok(());
                }
                // oder string-label: wenn val String und passt zu einem label
                if let Some(s) = val.as_str() {
                    if variants.iter().any(|(label, _)| label == s) {
                        return Ok(());
                    }
                }
                Err(SettingsError::ValidationFailed(
                    "invalid enum variant".into(),
                ))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// UI hints for rendering settings controls.
///
/// Used to guide the UI layer on how to render a setting (e.g., as a slider, dropdown, or toggle).
/// This ensures a consistent and user-friendly experience across different platforms.
///
/// # Examples
/// ```
/// use bevy_settings_meta::UiHint;
///
/// // Render a boolean setting as a toggle switch.
/// let hint = UiHint::Toggle;
/// ```
pub enum UiHint {
    /// A slider control for numeric settings (e.g., volume or brightness).
    Slider,
    /// A numeric input field for precise values.
    NumberInput,
    /// A dropdown menu for enumerated settings.
    Dropdown,
    /// A toggle switch for boolean settings.
    Toggle,
    /// A password field for sensitive text input.
    Password,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SettingDescriptor {
    pub key: String,
    pub label: LocalizedText,
    pub description: Option<LocalizedText>,
    pub kind: SettingKind,
    pub default: Value,
    #[serde(default)]
    pub read_only: bool,
    pub group: Option<String>,
    pub order: Option<i32>,
    pub ui_hint: Option<UiHint>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub meta: HashMap<String, Value>,
}

/// Errors that can occur during setting validation or manipulation.
///
/// Used to provide detailed feedback when a setting value is invalid or inaccessible.
///
/// # Examples
/// ```
/// use bevy_settings_meta::SettingsError;
/// use serde_json::json;
///
/// // Simulate a type mismatch error.
/// let error = SettingsError::TypeMismatch;
/// assert_eq!(error.to_string(), "type mismatch");
/// ```
#[derive(Debug, PartialEq)]
pub enum SettingsError {
    /// The requested setting key does not exist.
    UnknownKey,
    /// The provided value does not match the expected type.
    TypeMismatch,
    /// The value violates constraints (e.g., out of bounds or invalid format).
    ValidationFailed(String),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::UnknownKey => write!(f, "unknown key"),
            SettingsError::TypeMismatch => write!(f, "type mismatch"),
            SettingsError::ValidationFailed(s) => write!(f, "validation failed: {}", s),
        }
    }
}

impl std::error::Error for SettingsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_settings_error_display() {
        assert_eq!(SettingsError::UnknownKey.to_string(), "unknown key");
        assert_eq!(SettingsError::TypeMismatch.to_string(), "type mismatch");
        assert_eq!(
            SettingsError::ValidationFailed("out of bounds".to_string()).to_string(),
            "validation failed: out of bounds"
        );
    }
}

// Test: integer valid/invalid, numeric string parsing, float, text length, enum by label and by value.
