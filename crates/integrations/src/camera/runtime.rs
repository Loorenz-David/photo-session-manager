use super::{
    direct::{
        CAPTURE_ONE_WINDOW_OPEN, DirectCameraAdapter, SLEEP_REASON, StartOrder, StartReason,
        TakeBackService,
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
use psm_core::services::editing_startup::{DOCUMENT_MISSING, document_present};
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
        camera::{ADDRESS_CHANGED, FtpAddress},
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
    net::{IpAddr, Ipv4Addr, UdpSocket},
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
/// P60 #23: the automatic take-back asks Capture One to quit again this long after it did not.
const AUTO_RETRY_AFTER_DIALOG: Duration = Duration::from_secs(2);
/// The camera's USB product ids for the sidecar's IOKit match (R-I23): the A7R V.
const USB_PRODUCT_IDS: &str = "0x0E0C";

fn local_lan_address() -> String {
    let Ok(socket) = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)) else {
        return "this Mac's LAN address".into();
    };
    // UDP connect chooses the default route without sending a packet.
    if socket.connect((Ipv4Addr::new(192, 0, 2, 1), 9)).is_ok()
        && let Ok(addr) = socket.local_addr()
        && !addr.ip().is_unspecified()
    {
        return addr.ip().to_string();
    }
    "this Mac's LAN address".into()
}

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
    /// The document was chosen in Settings (P63 #27: missing → reported, never recreated).
    pub document_configured: bool,
}
struct RoutingSink(RwLock<Arc<dyn CaptureSink>>, Arc<UploadAddress>);
/// The address of this Mac the camera last uploaded to (P68): the local address of the FTP
/// receiver's last accepted connection, kept in `<data>/camera/last_upload_address` so a restart
/// on another network still notices the change. Never logged (the address is shown in the UI only).
struct UploadAddress {
    file: PathBuf,
    value: Mutex<Option<Option<IpAddr>>>,
}
impl UploadAddress {
    fn get(&self) -> Option<IpAddr> {
        *self.value.lock().unwrap().get_or_insert_with(|| {
            fs::read_to_string(&self.file)
                .ok()
                .and_then(|text| text.trim().parse().ok())
        })
    }
    fn set(&self, address: IpAddr) {
        if self.get() == Some(address) {
            return;
        }
        *self.value.lock().unwrap() = Some(Some(address));
        if let Some(dir) = self.file.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Err(error) = fs::write(&self.file, address.to_string()) {
            tracing::warn!(%error, "camera upload address not kept");
        }
    }
}
/// This Mac's addresses (P68); `set_network_addresses` is the test seam.
type NetworkAddresses = Arc<dyn Fn() -> Vec<IpAddr> + Send + Sync>;
impl CaptureSink for RoutingSink {
    fn connection_accepted(&self, local: IpAddr) {
        self.1.set(local);
    }
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
    auto_take_back_enabled: AtomicBool,
    auto_plugged: AtomicBool,
    auto_taken_this_plug: AtomicBool,
    auto_retry_after: Mutex<Option<Instant>>,
    auto_failure_message: Mutex<Option<String>>,
    ftp_error_delivery_id: Mutex<Option<i64>>,
    bundled_sidecar: RwLock<Option<PathBuf>>,
    staging_dir: Mutex<Option<PathBuf>>,
    /// P63 #27: the helper was not started because the document is missing; started once it is
    /// there.
    spawn_deferred: AtomicBool,
    /// P63 #27: when the document was last looked at (real time; a filesystem poll).
    document_checked: Mutex<Option<std::time::Instant>>,
    /// This Mac's addresses (P68); the test seam is `set_network_addresses`.
    network: RwLock<NetworkAddresses>,
    /// When the address was last compared with the camera's upload address (real time).
    address_checked: Mutex<Option<std::time::Instant>>,
}

/// How often the tick compares this Mac's address with the camera's upload address (P68).
const ADDRESS_CHECK_EVERY: Duration = Duration::from_millis(1000);
/// How often the tick looks whether the document is there (P63 #27).
const DOCUMENT_CHECK_EVERY: Duration = Duration::from_millis(250);
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
            sink: Arc::new(RoutingSink(
                RwLock::new(Arc::new(Unbound)),
                Arc::new(UploadAddress {
                    file: paths.root.join("camera/last_upload_address"),
                    value: Mutex::new(None),
                }),
            )),
            network: RwLock::new(Arc::new(psm_infra::net::local_addresses)),
            address_checked: Mutex::new(None),
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
            auto_take_back_enabled: AtomicBool::new(true),
            auto_plugged: AtomicBool::new(false),
            auto_taken_this_plug: AtomicBool::new(false),
            auto_retry_after: Mutex::new(None),
            auto_failure_message: Mutex::new(None),
            ftp_error_delivery_id: Mutex::new(None),
            bundled_sidecar: RwLock::new(None),
            staging_dir: Mutex::new(None),
            spawn_deferred: AtomicBool::new(false),
            document_checked: Mutex::new(None),
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
    /// Test seam: this Mac's addresses as the network reports them (P68).
    pub fn set_network_addresses(&self, addresses: NetworkAddresses) {
        *self.network.write().unwrap() = addresses;
        *self.address_checked.lock().unwrap() = None;
    }
    /// This Mac's address for the camera's Host Name, and whether it changed since the camera
    /// last uploaded (P68).
    pub fn ftp_address(&self) -> FtpAddress {
        let current = (self.network.read().unwrap().clone())();
        FtpAddress::of(&current, self.sink.1.get())
    }
    /// P68: Health (and with it the header) says when this Mac's address is no longer the one
    /// the camera uploads to.
    fn check_address(&self) {
        {
            let mut checked = self.address_checked.lock().unwrap();
            if checked.is_some_and(|at| at.elapsed() < ADDRESS_CHECK_EVERY) {
                return;
            }
            *checked = Some(std::time::Instant::now());
        }
        let Some(health) = self
            .binding
            .read()
            .unwrap()
            .as_ref()
            .map(|b| b.health.clone())
        else {
            return;
        };
        match self.ftp_address().warning() {
            Some(sentence) => health.set(ADDRESS_CHANGED, sentence),
            None => health.clear(ADDRESS_CHANGED),
        }
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
    pub fn set_auto_take_back(&self, enabled: bool) {
        self.auto_take_back_enabled.store(enabled, Ordering::SeqCst);
        if !enabled {
            self.direct
                .get()
                .inspect(|adapter| adapter.set_auto_waiting(None));
        }
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
    /// The capture root formula the receiver uses: `canon(<document>/Capture)`, and none while the
    /// document is missing (P63 #27: uploads are answered 451 and nothing is filed or created).
    fn capture_root_fn(&self) -> Arc<dyn Fn() -> Option<PathBuf> + Send + Sync> {
        let document = self.capture_root.clone();
        Arc::new(move || {
            let document = document.read().unwrap().clone();
            if !document_present(&document) {
                return None;
            }
            std::fs::canonicalize(document.join("Capture")).ok()
        })
    }
    /// The helper's Save-to-PC staging folder for the current document (R-I20).
    fn staging_for_document(&self) -> PathBuf {
        self.capture_root
            .read()
            .unwrap()
            .join("Capture/.incoming")
            .join(format!("sdk-{}", self.run_id))
    }
    /// P63 #27/#28, once per `DOCUMENT_CHECK_EVERY`: the header and Health name a missing session
    /// chosen in Settings; a helper deferred for a missing document starts once it is there; a
    /// helper whose staging is under another document is restarted with the current one, never
    /// while a take-back, an upload or a Save-to-PC transfer is in progress.
    fn check_document(&self) {
        {
            let mut checked = self.document_checked.lock().unwrap();
            if checked.is_some_and(|at| at.elapsed() < DOCUMENT_CHECK_EVERY) {
                return;
            }
            *checked = Some(std::time::Instant::now());
        }
        let document = self.capture_root.read().unwrap().clone();
        let present = document_present(&document);
        let configured = self
            .binding
            .read()
            .unwrap()
            .as_ref()
            .map(|b| (b.document_configured, b.health.clone()));
        if let Some((true, health)) = &configured {
            if present {
                health.clear("document_missing");
            } else {
                health.set("document_missing", DOCUMENT_MISSING);
            }
            if let Some(adapter) = self.bound_direct() {
                adapter.set_document_missing(!present);
            }
        }
        if !present {
            return;
        }
        if self.spawn_deferred.load(Ordering::SeqCst) {
            let mut guard = self.supervisor.lock().expect("camera supervisor");
            if guard.is_none() {
                self.spawn_deferred.store(false, Ordering::SeqCst);
                self.spawn_helper(&mut guard);
            }
            return;
        }
        let staging = self.staging_for_document();
        let moved = self
            .staging_dir
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|current| *current != staging);
        let protected = self.take_back_running.load(Ordering::SeqCst)
            || self.uploads_in_progress() > 0
            || self.capture_in_progress();
        if moved && !protected {
            self.restart_for_document();
        }
    }
    /// P63 #28: quits the helper and starts it again with the current document's staging folder,
    /// keeping the camera's cached endpoint (`--ip`/`--mac`).
    fn restart_for_document(&self) {
        let mut guard = self.supervisor.lock().expect("camera supervisor");
        let endpoint: Vec<String> = guard
            .as_ref()
            .map(|client| {
                let args = &client.supervisor().spec.args;
                let mut kept = Vec::new();
                for flag in ["--ip", "--mac"] {
                    if let Some(at) = args.iter().position(|arg| arg == flag)
                        && let Some(value) = args.get(at + 1)
                    {
                        kept.extend([flag.to_owned(), value.clone()]);
                    }
                }
                kept
            })
            .unwrap_or_default();
        if let Some(client) = guard.as_mut() {
            client.supervisor_mut().quit_and_wait();
        }
        *guard = None;
        self.helper_running.store(false, Ordering::SeqCst);
        if let Some(old) = self.staging_dir.lock().unwrap().take() {
            // Only an empty folder is removed. A camera transfer is never touched (H2).
            let _ = fs::remove_dir(old);
        }
        tracing::info!("camera helper restarted for the new Capture One session");
        if self.spawn_helper(&mut guard)
            && let Some(client) = guard.as_mut()
        {
            client.supervisor_mut().spec.args.extend(endpoint);
        }
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
    /// Whether a capture's Save-to-PC download is expected (10F §6: storage clean-up waits).
    pub fn capture_in_progress(&self) -> bool {
        self.direct
            .get()
            .is_some_and(|adapter| adapter.transfer_expected())
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
        *self.document_checked.lock().unwrap() = None;
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
        self.set_auto_take_back(settings.auto_take_back);
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
        if !self.spawn_helper(&mut guard) {
            return;
        }
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
    /// Spawns the helper into `guard` (R-I20 staging, credentials as the first line); `false` when
    /// nothing was spawned.
    fn spawn_helper(&self, guard: &mut Option<SidecarClient>) -> bool {
        let Some(program) = installed_sidecar(&self.paths) else {
            return false;
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
        // §Staging (R-I20): `<binding document>/Capture/.incoming/sdk-<run_id>`; crash restarts
        // reuse it, a document change moves it (P63 #28). A missing document gets no folder: the
        // helper starts once the document is there (P63 #27).
        let staging = self.staging_for_document();
        if !document_present(&self.capture_root.read().unwrap()) {
            tracing::warn!("camera helper waits: the Capture One session is missing");
            self.spawn_deferred.store(true, Ordering::SeqCst);
            return false;
        }
        if let Err(error) = fs::create_dir_all(&staging) {
            tracing::warn!(%error, "camera staging unavailable");
            return false;
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
        true
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
    pub fn tick(self: &Arc<Self>, now: Instant) {
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        let _serial = self
            .tick_serial
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.check_document();
        self.check_address();
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
        for stamped in &events {
            if let SidecarEvent::CameraDiscovered {
                transport: Transport::Wifi,
                ip_address,
                camera_id,
            } = &stamped.event
                && !ip_address.is_empty()
                && let Some(client) = self.supervisor.lock().unwrap().as_mut()
            {
                let args = &mut client.supervisor_mut().spec.args;
                if let Some(at) = args.iter().position(|arg| arg == "--ip") {
                    args.drain(at..at + 2);
                }
                if let Some(at) = args.iter().position(|arg| arg == "--mac") {
                    args.drain(at..at + 2);
                }
                args.extend([
                    "--ip".into(),
                    ip_address.clone(),
                    "--mac".into(),
                    camera_id.clone(),
                ]);
            }
        }
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
        for stamped in &events {
            if let SidecarEvent::CameraFtpState { fields } = &stamped.event {
                self.update_ftp_status(&adapter, fields);
            }
        }
        self.perform(&adapter, outcome, now);
        self.auto_take_back(&events, &adapter);
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
    fn update_ftp_status(
        &self,
        adapter: &DirectCameraAdapter,
        fields: &serde_json::Map<String, serde_json::Value>,
    ) {
        let value = |key: &str| {
            fields
                .get(key)
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_u64())
        };
        let error = value("ftp_connection_error");
        let status = value("ftp_connection_status");
        if status == Some(2) {
            *self.ftp_error_delivery_id.lock().unwrap() = None;
            adapter.set_ftp_unreachable(None);
        } else if error.is_some_and(|v| v != 0) {
            let baseline = self
                .ledger()
                .and_then(|ledger| ledger.deliveries().ok())
                .and_then(|rows| rows.last().map(|row| row.id))
                .unwrap_or(0);
            *self.ftp_error_delivery_id.lock().unwrap() = Some(baseline);
            let port = self
                .receiver_port()
                .or_else(|| {
                    self.binding
                        .read()
                        .unwrap()
                        .as_ref()
                        .map(|b| b.camera_settings.receiver_port)
                })
                .unwrap_or(2121);
            let address = local_lan_address();
            adapter.set_ftp_unreachable(Some(format!("The camera can't reach this Mac for uploads. On the camera, set the FTP server's host to {address} and port {port}.")));
        }
    }
    fn auto_take_back(self: &Arc<Self>, events: &[Stamped], adapter: &DirectCameraAdapter) {
        for stamped in events {
            match stamped.event {
                SidecarEvent::UsbInitial { present: true } => {
                    if !self.auto_plugged.swap(true, Ordering::SeqCst) {
                        self.auto_taken_this_plug.store(false, Ordering::SeqCst);
                    }
                }
                SidecarEvent::UsbInitial { present: false } => {
                    self.auto_plugged.store(false, Ordering::SeqCst);
                    self.auto_taken_this_plug.store(false, Ordering::SeqCst);
                    adapter.set_auto_waiting(None);
                    *self.auto_failure_message.lock().unwrap() = None;
                }
                SidecarEvent::UsbArrived { .. } => {
                    self.auto_plugged.store(true, Ordering::SeqCst);
                    self.auto_taken_this_plug.store(false, Ordering::SeqCst);
                }
                SidecarEvent::UsbRemoved { .. } => {
                    self.auto_plugged.store(false, Ordering::SeqCst);
                    self.auto_taken_this_plug.store(false, Ordering::SeqCst);
                    adapter.set_auto_waiting(None);
                    *self.auto_failure_message.lock().unwrap() = None;
                }
                _ => {}
            }
        }
        if !self.auto_take_back_enabled.load(Ordering::SeqCst)
            || !self.auto_plugged.load(Ordering::SeqCst)
            || self.auto_taken_this_plug.load(Ordering::SeqCst)
            || !adapter.holder_is_capture_one()
        {
            return;
        }
        // P60 #23: after Capture One did not quit (a window open), the same plug-in is tried again
        // shortly, the header keeping the reason until it works.
        if self
            .auto_retry_after
            .lock()
            .unwrap()
            .is_some_and(|at| self.clock.mono() < at)
        {
            return;
        }
        let Ok(service) = self.take_back() else {
            return;
        };
        match service.blocked_reason() {
            Ok(Some(reason)) => {
                adapter.set_auto_waiting(Some(format!("Waiting to take the cable: {reason}")))
            }
            Ok(None) => {
                adapter.set_auto_waiting(self.auto_failure_message.lock().unwrap().clone());
                if !self.auto_taken_this_plug.swap(true, Ordering::SeqCst) {
                    let runtime = self.clone();
                    std::thread::spawn(move || match service.run() {
                        Ok(_) => *runtime.auto_failure_message.lock().unwrap() = None,
                        Err(error) => {
                            match error {
                                psm_core::Error::TakeBackBlocked { .. } => {
                                    runtime.auto_taken_this_plug.store(false, Ordering::SeqCst);
                                }
                                psm_core::Error::TakeBackFailed { step: 3, .. } => {
                                    *runtime.auto_failure_message.lock().unwrap() =
                                        Some(CAPTURE_ONE_WINDOW_OPEN.into());
                                    *runtime.auto_retry_after.lock().unwrap() =
                                        Some(runtime.clock.mono() + AUTO_RETRY_AFTER_DIALOG);
                                    runtime.auto_taken_this_plug.store(false, Ordering::SeqCst);
                                }
                                _ => {}
                            }
                            tracing::warn!(%error, "automatic take-back failed");
                        }
                    });
                }
            }
            Err(error) => tracing::warn!(%error, "automatic take-back availability failed"),
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
            let mut error_at = self.ftp_error_delivery_id.lock().unwrap();
            if let Some(baseline) = *error_at
                && ledger
                    .deliveries()
                    .ok()
                    .and_then(|rows| {
                        rows.iter()
                            .rev()
                            .find(|row| row.via == Link::Wireless)
                            .map(|row| row.id)
                    })
                    .is_some_and(|id| id > baseline)
            {
                *error_at = None;
                adapter.set_ftp_unreachable(None);
            }
            drop(error_at);
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
