//! The direct camera adapter (10E MC-31 behaviour, MC-33 integration): the `CameraProvider` that
//! drives the camera sidecar through the runtime's protocol client and turns its events into the
//! neutral model (P46 plan §The adapter contract is the authority for the event table).
use super::{
    filenames::normalize_camera_filename,
    ownership::{HolderReport, name_programs},
    start_order::StartReason,
};
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
const OWNED_CODES: [&str; 12] = [
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
    "camera_ftp_unreachable",
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
    /// A start-order condition seen in this batch (10E MC-34 R-I31), the first one.
    pub start: Option<StartReason>,
    /// The idle-sleep assertion changes (R-I23): `Some(true)` hold, `Some(false)` release; only
    /// emitted on a change, so a repeated USB `connection.ready` never holds twice (P48 L15).
    pub sleep: Option<bool>,
    pub fingerprint_pending: Vec<(String, bool)>,
    pub usb_present: Option<bool>,
    pub usb_ready: Option<bool>,
}
impl StepOutcome {
    fn start(&mut self, reason: StartReason) {
        self.start.get_or_insert(reason);
    }
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
    /// The owner process of the last `usb.claimed_by` and the report built for it (R-I30).
    holder: Option<(String, HolderReport)>,
    /// How many `usb.claimed_by` events arrived (the take-back's step 4 watches it).
    claims: u64,
    /// Whether the batch wants the idle-sleep assertion held (a USB session is open).
    want_sleep: Option<bool>,
    /// Whether the adapter last asked for the assertion to be held.
    sleep_held: bool,
    pending_fingerprint: Option<(String, bool)>,
    usb_present: Option<bool>,
    auto_waiting: Option<String>,
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
        self.add_with_message(reason, holder, None);
    }
    fn add_with_message(
        &mut self,
        reason: NeedsActionReason,
        holder: Option<String>,
        message: Option<String>,
    ) {
        self.remove(&reason);
        self.inputs.needs_action.push((reason, holder, message));
    }
    fn remove(&mut self, reason: &NeedsActionReason) {
        self.inputs
            .needs_action
            .retain(|(r, _, _)| std::mem::discriminant(r) != std::mem::discriminant(reason));
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
                holder: None,
                claims: 0,
                want_sleep: None,
                sleep_held: false,
                pending_fingerprint: None,
                usb_present: None,
                auto_waiting: None,
            }),
        }
    }

    /// The program holding the camera's USB interface, while that is reported (R-I30): the
    /// owner process of the last `usb.claimed_by` and its report.
    fn held(&self) -> Option<(String, HolderReport)> {
        let state = locked(&self.state);
        state
            .inputs
            .needs_action
            .iter()
            .any(|(r, _, _)| *r == NeedsActionReason::HeldByOtherProgram)
            .then(|| state.holder.clone())
            .flatten()
    }

    /// Another process holds the USB interface (`usb.claimed_by` is the latest word on it).
    pub fn cable_held(&self) -> bool {
        self.held().is_some()
    }

    /// R-I32: the USB interface is held on Capture One's behalf (the take-back's last check).
    pub fn holder_is_capture_one(&self) -> bool {
        self.held()
            .is_some_and(|(_, report)| report.holder.as_deref() == Some("Capture One"))
    }

    /// The owner process behind the status, while the status is `HeldByOtherProgram`.
    pub fn status_holder_process(&self) -> Option<String> {
        let state = locked(&self.state);
        match derive_status(&state.inputs) {
            CameraStatus::NeedsAction {
                reason: NeedsActionReason::HeldByOtherProgram,
                ..
            } => state.holder.as_ref().map(|(owner, _)| owner.clone()),
            _ => None,
        }
    }

    /// Our SDK session runs over the cable.
    pub fn usb_session(&self) -> bool {
        matches!(locked(&self.state).inputs.session, Some((_, Link::Cable)))
    }
    pub fn current_link(&self) -> Option<Link> {
        locked(&self.state)
            .inputs
            .session
            .as_ref()
            .map(|(_, link)| link.clone())
    }
    /// Retry a disconnected helper immediately, with an observable status before its next tick.
    pub fn retry(&self) -> Result<CameraStatus> {
        self.runtime()?.send_command(Command::UsbRetry {
            cmd_id: String::new(),
        })?;
        let mut state = locked(&self.state);
        let link = if state.usb_present == Some(true) {
            Link::Cable
        } else {
            Link::Wireless
        };
        state.inputs.connecting = Some(link);
        state.inputs.busy = None;
        Ok(derive_status(&state.inputs))
    }
    pub fn confirm_fingerprint(&self, shown: &str) -> Result<()> {
        let (fingerprint, replace) = locked(&self.state)
            .pending_fingerprint
            .as_ref()
            .cloned()
            .ok_or(Error::FingerprintNotPending)?;
        if fingerprint != shown {
            return Err(Error::FingerprintChanged);
        }
        self.runtime()?.send_command(Command::AcceptFingerprint {
            fingerprint,
            replace,
            cmd_id: String::new(),
        })?;
        Ok(())
    }
    pub fn reject_fingerprint(&self) -> Result<()> {
        if locked(&self.state).pending_fingerprint.is_none() {
            return Err(Error::FingerprintNotPending);
        }
        Ok(())
    }

    /// How many `usb.claimed_by` events arrived so far.
    pub fn claims_seen(&self) -> u64 {
        locked(&self.state).claims
    }

    pub fn set_auto_waiting(&self, message: Option<String>) {
        let mut state = locked(&self.state);
        if state.auto_waiting == message {
            return;
        }
        state.auto_waiting = message.clone();
        let default_message = state
            .holder
            .as_ref()
            .and_then(|(_, report)| report.sentence.clone());
        for (reason, _, shown) in &mut state.inputs.needs_action {
            if *reason == NeedsActionReason::HeldByOtherProgram {
                *shown = message.clone().or_else(|| default_message.clone());
            }
        }
    }
    pub fn set_ftp_unreachable(&self, message: Option<String>) {
        let mut state = locked(&self.state);
        state.remove(&NeedsActionReason::FtpUnreachable);
        if let Some(message) = message {
            state.add_with_message(NeedsActionReason::FtpUnreachable, None, Some(message));
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
                    out.usb_ready = Some(false);
                    state.inputs.session = None;
                    state.inputs.connecting = None;
                    state.inputs.busy = None;
                    state.expected_transfer = None;
                    sink.transfer_in_progress(Link::Cable, false);
                    // A crashed sidecar must not leave the Mac unable to sleep (P48 L15).
                    state.want_sleep = Some(false);
                    out.start(StartReason::SidecarUnavailable);
                }
            }
        }
        if let Some(want) = state.want_sleep.take()
            && want != state.sleep_held
        {
            state.sleep_held = want;
            out.sleep = Some(want);
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
        if let SidecarEvent::SshFingerprint {
            status,
            fingerprint: Some(fp),
            ..
        } = &stamped.event
            && (status == "unknown" || status == "MISMATCH")
        {
            out.fingerprint_pending
                .push((fp.clone(), status == "MISMATCH"));
        }
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
                out.start(StartReason::SidecarUnavailable);
            }
            SidecarEvent::UsbClaimedBy { owners, .. } => {
                out.usb_ready = Some(false);
                // R-I30: one process listing per claim, read outside the state lock.
                let owner = holder_from_owners(owners);
                let running = self
                    .runtime
                    .upgrade()
                    .map(|rt| rt.process_list().running())
                    .unwrap_or_else(|| Ok(Vec::new()))
                    .unwrap_or_else(|error| {
                        tracing::warn!(%error, "process list unavailable");
                        Vec::new()
                    });
                let report = name_programs(&owner, &running);
                let mut state = locked(&self.state);
                state.add_with_message(
                    NeedsActionReason::HeldByOtherProgram,
                    report.holder.clone(),
                    report.sentence.clone(),
                );
                state.holder = Some((owner, report));
                state.claims += 1;
                if let Some(waiting) = state.auto_waiting.clone() {
                    for (reason, _, shown) in &mut state.inputs.needs_action {
                        if *reason == NeedsActionReason::HeldByOtherProgram {
                            *shown = Some(waiting.clone());
                        }
                    }
                }
            }
            SidecarEvent::UsbInitial { present } => {
                out.usb_present = Some(*present);
                locked(&self.state).usb_present = Some(*present);
                if !present {
                    out.start(StartReason::NoCameraOnCable);
                }
            }
            SidecarEvent::UsbArrived { .. } => {
                out.usb_present = Some(true);
                locked(&self.state).usb_present = Some(true);
            }
            SidecarEvent::UsbRemoved { .. } => {
                out.usb_present = Some(false);
                locked(&self.state).usb_present = Some(false);
            }
            SidecarEvent::UsbFallbackToWifi { .. } => {
                out.start(StartReason::FallbackToWifi);
                locked(&self.state).inputs.connecting = Some(Link::Wireless);
            }
            SidecarEvent::ConnectionReady {
                transport: Transport::Usb,
                ..
            } => {
                out.start(StartReason::UsbReady);
                out.usb_ready = Some(true);
                self.apply_status(&stamped.event, sink);
            }
            SidecarEvent::SessionClosed {
                transport: Transport::Usb,
                ..
            }
            | SidecarEvent::SessionLost {
                transport: Transport::Usb,
                ..
            } => {
                out.usb_ready = Some(false);
                self.apply_status(&stamped.event, sink);
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
                "DISCONNECTED" | "RECOVERING" => {
                    // Retryable gaps follow discovery/connect in the same stdout batch. Keep
                    // the link visible through backoff until ready or a terminal error.
                    state.inputs.busy = None;
                }
                "ERROR" => {
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
                    Link::Cable => {
                        state.remove(&NeedsActionReason::HeldByOtherProgram);
                        state.want_sleep = Some(true);
                    }
                    Link::Wireless => {
                        state.remove(&NeedsActionReason::AuthenticationRefused);
                        state.remove(&NeedsActionReason::FingerprintMismatch);
                        // One SDK session at a time: a Wi-Fi session means no USB session.
                        state.want_sleep = Some(false);
                    }
                }
            }
            SidecarEvent::SessionClosed { transport, .. }
            | SidecarEvent::SessionLost { transport, .. }
                if *transport == Transport::Usb =>
            {
                state.want_sleep = Some(false);
                Self::end_session(&mut state, event, sink);
            }
            SidecarEvent::SessionClosed { .. } | SidecarEvent::SessionLost { .. } => {
                Self::end_session(&mut state, event, sink);
            }
            SidecarEvent::CameraBusy { reason, .. } => match reason.as_str() {
                "exposure" => state.inputs.busy = Some(BusyReason::Exposure),
                "file_transfer" => state.inputs.busy = Some(BusyReason::Transfer),
                "none" => state.inputs.busy = None,
                _ => {}
            },
            SidecarEvent::SshFingerprint {
                status,
                fingerprint,
                ..
            } => match status.as_str() {
                "unknown" => {
                    state.add(NeedsActionReason::FingerprintUnconfirmed, None);
                    if let Some(fp) = fingerprint {
                        state.pending_fingerprint = Some((fp.clone(), false));
                    }
                }
                "MISMATCH" => {
                    state.add(NeedsActionReason::FingerprintMismatch, None);
                    if let Some(fp) = fingerprint {
                        state.pending_fingerprint = Some((fp.clone(), true));
                    }
                }
                "match" | "pinned" => {
                    state.pending_fingerprint = None;
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

    /// The session ended (`session.closed` keeps a transport switch's busy state).
    fn end_session(state: &mut AdapterState, event: &SidecarEvent, sink: &dyn CaptureSink) {
        state.inputs.session = None;
        if state.expected_transfer.take().is_some() {
            sink.transfer_in_progress(Link::Cable, false);
        }
        let switching = matches!(event, SidecarEvent::SessionClosed { .. })
            && state.inputs.busy == Some(BusyReason::Switching);
        if !switching {
            state.inputs.busy = None;
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
