use crate::clock::ManualClock;
use crate::fake_capture_one::FakeCaptureOneScripting;
use crate::fake_commerce::FakeCommerce;
use psm_core::Result;
use psm_core::{
    domain::photo::{NewPhoto, Photo, PhotoSource},
    events::AppEvent,
    ports::{clock::Clock, events::EventSink, store::Store},
    services::{items::ItemsService, review::ReviewService, sessions::SessionsService},
};
use psm_infra::paths::AppPaths;
use psm_integrations::App;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};
use tempfile::TempDir;

pub struct TestApp {
    pub dir: TempDir,
    pub app: App,
    pub clock: Arc<ManualClock>,
    pub fake: Arc<FakeCaptureOneScripting>,
    pub events: Arc<CollectingEventSink>,
    pub commerce: Option<Arc<FakeCommerce>>,
    next_hook: AtomicU64,
}

#[derive(Default)]
pub struct CollectingEventSink(pub std::sync::Mutex<Vec<AppEvent>>);
impl EventSink for CollectingEventSink {
    fn emit(&self, event: AppEvent) {
        self.0.lock().unwrap().push(event);
    }
}

impl TestApp {
    pub fn items(&self) -> ItemsService<'_> {
        ItemsService::new(self.app.store.as_ref(), self.clock.as_ref())
    }
    pub fn sessions(&self) -> SessionsService<'_> {
        SessionsService::new(self.app.store.as_ref(), self.clock.as_ref())
            .with_editing(self.app.editing.as_ref(), self.app.health.as_ref())
    }
    pub fn review(&self) -> ReviewService<'_> {
        ReviewService::new(self.app.store.as_ref(), self.clock.as_ref())
    }
    pub fn add_photo(&self, session_id: i64, raw: &str) -> Result<Photo> {
        let path = self.dir.path().join(raw);
        std::fs::write(&path, raw.as_bytes())?;
        self.app.store.photo_insert_or_get(NewPhoto {
            session_id,
            raw_path: path.to_string_lossy().into_owned(),
            raw_hash: Some(format!("hash-{raw}")),
            captured_at: Some(self.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: self.clock.now_ms(),
        })
    }
    pub fn mark_published(&self, session_id: i64) -> Result<()> {
        let conn = rusqlite::Connection::open(self.app.paths.db())
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        conn.execute(
            "UPDATE sessions SET published_at=?1,published_seq=change_seq WHERE id=?2",
            rusqlite::params![self.clock.now_ms(), session_id],
        )
        .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        Ok(())
    }
    pub fn store(&self) -> &dyn Store {
        self.app.store.as_ref()
    }
    pub fn new() -> Result<Self> {
        Self::create(false, false, "custom.damage_report")
    }
    pub fn with_fake_document() -> Result<Self> {
        Self::create(true, false, "custom.damage_report")
    }
    pub fn with_fake_shop() -> Result<Self> {
        Self::create(true, true, "custom.damage_report")
    }
    pub fn with_fake_shop_key(key: &str) -> Result<Self> {
        Self::create(true, true, key)
    }
    fn create(configure_document: bool, configure_shop: bool, condition_key: &str) -> Result<Self> {
        let dir = tempfile::tempdir()?;
        let paths = AppPaths::for_dir(dir.path());
        paths.create()?;
        let document_id = dir
            .path()
            .join("Capture One Session")
            .to_string_lossy()
            .into_owned();
        let capture_root = dir.path().join("Capture").to_string_lossy().into_owned();
        std::fs::create_dir_all(&document_id)?;
        std::fs::create_dir_all(&capture_root)?;
        let mut settings = psm_infra::settings_file::Settings::open(&paths)?;
        if configure_document {
            settings.document_id = Some(document_id.clone());
            // Exports stay inside the test's temporary directory, never under ~/Pictures.
            settings.export_folder = dir.path().join("Export").to_string_lossy().into_owned();
            std::fs::write(
                paths.settings(),
                serde_json::to_vec(&settings)
                    .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?,
            )?;
        }
        let fake = Arc::new(FakeCaptureOneScripting::new(document_id, capture_root));
        let clock = Arc::new(ManualClock::new(0));
        let commerce = configure_shop.then(|| {
            Arc::new(FakeCommerce::with_condition_key(
                clock.clone(),
                condition_key,
            ))
        });
        let app = App::open_with_adapter(
            paths,
            fake.clone(),
            clock.clone(),
            false,
            commerce
                .as_ref()
                .map(|c| c.clone() as Arc<dyn psm_core::ports::commerce::CommerceAdapter>),
        )?;
        let events = Arc::new(CollectingEventSink::default());
        app.events.set_target(events.clone());
        Ok(Self {
            dir,
            app,
            clock,
            fake,
            events,
            commerce,
            next_hook: AtomicU64::new(1),
        })
    }
    pub fn fake_shop(&self) -> &FakeCommerce {
        self.commerce.as_deref().expect("fake commerce configured")
    }
    pub fn kill_and_restart(&mut self) -> Result<()> {
        self.restart(false)
    }
    pub fn restart_with_startup(&mut self) -> Result<()> {
        self.restart(true)
    }
    fn restart(&mut self, run_startup: bool) -> Result<()> {
        let paths = self.app.paths.clone();
        let next = App::open_with_adapter(
            paths,
            self.fake.clone(),
            self.clock.clone(),
            run_startup,
            self.commerce
                .as_ref()
                .map(|c| c.clone() as Arc<dyn psm_core::ports::commerce::CommerceAdapter>),
        )?;
        let old = std::mem::replace(&mut self.app, next);
        drop(old);
        self.app.events.set_target(self.events.clone());
        Ok(())
    }
    pub fn deliver_hook(&self, json: &str) -> Result<()> {
        let id = self.next_hook.fetch_add(1, Ordering::Relaxed);
        self.app
            .spool
            .write_for_test(&format!("{id:016}-test.json"), json)?;
        self.app
            .spool
            .drain(self.app.store.as_ref(), self.clock.as_ref())
    }
    pub fn fake_c1(&self) -> &FakeCaptureOneScripting {
        &self.fake
    }
    pub fn tick(&self, ms: i64) -> Result<usize> {
        self.clock.advance(ms);
        self.app.arrival.tick()?;
        let count = self.app.jobs.tick(self.clock.now_ms())?;
        let count = count + self.app.publish_jobs.tick(self.clock.now_ms())?;
        self.app.previews.tick()?;
        Ok(count)
    }
    pub fn focus(&self) -> Result<()> {
        self.app.previews.on_focus()
    }
    pub fn complete_render(&self, job_id: &str) -> Result<PathBuf> {
        let entry = self
            .app
            .editing
            .render_completion(job_id)
            .ok_or_else(|| psm_core::Error::NotFound(job_id.into()))?;
        let output = self.dir.path().join("Output").join(format!(
            "{}.jpg",
            Path::new(&entry.raw)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
        ));
        std::fs::create_dir_all(output.parent().unwrap())?;
        std::fs::write(&output, FakeCaptureOneScripting::output_bytes(job_id))?;
        std::fs::File::options()
            .write(true)
            .open(&output)?
            .set_times(std::fs::FileTimes::new().set_modified(
                std::time::UNIX_EPOCH
                    + std::time::Duration::from_millis(self.clock.now_ms().max(0) as u64),
            ))?;
        self.deliver_hook(&serde_json::json!({"hook":"processing_done","job":job_id,"raw":entry.raw,"outputs":[output]}).to_string())?;
        Ok(output)
    }
    /// Completes every render still in flight (in issue order), including renders the adapter
    /// queued behind a busy base name and issued as earlier ones finished. Returns the job ids.
    pub fn complete_all_renders(&self) -> Result<Vec<String>> {
        let mut completed = Vec::new();
        loop {
            let open: Vec<String> = self
                .fake
                .state
                .lock()
                .unwrap()
                .issued
                .iter()
                .map(|(job, _, _)| job.clone())
                .filter(|job| self.app.editing.render_completion(job).is_some())
                .collect();
            if open.is_empty() {
                return Ok(completed);
            }
            for job in open {
                self.complete_render(&job)?;
                completed.push(job);
            }
        }
    }
    /// `<export folder>` from the settings of this app (inside the test's temporary directory).
    pub fn export_root(&self) -> PathBuf {
        PathBuf::from(&self.app.settings.export_folder)
    }
    pub fn write_sidecar(&self, raw: &str, mtime: SystemTime) -> Result<PathBuf> {
        let raw = Path::new(raw);
        let folder = raw.parent().unwrap().join("CaptureOne/Settings1680");
        std::fs::create_dir_all(&folder)?;
        let path = folder.join(format!(
            "{}.cos",
            raw.file_name().unwrap().to_string_lossy()
        ));
        std::fs::write(&path, b"settings")?;
        std::fs::File::options()
            .write(true)
            .open(&path)?
            .set_times(std::fs::FileTimes::new().set_modified(mtime))?;
        Ok(path)
    }
    pub fn watcher_event(&self, path: &std::path::Path) -> Result<()> {
        self.app.arrival.watcher_event(path)
    }
    pub fn scan(&self) -> Result<()> {
        self.app.arrival.scan()
    }
}

impl TestApp {
    pub fn insert_publish_item(
        &self,
        session_id: i64,
        step: &str,
        confirmed_at: i64,
        failed_step: Option<&str>,
    ) -> Result<()> {
        let conn = rusqlite::Connection::open(self.app.paths.db())
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        conn.execute("INSERT INTO publish_runs(integration,created_at,confirmed_at,confirmed_summary) VALUES ('test',?1,?1,'test')", [confirmed_at])
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        let run_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO publish_items(run_id,session_id,create_item,step,failed_step) VALUES (?1,?2,0,?3,?4)", rusqlite::params![run_id,session_id,step,failed_step])
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        Ok(())
    }
}

impl TestApp {
    /// Inserts a `publish_runs` row committed with `confirmed_at` = now and returns the mutation
    /// token built from it by the one public constructor (07A MC-10).
    pub fn confirmed_run(&self) -> Result<psm_core::domain::publish::ConfirmedRun> {
        let conn = rusqlite::Connection::open(self.app.paths.db())
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        conn.execute(
            "INSERT INTO publish_runs(integration,created_at,confirmed_at,confirmed_summary) VALUES ('shopify',?1,?1,'test run')",
            [self.clock.now_ms()],
        )
        .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        psm_core::domain::publish::ConfirmedRun::from_persisted(
            self.store(),
            conn.last_insert_rowid(),
        )
    }
}
