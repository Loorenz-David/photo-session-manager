//! Owner correction A21 (2026-11-18): photo files follow their session. Evidence (work Mac): eight
//! photos for T1-181126 were filed into T4-181126's folder, then moved to T1 in the app; their RAWs
//! and Capture One's files stayed in T4's folder, T1's folder stayed empty and T1's export waited
//! forever for Capture One. Now moving photos (and undo, redo, "Assign product") moves the RAW,
//! Capture One's settings, masks and cache files of it, and the app's caches of it, into the
//! session's folder; "Fix photo files location" rescues photos moved before.
use psm_core::{
    domain::photo::{NewPhoto, PhotoSource},
    mechanisms::{arrival::path_key, canon::canon},
    ports::clock::Clock,
    services::{history::HistoryService, review::ReviewService},
};
use psm_integrations::photo_files;
use psm_testkit::TestApp;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

fn session(a: &TestApp, sku: &str) -> (i64, String) {
    let (item, session) = a.items().create_local(sku, sku, "object", None).unwrap();
    (session.session.id, item.folder_key)
}

/// A RAW in `key`'s folder with its row, raw state and a preview cache folder; Capture One's
/// settings, mask, proxy and thumbnail of it, and another RAW's settings beside them.
fn photo(a: &TestApp, session_id: i64, key: &str, name: &str) -> (i64, String) {
    let folder = a.capture_root().join(key);
    let path = folder.join(name);
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(&path, format!("raw {name}")).unwrap();
    let raw = canon(&path).unwrap();
    for (relative, body) in [
        (format!("CaptureOne/Settings1690/{name}.cos"), "adjusted"),
        (format!("CaptureOne/Settings1690/{name}.comask"), "mask"),
        (format!("CaptureOne/Cache/Proxies/{name}.cop"), "proxy"),
        (
            format!("CaptureOne/Cache/Thumbnails/{name}.[00c977eb].cot"),
            "thumb",
        ),
    ] {
        let file = folder.join(relative);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, body).unwrap();
    }
    let row = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id,
            raw_path: raw.clone(),
            raw_hash: Some(format!("hash-{name}")),
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    a.store().upsert_raw_state(&raw, None).unwrap();
    let key_dir = preview_dir(a, &raw);
    std::fs::create_dir_all(&key_dir).unwrap();
    std::fs::write(key_dir.join("original.jpg"), b"preview").unwrap();
    Connection::open(a.app.paths.db())
        .unwrap()
        .execute(
            "UPDATE photos SET original_preview_path=?1 WHERE id=?2",
            rusqlite::params![key_dir.join("original.jpg").to_string_lossy(), row.id],
        )
        .unwrap();
    (row.id, raw)
}
fn preview_dir(a: &TestApp, raw: &str) -> PathBuf {
    let key = path_key(raw);
    a.app.paths.root.join("previews").join(&key[..2]).join(&key)
}
fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

/// Moving photos moves the RAW and every Capture One file of it into the target's folder, keeps
/// Capture One's layout, updates the rows (preview path included), and asks Capture One to look
/// at both folders. Another RAW's files stay.
#[test]
fn a21_a_moving_photos_moves_their_files() {
    let a = TestApp::with_fake_document().unwrap();
    let (t4, t4_key) = session(&a, "T4-181126");
    let (t1, t1_key) = session(&a, "T1-181126");
    let (moving, old) = photo(&a, t4, &t4_key, "DSC06785.ARW");
    let (staying, _) = photo(&a, t4, &t4_key, "DSC06784.ARW");
    a.fake.state.lock().unwrap().ops.clear();

    photo_files::move_photos(&a.app.handles(), &[moving], t1).unwrap();

    let t1_dir = canon(&a.capture_root().join(&t1_key)).unwrap();
    let t4_dir = canon(&a.capture_root().join(&t4_key)).unwrap();
    let new = format!("{t1_dir}/DSC06785.ARW");
    let row = a.store().photo(moving).unwrap();
    assert_eq!(row.session_id, t1);
    assert_eq!(row.raw_path, new);
    assert_eq!(read(Path::new(&new)), "raw DSC06785.ARW");
    assert!(!Path::new(&old).exists());
    for (relative, body) in [
        ("CaptureOne/Settings1690/DSC06785.ARW.cos", "adjusted"),
        ("CaptureOne/Settings1690/DSC06785.ARW.comask", "mask"),
        ("CaptureOne/Cache/Proxies/DSC06785.ARW.cop", "proxy"),
        (
            "CaptureOne/Cache/Thumbnails/DSC06785.ARW.[00c977eb].cot",
            "thumb",
        ),
    ] {
        assert_eq!(read(&Path::new(&t1_dir).join(relative)), body, "{relative}");
        assert!(!Path::new(&t4_dir).join(relative).exists(), "{relative}");
    }
    // The other photo and its files stay in T4.
    assert!(
        Path::new(&t4_dir)
            .join("CaptureOne/Settings1690/DSC06784.ARW.cos")
            .exists()
    );
    assert_eq!(a.store().photo(staying).unwrap().session_id, t4);
    // The app's preview cache follows the new path; the row points into it.
    let moved_preview = preview_dir(&a, &new).join("original.jpg");
    assert_eq!(read(&moved_preview), "preview");
    assert_eq!(
        row.original_preview_path.as_deref(),
        Some(moved_preview.to_string_lossy().as_ref())
    );
    assert!(!preview_dir(&a, &old).exists());
    let state: (String, String) = Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT raw_path, path_key FROM raw_state WHERE raw_path=?1",
            [&new],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(state, (new.clone(), path_key(&new)));
    let ops = ops(&a);
    assert!(ops.contains(&format!("sync_folder:{t4_dir}")), "{ops:?}");
    assert!(ops.contains(&format!("sync_folder:{t1_dir}")), "{ops:?}");
}

/// A same-named file in the target folder is never overwritten: the photo's files stay where they
/// are, its row keeps its path, and the photo is reported as left.
#[test]
fn a21_b_a_same_named_file_is_never_overwritten() {
    let a = TestApp::with_fake_document().unwrap();
    let (t4, t4_key) = session(&a, "T4-181126");
    let (t1, t1_key) = session(&a, "T1-181126");
    let (moving, old) = photo(&a, t4, &t4_key, "DSC06785.ARW");
    let taken = a
        .capture_root()
        .join(&t1_key)
        .join("CaptureOne/Settings1690/DSC06785.ARW.cos");
    std::fs::create_dir_all(taken.parent().unwrap()).unwrap();
    std::fs::write(&taken, "someone else's").unwrap();

    photo_files::move_photos(&a.app.handles(), &[moving], t1).unwrap();

    assert_eq!(read(&taken), "someone else's");
    assert_eq!(a.store().photo(moving).unwrap().raw_path, old);
    assert!(Path::new(&old).exists());
    assert!(!a.capture_root().join(&t1_key).join("DSC06785.ARW").exists());
    let misplaced = photo_files::misplaced(&a.app.handles(), t1).unwrap();
    assert_eq!(misplaced.len(), 1);
    let fixed = photo_files::fix(&a.app.handles(), t1).unwrap();
    assert_eq!(fixed.moved, 0);
    assert!(
        fixed.skipped[0].1.contains("already in T1-181126"),
        "{fixed:?}"
    );
}

/// The rescue for photos moved before A21 (rows only, files left behind): the session's ⋯ menu
/// counts them, and "Fix photo files location" moves them with their adjustments.
#[test]
fn a21_c_fix_rescues_photos_moved_before() {
    let a = TestApp::with_fake_document().unwrap();
    let (t4, t4_key) = session(&a, "T4-181126");
    let (t1, t1_key) = session(&a, "T1-181126");
    let ids: Vec<i64> = (85..93)
        .map(|n| photo(&a, t4, &t4_key, &format!("DSC067{n:02}.ARW")).0)
        .collect();
    // The pre-A21 move: rows only.
    ReviewService::new(a.app.store.as_ref(), &a.clock, a.app.history.as_ref())
        .move_photos(&ids, t1)
        .unwrap();
    assert_eq!(
        photo_files::misplaced(&a.app.handles(), t1).unwrap().len(),
        8
    );
    assert!(
        photo_files::misplaced(&a.app.handles(), t4)
            .unwrap()
            .is_empty()
    );

    let fixed = photo_files::fix(&a.app.handles(), t1).unwrap();
    assert_eq!((fixed.moved, fixed.sidecars), (8, 32), "{fixed:?}");
    assert!(
        photo_files::misplaced(&a.app.handles(), t1)
            .unwrap()
            .is_empty()
    );
    let t1_dir = a.capture_root().join(&t1_key);
    for id in ids {
        let raw = a.store().photo(id).unwrap().raw_path;
        assert_eq!(
            Path::new(&raw).parent().unwrap(),
            canon(&t1_dir).unwrap().as_str()
        );
        let name = Path::new(&raw).file_name().unwrap().to_string_lossy();
        assert_eq!(
            read(&t1_dir.join(format!("CaptureOne/Settings1690/{name}.cos"))),
            "adjusted"
        );
    }
}

/// Refused up front while an export or a publish step of a session involved runs: nothing moves.
#[test]
fn a21_d_refused_while_a_publish_runs() {
    let a = TestApp::with_fake_document().unwrap();
    let (t4, t4_key) = session(&a, "T4-181126");
    let (t1, _) = session(&a, "T1-181126");
    let (moving, old) = photo(&a, t4, &t4_key, "DSC06785.ARW");
    a.insert_publish_item(t4, "upload", a.clock.now_ms(), None)
        .unwrap();
    match photo_files::move_photos(&a.app.handles(), &[moving], t1) {
        Err(psm_core::Error::FilesBusy(reason)) => {
            assert!(reason.contains("published"), "{reason}")
        }
        other => panic!("not refused: {other:?}"),
    }
    assert_eq!(a.store().photo(moving).unwrap().session_id, t4);
    assert!(Path::new(&old).exists());
}

/// Undo of a move moves the files back (they follow the photo's session again); a copy shares
/// the RAW and moves nothing.
#[test]
fn a21_e_undo_moves_the_files_back_and_a_copy_moves_nothing() {
    let a = TestApp::with_fake_document().unwrap();
    let (t4, t4_key) = session(&a, "T4-181126");
    let (t1, t1_key) = session(&a, "T1-181126");
    let (moving, old) = photo(&a, t4, &t4_key, "DSC06785.ARW");
    photo_files::move_photos(&a.app.handles(), &[moving], t1).unwrap();
    assert!(!Path::new(&old).exists());

    let outcome = HistoryService::new(a.app.store.as_ref(), &a.clock, a.app.history.as_ref())
        .undo(t4)
        .unwrap();
    let sessions: Vec<i64> = outcome.sessions.iter().map(|s| s.session.id).collect();
    photo_files::after_history(&a.app.handles(), &sessions);
    assert_eq!(a.store().photo(moving).unwrap().session_id, t4);
    assert_eq!(a.store().photo(moving).unwrap().raw_path, old);
    assert_eq!(read(Path::new(&old)), "raw DSC06785.ARW");
    assert!(
        a.capture_root()
            .join(&t4_key)
            .join("CaptureOne/Settings1690/DSC06785.ARW.cos")
            .exists()
    );
    assert!(!a.capture_root().join(&t1_key).join("DSC06785.ARW").exists());

    // A copy: both rows name the same RAW, which stays where it is.
    ReviewService::new(a.app.store.as_ref(), &a.clock, a.app.history.as_ref())
        .copy_photos(&[moving], t1)
        .unwrap();
    assert!(
        photo_files::misplaced(&a.app.handles(), t1)
            .unwrap()
            .is_empty()
    );
    let fixed = photo_files::fix(&a.app.handles(), t1).unwrap();
    assert_eq!(fixed.moved, 0);
    assert!(Path::new(&old).exists());
}

/// Why T1's export waited forever: Capture One answers a lookup in a folder it is not showing from
/// its own list of that folder, refreshed only when it looks at it. A photo on disk that the list
/// lacks now makes the app ask Capture One to re-read the folder (once a minute at most), and the
/// next try finds it.
#[test]
fn a21_f_a_photo_capture_one_does_not_list_rereads_its_folder() {
    let a = TestApp::with_fake_document().unwrap();
    let (t4, t4_key) = session(&a, "T4-181126");
    let (_, raw) = photo(&a, t4, &t4_key, "DSC06787.ARW");
    a.fake.list_after_sync("DSC06787.ARW", "v-6787");
    a.fake.state.lock().unwrap().ops.clear();
    let folder = Path::new(&raw)
        .parent()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let editing = a.app.editing.clone();
    assert!(editing.apply_style(&raw, "PSM House").is_err());
    let synced = |ops: &[String]| {
        ops.iter()
            .filter(|op| **op == format!("sync_folder:{folder}"))
            .count()
    };
    assert_eq!(synced(&ops(&a)), 1, "{:?}", ops(&a));
    // A second miss within the minute does not ask again.
    assert!(editing.apply_style(&raw, "PSM House").is_err());
    assert_eq!(synced(&ops(&a)), 1);
    a.clock.advance(3_000);
    editing.apply_style(&raw, "PSM House").unwrap();
}
