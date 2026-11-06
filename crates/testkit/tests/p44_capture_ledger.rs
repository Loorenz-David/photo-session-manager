//! P44 — capture ledger (10E MC-33 R-I25…R-I29, 05A MC-2 rule 0, MC-4, 05B MC-22): behaviour rows
//! C3–C8 through `TestApp`. C1, C2 and C6(a) are the unit rows in `mechanisms::capture_ledger`.
use psm_core::{
    domain::camera::{
        Association, CaptureId, Delivery, DeliveryOutcome, DeliveryRow, DeliveryState,
        ExposureInitiator, ExposureObserved, ExposureStatus, Link,
    },
    mechanisms::{capture_ledger::Gap, naming::folder_key},
    ports::{camera::CaptureSink, clock::Clock},
};
use psm_testkit::TestApp;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant, UNIX_EPOCH},
};

fn app() -> TestApp {
    TestApp::with_camera().unwrap()
}
/// `(item id, session id)` of a local item whose SKU and title are `name`.
fn item(a: &TestApp, name: &str) -> (i64, i64) {
    let (item, session) = a.items().create_local(name, name, "object", None).unwrap();
    (item.id, session.session.id)
}
fn activate(a: &TestApp, session: i64) {
    a.sessions().activate(session).unwrap();
}
fn key_of(a: &TestApp, item: i64) -> String {
    a.store().item(item).unwrap().folder_key
}
/// Sets the clock to `t` (ms).
fn at(a: &TestApp, t: i64) {
    let now = a.clock.now_ms();
    assert!(t >= now, "the clock only moves forward");
    a.clock.advance(t - now);
}
fn expose(a: &TestApp, t: i64) -> i64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    at(a, t);
    a.ledger()
        .exposure_observed(ExposureObserved {
            capture_id: Some(CaptureId(format!(
                "capture-{}",
                NEXT.fetch_add(1, Ordering::Relaxed)
            ))),
            at: t,
            initiated: ExposureInitiator::App,
            provisional: false,
        })
        .unwrap()
}
fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn staging(a: &TestApp, name: &str, via: Link) -> PathBuf {
    let incoming = a.capture_root().join(".incoming");
    match via {
        Link::Wireless => incoming.join(format!(".{name}.part")),
        Link::Cable => incoming.join("sdk-x").join(name),
    }
}
/// Writes `bytes` where the receiver (Wi-Fi) or the Save-to-PC path (cable) stages them and hands
/// the delivery to the ledger, completed now.
fn deliver(a: &TestApp, name: &str, bytes: &[u8], via: Link) -> psm_core::Result<DeliveryOutcome> {
    let written = staging(a, name, via.clone());
    fs::create_dir_all(written.parent().unwrap()).unwrap();
    fs::write(&written, bytes).unwrap();
    a.ledger().delivery(Delivery {
        camera_filename: name.into(),
        written_path: written,
        sha256: sha(bytes),
        size: bytes.len() as u64,
        via,
        completed_at: a.clock.now_ms(),
    })
}
fn rows(a: &TestApp, name: &str) -> Vec<DeliveryRow> {
    a.ledger()
        .deliveries()
        .unwrap()
        .into_iter()
        .filter(|r| r.camera_filename == name)
        .collect()
}
fn row(a: &TestApp, name: &str) -> DeliveryRow {
    let mut all = rows(a, name);
    assert_eq!(all.len(), 1, "one row for {name}");
    all.remove(0)
}
fn exposure_status(a: &TestApp, id: i64) -> ExposureStatus {
    a.ledger()
        .exposures(None)
        .unwrap()
        .into_iter()
        .find(|e| e.id == id)
        .unwrap()
        .status
}
fn photos(a: &TestApp, session: i64) -> Vec<psm_core::domain::photo::Photo> {
    a.store().photos_in_session(session).unwrap()
}
/// The real arrival path (MC-2) for a file the ledger placed.
fn watch(a: &TestApp, path: impl AsRef<Path>) {
    a.watcher_event(path.as_ref()).unwrap();
    a.tick(1_000).unwrap();
}
fn health(a: &TestApp, code: &str) -> Option<String> {
    a.app
        .health
        .report("", false, false)
        .items
        .into_iter()
        .find(|i| i.code == code && !i.ok)
        .map(|i| i.detail)
}
fn files_in(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}
fn db(a: &TestApp) -> rusqlite::Connection {
    rusqlite::Connection::open(a.app.paths.db()).unwrap()
}
/// P42's `settle`: tick until `pred` holds (3 s bound).
fn settle(a: &TestApp, pred: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end {
        a.tick(0).unwrap();
        if pred() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("ledger result did not settle");
}

/// C3(a)'s fixture: session `s` active, one exposure, the same bytes by cable then by FTP.
fn cable_then_ftp(a: &TestApp) -> (i64, i64, Vec<u8>) {
    let (item_s, s) = item(a, "s");
    activate(a, s);
    expose(a, 0);
    let data = b"II*\0 DSC04622 raw".to_vec();
    at(a, 2_000);
    assert_eq!(
        deliver(a, "DSC04622.ARW", &data, Link::Cable).unwrap(),
        DeliveryOutcome::Photo
    );
    at(a, 4_000);
    assert_eq!(
        deliver(a, "DSC04622.ARW", &data, Link::Wireless).unwrap(),
        DeliveryOutcome::Duplicate
    );
    (item_s, s, data)
}

#[test]
fn c3a_cable_then_ftp_is_a_duplicate() {
    let a = app();
    let (item_s, s, data) = cable_then_ftp(&a);
    let root = a.capture_root();
    let all = rows(&a, "DSC04622.ARW");
    assert_eq!(all.len(), 2);
    let photo_path = root.join(key_of(&a, item_s)).join("DSC04622.ARW");
    let duplicate_path = root.join(".duplicates/DSC04622.ARW");
    assert_eq!(
        (all[0].via.clone(), all[0].outcome, all[0].association),
        (Link::Cable, DeliveryOutcome::Photo, Association::Fifo)
    );
    assert_eq!(all[0].final_path, photo_path.to_string_lossy());
    assert_eq!(
        (all[1].via.clone(), all[1].outcome, all[1].association),
        (
            Link::Wireless,
            DeliveryOutcome::Duplicate,
            Association::Filename
        )
    );
    assert_eq!(all[1].final_path, duplicate_path.to_string_lossy());
    assert_eq!(fs::read(&photo_path).unwrap(), data);
    assert_eq!(fs::read(&duplicate_path).unwrap(), data);
    watch(&a, &photo_path);
    watch(&a, &duplicate_path);
    assert_eq!(photos(&a, s).len(), 1);
}

#[test]
fn c3b_ftp_then_cable() {
    let a = app();
    let (item_s, s) = item(&a, "s");
    activate(&a, s);
    expose(&a, 0);
    let data = b"II*\0 DSC04623 raw".to_vec();
    at(&a, 2_000);
    deliver(&a, "DSC04623.ARW", &data, Link::Wireless).unwrap();
    at(&a, 4_000);
    deliver(&a, "DSC04623.ARW", &data, Link::Cable).unwrap();
    let all = rows(&a, "DSC04623.ARW");
    assert_eq!(
        all.iter()
            .map(|r| (r.via.clone(), r.outcome))
            .collect::<Vec<_>>(),
        vec![
            (Link::Wireless, DeliveryOutcome::Photo),
            (Link::Cable, DeliveryOutcome::Duplicate)
        ]
    );
    let root = a.capture_root();
    assert_eq!(
        all[0].final_path,
        root.join(key_of(&a, item_s))
            .join("DSC04623.ARW")
            .to_string_lossy()
    );
    assert_eq!(
        all[1].final_path,
        root.join(".duplicates/DSC04623.ARW").to_string_lossy()
    );
}

#[test]
fn c3c_duplicate_across_exposures_and_time() {
    let a = app();
    let (_, s) = item(&a, "s");
    activate(&a, s);
    expose(&a, 0);
    at(&a, 5_000);
    let d1 = b"II*\0 A".to_vec();
    deliver(&a, "A.ARW", &d1, Link::Wireless).unwrap();
    a.clock.advance(2 * 3_600_000);
    assert_eq!(
        deliver(&a, "A.ARW", &d1, Link::Wireless).unwrap(),
        DeliveryOutcome::Duplicate
    );
    let all = rows(&a, "A.ARW");
    assert_eq!(
        (all[1].outcome, all[1].association),
        (DeliveryOutcome::Duplicate, Association::Filename)
    );
    watch(&a, &all[0].final_path);
    watch(&a, &all[1].final_path);
    assert_eq!(photos(&a, s).len(), 1);
}

#[test]
fn c3d_same_name_other_bytes() {
    let a = app();
    let (item_s, s) = item(&a, "s");
    activate(&a, s);
    let bytes = [
        b"II*\0 d1".to_vec(),
        b"II*\0 d2".to_vec(),
        b"II*\0 d3".to_vec(),
    ];
    for b in &bytes {
        deliver(&a, "B.ARW", b, Link::Wireless).unwrap();
    }
    let folder = a.capture_root().join(key_of(&a, item_s));
    let all = rows(&a, "B.ARW");
    assert_eq!(
        all.iter()
            .map(|r| (r.outcome, r.final_path.clone()))
            .collect::<Vec<_>>(),
        vec![
            (
                DeliveryOutcome::Photo,
                folder.join("B.ARW").to_string_lossy().into_owned()
            ),
            (
                DeliveryOutcome::RenamedPhoto,
                folder.join("B__dup1.ARW").to_string_lossy().into_owned()
            ),
            (
                DeliveryOutcome::RenamedPhoto,
                folder.join("B__dup2.ARW").to_string_lossy().into_owned()
            ),
        ]
    );
    for (r, b) in all.iter().zip(&bytes) {
        assert_eq!(&fs::read(&r.final_path).unwrap(), b);
        watch(&a, &r.final_path);
    }
    assert_eq!(photos(&a, s).len(), 3);
}

#[test]
fn c3e_duplicates_do_not_complete_a_count() {
    let a = app();
    let (_, s) = item(&a, "s");
    activate(&a, s);
    let e = expose(&a, 0);
    a.set_expected_files(e, 2).unwrap();
    let d = b"II*\0 raw 10".to_vec();
    at(&a, 1_000);
    deliver(&a, "DSC00010.ARW", &d, Link::Wireless).unwrap();
    assert_eq!(exposure_status(&a, e), ExposureStatus::Expecting);
    assert!(!a.ledger().complete_for(e).unwrap());
    at(&a, 2_000);
    assert_eq!(
        deliver(&a, "DSC00010.ARW", &d, Link::Cable).unwrap(),
        DeliveryOutcome::Duplicate
    );
    assert_eq!(exposure_status(&a, e), ExposureStatus::Expecting);
    assert!(!a.ledger().complete_for(e).unwrap());
    at(&a, 3_000);
    deliver(&a, "DSC00010.JPG", b"jpeg 10", Link::Wireless).unwrap();
    assert_eq!(row(&a, "DSC00010.JPG").association, Association::Stem);
    assert_eq!(exposure_status(&a, e), ExposureStatus::Received);
    assert!(a.ledger().complete_for(e).unwrap());
    assert_eq!(files_in(&a.capture_root().join(".duplicates")).len(), 1);
}

#[test]
fn c3f_existing_destination_never_replaced() {
    let a = app();
    let (item_s, s) = item(&a, "s");
    activate(&a, s);
    let folder = a.capture_root().join(key_of(&a, item_s));
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("C.ARW"), b"x").unwrap();
    assert_eq!(
        deliver(&a, "C.ARW", b"x", Link::Wireless).unwrap(),
        DeliveryOutcome::Duplicate
    );
    assert_eq!(
        deliver(&a, "C.ARW", b"y", Link::Wireless).unwrap(),
        DeliveryOutcome::RenamedPhoto
    );
    let all = rows(&a, "C.ARW");
    assert_eq!(
        all[0].final_path,
        a.capture_root().join(".duplicates/C.ARW").to_string_lossy()
    );
    assert_eq!(
        all[1].final_path,
        folder.join("C__dup1.ARW").to_string_lossy()
    );
    assert_eq!(fs::read(folder.join("C.ARW")).unwrap(), b"x");
    assert_eq!(fs::read(folder.join("C__dup1.ARW")).unwrap(), b"y");
}

#[test]
fn c3g_reused_name_after_counter_wrap() {
    let a = app();
    let (item_s, s) = item(&a, "s");
    let (item_t, t) = item(&a, "t");
    activate(&a, s);
    expose(&a, 0);
    at(&a, 5_000);
    deliver(&a, "DSC01234.ARW", b"II*\0 first", Link::Wireless).unwrap();
    a.clock.advance(2 * 3_600_000);
    activate(&a, t);
    let now = a.clock.now_ms();
    let e2 = expose(&a, now);
    at(&a, now + 3_000);
    assert_eq!(
        deliver(&a, "DSC01234.ARW", b"II*\0 second", Link::Wireless).unwrap(),
        DeliveryOutcome::RenamedPhoto
    );
    let all = rows(&a, "DSC01234.ARW");
    let second = &all[1];
    assert_eq!(
        (second.association, second.exposure_id, second.session_id),
        (Association::Fifo, Some(e2), Some(t))
    );
    assert_eq!(
        second.final_path,
        a.capture_root()
            .join(key_of(&a, item_t))
            .join("DSC01234__dup1.ARW")
            .to_string_lossy()
    );
    assert_eq!(exposure_status(&a, e2), ExposureStatus::Received);
    watch(&a, &all[0].final_path);
    watch(&a, &second.final_path);
    assert_eq!(photos(&a, t).len(), 1);
    assert_eq!(photos(&a, s).len(), 1);
    assert_eq!(
        photos(&a, s)[0].raw_path,
        a.capture_root()
            .join(key_of(&a, item_s))
            .join("DSC01234.ARW")
            .to_string_lossy()
    );
}

#[test]
fn c4a_target_folder_of_the_matched_exposure() {
    let a = app();
    let (item_s, s) = item(&a, "s");
    let (_, t) = item(&a, "t");
    activate(&a, s);
    expose(&a, 0);
    activate(&a, t);
    at(&a, 3_000);
    deliver(&a, "D.ARW", b"II*\0 D", Link::Wireless).unwrap();
    let r = row(&a, "D.ARW");
    assert_eq!(
        r.final_path,
        a.capture_root()
            .join(key_of(&a, item_s))
            .join("D.ARW")
            .to_string_lossy()
    );
    watch(&a, &r.final_path);
    assert_eq!(photos(&a, s).len(), 1);
    assert!(photos(&a, t).is_empty());
}

#[test]
fn c4b_no_exposure_session_at_completion() {
    let a = app();
    let (item_t, t) = item(&a, "t");
    activate(&a, t);
    deliver(&a, "E.ARW", b"II*\0 E", Link::Wireless).unwrap();
    let r = row(&a, "E.ARW");
    assert_eq!(
        r.final_path,
        a.capture_root()
            .join(key_of(&a, item_t))
            .join("E.ARW")
            .to_string_lossy()
    );
    assert_eq!(r.association, Association::None);
    assert_eq!(r.session_id, Some(t));
}

#[test]
fn c4c_no_session_goes_to_unassigned() {
    let a = app();
    let (_, s) = item(&a, "s");
    activate(&a, s);
    a.sessions().deactivate().unwrap();
    deliver(&a, "F.ARW", b"II*\0 F", Link::Wireless).unwrap();
    let r = row(&a, "F.ARW");
    let expected = a.capture_root().join("_Unassigned/F.ARW");
    assert_eq!(r.final_path, expected.to_string_lossy());
    watch(&a, &expected);
    assert_eq!(photos(&a, 1).len(), 1);
    assert!(photos(&a, s).is_empty());
}

#[test]
fn c4d_folder_key_read_at_rename_time() {
    let a = app();
    let (item_s, s) = item(&a, "K1");
    activate(&a, s);
    expose(&a, 0);
    a.items().update_identity(item_s, Some("K2"), None).unwrap();
    let k2 = folder_key("K2").unwrap();
    assert_eq!(key_of(&a, item_s), k2);
    let folder = a.capture_root().join(&k2);
    assert!(!folder.exists());
    deliver(&a, "G.ARW", b"II*\0 G", Link::Wireless).unwrap();
    assert_eq!(
        row(&a, "G.ARW").final_path,
        folder.join("G.ARW").to_string_lossy()
    );
    assert!(folder.is_dir());
    watch(&a, folder.join("G.ARW"));
    assert_eq!(photos(&a, s).len(), 1);
}

#[test]
fn c4e_committed_delivery_freezes_the_key() {
    let a = app();
    let (item_s, s) = item(&a, "K1");
    activate(&a, s);
    a.items().update_identity(item_s, Some("K2"), None).unwrap();
    deliver(&a, "G2.ARW", b"II*\0 G2", Link::Wireless).unwrap();
    assert_eq!(row(&a, "G2.ARW").state, DeliveryState::Done);
    assert!(photos(&a, s).is_empty());
    a.items().update_identity(item_s, Some("K3"), None).unwrap();
    assert_eq!(key_of(&a, item_s), folder_key("K2").unwrap());
    assert!(a.store().item_folder_key_frozen(item_s).unwrap());
    // Control: no delivery and no photo → the same edit moves the key.
    let (item_u, _) = item(&a, "U1");
    a.items().update_identity(item_u, Some("U3"), None).unwrap();
    assert_eq!(key_of(&a, item_u), folder_key("U3").unwrap());
    assert!(!a.store().item_folder_key_frozen(item_u).unwrap());
}

#[test]
fn c4f_rule_zero_before_the_folder_rules() {
    let a = app();
    let (_, s) = item(&a, "K2");
    let (_, t) = item(&a, "T9");
    activate(&a, s);
    deliver(&a, "H.ARW", b"II*\0 H", Link::Wireless).unwrap();
    let r = row(&a, "H.ARW");
    assert_eq!(r.session_id, Some(s));
    db(&a)
        .execute("DELETE FROM sessions WHERE id=?1", [s])
        .unwrap();
    activate(&a, t);
    watch(&a, &r.final_path);
    assert_eq!(photos(&a, 1).len(), 1);
    assert!(photos(&a, t).is_empty());
}

#[test]
fn c4g_captured_at_from_the_exposure() {
    let a = app();
    let (_, s) = item(&a, "s");
    activate(&a, s);
    expose(&a, 1_000);
    at(&a, 5_000);
    deliver(&a, "X1.ARW", b"II*\0 X1", Link::Wireless).unwrap();
    let first = row(&a, "X1.ARW");
    watch(&a, &first.final_path);
    at(&a, 7_000);
    deliver(&a, "X2.ARW", b"II*\0 X2", Link::Wireless).unwrap();
    let second = row(&a, "X2.ARW");
    assert_eq!(second.association, Association::None);
    watch(&a, &second.final_path);
    let by_path = |path: &str| {
        photos(&a, s)
            .into_iter()
            .find(|p| p.raw_path == path)
            .unwrap()
    };
    let mtime = |path: &str| {
        fs::metadata(path)
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    };
    assert_eq!(by_path(&first.final_path).captured_at, Some(1_000));
    assert_eq!(by_path(&second.final_path).captured_at, Some(7_000));
    assert_ne!(mtime(&first.final_path), 1_000);
    assert_ne!(mtime(&second.final_path), 7_000);
}

#[test]
fn c5a_crash_ordered_persistence() {
    let mut a = app();
    let (item_s, s) = item(&a, "s");
    activate(&a, s);
    a.ledger_fs_fail_next_rename();
    let e = expose(&a, 0);
    assert!(deliver(&a, "I0.ARW", b"II*\0 I0", Link::Wireless).is_err());
    let r = row(&a, "I0.ARW");
    assert_eq!(r.state, DeliveryState::Pending);
    let written = staging(&a, "I0.ARW", Link::Wireless);
    assert_eq!(r.written_path, written.to_string_lossy());
    assert_eq!(
        r.final_path,
        a.capture_root()
            .join(key_of(&a, item_s))
            .join("I0.ARW")
            .to_string_lossy()
    );
    assert!(written.exists());
    assert_eq!(exposure_status(&a, e), ExposureStatus::Received);
    a.kill_and_restart().unwrap();
    let after = row(&a, "I0.ARW");
    assert_eq!(after.state, DeliveryState::Done);
    assert_eq!(after.final_path, r.final_path);
    assert_eq!(fs::read(&r.final_path).unwrap(), b"II*\0 I0");
    assert!(!written.exists());
}

#[test]
fn c5b_pending_with_the_final_file_present() {
    let mut a = app();
    let (_, s) = item(&a, "s");
    activate(&a, s);
    a.ledger_fs_fail_next_rename();
    assert!(deliver(&a, "I1.ARW", b"II*\0 I1", Link::Wireless).is_err());
    let r = row(&a, "I1.ARW");
    // A crash after the rename and before `done`: the file is already at its final path.
    fs::create_dir_all(Path::new(&r.final_path).parent().unwrap()).unwrap();
    fs::rename(&r.written_path, &r.final_path).unwrap();
    let count = a.ledger().deliveries().unwrap().len();
    a.kill_and_restart().unwrap();
    assert_eq!(row(&a, "I1.ARW").state, DeliveryState::Done);
    assert_eq!(fs::read(&r.final_path).unwrap(), b"II*\0 I1");
    assert_eq!(a.ledger().deliveries().unwrap().len(), count);
    assert!(files_in(&a.capture_root().join(".duplicates")).is_empty());
    let mut dup_files = Vec::new();
    for entry in fs::read_dir(a.capture_root()).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            dup_files.extend(files_in(&path).into_iter().filter(|n| n.contains("__dup")));
        }
    }
    assert!(dup_files.is_empty());
}

#[test]
fn c5c_late_file_after_restart_finds_its_exposure() {
    let mut a = app();
    let (_, s) = item(&a, "s");
    activate(&a, s);
    let e = expose(&a, 0);
    a.kill_and_restart().unwrap();
    at(&a, 60_000);
    deliver(&a, "I.ARW", b"II*\0 I", Link::Wireless).unwrap();
    let r = row(&a, "I.ARW");
    assert_eq!((r.association, r.exposure_id), (Association::Fifo, Some(e)));
    assert_eq!(exposure_status(&a, e), ExposureStatus::Received);
}

#[test]
fn c5d_resent_staging_file_takes_over_its_pending_row() {
    let a = app();
    let (item_s, s) = item(&a, "s");
    activate(&a, s);
    a.ledger_fs_fail_next_rename();
    expose(&a, 0);
    let d = b"II*\0 K".to_vec();
    at(&a, 2_000);
    assert!(deliver(&a, "K.ARW", &d, Link::Wireless).is_err());
    at(&a, 4_000);
    assert_eq!(
        deliver(&a, "K.ARW", &d, Link::Wireless).unwrap(),
        DeliveryOutcome::Photo
    );
    let r = row(&a, "K.ARW");
    assert_eq!(
        (r.state, r.completed_at, r.outcome),
        (DeliveryState::Done, 4_000, DeliveryOutcome::Photo)
    );
    let expected = a.capture_root().join(key_of(&a, item_s)).join("K.ARW");
    assert_eq!(r.final_path, expected.to_string_lossy());
    assert_eq!(fs::read(&expected).unwrap(), d);
    assert!(files_in(&a.capture_root().join(".duplicates")).is_empty());
    watch(&a, &expected);
    assert_eq!(photos(&a, s).len(), 1);
}

#[test]
fn c6b_raise_iff_count_and_no_transfer() {
    let a = app();
    expose(&a, 0);
    at(&a, 61_000);
    let now = a.clock.now_ms();
    let busy = a.ledger().watchdog(now, true).unwrap();
    assert!(!busy.raised);
    assert_eq!(health(&a, "camera_files_not_received"), None);
    let idle = a.ledger().watchdog(now, false).unwrap();
    assert!(idle.raised && idle.first_raise);
    assert_eq!(
        health(&a, "camera_files_not_received").as_deref(),
        Some("1 photos taken but not received for 60 s")
    );
    let again = a.ledger().watchdog(now, false).unwrap();
    assert!(again.raised && !again.first_raise);
}

#[test]
fn c6c_clear_and_dismiss() {
    let a = app();
    let (_, s) = item(&a, "s");
    activate(&a, s);
    // Cleared when the count reaches 0.
    expose(&a, 0);
    at(&a, 61_000);
    assert!(a.ledger().watchdog(61_000, false).unwrap().raised);
    deliver(&a, "L1.ARW", b"II*\0 L1", Link::Wireless).unwrap();
    let cleared = a.ledger().watchdog(a.clock.now_ms(), false).unwrap();
    assert_eq!((cleared.pending_count, cleared.raised), (0, false));
    assert_eq!(health(&a, "camera_files_not_received"), None);
    // Dismissed by the photographer.
    let e2 = expose(&a, 100_000);
    at(&a, 161_000);
    assert!(a.ledger().watchdog(161_000, false).unwrap().raised);
    a.ledger().dismiss().unwrap();
    assert_eq!(exposure_status(&a, e2), ExposureStatus::Dismissed);
    assert_eq!(health(&a, "camera_files_not_received"), None);
    assert_eq!(
        a.ledger().watchdog(161_000, false).unwrap().pending_count,
        0
    );
    at(&a, 170_000);
    deliver(&a, "L2.ARW", b"II*\0 L2", Link::Wireless).unwrap();
    assert_eq!(row(&a, "L2.ARW").association, Association::None);
    assert_eq!(exposure_status(&a, e2), ExposureStatus::Dismissed);
}

#[test]
fn c6d_gaps_on_the_health_item() {
    let a = app();
    deliver(&a, "DSC00001.ARW", b"II*\0 1", Link::Wireless).unwrap();
    at(&a, 1_000);
    deliver(&a, "DSC00004.ARW", b"II*\0 4", Link::Wireless).unwrap();
    assert_eq!(row(&a, "DSC00001.ARW").association, Association::None);
    assert_eq!(row(&a, "DSC00004.ARW").association, Association::None);
    expose(&a, 2_000);
    at(&a, 63_000);
    let state = a.ledger().watchdog(63_000, false).unwrap();
    assert_eq!(state.pending_count, 1);
    assert_eq!(
        state.gaps,
        vec![Gap {
            prefix: "DSC".into(),
            after: 1,
            before: 4,
            missing: 2
        }]
    );
    assert_eq!(
        health(&a, "camera_files_not_received").as_deref(),
        Some("1 photos taken but not received for 60 s · 1 gaps in the camera's numbering")
    );
}

#[test]
fn c7a_uncertain_after_session_loss() {
    let a = app();
    let (item_s, s) = item(&a, "s");
    activate(&a, s);
    let e = expose(&a, 0);
    a.ledger().session_uncertain().unwrap();
    assert_eq!(exposure_status(&a, e), ExposureStatus::Uncertain);
    at(&a, 9_000);
    deliver(&a, "J.ARW", b"II*\0 J", Link::Wireless).unwrap();
    let r = row(&a, "J.ARW");
    assert_eq!(
        (r.association, r.exposure_id, r.session_id),
        (Association::None, None, Some(s))
    );
    assert_eq!(
        r.final_path,
        a.capture_root()
            .join(key_of(&a, item_s))
            .join("J.ARW")
            .to_string_lossy()
    );
    assert_eq!(exposure_status(&a, e), ExposureStatus::Uncertain);
}

#[test]
fn c7b_split_candidate_set_lists_the_photo() {
    let a = app();
    let (item_s, s) = item(&a, "s");
    let (item_t, t) = item(&a, "t");
    activate(&a, s);
    let e1 = expose(&a, 0);
    activate(&a, t);
    let e2 = expose(&a, 1_000);
    activate(&a, s);
    expose(&a, 2_000);
    at(&a, 5_000);
    deliver(&a, "M.ARW", b"II*\0 M", Link::Wireless).unwrap();
    let m = row(&a, "M.ARW");
    assert_eq!(
        (m.exposure_id, m.association, m.session_id),
        (Some(e1), Association::Fifo, Some(s))
    );
    assert_eq!(
        m.final_path,
        a.capture_root()
            .join(key_of(&a, item_s))
            .join("M.ARW")
            .to_string_lossy()
    );
    assert_eq!(
        health(&a, "camera_photo_session_uncertain").as_deref(),
        Some("Photo M.ARW may belong to t")
    );
    at(&a, 6_000);
    deliver(&a, "N.ARW", b"II*\0 N", Link::Wireless).unwrap();
    let n = row(&a, "N.ARW");
    assert_eq!((n.exposure_id, n.session_id), (Some(e2), Some(t)));
    assert_eq!(
        n.final_path,
        a.capture_root()
            .join(key_of(&a, item_t))
            .join("N.ARW")
            .to_string_lossy()
    );
    assert_eq!(
        health(&a, "camera_photo_session_uncertain").as_deref(),
        Some("Photo N.ARW may belong to s")
    );
}

#[test]
fn c8a_receiver_sink_is_the_ledger() {
    let a = app();
    let (item_s, s) = item(&a, "s");
    activate(&a, s);
    expose(&a, 0);
    let data = b"II*\0 uploaded through the receiver".repeat(1_000);
    let mut c = a.ftp();
    assert_eq!(&c.stor("DSC00001.ARW", &data).unwrap()[..3], "226");
    settle(&a, || {
        rows(&a, "DSC00001.ARW")
            .first()
            .is_some_and(|r| r.state == DeliveryState::Done)
    });
    let r = row(&a, "DSC00001.ARW");
    assert_eq!(
        (r.via.clone(), r.association, r.outcome, r.state),
        (
            Link::Wireless,
            Association::Fifo,
            DeliveryOutcome::Photo,
            DeliveryState::Done
        )
    );
    let expected = a
        .capture_root()
        .join(key_of(&a, item_s))
        .join("DSC00001.ARW");
    assert_eq!(r.final_path, expected.to_string_lossy());
    assert_eq!(fs::read(&expected).unwrap(), data);
    assert!(
        files_in(&a.capture_root().join(".incoming"))
            .iter()
            .all(|n| !n.ends_with(".part"))
    );
    watch(&a, &expected);
    let in_s = photos(&a, s);
    assert_eq!(in_s.len(), 1);
    assert_eq!(in_s[0].raw_path, r.final_path);
    assert_eq!(
        c.cmd("SIZE DSC00001.ARW").unwrap(),
        format!("213 {}", data.len())
    );
}

#[test]
fn c8b_arrival_ignores_the_camera_folders() {
    let a = app();
    let (item_s, s, _) = cable_then_ftp(&a);
    watch(
        &a,
        a.capture_root()
            .join(key_of(&a, item_s))
            .join("DSC04622.ARW"),
    );
    assert_eq!(photos(&a, s).len(), 1);
    let staged = a.capture_root().join(".incoming/X.ARW");
    fs::write(&staged, b"II*\0 staged raw").unwrap();
    watch(&a, a.capture_root().join(".duplicates/DSC04622.ARW"));
    watch(&a, &staged);
    a.scan().unwrap();
    assert_eq!(photos(&a, s).len(), 1);
    let total: i64 = db(&a)
        .query_row("SELECT COUNT(*) FROM photos", [], |r| r.get(0))
        .unwrap();
    assert_eq!(total, 1);
}
