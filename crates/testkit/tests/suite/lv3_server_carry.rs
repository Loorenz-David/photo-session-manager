//! LV3·U0 — the server carry-overs from LV2's review (`LV3_phone_app.md` Unit 0): rows
//! LV3·C26–C29, at the phone server's HTTP routes and WebSocket (`TestPhone`, the real server on
//! 127.0.0.1) and at the desktop's session delete and settings save.
use futures_util::StreamExt;
use psm_core::{domain::photo::Classification as C, events::AppEvent};
use psm_integrations::{AppHandles, session_delete};
use psm_testkit::{TestApp, phone::TestPhone};
use serde_json::{Value, json};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

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
/// The next text message of `kind` within `within` (others are dropped), else `None`.
async fn message(ws: &mut Ws, kind: &str, within: Duration) -> Option<Value> {
    let end = tokio::time::Instant::now() + within;
    loop {
        match tokio::time::timeout_at(end, ws.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let value: Value = serde_json::from_str(text.as_str()).unwrap();
                if value["type"] == kind {
                    return Some(value);
                }
            }
            Ok(Some(Ok(_))) => continue,
            _ => return None,
        }
    }
}
struct Resp {
    status: u16,
    body: Vec<u8>,
}
impl Resp {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body)
            .unwrap_or_else(|_| panic!("not JSON: {}", String::from_utf8_lossy(&self.body)))
    }
}
async fn send(method: &str, url: &str, bearer: &str, body: Option<Value>) -> Resp {
    let client = reqwest::Client::new();
    let mut request = match method {
        "POST" => client.post(url),
        _ => client.get(url),
    }
    .bearer_auth(bearer);
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.unwrap();
    let status = response.status().as_u16();
    let body = response.bytes().await.unwrap().to_vec();
    Resp { status, body }
}
fn arrive(a: &TestApp, s: i64, n: u32) -> i64 {
    let path = a.arrival_photo(s, &format!("DSC{n:05}.ARW")).unwrap();
    a.store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .find(|p| p.raw_path == path)
        .unwrap()
        .id
}
/// An active (untemplated) session.
fn session(a: &TestApp, sku: &str) -> i64 {
    let s = a
        .items()
        .create_local(sku, "Oak Lounge Chair", "Chair", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    s
}
fn place(a: &TestApp, id: i64) -> (C, Option<i64>) {
    let p = a.store().photo(id).unwrap();
    (p.classification, p.position)
}
/// An active session with one Main photo and one Condition photo (returned).
fn with_condition_photo(a: &TestApp, sku: &str) -> (i64, i64) {
    let s = session(a, sku);
    arrive(a, s, 100);
    let q = arrive(a, s, 101);
    a.review().classify(q, C::Condition).unwrap();
    a.clock.advance(301_000);
    (s, q)
}

// ---- LV3·C26: a phone Delete announces the change and is one undo step ----

#[test]
fn lv3_c26a_phone_delete_emits_the_change_events_and_reaches_a_second_phone() {
    let a = TestApp::with_camera().unwrap();
    let (s, q) = with_condition_photo(&a, "LV3-C26A");
    let phone = TestPhone::start(&a);
    let (first, second) = (phone.pair(&a), phone.pair(&a));
    runtime().block_on(async {
        let mut other = connect(&phone, &second).await;
        message(&mut other, "hello", Duration::from_secs(2))
            .await
            .unwrap();
        a.events.0.lock().unwrap().clear();
        let r = send(
            "POST",
            &phone.url(&format!("/api/phone/v1/photos/{q}/delete")),
            &first,
            Some(json!({"session_id": s})),
        )
        .await;
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
        let events = a.events.0.lock().unwrap().clone();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, AppEvent::SessionChanged { session_id } if *session_id == s)),
            "{events:?}"
        );
        assert!(
            events.iter().any(|e| matches!(
                e,
                AppEvent::PhotoChanged { photo_id, session_id } if *photo_id == q && *session_id == s
            )),
            "{events:?}"
        );
        let told = message(&mut other, "session", Duration::from_secs(2))
            .await
            .expect("the second phone is told the session changed");
        assert_eq!(told["session_id"].as_i64(), Some(s));
    });
}

#[test]
fn lv3_c26b_phone_delete_is_one_undo_step_that_restores_the_photo() {
    let a = TestApp::with_camera().unwrap();
    let (s, q) = with_condition_photo(&a, "LV3-C26B");
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    let before = place(&a, q);
    let undo_before = a.history().state(s).undo;
    assert_eq!(before.0, C::Condition);
    runtime().block_on(async {
        let r = send(
            "POST",
            &phone.url(&format!("/api/phone/v1/photos/{q}/delete")),
            &token,
            Some(json!({"session_id": s})),
        )
        .await;
        assert_eq!(r.status, 200);
    });
    assert_eq!(place(&a, q).0, C::Deleted);
    a.history().undo(s).unwrap();
    assert_eq!(place(&a, q), before);
    assert!(a.store().photo_denial(q).unwrap().is_none());
    // One step only: what can be undone now is what could be undone before the delete.
    assert_eq!(a.history().state(s).undo, undo_before);
}

// ---- LV3·C27: deleting the active session on the Mac is announced ----

#[test]
fn lv3_c27_deleting_the_active_session_tells_the_phone_there_is_none() {
    let a = TestApp::with_camera().unwrap();
    let s = session(&a, "LV3-C27");
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    runtime().block_on(async {
        let mut ws = connect(&phone, &token).await;
        message(&mut ws, "hello", Duration::from_secs(2))
            .await
            .unwrap();
        session_delete::delete(&a.app.handles(), s).unwrap();
        let end = Instant::now() + Duration::from_secs(2);
        let mut told = None;
        while Instant::now() < end && told.is_none() {
            told = message(&mut ws, "session", Duration::from_millis(200))
                .await
                .filter(|m| m["session_id"].is_null());
        }
        let told = told.expect("a session message with session_id null");
        assert_eq!(told["type"], "session");
        assert!(told["session_id"].is_null(), "{told}");
        let r = send("GET", &phone.url("/api/phone/v1/session"), &token, None).await;
        assert_eq!(r.json(), json!({"session": null}));
    });
}

// ---- LV3·C28: a settings save that rebuilds the App keeps open sockets fed ----

#[test]
fn lv3_c28_an_open_socket_hears_of_a_photo_after_the_app_was_rebuilt() {
    let mut a = TestApp::with_camera().unwrap();
    let s = session(&a, "LV3-C28");
    let current: Arc<Mutex<AppHandles>> = Arc::new(Mutex::new(a.app.handles()));
    let phone = TestPhone::start_following(&a, current.clone());
    let token = phone.pair(&a);
    let rt = runtime();
    let mut ws = rt.block_on(async {
        let mut ws = connect(&phone, &token).await;
        message(&mut ws, "hello", Duration::from_secs(2))
            .await
            .unwrap();
        ws
    });
    // A setting whose save reopens the runtime (the App is rebuilt).
    let events_before = Arc::as_ptr(&a.app.events);
    a.settings_set(&json!({"style_name": "PSM House"})).unwrap();
    assert_ne!(
        Arc::as_ptr(&a.app.events),
        events_before,
        "the save rebuilt the App"
    );
    *current.lock().unwrap() = a.app.handles();
    rt.block_on(async {
        // Whatever the rebuild itself announced is read first.
        while message(&mut ws, "session", Duration::from_millis(300))
            .await
            .is_some()
        {}
        arrive(&a, s, 100);
        let told = message(&mut ws, "session", Duration::from_secs(2))
            .await
            .expect("a session message on the socket opened before the rebuild");
        assert_eq!(told["session_id"].as_i64(), Some(s));
    });
}

// ---- LV3·C29: error bodies carry no internal text ----

#[test]
fn lv3_c29a_an_internal_error_answers_a_fixed_generic_message() {
    let a = TestApp::with_camera().unwrap();
    session(&a, "LV3-C29A");
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    // The store fails under the route: the pairings table is gone.
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .execute(
            "ALTER TABLE phone_pairings RENAME TO phone_pairings_gone",
            [],
        )
        .unwrap();
    let r = runtime()
        .block_on(async { send("GET", &phone.url("/api/phone/v1/session"), &token, None).await });
    assert_eq!(r.status, 500, "{}", String::from_utf8_lossy(&r.body));
    assert_eq!(
        r.json(),
        json!({"error": {"code": "internal", "message": "Something went wrong in Session Manager."}})
    );
}

#[test]
fn lv3_c29b_a_missing_preview_file_is_404_without_its_path() {
    let a = TestApp::with_camera().unwrap();
    let s = session(&a, "LV3-C29B");
    let p = arrive(&a, s, 100);
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    let missing = phone.dir.path().join("previews-gone").join("p.jpg");
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .execute(
            "UPDATE photos SET adjusted_preview_path=?1 WHERE id=?2",
            rusqlite::params![missing.to_string_lossy(), p],
        )
        .unwrap();
    let r = runtime().block_on(async {
        send(
            "GET",
            &phone.url(&format!("/api/phone/v1/photos/{p}/preview")),
            &token,
            None,
        )
        .await
    });
    assert_eq!(r.status, 404);
    assert_eq!(
        r.json(),
        json!({"error": {"code": "not_found", "message": "Not found."}})
    );
    let body = String::from_utf8_lossy(&r.body).into_owned();
    assert!(!body.contains("previews-gone"), "{body}");
    assert!(
        !body.contains(&*phone.dir.path().to_string_lossy()),
        "{body}"
    );
}
