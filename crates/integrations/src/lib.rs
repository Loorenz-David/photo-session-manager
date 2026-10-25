pub mod camera;
pub mod editing;

use camera::via_capture_one::CameraViaCaptureOne;
use editing::capture_one::{
    adapter::CaptureOneAdapter, jxa::JxaScripting, mirror::MirrorTag,
    scripting::CaptureOneScripting,
};
use psm_core::{
    Result,
    ports::{
        camera::CameraProvider,
        clock::{Clock, SystemClock},
        editing::EditingProvider,
        store::Store,
    },
    services::{camera::CameraService, editing_startup::EditingStartup, health::HealthService},
};
use psm_infra::{
    jobs::JobRunner, nudge::NudgeServer, paths::AppPaths, settings_file::Settings, spool::Spool,
    sqlite::store::SqliteStore, watcher::FsWatcher,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

pub struct App {
    pub paths: AppPaths,
    pub settings: Settings,
    pub store: Arc<SqliteStore>,
    pub spool: Arc<Spool>,
    pub nudge: NudgeServer,
    pub hooks_compiled: bool,
    pub health: Arc<HealthService>,
    pub editing: Arc<dyn EditingProvider>,
    pub camera: Arc<dyn CameraProvider>,
    pub jobs: Arc<JobRunner>,
    pub clock: Arc<dyn Clock>,
    stop: Arc<AtomicBool>,
    background: Option<thread::JoinHandle<()>>,
}

impl App {
    pub fn open(paths: AppPaths) -> Result<Self> {
        paths.create()?;
        let scripting = Arc::new(JxaScripting::new(&paths)?);
        Self::open_with(paths, scripting, Arc::new(SystemClock), true)
    }

    pub fn open_with(
        paths: AppPaths,
        scripting: Arc<dyn CaptureOneScripting>,
        clock: Arc<dyn Clock>,
        run_startup: bool,
    ) -> Result<Self> {
        paths.create()?;
        let settings = Settings::open(&paths)?;
        let store = Arc::new(
            SqliteStore::open(&paths).map_err(|e| psm_core::Error::InvalidData(e.to_string()))?,
        );
        let spool = Arc::new(Spool::new(paths.clone()));
        store.requeue_running_jobs()?;
        spool.drain(store.as_ref(), clock.as_ref())?;
        let nudge = NudgeServer::start(spool.clone(), store.clone())?;
        let hooks_compiled = editing::capture_one::hooks::compile_all(&paths, nudge.port()).is_ok();
        let health = Arc::new(HealthService::new());
        let document_id = settings.document_id.clone().unwrap_or_default();
        if document_id.is_empty() {
            health.set("document_not_open", "No Capture One session configured");
            health.set("capture_one", "No Capture One session configured");
        }
        let editing: Arc<dyn EditingProvider> = Arc::new(CaptureOneAdapter::new(
            scripting.clone(),
            store.clone(),
            health.clone(),
            document_id.clone(),
            settings.preview_recipe.clone(),
            settings.web_recipe.clone(),
            paths.clone(),
        ));
        let camera: Arc<dyn CameraProvider> = Arc::new(CameraViaCaptureOne {
            scripting,
            document_id: document_id.clone(),
        });
        let mut runner = JobRunner::new(store.clone());
        runner.register(
            "mirror_tag",
            Arc::new(MirrorTag {
                editing: editing.clone(),
                store: store.clone(),
                health: health.clone(),
            }),
        );
        let jobs = Arc::new(runner);
        if run_startup && !document_id.is_empty() {
            let startup_editing = editing.clone();
            let startup_store = store.clone();
            let startup_health = health.clone();
            let startup_camera = camera.clone();
            thread::spawn(move || {
                let _ = (EditingStartup {
                    provider: startup_editing.as_ref(),
                    store: startup_store.as_ref(),
                    health: startup_health.as_ref(),
                })
                .run();
                let _ =
                    CameraService::new(startup_camera.as_ref(), startup_health.as_ref()).status();
            });
        }
        let stop = Arc::new(AtomicBool::new(false));
        let bg_stop = stop.clone();
        let bg_spool = spool.clone();
        let bg_store = store.clone();
        let bg_jobs = jobs.clone();
        let bg_clock = clock.clone();
        let spool_dir = paths.spool();
        let background = thread::spawn(move || {
            let mut watcher = FsWatcher::watch(&spool_dir).ok();
            let mut last_drain = std::time::Instant::now();
            while !bg_stop.load(Ordering::Relaxed) {
                if watcher
                    .as_mut()
                    .is_some_and(|(_, rx)| rx.try_recv().is_ok())
                    || last_drain.elapsed() >= Duration::from_secs(5)
                {
                    let _ = bg_spool.drain(bg_store.as_ref(), bg_clock.as_ref());
                    last_drain = std::time::Instant::now();
                }
                if run_startup {
                    let _ = bg_jobs.tick(bg_clock.now_ms());
                }
                thread::sleep(Duration::from_millis(50));
            }
        });
        Ok(Self {
            paths,
            settings,
            store,
            spool,
            nudge,
            hooks_compiled,
            health,
            editing,
            camera,
            jobs,
            clock,
            stop,
            background: Some(background),
        })
    }
    pub fn nudge_port(&self) -> u16 {
        self.nudge.port()
    }
    pub fn nudge_addr(&self) -> std::net::SocketAddr {
        self.nudge.addr()
    }
    pub fn startup(&self) -> Result<()> {
        (EditingStartup {
            provider: self.editing.as_ref(),
            store: self.store.as_ref(),
            health: self.health.as_ref(),
        })
        .run()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(bg) = self.background.take() {
            let _ = bg.join();
        }
    }
}
