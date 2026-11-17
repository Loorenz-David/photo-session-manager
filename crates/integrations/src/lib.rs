pub mod arranged;
pub mod camera;
pub mod commerce;
pub mod editing;
pub mod render_output;

use psm_research::api::ResearchApi;

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
        prepare_masters::PrepareMastersService,
        previews::PreviewService,
        publish::PublishService,
        settings::SettingsService,
        storage::{
            CopiedExports, PREVIEW_EXPORT_FOLDER, RetentionConfig, StorageLocations, StorageService,
        },
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
    storage_fs::LocalStorageFs,
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
/// How often the research runner looks for work (R7·U1).
const RESEARCH_WAKE: Duration = Duration::from_millis(250);
/// How long shutdown waits for the spool/arrival thread (P63).
const BACKGROUND_JOIN_BOUND: Duration = Duration::from_secs(1);

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
    pub research: Option<Arc<dyn ResearchApi>>,
    /// R7·U1: the research runner (its own jobs, its own thread).
    pub research_runner: Option<Arc<ResearchWiring>>,
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
    /// 10F part 1: duplicates, preview exports and masters cleared in the background (P61).
    pub storage: Arc<StorageService>,
    /// Changelog 47 (2), P69: masters of Ready sessions prepared in the background.
    pub prepare_masters: Arc<PrepareMastersService>,
    scripting: Arc<dyn CaptureOneScripting>,
    lanes: Lanes,
    run_startup: bool,
    commerce_override: Option<Arc<dyn CommerceAdapter>>,
    stop: Arc<AtomicBool>,
    background: Option<thread::JoinHandle<()>>,
    camera_background: Option<thread::JoinHandle<()>>,
    publish_background: Option<thread::JoinHandle<()>>,
    render_background: Option<thread::JoinHandle<()>>,
    storage_background: Option<thread::JoinHandle<()>>,
    prepare_background: Option<thread::JoinHandle<()>>,
    research_background: Option<thread::JoinHandle<()>>,
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
    pub research: Option<Arc<dyn ResearchApi>>,
    pub arrival: Arc<ArrivalService>,
    pub previews: Arc<PreviewService>,
    pub follow: Arc<CaptureOneFollow>,
    pub compositions: Arc<CompositionService>,
    pub export: Arc<ExportService>,
    pub publish: Option<Arc<PublishService>>,
    pub events: Arc<EventHub>,
    pub history: Arc<History>,
    pub storage: Arc<StorageService>,
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
    research: Option<Arc<dyn ResearchApi>>,
}

pub enum PreparedSettings {
    Simple(Box<Settings>),
    Runtime(Box<App>),
}

impl SettingsSetContext {
    pub fn prepare(self, patch: &serde_json::Value) -> Result<PreparedSettings> {
        // P65: the stock default is written by the publish (after a create) and by its own
        // command; the file holds its current value.
        let mut current = self.settings.clone();
        if let Ok(on_disk) = Settings::open(&self.paths) {
            current.commerce = on_disk.commerce;
            // R7·U1: the engine reads the `research` slot live from the file; it is the current.
            current.research = on_disk.research;
        }
        let previous = settings_value(&current)?;
        // R7·U1: a `research` patch passes the engine's validation first; its text is the refusal.
        if let Some(research) = patch.get("research") {
            match &self.research {
                Some(api) => api.validate_settings(research)?,
                None => psm_research::config::validate(research)
                    .map_err(psm_core::Error::InvalidData)?,
            }
        }
        let mut next = SettingsService::set(&previous, patch)?;
        if patch.get("research").is_some() {
            next["research"] = serde_json::to_value(
                psm_research::config::ResearchSettings::from_slot(next.get("research")),
            )
            .map_err(|e| psm_core::Error::InvalidValue(e.to_string()))?;
        }
        // P68: the shop domain and keys as the photographer meant them.
        if patch.get("integration").is_some_and(|v| v.is_object())
            && let Some(integration) = next.get_mut("integration").and_then(|v| v.as_object_mut())
        {
            if let Some(domain) = integration.get("shop_domain").and_then(|v| v.as_str()) {
                let domain = commerce::shopify::adapter::normalise_shop_domain(domain);
                integration.insert("shop_domain".into(), domain.into());
            }
            for key in ["client_id", "client_secret"] {
                if let Some(value) = integration.get(key).and_then(|v| v.as_str()) {
                    let value = value.trim().to_owned();
                    integration.insert(key.into(), value.into());
                }
            }
        }
        let settings: Settings = serde_json::from_value(next.clone())
            .map_err(|e| psm_core::Error::InvalidValue(e.to_string()))?;
        if let Some(integration) = settings.integration_settings()? {
            psm_infra::logging::add_secret(&integration.client_secret);
        }
        Settings::set(&self.paths, &settings)?;
        let mut a = previous;
        let mut b = next;
        for key in ["theme", "output_format", "storage", "export"] {
            a[key] = serde_json::Value::Null;
            b[key] = serde_json::Value::Null;
        }
        a["camera"]["auto_take_back"] = serde_json::Value::Null;
        b["camera"]["auto_take_back"] = serde_json::Value::Null;
        // R7·U1: research settings are read live by the engine; only a change of the chosen
        // adapters needs the runtime rebuilt (they are built with it).
        for value in [&mut a, &mut b] {
            let providers = value
                .get("research")
                .and_then(|r| r.get("providers"))
                .cloned();
            value["research"] = providers.unwrap_or(serde_json::Value::Null);
        }
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
            research: self.research.clone(),
        }
    }
    pub fn apply_settings(
        &mut self,
        prepared: PreparedSettings,
    ) -> Result<(serde_json::Value, Option<App>)> {
        let old = match prepared {
            PreparedSettings::Simple(settings) => {
                let min_free = settings.storage.min_free_bytes();
                if min_free != self.storage.min_free_bytes() {
                    self.storage.set_min_free_bytes(min_free);
                }
                let days = settings.storage.retention_days_after_publish;
                if days != self.storage.retention_days() {
                    self.storage.set_retention_days(days);
                }
                self.prepare_masters
                    .set_enabled(settings.export.prepare_masters);
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
        // R7·U1: a saved research setting is evaluated by the engine (`auto` off cancels queued
        // automatic runs, R-IA25).
        if let Some(research) = &self.research {
            research.settings_changed();
        }
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
            research: self.research.clone(),
            arrival: self.arrival.clone(),
            previews: self.previews.clone(),
            follow: self.follow.clone(),
            compositions: self.compositions.clone(),
            export: self.export.clone(),
            publish: self.publish.clone(),
            events: self.events.clone(),
            history: self.history.clone(),
            storage: self.storage.clone(),
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
            .with_disk(lanes.disk.clone())
            .with_configured_document(
                settings
                    .document_id
                    .as_deref()
                    .is_some_and(|id| !id.is_empty()),
            ),
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
            let service = Arc::new(PublishService::new(
                store.clone(),
                adapter.clone(),
                export.clone(),
                clock.clone(),
                events.clone(),
            ));
            // Changelog 45 (P65): `commerce.inventory_default` in settings.json.
            service.set_stock_defaults(Arc::new(SettingsStockDefaults::new(paths.clone())));
            service
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
        let copied_exports = Arc::new(CopiedExports::default());
        previews.copied_exports = Some(copied_exports.clone());
        let previews = Arc::new(previews);
        // P60 #22: while "Take the camera back" holds Capture One no render request leaves.
        let render_gate = lanes.gate.clone();
        previews.set_render_hold(Arc::new(move || render_gate.is_held()));
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
        let mut runner = JobRunner::new(store.clone()).excluding_kinds(&[
            "publish_step",
            "render_output",
            psm_research::job::KIND,
        ]);
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
        let stop = Arc::new(AtomicBool::new(false));
        // R7·U1: the research engine. A failure leaves the app without research, never without
        // its core workflow (HA4).
        let research_wiring = match build_research(ResearchParts {
            paths: &paths,
            settings: &settings,
            store: &store,
            clock: &clock,
            imaging: &imaging,
            secrets: camera_runtime.secret_store(),
            commerce: &commerce,
            client_secret: integration.as_ref().map(|i| i.client_secret.clone()),
            events: &events,
        }) {
            Ok(wiring) => Some(wiring),
            Err(error) => {
                tracing::warn!(%error, "research not available");
                None
            }
        };
        let research_background = match (&research_wiring, run_startup) {
            (Some(wiring), true) => {
                let worker_stop = stop.clone();
                let worker = wiring.clone();
                let worker_clock = clock.clone();
                Some(thread::spawn(move || {
                    while !worker_stop.load(Ordering::Relaxed) {
                        if let Err(error) = worker.tick(worker_clock.now_ms()) {
                            tracing::warn!(%error, "research turn deferred");
                        }
                        thread::sleep(RESEARCH_WAKE);
                    }
                }))
            }
            _ => None,
        };
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
        let storage = {
            let document = settings.effective_document_id(&paths);
            let disk = lanes.disk.clone();
            let editing = editing.clone();
            let gate = lanes.gate.clone();
            let runtime = camera_runtime.clone();
            let running_export = export.clone();
            let publish_store = store.clone();
            Arc::new(
                StorageService::new(
                    store.clone(),
                    Arc::new(LocalStorageFs::new(Arc::new(
                        move |path: &std::path::Path| disk.query(path),
                    ))),
                    clock.clone(),
                    health.clone(),
                    events.clone(),
                    export.clone(),
                    StorageLocations {
                        capture_root: document.join("Capture"),
                        preview_exports: document.join("Output").join(PREVIEW_EXPORT_FOLDER),
                        preview_cache: paths.root.join("previews"),
                        masters: paths.root.join("masters"),
                        outputs: paths.root.join("outputs"),
                    },
                    // 10F §6: never while a capture, an upload, a render, a take-back, an export
                    // or a publish is running.
                    Arc::new(move || {
                        let reason = if runtime.capture_in_progress() {
                            "capture"
                        } else if runtime.uploads_in_progress() > 0 {
                            "upload"
                        } else if editing.renders_in_flight() > 0 {
                            "render"
                        } else if gate.is_held() {
                            "take-back"
                        } else if running_export.running() {
                            "export"
                        } else if publish_store
                            .active_publish_items()
                            .map_or(true, |items| !items.is_empty())
                        {
                            "publish"
                        } else {
                            return None;
                        };
                        Some(reason.to_owned())
                    }),
                    copied_exports,
                    settings.storage.min_free_bytes(),
                )
                // P62 (H6): retention only with a commerce integration to verify against.
                .with_retention(
                    commerce
                        .clone()
                        .map(|commerce| RetentionConfig { commerce }),
                    settings.storage.retention_days_after_publish,
                )
                // P66: a denied photo's undo steps are dropped when its files go.
                .with_history(history.clone()),
            )
        };
        // Changelog 47 (2), P69: masters prepared once a session is Ready, one at a time, only
        // while Capture One is idle; never during a capture, a take-back, or below the minimum
        // free space (the budget would clear them again).
        let prepare_masters = {
            let runtime = camera_runtime.clone();
            let gate = lanes.gate.clone();
            let disk = lanes.disk.clone();
            let budget = storage.clone();
            let document = settings.effective_document_id(&paths);
            Arc::new(PrepareMastersService::new(
                store.clone(),
                editing.clone(),
                export.clone(),
                previews.clone(),
                clock.clone(),
                Arc::new(move || {
                    let reason =
                        if runtime.capture_in_progress() || runtime.uploads_in_progress() > 0 {
                            "capture"
                        } else if gate.is_held() {
                            "take-back"
                        } else if disk
                            .query(&document)
                            .is_ok_and(|space| space.available_bytes < budget.min_free_bytes())
                        {
                            "disk"
                        } else {
                            return None;
                        };
                    Some(reason.to_owned())
                }),
                settings.export.prepare_masters,
            ))
        };
        let prepare_background = if run_startup {
            let worker_stop = stop.clone();
            let worker = prepare_masters.clone();
            Some(thread::spawn(move || {
                let mut last = std::time::Instant::now() - Duration::from_secs(2);
                while !worker_stop.load(Ordering::Relaxed) {
                    if last.elapsed() >= Duration::from_secs(1) {
                        last = std::time::Instant::now();
                        if let Err(error) = worker.tick() {
                            tracing::warn!(%error, "master preparation deferred");
                        }
                    }
                    thread::sleep(Duration::from_millis(250));
                }
            }))
        } else {
            None
        };
        let storage_background = if run_startup {
            let worker_stop = stop.clone();
            let worker = storage.clone();
            Some(thread::spawn(move || {
                while !worker_stop.load(Ordering::Relaxed) {
                    if let Err(error) = worker.tick() {
                        tracing::warn!(%error, "storage clean-up deferred");
                    }
                    thread::sleep(Duration::from_millis(250));
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
            document_configured: settings
                .document_id
                .as_deref()
                .is_some_and(|id| !id.is_empty()),
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
            research: research_wiring
                .as_ref()
                .map(|wiring| wiring.service.clone() as Arc<dyn ResearchApi>),
            research_runner: research_wiring,
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
            storage,
            prepare_masters,
            scripting,
            lanes,
            run_startup,
            commerce_override,
            stop,
            background: Some(background),
            camera_background,
            publish_background,
            render_background,
            storage_background,
            prepare_background,
            research_background,
        })
    }
    /// One turn of the research runner (what its thread does; `false` while a turn is running).
    pub fn research_tick(&self, now_ms: i64) -> Result<bool> {
        match &self.research_runner {
            Some(runner) => runner.tick(now_ms),
            None => Ok(false),
        }
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
            // P63 (backlog P62 r3): starting a macOS folder watcher can take tens of seconds; a
            // thread still inside it leaves on its own (it checks `stop` first), so shutdown does
            // not wait for it longer than `BACKGROUND_JOIN_BOUND`.
            let end = std::time::Instant::now() + BACKGROUND_JOIN_BOUND;
            while !bg.is_finished() && std::time::Instant::now() < end {
                thread::sleep(Duration::from_millis(5));
            }
            if bg.is_finished() {
                let _ = bg.join();
            } else {
                tracing::info!(
                    "background thread still starting its folder watcher; not waited for"
                );
            }
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
        if let Some(bg) = self.storage_background.take() {
            let _ = bg.join();
        }
        if let Some(bg) = self.prepare_background.take() {
            let _ = bg.join();
        }
        if let Some(bg) = self.research_background.take() {
            // MC-A6 Quit: a run inside a provider call is not waited for; it stays `running` and
            // the next launch re-queues its job.
            let end = std::time::Instant::now() + BACKGROUND_JOIN_BOUND;
            while !bg.is_finished() && std::time::Instant::now() < end {
                thread::sleep(Duration::from_millis(5));
            }
            if bg.is_finished() {
                let _ = bg.join();
            }
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
    /// Sinks inside the app that hear every event as it is emitted (R7·U1: research).
    observers: Vec<Arc<dyn EventSink>>,
}
impl EventHub {
    pub fn transfer_target_to(&self, other: &EventHub) {
        if let Ok(state) = self.state.lock()
            && let Some(target) = state.target.clone()
        {
            other.set_target(target);
        }
    }
    /// Registers a sink that hears every event when it is emitted (before the target).
    pub fn observe(&self, observer: Arc<dyn EventSink>) {
        if let Ok(mut state) = self.state.lock() {
            state.observers.push(observer);
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
        let observers = self
            .state
            .lock()
            .map(|state| state.observers.clone())
            .unwrap_or_default();
        for observer in observers {
            observer.emit(event.clone());
        }
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

/// Changelog 45 (P65): the stock default for new items, `commerce.inventory_default` in
/// settings.json (read from the file each time; written in place, other keys unchanged).
pub struct SettingsStockDefaults {
    paths: AppPaths,
    write: std::sync::Mutex<()>,
}
impl SettingsStockDefaults {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            write: std::sync::Mutex::new(()),
        }
    }
}
impl psm_core::ports::commerce::StockDefaults for SettingsStockDefaults {
    fn get(&self) -> Result<Option<psm_core::domain::publish::StockChoice>> {
        Ok(Settings::open(&self.paths)?
            .commerce
            .inventory_default
            .map(|d| psm_core::domain::publish::StockChoice {
                location_id: d.location_id,
                location_name: d.location_name,
                quantity: d.quantity,
            }))
    }
    fn remember(&self, choice: &psm_core::domain::publish::StockChoice) -> Result<()> {
        let _write = self
            .write
            .lock()
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        let mut settings = Settings::open(&self.paths)?;
        settings.commerce.inventory_default = Some(psm_infra::settings_file::InventoryDefault {
            location_id: choice.location_id.clone(),
            location_name: choice.location_name.clone(),
            quantity: choice.quantity,
        });
        Settings::set(&self.paths, &settings)
    }
}
impl JobHandler for PublishJob {
    /// Changelog 45 (P65): a publish waiting for its export (or for Capture One) is looked at
    /// again at least every 15 s.
    fn max_backoff_ms(&self) -> Option<i64> {
        Some(15_000)
    }
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

/// R7·U1: the research engine joined to the app. H8: this only constructs and connects; every
/// research rule is in `psm_research`. One runner turn at a time (`busy`), so the engine's
/// `tick` is called from exactly one place (MC-A6: one running run app-wide; R4 review).
pub struct ResearchWiring {
    service: Arc<psm_research::service::ResearchService>,
    jobs: JobRunner,
    store: Arc<SqliteStore>,
    /// Sessions whose photos may have changed since the last turn: `true` = a photo event
    /// (created, changed, a source arrived), `false` = an undo step only (counted when it moved
    /// the session's `change_seq`, MC-6; an attribute edit does not).
    changed: std::sync::Mutex<std::collections::BTreeMap<i64, bool>>,
    /// The last `change_seq` seen per session.
    seen: std::sync::Mutex<std::collections::HashMap<i64, i64>>,
    busy: std::sync::Mutex<()>,
    /// When call records were last purged (MC-A11: at launch, then hourly; UTC ms).
    last_purge: std::sync::Mutex<i64>,
}

/// MC-A11: call records are purged at launch and then every hour.
const RESEARCH_PURGE_EVERY_MS: i64 = 3_600_000;

impl ResearchWiring {
    /// MC-A11 retention, at launch and when an hour has passed since the last purge.
    fn purge_if_due(&self, now_ms: i64, at_launch: bool) {
        let Ok(mut last) = self.last_purge.lock() else {
            return;
        };
        if !at_launch && now_ms.saturating_sub(*last) < RESEARCH_PURGE_EVERY_MS {
            return;
        }
        *last = now_ms;
        if let Err(error) = self.service.purge_expired(now_ms) {
            tracing::warn!(%error, "research call records not purged");
        }
    }

    /// One turn of the research runner: the engine hears of the sessions that changed (outside
    /// every other service's transaction), starts what may start, then runs the due research
    /// jobs. `Ok(false)` when another turn is in progress (a wake-up never runs a second one).
    pub fn tick(&self, now_ms: i64) -> Result<bool> {
        let _turn = match self.busy.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Ok(false),
        };
        let changed = std::mem::take(
            &mut *self
                .changed
                .lock()
                .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?,
        );
        for (session_id, photo_event) in changed {
            let seq = self
                .store
                .session(session_id)
                .map(|s| s.session.change_seq)
                .ok();
            let moved = {
                let mut seen = self
                    .seen
                    .lock()
                    .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
                match seq {
                    Some(seq) => seen.insert(session_id, seq) != Some(seq),
                    None => false,
                }
            };
            if photo_event || moved {
                self.service.session_changed(session_id);
            }
        }
        self.purge_if_due(now_ms, false);
        self.service.tick()?;
        self.jobs.tick(now_ms)?;
        Ok(true)
    }

    fn note(&self, session_id: i64, photo_event: bool) {
        if let Ok(mut changed) = self.changed.lock() {
            let entry = changed.entry(session_id).or_insert(false);
            *entry |= photo_event;
        }
    }
}

/// Hears the app's events and notes the sessions whose photos may have changed (MC-A6 "a
/// change"); the research runner's next turn tells the engine.
struct ResearchObserver(std::sync::Weak<ResearchWiring>);

impl EventSink for ResearchObserver {
    fn emit(&self, event: AppEvent) {
        let (session_id, photo_event) = match event {
            AppEvent::PhotoAdded { session_id, .. }
            | AppEvent::PhotoChanged { session_id, .. }
            | AppEvent::SessionChanged { session_id } => (session_id, true),
            AppEvent::HistoryChanged { session_id, .. } => (session_id, false),
            _ => return,
        };
        if let Some(wiring) = self.0.upgrade() {
            wiring.note(session_id, photo_event);
        }
    }
}

/// What a test puts in place of the research adapters and the real-time wait between attempts
/// (as `camera::support::with_test_manifest`, keyed by the data folder; survives a reopen).
#[derive(Clone)]
pub struct ResearchTestSeams {
    pub providers: psm_research::ports::Providers,
    pub wait: psm_research::gateway::Waiter,
}

static RESEARCH_TEST_SEAMS: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<std::path::PathBuf, ResearchTestSeams>>,
> = std::sync::OnceLock::new();

pub fn with_test_research(paths: &AppPaths, seams: ResearchTestSeams) {
    if let Ok(mut all) = RESEARCH_TEST_SEAMS.get_or_init(Default::default).lock() {
        all.insert(paths.root.clone(), seams);
    }
}

fn research_test_seams(paths: &AppPaths) -> Option<ResearchTestSeams> {
    RESEARCH_TEST_SEAMS
        .get()
        .and_then(|all| all.lock().ok()?.get(&paths.root).cloned())
}

/// Every secret the secrets port holds for the app (the research keys, the camera's passwords) and
/// the integration's client secret: what the redaction scan and the export check look for
/// (MC-A9, MC-A11). The camera's access user name is held there too but is a name, not a secret:
/// it is left out, since a short everyday word would be "found" in page text and image bytes
/// (owner correction A7, 2026-11-17).
fn research_secret_values(
    secrets: Arc<dyn SecretStore>,
    client_secret: Option<String>,
) -> psm_research::gateway::SecretValues {
    use psm_core::domain::camera::secrets as camera;
    use psm_research::service::{MODEL_ACCOUNT, READER_ACCOUNT, SEARCH_ACCOUNT};
    Arc::new(move || {
        [
            MODEL_ACCOUNT,
            SEARCH_ACCOUNT,
            READER_ACCOUNT,
            camera::FTP_PASSWORD,
            camera::ACCESS_AUTH_PASSWORD,
        ]
        .into_iter()
        .filter_map(|account| secrets.get(account).ok().flatten())
        .chain(client_secret.clone())
        .filter(|value| !value.trim().is_empty())
        .collect()
    })
}

/// The pieces `build_research` needs from `App::build`.
struct ResearchParts<'a> {
    paths: &'a AppPaths,
    settings: &'a Settings,
    store: &'a Arc<SqliteStore>,
    clock: &'a Arc<dyn Clock>,
    imaging: &'a Arc<dyn psm_core::ports::images::Imaging>,
    secrets: Arc<dyn SecretStore>,
    commerce: &'a Option<Arc<dyn CommerceAdapter>>,
    client_secret: Option<String>,
    events: &'a Arc<EventHub>,
}

/// R7·U1 task 1: the engine's tables, the service over the app's store, clock, imaging, secrets,
/// shop and events, its dedicated runner, and the observer on the app's events.
fn build_research(parts: ResearchParts<'_>) -> Result<Arc<ResearchWiring>> {
    use psm_research::{
        config::ResearchSettings,
        service::{ResearchDeps, ResearchService},
        shop::CommerceShopFacts,
    };
    psm_research::schema::ensure(parts.store)?;
    let seams = research_test_seams(parts.paths);
    let wait: psm_research::gateway::Waiter = match &seams {
        Some(seams) => seams.wait.clone(),
        None => Arc::new(|ms| thread::sleep(Duration::from_millis(ms))),
    };
    let providers = match &seams {
        Some(seams) => seams.providers.clone(),
        None => psm_research::adapters::build(
            &ResearchSettings::from_slot(parts.settings.research.as_ref()),
            parts.secrets.clone(),
            psm_research::files::ResearchFiles::new(&parts.paths.root),
            wait.clone(),
        ),
    };
    let settings_paths = parts.paths.clone();
    let service = Arc::new(ResearchService::new(ResearchDeps {
        store: parts.store.clone(),
        clock: parts.clock.clone(),
        imaging: parts.imaging.clone(),
        data_dir: parts.paths.root.clone(),
        providers,
        secret_values: research_secret_values(parts.secrets.clone(), parts.client_secret),
        secrets: parts.secrets,
        shop: parts
            .commerce
            .clone()
            .map(|adapter| Arc::new(CommerceShopFacts::new(adapter))),
        // The only commerce adapter is Shopify's; its definitions are the form's.
        integration: parts
            .commerce
            .is_some()
            .then(|| commerce::shopify::adapter::INTEGRATION.to_owned()),
        settings: Arc::new(move || {
            Settings::open(&settings_paths)
                .ok()
                .and_then(|settings| settings.research)
        }),
        events: parts.events.clone(),
        wait,
    }));
    let mut jobs = JobRunner::new(parts.store.clone()).only_kind(psm_research::job::KIND);
    jobs.register(
        psm_research::job::KIND,
        Arc::new(psm_research::job::ResearchJob::new(service.clone())),
    );
    let wiring = Arc::new(ResearchWiring {
        service,
        jobs,
        store: parts.store.clone(),
        changed: Default::default(),
        seen: Default::default(),
        busy: std::sync::Mutex::new(()),
        last_purge: std::sync::Mutex::new(parts.clock.now_ms()),
    });
    wiring.purge_if_due(parts.clock.now_ms(), true);
    parts
        .events
        .observe(Arc::new(ResearchObserver(Arc::downgrade(&wiring))));
    Ok(wiring)
}

#[cfg(test)]
mod research_secret_tests {
    use super::*;
    use psm_core::domain::camera::secrets as camera;
    use std::{collections::HashMap, sync::Mutex};

    #[derive(Default)]
    struct Store(Mutex<HashMap<String, String>>);
    impl SecretStore for Store {
        fn get(&self, account: &str) -> psm_core::Result<Option<String>> {
            Ok(self.0.lock().unwrap().get(account).cloned())
        }
        fn set(&self, account: &str, value: &str) -> psm_core::Result<()> {
            self.0
                .lock()
                .unwrap()
                .insert(account.to_owned(), value.to_owned());
            Ok(())
        }
        fn delete(&self, account: &str) -> psm_core::Result<()> {
            self.0.lock().unwrap().remove(account);
            Ok(())
        }
    }

    #[test]
    fn a7_the_camera_user_name_is_not_a_secret_but_its_passwords_are() {
        let store = Arc::new(Store::default());
        store.set(camera::ACCESS_AUTH_USER, "admin").unwrap();
        store
            .set(camera::ACCESS_AUTH_PASSWORD, "pw-access-1")
            .unwrap();
        store.set(camera::FTP_PASSWORD, "pw-ftp-2").unwrap();
        store
            .set(psm_research::service::MODEL_ACCOUNT, "sk-model-3")
            .unwrap();
        let values = research_secret_values(store, Some("client-4".into()))();
        assert!(!values.contains(&"admin".to_owned()), "{values:?}");
        for expected in ["pw-access-1", "pw-ftp-2", "sk-model-3", "client-4"] {
            assert!(values.contains(&expected.to_owned()), "{values:?}");
        }
    }
}
