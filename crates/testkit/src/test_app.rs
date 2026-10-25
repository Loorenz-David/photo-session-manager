use tempfile::TempDir;
use psm_core::{Result, ports::clock::SystemClock};
use psm_infra::paths::AppPaths;
use psm_integrations::App;

pub struct TestApp { pub dir: TempDir, pub app: App }

impl TestApp {
    pub fn new() -> Result<Self> {
        let dir = tempfile::tempdir()?;
        let app = App::open(AppPaths::for_dir(dir.path()))?;
        Ok(Self { dir, app })
    }
    pub fn deliver_hook(&self, json: &str) -> Result<()> {
        self.app.spool.write_for_test("0001-test.json", json)?;
        self.app.spool.drain(self.app.store.as_ref(), &SystemClock)
    }
}

