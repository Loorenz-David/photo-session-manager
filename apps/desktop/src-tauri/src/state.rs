use psm_core::{Error, Result};
use psm_integrations::{App, AppHandles};
use std::sync::RwLock;

pub struct AppState(pub RwLock<App>);
impl AppState {
    pub fn settings_set(&self, patch: &serde_json::Value) -> Result<serde_json::Value> {
        let context = self
            .0
            .read()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .settings_set_context();
        let prepared = context.prepare(patch)?;
        let (value, old) = self.mutate(|app| app.apply_settings(prepared))?;
        if let Some(mut old) = old {
            old.shutdown();
        }
        Ok(value)
    }
    pub fn handles(&self) -> Result<AppHandles> {
        self.0
            .read()
            .map(|app| app.handles())
            .map_err(|e| Error::InvalidData(e.to_string()))
    }
    pub fn mutate<T>(&self, f: impl FnOnce(&mut App) -> Result<T>) -> Result<T> {
        let mut app = self
            .0
            .write()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        f(&mut app)
    }
}
