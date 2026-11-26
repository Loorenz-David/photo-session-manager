//! LV2·U5 — the phone's WebSocket (`master_plan_live.md` §5 PH5, HB3–HB5): rows LV2·C24–C29,
//! WebSocket clients connected to `TestPhone`; frames published into the TestApp's live view hub
//! by the test (a started source and `CameraFrame`s), hub events reported by the test.
use futures_util::{SinkExt, StreamExt};
use psm_core::domain::{
    camera::Link,
    live::{
        CameraFrame, CaptureReportState, FocusId, FocusOutcome, LiveEvent, PublishOutcome,
        SourceStop,
    },
};
use psm_integrations::phone::{pairing::PhonePairings, server::PhoneTimings};
use psm_testkit::{TestApp, phone::TestPhone};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
const SOURCE: u64 = (1 << 32) | 1;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}
async fn connect(phone: &TestPhone, token: &str) -> Ws {
    let (ws, _) = connect_async(phone.ws_url(&format!("/api/phone/v1/ws?pair={token}")))
        .await
        .unwrap();
    ws
}
/// A client with a fixed 64 KB receive buffer (the OS grows an unset one), so a client that does
/// not read holds little in its network stack.
async fn connect_small_buffer(phone: &TestPhone, token: &str) -> Ws {
    let socket = tokio::net::TcpSocket::new_v4().unwrap();
    socket.set_recv_buffer_size(64 * 1024).unwrap();
    let stream = socket
        .connect(phone.server.local_addr().unwrap())
        .await
        .unwrap();
    let (ws, _) = tokio_tungstenite::client_async(
        phone.ws_url(&format!("/api/phone/v1/ws?pair={token}")),
        MaybeTlsStream::Plain(stream),
    )
    .await
    .unwrap();
    ws
}
/// The next message (pings answered by the client itself are skipped), or `None` after `within`.
async fn next(ws: &mut Ws, within: Duration) -> Option<Message> {
    let end = tokio::time::Instant::now() + within;
    loop {
        match tokio::time::timeout_at(end, ws.next()).await {
            Ok(Some(Ok(Message::Ping(_) | Message::Pong(_)))) => continue,
            Ok(Some(Ok(message))) => return Some(message),
            _ => return None,
        }
    }
}
fn text(message: &Message) -> Option<Value> {
    match message {
        Message::Text(t) => serde_json::from_str(t.as_str()).ok(),
        _ => None,
    }
}
/// Reads until a text message of `kind` (others are dropped); panics after 2 s.
async fn until(ws: &mut Ws, kind: &str) -> Value {
    let end = Instant::now() + Duration::from_secs(2);
    while Instant::now() < end {
        if let Some(message) = next(ws, Duration::from_millis(200)).await
            && let Some(value) = text(&message)
            && value["type"] == kind
        {
            return value;
        }
    }
    panic!("no {kind} message");
}
/// Sends a demand and waits for its `demand_ack`.
async fn demand(ws: &mut Ws, r#gen: u32, live: bool, visible: bool) {
    ws.send(Message::Text(
        json!({"type": "demand", "gen": r#gen, "live": live, "visible": visible})
            .to_string()
            .into(),
    ))
    .await
    .unwrap();
    loop {
        let value = until(ws, "demand_ack").await;
        if value["gen"] == r#gen {
            return;
        }
    }
}
/// The binary messages that arrive within `quiet` of each other.
async fn binaries(ws: &mut Ws, quiet: Duration) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    while let Some(message) = next(ws, quiet).await {
        if let Message::Binary(b) = message {
            out.push(b.to_vec());
        }
    }
    out
}
fn frame(n: u32, len: usize) -> CameraFrame {
    CameraFrame {
        source: SOURCE,
        frame_no: n,
        width: 640,
        height: 424,
        jpeg: vec![(n % 251) as u8; len],
        link: Link::Cable,
    }
}
fn publish(a: &TestApp, n: u32, len: usize) -> u64 {
    match a.live_hub().publish(frame(n, len)) {
        PublishOutcome::Forwarded { seq } => seq,
        other => panic!("{other:?}"),
    }
}
fn header(b: &[u8]) -> (u64, u32) {
    (
        u64::from_le_bytes(b[8..16].try_into().unwrap()),
        u32::from_le_bytes(b[16..20].try_into().unwrap()),
    )
}
async fn settle(pred: impl Fn() -> bool, within: Duration) -> bool {
    let end = Instant::now() + within;
    while Instant::now() < end {
        if pred() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    pred()
}
fn close_code(message: Option<Message>) -> Option<u16> {
    match message {
        Some(Message::Close(Some(frame))) => Some(u16::from(frame.code)),
        _ => None,
    }
}
/// The close frame that arrives (after any other message), within `within`.
async fn closed(ws: &mut Ws, within: Duration) -> Option<u16> {
    let end = tokio::time::Instant::now() + within;
    loop {
        match tokio::time::timeout_at(end, ws.next()).await {
            Ok(Some(Ok(Message::Close(frame)))) => {
                return close_code(Some(Message::Close(frame)));
            }
            Ok(Some(Ok(_))) => continue,
            _ => return None,
        }
    }
}

// ---- LV2·C24: the upgrade ----

#[test]
fn lv2_c24a_no_token_or_a_revoked_one_gets_401_and_no_upgrade() {
    let a = TestApp::with_camera().unwrap();
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    let pairings = PhonePairings::new(a.store(), a.clock.as_ref());
    pairings
        .revoke(pairings.verify(&token).unwrap().unwrap().id)
        .unwrap();
    runtime().block_on(async {
        for url in [
            phone.ws_url("/api/phone/v1/ws"),
            phone.ws_url(&format!("/api/phone/v1/ws?pair={token}")),
        ] {
            match connect_async(url).await {
                Err(tokio_tungstenite::tungstenite::Error::Http(response)) => {
                    assert_eq!(response.status().as_u16(), 401)
                }
                other => panic!("upgraded: {:?}", other.map(|_| ())),
            }
        }
    });
    assert_eq!(a.live_hub().viewer_count(), 0);
}

#[test]
fn lv2_c24b_the_first_message_is_hello_and_the_hub_has_one_more_viewer() {
    let a = TestApp::with_camera().unwrap();
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    let before = a.live_hub().viewer_count();
    runtime().block_on(async {
        let mut ws = connect(&phone, &token).await;
        let first = text(&next(&mut ws, Duration::from_secs(2)).await.unwrap()).unwrap();
        assert_eq!(first["type"], "hello");
        assert!(first["viewer_id"].is_u64());
        assert_eq!(a.live_hub().viewer_count(), before + 1);
    });
}

// ---- LV2·C25: frames by demand ----

#[test]
fn lv2_c25a_no_frames_before_a_demand() {
    let a = TestApp::with_camera().unwrap();
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    runtime().block_on(async {
        let mut ws = connect(&phone, &token).await;
        until(&mut ws, "hello").await;
        a.live_hub().source_started(Link::Cable, SOURCE);
        for n in 1..=10 {
            publish(&a, n, 64);
        }
        assert!(
            binaries(&mut ws, Duration::from_millis(300))
                .await
                .is_empty()
        );
    });
}

#[test]
fn lv2_c25bcd_frames_follow_the_demand_generation() {
    let a = TestApp::with_camera().unwrap();
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    runtime().block_on(async {
        let mut ws = connect(&phone, &token).await;
        until(&mut ws, "hello").await;
        a.live_hub().source_started(Link::Cable, SOURCE);
        publish(&a, 1, 64);
        // (b) gen 3, Live and visible: every frame published after the ack, in order.
        demand(&mut ws, 3, true, true).await;
        let newest_at_ack = a.live_hub().newest().unwrap().seq;
        let mut seqs = Vec::new();
        for n in 2..=6 {
            publish(&a, n, 64);
            let got = binaries(&mut ws, Duration::from_millis(300)).await;
            assert_eq!(got.len(), 1, "frame {n}");
            let (seq, r#gen) = header(&got[0]);
            assert_eq!(r#gen, 3);
            seqs.push(seq);
        }
        assert!(seqs.windows(2).all(|w| w[0] < w[1]), "{seqs:?}");
        assert!(seqs[0] > newest_at_ack);
        // (c) gen 4, not Live: nothing after the ack.
        demand(&mut ws, 4, false, true).await;
        for n in 7..=16 {
            publish(&a, n, 64);
        }
        assert!(
            binaries(&mut ws, Duration::from_millis(300))
                .await
                .is_empty()
        );
        // (d) gen 5, Live but not visible: nothing.
        demand(&mut ws, 5, true, false).await;
        for n in 17..=26 {
            publish(&a, n, 64);
        }
        assert!(
            binaries(&mut ws, Duration::from_millis(300))
                .await
                .is_empty()
        );
    });
}

// ---- LV2·C26: a slow viewer skips frames, the other gets all ----

#[test]
fn lv2_c26a_a_viewer_that_does_not_read_gets_only_the_newest_frame() {
    let a = TestApp::with_camera().unwrap();
    // Long limits: the slow viewer is kept open while it does not read.
    let phone = TestPhone::start_with(
        &a,
        PhoneTimings {
            ping: Duration::from_secs(30),
            idle_close: Duration::from_secs(60),
            stall_close: Duration::from_secs(30),
        },
    );
    let token = phone.pair(&a);
    runtime().block_on(async {
        let mut slow = connect_small_buffer(&phone, &token).await;
        let mut fast = connect(&phone, &token).await;
        demand(&mut slow, 1, true, true).await;
        demand(&mut fast, 1, true, true).await;
        a.live_hub().source_started(Link::Cable, SOURCE);
        // 1 MB frames: more than the sockets' buffers hold, so a viewer that does not read
        // blocks its writer and its slot keeps only the newest frame.
        let mut last = 0;
        let mut fast_got = 0;
        for n in 1..=50 {
            last = publish(&a, n, 1 << 20);
            loop {
                match next(&mut fast, Duration::from_secs(5)).await {
                    Some(Message::Binary(b)) => {
                        assert_eq!(header(&b).0, last);
                        fast_got += 1;
                        break;
                    }
                    Some(_) => continue,
                    None => panic!("frame {n} not received by the reading viewer"),
                }
            }
        }
        assert_eq!(fast_got, 50);
        let got = binaries(&mut slow, Duration::from_millis(500)).await;
        let seqs: Vec<u64> = got.iter().map(|b| header(b).0).collect();
        assert!(
            !got.is_empty() && got.len() <= 2,
            "{} frames: {seqs:?}, last {last}",
            got.len()
        );
        assert_eq!(header(got.last().unwrap()).0, last);
    });
}

// ---- LV2·C27: events reach every viewer ----

#[test]
fn lv2_c27abcd_events_reach_every_viewer_whatever_its_demand() {
    let a = TestApp::with_camera().unwrap();
    let s = a
        .items()
        .create_local("LV2-WS", "Chair", "Chair", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    runtime().block_on(async {
        let mut x = connect(&phone, &token).await;
        let mut y = connect(&phone, &token).await;
        demand(&mut x, 1, true, true).await;
        until(&mut y, "hello").await;
        // (a) focus
        a.live_hub().report(LiveEvent::Focus {
            focus_id: FocusId("k".into()),
            outcome: FocusOutcome::Focused,
        });
        for ws in [&mut x, &mut y] {
            assert_eq!(
                until(ws, "focus").await,
                json!({"type": "focus", "focus_id": "k", "outcome": "focused", "reason": null})
            );
        }
        // (b) a photo arrives in the active session: `session` with a higher `rev`
        a.arrival_photo(s, "DSC00100.ARW").unwrap();
        let first_x = until(&mut x, "session").await;
        let first_y = until(&mut y, "session").await;
        assert_eq!(
            (
                first_x["session_id"].as_i64(),
                first_y["session_id"].as_i64()
            ),
            (Some(s), Some(s))
        );
        a.arrival_photo(s, "DSC00101.ARW").unwrap();
        for (ws, first) in [(&mut x, &first_x), (&mut y, &first_y)] {
            let second = until(ws, "session").await;
            assert_eq!(second["session_id"].as_i64(), Some(s));
            assert!(
                second["rev"].as_u64() > first["rev"].as_u64(),
                "{second} after {first}"
            );
        }
        // (c) a capture report
        a.live_hub().report(LiveEvent::Capture {
            capture_id: "c1".into(),
            state: CaptureReportState::Exposed,
        });
        for ws in [&mut x, &mut y] {
            assert_eq!(
                until(ws, "capture").await,
                json!({"type": "capture", "capture_id": "c1", "state": "exposed"})
            );
        }
        // (d) live view unavailable on this camera
        a.live_hub().source_stopped(SourceStop::Unavailable {
            reason: "not_supported".into(),
        });
        for ws in [&mut x, &mut y] {
            let camera = until(ws, "camera").await;
            let camera = if camera["live"] == "unavailable" {
                camera
            } else {
                until(ws, "camera").await
            };
            assert_eq!(
                (camera["live"].as_str(), camera["reason"].as_str()),
                (Some("unavailable"), Some("not_supported"))
            );
        }
    });
}

// ---- LV2·C28: lifecycle ----

#[test]
fn lv2_c28a_a_closed_socket_leaves_the_hub() {
    let a = TestApp::with_camera().unwrap();
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    let before = a.live_hub().viewer_count();
    runtime().block_on(async {
        let mut ws = connect(&phone, &token).await;
        until(&mut ws, "hello").await;
        assert_eq!(a.live_hub().viewer_count(), before + 1);
        ws.close(None).await.unwrap();
        assert!(
            settle(
                || a.live_hub().viewer_count() == before,
                Duration::from_secs(1)
            )
            .await
        );
    });
}

#[test]
fn lv2_c28b_a_socket_that_stops_answering_is_closed_4408() {
    let a = TestApp::with_camera().unwrap();
    let phone = TestPhone::start_with(
        &a,
        PhoneTimings {
            ping: Duration::from_millis(200),
            idle_close: Duration::from_millis(600),
            stall_close: Duration::from_millis(500),
        },
    );
    let token = phone.pair(&a);
    let before = a.live_hub().viewer_count();
    runtime().block_on(async {
        let mut ws = connect(&phone, &token).await;
        until(&mut ws, "hello").await;
        // Not reading: no pong goes back.
        assert!(
            settle(
                || a.live_hub().viewer_count() == before,
                Duration::from_millis(1500)
            )
            .await
        );
        assert_eq!(closed(&mut ws, Duration::from_secs(1)).await, Some(4408));
    });
}

#[test]
fn lv2_c28c_removing_the_pairing_closes_its_socket_4401() {
    let a = TestApp::with_camera().unwrap();
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    runtime().block_on(async {
        let mut ws = connect(&phone, &token).await;
        until(&mut ws, "hello").await;
        let pairings = PhonePairings::new(a.store(), a.clock.as_ref());
        let id = pairings.verify(&token).unwrap().unwrap().id;
        assert!(phone.server.connected_pairings().contains(&id));
        pairings.revoke(id).unwrap();
        phone.server.close_pairing(id);
        assert_eq!(closed(&mut ws, Duration::from_secs(1)).await, Some(4401));
    });
}

// ---- LV2·C29: the frame message ----

#[test]
fn lv2_c29a_a_frame_is_the_header_then_the_jpeg_bytes_unchanged() {
    let a = TestApp::with_camera().unwrap();
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    runtime().block_on(async {
        let mut ws = connect(&phone, &token).await;
        demand(&mut ws, 9, true, true).await;
        a.live_hub().source_started(Link::Cable, SOURCE);
        let jpeg = vec![0xff, 0xd8, 1, 2, 3, 0xff, 0xd9];
        a.live_hub().publish(CameraFrame {
            source: SOURCE,
            frame_no: 1,
            width: 1024,
            height: 680,
            jpeg: jpeg.clone(),
            link: Link::Cable,
        });
        let newest = a.live_hub().newest().unwrap();
        let got = binaries(&mut ws, Duration::from_millis(300)).await;
        assert_eq!(got.len(), 1);
        let b = &got[0];
        assert_eq!(b.len(), 39);
        assert_eq!(&b[0..4], b"PSMF");
        assert_eq!((b[4], b[5], b[6], b[7]), (1, 0, 0, 0));
        assert_eq!(u64::from_le_bytes(b[8..16].try_into().unwrap()), newest.seq);
        assert_eq!(u32::from_le_bytes(b[16..20].try_into().unwrap()), 9);
        assert_eq!(u16::from_le_bytes(b[20..22].try_into().unwrap()), 1024);
        assert_eq!(u16::from_le_bytes(b[22..24].try_into().unwrap()), 680);
        assert_eq!(
            i64::from_le_bytes(b[24..32].try_into().unwrap()),
            newest.captured_at_ms
        );
        assert_eq!(&b[32..], jpeg.as_slice());
    });
}
