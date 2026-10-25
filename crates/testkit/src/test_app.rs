use crate::clock::ManualClock;
use psm_core::Result;
use psm_core::{
    ports::store::Store,
    services::{items::ItemsService, sessions::SessionsService},
};
use psm_infra::paths::AppPaths;
use psm_integrations::App;
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::TempDir;

pub struct TestApp {
    pub dir: TempDir,
    pub app: App,
    pub clock: ManualClock,
    next_hook: AtomicU64,
}

impl TestApp {
    pub fn items(&self) -> ItemsService<'_> {
        ItemsService::new(self.app.store.as_ref(), &self.clock)
    }
    pub fn sessions(&self) -> SessionsService<'_> {
        SessionsService::new(self.app.store.as_ref(), &self.clock)
    }
    pub fn store(&self) -> &dyn Store {
        self.app.store.as_ref()
    }
    pub fn new() -> Result<Self> {
        let dir = tempfile::tempdir()?;
        let app = App::open(AppPaths::for_dir(dir.path()))?;
        Ok(Self {
            dir,
            app,
            clock: ManualClock::new(0),
            next_hook: AtomicU64::new(1),
        })
    }
    pub fn deliver_hook(&self, json: &str) -> Result<()> {
        let id = self.next_hook.fetch_add(1, Ordering::Relaxed);
        self.app
            .spool
            .write_for_test(&format!("{id:016}-test.json"), json)?;
        self.app.spool.drain(self.app.store.as_ref(), &self.clock)
    }
}
