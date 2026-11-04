use crate::{Error, Result};
use serde_json::Value;

pub struct SettingsService;

impl SettingsService {
    pub fn get(settings: &Value) -> Value {
        let mut shown = settings.clone();
        if let Some(secret) = shown.pointer_mut("/integration/client_secret")
            && let Some(raw) = secret.as_str()
        {
            let end = raw
                .chars()
                .rev()
                .take(4)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>();
            *secret = Value::String(format!("••••{end}"));
        }
        let effective = settings
            .get("preview_delay_s")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(15)
            .clamp(10, 30);
        shown["effective_preview_delay_s"] = Value::from(effective);
        shown
    }

    pub fn set(current: &Value, patch: &Value) -> Result<Value> {
        let mut next = current.clone();
        let changes = patch
            .as_object()
            .ok_or_else(|| Error::InvalidValue("settings patch".into()))?;
        for (key, value) in changes {
            if ![
                "document_id",
                "preview_recipe",
                "master_recipe",
                "output_format",
                "style_name",
                "export_folder",
                "preview_delay_s",
                "theme",
                "sku_prefix",
                "integration",
                "default_shape",
            ]
            .contains(&key.as_str())
            {
                return Err(Error::InvalidValue(key.clone()));
            }
            if key == "export_folder"
                && !value
                    .as_str()
                    .is_some_and(|s| std::path::Path::new(s).is_absolute())
            {
                return Err(Error::RelativePath);
            }
            if key == "document_id" {
                let id = value
                    .as_str()
                    .ok_or_else(|| Error::InvalidValue("document_id".into()))?;
                if !id.is_empty() && !std::path::Path::new(id).is_absolute() {
                    return Err(Error::RelativePath);
                }
                next[key] = if id.is_empty() {
                    Value::Null
                } else {
                    value.clone()
                };
                continue;
            }
            if key == "default_shape"
                && !value
                    .as_str()
                    .is_some_and(|s| crate::domain::composition::Shape::parse(s).is_some())
            {
                return Err(Error::InvalidValue("default_shape".into()));
            }
            // 10D MC-28: JPEG, 4:4:4 and sRGB are fixed; the long edge and quality are settings.
            if key == "output_format" && !valid_output_format(value) {
                return Err(Error::InvalidValue("output_format".into()));
            }
            if key == "master_recipe" && !value.as_str().is_some_and(|s| !s.is_empty()) {
                return Err(Error::InvalidValue("master_recipe".into()));
            }
            if key == "integration" && !value.is_null() {
                let mut integration = value.clone();
                if let Some(secret) = integration.get("client_secret").and_then(Value::as_str)
                    && current
                        .pointer("/integration/client_secret")
                        .and_then(Value::as_str)
                        .is_some_and(|stored| {
                            let tail = stored
                                .chars()
                                .rev()
                                .take(4)
                                .collect::<Vec<_>>()
                                .into_iter()
                                .rev()
                                .collect::<String>();
                            secret == format!("••••{tail}")
                        })
                {
                    let previous = current
                        .pointer("/integration/client_secret")
                        .cloned()
                        .ok_or_else(|| Error::InvalidValue("client_secret".into()))?;
                    integration["client_secret"] = previous;
                }
                next[key] = integration;
            } else {
                next[key] = value.clone();
            }
        }
        Ok(next)
    }
}

/// `{"long_edge": 512..=8192, "quality": 60..=100}`, both integers.
fn valid_output_format(value: &Value) -> bool {
    let in_range = |key: &str, range: std::ops::RangeInclusive<u64>| {
        value
            .get(key)
            .and_then(Value::as_u64)
            .is_some_and(|v| range.contains(&v))
    };
    value.is_object() && in_range("long_edge", 512..=8192) && in_range("quality", 60..=100)
}
