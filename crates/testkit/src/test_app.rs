use crate::clock::ManualClock;
use crate::fake_capture_one::FakeCaptureOneScripting;
use crate::fake_commerce::FakeCommerce;
use crate::fake_processes::FakeProcessList;
use crate::fake_secrets::FakeSecrets;
use crate::fake_sleep::FakeSleepInhibitor;
use crate::ftp_client::FtpClient;
use psm_core::Result;
use psm_core::{
    domain::photo::{NewPhoto, Photo, PhotoSource},
    events::AppEvent,
    ports::{camera::CaptureSink, clock::Clock, events::EventSink, store::Store},
    services::{
        attributes::AttributesService, history::HistoryService, items::ItemsService,
        review::ReviewService, sessions::SessionsService,
    },
};
use psm_infra::paths::AppPaths;
use psm_integrations::App;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};
use tempfile::TempDir;

/// `2026-11-06T10:00:00.000Z`, the day the sidecar scripts are stamped (P46 L1):
/// `with_camera_script` starts the clock here so script stamps and the clock share one day.
pub const SCRIPT_EPOCH_MS: i64 = 1_793_959_200_000;

pub struct TestApp {
    pub dir: TempDir,
    pub app: App,
    pub clock: Arc<ManualClock>,
    pub fake: Arc<FakeCaptureOneScripting>,
    pub events: Arc<CollectingEventSink>,
    pub commerce: Option<Arc<FakeCommerce>>,
    pub secrets: Arc<FakeSecrets>,
    /// The process names the ownership guard reads (P48; empty by default).
    pub fake_processes: Arc<FakeProcessList>,
    /// The idle-sleep assertion (P48).
    pub fake_sleep: Arc<FakeSleepInhibitor>,
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
    pub fn with_test_manifest(
        &self,
        manifest: psm_integrations::camera::support::manifest::Manifest,
    ) {
        psm_integrations::camera::support::with_test_manifest(&self.app.paths, manifest);
    }
    pub fn bundled_sidecar_path(&self) -> PathBuf {
        let source = std::env::var_os("CARGO_BIN_EXE_fake_sidecar")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::current_exe()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("fake_sidecar")
            });
        let target = self.dir.path().join("bundled/camera-sidecar");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(source, &target).unwrap();
        std::fs::create_dir_all(target.parent().unwrap().join("rec")).unwrap();
        std::fs::write(
            target.parent().unwrap().join("rec/behaviour.json"),
            "{\"version_sdk\":\"2.02.00\"}",
        )
        .unwrap();
        let plist = self.dir.path().join("entitlements.plist");
        std::fs::write(&plist, "<?xml version=\"1.0\" encoding=\"UTF-8\"?><plist version=\"1.0\"><dict><key>com.apple.security.cs.disable-library-validation</key><true/></dict></plist>").unwrap();
        assert!(
            std::process::Command::new("codesign")
                .args([
                    "-s",
                    "-",
                    "--force",
                    "--options",
                    "runtime",
                    "--entitlements"
                ])
                .arg(&plist)
                .arg(&target)
                .status()
                .unwrap()
                .success()
        );
        psm_integrations::camera::support::with_test_bundled(&self.app.paths, target.clone());
        target
    }
    pub fn support_archive(&self, entries: &[(&str, &[u8])]) -> PathBuf {
        use std::io::Write;
        let path = self.dir.path().join(format!(
            "support-{}.zip",
            self.next_hook.fetch_add(1, Ordering::SeqCst)
        ));
        let file = std::fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        for (name, bytes) in entries {
            zip.start_file(
                format!("CrSDK_v2.02.00_Mac/external/crsdk/{name}"),
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap();
        path
    }
    pub fn camera_support_install(
        &mut self,
        archive: &Path,
        bundled: &Path,
    ) -> Result<psm_integrations::camera::support::InstalledSupport> {
        let support =
            psm_integrations::camera::support::sony::install(&self.app.paths, archive, bundled)?;
        self.app
            .camera_runtime
            .set_bundled_sidecar_path(bundled.to_path_buf());
        self.app.camera_runtime.support_changed();
        self.app.reopen()?;
        Ok(support)
    }
    /// Deliver a camera file to the ledger, then let the watcher accept its final name.
    pub fn arrival_photo(&self, _session_id: i64, name: &str) -> Result<String> {
        use sha2::{Digest, Sha256};
        let bytes = name.as_bytes();
        let staged = self
            .capture_root()
            .join(".incoming")
            .join(format!(".{name}.part"));
        std::fs::create_dir_all(staged.parent().unwrap())?;
        std::fs::write(&staged, bytes)?;
        let hash = Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        self.ledger().delivery(psm_core::domain::camera::Delivery {
            camera_filename: name.into(),
            written_path: staged,
            sha256: hash,
            size: bytes.len() as u64,
            via: psm_core::domain::camera::Link::Wireless,
            completed_at: self.clock.now_ms(),
        })?;
        let final_path = self
            .ledger()
            .deliveries()?
            .last()
            .expect("delivery recorded")
            .final_path
            .clone();
        self.app.arrival.watcher_event(Path::new(&final_path))?;
        self.tick(1_000)?;
        Ok(final_path)
    }
    pub fn settings_set(&mut self, patch: &serde_json::Value) -> Result<serde_json::Value> {
        self.app.settings_set(patch)
    }
    pub fn items(&self) -> ItemsService<'_> {
        ItemsService::new(self.app.store.as_ref(), self.clock.as_ref())
    }
    pub fn sessions(&self) -> SessionsService<'_> {
        SessionsService::new(self.app.store.as_ref(), self.clock.as_ref())
            .with_editing(
                self.app.editing.as_ref(),
                self.app.health.as_ref(),
                &self.app.provision,
            )
            .with_camera(self.app.camera.as_ref())
            .with_follow(self.app.follow.as_ref())
    }
    /// "Take the camera back" over the app's camera runtime (P48).
    pub fn take_back(&self) -> psm_integrations::camera::direct::TakeBackService {
        self.app
            .camera_runtime
            .take_back()
            .expect("camera runtime bound")
    }
    /// The process-wide editor-launch gate (P48).
    pub fn launch_gate(
        &self,
    ) -> Arc<psm_integrations::editing::capture_one::launch_gate::LaunchGate> {
        self.app.launch_gate()
    }
    /// The user-lane `GatedScripting` (P48).
    pub fn user_lane(
        &self,
    ) -> Arc<dyn psm_integrations::editing::capture_one::scripting::CaptureOneScripting> {
        self.app.user_lane()
    }
    pub fn review(&self) -> ReviewService<'_> {
        ReviewService::new(
            self.app.store.as_ref(),
            self.clock.as_ref(),
            self.app.history.as_ref(),
        )
    }
    pub fn history(&self) -> HistoryService<'_> {
        HistoryService::new(
            self.app.store.as_ref(),
            self.clock.as_ref(),
            self.app.history.as_ref(),
        )
    }
    pub fn attributes<'a>(&'a self, integration: Option<&'a str>) -> AttributesService<'a> {
        AttributesService::new(self.app.store.as_ref(), self.clock.as_ref(), integration)
            .with_history(self.app.history.as_ref())
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
    pub fn composition(&self, photo_id: i64) -> Result<psm_core::domain::composition::Composition> {
        self.app.store.composition(photo_id)
    }
    pub fn new() -> Result<Self> {
        Self::create(false, false, "custom.damage_report", None, 0)
    }
    pub fn with_fake_document() -> Result<Self> {
        Self::create(true, false, "custom.damage_report", None, 0)
    }
    pub fn with_fake_shop() -> Result<Self> {
        Self::create(true, true, "custom.damage_report", None, 0)
    }
    pub fn with_fake_shop_key(key: &str) -> Result<Self> {
        Self::create(true, true, key, None, 0)
    }
    pub fn with_camera() -> Result<Self> {
        Self::with_camera_behaviour(serde_json::json!({}))
    }
    pub fn with_camera_behaviour(behaviour: serde_json::Value) -> Result<Self> {
        Self::create(true, false, "custom.damage_report", Some(behaviour), 0)
    }
    /// A camera app whose fake sidecar replays `script` (P45 §The fake sidecar): JSON lines are
    /// printed (with `{capture_id}`/`{cmd_id}` from the last `capture` command), plus `sleep <ms>`,
    /// `await <cmd>`, `exit <code>`, `stderr <text>`; no `fake.started` line. The clock starts at
    /// `SCRIPT_EPOCH_MS` (P46 L1); after the script the fake runs `inject` lines.
    pub fn with_camera_script(script: &str) -> Result<Self> {
        Self::with_camera_script_cache_mode(script, true)
    }
    pub fn with_camera_script_running(script: &str) -> Result<Self> {
        Self::with_camera_script_running_and_delays(script, 0, 0)
    }
    pub fn with_camera_script_running_and_delays(
        script: &str,
        call_delay_ms: u64,
        document_info_delay_ms: u64,
    ) -> Result<Self> {
        Self::create(
            true,
            false,
            "custom.damage_report",
            Some(serde_json::json!({
                "script_text": script,
                "run_startup": true,
                "fake_call_delay_ms": call_delay_ms,
                "fake_document_info_delay_ms": document_info_delay_ms
            })),
            SCRIPT_EPOCH_MS,
        )
    }
    pub fn with_camera_script_no_cache(script: &str) -> Result<Self> {
        Self::with_camera_script_cache_mode(script, false)
    }
    fn with_camera_script_cache_mode(script: &str, cache_enabled: bool) -> Result<Self> {
        Self::create(
            true,
            false,
            "custom.damage_report",
            Some(
                serde_json::json!({ "script_text": script, "secret_cache_enabled": cache_enabled }),
            ),
            SCRIPT_EPOCH_MS,
        )
    }
    fn create(
        configure_document: bool,
        configure_shop: bool,
        condition_key: &str,
        sidecar: Option<serde_json::Value>,
        clock_start: i64,
    ) -> Result<Self> {
        let dir = tempfile::tempdir()?;
        let mut paths = AppPaths::for_dir(dir.path());
        paths.home = dir.path().to_path_buf();
        paths.create()?;
        let with_camera = sidecar.is_some();
        let mut secret_cache_enabled = true;
        let mut run_startup = false;
        let mut fake_call_delay_ms = 0;
        let mut fake_document_info_delay_ms = 0;
        if let Some(mut behaviour) = sidecar {
            psm_integrations::camera::support::with_test_manifest(
                &paths,
                psm_integrations::camera::support::manifest::Manifest {
                    version: "test".into(),
                    files: vec![],
                },
            );
            let support = paths.root.join("camera/sony/test");
            let rec = support.join("rec");
            std::fs::create_dir_all(&rec)?;
            // `with_camera_script`: the script file sits beside behaviour.json, both written
            // before the first spawn.
            if let Some(text) = behaviour
                .as_object_mut()
                .and_then(|o| o.remove("script_text"))
            {
                let script = rec.join("script.txt");
                std::fs::write(&script, text.as_str().unwrap_or_default())?;
                behaviour["script"] = serde_json::Value::String(script.to_string_lossy().into());
            }
            if let Some(enabled) = behaviour
                .as_object_mut()
                .and_then(|o| o.remove("secret_cache_enabled"))
                .and_then(|value| value.as_bool())
            {
                secret_cache_enabled = enabled;
            }
            if let Some(enabled) = behaviour
                .as_object_mut()
                .and_then(|o| o.remove("run_startup"))
                .and_then(|v| v.as_bool())
            {
                run_startup = enabled;
            }
            if let Some(delay) = behaviour
                .as_object_mut()
                .and_then(|o| o.remove("fake_call_delay_ms"))
                .and_then(|value| value.as_u64())
            {
                fake_call_delay_ms = delay;
            }
            if let Some(delay) = behaviour
                .as_object_mut()
                .and_then(|o| o.remove("fake_document_info_delay_ms"))
                .and_then(|value| value.as_u64())
            {
                fake_document_info_delay_ms = delay;
            }
            std::fs::write(
                rec.join("behaviour.json"),
                serde_json::to_vec(&behaviour).unwrap(),
            )?;
            let source = std::env::var_os("CARGO_BIN_EXE_fake_sidecar")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    let exe = std::env::current_exe().expect("test exe");
                    exe.parent().unwrap().parent().unwrap().join("fake_sidecar")
                });
            std::fs::copy(source, support.join("camera-sidecar"))?;
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                support.join("camera-sidecar"),
                std::fs::Permissions::from_mode(0o755),
            )?;
        }
        let document_id = dir
            .path()
            .join("Capture One Session")
            .to_string_lossy()
            .into_owned();
        let capture_root = dir
            .path()
            .join("Capture One Session")
            .join("Capture")
            .to_string_lossy()
            .into_owned();
        std::fs::create_dir_all(&document_id)?;
        std::fs::create_dir_all(&capture_root)?;
        std::fs::write(
            Path::new(&document_id).join("Capture One Session.cosessiondb"),
            b"",
        )?;
        let mut settings = psm_infra::settings_file::Settings::open(&paths)?;
        if with_camera {
            settings.camera.receiver_port = 0;
            settings.camera.passive_port_start = 0;
            settings.camera.passive_port_end = 0;
        }
        if configure_document {
            settings.document_id = Some(document_id.clone());
            // Exports stay inside the test's temporary directory, never under ~/Pictures.
            settings.export_folder = dir.path().join("Export").to_string_lossy().into_owned();
            std::fs::write(
                paths.settings(),
                serde_json::to_vec(&settings)
                    .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?,
            )?;
        }
        let fake = Arc::new(FakeCaptureOneScripting::new(document_id, capture_root));
        {
            let mut state = fake.state.lock().unwrap();
            state.call_delay_ms = fake_call_delay_ms;
            state.delay_document_info_ms = fake_document_info_delay_ms;
        }
        let clock = Arc::new(ManualClock::new(clock_start));
        fake.set_clock(clock.clone());
        let secrets = Arc::new(FakeSecrets::default());
        if !secret_cache_enabled {
            secrets.disable_cache();
        }
        use psm_core::ports::secrets::SecretStore;
        secrets.set("access-auth-user", "u name")?;
        secrets.set("access-auth-password", "p/w")?;
        secrets.set("ftp-password", "ftp-pw-unique-1")?;
        let commerce = configure_shop.then(|| {
            Arc::new(FakeCommerce::with_condition_key(
                clock.clone(),
                condition_key,
            ))
        });
        let app = App::open_with_adapter(
            paths,
            fake.clone(),
            clock.clone(),
            run_startup,
            commerce
                .as_ref()
                .map(|c| c.clone() as Arc<dyn psm_core::ports::commerce::CommerceAdapter>),
            secrets.clone(),
        )?;
        let events = Arc::new(CollectingEventSink::default());
        app.events.set_target(events.clone());
        let fake_processes = Arc::new(FakeProcessList::default());
        let fake_sleep = Arc::new(FakeSleepInhibitor::default());
        app.camera_runtime.set_process_list(fake_processes.clone());
        app.camera_runtime.set_sleep_inhibitor(fake_sleep.clone());
        roomy_disk(&app);
        Ok(Self {
            dir,
            app,
            clock,
            fake,
            events,
            commerce,
            secrets,
            fake_processes,
            fake_sleep,
            next_hook: AtomicU64::new(1),
        })
    }
    pub fn fake_shop(&self) -> &FakeCommerce {
        self.commerce.as_deref().expect("fake commerce configured")
    }
    pub fn kill_and_restart(&mut self) -> Result<()> {
        self.restart(false)
    }
    pub fn restart_with_startup(&mut self) -> Result<()> {
        self.restart(true)
    }
    fn restart(&mut self, run_startup: bool) -> Result<()> {
        let paths = self.app.paths.clone();
        let next = App::open_with_adapter(
            paths,
            self.fake.clone(),
            self.clock.clone(),
            run_startup,
            self.commerce
                .as_ref()
                .map(|c| c.clone() as Arc<dyn psm_core::ports::commerce::CommerceAdapter>),
            self.secrets.clone(),
        )?;
        self.app_seams(&next);
        let old = std::mem::replace(&mut self.app, next);
        drop(old);
        self.app.events.set_target(self.events.clone());
        Ok(())
    }
    fn app_seams(&self, app: &App) {
        app.camera_runtime
            .set_process_list(self.fake_processes.clone());
        app.camera_runtime
            .set_sleep_inhibitor(self.fake_sleep.clone());
        roomy_disk(app);
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
        self.app.follow.tick()?;
        let count = self.app.jobs.tick(self.clock.now_ms())?;
        // Output renders before publish steps, so a publish sees its export done in the same tick.
        let count = count + self.app.render_jobs.tick(self.clock.now_ms())?;
        let count = count + self.app.publish_jobs.tick(self.clock.now_ms())?;
        self.app.previews.tick()?;
        self.app.camera_runtime.tick(self.clock.mono());
        Ok(count)
    }
    pub fn camera_runtime(&self) -> Arc<psm_integrations::camera::runtime::CameraRuntime> {
        self.app.camera_runtime.clone()
    }
    /// `CameraService::capture` over the app's camera provider.
    pub fn capture(&self) -> Result<psm_core::domain::camera::CaptureId> {
        psm_core::services::camera::CameraService::new(
            self.app.camera.as_ref(),
            self.app.health.as_ref(),
        )
        .capture()
    }
    /// `CameraService::status` over the app's camera provider.
    pub fn camera_status(&self) -> Result<psm_core::domain::camera::CameraStatus> {
        psm_core::services::camera::CameraService::new(
            self.app.camera.as_ref(),
            self.app.health.as_ref(),
        )
        .status()
    }
    fn inject_line(&self, line: &str) -> Result<()> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.app.paths.root.join("camera/sony/test/rec/inject.txt"))?;
        file.write_all(format!("{line}\n").as_bytes())?;
        Ok(())
    }
    /// Has the fake sidecar print `{timestamp: <clock now>, source: "sdk", event, ..fields}`
    /// (after its script ended; P46 L14). The stamp is the test clock, never wall time.
    pub fn inject(&self, event: &str, fields: serde_json::Value) -> Result<()> {
        self.inject_line(&self.inject_text(event, fields))
    }
    /// Several injected lines in one write (the fake prints them back to back).
    pub fn inject_all(&self, lines: &[(&str, serde_json::Value)]) -> Result<()> {
        let text: Vec<String> = lines
            .iter()
            .map(|(event, fields)| self.inject_text(event, fields.clone()))
            .collect();
        self.inject_line(&text.join("\n"))
    }
    fn inject_text(&self, event: &str, fields: serde_json::Value) -> String {
        let stamp = chrono::DateTime::from_timestamp_millis(self.clock.now_ms())
            .expect("clock in range")
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();
        let mut line = serde_json::json!({ "timestamp": stamp, "source": "sdk", "event": event });
        if let (Some(line), Some(fields)) = (line.as_object_mut(), fields.as_object()) {
            for (k, v) in fields {
                line.insert(k.clone(), v.clone());
            }
        }
        line.to_string()
    }
    /// Has the fake sidecar exit with `code`.
    pub fn inject_exit(&self, code: i32) -> Result<()> {
        self.inject_line(&format!("exit {code}"))
    }
    /// The Save-to-PC staging folder (`--save-dir`): `<capture root>/.incoming/sdk-<run id>`.
    pub fn staging_dir(&self) -> PathBuf {
        self.capture_root()
            .join(".incoming")
            .join(format!("sdk-{}", self.app.camera_runtime.run_id()))
    }
    /// Writes `bytes` to `staging_dir()/<name>` and injects its `sdk.download.completed`.
    pub fn sdk_download(&self, name: &str, bytes: &[u8]) -> Result<PathBuf> {
        let path = self.staging_dir().join(name);
        std::fs::create_dir_all(self.staging_dir())?;
        std::fs::write(&path, bytes)?;
        self.inject(
            "sdk.download.completed",
            serde_json::json!({
                "filename": path.to_string_lossy(),
                "size_bytes": bytes.len(),
                "success": true,
                "transport": "usb",
            }),
        )?;
        Ok(path)
    }
    /// The events the sidecar client delivered (the runtime's ring buffer).
    pub fn sidecar_events(&self) -> Vec<psm_integrations::camera::sidecar::protocol::Stamped> {
        self.app.camera_runtime.sidecar_events()
    }
    pub fn receiver_port(&self) -> u16 {
        self.app
            .camera_runtime
            .receiver_port()
            .expect("receiver running")
    }
    pub fn ftp(&self) -> FtpClient {
        let mut c = FtpClient::connect(self.receiver_port()).unwrap();
        assert!(
            c.login("sessionmanager", "ftp-pw-unique-1")
                .unwrap()
                .starts_with("230")
        );
        c
    }
    /// The capture ledger of the current binding (re-fetch after a restart or a settings save).
    pub fn ledger(&self) -> Arc<psm_core::services::camera_ledger::LedgerService> {
        self.app.camera_runtime.ledger().expect("ledger bound")
    }
    /// The ledger's `pending` and `done` delivery rows, as the deliveries that produced them.
    pub fn deliveries(&self) -> Vec<psm_core::domain::camera::Delivery> {
        self.ledger()
            .deliveries()
            .unwrap()
            .into_iter()
            .map(|row| psm_core::domain::camera::Delivery {
                camera_filename: row.camera_filename,
                written_path: PathBuf::from(row.written_path),
                sha256: row.sha256,
                size: row.size,
                via: row.via,
                completed_at: row.completed_at,
            })
            .collect()
    }
    /// The next rename the ledger attempts fails with an I/O error (not `EEXIST`); later ones
    /// use the real `CameraFiles`.
    pub fn ledger_fs_fail_next_rename(&self) {
        self.app
            .camera_runtime
            .set_delivery_fs(Arc::new(FailNextRename::default()));
    }
    /// Fixture write (no input path sets it in 10E): the exposure's expected file count.
    pub fn set_expected_files(&self, exposure_id: i64, n: u32) -> Result<()> {
        let conn = rusqlite::Connection::open(self.app.paths.db())
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        conn.execute(
            "UPDATE camera_exposures SET expected_files=?2 WHERE id=?1",
            rusqlite::params![exposure_id, n],
        )
        .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        Ok(())
    }
    pub fn capture_root(&self) -> PathBuf {
        std::fs::canonicalize(
            self.app
                .settings
                .effective_document_id(&self.app.paths)
                .join("Capture"),
        )
        .unwrap()
    }
    pub fn set_fake_sidecar_behaviour(&self, behaviour: serde_json::Value) -> Result<()> {
        std::fs::write(
            self.app
                .paths
                .root
                .join("camera/sony/test/rec/behaviour.json"),
            serde_json::to_vec(&behaviour).unwrap(),
        )?;
        Ok(())
    }
    pub fn fake_sidecar_recording(&self) -> Recording {
        let rec = self.app.paths.root.join("camera/sony/test/rec");
        let lines = |name: &str| {
            std::fs::read_to_string(rec.join(name))
                .unwrap_or_default()
                .lines()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        let pids: Vec<u32> = lines("pids")
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        Recording {
            argv: pids
                .iter()
                .map(|p| {
                    serde_json::from_slice(
                        &std::fs::read(rec.join(format!("argv.{p}.json"))).unwrap_or_default(),
                    )
                    .unwrap_or_default()
                })
                .collect(),
            env: pids
                .iter()
                .map(|p| {
                    serde_json::from_slice(
                        &std::fs::read(rec.join(format!("env.{p}.json"))).unwrap_or_default(),
                    )
                    .unwrap_or_default()
                })
                .collect(),
            stdin: lines("stdin.log")
                .iter()
                .filter_map(|s| {
                    s.split_once(' ')
                        .and_then(|(p, l)| Some((p.parse().ok()?, l.to_owned())))
                })
                .collect(),
            exits: lines("exits")
                .iter()
                .filter_map(|s| {
                    s.split_once(' ')
                        .and_then(|(p, c)| Some((p.parse().ok()?, c.parse().ok()?)))
                })
                .collect(),
            pids,
        }
    }
    pub fn focus(&self) -> Result<()> {
        self.app.previews.on_focus()
    }
    pub fn complete_render(&self, job_id: &str) -> Result<PathBuf> {
        self.complete_render_with(job_id, &FakeCaptureOneScripting::output_bytes(job_id))
    }
    pub fn complete_render_with(&self, job_id: &str, bytes: &[u8]) -> Result<PathBuf> {
        self.complete_render_as(job_id, bytes, "jpg")
    }
    /// Completes master render `job_id` with `master` written as Capture One's TIFF output.
    pub fn complete_master_render(
        &self,
        job_id: &str,
        master: &crate::fixtures::synthetic_master::SyntheticMaster,
    ) -> Result<PathBuf> {
        self.complete_render_as(job_id, &master.bytes(), "tif")
    }
    /// Completes master render `job_id` with `bytes` as Capture One's TIFF output (P69: the
    /// 8-bit master, `SyntheticMaster::bytes_8bit`).
    pub fn complete_master_render_bytes(&self, job_id: &str, bytes: &[u8]) -> Result<PathBuf> {
        self.complete_render_as(job_id, bytes, "tif")
    }
    /// Whether `job_id` was issued with the master recipe of this app's settings.
    pub fn is_master_job(&self, job_id: &str) -> bool {
        self.fake
            .state
            .lock()
            .unwrap()
            .issued
            .iter()
            .any(|(job, _, recipe)| job == job_id && *recipe == self.app.settings.master_recipe)
    }
    /// Completes preview render `job_id` as Capture One does with the app's `PSM Preview` recipe:
    /// the JPEG in `<document>/Output/_session_manager/` (10C; P61).
    pub fn complete_preview_export(&self, job_id: &str) -> Result<PathBuf> {
        let dir = self
            .app
            .settings
            .effective_document_id(&self.app.paths)
            .join("Output/_session_manager");
        self.complete_render_in(
            job_id,
            &FakeCaptureOneScripting::output_bytes(job_id),
            "jpg",
            &dir,
        )
    }
    fn complete_render_as(&self, job_id: &str, bytes: &[u8], extension: &str) -> Result<PathBuf> {
        self.complete_render_in(job_id, bytes, extension, &self.dir.path().join("Output"))
    }
    fn complete_render_in(
        &self,
        job_id: &str,
        bytes: &[u8],
        extension: &str,
        dir: &Path,
    ) -> Result<PathBuf> {
        let entry = self
            .app
            .editing
            .render_completion(job_id)
            .ok_or_else(|| psm_core::Error::NotFound(job_id.into()))?;
        // Named as the fake Capture One names it (P56: "add suffix" when configured); the hook
        // reports what Capture One reports.
        let (output, reported) = self.fake.write_output(
            &entry.raw,
            dir,
            extension,
            bytes,
            std::time::UNIX_EPOCH
                + std::time::Duration::from_millis(self.clock.now_ms().max(0) as u64),
        )?;
        self.deliver_hook(&serde_json::json!({"hook":"processing_done","job":job_id,"raw":entry.raw,"outputs":[reported]}).to_string())?;
        Ok(output)
    }
    /// Completes every render still in flight (in issue order), including renders the adapter
    /// queued behind a busy base name and issued as earlier ones finished: a master job gets a
    /// 600×400 solid synthetic master, a preview the stub JPEG. Returns the job ids.
    pub fn complete_all_renders(&self) -> Result<Vec<String>> {
        let mut completed = Vec::new();
        loop {
            let open: Vec<String> = self
                .fake
                .state
                .lock()
                .unwrap()
                .issued
                .iter()
                .map(|(job, _, _)| job.clone())
                .filter(|job| self.app.editing.render_completion(job).is_some())
                .collect();
            if open.is_empty() {
                return Ok(completed);
            }
            for job in open {
                if self.is_master_job(&job) {
                    self.complete_master_render(
                        &job,
                        &crate::fixtures::synthetic_master::SyntheticMaster::solid(
                            600,
                            400,
                            crate::fixtures::synthetic_master::PAPER,
                        ),
                    )?;
                } else {
                    self.complete_render(&job)?;
                }
                completed.push(job);
            }
        }
    }
    /// Decodes an image file (an exported output) to 8-bit RGB.
    pub fn decode(&self, path: &Path) -> Result<image::RgbImage> {
        image::open(path)
            .map(|img| img.to_rgb8())
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))
    }
    /// `outputs/<pk[0..2]>/<pk>/` of the RAW (the output cache).
    pub fn outputs_dir(&self, raw: &str) -> PathBuf {
        self.media_dir("outputs", raw)
    }
    /// `masters/<pk[0..2]>/<pk>/` of the RAW (the master cache).
    pub fn masters_dir(&self, raw: &str) -> PathBuf {
        self.media_dir("masters", raw)
    }
    fn media_dir(&self, kind: &str, raw: &str) -> PathBuf {
        let key = psm_core::mechanisms::arrival::path_key(raw);
        self.app.paths.root.join(kind).join(&key[..2]).join(key)
    }
    /// `<export folder>` from the settings of this app (inside the test's temporary directory).
    pub fn export_root(&self) -> PathBuf {
        PathBuf::from(&self.app.settings.export_folder)
    }
    pub fn write_sidecar(&self, raw: &str, mtime: SystemTime) -> Result<PathBuf> {
        let raw = Path::new(raw);
        let folder = raw.parent().unwrap().join("CaptureOne/Settings1680");
        std::fs::create_dir_all(&folder)?;
        let path = folder.join(format!(
            "{}.cos",
            raw.file_name().unwrap().to_string_lossy()
        ));
        std::fs::write(&path, b"settings")?;
        std::fs::File::options()
            .write(true)
            .open(&path)?
            .set_times(std::fs::FileTimes::new().set_modified(mtime))?;
        Ok(path)
    }
    pub fn watcher_event(&self, path: &std::path::Path) -> Result<()> {
        self.app.arrival.watcher_event(path)
    }
    pub fn scan(&self) -> Result<()> {
        self.app.arrival.scan()
    }
}

/// The master-render disk guard (P56 C3(d)) sees plenty of space unless a test says otherwise, so no
/// test depends on the free space of the machine it runs on.
fn roomy_disk(app: &App) {
    app.disk_space().set(Arc::new(|_: &Path| {
        Ok(psm_infra::disk::FreeSpace {
            available_bytes: 1 << 50,
            disk: "the test disk".into(),
        })
    }));
}

/// `CameraFiles` whose first `rename_exclusive` fails like a non-APFS volume would (`ENOTSUP`).
#[derive(Default)]
pub struct FailNextRename {
    failed: std::sync::atomic::AtomicBool,
}
impl psm_core::ports::camera::DeliveryFs for FailNextRename {
    fn rename_exclusive(
        &self,
        from: &Path,
        to: &Path,
    ) -> Result<psm_core::ports::camera::RenameOutcome> {
        if !self.failed.swap(true, Ordering::SeqCst) {
            return Err(std::io::Error::from_raw_os_error(libc::ENOTSUP).into());
        }
        psm_infra::camera_files::CameraFiles.rename_exclusive(from, to)
    }
    fn create_dir_all(&self, dir: &Path) -> Result<()> {
        psm_infra::camera_files::CameraFiles.create_dir_all(dir)
    }
    fn file_size(&self, path: &Path) -> Result<u64> {
        psm_infra::camera_files::CameraFiles.file_size(path)
    }
}

pub struct Recording {
    pub pids: Vec<u32>,
    pub argv: Vec<Vec<String>>,
    pub env: Vec<std::collections::HashMap<String, String>>,
    pub stdin: Vec<(u32, String)>,
    pub exits: Vec<(u32, i32)>,
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

impl TestApp {
    /// Inserts a `publish_runs` row committed with `confirmed_at` = now and returns the mutation
    /// token built from it by the one public constructor (07A MC-10).
    pub fn confirmed_run(&self) -> Result<psm_core::domain::publish::ConfirmedRun> {
        let conn = rusqlite::Connection::open(self.app.paths.db())
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        conn.execute(
            "INSERT INTO publish_runs(integration,created_at,confirmed_at,confirmed_summary) VALUES ('shopify',?1,?1,'test run')",
            [self.clock.now_ms()],
        )
        .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        psm_core::domain::publish::ConfirmedRun::from_persisted(
            self.store(),
            conn.last_insert_rowid(),
        )
    }
}
