use super::{
    direct::{
        DirectCameraAdapter, SLEEP_REASON, StartOrder, StartReason, TakeBackService,
        adapter::{HelperEvent, StepOutcome},
    },
    sidecar::{
        client::{CmdId, CommandOutcome, SidecarClient},
        protocol::{Command, SidecarEvent, Stamped, Transport},
        supervisor::{SidecarSpec, Supervisor, SupervisorEvent},
    },
    support::installed_sidecar,
};
use crate::editing::capture_one::{launch_gate::LaunchGate, scripting::CaptureOneScripting};
use psm_core::{
    domain::camera::{CameraSettings, Delivery, DeliveryOutcome, ExposureObserved, Link, secrets},
    domain::editing::ProvisionInputs,
    events::AppEvent,
    ports::{
        camera::{CameraProvider, CaptureSink, DeliveryFs},
        clock::Clock,
        editing::EditingProvider,
        secrets::SecretStore,
        store::Store,
        system::{ProcessList, SleepInhibitor},
    },
    services::{
        camera_ledger::{LedgerService, LedgerShared},
        health::HealthService,
    },
};
use psm_infra::{
    camera_files::CameraFiles,
    ftp::{CompletedNames, FTP_USERNAME, FtpReceiver, ReceiverConfig, ReceiverHandle},
    logging,
    paths::AppPaths,
    processes::PsProcessList,
    sleep::IokitSleepInhibitor,
};
use std::{
    collections::VecDeque,
    fs,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    sync::{
        Arc, Mutex, OnceLock, RwLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use uuid::Uuid;

/// The sidecar events `sidecar_events()` keeps (P45; also what the Health page shows).
pub const SIDECAR_EVENTS_KEPT: usize = 1_000;
/// Stderr lines (redacted) and unparsable stdout lines `helper_log()` keeps.
const HELPER_LOG_KEPT: usize = 50;
/// The camera's USB product ids for the sidecar's IOKit match (R-I23): the A7R V.
const USB_PRODUCT_IDS: &str = "0x0E0C";

pub struct AppBinding {
    pub store: Arc<dyn Store>,
    pub events: Arc<crate::EventHub>,
    pub health: Arc<HealthService>,
    pub previous_health: Option<Arc<HealthService>>,
    pub provider: Arc<dyn CameraProvider>,
    pub paths: AppPaths,
    pub camera_settings: CameraSettings,
    pub capture_root: std::path::PathBuf,
    /// What "Take the camera back" needs from this build (P48 L14): the editing provider, the
    /// ungated scripting, the provision inputs, the document and the process-wide launch gate.
    pub editing: Arc<dyn EditingProvider>,
    pub scripting: Arc<dyn CaptureOneScripting>,
    pub provision: ProvisionInputs,
    pub document_id: String,
    pub gate: Arc<LaunchGate>,
}
struct RoutingSink(RwLock<Arc<dyn CaptureSink>>);
impl CaptureSink for RoutingSink {
    fn exposure_observed(&self, e: ExposureObserved) -> psm_core::Result<i64> {
        self.0.read().unwrap().exposure_observed(e)
    }
    fn delivery(&self, d: Delivery) -> psm_core::Result<DeliveryOutcome> {
        self.0.read().unwrap().delivery(d)
    }
    fn session_uncertain(&self) -> psm_core::Result<()> {
        self.0.read().unwrap().session_uncertain()
    }
    fn transfer_in_progress(&self, source: Link, active: bool) {
        self.0.read().unwrap().transfer_in_progress(source, active);
    }
}
/// Before the first bind there is no ledger: a delivery is refused (the receiver answers 426 and
/// the camera re-sends) and nothing is listed.
struct Unbound;
impl CaptureSink for Unbound {
    fn exposure_observed(&self, _: ExposureObserved) -> psm_core::Result<i64> {
        Err(psm_core::Error::Provider("camera ledger not bound".into()))
    }
    fn delivery(&self, _: Delivery) -> psm_core::Result<DeliveryOutcome> {
        Err(psm_core::Error::Provider("camera ledger not bound".into()))
    }
    fn session_uncertain(&self) -> psm_core::Result<()> {
        Err(psm_core::Error::Provider("camera ledger not bound".into()))
    }
    fn transfer_in_progress(&self, _: Link, _: bool) {}
}
impl CompletedNames for Unbound {
    fn completed_names_since(&self, _: i64) -> Vec<(String, u64, i64)> {
        vec![]
    }
}
struct RoutingNames(RwLock<Arc<dyn CompletedNames>>);
impl CompletedNames for RoutingNames {
    fn completed_names_since(&self, since: i64) -> Vec<(String, u64, i64)> {
        self.0.read().unwrap().completed_names_since(since)
    }
}
pub struct CameraRuntime {
    pub run_id: String,
    /// The sidecar protocol client over its supervisor (P45); one per process (R-I18).
    supervisor: Mutex<Option<SidecarClient>>,
    /// The last `SIDECAR_EVENTS_KEPT` events delivered by the client.
    sidecar_events: Mutex<VecDeque<Stamped>>,
    receiver: Mutex<Option<ReceiverHandle>>,
    receiver_settings: Mutex<Option<CameraSettings>>,
    receiver_generation: std::sync::atomic::AtomicU64,
    receiver_bind: IpAddr,
    capture_root: Arc<RwLock<std::path::PathBuf>>,
    sink: Arc<RoutingSink>,
    names: Arc<RoutingNames>,
    /// One per process (R-I18; P44 L3): the ledger worker lock and the watchdog's raised flag,
    /// shared by every `LedgerService` a bind builds.
    ledger_shared: Arc<LedgerShared>,
    ledger: RwLock<Option<Arc<LedgerService>>>,
    delivery_fs: RwLock<Arc<dyn DeliveryFs>>,
    binding: RwLock<Option<AppBinding>>,
    secrets: Arc<dyn SecretStore>,
    credentials: Mutex<Option<(String, String)>>,
    spawned_credentials: Mutex<Option<String>>,
    clock: Arc<dyn Clock>,
    paths: AppPaths,
    helper_log: Mutex<VecDeque<String>>,
    stopped: AtomicBool,
    /// The direct camera adapter, created once per runtime (P46 L18) so its status inputs
    /// survive a settings save.
    direct: OnceLock<Arc<DirectCameraAdapter>>,
    /// Serialises `tick` (two background loops run during a rebuild) without holding the
    /// supervisor lock across the adapter step (a download's hash, P46 L19).
    tick_serial: Mutex<()>,
    /// The supervisor has a live child (`Started` … `Exited`).
    helper_running: AtomicBool,
    /// When the watchdog glue last ran (once per mono second).
    watchdog_at: Mutex<Option<Instant>>,
    /// The pending count of the last watchdog run (the status mirrors it while raised).
    watchdog_pending: AtomicUsize,
    /// The app-start launch rule (R-I31), one per process (P48 L14).
    start_order: Arc<StartOrder>,
    /// Who could hold the camera (R-I30); `set_process_list` is the test seam.
    process_list: RwLock<Arc<dyn ProcessList>>,
    /// The idle-sleep assertion (R-I23); `set_sleep_inhibitor` is the test seam.
    sleep: RwLock<Arc<dyn SleepInhibitor>>,
    /// "Take the camera back" is running (one at a time per process).
    take_back_running: Arc<AtomicBool>,
    bundled_sidecar: RwLock<Option<PathBuf>>,
    staging_dir: Mutex<Option<PathBuf>>,
}
impl CameraRuntime {
    pub fn new(secrets: Arc<dyn SecretStore>, clock: Arc<dyn Clock>, paths: AppPaths) -> Arc<Self> {
        let start_order = StartOrder::new(super::support::installed(&paths).is_some());
        Self::new_with_receiver_bind(
            secrets,
            clock,
            paths,
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            start_order,
        )
    }
    pub fn new_with_receiver_bind(
        secrets: Arc<dyn SecretStore>,
        clock: Arc<dyn Clock>,
        paths: AppPaths,
        receiver_bind: IpAddr,
        start_order: Arc<StartOrder>,
    ) -> Arc<Self> {
        let runtime = Arc::new(Self {
            run_id: Uuid::new_v4().to_string(),
            supervisor: Mutex::new(None),
            sidecar_events: Mutex::new(VecDeque::new()),
            receiver: Mutex::new(None),
            receiver_settings: Mutex::new(None),
            receiver_generation: std::sync::atomic::AtomicU64::new(0),
            receiver_bind,
            capture_root: Arc::new(RwLock::new(paths.root.clone())),
            sink: Arc::new(RoutingSink(RwLock::new(Arc::new(Unbound)))),
            names: Arc::new(RoutingNames(RwLock::new(Arc::new(Unbound)))),
            ledger_shared: Arc::new(LedgerShared::default()),
            ledger: RwLock::new(None),
            delivery_fs: RwLock::new(Arc::new(CameraFiles)),
            binding: RwLock::new(None),
            secrets,
            credentials: Mutex::new(None),
            spawned_credentials: Mutex::new(None),
            clock,
            paths,
            helper_log: Mutex::new(VecDeque::with_capacity(HELPER_LOG_KEPT)),
            stopped: AtomicBool::new(false),
            direct: OnceLock::new(),
            tick_serial: Mutex::new(()),
            helper_running: AtomicBool::new(false),
            watchdog_at: Mutex::new(None),
            watchdog_pending: AtomicUsize::new(0),
            start_order,
            process_list: RwLock::new(Arc::new(PsProcessList)),
            sleep: RwLock::new(Arc::new(IokitSleepInhibitor::default())),
            take_back_running: Arc::new(AtomicBool::new(false)),
            bundled_sidecar: RwLock::new(None),
            staging_dir: Mutex::new(None),
        });
        let weak = Arc::downgrade(&runtime);
        runtime.start_order.set_relaunch_retry(Arc::new(move || {
            if let Some(runtime) = weak.upgrade()
                && let Err(error) = runtime.send_command(Command::UsbRetry {
                    cmd_id: String::new(),
                })
            {
                tracing::warn!(%error, "relaunch wait: usb-retry not sent");
            }
        }));
        runtime
    }
    /// The process-wide start order (R-I31).
    pub fn start_order(&self) -> Arc<StartOrder> {
        self.start_order.clone()
    }
    /// The process list the adapter reads at each `usb.claimed_by` (R-I30).
    pub fn process_list(&self) -> Arc<dyn ProcessList> {
        self.process_list.read().unwrap().clone()
    }
    /// Test seam (set before the first tick, like `set_delivery_fs`).
    pub fn set_process_list(&self, list: Arc<dyn ProcessList>) {
        *self.process_list.write().unwrap() = list;
    }
    /// Test seam (set before the first tick).
    pub fn set_sleep_inhibitor(&self, sleep: Arc<dyn SleepInhibitor>) {
        *self.sleep.write().unwrap() = sleep;
    }
    /// "Take the camera back" over the current binding (R-I32; also "Test camera connection").
    pub fn take_back(self: &Arc<Self>) -> psm_core::Result<TakeBackService> {
        let binding = self.binding.read().expect("camera binding");
        let binding = binding
            .as_ref()
            .ok_or_else(|| psm_core::Error::Provider("camera runtime not bound".into()))?;
        Ok(TakeBackService {
            runtime: self.clone(),
            adapter: self.direct_adapter(),
            store: binding.store.clone(),
            health: binding.health.clone(),
            editing: binding.editing.clone(),
            camera: binding.provider.clone(),
            scripting: binding.scripting.clone(),
            gate: binding.gate.clone(),
            provision: binding.provision.clone(),
            document_id: binding.document_id.clone(),
            clock: self.clock.clone(),
            running: self.take_back_running.clone(),
        })
    }
    /// The direct camera adapter of this runtime: created at the first call, the same `Arc`
    /// afterwards (every later `App::build` and bind reuses it, P46 L18).
    pub fn direct_adapter(self: &Arc<Self>) -> Arc<DirectCameraAdapter> {
        self.direct
            .get_or_init(|| Arc::new(DirectCameraAdapter::new(Arc::downgrade(self))))
            .clone()
    }
    /// The camera provider of the current binding.
    pub fn provider(&self) -> Option<Arc<dyn CameraProvider>> {
        self.binding
            .read()
            .expect("camera binding")
            .as_ref()
            .map(|b| b.provider.clone())
    }
    /// The direct adapter when it is the current binding's provider.
    fn bound_direct(&self) -> Option<Arc<DirectCameraAdapter>> {
        let direct = self.direct.get()?;
        let provider = self.provider()?;
        (Arc::as_ptr(&provider) as *const () == Arc::as_ptr(direct) as *const ())
            .then(|| direct.clone())
    }
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
    pub fn set_capture_sink(&self, sink: Arc<dyn CaptureSink>, names: Arc<dyn CompletedNames>) {
        *self.sink.0.write().unwrap() = sink;
        *self.names.0.write().unwrap() = names;
    }
    /// The ledger of the current binding (rebuilt over the new store at every bind).
    pub fn ledger(&self) -> Option<Arc<LedgerService>> {
        self.ledger.read().unwrap().clone()
    }
    /// Test seam (P44 L11): the `DeliveryFs` the ledger uses, read at each bind; the current
    /// ledger is rebuilt over it at once (same store, same shared worker lock).
    pub fn set_delivery_fs(&self, fs: Arc<dyn DeliveryFs>) {
        *self.delivery_fs.write().unwrap() = fs;
        let current = self.ledger();
        if let Some(ledger) = current {
            self.install_ledger(LedgerService {
                store: ledger.store.clone(),
                clock: ledger.clock.clone(),
                fs: self.delivery_fs.read().unwrap().clone(),
                events: ledger.events.clone(),
                health: ledger.health.clone(),
                capture_root: ledger.capture_root.clone(),
                shared: self.ledger_shared.clone(),
            });
        }
    }
    fn install_ledger(&self, ledger: LedgerService) -> Arc<LedgerService> {
        let ledger = Arc::new(ledger);
        *self.ledger.write().unwrap() = Some(ledger.clone());
        self.set_capture_sink(ledger.clone(), ledger.clone());
        ledger
    }
    /// The capture root formula the receiver uses: `canon(<document>/Capture)`.
    fn capture_root_fn(&self) -> Arc<dyn Fn() -> Option<PathBuf> + Send + Sync> {
        let document = self.capture_root.clone();
        Arc::new(move || std::fs::canonicalize(document.read().unwrap().join("Capture")).ok())
    }
    pub fn receiver_port(&self) -> Option<u16> {
        self.receiver
            .lock()
            .unwrap()
            .as_ref()
            .map(ReceiverHandle::port)
    }
    pub fn uploads_in_progress(&self) -> usize {
        self.receiver
            .lock()
            .unwrap()
            .as_ref()
            .map(ReceiverHandle::uploads_in_progress)
            .unwrap_or(0)
    }
    pub fn receiver_generation(&self) -> u64 {
        self.receiver_generation.load(Ordering::SeqCst)
    }
    pub fn helper_log(&self) -> Vec<String> {
        self.helper_log
            .lock()
            .expect("helper log")
            .iter()
            .cloned()
            .collect()
    }
    pub fn set_bundled_sidecar_path(&self, path: PathBuf) {
        *self.bundled_sidecar.write().unwrap() = Some(path);
    }
    pub fn secret_store(&self) -> Arc<dyn SecretStore> {
        self.secrets.clone()
    }
    pub fn bind(self: &Arc<Self>, binding: AppBinding) {
        if let Some(old) = &binding.previous_health {
            for item in old
                .report("", false, false)
                .items
                .into_iter()
                .filter(|i| i.code == "camera" || i.code.starts_with("camera_"))
            {
                if item.ok {
                    binding.health.set_ok(&item.code, item.detail);
                } else {
                    binding.health.set(&item.code, item.detail);
                }
            }
        }
        let settings = binding.camera_settings;
        let capture_root = binding.capture_root.clone();
        let health = binding.health.clone();
        *self.capture_root.write().unwrap() = capture_root;
        // §6.11 Runtime (b): the ledger over the new store, sharing the process-wide worker lock;
        // pending rows are recovered under that lock before anything new arrives.
        let ledger = self.install_ledger(LedgerService {
            store: binding.store.clone(),
            clock: self.clock.clone(),
            fs: self.delivery_fs.read().unwrap().clone(),
            events: binding.events.clone(),
            health: binding.health.clone(),
            capture_root: self.capture_root_fn(),
            shared: self.ledger_shared.clone(),
        });
        if let Err(error) = ledger.recover_pending() {
            tracing::warn!(%error, "camera ledger recovery failed");
        }
        *self.binding.write().expect("camera binding") = Some(binding);
        if super::support::damaged(&self.paths) {
            health.set(
                "camera_support_not_installed",
                "Camera support for Sony is damaged: add it again in Settings → Camera",
            );
        } else {
            health.clear("camera_support_not_installed");
        }
        // From P46 the direct adapter re-publishes the codes it owns from its own inputs.
        if let Some(adapter) = self.bound_direct() {
            adapter.publish(&health, true);
        }
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        if super::support::installed(&self.paths).is_some() {
            if let Some(password) = self.secrets.cached(secrets::FTP_PASSWORD) {
                self.bind_receiver_with_password(settings, password);
            } else {
                let runtime = self.clone();
                std::thread::spawn(move || {
                    if let Ok(Some(password)) = runtime.secrets.get(secrets::FTP_PASSWORD) {
                        runtime.bind_receiver_with_password(settings, password);
                    }
                });
            }
        }
        let mut guard = self.supervisor.lock().expect("camera supervisor");
        if guard.is_some() {
            return;
        }
        let bundled = self
            .bundled_sidecar
            .read()
            .unwrap()
            .clone()
            .or_else(|| super::support::bundled_for(&self.paths));
        if let Some(bundled) = bundled
            && super::support::installed(&self.paths).is_some_and(|s| s.version != "test")
            && let Err(error) = super::support::sony::refresh_sidecar_copy(&self.paths, &bundled)
        {
            tracing::warn!(%error, "camera sidecar refresh failed");
        }
        let Some(program) = installed_sidecar(&self.paths) else {
            return;
        };
        if self.credentials.lock().unwrap().is_none()
            && let (Some(user), Some(password)) = (
                self.secrets.cached(secrets::ACCESS_AUTH_USER),
                self.secrets.cached(secrets::ACCESS_AUTH_PASSWORD),
            )
        {
            *self.credentials.lock().unwrap() = Some((user, password));
        }
        let (user, password) = self.credentials.lock().unwrap().clone().unwrap_or_else(|| {
            (
                self.secrets
                    .cached(secrets::ACCESS_AUTH_USER)
                    .unwrap_or_default(),
                self.secrets
                    .cached(secrets::ACCESS_AUTH_PASSWORD)
                    .unwrap_or_default(),
            )
        });
        logging::add_secret(&user);
        logging::add_secret(&password);
        // §Staging (R-I20): `<binding document>/Capture/.incoming/sdk-<run_id>`, computed at the
        // first spawn and never moved for this process (restarts reuse it).
        let staging = self
            .capture_root
            .read()
            .unwrap()
            .join("Capture/.incoming")
            .join(format!("sdk-{}", self.run_id));
        if let Err(error) = fs::create_dir_all(&staging) {
            tracing::warn!(%error, "camera staging unavailable");
            return;
        }
        *self.staging_dir.lock().unwrap() = Some(staging.clone());
        let spec = SidecarSpec {
            program,
            args: vec![
                "auto".into(),
                "--save-dir".into(),
                staging.to_string_lossy().into_owned(),
                "--usb-product-id".into(),
                USB_PRODUCT_IDS.into(),
            ],
            env: vec![(
                "CRLAB_FINGERPRINT_DIR".into(),
                self.paths
                    .root
                    .join("camera/fingerprints")
                    .to_string_lossy()
                    .into_owned(),
            )],
            first_line: Command::Credentials { user, password }.encode(),
        };
        if let Some(adapter) = self.direct.get() {
            adapter.set_staging_dir(&staging);
        }
        let mut supervisor = Supervisor::new(spec);
        match supervisor.spawn(self.clock.mono()) {
            Ok(_) => {
                self.helper_running.store(true, Ordering::SeqCst);
                *self.spawned_credentials.lock().unwrap() =
                    Some(supervisor.spec.first_line.clone());
            }
            Err(error) => {
                tracing::warn!(%error, "camera sidecar unavailable");
                self.start_order.satisfy(StartReason::SidecarUnavailable);
            }
        }
        *guard = Some(SidecarClient::new(supervisor));
        drop(guard);
        if self.credentials.lock().unwrap().is_none() {
            let runtime = self.clone();
            std::thread::spawn(move || {
                let user = runtime
                    .secrets
                    .get(secrets::ACCESS_AUTH_USER)
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                let password = runtime
                    .secrets
                    .get(secrets::ACCESS_AUTH_PASSWORD)
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                logging::add_secret(&user);
                logging::add_secret(&password);
                let has_credentials = !user.is_empty() || !password.is_empty();
                *runtime.credentials.lock().unwrap() = Some((user, password));
                runtime.update_spawn_credentials();
                if has_credentials {
                    runtime.restart_if_not_usb();
                }
            });
        }
    }
    /// Writes `command` to the sidecar; the returned id is the one on the wire.
    pub fn send_command(&self, command: Command) -> psm_core::Result<CmdId> {
        let mut guard = self.supervisor.lock().expect("camera supervisor");
        let client = guard
            .as_mut()
            .ok_or_else(|| psm_core::Error::Provider("camera sidecar not running".into()))?;
        client
            .send(command)
            .map_err(|e| psm_core::Error::Provider(format!("camera sidecar: {e}")))
    }
    /// The outcome of the command sent as `cmd_id`, once the sidecar reported it.
    pub fn command_outcome(&self, cmd_id: &str) -> Option<CommandOutcome> {
        self.supervisor
            .lock()
            .expect("camera supervisor")
            .as_ref()
            .and_then(|client| client.outcome(cmd_id))
    }
    /// The last `SIDECAR_EVENTS_KEPT` events the sidecar delivered, oldest first.
    pub fn sidecar_events(&self) -> Vec<Stamped> {
        self.sidecar_events
            .lock()
            .expect("sidecar events")
            .iter()
            .cloned()
            .collect()
    }
    fn push_helper_log(&self, line: &str) {
        let redacted = logging::redact_registered(line);
        let mut log = self.helper_log.lock().expect("helper log");
        if log.len() == HELPER_LOG_KEPT {
            log.pop_front();
        }
        log.push_back(redacted.clone());
        tracing::warn!("camera sidecar: {redacted}");
    }
    fn bind_receiver_with_password(&self, settings: CameraSettings, password: String) {
        let mut receiver = self.receiver.lock().unwrap();
        let mut previous = self.receiver_settings.lock().unwrap();
        if receiver.is_some() && *previous == Some(settings) {
            return;
        }
        let port = if settings.receiver_port == 0 {
            receiver.as_ref().map(ReceiverHandle::port).unwrap_or(0)
        } else {
            settings.receiver_port
        };
        if let Some(mut old) = receiver.take() {
            old.stop();
        }
        *previous = None;
        if installed_sidecar(&self.paths).is_none() {
            return;
        }
        if password.is_empty() {
            return;
        }
        logging::add_secret(&password);
        let config = ReceiverConfig {
            bind: self.receiver_bind,
            port,
            passive: settings.passive_port_start..=settings.passive_port_end,
            allowed_peers: if self.receiver_bind == IpAddr::V4(Ipv4Addr::LOCALHOST) {
                vec!["127.0.0.0/8".parse().unwrap()]
            } else {
                ReceiverConfig::default_allowed_peers()
            },
            username: FTP_USERNAME.into(),
            password,
            capture_root: self.capture_root_fn(),
            sink: self.sink.clone(),
            names: self.names.clone(),
            clock: self.clock.clone(),
        };
        match FtpReceiver::start(config) {
            Ok(handle) => {
                *receiver = Some(handle);
                *previous = Some(settings);
                self.receiver_generation.fetch_add(1, Ordering::SeqCst);
            }
            Err(error) => tracing::warn!(%error, "camera FTP receiver unavailable"),
        }
    }
    fn restart_if_not_usb(&self) {
        if self.direct.get().is_some_and(|a| a.usb_session()) {
            return;
        }
        let credentials = self.credentials.lock().unwrap().clone().unwrap_or_default();
        let mut guard = self.supervisor.lock().unwrap();
        if let Some(client) = guard.as_mut() {
            let mut spec = client.supervisor().spec.clone();
            spec.first_line = Command::Credentials {
                user: credentials.0,
                password: credentials.1,
            }
            .encode();
            client.supervisor_mut().quit_and_wait();
            let mut supervisor = Supervisor::new(spec);
            if supervisor.spawn(self.clock.mono()).is_ok() {
                self.helper_running.store(true, Ordering::SeqCst);
                *self.spawned_credentials.lock().unwrap() =
                    Some(supervisor.spec.first_line.clone());
            }
            *guard = Some(SidecarClient::new(supervisor));
        }
    }
    fn update_spawn_credentials(&self) {
        let credentials = self.credentials.lock().unwrap().clone().unwrap_or_default();
        if let Some(client) = self.supervisor.lock().unwrap().as_mut() {
            client.supervisor_mut().spec.first_line = Command::Credentials {
                user: credentials.0,
                password: credentials.1,
            }
            .encode();
        }
    }
    pub fn secrets_changed(&self) {
        let user = self
            .secrets
            .get(secrets::ACCESS_AUTH_USER)
            .ok()
            .flatten()
            .unwrap_or_default();
        let password = self
            .secrets
            .get(secrets::ACCESS_AUTH_PASSWORD)
            .ok()
            .flatten()
            .unwrap_or_default();
        logging::add_secret(&user);
        logging::add_secret(&password);
        *self.credentials.lock().unwrap() = Some((user.clone(), password.clone()));
        if let Some(client) = self.supervisor.lock().unwrap().as_mut() {
            client.supervisor_mut().spec.first_line =
                Command::Credentials { user, password }.encode();
        }
        self.restart_if_not_usb();
    }
    pub fn ftp_password_changed(&self) {
        if let Some(binding) = self.binding.read().unwrap().as_ref()
            && let Ok(Some(password)) = self.secrets.get(secrets::FTP_PASSWORD)
        {
            *self.receiver_settings.lock().unwrap() = None;
            self.bind_receiver_with_password(binding.camera_settings, password);
        }
    }
    pub fn support_changed(self: &Arc<Self>) {
        if let Some(client) = self.supervisor.lock().unwrap().as_mut() {
            client.supervisor_mut().quit_and_wait();
        }
        *self.supervisor.lock().unwrap() = None;
        self.helper_running.store(false, Ordering::SeqCst);
    }
    pub fn tick(&self, now: Instant) {
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        let _serial = self
            .tick_serial
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // One `client.poll` per tick (P46 L19); the supervisor lock is released before the
        // adapter step so `capture()` never waits on a download's hash.
        let (events, side, exits) = {
            let mut guard = self.supervisor.lock().expect("camera supervisor");
            let Some(client) = guard.as_mut() else {
                return;
            };
            let mut side = Vec::new();
            let events = client.poll(now, &mut |event| side.push(event));
            (events, side, client.supervisor().consecutive_exits())
        };
        // Output first, exits after (as the supervisor reports them): a sidecar that prints and
        // then dies within one tick still counts as unavailable.
        if !events.is_empty() {
            if let Some(binding) = self.binding.read().expect("camera binding").as_ref() {
                binding.health.clear("camera_sidecar_unavailable");
            }
            let mut kept = self.sidecar_events.lock().expect("sidecar events");
            for event in &events {
                if kept.len() == SIDECAR_EVENTS_KEPT {
                    kept.pop_front();
                }
                kept.push_back(event.clone());
            }
        }
        let mut helper = Vec::new();
        for event in side {
            match event {
                SupervisorEvent::Exited { .. } => {
                    self.helper_running.store(false, Ordering::SeqCst);
                    helper.push(HelperEvent::Exited);
                    if exits >= 3
                        && let Some(binding) = self.binding.read().expect("camera binding").as_ref()
                    {
                        binding
                            .health
                            .set("camera_sidecar_unavailable", "Camera helper is unavailable");
                    }
                }
                SupervisorEvent::Started { .. } => {
                    self.helper_running.store(true, Ordering::SeqCst);
                    if let Some(client) = self.supervisor.lock().unwrap().as_ref() {
                        *self.spawned_credentials.lock().unwrap() =
                            Some(client.supervisor().spec.first_line.clone());
                    }
                    helper.push(HelperEvent::Started);
                }
                SupervisorEvent::Stderr(line) => self.push_helper_log(&line),
                SupervisorEvent::Line(_) => {}
            }
        }
        let Some(adapter) = self.bound_direct() else {
            return;
        };
        let outcome = adapter.step(
            &events,
            &helper,
            self.helper_running.load(Ordering::SeqCst),
            now,
            self.sink.as_ref(),
        );
        self.perform(&adapter, outcome, now);
        if events.iter().any(|event| {
            matches!(
                &event.event,
                SidecarEvent::SessionLost {
                    transport: Transport::Usb,
                    ..
                }
            )
        }) {
            let credentials = self.credentials.lock().unwrap().clone();
            if let Some((user, password)) = credentials {
                let expected = Command::Credentials { user, password }.encode();
                let has_stale_line =
                    self.spawned_credentials.lock().unwrap().as_deref() != Some(expected.as_str());
                if has_stale_line {
                    self.restart_if_not_usb();
                }
            }
        }
    }
    /// The adapter's effects against the current binding, then the watchdog glue (P46 L10, L13).
    fn perform(&self, adapter: &DirectCameraAdapter, outcome: StepOutcome, now: Instant) {
        if let Some(binding) = self.binding.read().expect("camera binding").as_ref() {
            for (fingerprint, replace) in &outcome.fingerprint_pending {
                psm_core::ports::events::EventSink::emit(
                    binding.events.as_ref(),
                    AppEvent::FingerprintPending {
                        fingerprint: fingerprint.clone(),
                        replace: *replace,
                    },
                );
            }
        }
        for line in &outcome.helper_log {
            self.push_helper_log(line);
        }
        if let Some(reason) = outcome.start {
            self.start_order.satisfy(reason);
        }
        self.start_order
            .camera_state(outcome.usb_present, outcome.usb_ready);
        match outcome.sleep {
            Some(true) => {
                if let Err(error) = self.sleep.read().unwrap().hold(SLEEP_REASON) {
                    tracing::warn!(%error, "idle-sleep assertion not held");
                }
            }
            Some(false) => self.sleep.read().unwrap().release(),
            None => {}
        }
        if outcome.sdk_unavailable
            && let Some(binding) = self.binding.read().expect("camera binding").as_ref()
        {
            // After the per-batch clear above, so the set wins (L21).
            binding.health.set(
                "camera_sidecar_unavailable",
                "The camera helper could not start",
            );
        }
        let ledger = self.ledger();
        if let (Some(note), Some(ledger)) = (outcome.ftp_note, ledger.as_ref()) {
            ledger.set_ftp_note(Some(note));
        }
        if let Some(ledger) = ledger.as_ref() {
            let due = {
                let mut at = self.watchdog_at.lock().expect("watchdog time");
                let due =
                    at.is_none_or(|t| now.saturating_duration_since(t) >= Duration::from_secs(1));
                if due {
                    *at = Some(now);
                }
                due
            };
            if due {
                let transfer_active = self.uploads_in_progress() > 0 || adapter.transfer_expected();
                match ledger.watchdog(self.clock.now_ms(), transfer_active) {
                    Ok(state) => {
                        self.watchdog_pending
                            .store(state.pending_count, Ordering::SeqCst);
                        if state.first_raise
                            && let Some(provider) = self.provider()
                            && let Err(error) = provider.request_ftp_status()
                        {
                            tracing::warn!(%error, "camera ftp-status not sent");
                        }
                    }
                    Err(error) => tracing::warn!(%error, "camera watchdog failed"),
                }
            }
            adapter.set_files_not_received(
                ledger
                    .raised()
                    .then(|| self.watchdog_pending.load(Ordering::SeqCst)),
            );
        }
        let (events, health) = match self.binding.read().expect("camera binding").as_ref() {
            Some(binding) => (binding.events.clone(), binding.health.clone()),
            None => return,
        };
        if let Some(status) = adapter.status_change() {
            psm_core::ports::events::EventSink::emit(
                events.as_ref(),
                AppEvent::CameraStatusChanged { status },
            );
        }
        adapter.publish(&health, false);
    }
    pub fn shutdown(&self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        if let Some(mut receiver) = self.receiver.lock().unwrap().take() {
            receiver.stop();
        }
        if let Some(supervisor) = self.supervisor.lock().expect("camera supervisor").as_mut() {
            supervisor.supervisor_mut().quit_and_wait();
        }
        self.sleep.read().unwrap().release();
        if let Some(staging) = self.staging_dir.lock().unwrap().take() {
            // Only an empty folder is removed. A camera transfer is never touched (H2).
            let _ = fs::remove_dir(staging);
        }
    }
}
impl Drop for CameraRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}
