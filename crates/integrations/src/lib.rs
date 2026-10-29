pub mod camera;
pub mod commerce;
pub mod editing;

use camera::via_capture_one::CameraViaCaptureOne;
use commerce::shopify::{
    adapter::{PollConfig, ShopifyAdapter, ShopifyConfig},
    transport::ReqwestTransport,
};
use editing::capture_one::{
    adapter::CaptureOneAdapter, jxa::JxaScripting, mirror::MirrorTag,
    scripting::CaptureOneScripting,
};
use psm_core::{
    Result,
    domain::{editing::Job, session::PublishStep},
    events::AppEvent,
    mechanisms::{arrival::path_key, hash::sha256_file},
    ports::{
        camera::CameraProvider,
        clock::{Clock, SystemClock},
        commerce::CommerceAdapter,
        editing::EditingProvider,
        events::EventSink,
        raw_previews::RawPreviews,
        store::Store,
    },
    services::{
        arrival::ArrivalService, attributes::AttributesService, camera::CameraService,
        editing_events::EditingEventsDispatcher, editing_startup::EditingStartup,
        export::ExportService, health::HealthService, previews::PreviewService,
        publish::PublishService, settings::SettingsService,
    },
};
use psm_infra::{
    backup,
    export_fs::ExportFs,
    jobs::{JobHandler, JobRunner},
    nudge::NudgeServer,
    paths::AppPaths,
    previews_store::PreviewStore,
    rawpreview::EmbeddedJpeg,
    settings_file::Settings,
    spool::Spool,
    sqlite::store::SqliteStore,
    watcher::FsWatcher,
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
    pub commerce: Option<Arc<dyn CommerceAdapter>>,
    pub jobs: Arc<JobRunner>,
    pub publish_jobs: Arc<JobRunner>,
    pub arrival: Arc<ArrivalService>,
    pub previews: Arc<PreviewService>,
    pub export: Arc<ExportService>,
    pub publish: Option<Arc<PublishService>>,
    pub events: Arc<EventHub>,
    pub clock: Arc<dyn Clock>,
    scripting: Arc<dyn CaptureOneScripting>,
    user_scripting: Arc<dyn CaptureOneScripting>,
    background_scripting: Arc<dyn CaptureOneScripting>,
    run_startup: bool,
    commerce_override: Option<Arc<dyn CommerceAdapter>>,
    stop: Arc<AtomicBool>,
    background: Option<thread::JoinHandle<()>>,
    publish_background: Option<thread::JoinHandle<()>>,
}

#[derive(Clone)]
pub struct AppHandles {
    pub paths: AppPaths,
    pub settings: Settings,
    pub store: Arc<SqliteStore>,
    pub health: Arc<HealthService>,
    pub clock: Arc<dyn Clock>,
    pub editing: Arc<dyn EditingProvider>,
    pub camera: Arc<dyn CameraProvider>,
    pub commerce: Option<Arc<dyn CommerceAdapter>>,
    pub arrival: Arc<ArrivalService>,
    pub previews: Arc<PreviewService>,
    pub export: Arc<ExportService>,
    pub publish: Option<Arc<PublishService>>,
    pub events: Arc<EventHub>,
    pub hooks_compiled: bool,
    pub nudge_running: bool,
}

pub struct SettingsSetContext {
    paths: AppPaths,
    settings: Settings,
    health: Arc<HealthService>,
    scripting: Arc<dyn CaptureOneScripting>,
    lanes: (Arc<dyn CaptureOneScripting>, Arc<dyn CaptureOneScripting>),
    clock: Arc<dyn Clock>,
    run_startup: bool,
    commerce_override: Option<Arc<dyn CommerceAdapter>>,
}

pub enum PreparedSettings {
    Theme(Settings),
    Runtime(Box<App>),
}

impl SettingsSetContext {
    pub fn prepare(self, patch: &serde_json::Value) -> Result<PreparedSettings> {
        let previous = settings_value(&self.settings)?;
        let next = SettingsService::set(&previous, patch)?;
        let settings: Settings = serde_json::from_value(next.clone())
            .map_err(|e| psm_core::Error::InvalidValue(e.to_string()))?;
        if let Some(integration) = settings.integration_settings()? {
            psm_infra::logging::add_secret(&integration.client_secret);
        }
        Settings::set(&self.paths, &settings)?;
        let mut a = previous;
        let mut b = next;
        a["theme"] = serde_json::Value::Null;
        b["theme"] = serde_json::Value::Null;
        if a == b {
            return Ok(PreparedSettings::Theme(settings));
        }
        match App::build(
            self.paths.clone(),
            self.scripting,
            self.lanes,
            self.clock,
            self.run_startup,
            self.commerce_override,
            false,
        ) {
            Ok(app) => {
                if let Some(failure) = self
                    .health
                    .report("", false, false)
                    .items
                    .into_iter()
                    .find(|item| item.code == "backup_failed" && !item.ok)
                {
                    app.health.set("backup_failed", failure.detail);
                }
                Ok(PreparedSettings::Runtime(Box::new(app)))
            }
            Err(error) => {
                let _ = Settings::set(&self.paths, &self.settings);
                Err(error)
            }
        }
    }
}

impl AppHandles {
    pub fn health_check(&self) -> Result<psm_core::domain::HealthReport> {
        if self
            .settings
            .document_id
            .as_deref()
            .is_some_and(|d| !d.is_empty())
        {
            if let Err(error) = (EditingStartup {
                provider: self.editing.as_ref(),
                store: self.store.as_ref(),
                health: self.health.as_ref(),
            })
            .run()
            {
                tracing::warn!(%error, "health: editing checks failed");
            }
            if let Err(error) =
                CameraService::new(self.camera.as_ref(), self.health.as_ref()).status()
            {
                tracing::warn!(%error, "health: camera status failed");
            }
        }
        self.health.check_integration(
            self.store.as_ref(),
            self.clock.as_ref(),
            self.commerce.as_deref(),
        )?;
        Ok(self.health.report(
            &self.paths.root.to_string_lossy(),
            self.hooks_compiled,
            self.nudge_running,
        ))
    }
    pub fn settings_get(&self) -> Result<serde_json::Value> {
        let mut value = settings_value(&self.settings)?;
        if let Ok(Some(integration)) = self.settings.integration_settings() {
            value["integration"] = serde_json::to_value(integration)
                .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        } else {
            value["integration"] = serde_json::Value::Null;
        }
        Ok(SettingsService::get(&value))
    }
    pub fn on_window_focus(&self) {
        let previews = self.previews.clone();
        thread::spawn(move || {
            if let Err(error) = previews.on_focus() {
                tracing::warn!(%error, "window focus preview check failed");
            }
        });
    }
}

impl App {
    pub fn settings_set_context(&self) -> SettingsSetContext {
        SettingsSetContext {
            paths: self.paths.clone(),
            settings: self.settings.clone(),
            health: self.health.clone(),
            scripting: self.scripting.clone(),
            lanes: (
                self.user_scripting.clone(),
                self.background_scripting.clone(),
            ),
            clock: self.clock.clone(),
            run_startup: self.run_startup,
            commerce_override: self.commerce_override.clone(),
        }
    }
    pub fn apply_settings(
        &mut self,
        prepared: PreparedSettings,
    ) -> Result<(serde_json::Value, Option<App>)> {
        let old = match prepared {
            PreparedSettings::Theme(settings) => {
                self.settings = settings;
                self.health.clear("settings_invalid");
                None
            }
            PreparedSettings::Runtime(next) => {
                self.events.transfer_target_to(&next.events);
                let old = std::mem::replace(self, *next);
                self.events.emit(AppEvent::HealthChanged);
                Some(old)
            }
        };
        Ok((self.settings_get()?, old))
    }
    pub fn handles(&self) -> AppHandles {
        AppHandles {
            paths: self.paths.clone(),
            settings: self.settings.clone(),
            store: self.store.clone(),
            health: self.health.clone(),
            clock: self.clock.clone(),
            editing: self.editing.clone(),
            camera: self.camera.clone(),
            commerce: self.commerce.clone(),
            arrival: self.arrival.clone(),
            previews: self.previews.clone(),
            export: self.export.clone(),
            publish: self.publish.clone(),
            events: self.events.clone(),
            hooks_compiled: self.hooks_compiled,
            nudge_running: self.nudge_port() != 0,
        }
    }
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
        Self::open_with_adapter(paths, scripting, clock, run_startup, None)
    }

    pub fn open_with_adapter(
        paths: AppPaths,
        scripting: Arc<dyn CaptureOneScripting>,
        clock: Arc<dyn Clock>,
        run_startup: bool,
        commerce_override: Option<Arc<dyn CommerceAdapter>>,
    ) -> Result<Self> {
        let (user_scripting, background_scripting) =
            editing::capture_one::gate::GatedScripting::pair(scripting.clone());
        Self::build(
            paths,
            scripting,
            (user_scripting, background_scripting),
            clock,
            run_startup,
            commerce_override,
            true,
        )
    }

    fn build(
        paths: AppPaths,
        scripting: Arc<dyn CaptureOneScripting>,
        lanes: (Arc<dyn CaptureOneScripting>, Arc<dyn CaptureOneScripting>),
        clock: Arc<dyn Clock>,
        run_startup: bool,
        commerce_override: Option<Arc<dyn CommerceAdapter>>,
        start_backup: bool,
    ) -> Result<Self> {
        let (user_scripting, background_scripting) = lanes;
        paths.create()?;
        let (settings, settings_problem) = Settings::open_lenient(&paths)?;
        let integration_result = settings.integration_settings();
        let integration_invalid = integration_result.is_err();
        let integration = match integration_result {
            Ok(value) => value,
            Err(error) => {
                tracing::warn!(%error, "integration settings invalid; using Export mode");
                None
            }
        };
        if let Some(integration) = &integration {
            psm_infra::logging::add_secret(&integration.client_secret);
        }
        let commerce = match commerce_override.clone() {
            Some(adapter) if !integration_invalid => Some(adapter),
            None if integration.is_some() => commerce_from_settings(&settings, clock.clone())?,
            _ => None,
        };
        let store = Arc::new(
            SqliteStore::open(&paths).map_err(|e| psm_core::Error::InvalidData(e.to_string()))?,
        );
        // ch12 §Backup: a copy at every start (a settings reopen is not a start).
        let backup_problem = if start_backup {
            backup::create(&paths).err().map(|e| e.to_string())
        } else {
            None
        };
        if start_backup {
            store.requeue_running_jobs()?;
            if commerce.is_some() {
                for id in store.active_publish_items()? {
                    store.enqueue_publish_item(id, clock.now_ms())?;
                }
            }
        }
        let spool = Arc::new(Spool::new(paths.clone()));
        let health = Arc::new(HealthService::new());
        if let Some(problem) = settings_problem {
            health.set("settings_invalid", format!("settings.json: {problem}"));
        } else if let Err(error) = settings.integration_settings() {
            health.set("settings_invalid", format!("integration: {error}"));
        }
        if let Some(problem) = backup_problem {
            tracing::warn!(problem, "startup backup failed");
            health.set("backup_failed", problem);
        }
        let document_id = settings.document_id.clone().unwrap_or_default();
        if document_id.is_empty() {
            health.set("document_not_open", "No Capture One session configured");
        }
        let editing: Arc<dyn EditingProvider> = Arc::new(CaptureOneAdapter::new(
            (user_scripting.clone(), background_scripting.clone()),
            store.clone(),
            health.clone(),
            document_id.clone(),
            settings.preview_recipe.clone(),
            settings.web_recipe.clone(),
            paths.clone(),
        ));
        let camera: Arc<dyn CameraProvider> = Arc::new(CameraViaCaptureOne {
            scripting: user_scripting.clone(),
            document_id: document_id.clone(),
        });
        let events = Arc::new(EventHub::default());
        let arrival = Arc::new(ArrivalService::new(
            store.clone(),
            editing.clone(),
            camera.clone(),
            events.clone(),
            clock.clone(),
        ));
        let export = Arc::new(ExportService::new(
            store.clone(),
            editing.clone(),
            events.clone(),
            clock.clone(),
            Arc::new(ExportFs::new(&settings.export_folder)),
        ));
        let publish = commerce.as_ref().map(|adapter| {
            Arc::new(PublishService::new(
                store.clone(),
                adapter.clone(),
                export.clone(),
                clock.clone(),
                events.clone(),
            ))
        });
        let mut previews = PreviewService::new(
            store.clone(),
            editing.clone(),
            events.clone(),
            clock.clone(),
            Arc::new(PreviewStore(paths.clone())),
            Some(&settings.preview_delay_s.to_string()),
        );
        previews.export_staging = Some(export.clone());
        let previews = Arc::new(previews);
        spool.set_dispatcher(Arc::new(EditingEventsDispatcher {
            arrival: arrival.clone(),
            handler: previews.clone(),
        }))?;
        let nudge = NudgeServer::start(spool.clone(), store.clone())?;
        let hooks_compiled = editing::capture_one::hooks::compile_all(&paths, nudge.port()).is_ok();
        if let Err(error) = spool.drain(store.as_ref(), clock.as_ref()) {
            tracing::warn!(%error, "startup spool drain deferred");
        }
        if !document_id.is_empty()
            && let Err(error) = arrival.scan()
        {
            tracing::warn!(%error, "startup capture scan deferred");
        }
        if let Err(error) = previews.on_startup() {
            tracing::warn!(%error, "startup sidecar check deferred");
        }
        let mut runner = JobRunner::new(store.clone()).excluding_kind("publish_step");
        let mut publish_runner = JobRunner::new(store.clone()).only_kind("publish_step");
        runner.register(
            "mirror_tag",
            Arc::new(MirrorTag {
                editing: editing.clone(),
                store: store.clone(),
                health: health.clone(),
            }),
        );
        runner.register(
            "hash",
            Arc::new(HashJob {
                store: store.clone(),
            }),
        );
        runner.register(
            "original_preview",
            Arc::new(OriginalPreviewJob {
                store: store.clone(),
                paths: paths.clone(),
            }),
        );
        runner.register(
            "render_preview",
            Arc::new(RenderPreviewJob {
                previews: previews.clone(),
            }),
        );
        runner.register("render_export", Arc::new(InterruptedExport));
        if let Some(service) = publish.clone() {
            publish_runner.register(
                "publish_step",
                Arc::new(PublishJob {
                    service,
                    store: store.clone(),
                    paths: paths.clone(),
                    health: health.clone(),
                }),
            );
        }
        let jobs = Arc::new(runner);
        let publish_jobs = Arc::new(publish_runner);
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
        if run_startup && let Some(adapter) = commerce.clone() {
            let schema_store = store.clone();
            let schema_clock = clock.clone();
            thread::spawn(move || {
                if let Err(error) = AttributesService::new(
                    schema_store.as_ref(),
                    schema_clock.as_ref(),
                    Some("shopify"),
                )
                .refresh_schema(adapter.as_ref())
                {
                    tracing::warn!(%error,"schema refresh deferred");
                }
            });
        }
        let stop = Arc::new(AtomicBool::new(false));
        let bg_stop = stop.clone();
        let bg_spool = spool.clone();
        let bg_store = store.clone();
        let bg_jobs = jobs.clone();
        let bg_clock = clock.clone();
        let bg_arrival = arrival.clone();
        let bg_previews = previews.clone();
        let spool_dir = paths.spool();
        let background = thread::spawn(move || {
            let mut watcher = FsWatcher::watch(&spool_dir).ok();
            let mut capture_watcher = bg_arrival
                .capture_root()
                .ok()
                .and_then(|root| FsWatcher::watch(&root).ok());
            let mut last_drain = std::time::Instant::now();
            let mut last_capture_retry = std::time::Instant::now() - Duration::from_secs(5);
            while !bg_stop.load(Ordering::Relaxed) {
                if run_startup
                    && !document_id.is_empty()
                    && capture_watcher.is_none()
                    && last_capture_retry.elapsed() >= Duration::from_secs(5)
                {
                    last_capture_retry = std::time::Instant::now();
                    if let Ok(root) = bg_arrival.capture_root()
                        && let Ok(watcher) = FsWatcher::watch(&root)
                    {
                        capture_watcher = Some(watcher);
                        let _ = bg_arrival.scan();
                    }
                }
                if watcher
                    .as_mut()
                    .is_some_and(|(_, rx)| rx.try_recv().is_ok())
                    || last_drain.elapsed() >= Duration::from_secs(5)
                {
                    let _ = bg_spool.drain(bg_store.as_ref(), bg_clock.as_ref());
                    last_drain = std::time::Instant::now();
                }
                if run_startup {
                    if let Some((_, rx)) = capture_watcher.as_mut() {
                        while let Ok(event) = rx.try_recv() {
                            let _ = bg_arrival.watcher_event(&event.path);
                        }
                    }
                    let _ = bg_arrival.tick();
                    let _ = bg_jobs.tick(bg_clock.now_ms());
                    let _ = bg_previews.tick();
                }
                thread::sleep(Duration::from_millis(50));
            }
        });
        let publish_background = if run_startup {
            let worker_stop = stop.clone();
            let worker_jobs = publish_jobs.clone();
            let worker_clock = clock.clone();
            Some(thread::spawn(move || {
                while !worker_stop.load(Ordering::Relaxed) {
                    let _ = worker_jobs.tick(worker_clock.now_ms());
                    thread::sleep(Duration::from_millis(50));
                }
            }))
        } else {
            None
        };
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
            commerce,
            jobs,
            publish_jobs,
            arrival,
            previews,
            export,
            publish,
            events,
            clock,
            scripting,
            user_scripting,
            background_scripting,
            run_startup,
            commerce_override,
            stop,
            background: Some(background),
            publish_background,
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

    /// ch12 §Health: re-runs the editing startup checks (document, hooks, recipes, capture
    /// folder), the camera status and the integration checks (09A MC-23/MC-24), then reports.
    pub fn health_check(&self) -> Result<psm_core::domain::HealthReport> {
        self.handles().health_check()
    }

    /// The settings as shown (client secret masked, effective preview delay; ch12 §Settings).
    pub fn settings_get(&self) -> Result<serde_json::Value> {
        self.handles().settings_get()
    }

    /// Applies a settings patch: validated, written atomically with mode 0600, then the runtime
    /// is reopened over the new settings so providers, export folder and preview delay take
    /// effect, and `health_changed` is emitted. A theme-only change does not reopen.
    pub fn settings_set(&mut self, patch: &serde_json::Value) -> Result<serde_json::Value> {
        let prepared = self.settings_set_context().prepare(patch)?;
        let (value, old) = self.apply_settings(prepared)?;
        if let Some(mut old) = old {
            old.shutdown();
        }
        Ok(value)
    }

    /// Stops the background workers (idempotent; also run on drop).
    pub fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(bg) = self.background.take() {
            let _ = bg.join();
        }
        if let Some(bg) = self.publish_background.take() {
            let _ = bg.join();
        }
    }

    /// Rebuilds the runtime over the current `settings.json`, keeping the event target.
    pub fn reopen(&mut self) -> Result<()> {
        let next = Self::build(
            self.paths.clone(),
            self.scripting.clone(),
            (
                self.user_scripting.clone(),
                self.background_scripting.clone(),
            ),
            self.clock.clone(),
            self.run_startup,
            self.commerce_override.clone(),
            false,
        )?;
        self.shutdown();
        self.events.transfer_target_to(&next.events);
        let old = std::mem::replace(self, next);
        drop(old);
        Ok(())
    }
}

fn settings_value(settings: &Settings) -> Result<serde_json::Value> {
    serde_json::to_value(settings).map_err(|e| psm_core::Error::InvalidData(e.to_string()))
}

pub fn commerce_from_settings(
    settings: &Settings,
    clock: Arc<dyn Clock>,
) -> Result<Option<Arc<dyn CommerceAdapter>>> {
    let Some(integration) = settings.integration_settings()? else {
        return Ok(None);
    };
    if integration.kind != "shopify" {
        return Ok(None);
    }
    let transport = Arc::new(ReqwestTransport::new()?);
    Ok(Some(Arc::new(ShopifyAdapter::new(
        ShopifyConfig::from(&integration),
        transport,
        clock,
        PollConfig::default(),
    ))))
}

#[derive(Default)]
pub struct EventHub {
    state: std::sync::Mutex<EventHubState>,
}
#[derive(Default)]
struct EventHubState {
    target: Option<Arc<dyn EventSink>>,
    queued: Vec<AppEvent>,
}
impl EventHub {
    pub fn transfer_target_to(&self, other: &EventHub) {
        if let Ok(state) = self.state.lock()
            && let Some(target) = state.target.clone()
        {
            other.set_target(target);
        }
    }
    pub fn set_target(&self, target: Arc<dyn EventSink>) {
        if let Ok(mut state) = self.state.lock() {
            let queued = std::mem::take(&mut state.queued);
            state.target = Some(target.clone());
            drop(state);
            for event in queued {
                target.emit(event);
            }
        }
    }
}
impl EventSink for EventHub {
    fn emit(&self, event: AppEvent) {
        if let Ok(mut state) = self.state.lock() {
            if let Some(target) = state.target.clone() {
                drop(state);
                target.emit(event);
            } else {
                state.queued.push(event);
            }
        }
    }
}

struct HashJob {
    store: Arc<dyn Store>,
}
impl JobHandler for HashJob {
    fn run(&self, job: &Job, _: i64) -> Result<()> {
        self.store.set_raw_hash(
            &job.target,
            &sha256_file(std::path::Path::new(&job.target))?,
        )
    }
}
struct OriginalPreviewJob {
    store: Arc<dyn Store>,
    paths: AppPaths,
}
impl JobHandler for OriginalPreviewJob {
    fn run(&self, job: &Job, _: i64) -> Result<()> {
        let key = path_key(&job.target);
        let dest = self
            .paths
            .root
            .join("previews")
            .join(&key[..2])
            .join(&key)
            .join("original.jpg");
        let extracted = EmbeddedJpeg.extract(std::path::Path::new(&job.target), &dest)?;
        self.store.set_original_preview(
            &job.target,
            extracted
                .map(|_| dest.to_string_lossy().into_owned())
                .as_deref(),
            extracted.and_then(|e| e.orientation).map(i64::from),
        )
    }
}
struct RenderPreviewJob {
    previews: Arc<PreviewService>,
}
impl JobHandler for RenderPreviewJob {
    fn run(&self, job: &Job, _: i64) -> Result<()> {
        self.previews.request_initial(&job.target)
    }
}

/// An export interrupted by a restart: its renders and staging folder belonged to the previous
/// process, so the run is abandoned (the user exports again); its row must not block the queue.
struct InterruptedExport;
impl JobHandler for InterruptedExport {
    fn run(&self, job: &Job, _: i64) -> Result<()> {
        tracing::info!(job = job.id, "export interrupted by a restart; abandoned");
        Ok(())
    }
}
struct PublishJob {
    service: Arc<PublishService>,
    store: Arc<dyn Store>,
    paths: AppPaths,
    health: Arc<HealthService>,
}
impl JobHandler for PublishJob {
    fn run(&self, job: &Job, _: i64) -> Result<()> {
        let id = job
            .target
            .parse::<i64>()
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        let was_done = self.store.publish_item(id)?.step == PublishStep::Done;
        self.service.run_item(id)?;
        if !was_done && self.store.publish_item(id)?.step == PublishStep::Done {
            match backup::create(&self.paths) {
                Ok(()) => self.health.clear("backup_failed"),
                Err(error) => {
                    tracing::warn!(%error, "post-publish backup failed");
                    self.health.set("backup_failed", error.to_string());
                }
            }
        }
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.shutdown();
    }
}
