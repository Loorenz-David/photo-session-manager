use std::{fs, path::Path, sync::Mutex};
use serde::{Deserialize, Serialize};
use psm_core::{Error, Result, ports::{clock::Clock, store::Store}};
use crate::paths::AppPaths;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "hook", rename_all = "snake_case")]
pub enum HookPayload {
    CaptureDone { argv: Vec<String>, #[serde(default)] last_captured_file: Option<String> },
    ProcessingDone { job: String, raw: String, outputs: Vec<String> },
    VariantAdjusted { variant_id: String, raw: String },
}

pub fn parse_spool_file(bytes: &[u8]) -> Result<HookPayload> {
    serde_json::from_slice(bytes).map_err(|e| Error::InvalidData(e.to_string()))
}

pub struct Spool { paths: AppPaths, drain_lock: Mutex<()> }

impl Spool {
    pub fn new(paths: AppPaths) -> Self { Self { paths, drain_lock: Mutex::new(()) } }

    pub fn drain(&self, store: &dyn Store, clock: &dyn Clock) -> Result<()> {
        let _guard = self.drain_lock.lock().map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut files = fs::read_dir(self.paths.spool())?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|s| s == "json"))
            .collect::<Vec<_>>();
        files.sort();
        for path in files {
            let bytes = fs::read(&path)?;
            match parse_spool_file(&bytes) {
                Ok(payload) => {
                    let content = String::from_utf8(bytes).map_err(|e| Error::InvalidData(e.to_string()))?;
                    let hook = match payload { HookPayload::CaptureDone { .. } => "capture_done", HookPayload::ProcessingDone { .. } => "processing_done", HookPayload::VariantAdjusted { .. } => "variant_adjusted" };
                    store.record_hook(hook, &content, clock.now_ms())?;
                    fs::remove_file(path)?;
                }
                Err(_) => {
                    let dest = self.paths.spool().join("bad").join(path.file_name().unwrap_or_default());
                    fs::rename(path, dest)?;
                }
            }
        }
        Ok(())
    }

    pub fn write_for_test(&self, name: &str, content: &str) -> Result<()> {
        fs::write(self.paths.spool().join(Path::new(name)), content)?;
        Ok(())
    }
}

