use crate::paths::AppPaths;
use psm_core::{
    Error, Result,
    domain::{camera::CameraSettings, output::OutputFormat},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::PathBuf,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub camera: CameraSettings,
    pub document_id: Option<String>,
    pub preview_recipe: String,
    pub master_recipe: String,
    /// 10D MC-28: the size and quality of every output (replaces the retired `web_recipe`, which
    /// is ignored on read and dropped on the next save).
    pub output_format: OutputFormat,
    pub style_name: String,
    pub default_shape: String,
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
            camera: CameraSettings::default(),
            document_id: None,
            preview_recipe: "PSM Preview".into(),
            master_recipe: "PSM Master".into(),
            output_format: OutputFormat::default(),
            style_name: String::new(),
            default_shape: "1:1".into(),
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
    fn from_legacy(raw: &[u8]) -> Result<Self> {
        let value: serde_json::Value =
            serde_json::from_slice(raw).map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut settings: Self =
            serde_json::from_value(value.clone()).map_err(|e| Error::InvalidData(e.to_string()))?;
        if value.get("default_shape").is_none()
            && let Some(legacy) = value.get("crop_on_arrival").and_then(|v| v.as_str())
        {
            settings.default_shape = legacy_default_shape(legacy).into();
        }
        Ok(settings)
    }
    pub fn default_document_id(&self, paths: &AppPaths) -> PathBuf {
        paths.home.join("Pictures/Session Manager")
    }
    pub fn effective_document_id(&self, paths: &AppPaths) -> PathBuf {
        self.document_id
            .as_deref()
            .filter(|id| !id.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.default_document_id(paths))
    }
    pub fn open_lenient(paths: &AppPaths) -> Result<(Self, Option<String>)> {
        let path = paths.settings();
        if !path.exists() {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)?;
            serde_json::to_writer_pretty(&mut file, &Self::default())
                .map_err(|e| Error::InvalidData(e.to_string()))?;
        }
        let raw = fs::read(&path)?;
        match Self::from_legacy(&raw) {
            Ok(settings) => Ok((settings, None)),
            Err(error) => {
                tracing::warn!(%error, "settings.json invalid; using defaults");
                Ok((Self::default(), Some(error.to_string())))
            }
        }
    }
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
        let settings = Self::from_legacy(&raw)?;
        Ok(settings)
    }

    pub fn set(paths: &AppPaths, settings: &Self) -> Result<()> {
        let path = paths.settings();
        let temp = path.with_extension(format!("{}.tmp", std::process::id()));
        let result = (|| -> Result<()> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temp)?;
            serde_json::to_writer_pretty(&mut file, settings)
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            file.flush()?;
            file.sync_all()?;
            fs::rename(&temp, &path)?;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result
    }
}

pub fn legacy_default_shape(value: &str) -> &'static str {
    match value {
        "off" => "original",
        "4x5" => "4:5",
        "2x3" => "2:3",
        "16x9" => "16:9",
        _ => "1:1",
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
