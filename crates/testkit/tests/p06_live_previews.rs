use psm_core::{
    domain::photo::{Classification as C, Photo},
    events::AppEvent,
    ports::clock::Clock,
    services::previews::PreviewService,
};
use psm_infra::previews_store::PreviewStore;
use psm_testkit::TestApp;
use std::{
    fs,
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn app() -> TestApp {
    let a = TestApp::with_fake_document().unwrap();
    a.clock.advance(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    );
    a
}
fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, sku, "object", None)
        .unwrap()
        .1
        .session
        .id
}
fn photo(a: &TestApp, s: i64, name: &str, class: C) -> Photo {
    fs::create_dir_all(a.dir.path().join(name).parent().unwrap()).unwrap();
    let p = a.add_photo(s, name).unwrap();
    a.store().upsert_raw_state(&p.raw_path, None).unwrap();
    let variant = format!("v{}", p.id);
    a.fake.add_image(&p.raw_path, &[&variant], false);
    if class != C::Unreviewed {
        a.review().classify(p.id, class).unwrap();
    }
    a.store().photo(p.id).unwrap()
}
fn adjusted(a: &TestApp, raw: &str) {
    a.deliver_hook(
        &serde_json::json!({"hook":"variant_adjusted","variant_id":"v1","raw":raw}).to_string(),
    )
    .unwrap();
}
fn process_ops(a: &TestApp) -> Vec<String> {
    a.fake
        .state
        .lock()
        .unwrap()
        .ops
        .iter()
        .filter(|op| op.starts_with("process:"))
        .cloned()
        .collect()
}
fn state(a: &TestApp, raw: &str) -> psm_core::ports::store::RawPreviewState {
    a.store().preview_state(raw).unwrap().unwrap()
}
fn start(a: &TestApp, raw: &str) {
    a.app.previews.request_initial(raw).unwrap();
}
fn finish(a: &TestApp, job: &str) {
    a.complete_render(job).unwrap();
}
fn out(a: &TestApp, raw: &str) -> String {
    state(a, raw).adjusted_preview_path.unwrap()
}
fn mtime(path: &str) -> SystemTime {
    fs::metadata(path).unwrap().modified().unwrap()
}

#[test]
fn c1a_completion_records_bytes_for_every_copy() {
    let a = app();
    let s1 = session(&a, "S1");
    let s2 = session(&a, "S2");
    let p1 = photo(&a, s1, "K/X.ARW", C::Unreviewed);
    let p2 = photo(&a, s2, "K/X.ARW", C::Unreviewed);
    start(&a, &p1.raw_path);
    let output = a.complete_render("job-1").unwrap();
    let path = out(&a, &p1.raw_path);
    assert!(path.ends_with("/adjusted-1.jpg"));
    assert_eq!(fs::read(&path).unwrap(), fs::read(output).unwrap());
    assert_eq!(
        a.store().photo(p2.id).unwrap().adjusted_preview_path,
        Some(path)
    );
    assert_eq!(state(&a, &p1.raw_path).rendered_gen, 1);
    let changed: Vec<_> = a
        .events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::PhotoChanged { photo_id, .. } => Some(*photo_id),
            _ => None,
        })
        .collect();
    assert_eq!(changed, vec![p1.id, p2.id]);
}

#[test]
fn rendered_raw_gets_preview_on_later_session_row() {
    let a = app();
    let s1 = session(&a, "S1");
    let s2 = session(&a, "S2");
    let first = photo(&a, s1, "K/X.ARW", C::Unreviewed);
    start(&a, &first.raw_path);
    finish(&a, "job-1");
    let path = out(&a, &first.raw_path);
    a.events.0.lock().unwrap().clear();

    let later = photo(&a, s2, "K/X.ARW", C::Unreviewed);
    assert_eq!(
        a.store().photo(later.id).unwrap().adjusted_preview_path,
        None
    );
    start(&a, &later.raw_path);

    assert_eq!(
        a.store()
            .photo(later.id)
            .unwrap()
            .adjusted_preview_path
            .as_deref(),
        Some(path.as_str())
    );
    assert_eq!(process_ops(&a).len(), 1);
    let changed: Vec<_> = a
        .events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|event| match event {
            AppEvent::PhotoChanged {
                photo_id,
                session_id,
            } => Some((*photo_id, *session_id)),
            _ => None,
        })
        .collect();
    assert_eq!(changed, vec![(later.id, s2)]);
}

#[test]
fn c1b_old_output_rejected() {
    let a = app();
    let s = session(&a, "S1");
    let p = photo(&a, s, "X.ARW", C::Unreviewed);
    start(&a, &p.raw_path);
    let path = a.dir.path().join("old.jpg");
    fs::write(&path, b"old").unwrap();
    let issued = a
        .app
        .editing
        .render_completion("job-1")
        .unwrap()
        .issued_at_ms;
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(
            fs::FileTimes::new()
                .set_modified(UNIX_EPOCH + Duration::from_millis((issued - 5_000) as u64)),
        )
        .unwrap();
    a.deliver_hook(&serde_json::json!({"hook":"processing_done","job":"job-1","raw":p.raw_path,"outputs":[path]}).to_string()).unwrap();
    assert_eq!(a.store().photo(p.id).unwrap().adjusted_preview_path, None);
    assert_eq!(state(&a, &p.raw_path).rendered_gen, 0);
}
#[test]
fn c1c_unknown_job_ignored() {
    let a = app();
    let s = session(&a, "S1");
    let p = photo(&a, s, "X.ARW", C::Unreviewed);
    let path = a.dir.path().join("out.jpg");
    fs::write(&path, b"out").unwrap();
    a.deliver_hook(&serde_json::json!({"hook":"processing_done","job":"unknown","raw":p.raw_path,"outputs":[path]}).to_string()).unwrap();
    assert_eq!(a.store().photo(p.id).unwrap().adjusted_preview_path, None);
    assert!(a.events.0.lock().unwrap().is_empty());
}
#[test]
fn c1d_new_generation_replaces_old_file() {
    let a = app();
    let s = session(&a, "S1");
    let p = photo(&a, s, "X.ARW", C::Unreviewed);
    start(&a, &p.raw_path);
    finish(&a, "job-1");
    let first = out(&a, &p.raw_path);
    adjusted(&a, &p.raw_path);
    a.tick(15_000).unwrap();
    finish(&a, "job-2");
    assert!(out(&a, &p.raw_path).ends_with("/adjusted-2.jpg"));
    assert!(!Path::new(&first).exists());
}
#[test]
fn c1e_timeout_reissues_once_then_fails() {
    let a = app();
    let s = session(&a, "S1");
    let p = photo(&a, s, "X.ARW", C::Unreviewed);
    start(&a, &p.raw_path);
    a.tick(60_000).unwrap();
    assert_eq!(process_ops(&a).len(), 2);
    a.tick(60_000).unwrap();
    let db = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let failed:(String,String)=db.query_row("SELECT status,error FROM jobs WHERE kind='render_preview' AND status='failed' ORDER BY id DESC LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(failed, ("failed".into(), "timeout".into()));
    assert_eq!(process_ops(&a).len(), 2);
    assert_eq!(a.store().photo(p.id).unwrap().adjusted_preview_path, None);
}
#[test]
fn c2a_delay_triggers_at_fifteen_seconds() {
    let a = app();
    let s = session(&a, "S1");
    let p = photo(&a, s, "X.ARW", C::Unreviewed);
    adjusted(&a, &p.raw_path);
    assert_eq!(state(&a, &p.raw_path).adjusted_gen, 1);
    a.tick(14_999).unwrap();
    assert!(process_ops(&a).is_empty());
    a.tick(1).unwrap();
    assert_eq!(process_ops(&a).len(), 1);
}
#[test]
fn c2b_second_event_restarts_delay() {
    let a = app();
    let s = session(&a, "S1");
    let p = photo(&a, s, "X.ARW", C::Unreviewed);
    adjusted(&a, &p.raw_path);
    a.tick(5_000).unwrap();
    adjusted(&a, &p.raw_path);
    a.tick(14_999).unwrap();
    assert!(process_ops(&a).is_empty());
    a.tick(1).unwrap();
    assert_eq!(process_ops(&a).len(), 1);
}
#[test]
fn c2c_event_during_render_stays_stale() {
    let a = app();
    let s = session(&a, "S1");
    let p = photo(&a, s, "X.ARW", C::Unreviewed);
    start(&a, &p.raw_path);
    adjusted(&a, &p.raw_path);
    a.tick(15_000).unwrap();
    assert_eq!(process_ops(&a).len(), 1);
    finish(&a, "job-1");
    let st = state(&a, &p.raw_path);
    assert_eq!((st.rendered_gen, st.adjusted_gen), (1, 2));
    assert_eq!(process_ops(&a).len(), 2);
}
#[test]
fn c2d_delay_settings_are_clamped_by_service() {
    let a = app();
    for (input, expected) in [
        (Some("5"), 10),
        (Some("40"), 30),
        (None, 15),
        (Some("abc"), 15),
    ] {
        let service = PreviewService::new(
            a.app.store.clone(),
            a.app.editing.clone(),
            a.app.events.clone(),
            a.clock.clone(),
            Arc::new(PreviewStore(a.app.paths.clone())),
            input,
        );
        assert_eq!(service.delay.as_secs(), expected);
    }
}
#[test]
fn c2e_unknown_raw_adjustment_ignored() {
    let a = app();
    let raw = a.dir.path().join("missing.ARW");
    fs::write(&raw, b"raw").unwrap();
    adjusted(&a, &raw.to_string_lossy());
    a.tick(30_000).unwrap();
    assert!(
        a.store()
            .preview_state(&raw.to_string_lossy())
            .unwrap()
            .is_none()
    );
    assert!(process_ops(&a).is_empty());
}
#[test]
fn c3a_focus_renders_stale_active_rows_first() {
    let a = app();
    let s = session(&a, "S1");
    let other = session(&a, "S2");
    let p2 = photo(&a, s, "P2.ARW", C::Main);
    let p1 = photo(&a, s, "P1.ARW", C::Main);
    a.review().reorder(p1.id, 1).unwrap();
    let p3 = photo(&a, other, "P3.ARW", C::Main);
    let fresh = photo(&a, other, "FRESH.ARW", C::Unreviewed);
    start(&a, &fresh.raw_path);
    finish(&a, "job-1");
    a.store().set_active_session_id(Some(s)).unwrap();
    for p in [&p2, &p1, &p3] {
        adjusted(&a, &p.raw_path);
    }
    let before = process_ops(&a).len();
    a.focus().unwrap();
    let ops = process_ops(&a);
    assert_eq!(
        &ops[before..],
        &[
            format!("process:v{}:PSM Preview", p1.id),
            format!("process:v{}:PSM Preview", p2.id),
            format!("process:v{}:PSM Preview", p3.id)
        ]
    );
}
#[test]
fn c4a_new_sidecar_creates_one_adjustment() {
    let a = app();
    let s = session(&a, "S1");
    let p = photo(&a, s, "X.ARW", C::Unreviewed);
    start(&a, &p.raw_path);
    finish(&a, "job-1");
    a.write_sidecar(
        &p.raw_path,
        mtime(&out(&a, &p.raw_path)) + Duration::from_secs(20),
    )
    .unwrap();
    a.focus().unwrap();
    a.focus().unwrap();
    assert_eq!(state(&a, &p.raw_path).adjusted_gen, 2);
}
#[test]
fn c4b_old_sidecar_does_not_adjust() {
    let a = app();
    let s = session(&a, "S1");
    let p = photo(&a, s, "X.ARW", C::Unreviewed);
    start(&a, &p.raw_path);
    finish(&a, "job-1");
    a.write_sidecar(
        &p.raw_path,
        mtime(&out(&a, &p.raw_path)) - Duration::from_secs(1),
    )
    .unwrap();
    a.focus().unwrap();
    assert_eq!(state(&a, &p.raw_path).adjusted_gen, 1);
}
#[test]
fn c4c_sidecar_within_own_write_window_ignored() {
    let a = app();
    let s = session(&a, "S1");
    let p = photo(&a, s, "X.ARW", C::Unreviewed);
    start(&a, &p.raw_path);
    finish(&a, "job-1");
    a.store()
        .mark_own_write(&p.raw_path, a.clock.now_ms())
        .unwrap();
    let own = UNIX_EPOCH + Duration::from_millis(a.clock.now_ms() as u64);
    a.write_sidecar(&p.raw_path, own + Duration::from_secs(10))
        .unwrap();
    a.focus().unwrap();
    assert_eq!(state(&a, &p.raw_path).adjusted_gen, 1);
}
#[test]
fn c5a_retouch_dirties_only_reviewed_session() {
    let a = app();
    let s1 = session(&a, "S1");
    let s2 = session(&a, "S2");
    let p = photo(&a, s1, "X.ARW", C::Main);
    photo(&a, s2, "X.ARW", C::Deleted);
    a.mark_published(s1).unwrap();
    a.mark_published(s2).unwrap();
    let before1 = a.store().session(s1).unwrap().session.change_seq;
    let before2 = a.store().session(s2).unwrap().session.change_seq;
    adjusted(&a, &p.raw_path);
    assert_eq!(state(&a, &p.raw_path).retouch_gen, 1);
    assert_eq!(
        a.store().session(s1).unwrap().session.change_seq,
        before1 + 1
    );
    assert!(a.store().session(s1).unwrap().dirty);
    assert_eq!(a.store().session(s2).unwrap().session.change_seq, before2);
    assert!(
        a.events
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e,AppEvent::SessionChanged{session_id} if *session_id==s1))
    );
}
#[test]
fn c5b_own_tag_echo_is_stale_but_not_retouch() {
    let a = app();
    let s = session(&a, "S1");
    let p = photo(&a, s, "X.ARW", C::Main);
    a.tick(0).unwrap();
    a.mark_published(s).unwrap();
    a.clock.advance(10_000);
    adjusted(&a, &p.raw_path);
    a.tick(15_000).unwrap();
    assert_eq!(state(&a, &p.raw_path).retouch_gen, 0);
    assert!(!a.store().session(s).unwrap().dirty);
    assert_eq!(process_ops(&a).len(), 1);
}
#[test]
fn c5c_retouch_dirties_both_copy_sessions() {
    let a = app();
    let s1 = session(&a, "S1");
    let s2 = session(&a, "S2");
    let p = photo(&a, s1, "X.ARW", C::Main);
    photo(&a, s2, "X.ARW", C::Main);
    a.mark_published(s1).unwrap();
    a.mark_published(s2).unwrap();
    adjusted(&a, &p.raw_path);
    assert!(a.store().session(s1).unwrap().dirty);
    assert!(a.store().session(s2).unwrap().dirty);
}
#[test]
fn c6a_same_base_name_waits_for_first_completion() {
    let a = app();
    let s = session(&a, "S1");
    let first = photo(&a, s, "K1/X.ARW", C::Unreviewed);
    let second = photo(&a, s, "K2/x.arw", C::Unreviewed);
    start(&a, &first.raw_path);
    start(&a, &second.raw_path);
    assert_eq!(process_ops(&a).len(), 1);
    finish(&a, "job-1");
    assert_eq!(process_ops(&a).len(), 2);
}
