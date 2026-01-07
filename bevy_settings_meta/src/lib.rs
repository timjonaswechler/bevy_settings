use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LocalizedText {
    /// e.g. "settings.server.port.label"
    pub key: String,
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
pub enum UiHint {
    Slider,
    NumberInput,
    Dropdown,
    Toggle,
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

/// Fehler, die beim set_by_key auftreten können
#[derive(Debug)]
pub enum SettingsError {
    UnknownKey,
    TypeMismatch,
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

impl std::error::Error for SettingsError {}

// Test: integer valid/invalid, numeric string parsing, float, text length, enum by label and by value.
