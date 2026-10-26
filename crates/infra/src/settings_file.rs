use crate::paths::AppPaths;
use psm_core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub document_id: Option<String>,
    pub preview_recipe: String,
    pub web_recipe: String,
    pub style_name: String,
    pub export_folder: String,
    #[serde(deserialize_with = "deserialize_preview_delay")]
    pub preview_delay_s: u32,
    pub theme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku_prefix: Option<String>,
    pub integration: Option<serde_json::Value>,
}

fn deserialize_preview_delay<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<u32, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Number(n) => {
            n.as_u64().and_then(|v| u32::try_from(v).ok()).unwrap_or(15)
        }
        serde_json::Value::String(s) => s.parse().unwrap_or(15),
        _ => 15,
    })
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            document_id: None,
            preview_recipe: "PSM Preview".into(),
            web_recipe: "PSM Web".into(),
            style_name: "PSM House".into(),
            export_folder: PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join("Pictures/Session Manager Export")
                .to_string_lossy()
                .into_owned(),
            preview_delay_s: 15,
            theme: "light".into(),
            sku_prefix: None,
            integration: None,
        }
    }
}

impl Settings {
    pub fn open(paths: &AppPaths) -> Result<Self> {
        let path = paths.settings();
        if !path.exists() {
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)?;
            f.write_all(
                serde_json::to_string_pretty(&Self::default())
                    .map_err(|e| Error::InvalidData(e.to_string()))?
                    .as_bytes(),
            )?;
        }
        let raw = fs::read(&path)?;
        let mut settings: Self =
            serde_json::from_slice(&raw).map_err(|e| Error::InvalidData(e.to_string()))?;
        settings.preview_delay_s = settings.preview_delay_s.clamp(10, 30);
        Ok(settings)
    }
}

/// The typed `integration` block (09A MC-23/MC-24):
/// `{"kind":"shopify","shop_domain","api_version","client_id","client_secret","condition_photos_key"}`.
/// An absent `api_version` reads as `"2026-10"`; an absent `condition_photos_key` as
/// `"custom.damage_report"`. `Debug` never shows the client secret.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationSettings {
    pub kind: String,
    pub shop_domain: String,
    #[serde(default = "default_api_version")]
    pub api_version: String,
    pub client_id: String,
    pub client_secret: String,
    #[serde(default = "default_condition_photos_key")]
    pub condition_photos_key: String,
}

fn default_api_version() -> String {
    "2026-10".into()
}

fn default_condition_photos_key() -> String {
    "custom.damage_report".into()
}

impl std::fmt::Debug for IntegrationSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntegrationSettings")
            .field("kind", &self.kind)
            .field("shop_domain", &self.shop_domain)
            .field("api_version", &self.api_version)
            .field("client_id", &self.client_id)
            .field("client_secret", &"<redacted>")
            .field("condition_photos_key", &self.condition_photos_key)
            .finish()
    }
}

impl Settings {
    /// The typed integration block; `None` when the key is absent or `null`.
    pub fn integration_settings(&self) -> Result<Option<IntegrationSettings>> {
        match &self.integration {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(value) => serde_json::from_value(value.clone())
                .map(Some)
                .map_err(|e| Error::InvalidData(format!("integration settings: {e}"))),
        }
    }
}
