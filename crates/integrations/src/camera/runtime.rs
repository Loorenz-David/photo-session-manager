use super::{
    sidecar::supervisor::{SidecarSpec, Supervisor, SupervisorEvent},
    support::installed_sidecar,
};
use psm_core::{
    domain::camera::secrets,
    ports::{camera::CameraProvider, clock::Clock, secrets::SecretStore},
    services::health::HealthService,
};
use psm_infra::{logging, paths::AppPaths};
use std::{
    collections::VecDeque,
    fs,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
use uuid::Uuid;

pub struct AppBinding {
    pub events: Arc<crate::EventHub>,
    pub health: Arc<HealthService>,
    pub previous_health: Option<Arc<HealthService>>,
    pub provider: Arc<dyn CameraProvider>,
    pub paths: AppPaths,
}
pub struct CameraRuntime {
    pub run_id: String,
    supervisor: Mutex<Option<Supervisor>>,
    binding: RwLock<Option<AppBinding>>,
    secrets: Arc<dyn SecretStore>,
    credentials: Mutex<Option<(String, String)>>,
    clock: Arc<dyn Clock>,
    paths: AppPaths,
    helper_log: Mutex<VecDeque<String>>,
    stopped: AtomicBool,
}
impl CameraRuntime {
    pub fn new(secrets: Arc<dyn SecretStore>, clock: Arc<dyn Clock>, paths: AppPaths) -> Arc<Self> {
        Arc::new(Self {
            run_id: Uuid::new_v4().to_string(),
            supervisor: Mutex::new(None),
            binding: RwLock::new(None),
            secrets,
            credentials: Mutex::new(None),
            clock,
            paths,
            helper_log: Mutex::new(VecDeque::with_capacity(50)),
            stopped: AtomicBool::new(false),
        })
    }
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
    pub fn helper_log(&self) -> Vec<String> {
        self.helper_log
            .lock()
            .expect("helper log")
            .iter()
            .cloned()
            .collect()
    }
    pub fn bind(&self, binding: AppBinding) {
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
        *self.binding.write().expect("camera binding") = Some(binding);
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        let mut guard = self.supervisor.lock().expect("camera supervisor");
        if guard.is_some() {
            return;
        }
        let Some(program) = installed_sidecar(&self.paths) else {
            return;
        };
        let mut credentials = self.credentials.lock().expect("camera credentials");
        let (user, password) = credentials
            .get_or_insert_with(|| {
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
                (user, password)
            })
            .clone();
        logging::add_secret(&user);
        logging::add_secret(&password);
        let staging = self
            .paths
            .root
            .join("camera/staging")
            .join(format!("sdk-{}", self.run_id));
        if let Err(error) = fs::create_dir_all(&staging) {
            tracing::warn!(%error, "camera staging unavailable");
            return;
        }
        let spec = SidecarSpec {
            program,
            args: vec![
                "auto".into(),
                "--save-dir".into(),
                staging.to_string_lossy().into_owned(),
            ],
            env: vec![(
                "CRLAB_FINGERPRINT_DIR".into(),
                self.paths
                    .root
                    .join("camera/fingerprints")
                    .to_string_lossy()
                    .into_owned(),
            )],
            first_line: format!(
                "credentials user={} password={}",
                pct(&user),
                pct(&password)
            ),
        };
        let mut supervisor = Supervisor::new(spec);
        if let Err(error) = supervisor.spawn(self.clock.mono()) {
            tracing::warn!(%error, "camera sidecar unavailable");
        }
        *guard = Some(supervisor);
    }
    pub fn tick(&self, now: Instant) {
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        let mut guard = self.supervisor.lock().expect("camera supervisor");
        let Some(supervisor) = guard.as_mut() else {
            return;
        };
        for event in supervisor.tick(now) {
            match event {
                SupervisorEvent::Line(_line) => {
                    if let Some(binding) = self.binding.read().expect("camera binding").as_ref() {
                        binding.health.clear("camera_sidecar_unavailable");
                    }
                }
                SupervisorEvent::Exited { .. } if supervisor.consecutive_exits() >= 3 => {
                    if let Some(binding) = self.binding.read().expect("camera binding").as_ref() {
                        binding
                            .health
                            .set("camera_sidecar_unavailable", "Camera helper is unavailable");
                    }
                }
                SupervisorEvent::Stderr(line) => {
                    let redacted = logging::redact_registered(&line);
                    let mut log = self.helper_log.lock().expect("helper log");
                    if log.len() == 50 {
                        log.pop_front();
                    }
                    log.push_back(redacted.clone());
                    tracing::warn!("camera sidecar: {redacted}");
                }
                SupervisorEvent::Started { .. } | SupervisorEvent::Exited { .. } => {}
            }
        }
    }
    pub fn shutdown(&self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        if let Some(supervisor) = self.supervisor.lock().expect("camera supervisor").as_mut() {
            supervisor.quit_and_wait();
        }
    }
}
impl Drop for CameraRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn pct(value: &str) -> String {
    let mut result = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            result.push(byte as char);
        } else {
            result.push_str(&format!("%{byte:02X}"));
        }
    }
    result
}
