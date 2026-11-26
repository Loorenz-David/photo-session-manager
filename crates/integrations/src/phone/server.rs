//! PH2: one phone server per app process, on its own thread with its own tokio runtime (as
//! `psm_infra::nudge` does). It resolves the current `AppHandles` per request (App rebuilds are
//! invisible to it) and holds the runtime's live view hub. A bind failure is reported in
//! [`PhoneServer::status`] and the app runs on.
use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use psm_core::{Result, domain::live::PhoneSettings, services::live_view::LiveViewHub};

use crate::{AppHandles, phone::host::HostInfo};

/// Resolves the App's current handles (called per request; never kept across requests).
pub type HandlesFn = Arc<dyn Fn() -> Result<AppHandles> + Send + Sync>;

/// PH5: WebSocket keep-alive and write limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhoneTimings {
    pub ping: Duration,
    pub idle_close: Duration,
    pub write_timeout: Duration,
}

impl Default for PhoneTimings {
    fn default() -> Self {
        Self {
            ping: Duration::from_secs(5),
            idle_close: Duration::from_secs(15),
            write_timeout: Duration::from_secs(2),
        }
    }
}

/// How the server is started.
#[derive(Clone)]
pub struct PhoneConfig {
    /// `phone.enabled`.
    pub enabled: bool,
    /// `0.0.0.0` in the app (the phones on the Wi-Fi); `127.0.0.1` in tests.
    pub bind: IpAddr,
    /// `phone.port` (`0` = any free port, tests).
    pub port: u16,
    /// Where `phone.html` and `assets/` are (`web_root::resolve`); `None` serves no page.
    pub web_root: Option<PathBuf>,
    pub timings: PhoneTimings,
    /// The Mac's name for `hello`.
    pub host: Arc<dyn HostInfo>,
}

impl PhoneConfig {
    /// The app's configuration for `phone` settings.
    pub fn for_settings(
        settings: PhoneSettings,
        web_root: Option<PathBuf>,
        host: Arc<dyn HostInfo>,
    ) -> Self {
        Self {
            enabled: settings.enabled,
            bind: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: settings.port,
            web_root,
            timings: PhoneTimings::default(),
            host,
        }
    }
}

/// `phone_server_status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneServerStatus {
    pub enabled: bool,
    pub running: bool,
    pub port: u16,
    pub error: Option<String>,
}

/// The open WebSockets by pairing (PH1: a removed phone's sockets close; Settings shows which
/// phones are connected).
#[derive(Default)]
pub(crate) struct Sockets {
    #[allow(dead_code)] // the WebSocket adds sockets (LV2·U5)
    next: AtomicU64,
    open: Mutex<HashMap<u64, (i64, Arc<tokio::sync::Notify>)>>,
}

impl Sockets {
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<u64, (i64, Arc<tokio::sync::Notify>)>> {
        self.open.lock().unwrap_or_else(|e| e.into_inner())
    }
    /// A socket of `pairing` opened: its id and the signal that closes it (4401).
    #[allow(dead_code)] // the WebSocket adds sockets (LV2·U5)
    pub fn add(&self, pairing: i64) -> (u64, Arc<tokio::sync::Notify>) {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let revoked = Arc::new(tokio::sync::Notify::new());
        self.lock().insert(id, (pairing, revoked.clone()));
        (id, revoked)
    }
    #[allow(dead_code)] // the WebSocket removes sockets (LV2·U5)
    pub fn remove(&self, id: u64) {
        self.lock().remove(&id);
    }
    pub fn pairings(&self) -> HashSet<i64> {
        self.lock().values().map(|(p, _)| *p).collect()
    }
    pub fn close_pairing(&self, pairing: i64) {
        for (p, revoked) in self.lock().values() {
            if *p == pairing {
                revoked.notify_one();
            }
        }
    }
}

/// What every request reads (no App: the handles are resolved per request).
pub(crate) struct Shared {
    pub handles: HandlesFn,
    #[allow(dead_code)] // read by the WebSocket (LV2·U5)
    pub hub: Arc<LiveViewHub>,
    pub web_root: Option<PathBuf>,
    #[allow(dead_code)] // read by the WebSocket (LV2·U5)
    pub timings: PhoneTimings,
    pub host: Arc<dyn HostInfo>,
    pub sockets: Sockets,
}

struct Listening {
    addr: SocketAddr,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Listening {
    fn stop(mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct State {
    enabled: bool,
    bind: IpAddr,
    port: u16,
    listening: Option<Listening>,
    error: Option<String>,
}

/// The phone server (PH2).
pub struct PhoneServer {
    shared: Arc<Shared>,
    state: Mutex<State>,
}

impl PhoneServer {
    /// Creates the server and, when enabled, listens on `config.bind:config.port`.
    pub fn start(config: PhoneConfig, handles: HandlesFn, hub: Arc<LiveViewHub>) -> Self {
        let server = Self {
            shared: Arc::new(Shared {
                handles,
                hub,
                web_root: config.web_root,
                timings: config.timings,
                host: config.host,
                sockets: Sockets::default(),
            }),
            state: Mutex::new(State {
                enabled: config.enabled,
                bind: config.bind,
                port: config.port,
                listening: None,
                error: None,
            }),
        };
        server.restart();
        server
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Stops listening, then listens again with the stored switch and port.
    fn restart(&self) {
        let mut state = self.lock();
        if let Some(listening) = state.listening.take() {
            listening.stop();
        }
        state.error = None;
        if !state.enabled {
            return;
        }
        match listen(self.shared.clone(), SocketAddr::new(state.bind, state.port)) {
            Ok(listening) => {
                tracing::info!(
                    target: "psm::phone",
                    "phone.server listening port={} addresses={}",
                    listening.addr.port(),
                    addresses(state.bind)
                );
                state.listening = Some(listening);
            }
            Err(error) => {
                tracing::warn!(target: "psm::phone", "phone.server error={error}");
                state.error = Some(error);
            }
        }
    }

    /// A saved port change (§4.5): listens on `port` from now on.
    pub fn rebind(&self, port: u16) {
        self.lock().port = port;
        self.restart();
    }

    /// Saved `phone` settings: rebinds or stops only when the switch or the port changed.
    pub fn apply(&self, settings: PhoneSettings) {
        {
            let mut state = self.lock();
            if state.enabled == settings.enabled
                && state.port == settings.port
                && (state.listening.is_some() || state.error.is_some() || !state.enabled)
            {
                return;
            }
            state.enabled = settings.enabled;
            state.port = settings.port;
        }
        self.restart();
    }

    /// Stops listening (app exit); open sockets are closed.
    pub fn stop(&self) {
        let mut state = self.lock();
        if let Some(listening) = state.listening.take() {
            listening.stop();
        }
    }

    pub fn status(&self) -> PhoneServerStatus {
        let state = self.lock();
        PhoneServerStatus {
            enabled: state.enabled,
            running: state.listening.is_some(),
            port: state
                .listening
                .as_ref()
                .map_or(state.port, |l| l.addr.port()),
            error: state.error.clone(),
        }
    }

    /// The pairings with an open WebSocket now.
    pub fn connected_pairings(&self) -> HashSet<i64> {
        self.shared.sockets.pairings()
    }

    /// PH1: closes every open socket of `pairing` (code 4401); its token no longer verifies.
    pub fn close_pairing(&self, pairing: i64) {
        self.shared.sockets.close_pairing(pairing);
    }

    /// The address listened on (tests bind port 0).
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.lock().listening.as_ref().map(|l| l.addr)
    }
}

impl Drop for PhoneServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The phone-reachable addresses for the log line.
fn addresses(bind: IpAddr) -> String {
    if !bind.is_unspecified() {
        return bind.to_string();
    }
    let found: Vec<String> = psm_infra::net::local_addresses()
        .into_iter()
        .filter(|a| a.is_ipv4())
        .map(|a| a.to_string())
        .collect();
    if found.is_empty() {
        "none".into()
    } else {
        found.join(",")
    }
}

/// Binds `addr` on a new thread with its own runtime and serves until stopped.
fn listen(shared: Arc<Shared>, addr: SocketAddr) -> std::result::Result<Listening, String> {
    let (tx, rx) = mpsc::sync_channel(1);
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let thread = thread::Builder::new()
        .name("phone-server".into())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_name("phone-server-rt")
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                    return;
                }
            };
            runtime.block_on(async move {
                let listener = match tokio::net::TcpListener::bind(addr).await {
                    Ok(listener) => listener,
                    Err(e) => {
                        let _ = tx.send(Err(match e.kind() {
                            std::io::ErrorKind::AddrInUse => format!("port {} in use", addr.port()),
                            _ => format!("port {}: {e}", addr.port()),
                        }));
                        return;
                    }
                };
                let local = match listener.local_addr() {
                    Ok(local) => local,
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string()));
                        return;
                    }
                };
                let _ = tx.send(Ok(local));
                let app = crate::phone::routes::router(shared)
                    .into_make_service_with_connect_info::<SocketAddr>();
                tokio::select! {
                    _ = axum::serve(listener, app) => {}
                    _ = stop_rx => {}
                }
            });
            // Open sockets (WebSockets included) end with the runtime.
            runtime.shutdown_timeout(Duration::from_millis(300));
        })
        .map_err(|e| e.to_string())?;
    let addr = rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|e| e.to_string())
        .and_then(|r| r);
    match addr {
        Ok(addr) => Ok(Listening {
            addr,
            stop: Some(stop_tx),
            thread: Some(thread),
        }),
        Err(error) => {
            let _ = thread.join();
            Err(error)
        }
    }
}
