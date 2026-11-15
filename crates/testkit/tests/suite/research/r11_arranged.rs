//! R11 U4: arranged is a session fact, and preview starts research without holding publishing.

use std::{
    fs,
    sync::{Arc, Mutex, mpsc},
};

use psm_core::{
    domain::photo::{Classification as C, NewPhoto, PhotoSource},
    events::AppEvent,
    mechanisms::hash::sha256_file,
    ports::{clock::Clock, store::Store},
};
use psm_infra::sqlite::store::SqliteStore;
use psm_integrations::arranged::{arrange_and_next, preview_for_publish};
use psm_research::{
    domain::{RunState, Stage},
    store::{self as rstore, RunRow},
};
use psm_testkit::{TestApp, fixtures::synthetic_jpeg::SyntheticJpeg};

fn app() -> TestApp {
    let a = TestApp::with_research().unwrap();
    a.clock.advance(1_762_900_000_000);
    a
}

fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, "Oak chair", "Chair", None)
        .unwrap()
        .1
        .session
        .id
}

fn photo(a: &TestApp, s: i64, name: &str, class: C, preview: bool) -> i64 {
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(format!("{name}.ARW"));
    fs::write(&raw, name).unwrap();
    let p = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id: s,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    a.fake
        .add_image(&p.raw_path, &[&format!("v{}", p.id)], false);
    if class == C::Deleted {
        a.review().delete(p.id).unwrap();
    } else if class != C::Unreviewed {
        a.review().classify(p.id, class).unwrap();
    }
    if preview {
        a.store().adjusted_event(&p.raw_path, false).unwrap();
        let generation = a.store().adjusted_gen(&p.raw_path).unwrap();
        let path = a
            .dir
            .path()
            .join(format!("adjusted-{}-{generation}.jpg", p.id));
        fs::write(&path, SyntheticJpeg::rgb(1600, 1200, [180, 80, 40])).unwrap();
        a.store()
            .complete_preview(&p.raw_path, generation, path.to_str().unwrap(), None, None)
            .unwrap();
    }
    p.id
}

fn ready(a: &TestApp, sku: &str) -> i64 {
    let s = session(a, sku);
    photo(a, s, sku, C::Main, true);
    s
}

fn runs(a: &TestApp, s: Option<i64>) -> Vec<RunRow> {
    a.app
        .store
        .research_transaction(|tx| rstore::runs(tx.conn(), s))
        .unwrap()
}

#[test]
fn a25_c17a_arrange_marks_queues_and_emits() {
    let a = app();
    let s = ready(&a, "A17A");
    let at = a.clock.now_ms();
    let result = arrange_and_next(&a.app.handles(), s).unwrap();
    assert_eq!(a.sessions().get(s).unwrap().session.arranged_at, Some(at));
    assert!(result.research_queued);
    let rows = runs(&a, Some(s));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].state, RunState::Queued);
    assert_eq!(rows[0].trigger.as_str(), "arranged");
    assert!(
        a.events
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, AppEvent::SessionChanged { session_id } if *session_id == s))
    );
}

#[test]
fn a25_c17b_second_arrange_keeps_mark_and_run() {
    let a = app();
    let s = ready(&a, "A17B");
    arrange_and_next(&a.app.handles(), s).unwrap();
    let first = a.sessions().get(s).unwrap().session.arranged_at;
    a.tick(1_100).unwrap();
    a.clock.advance(3_600_000);
    assert!(
        !arrange_and_next(&a.app.handles(), s)
            .unwrap()
            .research_queued
    );
    assert_eq!(a.sessions().get(s).unwrap().session.arranged_at, first);
    assert_eq!(runs(&a, Some(s)).len(), 1);
}

#[test]
fn a25_c17c_unassigned_refused() {
    let a = app();
    let id = a.sessions().unassigned().unwrap().session.id;
    assert!(a.sessions().arrange(id).is_err());
    assert_eq!(a.sessions().get(id).unwrap().session.arranged_at, None);
}

#[test]
fn a25_c17d_old_database_gains_and_keeps_arranged_mark() {
    let a = app();
    let x = session(&a, "MIG-X");
    let y = session(&a, "MIG-Y");
    let paths = a.app.paths.clone();
    let TestApp { dir: _dir, app, .. } = a;
    drop(app);
    // Simulate the previous schema while keeping both session records.
    let conn = rusqlite::Connection::open(paths.db()).unwrap();
    conn.execute("ALTER TABLE sessions DROP COLUMN arranged_at", [])
        .unwrap();
    drop(conn);
    let store = SqliteStore::open(&paths).unwrap();
    assert_eq!(store.session(x).unwrap().session.arranged_at, None);
    assert_eq!(store.session(y).unwrap().session.arranged_at, None);
    assert!(psm_core::ports::store::Store::arrange_session_once(&store, x, 12345).unwrap());
    drop(store);
    let store = SqliteStore::open(&paths).unwrap();
    assert_eq!(store.session(x).unwrap().session.arranged_at, Some(12345));
    assert_eq!(store.session(y).unwrap().session.arranged_at, None);
}

#[test]
fn a25_c18a_next_follows_creation_order() {
    let a = app();
    let ids: Vec<_> = ["A", "B", "C", "D"].iter().map(|s| ready(&a, s)).collect();
    assert_eq!(
        arrange_and_next(&a.app.handles(), ids[1])
            .unwrap()
            .next_session_id,
        Some(ids[2])
    );
}

#[test]
fn a25_c18b_next_wraps() {
    let a = app();
    let ids: Vec<_> = ["A", "B", "C", "D"].iter().map(|s| ready(&a, s)).collect();
    a.sessions().arrange(ids[1]).unwrap();
    a.sessions().arrange(ids[2]).unwrap();
    assert_eq!(
        arrange_and_next(&a.app.handles(), ids[3])
            .unwrap()
            .next_session_id,
        Some(ids[0])
    );
}

#[test]
fn a25_c18c_all_other_sessions_marked() {
    let a = app();
    let x = ready(&a, "X");
    let y = ready(&a, "Y");
    a.sessions().arrange(y).unwrap();
    assert_eq!(
        arrange_and_next(&a.app.handles(), x)
            .unwrap()
            .next_session_id,
        None
    );
}

#[test]
fn a25_c18d_ineligible_sessions_are_skipped() {
    let a = app();
    let current = ready(&a, "CUR");
    let published = ready(&a, "PUB");
    a.mark_published(published).unwrap();
    let archived = ready(&a, "ARC");
    let item = a.sessions().get(archived).unwrap().session.item_id.unwrap();
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .execute(
            "INSERT INTO item_retention(item_id,cleared_at) VALUES (?1,?2)",
            rusqlite::params![item, a.clock.now_ms()],
        )
        .unwrap();
    let deleted = session(&a, "DEL");
    photo(&a, deleted, "DEL", C::Deleted, false);
    // The Unassigned session (id 1) holds a photo that is not deleted, so only its own rule can skip it.
    photo(&a, 1, "UNA", C::Unreviewed, false);
    assert_eq!(
        arrange_and_next(&a.app.handles(), current)
            .unwrap()
            .next_session_id,
        None
    );
}

#[test]
fn a25_c19a_missing_integration_and_key_does_not_block() {
    let a = TestApp::with_research_fakes_only().unwrap();
    let x = ready(&a, "X");
    let y = ready(&a, "Y");
    let result = arrange_and_next(&a.app.handles(), x).unwrap();
    assert_eq!(result.next_session_id, Some(y));
    assert!(!result.research_queued);
    assert!(a.sessions().get(x).unwrap().session.arranged_at.is_some());
    assert!(runs(&a, Some(x)).is_empty());
    assert!(a.research_fakes().model.requests().is_empty());
    assert!(a.research_fakes().search.requests().is_empty());
    assert!(a.research_fakes().reader.requests().is_empty());
}

#[test]
fn a25_c20a_preview_marks_and_queues() {
    let a = app();
    let s = ready(&a, "P20A");
    let preview = preview_for_publish(&a.app.handles(), &[s]).unwrap();
    assert!(a.sessions().get(s).unwrap().session.arranged_at.is_some());
    assert_eq!(preview.researching, [true]);
    let rows = runs(&a, Some(s));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].state, RunState::Queued);
    assert_eq!(rows[0].trigger.as_str(), "arranged");
}

#[test]
fn a25_c20c_finished_preview_does_not_queue_again() {
    let a = app();
    let s = ready(&a, "P20C");
    preview_for_publish(&a.app.handles(), &[s]).unwrap();
    a.tick(1_100).unwrap();
    let preview = preview_for_publish(&a.app.handles(), &[s]).unwrap();
    assert_eq!(preview.researching, [false]);
    assert_eq!(runs(&a, Some(s)).len(), 1);
}

#[test]
fn a25_c20b_finished_research_shows_withheld_suggestion() {
    let a = app();
    let s = ready(&a, "P20B");
    let p = a.store().photos_in_session(s).unwrap().pop().unwrap();
    a.research_fakes().model.reply(
        Stage::Observe,
        &format!("photo:{}", p.id),
        serde_json::json!({"reads": {"custom.designer": {"value": "Tentative", "sure": false}}}),
    );
    preview_for_publish(&a.app.handles(), &[s]).unwrap();
    a.tick(1_100).unwrap();
    assert_eq!(runs(&a, Some(s)).last().unwrap().state, RunState::Finished);
    let preview = preview_for_publish(&a.app.handles(), &[s]).unwrap();
    assert_eq!(preview.researching, [false]);
    assert_eq!(preview.preview.items[0].n_withheld, 1);
    assert_eq!(runs(&a, Some(s)).len(), 1);
}

#[test]
fn a25_c20d_preview_without_key_keeps_core_preview() {
    let a = app();
    a.secrets
        .values
        .lock()
        .unwrap()
        .remove(psm_research::service::MODEL_ACCOUNT);
    let s = ready(&a, "P20D");
    let preview = preview_for_publish(&a.app.handles(), &[s]).unwrap();
    let core = a.app.publish.as_ref().unwrap().preview(&[s]).unwrap();
    assert!(a.sessions().get(s).unwrap().session.arranged_at.is_some());
    assert!(runs(&a, Some(s)).is_empty());
    assert_eq!(preview.researching, [false]);
    assert_eq!(
        serde_json::to_value(&preview.preview).unwrap(),
        serde_json::to_value(core).unwrap()
    );
}

#[test]
fn a25_c20e_confirm_while_research_runs_freezes_only_committed_values() {
    let a = app();
    let s = ready(&a, "P20E");
    let p = a.store().photos_in_session(s).unwrap().pop().unwrap();
    a.research_fakes().model.reply(
        Stage::Observe,
        &format!("photo:{}", p.id),
        serde_json::json!({"reads": {"custom.designer": {"value": "Run value", "sure": true}}}),
    );
    let preview = preview_for_publish(&a.app.handles(), &[s]).unwrap();
    assert_eq!(preview.researching, [true]);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));
    // Holds only the run's first model call: the run's later calls (it goes on to write) pass through.
    let mut first = true;
    a.research_fakes().model.before_call(move |_| {
        if std::mem::take(&mut first) {
            entered_tx.send(()).unwrap();
            release_rx.lock().unwrap().recv().unwrap();
        }
    });
    a.clock.advance(1_100);
    let runner = a.app.research_runner.clone().unwrap();
    let now = a.clock.now_ms();
    let thread = std::thread::spawn(move || runner.tick(now).unwrap());
    entered_rx.recv().unwrap();
    let shown = preview
        .preview
        .items
        .iter()
        .map(|item| psm_core::domain::publish::ShownSuggestions {
            session_id: item.session_id,
            n_withheld: item.n_withheld,
            conflict_keys: item.conflicts.iter().map(|c| c.key.clone()).collect(),
        })
        .collect::<Vec<_>>();
    let id = a
        .app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], &preview.preview.summary, &shown)
        .unwrap();
    let item = a.store().publish_item(id).unwrap();
    assert!(!item.frozen_attributes.contains_key("custom.designer"));
    a.research_fakes().model.clear_before_call();
    release_tx.send(()).unwrap();
    thread.join().unwrap();
    // The run went on to write the value after the confirmation; the frozen publish item still does not hold it.
    let item_id = a.sessions().get(s).unwrap().session.item_id.unwrap();
    let mut written = Vec::new();
    a.store()
        .transaction(&mut |tx| {
            written = tx.attribute_values(item_id)?;
            Ok(())
        })
        .unwrap();
    assert!(
        written
            .iter()
            .any(|v| v.key == "custom.designer" && v.value == "\"Run value\"")
    );
    let item = a.store().publish_item(id).unwrap();
    assert!(!item.frozen_attributes.contains_key("custom.designer"));
}

#[test]
fn a25_c22a_reset_restores_defaults_and_keeps_trusted_sites() {
    let mut a = app();
    let s = ready(&a, "SET");
    let mut old = psm_testkit::research_test_settings();
    old["stages"] = serde_json::json!({});
    old["prices"] = serde_json::json!([]);
    old["trusted_sites"] = serde_json::json!(["https://one.example", "https://two.example"]);
    a.set_research_settings(old).unwrap();
    let reset = a.app.research.as_ref().unwrap().settings_reset().unwrap();
    a.settings_set(&serde_json::json!({"research": reset}))
        .unwrap();
    let saved = psm_infra::settings_file::Settings::open(&a.app.paths)
        .unwrap()
        .research
        .unwrap();
    assert!(!saved["stages"].as_object().unwrap().is_empty());
    assert!(!saved["prices"].as_array().unwrap().is_empty());
    assert_eq!(
        saved["trusted_sites"],
        serde_json::json!(["https://one.example", "https://two.example"])
    );
    let status = a.app.research.as_ref().unwrap().status(s).unwrap();
    assert_ne!(status.reason.map(|r| r.code), Some("price_missing".into()));
}

#[test]
fn a25_c21a_classify_and_wait_never_queues() {
    let a = app();
    let s = session(&a, "C21A");
    photo(&a, s, "C21A", C::Main, true);
    a.tick(0).unwrap();
    a.tick(600_000).unwrap();
    assert!(runs(&a, Some(s)).is_empty());
}
