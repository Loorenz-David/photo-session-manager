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
                "web_recipe",
                "style_name",
                "export_folder",
                "preview_delay_s",
                "theme",
                "sku_prefix",
                "integration",
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
            if key == "integration" && !value.is_null() {
                let mut integration = value.clone();
                if let Some(secret) = integration.get("client_secret").and_then(Value::as_str)
                    && secret.starts_with("••••")
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
