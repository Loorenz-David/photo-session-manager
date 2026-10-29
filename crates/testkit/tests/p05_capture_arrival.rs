use psm_core::{
    domain::{item::NewItem, photo::PhotoSource},
    mechanisms::{arrival::path_key, canon::canon},
    services::photos::PhotosService,
};
use psm_integrations::App;
use psm_testkit::{TestApp, fixtures::synthetic_raw::SyntheticRaw};
use rusqlite::Connection;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn app() -> TestApp {
    TestApp::with_fake_document().unwrap()
}
fn root(a: &TestApp) -> PathBuf {
    a.dir.path().join("Capture One Session").join("Capture")
}
fn file(a: &TestApp, relative: &str, bytes: &[u8]) -> PathBuf {
    let path = root(a).join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    path
}
fn external(a: &TestApp, name: &str, bytes: &[u8]) -> PathBuf {
    let p = a.dir.path().join(name);
    fs::write(&p, bytes).unwrap();
    p
}
fn db(a: &TestApp) -> Connection {
    Connection::open(a.app.paths.db()).unwrap()
}
fn count(a: &TestApp, table: &str) -> i64 {
    db(a)
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, sku, "object", None)
        .unwrap()
        .1
        .session
        .id
}
fn watch(a: &TestApp, path: &Path) {
    a.watcher_event(path).unwrap();
    a.tick(1000).unwrap();
}
fn hook(a: &TestApp, path: &Path) {
    a.deliver_hook(
        &serde_json::json!({"hook":"capture_done","argv":[path.to_string_lossy()]}).to_string(),
    )
    .unwrap();
}
fn row(a: &TestApp) -> (i64, String, Option<i64>, String) {
    db(a)
        .query_row(
            "SELECT session_id,classification,position,source FROM photos ORDER BY id LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap()
}
fn preview(a: &TestApp, path: &Path) -> (Option<String>, Option<i64>) {
    db(a)
        .query_row(
            "SELECT original_preview_path,orientation FROM photos WHERE raw_path=?1",
            [canon(path).unwrap()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
}
fn service(a: &TestApp) -> PhotosService<'_> {
    PhotosService {
        arrival: a.app.arrival.as_ref(),
        editing: a.app.editing.as_ref(),
        store: a.store(),
        clock: a.clock.as_ref(),
    }
}

#[test]
fn c1a_hidden_file_ignored() {
    let a = app();
    let p = file(&a, "K1/.hidden.ARW", b"raw");
    watch(&a, &p);
    assert_eq!((count(&a, "photos"), count(&a, "capture_events")), (0, 0));
}
#[test]
fn c1b_capture_one_folder_ignored() {
    let a = app();
    let p = file(&a, "K1/CaptureOne/x.ARW", b"raw");
    watch(&a, &p);
    assert_eq!((count(&a, "photos"), count(&a, "capture_events")), (0, 0));
}
#[test]
fn c1c_dot_capture_folder_ignored() {
    let a = app();
    let p = file(&a, ".capture/x.ARW", b"raw");
    watch(&a, &p);
    assert_eq!((count(&a, "photos"), count(&a, "capture_events")), (0, 0));
}
#[test]
fn c1d_jpeg_ignored() {
    let a = app();
    let p = file(&a, "K1/x.jpg", b"jpeg");
    watch(&a, &p);
    assert_eq!((count(&a, "photos"), count(&a, "capture_events")), (0, 0));
}
#[test]
fn c1e_two_levels_ignored() {
    let a = app();
    let p = file(&a, "K1/sub/x.ARW", b"raw");
    watch(&a, &p);
    assert_eq!((count(&a, "photos"), count(&a, "capture_events")), (0, 0));
}
#[test]
fn c1f_lowercase_extension_accepted() {
    let a = app();
    let p = file(&a, "K1/x.arw", b"raw");
    watch(&a, &p);
    assert_eq!(count(&a, "photos"), 1);
}
#[test]
fn c1g_directory_ignored() {
    let a = app();
    let p = root(&a).join("K1/x.ARW");
    fs::create_dir_all(&p).unwrap();
    watch(&a, &p);
    assert_eq!((count(&a, "photos"), count(&a, "capture_events")), (0, 0));
}

#[test]
fn c2a_growth_requires_next_stable_stat() {
    let a = app();
    let p = file(&a, "x.ARW", b"one");
    a.watcher_event(&p).unwrap();
    fs::write(&p, b"longer").unwrap();
    a.tick(1000).unwrap();
    assert_eq!(count(&a, "photos"), 0);
    a.tick(1000).unwrap();
    assert_eq!(count(&a, "photos"), 1);
}
#[test]
fn c2b_growth_times_out() {
    let a = app();
    let p = file(&a, "x.ARW", b"x");
    a.watcher_event(&p).unwrap();
    for n in 1..=121 {
        fs::write(&p, vec![b'x'; n + 1]).unwrap();
        a.tick(1000).unwrap();
    }
    let c = db(&a);
    let errors: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM capture_events WHERE error IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        (errors, count(&a, "capture_events"), count(&a, "photos")),
        (1, 1, 0)
    );
}

#[test]
fn c3a_unassigned_folder_wins() {
    let a = app();
    let s = session(&a, "K1");
    a.store().set_active_session_id(Some(s)).unwrap();
    let p = file(&a, "_Unassigned/x.ARW", b"raw");
    hook(&a, &p);
    assert_eq!(row(&a).0, 1);
}
#[test]
fn c3b_published_folder_becomes_dirty() {
    let a = app();
    let s = session(&a, "K1");
    let s2 = session(&a, "K2");
    a.mark_published(s).unwrap();
    a.store().set_active_session_id(Some(s2)).unwrap();
    let p = file(&a, "K1/x.ARW", b"raw");
    hook(&a, &p);
    assert_eq!(row(&a).0, s);
    assert!(a.store().session(s).unwrap().dirty);
}
#[test]
fn c3c_folder_wins_over_active() {
    let a = app();
    let s = session(&a, "K1");
    let s2 = session(&a, "K2");
    a.store().set_active_session_id(Some(s2)).unwrap();
    let p = file(&a, "K1/x.ARW", b"raw");
    hook(&a, &p);
    assert_eq!(row(&a).0, s);
}
#[test]
fn c3d_unknown_folder_uses_active() {
    let a = app();
    let s = session(&a, "K2");
    a.store().set_active_session_id(Some(s)).unwrap();
    let p = file(&a, "Other/x.ARW", b"raw");
    hook(&a, &p);
    assert_eq!(row(&a).0, s);
}
#[test]
fn c3e_unknown_folder_uses_unassigned() {
    let a = app();
    let p = file(&a, "Other/x.ARW", b"raw");
    hook(&a, &p);
    assert_eq!(row(&a).0, 1);
}
#[test]
fn c3f_root_uses_active() {
    let a = app();
    let s = session(&a, "K2");
    a.store().set_active_session_id(Some(s)).unwrap();
    let p = file(&a, "x.ARW", b"raw");
    hook(&a, &p);
    assert_eq!(row(&a).0, s);
}

#[test]
fn c4a_hook_watcher_scan_one_row() {
    let a = app();
    let p = file(&a, "x.ARW", b"raw");
    hook(&a, &p);
    watch(&a, &p);
    a.scan().unwrap();
    assert_eq!(count(&a, "photos"), 1);
}
#[test]
fn c4b_watcher_then_hook_one_row() {
    let a = app();
    let p = file(&a, "x.ARW", b"raw");
    watch(&a, &p);
    hook(&a, &p);
    assert_eq!(count(&a, "photos"), 1);
}
#[test]
fn c4c_concurrent_delivery_one_row() {
    let a = app();
    let p = file(&a, "x.ARW", b"raw");
    let arrival = a.app.arrival.clone();
    let spool = a.app.spool.clone();
    let store = a.app.store.clone();
    let clock = a.clock.clone();
    std::thread::scope(|scope| {
        let p1 = p.clone();
        scope.spawn(move || arrival.watcher_event(&p1).unwrap());
        let arrival2 = a.app.arrival.clone();
        scope.spawn(move || arrival2.scan().unwrap());
        let json =
            serde_json::json!({"hook":"capture_done","argv":[p.to_string_lossy()]}).to_string();
        scope.spawn(move || {
            spool.write_for_test("concurrent.json", &json).unwrap();
            spool.drain(store.as_ref(), clock.as_ref()).unwrap();
        });
    });
    a.tick(1000).unwrap();
    assert_eq!(count(&a, "photos"), 1);
}
#[test]
fn c4d_every_delivery_recorded() {
    let a = app();
    let p = file(&a, "x.ARW", b"raw");
    hook(&a, &p);
    watch(&a, &p);
    a.scan().unwrap();
    let c = db(&a);
    let mut s = c
        .prepare("SELECT source FROM capture_events ORDER BY source")
        .unwrap();
    let sources: Vec<String> = s
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(sources, ["hook", "scan", "watcher"]);
}

#[test]
fn c5a_row_defaults_uid_and_raw_state() {
    let a = app();
    let p = file(&a, "x.ARW", b"raw");
    hook(&a, &p);
    let r = row(&a);
    assert_eq!(
        (r.1.as_str(), r.2, r.3.as_str()),
        ("unreviewed", None, "capture")
    );
    let c = db(&a);
    let (uid, key): (String, String) = c
        .query_row(
            "SELECT p.uid,r.path_key FROM photos p JOIN raw_state r ON r.raw_path=p.raw_path",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        (
            uid.len(),
            uid.chars().nth(14),
            uid.chars().nth(19).unwrap().to_digit(16).unwrap() & 0b1100
        ),
        (36, Some('4'), 0b1000)
    );
    assert_eq!(key, path_key(&canon(&p).unwrap()));
    assert!(matches!(
        a.events.0.lock().unwrap().as_slice(),
        [psm_core::events::AppEvent::PhotoAdded {
            photo_id: 1,
            session_id: 1
        }]
    ));
}
#[test]
fn c5b_file_mtime_not_exif() {
    let a = app();
    let (bytes, _) = SyntheticRaw::build(&[(1088, 1080, true)], 1, Some("2026:10:25 22:00:00"));
    let p = file(&a, "x.ARW", &bytes);
    fs::File::open(&p)
        .unwrap()
        .set_times(
            fs::FileTimes::new()
                .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1792958400)),
        )
        .unwrap();
    let m = fs::metadata(&p)
        .unwrap()
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    hook(&a, &p);
    let actual: i64 = db(&a)
        .query_row("SELECT captured_at FROM photos", [], |r| r.get(0))
        .unwrap();
    assert_eq!(actual, m);
}
#[test]
fn c5c_four_followup_jobs() {
    let a = app();
    let p = file(&a, "x.ARW", b"raw");
    hook(&a, &p);
    let c = db(&a);
    let mut s = c
        .prepare("SELECT kind,target,status FROM jobs ORDER BY kind")
        .unwrap();
    let jobs: Vec<(String, String, String)> = s
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let raw = canon(&p).unwrap();
    assert_eq!(
        jobs,
        vec![
            ("crop_on_arrival".into(), raw.clone(), "queued".into()),
            ("hash".into(), raw.clone(), "queued".into()),
            ("original_preview".into(), raw.clone(), "queued".into()),
            ("render_preview".into(), raw, "queued".into())
        ]
    );
}

#[test]
fn c6a_largest_ifd_stream_copied() {
    let a = app();
    let (bytes, jpegs) = SyntheticRaw::build(&[(1088, 1080, true), (160, 120, true)], 1, None);
    let p = file(&a, "x.ARW", &bytes);
    hook(&a, &p);
    a.tick(0).unwrap();
    let (dest, _) = preview(&a, &p);
    let dest = dest.unwrap();
    assert_eq!(fs::read(&dest).unwrap(), jpegs[0]);
    assert!(dest.ends_with(&format!(
        "/previews/{}/{}/original.jpg",
        &path_key(&canon(&p).unwrap())[..2],
        path_key(&canon(&p).unwrap())
    )));
}
#[test]
fn c6b_invalid_largest_uses_smaller() {
    let a = app();
    let (bytes, jpegs) = SyntheticRaw::build(&[(1088, 1080, false), (160, 120, true)], 1, None);
    let p = file(&a, "x.ARW", &bytes);
    hook(&a, &p);
    a.tick(0).unwrap();
    assert_eq!(fs::read(preview(&a, &p).0.unwrap()).unwrap(), jpegs[1]);
}
#[test]
fn c6c_orientation_stored() {
    let a = app();
    let (bytes, _) = SyntheticRaw::build(&[(100, 100, true)], 6, None);
    let p = file(&a, "x.ARW", &bytes);
    hook(&a, &p);
    a.tick(0).unwrap();
    assert_eq!(preview(&a, &p).1, Some(6));
}
#[test]
fn c6d_raw_unchanged() {
    let a = app();
    let (bytes, _) = SyntheticRaw::build(&[(100, 100, true)], 1, None);
    let p = file(&a, "x.ARW", &bytes);
    let before = fs::metadata(&p).unwrap().modified().unwrap();
    hook(&a, &p);
    a.tick(0).unwrap();
    assert_eq!(fs::read(&p).unwrap(), bytes);
    assert_eq!(fs::metadata(&p).unwrap().modified().unwrap(), before);
}
#[test]
fn c6e_no_jpeg_finishes_without_preview() {
    let a = app();
    let (bytes, _) = SyntheticRaw::build(&[], 1, None);
    let p = file(&a, "x.ARW", &bytes);
    hook(&a, &p);
    a.tick(0).unwrap();
    assert_eq!(preview(&a, &p).0, None);
    let status: String = db(&a)
        .query_row(
            "SELECT status FROM jobs WHERE kind='original_preview'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "done");
}

fn reopen(a: &TestApp) -> App {
    App::open_with(a.app.paths.clone(), a.fake.clone(), a.clock.clone(), false).unwrap()
}
#[test]
fn c7a_startup_scans_root_and_children() {
    let a = app();
    let s = session(&a, "K1");
    a.store().set_active_session_id(Some(s)).unwrap();
    let p1 = file(&a, "a.ARW", b"a");
    let p2 = file(&a, "K1/b.ARW", b"b");
    let p3 = file(&a, "_Unassigned/c.ARW", b"c");
    let b = reopen(&a);
    let c = Connection::open(b.paths.db()).unwrap();
    let got: Vec<(String, i64)> = {
        let mut stmt = c
            .prepare("SELECT raw_path,session_id FROM photos ORDER BY raw_path")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert_eq!(
        got,
        vec![
            (canon(&p2).unwrap(), s),
            (canon(&p3).unwrap(), 1),
            (canon(&p1).unwrap(), s)
        ]
    );
}
#[test]
fn c7b_startup_ignores_cache_and_known_raw() {
    let a = app();
    let known = file(&a, "_Unassigned/known.ARW", b"known");
    hook(&a, &known);
    let mut item_id = 0;
    a.store()
        .transaction(&mut |tx| {
            item_id = tx.insert_item(NewItem {
                sku: "K1",
                folder_key: "K1",
                barcode: None,
                item_type: "object",
                title: "K1",
                origin: "local",
                at: 0,
            })?;
            Ok(())
        })
        .unwrap();
    service(&a).assign_unassigned(item_id).unwrap();
    let assigned_session = a.store().session_for_item(item_id).unwrap().unwrap();
    file(&a, "K1/CaptureOne/x.ARW", b"cache");
    let b = reopen(&a);
    let got: Vec<(String, i64)> = {
        let c = Connection::open(b.paths.db()).unwrap();
        let mut stmt = c
            .prepare("SELECT raw_path,session_id FROM photos ORDER BY raw_path")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert_eq!(got, vec![(canon(&known).unwrap(), assigned_session)]);
}

#[test]
fn c8a_import_active_and_provider_called() {
    let a = app();
    let s = session(&a, "K2");
    a.store().set_active_session_id(Some(s)).unwrap();
    let p = external(&a, "outside.ARW", b"raw");
    service(&a)
        .import(&[p.to_string_lossy().into_owned()])
        .unwrap();
    assert_eq!(row(&a), (s, "unreviewed".into(), None, "import".into()));
    assert!(
        a.fake
            .state
            .lock()
            .unwrap()
            .ops
            .contains(&"import:1".into())
    );
}
#[test]
fn c8b_import_without_active_unassigned() {
    let a = app();
    let p = external(&a, "outside.ARW", b"raw");
    service(&a)
        .import(&[p.to_string_lossy().into_owned()])
        .unwrap();
    assert_eq!((row(&a).0, row(&a).3), (1, "import".into()));
}
#[test]
fn c8c_import_twice_one_row() {
    let a = app();
    let p = external(&a, "outside.ARW", b"raw");
    let paths = [p.to_string_lossy().into_owned()];
    service(&a).import(&paths).unwrap();
    service(&a).import(&paths).unwrap();
    assert_eq!(count(&a, "photos"), 1);
}
#[test]
fn c8d_assign_whole_unassigned_set() {
    let a = app();
    let p1 = file(&a, "_Unassigned/u1.ARW", b"one");
    let p2 = file(&a, "_Unassigned/u2.ARW", b"two");
    hook(&a, &p1);
    hook(&a, &p2);
    let mut item_id = 0;
    a.store()
        .transaction(&mut |tx| {
            item_id = tx.insert_item(NewItem {
                sku: "K1",
                folder_key: "K1",
                barcode: None,
                item_type: "object",
                title: "K1",
                origin: "local",
                at: 0,
            })?;
            Ok(())
        })
        .unwrap();
    service(&a).assign_unassigned(item_id).unwrap();
    let target = a.store().session_for_item(item_id).unwrap().unwrap();
    assert_eq!(a.store().session(1).unwrap().photo_count, 0);
    let moved = a.store().photos_in_session(target).unwrap();
    assert_eq!(
        moved.iter().map(|p| p.raw_path.clone()).collect::<Vec<_>>(),
        vec![canon(&p1).unwrap(), canon(&p2).unwrap()]
    );
    assert!(moved.iter().all(|p| p.source == PhotoSource::Capture));
}
