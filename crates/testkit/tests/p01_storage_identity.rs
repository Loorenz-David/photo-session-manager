use psm_core::{
    Error,
    domain::{
        item::ItemOrigin,
        photo::{Integrity, NewPhoto, PhotoSource},
    },
    mechanisms::{canon::canon, hash::sha256_file, naming::folder_key},
    ports::clock::Clock,
    services::integrity::IntegrityCheck,
};
use psm_testkit::TestApp;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};
use unicode_normalization::UnicodeNormalization;

fn photo(
    app: &TestApp,
    path: &Path,
    session_id: i64,
    hash: Option<String>,
) -> psm_core::domain::photo::Photo {
    app.store()
        .photo_insert_or_get(NewPhoto {
            session_id,
            raw_path: path.to_string_lossy().into_owned(),
            raw_hash: hash,
            captured_at: Some(app.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: app.clock.now_ms(),
        })
        .unwrap()
}

fn item_session(app: &TestApp, sku: &str) -> i64 {
    app.items()
        .create_local(sku, "Chair", "Seating", None)
        .unwrap()
        .1
        .session
        .id
}

#[test]
fn c1a_open_creates_all_domain_columns() {
    let app = TestApp::new().unwrap();
    let conn = Connection::open(app.app.paths.db()).unwrap();
    let expected: &[(&str, &[&str])] = &[
        ("settings", &["key", "value"]),
        (
            "capture_events",
            &["id", "source", "hook", "payload", "received_at", "error"],
        ),
        (
            "items",
            &[
                "id",
                "sku",
                "folder_key",
                "folder_key_ci",
                "barcode",
                "item_type",
                "title",
                "origin",
                "created_at",
                "updated_at",
            ],
        ),
        (
            "item_links",
            &[
                "item_id",
                "integration",
                "external_id",
                "external_url",
                "last_synced_at",
                "create_pending",
                "tags_base",
            ],
        ),
        (
            "sessions",
            &[
                "id",
                "item_id",
                "published_at",
                "published_seq",
                "change_seq",
                "last_activity_at",
                "created_at",
                "undo_slot",
            ],
        ),
        (
            "photos",
            &[
                "id",
                "uid",
                "session_id",
                "raw_path",
                "raw_hash",
                "integrity",
                "provider_ref",
                "original_preview_path",
                "adjusted_preview_path",
                "orientation",
                "classification",
                "position",
                "captured_at",
                "source",
                "created_at",
            ],
        ),
        (
            "raw_state",
            &[
                "raw_path",
                "path_key",
                "adjusted_gen",
                "rendered_gen",
                "retouch_gen",
                "last_own_write_at",
            ],
        ),
        (
            "photo_links",
            &[
                "id",
                "photo_id",
                "integration",
                "item_id",
                "role",
                "external_id",
                "revision",
                "retouch_gen",
                "superseded_ids",
                "upload_state",
                "export_sha256",
                "error",
                "updated_at",
            ],
        ),
        (
            "attribute_definitions",
            &[
                "integration",
                "key",
                "label",
                "core_type",
                "choices",
                "validations",
                "grp",
                "description",
                "writable",
            ],
        ),
        (
            "attribute_definition_links",
            &["integration", "key", "platform_type", "platform_meta"],
        ),
        ("attribute_presets", &["item_type", "grp", "layout"]),
        (
            "attribute_values",
            &[
                "item_id",
                "key",
                "value",
                "valid",
                "provenance",
                "ai_confidence",
                "ai_status",
                "ai_reason",
                "ai_sources",
                "updated_at",
            ],
        ),
        (
            "publish_runs",
            &[
                "id",
                "integration",
                "created_at",
                "confirmed_at",
                "confirmed_summary",
            ],
        ),
        (
            "publish_items",
            &[
                "id",
                "run_id",
                "session_id",
                "create_item",
                "title",
                "frozen_main",
                "frozen_condition",
                "frozen_attributes",
                "announced_removals",
                "confirmed_change_seq",
                "step",
                "failed_step",
                "uploaded",
                "total",
                "error",
                "finished_at",
            ],
        ),
        (
            "jobs",
            &[
                "id",
                "kind",
                "target",
                "payload",
                "status",
                "attempts",
                "next_attempt_at",
                "error",
                "created_at",
                "updated_at",
            ],
        ),
        (
            "notifications",
            &["id", "session_id", "run_id", "kind", "text", "created_at"],
        ),
    ];
    let actual_tables: BTreeSet<String> = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '__rusqlite_%'").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
    let expected_tables: BTreeSet<String> = expected
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect();
    assert_eq!(actual_tables, expected_tables);
    for (table, columns) in expected {
        let actual: Vec<String> = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap()
            .query_map([], |r| r.get(1))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(actual, *columns, "{table}");
    }
}

#[test]
fn c1b_unassigned_is_one_null_item_session() {
    let app = TestApp::new().unwrap();
    let unassigned = app.sessions().unassigned().unwrap();
    assert_eq!(unassigned.session.id, 1);
    assert_eq!(unassigned.session.item_id, None);
    let conn = Connection::open(app.app.paths.db()).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE item_id IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn c2a_canon_normalizes_nfd_filename() {
    let dir = tempfile::tempdir().unwrap();
    let nfc = dir.path().join("caf\u{e9}.ARW");
    fs::write(&nfc, b"raw").unwrap();
    let nfd = dir.path().join("cafe\u{301}.ARW");
    assert_eq!(canon(&nfd).unwrap(), canon(&nfc).unwrap());
    let result = canon(&nfd).unwrap();
    assert_eq!(result, result.nfc().collect::<String>());
}

#[test]
fn c2b_canon_converges_tmp_private_tmp() {
    let dir = tempfile::tempdir_in("/tmp").unwrap();
    let path = dir.path().join("x.ARW");
    fs::write(&path, b"raw").unwrap();
    let private = Path::new("/private").join(path.strip_prefix("/").unwrap());
    assert_eq!(canon(&path).unwrap(), canon(&private).unwrap());
}

#[test]
fn c2c_canon_rejects_relative() {
    assert!(matches!(
        canon(Path::new("Capture/x.ARW")),
        Err(Error::RelativePath)
    ));
}

#[test]
fn c2d_canon_uses_disk_case() {
    let dir = tempfile::tempdir().unwrap();
    let disk = dir.path().join("Foo.ARW");
    let lower = dir.path().join("foo.arw");
    fs::write(&disk, b"raw").unwrap();
    if !lower.exists() {
        eprintln!("skip C2(d): temporary volume is case-sensitive");
        return;
    }
    assert_eq!(canon(&lower).unwrap(), canon(&disk).unwrap());
    assert!(canon(&lower).unwrap().ends_with("/Foo.ARW"));
}

#[test]
fn c2e_canon_resolves_parent_but_keeps_file_symlink() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real");
    fs::create_dir(&real).unwrap();
    fs::write(real.join("y.ARW"), b"raw").unwrap();
    symlink("y.ARW", real.join("x.ARW")).unwrap();
    symlink("real", dir.path().join("link")).unwrap();
    assert_eq!(
        canon(&dir.path().join("link/x.ARW")).unwrap(),
        format!("{}/x.ARW", real.canonicalize().unwrap().display())
    );
}

#[test]
fn c2f_canon_removes_parent_dotdot_and_trailing_slash() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("a")).unwrap();
    fs::create_dir(dir.path().join("b")).unwrap();
    fs::write(dir.path().join("b/x.ARW"), b"raw").unwrap();
    let input = format!("{}/a/../b/x.ARW/", dir.path().display());
    assert_eq!(
        canon(Path::new(&input)).unwrap(),
        canon(&dir.path().join("b/x.ARW")).unwrap()
    );
}

#[test]
fn c3a_store_deduplicates_normalized_raw_path() {
    let app = TestApp::new().unwrap();
    let session = item_session(&app, "SKU");
    assert_eq!(session, 2);
    let nfc = app.dir.path().join("caf\u{e9}.ARW");
    let nfd = app.dir.path().join("cafe\u{301}.ARW");
    fs::write(&nfc, b"raw").unwrap();
    let first = photo(&app, &nfd, session, None);
    let second = photo(&app, &nfc, session, None);
    assert_eq!(first.id, second.id);
    assert_eq!(first.raw_path, canon(&nfc).unwrap());
    assert_eq!(app.sessions().get(session).unwrap().photo_count, 1);
}

#[test]
fn c4a_sha256_streams_without_changing_raw() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("five-mib.ARW");
    let mut bytes = vec![0u8; 5 * 1024 * 1024];
    let mut state = 0x1234_5678u32;
    for byte in &mut bytes {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        *byte = state as u8;
    }
    fs::write(&path, &bytes).unwrap();
    let modified = fs::metadata(&path).unwrap().modified().unwrap();
    let expected: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let actual = sha256_file(&path).unwrap();
    assert_eq!(actual.len(), 64);
    assert_eq!(actual, expected);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
}

#[test]
fn c4b_changed_raw_sets_integrity_without_rehashing_record() {
    let app = TestApp::new().unwrap();
    let path = app.dir.path().join("changed.ARW");
    fs::write(&path, b"original").unwrap();
    let hash = sha256_file(&path).unwrap();
    let stored = photo(&app, &path, item_session(&app, "A"), Some(hash.clone()));
    use std::io::Write;
    fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap()
        .write_all(b"changed")
        .unwrap();
    assert_eq!(
        IntegrityCheck::new(app.store()).verify(stored.id).unwrap(),
        Integrity::RawChanged
    );
    let stored = app.store().photo(stored.id).unwrap();
    assert_eq!(stored.integrity, Integrity::RawChanged);
    assert_eq!(stored.raw_hash, Some(hash));
}

#[test]
fn c4c_missing_raw_sets_integrity_without_rehashing_record() {
    let app = TestApp::new().unwrap();
    let path = app.dir.path().join("missing.ARW");
    fs::write(&path, b"original").unwrap();
    let hash = sha256_file(&path).unwrap();
    let stored = photo(&app, &path, item_session(&app, "A"), Some(hash.clone()));
    fs::remove_file(path).unwrap();
    assert_eq!(
        IntegrityCheck::new(app.store()).verify(stored.id).unwrap(),
        Integrity::RawMissing
    );
    let stored = app.store().photo(stored.id).unwrap();
    assert_eq!(stored.integrity, Integrity::RawMissing);
    assert_eq!(stored.raw_hash, Some(hash));
}

#[test]
fn c4d_unchanged_raw_stays_ok() {
    let app = TestApp::new().unwrap();
    let path = app.dir.path().join("good.ARW");
    fs::write(&path, b"original").unwrap();
    let hash = sha256_file(&path).unwrap();
    let stored = photo(&app, &path, item_session(&app, "A"), Some(hash));
    assert_eq!(
        IntegrityCheck::new(app.store()).verify(stored.id).unwrap(),
        Integrity::Ok
    );
    assert_eq!(
        app.store().photo(stored.id).unwrap().integrity,
        Integrity::Ok
    );
}

#[test]
fn c5a_folder_key_replaces_non_ascii_and_spaces() {
    assert_eq!(folder_key("Ab c/é.1").unwrap(), "Ab_c__.1");
}

#[test]
fn c5b_folder_key_replaces_leading_dot_or_underscore() {
    assert_eq!(folder_key(".hidden").unwrap(), "-hidden");
    assert_eq!(folder_key("_x").unwrap(), "-x");
}

#[test]
fn c5c_folder_key_enforces_length() {
    assert!(matches!(folder_key(""), Err(Error::InvalidValue(_))));
    assert!(matches!(
        folder_key(&"a".repeat(101)),
        Err(Error::InvalidValue(_))
    ));
    assert_eq!(folder_key(&"a".repeat(100)).unwrap(), "a".repeat(100));
}

#[test]
fn c5d_create_refuses_case_insensitive_folder_collision() {
    let app = TestApp::new().unwrap();
    app.items().create_local("abc", "A", "Chair", None).unwrap();
    let collision = app.items().create_local("ABC", "B", "Chair", None);
    assert!(
        matches!(collision, Err(Error::SkuCollision { existing_sku }) if existing_sku == "abc")
    );
    let second = app
        .items()
        .create_local("abd", "C", "Chair", None)
        .unwrap()
        .0;
    assert_eq!(second.id, 2);
    let conn = Connection::open(app.app.paths.db()).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn c6a_create_local_creates_one_session_at_clock_time() {
    let app = TestApp::new().unwrap();
    app.clock.advance(1234);
    let (item, session) = app
        .items()
        .create_local("A", "A chair", "Chair", None)
        .unwrap();
    assert_eq!(item.origin, ItemOrigin::Local);
    assert_eq!(session.session.item_id, Some(item.id));
    assert_eq!(item.created_at, app.clock.now_ms());
    let conn = Connection::open(app.app.paths.db()).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE item_id=?1",
            [item.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn c6b_sku_change_freezes_key_after_first_photo_and_checks_stored_key() {
    let app = TestApp::new().unwrap();
    let (item, session) = app
        .items()
        .create_local("A", "A chair", "Chair", None)
        .unwrap();
    let first = app
        .items()
        .update_identity(item.id, Some("B"), None)
        .unwrap();
    assert_eq!(
        (&first.sku, &first.folder_key),
        (&"B".to_owned(), &"B".to_owned())
    );
    let path = app.dir.path().join("photo.ARW");
    fs::write(&path, b"raw").unwrap();
    photo(&app, &path, session.session.id, None);
    let second = app
        .items()
        .update_identity(item.id, Some("C"), None)
        .unwrap();
    assert_eq!(
        (&second.sku, &second.folder_key),
        (&"C".to_owned(), &"B".to_owned())
    );
    let other = app
        .items()
        .create_local("X", "Another", "Chair", None)
        .unwrap()
        .0;
    let result = app.items().update_identity(other.id, Some("b"), None);
    assert!(matches!(result, Err(Error::SkuCollision { existing_sku }) if existing_sku == "C"));
    assert_eq!(app.store().item(other.id).unwrap().sku, "X");
}

#[test]
fn c7a_item_and_session_timestamps_are_integer_milliseconds() {
    let app = TestApp::new().unwrap();
    app.clock.advance(1_761_400_000_123);
    let (item, session) = app
        .items()
        .create_local("A", "Chair", "Seating", None)
        .unwrap();
    assert_eq!(item.created_at, 1_761_400_000_123);
    assert_eq!(session.session.created_at, 1_761_400_000_123);
    assert_eq!(session.session.last_activity_at, 1_761_400_000_123);
}
