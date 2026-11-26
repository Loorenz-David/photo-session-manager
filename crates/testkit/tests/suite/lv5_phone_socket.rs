use futures_util::{SinkExt, StreamExt};
use psm_core::domain::{
    camera::Link,
    live::{CameraFrame, PublishOutcome},
};
use psm_integrations::phone::{pairing::PhonePairings, server::PhoneTimings};
use psm_testkit::{TestApp, phone::TestPhone};
use serde_json::{Value, json};
use std::time::Duration;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
const SOURCE: u64 = (1 << 32) | 1;
fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}
fn timings(idle: Duration) -> PhoneTimings {
    PhoneTimings {
        ping: Duration::from_millis(200),
        idle_close: idle,
        stall_close: Duration::from_secs(3),
    }
}
async fn connect(phone: &TestPhone, token: &str) -> Ws {
    let socket = tokio::net::TcpSocket::new_v4().unwrap();
    socket.set_recv_buffer_size(64 * 1024).unwrap();
    let stream = socket
        .connect(phone.server.local_addr().unwrap())
        .await
        .unwrap();
    tokio_tungstenite::client_async(
        phone.ws_url(&format!("/api/phone/v1/ws?pair={token}")),
        MaybeTlsStream::Plain(stream),
    )
    .await
    .unwrap()
    .0
}
async fn demand(ws: &mut Ws) {
    ws.send(Message::Text(
        json!({"type":"demand","gen":1,"live":true,"visible":true})
            .to_string()
            .into(),
    ))
    .await
    .unwrap();
    loop {
        if let Some(Ok(Message::Text(text))) = ws.next().await {
            let v: Value = serde_json::from_str(text.as_str()).unwrap();
            if v["type"] == "demand_ack" {
                break;
            }
        }
    }
}
fn publish(a: &TestApp, n: u32) -> u64 {
    let f = CameraFrame {
        source: SOURCE,
        frame_no: n,
        width: 640,
        height: 424,
        jpeg: vec![(n % 251) as u8; 1 << 20],
        link: Link::Cable,
    };
    match a.live_hub().publish(f) {
        PublishOutcome::Forwarded { seq } => seq,
        other => panic!("{other:?}"),
    }
}
async fn flood(a: &TestApp) {
    for n in 1..=40 {
        publish(a, n);
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}
async fn gone(a: &TestApp, before: usize, within: Duration) -> bool {
    let end = tokio::time::Instant::now() + within;
    while tokio::time::Instant::now() < end {
        if a.live_hub().viewer_count() == before {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}
#[test]
fn lv5_c9a_a_slow_reader_skips_frames_and_recovers_without_disconnecting() {
    let a = TestApp::with_camera().unwrap();
    let phone = TestPhone::start_with(&a, timings(Duration::from_secs(5)));
    let token = phone.pair(&a);
    rt().block_on(async {
        let mut ws = connect(&phone, &token).await;
        demand(&mut ws).await;
        a.live_hub().source_started(Link::Cable, SOURCE);
        flood(&a).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert_eq!(a.live_hub().viewer_count(), 1);
        let mut seqs = Vec::new();
        let end = tokio::time::Instant::now() + Duration::from_secs(2);
        while tokio::time::Instant::now() < end {
            match tokio::time::timeout(Duration::from_millis(300), ws.next()).await {
                Ok(Some(Ok(Message::Binary(b)))) => {
                    seqs.push(u64::from_le_bytes(b[8..16].try_into().unwrap()));
                    if *seqs.last().unwrap() == 40 {
                        break;
                    }
                }
                Ok(Some(Ok(Message::Close(c)))) => panic!("closed during stall: {c:?}"),
                _ => {}
            }
        }
        assert!(!seqs.is_empty() && seqs.len() <= 2, "{seqs:?}");
        assert_eq!(*seqs.last().unwrap(), 40);
        let next = publish(&a, 41);
        let end = tokio::time::Instant::now() + Duration::from_secs(1);
        let mut received = false;
        while tokio::time::Instant::now() < end {
            if let Ok(Some(Ok(Message::Binary(b)))) =
                tokio::time::timeout(Duration::from_millis(200), ws.next()).await
                && u64::from_le_bytes(b[8..16].try_into().unwrap()) == next
            {
                received = true;
                break;
            }
        }
        assert!(received);
        assert_eq!(a.live_hub().viewer_count(), 1);
    });
}
#[test]
fn lv5_c9b_a_reader_that_never_resumes_is_unsubscribed_after_stall() {
    let a = TestApp::with_camera().unwrap();
    let phone = TestPhone::start_with(&a, timings(Duration::from_secs(10)));
    let token = phone.pair(&a);
    rt().block_on(async {
        let before = a.live_hub().viewer_count();
        let mut ws = connect(&phone, &token).await;
        demand(&mut ws).await;
        a.live_hub().source_started(Link::Cable, SOURCE);
        flood(&a).await;
        assert!(gone(&a, before, Duration::from_secs(4)).await);
    });
}
#[test]
fn lv5_c9c_revoke_interrupts_a_stuck_write_within_one_second() {
    let a = TestApp::with_camera().unwrap();
    let phone = TestPhone::start_with(&a, timings(Duration::from_secs(10)));
    let token = phone.pair(&a);
    rt().block_on(async {
        let before = a.live_hub().viewer_count();
        let mut ws = connect(&phone, &token).await;
        demand(&mut ws).await;
        a.live_hub().source_started(Link::Cable, SOURCE);
        flood(&a).await;
        let pairings = PhonePairings::new(a.store(), a.clock.as_ref());
        let id = pairings.verify(&token).unwrap().unwrap().id;
        pairings.revoke(id).unwrap();
        phone.server.close_pairing(id);
        assert!(gone(&a, before, Duration::from_secs(1)).await);
    });
}
#[test]
fn lv5_c9d_production_timings_survive_a_long_wifi_stall() {
    let t = PhoneTimings::default();
    assert!(t.stall_close >= Duration::from_secs(10));
    assert!(t.ping < t.idle_close);
}
