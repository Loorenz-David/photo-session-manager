use crate::clock::ManualClock;
use crate::fake_capture_one::FakeCaptureOneScripting;
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
use tempfile::TempDir;

pub struct TestApp {
    pub dir: TempDir,
    pub app: App,
    pub clock: Arc<ManualClock>,
    pub fake: Arc<FakeCaptureOneScripting>,
    pub events: Arc<CollectingEventSink>,
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
        Self::create(false)
    }
    pub fn with_fake_document() -> Result<Self> {
        Self::create(true)
    }
    fn create(configure_document: bool) -> Result<Self> {
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
            std::fs::write(
                paths.settings(),
                serde_json::to_vec(&settings)
                    .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?,
            )?;
        }
        let fake = Arc::new(FakeCaptureOneScripting::new(document_id, capture_root));
        let clock = Arc::new(ManualClock::new(0));
        let app = App::open_with(paths, fake.clone(), clock.clone(), false)?;
        let events = Arc::new(CollectingEventSink::default());
        app.events.set_target(events.clone());
        Ok(Self {
            dir,
            app,
            clock,
            fake,
            events,
            next_hook: AtomicU64::new(1),
        })
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
        self.app.jobs.tick(self.clock.now_ms())
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
