pub mod camera;
pub mod commerce;
pub mod editing;
pub mod render_output;

use camera::direct::StartOrder;
use camera::runtime::{AppBinding, CameraRuntime};
use camera::via_capture_one::CameraViaCaptureOne;
use commerce::shopify::{
    adapter::{PollConfig, ShopifyAdapter, ShopifyConfig},
    transport::ReqwestTransport,
};
use editing::capture_one::{
    adapter::CaptureOneAdapter, follow::CaptureOneFollow, jxa::JxaScripting,
    launch_gate::LaunchGate, mirror::MirrorTag, scripting::CaptureOneScripting,
};
use psm_core::{
    Result,
    domain::{
        composition::{EditorGeometry, Fingerprint, FrameSource},
        editing::{Job, ProvisionInputs},
        session::PublishStep,
    },
    events::AppEvent,
    mechanisms::{arrival::path_key, hash::sha256_file},
    ports::{
        camera::CameraProvider,
        clock::{Clock, SystemClock},
        commerce::CommerceAdapter,
        editing::EditingProvider,
        events::EventSink,
        raw_previews::RawPreviews,
        secrets::SecretStore,
        store::Store,
    },
    services::{
        arrival::ArrivalService,
        attributes::AttributesService,
        camera::CameraService,
        composition::CompositionService,
        editing_events::EditingEventsDispatcher,
        editing_startup::EditingStartup,
        export::{ExportRender, ExportService},
        health::HealthService,
        history::{History, HistoryStore},
        previews::PreviewService,
        publish::PublishService,
        settings::SettingsService,
    },
};
use psm_infra::{
    backup,
    export_fs::ExportFs,
    imaging::ImageOpsImpl,
    jobs::{JobHandler, JobRunner},
    keychain::KeychainSecrets,
    media_store::{MasterFiles, OutputFiles},
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

pub const EDITOR_WAIT: Duration = Duration::from_secs(45);

pub struct App {
    pub paths: AppPaths,
    pub settings: Settings,
    pub provision: ProvisionInputs,
    pub store: Arc<SqliteStore>,
    pub spool: Arc<Spool>,
    pub nudge: NudgeServer,
    pub hooks_compiled: bool,
    pub health: Arc<HealthService>,
    pub editing: Arc<dyn EditingProvider>,
    pub camera: Arc<dyn CameraProvider>,
    pub camera_runtime: Arc<CameraRuntime>,
    pub commerce: Option<Arc<dyn CommerceAdapter>>,
    pub jobs: Arc<JobRunner>,
    pub publish_jobs: Arc<JobRunner>,
    /// The `render_output` runner (10D MC-28: one output render at a time, on its own thread).
    pub render_jobs: Arc<JobRunner>,
    pub arrival: Arc<ArrivalService>,
    pub previews: Arc<PreviewService>,
    pub follow: Arc<CaptureOneFollow>,
    pub compositions: Arc<CompositionService>,
    pub export: Arc<ExportService>,
    pub publish: Option<Arc<PublishService>>,
    pub events: Arc<EventHub>,
    pub history: Arc<History>,
    pub clock: Arc<dyn Clock>,
    scripting: Arc<dyn CaptureOneScripting>,
    lanes: Lanes,
    run_startup: bool,
    commerce_override: Option<Arc<dyn CommerceAdapter>>,
    stop: Arc<AtomicBool>,
    background: Option<thread::JoinHandle<()>>,
    camera_background: Option<thread::JoinHandle<()>>,
    publish_background: Option<thread::JoinHandle<()>>,
    render_background: Option<thread::JoinHandle<()>>,
}

#[derive(Clone)]
pub struct AppHandles {
    pub paths: AppPaths,
    pub settings: Settings,
    pub provision: ProvisionInputs,
    pub store: Arc<SqliteStore>,
    pub health: Arc<HealthService>,
    pub clock: Arc<dyn Clock>,
    pub editing: Arc<dyn EditingProvider>,
    pub camera: Arc<dyn CameraProvider>,
    pub camera_runtime: Arc<CameraRuntime>,
    pub commerce: Option<Arc<dyn CommerceAdapter>>,
    pub arrival: Arc<ArrivalService>,
    pub previews: Arc<PreviewService>,
    pub follow: Arc<CaptureOneFollow>,
    pub compositions: Arc<CompositionService>,
    pub export: Arc<ExportService>,
    pub publish: Option<Arc<PublishService>>,
    pub events: Arc<EventHub>,
    pub history: Arc<History>,
    pub hooks_compiled: bool,
    pub nudge_running: bool,
}

/// The Capture One scripting lanes and what guards them, created once per app process in
/// `open_with_adapter_bind` and carried through every rebuild (P48 L14): a settings save never
/// produces a lane without the editor-launch gate or a second start order.
#[derive(Clone)]
pub struct Lanes {
    pub user: Arc<dyn CaptureOneScripting>,
    pub background: Arc<dyn CaptureOneScripting>,
    pub gate: Arc<LaunchGate>,
    pub start_order: Arc<StartOrder>,
    /// The master-render disk guard's free-space source (P56 C3(d)).
    pub disk: Arc<editing::capture_one::adapter::DiskSpace>,
}

pub struct SettingsSetContext {
    paths: AppPaths,
    settings: Settings,
    health: Arc<HealthService>,
    camera_runtime: Arc<CameraRuntime>,
    scripting: Arc<dyn CaptureOneScripting>,
    lanes: Lanes,
    clock: Arc<dyn Clock>,
    run_startup: bool,
    commerce_override: Option<Arc<dyn CommerceAdapter>>,
    history_store: HistoryStore,
}

pub enum PreparedSettings {
    Simple(Box<Settings>),
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
        for key in ["theme", "output_format"] {
            a[key] = serde_json::Value::Null;
            b[key] = serde_json::Value::Null;
        }
        a["camera"]["auto_take_back"] = serde_json::Value::Null;
        b["camera"]["auto_take_back"] = serde_json::Value::Null;
        if a == b {
            return Ok(PreparedSettings::Simple(Box::new(settings)));
        }
        match App::build(
            self.paths.clone(),
            self.scripting,
            self.lanes,
            self.clock,
            self.run_startup,
            self.commerce_override,
            false,
            Some(self.history_store),
            self.camera_runtime,
            Some(self.health.clone()),
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
        {
            if let Err(error) = (EditingStartup {
                provider: self.editing.as_ref(),
                store: self.store.as_ref(),
                health: self.health.as_ref(),
                provision: &self.provision,
                camera: Some(self.camera.as_ref()),
                follow: Some(self.follow.as_ref()),
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
        let mut shown = SettingsService::get(&value);
        shown["effective_document_id"] = serde_json::Value::String(
            self.settings
                .effective_document_id(&self.paths)
                .to_string_lossy()
                .into_owned(),
        );
        shown["default_document_id"] = serde_json::Value::String(
            self.settings
                .default_document_id(&self.paths)
                .to_string_lossy()
                .into_owned(),
        );
        Ok(shown)
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
            camera_runtime: self.camera_runtime.clone(),
            scripting: self.scripting.clone(),
            lanes: self.lanes.clone(),
            clock: self.clock.clone(),
            run_startup: self.run_startup,
            commerce_override: self.commerce_override.clone(),
            history_store: self.history.store(),
        }
    }
    pub fn apply_settings(
        &mut self,
        prepared: PreparedSettings,
    ) -> Result<(serde_json::Value, Option<App>)> {
        let old = match prepared {
            PreparedSettings::Simple(settings) => {
                self.settings = *settings;
                self.export.set_format(self.settings.output_format);
                self.camera_runtime
                    .set_auto_take_back(self.settings.camera.auto_take_back);
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
            provision: self.provision.clone(),
            store: self.store.clone(),
            health: self.health.clone(),
            clock: self.clock.clone(),
            editing: self.editing.clone(),
            camera: self.camera.clone(),
            camera_runtime: self.camera_runtime.clone(),
            commerce: self.commerce.clone(),
            arrival: self.arrival.clone(),
            previews: self.previews.clone(),
            follow: self.follow.clone(),
            compositions: self.compositions.clone(),
            export: self.export.clone(),
            publish: self.publish.clone(),
            events: self.events.clone(),
            history: self.history.clone(),
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
        Self::open_with_adapter_bind(
            paths,
            scripting,
            clock,
            run_startup,
            None,
            Arc::new(KeychainSecrets::default()),
            std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
        )
    }

    pub fn open_with_adapter(
        paths: AppPaths,
        scripting: Arc<dyn CaptureOneScripting>,
        clock: Arc<dyn Clock>,
        run_startup: bool,
        commerce_override: Option<Arc<dyn CommerceAdapter>>,
        secrets: Arc<dyn SecretStore>,
    ) -> Result<Self> {
        Self::open_with_adapter_bind(
            paths,
            scripting,
            clock,
            run_startup,
            commerce_override,
            secrets,
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn open_with_adapter_bind(
        paths: AppPaths,
        scripting: Arc<dyn CaptureOneScripting>,
        clock: Arc<dyn Clock>,
        run_startup: bool,
        commerce_override: Option<Arc<dyn CommerceAdapter>>,
        secrets: Arc<dyn SecretStore>,
        receiver_bind: std::net::IpAddr,
    ) -> Result<Self> {
        // One launch gate and one start order per process (P48 L14), beside the lanes.
        let gate = LaunchGate::new();
        let start_order = StartOrder::new(camera::support::installed(&paths).is_some());
        let (user, background) = editing::capture_one::gate::GatedScripting::pair(
            scripting.clone(),
            gate.clone(),
            start_order.clone(),
            clock.clone(),
        );
        let camera_runtime = CameraRuntime::new_with_receiver_bind(
            secrets,
            clock.clone(),
            paths.clone(),
            receiver_bind,
            start_order.clone(),
        );
        Self::build(
            paths,
            scripting,
            Lanes {
                user,
                background,
                gate,
                start_order,
                disk: Arc::new(editing::capture_one::adapter::DiskSpace::default()),
            },
            clock,
            run_startup,
            commerce_override,
            true,
            None,
            camera_runtime,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        paths: AppPaths,
        scripting: Arc<dyn CaptureOneScripting>,
        lanes: Lanes,
        clock: Arc<dyn Clock>,
        run_startup: bool,
        commerce_override: Option<Arc<dyn CommerceAdapter>>,
        start_backup: bool,
        history_store: Option<HistoryStore>,
        camera_runtime: Arc<CameraRuntime>,
        previous_health: Option<Arc<HealthService>>,
    ) -> Result<Self> {
        let user_scripting = lanes.user.clone();
        let background_scripting = lanes.background.clone();
        paths.create()?;
        tracing::info!(data_dir = %paths.root.display(), start = start_backup, "app open");
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
            SqliteStore::open_with_default_shape(&paths, &settings.default_shape)
                .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?,
        );
        store.backfill_compositions(&settings.default_shape, clock.now_ms())?;
        // ch12 §Backup: a copy at every start (a settings reopen is not a start).
        let backup_problem = if start_backup {
            backup::create(&paths).err().map(|e| e.to_string())
        } else {
            None
        };
        if start_backup {
            store.mark_previews_stale_once("rerender_after_composition", clock.now_ms())?;
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
        let document_id = settings
            .effective_document_id(&paths)
            .to_string_lossy()
            .into_owned();
        let provision = ProvisionInputs {
            style_name: settings.style_name.clone(),
            wait: EDITOR_WAIT,
        };
        let editing: Arc<dyn EditingProvider> = Arc::new(
            CaptureOneAdapter::new(
                (user_scripting.clone(), background_scripting.clone()),
                store.clone(),
                health.clone(),
                document_id.clone(),
                settings.preview_recipe.clone(),
                settings.master_recipe.clone(),
                paths.clone(),
            )
            .with_disk(lanes.disk.clone()),
        );
        // 10E §Camera support packages (P46 L18): the direct adapter whenever a camera support
        // package is installed, the Capture One camera adapter otherwise; decided up front.
        let direct_camera = camera::support::installed(&paths).is_some();
        let camera: Arc<dyn CameraProvider> = if direct_camera {
            camera_runtime.direct_adapter()
        } else {
            Arc::new(CameraViaCaptureOne {
                scripting: user_scripting.clone(),
                document_id: document_id.clone(),
            })
        };
        let events = Arc::new(EventHub::default());
        let history = Arc::new(match history_store {
            Some(store) => History::with_store(store, events.clone()),
            None => History::new(events.clone()),
        });
        let compositions = Arc::new(CompositionService {
            store: store.clone(),
            clock: clock.clone(),
            events: events.clone(),
            history: history.clone(),
            default_shape: psm_core::domain::composition::Shape::parse(&settings.default_shape)
                .unwrap_or(psm_core::domain::composition::Shape::Square),
        });
        let imaging: Arc<dyn psm_core::ports::images::Imaging> = Arc::new(ImageOpsImpl);
        let masters: Arc<dyn psm_core::ports::images::MasterStore> =
            Arc::new(MasterFiles(paths.clone()));
        let outputs: Arc<dyn psm_core::ports::images::OutputStore> =
            Arc::new(OutputFiles(paths.clone()));
        let export = Arc::new(ExportService::new(
            store.clone(),
            editing.clone(),
            events.clone(),
            clock.clone(),
            Arc::new(ExportFs::new(&settings.export_folder)),
            ExportRender {
                imaging: imaging.clone(),
                masters: masters.clone(),
                outputs: outputs.clone(),
                format: settings.output_format,
            },
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
        previews.imaging = imaging.clone();
        previews.export_staging = Some(export.clone());
        previews.health = Some(health.clone());
        let previews = Arc::new(previews);
        let follow = Arc::new(CaptureOneFollow::new(
            user_scripting.clone(),
            editing.clone(),
            store.clone(),
            previews.clone(),
            health.clone(),
            clock.clone(),
            settings.style_name.clone(),
        ));
        let weak_follow = Arc::downgrade(&follow);
        previews.set_before_render(Arc::new(move |raw| {
            weak_follow
                .upgrade()
                .map_or(Ok(()), |follow| follow.prepare_render(raw))
        }));
        let arrival = ArrivalService::new(
            store.clone(),
            editing.clone(),
            camera.clone(),
            events.clone(),
            clock.clone(),
        )
        .with_follow(follow.clone(), direct_camera);
        // R-I28: the direct adapter's capture root is the receiver's document formula.
        let arrival = Arc::new(if direct_camera {
            let document = settings.effective_document_id(&paths);
            arrival.with_capture_root(Arc::new(move || {
                Ok(std::fs::canonicalize(document.join("Capture"))?)
            }))
        } else {
            arrival
        });
        spool.set_dispatcher(Arc::new(EditingEventsDispatcher {
            arrival: arrival.clone(),
            handler: previews.clone(),
        }))?;
        let nudge = NudgeServer::start(spool.clone(), store.clone())?;
        let hooks_compiled = editing::capture_one::hooks::compile_all(&paths, nudge.port()).is_ok();
        if let Err(error) = spool.drain(store.as_ref(), clock.as_ref()) {
            tracing::warn!(%error, "startup spool drain deferred");
        }
        if let Err(error) = arrival.scan() {
            tracing::warn!(%error, "startup capture scan deferred");
        }
        if let Err(error) = previews.on_startup() {
            tracing::warn!(%error, "startup sidecar check deferred");
        }
        let mut runner =
            JobRunner::new(store.clone()).excluding_kinds(&["publish_step", "render_output"]);
        let mut publish_runner = JobRunner::new(store.clone()).only_kind("publish_step");
        let mut render_runner = JobRunner::new(store.clone()).only_kind("render_output");
        render_runner.register(
            "render_output",
            Arc::new(render_output::RenderOutputJob {
                store: store.clone(),
                imaging: imaging.clone(),
                masters: masters.clone(),
                outputs: outputs.clone(),
                export: export.clone(),
            }),
        );
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
                events: events.clone(),
            }),
        );
        runner.register(
            "render_preview",
            Arc::new(RenderPreviewJob {
                previews: previews.clone(),
            }),
        );
        runner.register("render_export", Arc::new(InterruptedExport));
        runner.register("render_master", Arc::new(InterruptedExport));
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
        let render_jobs = Arc::new(render_runner);
        if run_startup {
            let startup_editing = editing.clone();
            let startup_store = store.clone();
            let startup_health = health.clone();
            let startup_camera = camera.clone();
            let startup_provision = provision.clone();
            let startup_follow = follow.clone();
            thread::spawn(move || {
                let _ = (EditingStartup {
                    provider: startup_editing.as_ref(),
                    store: startup_store.as_ref(),
                    health: startup_health.as_ref(),
                    provision: &startup_provision,
                    camera: Some(startup_camera.as_ref()),
                    follow: Some(startup_follow.as_ref()),
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
        let bg_follow = follow.clone();
        let camera_stop = stop.clone();
        let bg_camera_runtime = camera_runtime.clone();
        let camera_clock = clock.clone();
        let camera_background = run_startup.then(|| {
            thread::spawn(move || {
                while !camera_stop.load(Ordering::Relaxed) {
                    bg_camera_runtime.tick(camera_clock.mono());
                    thread::sleep(Duration::from_millis(20));
                }
            })
        });
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
                    let _ = bg_follow.tick();
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
        let render_background = if run_startup {
            let worker_stop = stop.clone();
            let worker_jobs = render_jobs.clone();
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
        camera_runtime.bind(AppBinding {
            store: store.clone(),
            events: events.clone(),
            health: health.clone(),
            previous_health,
            provider: camera.clone(),
            paths: paths.clone(),
            camera_settings: settings.camera,
            capture_root: settings.effective_document_id(&paths),
            editing: editing.clone(),
            scripting: scripting.clone(),
            provision: provision.clone(),
            document_id: settings
                .effective_document_id(&paths)
                .to_string_lossy()
                .into_owned(),
            gate: lanes.gate.clone(),
        });
        Ok(Self {
            paths,
            settings,
            provision,
            store,
            spool,
            nudge,
            hooks_compiled,
            health,
            editing,
            camera,
            camera_runtime,
            commerce,
            jobs,
            publish_jobs,
            render_jobs,
            arrival,
            previews,
            follow,
            compositions,
            export,
            publish,
            events,
            history,
            clock,
            scripting,
            lanes,
            run_startup,
            commerce_override,
            stop,
            background: Some(background),
            camera_background,
            publish_background,
            render_background,
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
            provision: &self.provision,
            camera: Some(self.camera.as_ref()),
            follow: Some(self.follow.as_ref()),
        })
        .run()
    }

    /// The user-lane scripting (gated; R-I32's check applies to it).
    pub fn user_lane(&self) -> Arc<dyn CaptureOneScripting> {
        self.lanes.user.clone()
    }
    /// The process-wide editor-launch gate (R-I32).
    pub fn launch_gate(&self) -> Arc<LaunchGate> {
        self.lanes.gate.clone()
    }
    /// The process-wide start order (R-I31).
    pub fn start_order(&self) -> Arc<StartOrder> {
        self.lanes.start_order.clone()
    }
    /// The master-render disk guard's free-space source (P56 C3(d); a test stands in for the disk).
    pub fn disk_space(&self) -> Arc<editing::capture_one::adapter::DiskSpace> {
        self.lanes.disk.clone()
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
        if let Some(bg) = self.camera_background.take() {
            let _ = bg.join();
        }
        if let Some(bg) = self.publish_background.take() {
            let _ = bg.join();
        }
        if let Some(bg) = self.render_background.take() {
            let _ = bg.join();
        }
    }

    /// Rebuilds the runtime over the current `settings.json`, keeping the event target.
    pub fn reopen(&mut self) -> Result<()> {
        let next = Self::build(
            self.paths.clone(),
            self.scripting.clone(),
            self.lanes.clone(),
            self.clock.clone(),
            self.run_startup,
            self.commerce_override.clone(),
            false,
            Some(self.history.store()),
            self.camera_runtime.clone(),
            Some(self.health.clone()),
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
    events: Arc<dyn EventSink>,
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
        )?;
        if let Some(frame) = extracted.and_then(|e| e.compose_frame) {
            self.store.adopt_frame(
                &job.target,
                Fingerprint {
                    frame,
                    source: FrameSource::Camera,
                    geometry: EditorGeometry::default(),
                },
            )?;
        }
        for photo in self.store.photos_for_raw(&job.target)? {
            self.events.emit(AppEvent::PhotoChanged {
                photo_id: photo.id,
                session_id: photo.session_id,
            });
        }
        Ok(())
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
/// process, so the run (and its `render_master` tracking rows) is abandoned (the user exports
/// again); its rows must not block the queue.
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
