//! R7·U1 (rows A13·C1(c), C2(a), C8(c), C8(d) and the rows carried from the R4 and R6 reviews):
//! the research engine wired into the app (08B MC-A6, MC-A10; book 01 HA4). The boundary is the
//! wired `TestApp` (the real `App` start-up path), photo changes through the app's own services,
//! `TestApp::tick` (one turn of the research runner) and `kill_and_restart`.

use std::{
    fs,
    sync::{Arc, Barrier, Mutex, mpsc},
};

use psm_core::{
    domain::{
        attribute::AiConfidence,
        photo::{Classification as C, NewPhoto, Photo, PhotoSource},
        session::PublishStep,
    },
    events::{AppEvent, ExportState},
    mechanisms::hash::sha256_file,
    ports::clock::Clock,
};
use psm_research::{
    domain::RunState,
    store::{self as rstore, RunRow},
};
use psm_testkit::{RESEARCH_INTEGRATION, TestApp, fixtures::synthetic_jpeg::SyntheticJpeg};
use serde_json::json;

const NOW_MS: i64 = 1_762_900_000_000;

fn research_app() -> TestApp {
    let a = TestApp::with_research().unwrap();
    a.clock.advance(NOW_MS);
    a
}

/// A new item with its session: `(session, item)`.
fn session(a: &TestApp, sku: &str) -> (i64, i64) {
    let (item, session) = a
        .items()
        .create_local(sku, "Oak chair", "Chair", None)
        .unwrap();
    (session.session.id, item.id)
}

/// A RAW on disk known to the fake Capture One document, classified through the review service.
fn photo(a: &TestApp, session_id: i64, name: &str, class: C) -> Photo {
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(format!("{name}.ARW"));
    fs::write(&raw, format!("raw bytes of {name}")).unwrap();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    match class {
        C::Unreviewed => {}
        C::Deleted => {
            a.review().delete(photo.id).unwrap();
        }
        _ => {
            a.review().classify(photo.id, class).unwrap();
        }
    }
    a.store().photo(photo.id).unwrap()
}

/// Gives the photo an adjusted preview (its research source, MC-A5).
fn give_preview(a: &TestApp, photo: &Photo, colour: [u8; 3]) {
    a.store().adjusted_event(&photo.raw_path, false).unwrap();
    let generation = a.store().adjusted_gen(&photo.raw_path).unwrap();
    let path = a
        .dir
        .path()
        .join(format!("adjusted-{}-g{generation}.jpg", photo.id));
    fs::write(&path, SyntheticJpeg::rgb(1600, 1200, colour)).unwrap();
    a.store()
        .complete_preview(
            &photo.raw_path,
            generation,
            path.to_str().unwrap(),
            None,
            None,
        )
        .unwrap();
}

/// A session with one Main photo that has its research source: `(session, item)`.
fn ready_session(a: &TestApp, sku: &str, colour: [u8; 3]) -> (i64, i64) {
    let (s, item) = session(a, sku);
    let p = photo(a, s, sku, C::Main);
    give_preview(a, &p, colour);
    a.app.research.as_ref().unwrap().arranged(s).unwrap();
    (s, item)
}

fn runs(a: &TestApp, session_id: Option<i64>) -> Vec<RunRow> {
    a.app
        .store
        .research_transaction(|tx| rstore::runs(tx.conn(), session_id))
        .unwrap()
}

fn running(a: &TestApp) -> usize {
    a.app
        .store
        .research_transaction(|tx| rstore::runs_in_state(tx.conn(), RunState::Running))
        .unwrap()
        .len()
}

fn count(a: &TestApp, sql: &str) -> i64 {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(sql, [], |r| r.get(0))
        .unwrap()
}

/// What a killed process leaves: unwinds out of the provider call without the panic hook.
struct Killed;

#[test]
fn a13_c1c_core_workflow_is_untouched_without_research() {
    let a = TestApp::with_research_fakes_only().unwrap();
    a.clock.advance(NOW_MS);
    let (s, _) = session(&a, "K");
    photo(&a, s, "a", C::Main);
    photo(&a, s, "b", C::Main);
    photo(&a, s, "d", C::Condition);
    photo(&a, s, "f", C::Deleted);
    a.tick(1_100).unwrap();
    a.app.export.run(s).unwrap();
    a.complete_all_renders().unwrap();
    a.tick(0).unwrap();
    a.tick(130_000).unwrap();
    let mut names: Vec<String> = fs::read_dir(a.export_root().join("K"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "K_01_condition.jpg",
            "K_01_main.jpg",
            "K_02_main.jpg",
            "item.json",
            "manifest.json"
        ]
    );
    let last_export = a
        .events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::ExportProgress {
                done, total, state, ..
            } => Some((*done, *total, *state)),
            _ => None,
        })
        .next_back();
    assert_eq!(last_export, Some((3, 3, ExportState::Done)));
    assert_eq!(count(&a, "SELECT COUNT(*) FROM research_runs"), 0);
    assert_eq!(count(&a, "SELECT COUNT(*) FROM research_calls"), 0);
    let fakes = a.research_fakes();
    assert!(fakes.model.requests().is_empty());
    assert!(fakes.search.requests().is_empty());
    assert!(fakes.reader.requests().is_empty());
}

#[test]
fn a13_c8c_a_running_run_is_resumed_after_a_restart() {
    let mut a = research_app();
    let (s, _) = ready_session(&a, "R7-C8C", [200, 40, 40]);
    a.research_fakes()
        .model
        .before_call(|_| std::panic::resume_unwind(Box::new(Killed)));
    let killed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| a.tick(1_100)));
    assert!(killed.expect_err("the run was not killed").is::<Killed>());
    a.research_fakes().model.clear_before_call();
    let run = runs(&a, Some(s)).pop().unwrap();
    assert_eq!(run.state, RunState::Running);
    a.kill_and_restart().unwrap();
    let now = a.clock.now_ms();
    assert_eq!(
        count(
            &a,
            &format!(
                "SELECT COUNT(*) FROM jobs WHERE kind='research' AND status='queued' \
                 AND next_attempt_at<={now}"
            )
        ),
        1
    );
    a.tick(0).unwrap();
    let run = runs(&a, Some(s)).pop().unwrap();
    assert_eq!(run.state, RunState::Finished);
}

#[test]
fn a13_c8d_research_has_its_own_runner() {
    let a = research_app();
    let (research_session, _) = ready_session(&a, "R7-C8D-R", [40, 200, 40]);
    a.clock.advance(1_100);
    let (entered_tx, entered_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let release_rx = Mutex::new(release_rx);
    a.research_fakes().model.before_call(move |_| {
        let _ = entered_tx.send(());
        let _ = release_rx.lock().unwrap().recv();
    });
    let runner = a.app.research_runner.clone().unwrap();
    let now = a.clock.now_ms();
    let research = std::thread::spawn(move || runner.tick(now).unwrap());
    entered_rx.recv().unwrap();
    // The research job is inside a provider call. A publish becomes due meanwhile.
    let (s, _) = session(&a, "R7-C8D-P");
    photo(&a, s, "publish", C::Main);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    for _ in 0..15 {
        a.tick(1_100).unwrap();
        a.complete_all_renders().unwrap();
    }
    let item = a
        .store()
        .publish_item(a.store().latest_publish(s).unwrap().unwrap().item_id)
        .unwrap();
    assert_eq!(item.step, PublishStep::Done);
    assert_eq!(
        runs(&a, Some(research_session)).pop().unwrap().state,
        RunState::Running
    );
    a.research_fakes().model.clear_before_call();
    release_tx.send(()).unwrap();
    assert!(research.join().unwrap());
}

/// Carried (R4 review, MA13): `ResearchService::tick` is called from one place; two wake-ups at
/// the same time never leave two runs `running`.
#[test]
fn r7_u1_two_concurrent_wake_ups_never_run_two() {
    let a = research_app();
    let (s1, _) = ready_session(&a, "R7-MA13-A", [10, 10, 200]);
    let (s2, _) = ready_session(&a, "R7-MA13-B", [10, 200, 200]);
    assert_eq!(runs(&a, None).len(), 2);
    a.clock.advance(1_100);
    let seen: Arc<Mutex<Vec<usize>>> = Arc::default();
    let (store, out) = (a.app.store.clone(), seen.clone());
    a.research_fakes().model.before_call(move |_| {
        let n = store
            .research_transaction(|tx| rstore::runs_in_state(tx.conn(), RunState::Running))
            .unwrap()
            .len();
        out.lock().unwrap().push(n);
    });
    let barrier = Arc::new(Barrier::new(2));
    let now = a.clock.now_ms();
    let wakes: Vec<_> = (0..2)
        .map(|_| {
            let runner = a.app.research_runner.clone().unwrap();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                runner.tick(now).unwrap()
            })
        })
        .collect();
    for wake in wakes {
        wake.join().unwrap();
    }
    assert!(running(&a) <= 1);
    a.tick(0).unwrap();
    a.tick(0).unwrap();
    let seen = seen.lock().unwrap();
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|n| *n == 1), "{seen:?}");
    for s in [s1, s2] {
        assert_eq!(runs(&a, Some(s)).pop().unwrap().state, RunState::Finished);
    }
}

/// Carried (R6 review, task 4): a `research` patch is validated by the engine, saved, and reads
/// back; an invalid one is refused with the validation message and nothing is saved.
#[test]
fn r7_u1_research_settings_are_validated_and_saved() {
    let mut a = research_app();
    let saved = a
        .settings_set(&json!({"research": {"max_cost_per_run": "0.75"}}))
        .unwrap();
    assert_eq!(saved["research"]["max_cost_per_run"], json!("0.75"));
    assert_eq!(saved["research"]["max_cost_per_day"], json!("15"));
    let error = a
        .settings_set(&json!({"research": {"max_cost_per_run": "0,5"}}))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains(
            "max_cost_per_run must be an amount in USD like 0.50, with at most 6 decimals"
        ),
        "{error}"
    );
    let stored = psm_infra::settings_file::Settings::open(&a.app.paths)
        .unwrap()
        .research
        .unwrap();
    assert_eq!(stored["max_cost_per_run"], json!("0.75"));
}

/// Carried (R6 review): one release signal for every path. A redo that resolves the last
/// conflict of a held publish item emits `session_changed` and `publish_progress`.
#[test]
fn r7_u1_a_redo_that_releases_emits_the_event() {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(NOW_MS);
    a.research_fields(&["k"]).unwrap();
    let (s, item) = session(&a, "R7-REDO");
    photo(&a, s, "redo", C::Main);
    a.research_conflict(item, "k", "Oak", "Walnut", AiConfidence::High)
        .unwrap();
    a.attributes(Some(RESEARCH_INTEGRATION))
        .clear(item, "k")
        .unwrap();
    a.history().undo(s).unwrap();
    a.confirm_as_shown(&[s], "Publish").unwrap();
    let held = a.store().latest_publish(s).unwrap().unwrap().item_id;
    assert_eq!(
        a.store().publish_item(held).unwrap().step,
        PublishStep::Held
    );
    a.events.0.lock().unwrap().clear();
    a.history().redo(s).unwrap();
    assert_eq!(
        a.store().publish_item(held).unwrap().step,
        PublishStep::Prepare
    );
    let events = a.events.0.lock().unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AppEvent::SessionChanged { session_id } if *session_id == s)),
        "{events:?}"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            AppEvent::PublishProgress { session_id, item_id, step: PublishStep::Prepare, .. }
                if *session_id == s && *item_id == held
        )),
        "{events:?}"
    );
}

/// A9 (owner correction 2026-11-18): the camera's passwords are not research secrets. A page
/// title that happens to hold the generated FTP code (a short code of digits) is recorded as it
/// is and the export goes through; before, the code was redacted from the record and stopped the
/// export as "contains a key".
#[test]
fn a9_a_camera_password_in_a_record_neither_redacts_nor_stops_the_export() {
    use psm_core::{domain::camera::secrets as camera, ports::secrets::SecretStore};
    let a = research_app();
    a.secrets.set(camera::FTP_PASSWORD, "4821").unwrap();
    a.secrets
        .set(camera::ACCESS_AUTH_PASSWORD, "camera-access-pw")
        .unwrap();
    a.research_fakes()
        .search
        .set_results(vec![psm_research::ports::SearchCandidate {
            url: "https://dealer.example/item/4821".into(),
            rank: 1,
            title: "Teak sideboard, model 4821".into(),
        }]);
    let (s, item) = ready_session(&a, "A9-CAM", [40, 120, 200]);
    a.tick(1_100).unwrap();
    assert_eq!(runs(&a, Some(s)).pop().unwrap().state, RunState::Finished);
    assert_eq!(
        count(
            &a,
            "SELECT COUNT(*) FROM research_calls WHERE redaction_hit = 1"
        ),
        0
    );
    let dest = a.dir.path().join("exports");
    fs::create_dir_all(&dest).unwrap();
    let zip = a
        .app
        .research
        .as_ref()
        .unwrap()
        .export(psm_research::api::ExportSelection::Items(vec![item]), &dest)
        .expect("the export goes through");
    assert!(zip.exists());
}
