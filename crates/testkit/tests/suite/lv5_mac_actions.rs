use futures_util::StreamExt;
use psm_core::{
    domain::{composition::Shape, photo::Classification as C, template::Template},
    events::AppEvent,
    ports::clock::Clock,
    services::{
        phone_view::phone_view,
        template_placement::{self, HandAction},
        templates::TemplatesService,
    },
};
use psm_infra::imaging::ImageOpsImpl;
use psm_integrations::photo_files;
use psm_testkit::{TestApp, phone::TestPhone};
use serde_json::Value;
use std::time::Duration;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}
async fn session_message(ws: &mut Ws) -> Value {
    let end = tokio::time::Instant::now() + Duration::from_secs(1);
    loop {
        match tokio::time::timeout_at(end, ws.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let v: Value = serde_json::from_str(text.as_str()).unwrap();
                if v["type"] == "session" {
                    return v;
                }
            }
            Ok(Some(Ok(_))) => {}
            other => panic!("no session message: {other:?}"),
        }
    }
}
fn announced(a: &TestApp, s: i64, action: impl FnOnce()) {
    let phone = TestPhone::start(a);
    let token = phone.pair(a);
    runtime().block_on(async {
        let (mut ws, _) = connect_async(phone.ws_url(&format!("/api/phone/v1/ws?pair={token}")))
            .await
            .unwrap();
        assert!(matches!(ws.next().await, Some(Ok(Message::Text(_)))));
        a.events.0.lock().unwrap().clear();
        action();
        let m = session_message(&mut ws).await;
        assert_eq!(m["session_id"], s);
        assert!(m["rev"].as_i64().unwrap() > 0);
        assert!(
            a.events
                .0
                .lock()
                .unwrap()
                .iter()
                .any(|e| matches!(e,AppEvent::SessionChanged{session_id} if *session_id==s))
        );
    });
}
fn t3() -> (TestApp, i64, [i64; 3], [i64; 5], i64) {
    let a = TestApp::with_camera().unwrap();
    let templates = || {
        TemplatesService::new(
            a.store(),
            a.clock.as_ref(),
            &ImageOpsImpl,
            a.app.paths.root.join("templates"),
        )
    };
    let t = templates().create("Chair").unwrap();
    let t = (0..3).fold(t, |t: Template, _| {
        templates().container_add(t.id, None).unwrap()
    });
    let containers = [t.containers[0].id, t.containers[1].id, t.containers[2].id];
    templates()
        .shooting_order_set(t.id, &[containers[1], containers[0], containers[2]])
        .unwrap();
    let s = a
        .items()
        .create_local("LV5-U2", "LV5-U2", "Chair", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    let mut photos = [0; 5];
    for (i, p) in photos.iter_mut().enumerate() {
        let path = a
            .arrival_photo(s, &format!("DSC{:05}.ARW", 101 + i))
            .unwrap();
        *p = a
            .store()
            .photos_in_session(s)
            .unwrap()
            .into_iter()
            .find(|p| p.raw_path == path)
            .unwrap()
            .id;
    }
    a.review().delete(photos[4]).unwrap();
    let second = a
        .items()
        .create_local("LV5-U2-B", "LV5-U2-B", "Table", None)
        .unwrap()
        .1
        .session
        .id;
    (a, s, containers, photos, second)
}
fn hand(a: &TestApp) -> HandAction<'_> {
    HandAction {
        store: a.store(),
        clock: a.clock.as_ref(),
        events: a.app.events.as_ref(),
        history: a.app.history.as_ref(),
        shots: a.app.camera_runtime.current_shots(),
    }
}

#[test]
fn lv5_c5a_mac_classify_announces() {
    let (a, s, _, p, _) = t3();
    announced(&a, s, || {
        a.review().classify(p[1], C::Condition).unwrap();
    });
}
#[test]
fn lv5_c5b_mac_restore_announces() {
    let (a, s, _, p, _) = t3();
    announced(&a, s, || {
        a.review().classify(p[4], C::Main).unwrap();
    });
}
#[test]
fn lv5_c5c_mac_reorder_announces() {
    let a = TestApp::new().unwrap();
    let s = a
        .items()
        .create_local("LV5-U2-C", "LV5-U2-C", "Table", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    let p1 = a.add_photo(s, "c5c1").unwrap().id;
    let p2 = a.add_photo(s, "c5c2").unwrap().id;
    a.review().classify(p1, C::Main).unwrap();
    a.review().classify(p2, C::Main).unwrap();
    announced(&a, s, || {
        a.review().reorder(p2, 1).unwrap();
    });
}
#[test]
fn lv5_c5d_mac_deny_announces_photo_and_session() {
    let (a, s, _, p, _) = t3();
    announced(&a, s, || {
        a.review().delete(p[1]).unwrap();
    });
    assert!(a.events.0.lock().unwrap().iter().any(|e|matches!(e,AppEvent::PhotoChanged{photo_id,session_id} if *photo_id==p[1] && *session_id==s)));
}
#[test]
fn lv5_c5e_keep_published_announces() {
    let (a, s, _, _, _) = t3();
    a.mark_published(s).unwrap();
    a.arrival_photo(s, "DSC00120.ARW").unwrap();
    announced(&a, s, || {
        a.review().keep_published(s).unwrap();
    });
}
#[test]
fn lv5_c5f_mac_move_announces_source() {
    let (a, s, _, p, target) = t3();
    announced(&a, s, || {
        photo_files::move_photos(&a.app.handles(), &[p[3]], target).unwrap();
    });
}
#[test]
fn lv5_c5g_mac_copy_announces_target() {
    let (a, s, _, _, second) = t3();
    let from = a.add_photo(second, "c5g").unwrap().id;
    announced(&a, s, || {
        a.review().copy_photos(&[from], s).unwrap();
    });
}
#[test]
fn lv5_c6a_undo_classify_announces() {
    let (a, s, _, p, _) = t3();
    a.review().classify(p[1], C::Condition).unwrap();
    announced(&a, s, || {
        a.history().undo(s).unwrap();
    });
}
#[test]
fn lv5_c6b_redo_classify_announces() {
    let (a, s, _, p, _) = t3();
    a.review().classify(p[1], C::Condition).unwrap();
    a.history().undo(s).unwrap();
    announced(&a, s, || {
        a.history().redo(s).unwrap();
    });
}
#[test]
fn lv5_c6c_undo_reorder_announces() {
    let a = TestApp::new().unwrap();
    let s = a
        .items()
        .create_local("LV5-U2-R", "LV5-U2-R", "Table", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    let p1 = a.add_photo(s, "c6c1").unwrap().id;
    let p2 = a.add_photo(s, "c6c2").unwrap().id;
    a.review().classify(p1, C::Main).unwrap();
    a.review().classify(p2, C::Main).unwrap();
    a.review().reorder(p2, 1).unwrap();
    announced(&a, s, || {
        a.history().undo(s).unwrap();
    });
}
#[test]
fn lv5_c6d_undo_deny_announces_and_restores_photo() {
    let (a, s, _, p, _) = t3();
    a.review().delete(p[1]).unwrap();
    announced(&a, s, || {
        a.history().undo(s).unwrap();
    });
    assert_eq!(a.store().photo(p[1]).unwrap().classification, C::Main);
}
#[test]
fn lv5_c6e_undo_move_announces_source() {
    let (a, s, _, p, target) = t3();
    photo_files::move_photos(&a.app.handles(), &[p[3]], target).unwrap();
    announced(&a, s, || {
        a.history().undo(s).unwrap();
    });
}
#[test]
fn lv5_c7a_drop_on_filled_container_announces() {
    let (a, s, c, p, _) = t3();
    announced(&a, s, || {
        template_placement::drop_on_container(&hand(&a), p[3], c[0]).unwrap();
    });
}
#[test]
fn lv5_c7b_back_to_template_announces() {
    let (a, s, _, p, _) = t3();
    a.review().classify(p[1], C::Condition).unwrap();
    announced(&a, s, || {
        template_placement::back_to_template(&hand(&a), s).unwrap();
    });
}
#[test]
fn lv5_c7c_shift_back_announces() {
    let (a, s, _, p, _) = t3();
    a.review().delete(p[1]).unwrap();
    announced(&a, s, || {
        template_placement::shift_back(&hand(&a), s).unwrap();
    });
}

fn preview(a: &TestApp, s: i64, p: i64) -> Option<String> {
    let view = phone_view(
        a.store(),
        a.app.camera_runtime.current_shots().as_ref(),
        Shape::Square,
        Some(s),
        a.clock.now_ms(),
    )
    .unwrap()
    .session
    .unwrap();
    view.shots
        .into_iter()
        .filter_map(|shot| shot.photo)
        .chain(view.extras)
        .chain(view.condition)
        .find(|image| image.photo_id == p)
        .unwrap()
        .preview
}
#[test]
fn lv5_c8a_preview_token_appears_only_when_the_file_exists() {
    let (a, s, _, p, _) = t3();
    let path = a.dir.path().join("preview.jpg");
    let db = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    db.execute(
        "UPDATE photos SET adjusted_preview_path=? WHERE id=?",
        rusqlite::params![path.to_string_lossy().as_ref(), p[1]],
    )
    .unwrap();
    assert_eq!(preview(&a, s, p[1]), None);
    std::fs::write(&path, b"one").unwrap();
    let token = preview(&a, s, p[1]).unwrap();
    assert!(!token.is_empty() && !token.contains('/') && !token.contains('\\'));
}
#[test]
fn lv5_c8b_preview_token_changes_when_the_file_is_rerendered() {
    let (a, s, _, p, _) = t3();
    let path = a.dir.path().join("preview.jpg");
    let db = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    db.execute(
        "UPDATE photos SET adjusted_preview_path=? WHERE id=?",
        rusqlite::params![path.to_string_lossy().as_ref(), p[1]],
    )
    .unwrap();
    std::fs::write(&path, b"one").unwrap();
    let before = preview(&a, s, p[1]);
    std::thread::sleep(Duration::from_millis(5));
    std::fs::write(&path, b"two").unwrap();
    assert_ne!(preview(&a, s, p[1]), before);
}
#[test]
fn lv5_c8c_preview_changed_event_reaches_the_phone_socket() {
    let a = TestApp::with_fake_document().unwrap();
    let s = a
        .items()
        .create_local("LV5-C8C", "LV5-C8C", "Table", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    let p = a.add_photo(s, "c8c.ARW").unwrap();
    a.store().upsert_raw_state(&p.raw_path, None).unwrap();
    a.fake.add_image(&p.raw_path, &["v1"], false);
    a.review().classify(p.id, C::Main).unwrap();
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    runtime().block_on(async {
        let (mut ws, _) = connect_async(phone.ws_url(&format!("/api/phone/v1/ws?pair={token}")))
            .await
            .unwrap();
        ws.next().await.unwrap().unwrap();
        a.app.previews.request_initial(&p.raw_path).unwrap();
        a.complete_render("job-1").unwrap();
        let m = session_message(&mut ws).await;
        assert_eq!(m["session_id"], s);
    });
    assert!(preview(&a, s, p.id).is_some());
}
