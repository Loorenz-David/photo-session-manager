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
    domain::editing::Job,
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
        publish::PublishService,
    },
};
use psm_infra::{
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
    pub arrival: Arc<ArrivalService>,
    pub previews: Arc<PreviewService>,
    pub export: Arc<ExportService>,
    pub publish: Option<Arc<PublishService>>,
    pub events: Arc<EventHub>,
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
        Self::open_with_adapter(paths, scripting, clock, run_startup, None)
    }

    pub fn open_with_adapter(
        paths: AppPaths,
        scripting: Arc<dyn CaptureOneScripting>,
        clock: Arc<dyn Clock>,
        run_startup: bool,
        commerce_override: Option<Arc<dyn CommerceAdapter>>,
    ) -> Result<Self> {
        paths.create()?;
        let settings = Settings::open(&paths)?;
        let commerce = match commerce_override {
            Some(adapter) => Some(adapter),
            None => commerce_from_settings(&settings, clock.clone())?,
        };
        let store = Arc::new(
            SqliteStore::open(&paths).map_err(|e| psm_core::Error::InvalidData(e.to_string()))?,
        );
        let spool = Arc::new(Spool::new(paths.clone()));
        store.requeue_running_jobs()?;
        if commerce.is_some() {
            for id in store.active_publish_items()? {
                store.enqueue_publish_item(id, clock.now_ms())?;
            }
        }
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
        let mut runner = JobRunner::new(store.clone());
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
            runner.register("publish_step", Arc::new(PublishJob { service }));
        }
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
            arrival,
            previews,
            export,
            publish,
            events,
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
}
impl JobHandler for PublishJob {
    fn run(&self, job: &Job, _: i64) -> Result<()> {
        let id = job
            .target
            .parse::<i64>()
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        self.service.run_item(id)
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
