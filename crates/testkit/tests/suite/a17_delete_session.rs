//! Owner correction A17 (2026-11-18): "Delete session", for test items (test-4-4: its Shopify
//! product was deleted, so retention can never clear it) and mistakes. Session Manager only: the
//! files go to the Trash, the rows go in one transaction, Shopify is never asked anything.
use psm_core::{
    domain::photo::{NewPhoto, PhotoSource},
    mechanisms::{arrival::path_key, canon::canon},
    ports::{clock::Clock, store::Store},
};
use psm_integrations::session_delete;
use psm_testkit::TestApp;
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};

fn session(a: &TestApp, sku: &str) -> (i64, i64, String) {
    let (item, session) = a.items().create_local(sku, sku, "object", None).unwrap();
    (session.session.id, item.id, item.folder_key)
}

/// A RAW in the session's capture folder, its photo row and its raw state.
fn photo(a: &TestApp, session_id: i64, key: &str, name: &str) -> (i64, String) {
    let path = a.capture_root().join(key).join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, name.as_bytes()).unwrap();
    let raw = canon(&path).unwrap();
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
    (row.id, raw)
}

/// Capture One's settings file of the RAW and the app's preview, master and output folders.
fn side_files(a: &TestApp, raw: &str) -> Vec<PathBuf> {
    let raw_path = Path::new(raw);
    let name = raw_path.file_name().unwrap().to_string_lossy();
    let cos = raw_path
        .parent()
        .unwrap()
        .join("CaptureOne/Settings153")
        .join(format!("{name}.cos"));
    let key = path_key(raw);
    let preview = a.app.paths.root.join("previews").join(&key[..2]).join(&key);
    let mut files = vec![cos];
    for dir in [preview, a.masters_dir(raw), a.outputs_dir(raw)] {
        files.push(dir.join("file.jpg"));
    }
    for file in &files {
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, b"x").unwrap();
    }
    files
}

fn db(a: &TestApp) -> Connection {
    Connection::open(a.app.paths.db()).unwrap()
}

fn count(conn: &Connection, sql: &str, id: i64) -> i64 {
    conn.query_row(sql, [id], |r| r.get(0)).unwrap()
}

/// Every row of every table that still names the deleted session, its item, photos, RAWs,
/// publish items or research runs (the schema check: a new table that names one of them and is
/// not cleared by the delete shows up here).
struct Deleted {
    session: i64,
    item: i64,
    photos: Vec<i64>,
    raws: Vec<String>,
    publish_items: Vec<i64>,
    runs: Vec<i64>,
}
fn dangling(conn: &Connection, d: &Deleted) -> Vec<String> {
    let tables: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            )
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    let ints = |v: &[i64]| serde_json::to_string(v).unwrap();
    let mut out = Vec::new();
    for table in tables {
        let columns: Vec<String> = {
            let mut stmt = conn
                .prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))
                .unwrap();
            stmt.query_map([], |r| r.get(0))
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap()
        };
        for column in columns {
            let list = match (table.as_str(), column.as_str()) {
                ("sessions", "id") | (_, "session_id") => ints(&[d.session]),
                ("items", "id") | (_, "item_id") => ints(&[d.item]),
                ("photos", "id") | (_, "photo_id") => ints(&d.photos),
                (_, "raw_path") => serde_json::to_string(&d.raws).unwrap(),
                ("publish_items", "id") | (_, "publish_item_id") => ints(&d.publish_items),
                ("research_runs", "id") => ints(&d.runs),
                (t, c) if t.starts_with("research_") && c.ends_with("run_id") => ints(&d.runs),
                _ => continue,
            };
            let n: i64 = conn
                .query_row(
                    &format!(
                        "SELECT COUNT(*) FROM {table} WHERE {column} IN (SELECT value FROM json_each(?1))"
                    ),
                    [&list],
                    |r| r.get(0),
                )
                .unwrap();
            if n > 0 {
                out.push(format!("{table}.{column}: {n}"));
            }
        }
    }
    let targets: Vec<String> = d
        .raws
        .iter()
        .cloned()
        .chain([format!("session:{}", d.session)])
        .collect();
    let jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE target IN (SELECT value FROM json_each(?1))",
            [serde_json::to_string(&targets).unwrap()],
            |r| r.get(0),
        )
        .unwrap();
    if jobs > 0 {
        out.push(format!("jobs.target: {jobs}"));
    }
    let settings: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM settings WHERE (key='active_session_id' AND value=?1) OR key LIKE 'publish_export:%'",
            [d.session.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    if settings > 0 {
        out.push(format!("settings: {settings}"));
    }
    out
}

/// Every table that names a session, its item or its photos is cleared, the files go to the
/// Trash (the RAWs, Capture One's files, the app's caches, then the emptied folder), and the
/// rows of another session stay untouched.
#[test]
fn a17_delete_removes_every_row_and_sends_the_files_to_the_trash() {
    let a = TestApp::with_research().unwrap();
    let (keep, keep_item, keep_key) = session(&a, "KEEP-1");
    let (gone, item, key) = session(&a, "test-4-4");
    let (kept_photo, kept_raw) = photo(&a, keep, &keep_key, "K.ARW");
    let (p1, r1) = photo(&a, gone, &key, "DSC05166.ARW");
    let (p2, r2) = photo(&a, gone, &key, "DSC05167.ARW");
    let mut files = side_files(&a, &r1);
    files.extend(side_files(&a, &r2));
    let kept_files = side_files(&a, &kept_raw);

    // Rows in every table that can name the session, its item or its photos.
    let conn = db(&a);
    let now = a.clock.now_ms();
    a.insert_publish_item(gone, "done", now, None).unwrap();
    let publish_item: i64 = conn
        .query_row(
            "SELECT id FROM publish_items WHERE session_id=?1",
            [gone],
            |r| r.get(0),
        )
        .unwrap();
    for sql in [
        "INSERT INTO publish_item_stock(publish_item_id,location_id,location_name,quantity) VALUES (?1,'l','L',1)",
        "INSERT INTO settings(key,value) VALUES ('publish_export:' || ?1, '7')",
    ] {
        conn.execute(sql, [publish_item]).unwrap();
    }
    for photo_id in [p1, p2, kept_photo] {
        conn.execute(
            "INSERT INTO photo_places(photo_id,slot) VALUES (?1,1)",
            [photo_id],
        )
        .unwrap();
    }
    conn.execute(
        "INSERT INTO photo_denials(photo_id,denied_at) VALUES (?1,?2)",
        params![p2, now],
    )
    .unwrap();
    conn.execute("INSERT INTO photo_links(photo_id,integration,item_id,role,updated_at) VALUES (?1,'shopify',?2,'gallery',?3)", params![p1, item, now]).unwrap();
    conn.execute("INSERT INTO attribute_values(item_id,key,value,provenance,updated_at) VALUES (?1,'title','\"x\"','human',?2)", params![item, now]).unwrap();
    conn.execute("INSERT INTO attribute_values(item_id,key,value,provenance,updated_at) VALUES (?1,'title','\"y\"','human',?2)", params![keep_item, now]).unwrap();
    for sql in [
        "INSERT INTO item_links(item_id,integration,external_id) VALUES (?1,'shopify','gid://x')",
        "INSERT INTO item_stock(item_id,integration,location_id,location_name,quantity) VALUES (?1,'shopify','l','L',1)",
        "INSERT INTO item_remote_titles(item_id,integration,title) VALUES (?1,'shopify','t')",
        "INSERT INTO item_retention(item_id,keep_files) VALUES (?1,1)",
        "INSERT INTO archived_images(item_id,role,position,media_id,url) VALUES (?1,'gallery',1,'m','https://x')",
        "INSERT INTO saved_pages(item_id,url,title,saved_at) VALUES (?1,'https://example.com/a','A',1)",
        "INSERT INTO research_feedback(item_id,key,suggested_value,confidence,action,at) VALUES (?1,'k','v','high','accepted',1)",
    ] {
        conn.execute(sql, [item]).unwrap();
    }
    for sql in [
        "INSERT INTO notifications(session_id,kind,text,created_at) VALUES (?1,'k','t',1)",
        "INSERT INTO session_changes(session_id,seq,kind,at) VALUES (?1,1,'new_photo',1)",
        "INSERT INTO camera_exposures(exposed_at,initiated,session_id,status,created_at) VALUES (1,'camera',?1,'received',1)",
        "INSERT INTO jobs(kind,target,payload,status,created_at,updated_at) VALUES ('render_export','session:' || ?1,'{}','done',1,1)",
    ] {
        conn.execute(sql, [gone]).unwrap();
    }
    conn.execute("INSERT INTO jobs(kind,target,payload,status,created_at,updated_at) VALUES ('render_preview',?1,'{}','queued',1,1)", [&r1]).unwrap();
    conn.execute("INSERT INTO jobs(kind,target,payload,status,created_at,updated_at) VALUES ('mirror_tag',?1,?2,'done',1,1)", params![r2, format!("{{\"photo_id\":{p2}}}")]).unwrap();
    conn.execute("INSERT INTO jobs(kind,target,payload,status,created_at,updated_at) VALUES ('render_preview',?1,'{}','queued',1,1)", [&kept_raw]).unwrap();
    conn.execute(
        "INSERT INTO research_runs(item_id,session_id,trigger,state,queued_at) VALUES (?1,?2,'manual','done',1)",
        params![item, gone],
    )
    .unwrap();
    let run = conn.last_insert_rowid();
    for sql in [
        "INSERT INTO research_steps(run_id,seq,stage,kind,summary,photo_ids,field_keys,created_at) VALUES (?1,1,'read','read','s','[]','[]',1)",
        "INSERT INTO research_calls(run_id,stage,unit_key,attempt,provider,model,effort,state,request,started_at) VALUES (?1,'read','u',1,'p','m','e','done','{}',1)",
        "INSERT INTO research_outbound(run_id,call_id,photo_id,provider,sent_at,sha256) VALUES (?1,1,1,'p',1,'s')",
        "INSERT INTO research_stage_outputs(run_id,stage,unit_key,fingerprint,output,committed_at) VALUES (?1,'read','u','f','{}',1)",
        "INSERT INTO jobs(kind,target,payload,status,created_at,updated_at) VALUES ('research',CAST(?1 AS TEXT),'{}','done',1,1)",
    ] {
        conn.execute(sql, [run]).unwrap();
    }

    let deleted = session_delete::delete(&a.app.handles(), gone).unwrap();
    assert_eq!(deleted.sku, "test-4-4");
    assert_eq!(deleted.photos, 2);
    assert_eq!(deleted.research_runs, 1);
    assert!(
        deleted.folder_trashed,
        "the emptied capture folder goes too"
    );

    let leftover = dangling(
        &conn,
        &Deleted {
            session: gone,
            item,
            photos: vec![p1, p2],
            raws: vec![r1.clone(), r2.clone()],
            publish_items: vec![publish_item],
            runs: vec![run],
        },
    );
    assert!(
        leftover.is_empty(),
        "rows still name the deleted session: {leftover:?}"
    );
    // The capture ledger keeps its history without a session.
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM camera_exposures WHERE session_id IS NULL AND exposed_at=?1",
            1
        ),
        1
    );

    // Files: in the Trash, nothing left on disk; the other session's are untouched.
    let moved = a.trash.moved_paths();
    for path in [PathBuf::from(&r1), PathBuf::from(&r2)] {
        assert!(
            moved.contains(&path),
            "{} not in the Trash: {moved:?}",
            path.display()
        );
        assert!(!path.exists());
    }
    for file in &files {
        assert!(!file.exists(), "{} left on disk", file.display());
    }
    assert!(!a.capture_root().join(&key).exists());
    assert!(moved.contains(&a.capture_root().join(&key)));
    assert!(Path::new(&kept_raw).exists());
    for file in &kept_files {
        assert!(
            file.exists(),
            "{} of another session was touched",
            file.display()
        );
    }
    assert!(a.store().session(keep).is_ok());
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM attribute_values WHERE item_id=?1",
            keep_item
        ),
        1
    );
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM jobs WHERE target=(SELECT raw_path FROM photos WHERE id=?1)",
            kept_photo
        ),
        1
    );
}

/// test-4-4: a published session whose files retention already cleared (the RAWs are gone) is
/// deleted all the same, and Shopify is not asked anything (Session Manager only).
#[test]
fn a17_published_and_cleared_session_is_deleted_without_asking_shopify() {
    let a = TestApp::with_fake_shop().unwrap();
    let (gone, item, key) = session(&a, "test-4-4");
    let (_, r1) = photo(&a, gone, &key, "DSC05166.ARW");
    a.mark_published(gone).unwrap();
    a.store()
        .mark_item_cleared(item, a.clock.now_ms(), &[])
        .unwrap();
    std::fs::remove_file(&r1).unwrap();
    let reads = a.fake_shop().state.lock().unwrap().reads.len();
    let mutations = a.fake_shop().mutation_ops().len();

    let preview = session_delete::preview(&a.app.handles(), gone).unwrap();
    assert!(preview.published && preview.archived);
    assert_eq!(preview.photos, 1);
    let deleted = session_delete::delete(&a.app.handles(), gone).unwrap();
    assert_eq!(deleted.photos, 1);
    assert!(deleted.files_missing >= 1, "{deleted:?}");
    assert!(a.store().session(gone).is_err());
    assert_eq!(a.fake_shop().state.lock().unwrap().reads.len(), reads);
    assert_eq!(a.fake_shop().mutation_ops().len(), mutations);
}

/// Refused: the Unassigned session; while the session is being published; while research runs
/// for it. Nothing is moved or removed then.
#[test]
fn a17_delete_is_refused_for_unassigned_a_running_publish_and_running_research() {
    let a = TestApp::with_research().unwrap();
    let refused = |id: i64| match session_delete::delete(&a.app.handles(), id) {
        Err(psm_core::Error::DeleteRefused(reason)) => reason,
        other => panic!("not refused: {other:?}"),
    };
    assert!(refused(1).contains("Unassigned"));

    let (publishing, _, key) = session(&a, "PUB-1");
    let (_, raw) = photo(&a, publishing, &key, "P.ARW");
    a.insert_publish_item(publishing, "upload", a.clock.now_ms(), None)
        .unwrap();
    assert!(refused(publishing).contains("published"));
    assert!(Path::new(&raw).exists());
    assert!(a.store().session(publishing).is_ok());

    let (researching, item, _) = session(&a, "RES-1");
    db(&a)
        .execute(
            "INSERT INTO research_runs(item_id,session_id,trigger,state,queued_at) VALUES (?1,?2,'manual','running',1)",
            params![item, researching],
        )
        .unwrap();
    assert!(refused(researching).contains("Research"));
    assert!(a.store().session(researching).is_ok());
    assert!(a.trash.moved_paths().is_empty());
}

/// The session photos are filed into: the filing target leaves it before the first file moves,
/// so no new photo can be filed into a folder that is going away.
#[test]
fn a17_the_filing_target_leaves_the_session_before_any_file_moves() {
    let a = TestApp::with_camera().unwrap();
    let (gone, _, key) = session(&a, "ACT-1");
    let (_, raw) = photo(&a, gone, &key, "A.ARW");
    a.store().set_active_session_id(Some(gone)).unwrap();
    let store = a.app.store.clone();
    a.trash
        .observe(move |_| format!("{:?}", store.active_session_id().unwrap()));

    // The storage service on its own (the command also asks Capture One to leave first).
    a.app.storage.delete_session(gone, "test").unwrap();
    let moved = a.trash.moved.lock().unwrap().clone();
    assert!(moved.iter().any(|(p, _)| p == Path::new(&raw)));
    assert!(
        moved.iter().all(|(_, active)| active == "None"),
        "a file moved while photos were still filed into the session: {moved:?}"
    );
    assert_eq!(a.store().active_session_id().unwrap(), None);
}

/// Through the command's path, Capture One is asked to leave the session (back to Unassigned)
/// before the delete; when it is not reachable the delete still happens.
#[test]
fn a17_capture_one_leaves_the_active_session_first() {
    let a = TestApp::with_camera().unwrap();
    let (gone, _, key) = session(&a, "ACT-2");
    photo(&a, gone, &key, "B.ARW");
    a.sessions().activate(gone).unwrap();
    a.fake.state.lock().unwrap().ops.clear();

    session_delete::delete(&a.app.handles(), gone).unwrap();
    let ops = a.fake.state.lock().unwrap().ops.clone();
    assert!(
        ops.iter().any(|op| op.contains("_Unassigned")),
        "Capture One was not moved to Unassigned: {ops:?}"
    );
    assert_eq!(a.store().active_session_id().unwrap(), None);
    assert!(a.store().session(gone).is_err());
}

/// A file that cannot be moved to the Trash keeps the session (rows and the rest): the owner is
/// told, and what did move is in the Trash.
#[test]
fn a17_a_file_that_cannot_be_moved_keeps_the_session() {
    let a = TestApp::with_fake_document().unwrap();
    let (gone, _, key) = session(&a, "LOCK-1");
    let (_, r1) = photo(&a, gone, &key, "L1.ARW");
    photo(&a, gone, &key, "L2.ARW");
    *a.trash.refuse.lock().unwrap() = Some("L1.ARW".into());
    match session_delete::delete(&a.app.handles(), gone) {
        Err(psm_core::Error::DeleteRefused(reason)) => {
            assert!(
                reason.contains("could not be moved to the Trash"),
                "{reason}"
            )
        }
        other => panic!("not refused: {other:?}"),
    }
    assert!(a.store().session(gone).is_ok());
    assert!(Path::new(&r1).exists());
    assert!(a.capture_root().join(&key).exists());
}

/// A RAW another session still uses (a copy) stays with that session; the deleted session's
/// undo history goes.
#[test]
fn a17_a_raw_another_session_uses_stays() {
    let a = TestApp::with_fake_document().unwrap();
    let (gone, _, key) = session(&a, "COPY-A");
    let (other, _, _) = session(&a, "COPY-B");
    let (_, raw) = photo(&a, gone, &key, "C.ARW");
    a.store()
        .photo_insert_or_get(NewPhoto {
            session_id: other,
            raw_path: raw.clone(),
            raw_hash: Some("hash-C.ARW".into()),
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    let deleted = session_delete::delete(&a.app.handles(), gone).unwrap();
    assert!(Path::new(&raw).exists(), "the copy's RAW was moved");
    assert!(
        !deleted.folder_trashed,
        "the folder still holds the copy's RAW"
    );
    assert_eq!(a.store().photos_for_raw(&raw).unwrap().len(), 1);
    assert_eq!(
        db(&a)
            .query_row(
                "SELECT COUNT(*) FROM raw_state WHERE raw_path=?1",
                [&raw],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(a.app.history.state(gone).undo, None);
}
