use psm_core::{Error, Result};
use psm_integrations::{App, AppHandles};
use std::sync::{Mutex, RwLock};

pub struct AppState {
    app: RwLock<App>,
    save: Mutex<()>,
}
impl AppState {
    pub fn new(app: App) -> Self {
        Self {
            app: RwLock::new(app),
            save: Mutex::new(()),
        }
    }
    pub fn settings_set(&self, patch: &serde_json::Value) -> Result<serde_json::Value> {
        let _save = self
            .save
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let context = self
            .app
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
        self.app
            .read()
            .map(|app| app.handles())
            .map_err(|e| Error::InvalidData(e.to_string()))
    }
    pub fn mutate<T>(&self, f: impl FnOnce(&mut App) -> Result<T>) -> Result<T> {
        let mut app = self
            .app
            .write()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        f(&mut app)
    }
}
