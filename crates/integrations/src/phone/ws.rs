//! PH5: `GET /api/phone/v1/ws?pair=<token>`. Each socket is a viewer of the live view hub (a
//! [`LiveOutlet`]): frames by demand through a one-slot mailbox, events through a queue of at most
//! 64 messages; a writer task sends them, drops a frame of an older demand generation, pings every
//! `ping`, and closes after `idle_close` without a message (4408), on a frame write slower than
//! `write_timeout` (4408), on a full event queue (4409) and when the pairing is removed (4401).
//! The hub's `offer`/`event` only `put` and wake the writer (HB: never blocking under the hub's
//! mutex).
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use axum::{
    Json,
    extract::{
        RawQuery, State,
        ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use psm_core::{
    Error,
    domain::{
        camera::CameraStatus,
        live::{
            CaptureReportState, FocusFrameState, FocusOutcome, FocusRefusal, LiveEvent, LiveFrame,
            LiveState, ViewerDemand, ViewerId,
        },
    },
    events::AppEvent,
    ports::{events::EventSink, store::Store},
    services::live_view::{LatestSlot, LiveOutlet},
};
use serde_json::json;

use crate::{
    AppHandles,
    phone::{auth::token_of, routes::verify, server::Shared, wire},
};

/// PH5: at most this many event messages wait for a socket (then it is closed, 4409).
pub const EVENT_QUEUE_MAX: usize = 64;

/// The camera as the phone is told about it (`{type: "camera", …}`), shared by the server.
#[derive(Default, Clone, PartialEq)]
pub(crate) struct CameraView {
    status: Option<String>,
    live: Option<(String, Option<String>)>,
    exposure: Option<(String, String, String)>,
}

impl CameraView {
    fn message(&self) -> String {
        let (live, reason) = self
            .live
            .clone()
            .unwrap_or_else(|| ("off".to_owned(), None));
        json!({
            "type": "camera",
            "status": self.status.clone().unwrap_or_else(|| "disconnected".into()),
            "live": live,
            "reason": reason,
            "exposure": self.exposure.as_ref().map(|(shutter, aperture, iso)| json!({
                "shutter": shutter, "aperture": aperture, "iso": iso,
            })),
        })
        .to_string()
    }
}

fn status_name(status: &CameraStatus) -> &'static str {
    match status {
        CameraStatus::Connected { .. } => "connected",
        CameraStatus::Connecting { .. } => "connecting",
        CameraStatus::Busy { .. } => "busy",
        CameraStatus::NeedsAction { .. } => "needs_action",
        CameraStatus::Disconnected => "disconnected",
    }
}

fn live_of(state: &LiveState) -> (String, Option<String>) {
    match state {
        LiveState::Off => ("off".into(), None),
        LiveState::Unavailable { reason } => ("unavailable".into(), Some(reason.clone())),
        LiveState::Live { .. } => ("live".into(), None),
    }
}

/// The server's state for the sockets: the camera view, the active session and the `rev` counter
/// of `session` messages, fed by the App's events (one observer, carried across App rebuilds).
#[derive(Default)]
pub(crate) struct Feed {
    camera: Mutex<CameraView>,
    active: Mutex<Option<i64>>,
    rev: AtomicU64,
    /// The one observer of the App's events (carried from hub to hub on a rebuild, LV3·C28).
    bridge: std::sync::OnceLock<Arc<dyn EventSink>>,
}

impl Feed {
    fn camera(&self) -> std::sync::MutexGuard<'_, CameraView> {
        self.camera.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Observes the App's event hub for the sockets (`session` and `camera` messages).
struct AppEventsBridge(Weak<Shared>);

impl EventSink for AppEventsBridge {
    fn emit(&self, event: AppEvent) {
        let Some(shared) = self.0.upgrade() else {
            return;
        };
        let feed = &shared.feed;
        let message = match event {
            AppEvent::ActiveSessionChanged { session_id } => {
                *feed.active.lock().unwrap_or_else(|e| e.into_inner()) = session_id;
                Some(session_message(feed, session_id))
            }
            AppEvent::SessionChanged { session_id }
            | AppEvent::PhotoAdded { session_id, .. }
            | AppEvent::PhotoChanged { session_id, .. } => {
                let active = *feed.active.lock().unwrap_or_else(|e| e.into_inner());
                (active == Some(session_id)).then(|| session_message(feed, active))
            }
            AppEvent::CameraStatusChanged { status } => {
                let mut camera = feed.camera();
                let name = Some(status_name(&status).to_owned());
                (camera.status != name).then(|| {
                    camera.status = name;
                    camera.message()
                })
            }
            _ => None,
        };
        if let Some(message) = message {
            shared.sockets.broadcast(&message);
        }
    }
}

fn session_message(feed: &Feed, session_id: Option<i64>) -> String {
    let rev = feed.rev.fetch_add(1, Ordering::Relaxed) + 1;
    json!({"type": "session", "session_id": session_id, "rev": rev}).to_string()
}

/// Makes sure the App's event hub feeds the sockets, and reads the active session and the
/// camera's status (outside any store lock: called on a blocking thread). The observer is
/// registered as carried: a settings save that rebuilds the App hands it to the new hub, so open
/// sockets keep hearing without a request or a reconnect (LV3·C28).
pub(crate) fn ensure_fed(shared: &Arc<Shared>, handles: &AppHandles) {
    let bridge = shared
        .feed
        .bridge
        .get_or_init(|| Arc::new(AppEventsBridge(Arc::downgrade(shared))))
        .clone();
    handles.events.observe_carried(bridge);
    if let Ok(active) = handles.store.active_session_id() {
        *shared.feed.active.lock().unwrap_or_else(|e| e.into_inner()) = active;
    }
    if let Ok(status) = handles.camera.status() {
        shared.feed.camera().status = Some(status_name(&status).to_owned());
    }
    let live = live_of(&shared.hub.state());
    shared.feed.camera().live = Some(live);
}

/// One socket's mailboxes (the hub's outlet).
pub(crate) struct PhoneOutlet {
    frames: LatestSlot<(Arc<LiveFrame>, u32)>,
    events: Mutex<VecDeque<String>>,
    overflow: AtomicBool,
    wake: tokio::sync::Notify,
    feed: Weak<Shared>,
    offered: AtomicU32,
    sent: AtomicU32,
}

impl PhoneOutlet {
    fn new(feed: Weak<Shared>) -> Self {
        Self {
            frames: LatestSlot::default(),
            events: Mutex::new(VecDeque::new()),
            overflow: AtomicBool::new(false),
            wake: tokio::sync::Notify::new(),
            feed,
            offered: AtomicU32::new(0),
            sent: AtomicU32::new(0),
        }
    }

    /// Queues a text message (≤ 64 waiting; more marks the socket for closing, 4409).
    pub(crate) fn push(&self, message: String) {
        let mut queue = self.events.lock().unwrap_or_else(|e| e.into_inner());
        if queue.len() >= EVENT_QUEUE_MAX {
            self.overflow.store(true, Ordering::Relaxed);
        } else {
            queue.push_back(message);
        }
        drop(queue);
        self.wake.notify_one();
    }

    fn pop(&self) -> Option<String> {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .pop_front()
    }
}

impl LiveOutlet for PhoneOutlet {
    fn offer(&self, frame: Arc<LiveFrame>, generation: u32) {
        self.offered.fetch_add(1, Ordering::Relaxed);
        self.frames.put((frame, generation));
        self.wake.notify_one();
    }
    fn withdraw(&self) {
        self.frames.clear();
    }
    fn event(&self, event: &LiveEvent) {
        let message = match event {
            LiveEvent::Focus { focus_id, outcome } => {
                let (name, reason) = match outcome {
                    FocusOutcome::Focused => ("focused", None),
                    FocusOutcome::NotFocused => ("not_focused", None),
                    FocusOutcome::NoReport => ("no_report", None),
                    FocusOutcome::Superseded => ("superseded", None),
                    FocusOutcome::Failed => ("failed", None),
                    FocusOutcome::Refused(r) => (
                        "refused",
                        Some(match r {
                            FocusRefusal::AreaNotMovable => "focus_area_not_movable",
                            FocusRefusal::NotConnected => "not_connected",
                            FocusRefusal::Unsupported => "unsupported",
                            FocusRefusal::BadPoint => "bad_point",
                        }),
                    ),
                };
                json!({"type": "focus", "focus_id": focus_id.0, "outcome": name, "reason": reason})
            }
            LiveEvent::FocusFrames(frames) => json!({
                "type": "focus_frames",
                "frames": frames.iter().map(|f| json!({
                    "cx": f.cx, "cy": f.cy, "w": f.w, "h": f.h,
                    "state": match f.state {
                        FocusFrameState::Focused => "focused",
                        FocusFrameState::NotFocused => "not_focused",
                        FocusFrameState::Unknown => "unknown",
                    },
                })).collect::<Vec<_>>(),
            }),
            LiveEvent::Capture { capture_id, state } => json!({
                "type": "capture",
                "capture_id": capture_id,
                "state": match state {
                    CaptureReportState::Acknowledged => "acknowledged",
                    CaptureReportState::Exposed => "exposed",
                    CaptureReportState::Failed => "failed",
                },
            }),
            LiveEvent::State(state) => {
                let Some(shared) = self.feed.upgrade() else {
                    return;
                };
                let mut camera = shared.feed.camera();
                camera.live = Some(live_of(state));
                self.push(camera.message());
                return;
            }
            LiveEvent::Exposure(e) => {
                let Some(shared) = self.feed.upgrade() else {
                    return;
                };
                let mut camera = shared.feed.camera();
                camera.exposure = Some((e.shutter.clone(), e.aperture.clone(), e.iso.clone()));
                self.push(camera.message());
                return;
            }
        };
        self.push(message.to_string());
    }
}

/// The upgrade: the token is checked first (401, no upgrade).
pub(crate) async fn upgrade(
    State(shared): State<Arc<Shared>>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
    ws: WebSocketUpgrade,
) -> Response {
    let token = token_of(&headers, query.as_deref());
    let checked = tokio::task::spawn_blocking({
        let shared = shared.clone();
        move || -> psm_core::Result<i64> {
            let handles = (shared.handles)()?;
            let pairing = verify(&handles, token.as_deref())?;
            ensure_fed(&shared, &handles);
            Ok(pairing.id)
        }
    })
    .await
    .unwrap_or_else(|e| Err(Error::InvalidData(e.to_string())));
    match checked {
        Ok(pairing) => ws.on_upgrade(move |socket| serve(shared, pairing, socket)),
        Err(error) => {
            let status = StatusCode::from_u16(wire::error_code(&error).0)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            tracing::info!(
                target: "psm::phone",
                "phone.request route=/api/phone/v1/ws status={} pairing=none",
                status.as_u16()
            );
            (status, Json(wire::error_body(&error))).into_response()
        }
    }
}

fn close(code: u16, reason: &'static str) -> Message {
    Message::Close(Some(CloseFrame {
        code,
        reason: reason.into(),
    }))
}

/// One socket: a hub viewer until it closes.
async fn serve(shared: Arc<Shared>, pairing: i64, socket: WebSocket) {
    let outlet = Arc::new(PhoneOutlet::new(Arc::downgrade(&shared)));
    let viewer = shared.hub.subscribe(outlet.clone());
    let (socket_id, revoked) = shared.sockets.add(pairing, outlet.clone());
    tracing::info!(
        target: "psm::phone",
        "phone.viewer opened viewer={} pairing={pairing} reason=open",
        viewer.0
    );
    let (mut sink, mut stream) = socket.split();
    let timings = shared.timings;
    let generation = Arc::new(AtomicU32::new(0));
    let last_heard = Arc::new(Mutex::new(Instant::now()));
    let reader_done = Arc::new(tokio::sync::Notify::new());
    let reader = tokio::spawn({
        let (shared, outlet, generation, last_heard, reader_done) = (
            shared.clone(),
            outlet.clone(),
            generation.clone(),
            last_heard.clone(),
            reader_done.clone(),
        );
        async move {
            while let Some(Ok(message)) = stream.next().await {
                *last_heard.lock().unwrap_or_else(|e| e.into_inner()) = Instant::now();
                match message {
                    Message::Text(text) => {
                        if let Some(demand) = demand_of(text.as_str()) {
                            generation.store(demand.r#gen, Ordering::Relaxed);
                            shared.hub.set_demand(viewer, demand);
                            log_demand(viewer, pairing, demand, &outlet);
                            outlet.push(
                                json!({"type": "demand_ack", "gen": demand.r#gen}).to_string(),
                            );
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            reader_done.notify_one();
        }
    });
    let _ = sink
        .send(Message::Text(
            json!({"type": "hello", "viewer_id": viewer.0})
                .to_string()
                .into(),
        ))
        .await;
    // The camera as it is now (later messages follow its changes).
    let camera = shared.feed.camera().message();
    outlet.push(camera);
    let mut ping = tokio::time::interval(timings.ping);
    ping.tick().await;
    let mut report = tokio::time::interval(Duration::from_secs(10));
    report.tick().await;
    let reason = loop {
        tokio::select! {
            _ = outlet.wake.notified() => {}
            _ = ping.tick() => {
                let heard = *last_heard.lock().unwrap_or_else(|e| e.into_inner());
                if heard.elapsed() >= timings.idle_close {
                    let _ = tokio::time::timeout(timings.write_timeout, sink.send(close(4408, "idle"))).await;
                    break "idle";
                }
                if tokio::time::timeout(timings.write_timeout, sink.send(Message::Ping(Vec::new().into()))).await.map_or(true, |r| r.is_err()) {
                    break "write";
                }
                continue;
            }
            _ = report.tick() => {
                log_demand_counts(viewer, pairing, &outlet);
                continue;
            }
            _ = revoked.notified() => {
                let _ = tokio::time::timeout(timings.write_timeout, sink.send(close(4401, "pairing removed"))).await;
                break "revoked";
            }
            _ = reader_done.notified() => break "closed",
        }
        if outlet.overflow.load(Ordering::Relaxed) {
            let _ = tokio::time::timeout(
                timings.write_timeout,
                sink.send(close(4409, "too many events")),
            )
            .await;
            break "overflow";
        }
        let mut failed = None;
        while let Some(text) = outlet.pop() {
            match tokio::time::timeout(timings.write_timeout, sink.send(Message::Text(text.into())))
                .await
            {
                Ok(Ok(())) => {}
                Ok(Err(_)) => {
                    failed = Some("write");
                    break;
                }
                Err(_) => {
                    failed = Some("write_timeout");
                    break;
                }
            }
        }
        if let Some(reason) = failed {
            if reason == "write_timeout" {
                let _ = tokio::time::timeout(timings.write_timeout, sink.send(close(4408, "slow")))
                    .await;
            }
            break reason;
        }
        if let Some((frame, frame_gen)) = outlet.frames.take() {
            if frame_gen != generation.load(Ordering::Relaxed) {
                continue;
            }
            let mut bytes = Vec::with_capacity(32 + frame.jpeg.len());
            bytes.extend_from_slice(&wire::frame_header(
                frame.seq,
                frame_gen,
                frame.width,
                frame.height,
                frame.captured_at_ms,
            ));
            bytes.extend_from_slice(&frame.jpeg);
            match tokio::time::timeout(
                timings.write_timeout,
                sink.send(Message::Binary(bytes.into())),
            )
            .await
            {
                Ok(Ok(())) => {
                    outlet.sent.fetch_add(1, Ordering::Relaxed);
                }
                Ok(Err(_)) => break "write",
                Err(_) => {
                    let _ =
                        tokio::time::timeout(timings.write_timeout, sink.send(close(4408, "slow")))
                            .await;
                    break "write_timeout";
                }
            }
        }
    };
    shared.hub.unsubscribe(viewer);
    shared.sockets.remove(socket_id);
    reader.abort();
    tracing::info!(
        target: "psm::phone",
        "phone.viewer closed viewer={} pairing={pairing} reason={reason}",
        viewer.0
    );
}

/// `{type: "demand", gen, live, visible}`.
fn demand_of(text: &str) -> Option<ViewerDemand> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    if value.get("type")?.as_str()? != "demand" {
        return None;
    }
    Some(ViewerDemand {
        r#gen: u32::try_from(value.get("gen")?.as_u64()?).ok()?,
        live: value.get("live")?.as_bool()?,
        visible: value.get("visible")?.as_bool()?,
    })
}

/// `(sent, skipped)` since the last report: a frame offered and not sent was skipped (HB4).
fn counts(outlet: &PhoneOutlet, reset: bool) -> (u32, u32) {
    let (offered, sent) = if reset {
        (
            outlet.offered.swap(0, Ordering::Relaxed),
            outlet.sent.swap(0, Ordering::Relaxed),
        )
    } else {
        (
            outlet.offered.load(Ordering::Relaxed),
            outlet.sent.load(Ordering::Relaxed),
        )
    };
    (sent, offered.saturating_sub(sent))
}

fn log_demand(viewer: ViewerId, pairing: i64, demand: ViewerDemand, outlet: &PhoneOutlet) {
    let (sent, skipped) = counts(outlet, false);
    tracing::info!(
        target: "psm::live",
        "live.viewer viewer={} pairing={pairing} live={} visible={} sent_10s={sent} skipped_10s={skipped}",
        viewer.0,
        demand.live,
        demand.visible,
    );
}

fn log_demand_counts(viewer: ViewerId, pairing: i64, outlet: &PhoneOutlet) {
    let (sent, skipped) = counts(outlet, true);
    tracing::info!(
        target: "psm::live",
        "live.viewer viewer={} pairing={pairing} sent_10s={sent} skipped_10s={skipped}",
        viewer.0,
    );
}
