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
    pub preview_delay_s: u32,
    pub theme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku_prefix: Option<String>,
    pub integration: Option<serde_json::Value>,
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
