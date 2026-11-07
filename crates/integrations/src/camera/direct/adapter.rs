//! The direct camera adapter (10E MC-31 behaviour, MC-33 integration): the `CameraProvider` that
//! drives the camera sidecar through the runtime's protocol client and turns its events into the
//! neutral model (P46 plan §The adapter contract is the authority for the event table).
use super::filenames::normalize_camera_filename;
use crate::camera::{
    runtime::CameraRuntime,
    sidecar::protocol::{Command, SidecarEvent, Stamped, StoreDestination, Transport},
};
use psm_core::{
    Error, Result,
    domain::{
        HealthItem,
        camera::{
            BusyReason, CameraCapabilities, CameraName, CameraStatus, CaptureId, Delivery,
            ExposureInitiator, ExposureObserved, Link, NeedsActionReason, TRANSFER_EXPECTED_MS,
        },
    },
    mechanisms::{
        camera_status::{StatusInputs, derive_status, health_items},
        hash::sha256_file,
    },
    ports::camera::{ArrivalSource, CameraProvider, CaptureSink},
    services::health::HealthService,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, Weak},
    time::{Duration, Instant},
};

/// The health codes the adapter owns (P46 L11): every camera code it derives except
/// `camera_files_not_received` (the ledger's) — set when present, cleared otherwise.
const OWNED_CODES: [&str; 11] = [
    "camera",
    "camera_connecting",
    "camera_busy",
    "camera_disconnected",
    "camera_auth_refused",
    "camera_fingerprint_mismatch",
    "camera_fingerprint_unconfirmed",
    "camera_phone_connected",
    "camera_held",
    "camera_store_destination_unverified",
    "camera_support_not_installed",
];
const FILES_NOT_RECEIVED: &str = "camera_files_not_received";
/// The short form of the camera's FTP report is cut to this many characters (P46 L13).
const FTP_NOTE_MAX_CHARS: usize = 160;

/// What the supervisor reported besides stdout lines, in the order it reported them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelperEvent {
    Started,
    Exited,
}

/// The effects of one batch the runtime performs against the current binding.
#[derive(Debug, Default)]
pub struct StepOutcome {
    /// Lines for the runtime's helper log (redacted there).
    pub helper_log: Vec<String>,
    /// The short form of a `camera.ftp_state` that arrived in this batch.
    pub ftp_note: Option<String>,
    /// `sdk.unavailable` arrived: `camera_sidecar_unavailable` is set after the runtime's clear.
    pub sdk_unavailable: bool,
}

struct AdapterState {
    inputs: StatusInputs,
    expected_transfer: Option<Instant>,
    sdk_unavailable: bool,
    /// The `--save-dir` of the first spawn, canonical (R-I20 staging).
    staging_dir: Option<PathBuf>,
    /// The status last emitted as `camera_status_changed`.
    emitted: CameraStatus,
    /// The owned health items last published, as `(code, ok, detail)`.
    published: Option<Vec<(String, bool, String)>>,
}

pub struct DirectCameraAdapter {
    runtime: Weak<CameraRuntime>,
    state: Mutex<AdapterState>,
}

fn link_of(transport: Transport) -> Option<Link> {
    match transport {
        Transport::Usb => Some(Link::Cable),
        Transport::Wifi => Some(Link::Wireless),
        Transport::None => None,
    }
}

/// The `NeedsAction` a `connect.failed.error` text names (`camera_connection.cpp` `describe_error`,
/// with or without the callback path's `connection failed: ` prefix); `None` for any other text.
pub fn needs_action_from_connect_failed(error: &str) -> Option<NeedsActionReason> {
    if error.contains("ssh user authentication failed") {
        Some(NeedsActionReason::AuthenticationRefused)
    } else if error.contains("fingerprint mismatch") {
        Some(NeedsActionReason::FingerprintMismatch)
    } else {
        None
    }
}

/// The program holding the camera's USB interface, from `usb.claimed_by.owners` (owners joined
/// with `"; "`, each `pid <n>, <name>` or a bare name): the first owner's name.
pub fn holder_from_owners(owners: &str) -> String {
    let first = owners.split("; ").next().unwrap_or(owners);
    match first.rfind(',') {
        Some(at) => first[at + 1..].trim().to_owned(),
        None => first.trim().to_owned(),
    }
}

/// `camera FTP: k=v, …` over every field but `cmd_id`, sorted by key, strings without quotes.
fn ftp_note(fields: &serde_json::Map<String, Value>) -> String {
    let mut pairs: Vec<(&String, &Value)> = fields.iter().filter(|(k, _)| *k != "cmd_id").collect();
    pairs.sort_by(|a, b| a.0.cmp(b.0));
    let body = pairs
        .iter()
        .map(|(k, v)| match v {
            Value::String(s) => format!("{k}={s}"),
            other => format!("{k}={other}"),
        })
        .collect::<Vec<_>>()
        .join(", ");
    let note = format!("camera FTP: {body}");
    if note.chars().count() > FTP_NOTE_MAX_CHARS {
        let mut cut: String = note.chars().take(FTP_NOTE_MAX_CHARS - 1).collect();
        cut.push('…');
        cut
    } else {
        note
    }
}

/// The canonical form of `path`, also when the file itself is missing (its folder resolved).
fn canonical_lenient(path: &Path) -> PathBuf {
    if let Ok(p) = fs::canonicalize(path) {
        return p;
    }
    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => fs::canonicalize(parent)
            .map(|p| p.join(name))
            .unwrap_or_else(|_| path.to_path_buf()),
        _ => path.to_path_buf(),
    }
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl AdapterState {
    fn add(&mut self, reason: NeedsActionReason, holder: Option<String>) {
        self.remove(&reason);
        self.inputs.needs_action.push((reason, holder));
    }
    fn remove(&mut self, reason: &NeedsActionReason) {
        self.inputs
            .needs_action
            .retain(|(r, _)| std::mem::discriminant(r) != std::mem::discriminant(reason));
    }
    fn end_transfer(&mut self, sink: &dyn CaptureSink) {
        if self.expected_transfer.take().is_some() {
            sink.transfer_in_progress(Link::Cable, false);
        }
        if self.inputs.busy == Some(BusyReason::Transfer) {
            self.inputs.busy = None;
        }
    }
}

impl DirectCameraAdapter {
    pub(crate) fn new(runtime: Weak<CameraRuntime>) -> Self {
        Self {
            runtime,
            state: Mutex::new(AdapterState {
                inputs: StatusInputs {
                    needs_action: Vec::new(),
                    busy: None,
                    session: None,
                    connecting: None,
                    helper_running: false,
                },
                expected_transfer: None,
                sdk_unavailable: false,
                staging_dir: None,
                emitted: CameraStatus::Disconnected,
                published: None,
            }),
        }
    }

    /// The first spawn's `--save-dir` (kept for the life of the process, R-I20).
    pub(crate) fn set_staging_dir(&self, dir: &Path) {
        let mut state = locked(&self.state);
        if state.staging_dir.is_none() {
            state.staging_dir = Some(canonical_lenient(dir));
        }
    }

    /// Whether a Save-to-PC download is expected (the watchdog's `transfer_active`).
    pub(crate) fn transfer_expected(&self) -> bool {
        locked(&self.state).expected_transfer.is_some()
    }

    /// Mirrors the ledger's raised notice into the status inputs (P46 L10).
    pub(crate) fn set_files_not_received(&self, count: Option<usize>) {
        let mut state = locked(&self.state);
        state.remove(&NeedsActionReason::FilesNotReceived { count: 0 });
        if let Some(count) = count {
            state.inputs.needs_action.push((
                NeedsActionReason::FilesNotReceived {
                    count: count as u32,
                },
                None,
            ));
        }
    }

    /// The derived status when it differs from the one last emitted.
    pub(crate) fn status_change(&self) -> Option<CameraStatus> {
        let mut state = locked(&self.state);
        let status = derive_status(&state.inputs);
        (status != state.emitted).then(|| {
            state.emitted = status.clone();
            status
        })
    }

    /// Publishes the owned health codes from the derived inputs: set or set_ok when present,
    /// clear otherwise — when they changed, or always with `force` (a new binding's health).
    pub(crate) fn publish(&self, health: &HealthService, force: bool) {
        let mut state = locked(&self.state);
        let items: Vec<(String, bool, String)> = health_items(&state.inputs)
            .into_iter()
            .filter(|item: &HealthItem| item.code != FILES_NOT_RECEIVED)
            .map(|item| (item.code, item.ok, item.detail))
            .collect();
        if !force && state.published.as_ref() == Some(&items) {
            return;
        }
        for code in OWNED_CODES {
            match items.iter().find(|(c, _, _)| c == code) {
                Some((_, true, detail)) => health.set_ok(code, detail.clone()),
                Some((_, false, detail)) => health.set(code, detail.clone()),
                None => health.clear(code),
            }
        }
        state.published = Some(items);
    }

    /// Applies one batch: the stdout events in order, then the supervisor's side events, then the
    /// Save-to-PC deadline (P46 §The adapter contract). Hashing runs here, outside every lock but
    /// the runtime's tick serialisation.
    pub(crate) fn step(
        &self,
        batch: &[Stamped],
        side: &[HelperEvent],
        helper_running: bool,
        now: Instant,
        sink: &dyn CaptureSink,
    ) -> StepOutcome {
        let mut out = StepOutcome::default();
        for stamped in batch {
            self.apply(stamped, now, sink, &mut out);
        }
        let mut state = locked(&self.state);
        for event in side {
            match event {
                HelperEvent::Started => {
                    state.sdk_unavailable = false;
                    state.remove(&NeedsActionReason::AuthenticationRefused);
                    state.remove(&NeedsActionReason::FingerprintMismatch);
                }
                HelperEvent::Exited => {
                    state.inputs.session = None;
                    state.inputs.connecting = None;
                    state.inputs.busy = None;
                    state.expected_transfer = None;
                    sink.transfer_in_progress(Link::Cable, false);
                }
            }
        }
        state.inputs.helper_running = helper_running;
        if let Some(t) = state.expected_transfer
            && now.saturating_duration_since(t)
                >= Duration::from_millis(TRANSFER_EXPECTED_MS as u64)
        {
            state.end_transfer(sink);
        }
        out
    }

    fn apply(
        &self,
        stamped: &Stamped,
        now: Instant,
        sink: &dyn CaptureSink,
        out: &mut StepOutcome,
    ) {
        match &stamped.event {
            SidecarEvent::SdkDownloadCompleted {
                filename,
                size_bytes,
                success,
                ..
            } => self.download(
                filename,
                *size_bytes,
                *success,
                stamped.timestamp_ms,
                sink,
                out,
            ),
            SidecarEvent::CaptureExposed {
                capture_id,
                transport,
                association,
            } => {
                let camera_initiated = association.starts_with("none");
                if let Err(error) = sink.exposure_observed(ExposureObserved {
                    capture_id: capture_id
                        .clone()
                        .filter(|_| !camera_initiated)
                        .map(CaptureId),
                    at: stamped.timestamp_ms,
                    initiated: if camera_initiated {
                        ExposureInitiator::Camera
                    } else {
                        ExposureInitiator::App
                    },
                    provisional: association == "latest_release_provisional",
                }) {
                    tracing::warn!(%error, "camera exposure not recorded");
                }
                if *transport == Transport::Usb {
                    let mut state = locked(&self.state);
                    state.expected_transfer = Some(now);
                    state.inputs.busy = Some(BusyReason::Transfer);
                    drop(state);
                    sink.transfer_in_progress(Link::Cable, true);
                }
            }
            SidecarEvent::CaptureUncertain { .. } => {
                if let Err(error) = sink.session_uncertain() {
                    tracing::warn!(%error, "camera uncertain exposures not marked");
                }
            }
            SidecarEvent::CameraFtpState { fields } => out.ftp_note = Some(ftp_note(fields)),
            SidecarEvent::SdkUnavailable { .. } => {
                locked(&self.state).sdk_unavailable = true;
                out.sdk_unavailable = true;
            }
            event => self.apply_status(event, sink),
        }
    }

    fn apply_status(&self, event: &SidecarEvent, sink: &dyn CaptureSink) {
        let mut state = locked(&self.state);
        match event {
            SidecarEvent::TransportState { to, reason, .. } => match to.as_str() {
                "DISCOVERING" => {
                    if reason.ends_with("over usb") {
                        state.inputs.connecting = Some(Link::Cable);
                    } else if reason.ends_with("over wifi") {
                        state.inputs.connecting = Some(Link::Wireless);
                    }
                }
                "CONNECTING_USB" => state.inputs.connecting = Some(Link::Cable),
                "CONNECTING_WIFI" => state.inputs.connecting = Some(Link::Wireless),
                "CONNECTED_USB" | "CONNECTED_WIFI" => state.inputs.connecting = None,
                "SWITCHING" => state.inputs.busy = Some(BusyReason::Switching),
                "DISCONNECTED" | "RECOVERING" | "ERROR" => {
                    state.inputs.connecting = None;
                    state.inputs.busy = None;
                }
                _ => {}
            },
            SidecarEvent::ConnectionReady {
                transport,
                model,
                store_destination,
                ..
            } => {
                let Some(link) = link_of(*transport) else {
                    return;
                };
                state.inputs.session = Some((
                    CameraName(model.clone().unwrap_or_else(|| "Camera".into())),
                    link.clone(),
                ));
                state.inputs.connecting = None;
                state.inputs.busy = None;
                match store_destination {
                    Some(StoreDestination::Failed) => {
                        state.add(NeedsActionReason::StoreDestinationUnverified, None)
                    }
                    Some(StoreDestination::Verified) => {
                        state.remove(&NeedsActionReason::StoreDestinationUnverified)
                    }
                    None => {}
                }
                match link {
                    // Our own USB session holds the interface now (planner reading 4).
                    Link::Cable => state.remove(&NeedsActionReason::HeldByOtherProgram),
                    Link::Wireless => {
                        state.remove(&NeedsActionReason::AuthenticationRefused);
                        state.remove(&NeedsActionReason::FingerprintMismatch);
                    }
                }
            }
            SidecarEvent::SessionClosed { .. } => {
                state.inputs.session = None;
                if state.expected_transfer.take().is_some() {
                    sink.transfer_in_progress(Link::Cable, false);
                }
                if state.inputs.busy != Some(BusyReason::Switching) {
                    state.inputs.busy = None;
                }
            }
            SidecarEvent::SessionLost { .. } => {
                state.inputs.session = None;
                state.inputs.busy = None;
                if state.expected_transfer.take().is_some() {
                    sink.transfer_in_progress(Link::Cable, false);
                }
            }
            SidecarEvent::CameraBusy { reason, .. } => match reason.as_str() {
                "exposure" => state.inputs.busy = Some(BusyReason::Exposure),
                "file_transfer" => state.inputs.busy = Some(BusyReason::Transfer),
                "none" => state.inputs.busy = None,
                _ => {}
            },
            SidecarEvent::UsbClaimedBy { owners, .. } => state.add(
                NeedsActionReason::HeldByOtherProgram,
                Some(holder_from_owners(owners)),
            ),
            SidecarEvent::SshFingerprint { status, .. } => match status.as_str() {
                "unknown" => state.add(NeedsActionReason::FingerprintUnconfirmed, None),
                "MISMATCH" => state.add(NeedsActionReason::FingerprintMismatch, None),
                "match" | "pinned" => {
                    state.remove(&NeedsActionReason::FingerprintUnconfirmed);
                    state.remove(&NeedsActionReason::FingerprintMismatch);
                }
                _ => {}
            },
            SidecarEvent::ConnectFailed { error, .. } => {
                if let Some(reason) = needs_action_from_connect_failed(error) {
                    state.add(reason, None);
                }
            }
            _ => {}
        }
    }

    /// R-I20: a completed Save-to-PC download inside the staging folder is re-read, hashed and
    /// handed to the ledger as a cable delivery; anything else is logged and left where it is.
    fn download(
        &self,
        filename: &str,
        size_bytes: i64,
        success: bool,
        completed_at: i64,
        sink: &dyn CaptureSink,
        out: &mut StepOutcome,
    ) {
        let path = Path::new(filename);
        let base = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| filename.to_owned());
        let staging = locked(&self.state).staging_dir.clone();
        let refused = if !success {
            Some("failed")
        } else if size_bytes <= 0 {
            Some("size 0")
        } else if !staging.is_some_and(|dir| canonical_lenient(path).starts_with(dir)) {
            Some("outside staging")
        } else if !path.is_file() {
            Some("missing")
        } else {
            None
        };
        if let Some(reason) = refused {
            out.helper_log
                .push(format!("download ignored: {base} ({reason})"));
        } else {
            let read = fs::metadata(path)
                .map(|m| m.len())
                .map_err(Error::from)
                .and_then(|size| Ok((size, sha256_file(path)?)));
            match read {
                Ok((size, sha256)) => {
                    if let Err(error) = sink.delivery(Delivery {
                        camera_filename: normalize_camera_filename(&base),
                        written_path: path.to_path_buf(),
                        sha256,
                        size,
                        via: Link::Cable,
                        completed_at,
                    }) {
                        tracing::warn!(%error, file = %base, "camera download not placed");
                    }
                }
                Err(error) => tracing::warn!(%error, file = %base, "camera download unreadable"),
            }
        }
        locked(&self.state).end_transfer(sink);
    }

    fn runtime(&self) -> Result<std::sync::Arc<CameraRuntime>> {
        self.runtime.upgrade().ok_or(Error::SidecarUnavailable)
    }
}

impl CameraProvider for DirectCameraAdapter {
    fn raw_extensions(&self) -> &'static [&'static str] {
        &["arw"]
    }
    fn status(&self) -> Result<CameraStatus> {
        Ok(derive_status(&locked(&self.state).inputs))
    }
    /// Sends `capture` and returns its id at once: `capture.acknowledged` and `capture.exposed`
    /// arrive as events, and neither means a file exists (C3).
    fn capture(&self) -> Result<CaptureId> {
        {
            let state = locked(&self.state);
            if state.sdk_unavailable {
                return Err(Error::SidecarUnavailable);
            }
            if state.inputs.session.is_none() || !state.inputs.helper_running {
                return Err(Error::CameraNotConnected);
            }
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.runtime()?.send_command(Command::Capture {
            id: id.clone(),
            af: true,
            cmd_id: String::new(),
        })?;
        Ok(CaptureId(id))
    }
    fn capabilities(&self) -> CameraCapabilities {
        CameraCapabilities {
            status: true,
            capture: true,
            transfer: true,
            exposure_events: true,
            settings_read: true,
            settings_write_store_destination: true,
            depends_on_editing_provider: false,
        }
    }
    /// R-I29: only on a Wi-Fi session (a reconnect on USB gives the camera to Capture One, V6).
    fn reconnect(&self) -> Result<()> {
        let session = locked(&self.state).inputs.session.clone();
        match session {
            Some((_, Link::Wireless)) => {
                self.runtime()?.send_command(Command::Reconnect {
                    cmd_id: String::new(),
                })?;
                Ok(())
            }
            Some((_, Link::Cable)) => Err(Error::ReconnectOnCableRefused),
            None => Err(Error::CameraNotConnected),
        }
    }
    fn request_ftp_status(&self) -> Result<()> {
        self.runtime()?.send_command(Command::FtpStatus {
            cmd_id: String::new(),
        })?;
        Ok(())
    }
}

impl ArrivalSource for DirectCameraAdapter {
    fn via(&self) -> Link {
        Link::Cable
    }
    fn transfer_in_progress(&self) -> bool {
        self.transfer_expected()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c4c_connect_failed_texts() {
        for (text, expected) in [
            (
                "ssh user authentication failed (check Access Authen. user/password)",
                Some(NeedsActionReason::AuthenticationRefused),
            ),
            (
                "connection failed: ssh user authentication failed (check Access Authen. user/password)",
                Some(NeedsActionReason::AuthenticationRefused),
            ),
            (
                "ssh server authentication failed (fingerprint mismatch)",
                Some(NeedsActionReason::FingerprintMismatch),
            ),
            (
                "connection failed: ssh server authentication failed (fingerprint mismatch)",
                Some(NeedsActionReason::FingerprintMismatch),
            ),
            ("connection error", None),
        ] {
            assert_eq!(needs_action_from_connect_failed(text), expected, "{text}");
        }
    }

    #[test]
    fn c7a_holder_shapes() {
        for owners in [
            "pid 512, ptpcamerad",
            "ptpcamerad",
            "pid 512, ptpcamerad; pid 9, Other",
        ] {
            assert_eq!(holder_from_owners(owners), "ptpcamerad", "{owners}");
        }
    }
}
