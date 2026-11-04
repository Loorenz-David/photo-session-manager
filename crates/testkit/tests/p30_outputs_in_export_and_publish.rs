//! P30 outputs in export and publishing (10D MC-29, M35): two files per split photo, upload names
//! with `-L`/`-R`, links per output slot with `composition_rev`, replacement and retirement, the
//! gallery and damage report per output, refusal while `needs_review`, masters after publish.

use psm_core::{
    Error,
    domain::{
        attribute::{AttributeGroup, CoreType, Definition},
        composition::{
            CompositionKind, CompositionState, CompositionValues, EditorGeometry, Flip,
            Orientation, OutputSlot, Rect, Shape,
        },
        photo::{Classification as C, NewPhoto, Photo, PhotoSource},
        publish::{DefinitionLink, PhotoLink},
        session::PublishStep,
    },
    mechanisms::{
        composition::{default_rect, turned_frame},
        hash::sha256_file,
    },
    ports::{clock::Clock, commerce::CommerceAdapter, images::uid12},
    services::{attributes::AttributesService, sessions::SessionsService},
};
use psm_infra::{paths::AppPaths, sqlite::store::SqliteStore};
use psm_testkit::{TestApp, fixtures::synthetic_jpeg::SyntheticJpeg};
use serde_json::{Value as Json, json};
use std::{fs, path::Path};

const NOW_MS: i64 = 1_762_250_400_000; // 2025-11-04T10:00:00Z

fn shop_app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(NOW_MS);
    a
}
fn document_app() -> TestApp {
    let a = TestApp::with_fake_document().unwrap();
    a.clock.advance(NOW_MS);
    a
}
fn session(a: &TestApp) -> i64 {
    a.items()
        .create_local("K", "Oak", "Sideboard", None)
        .unwrap()
        .1
        .session
        .id
}
fn item_id(a: &TestApp, s: i64) -> i64 {
    a.store().session(s).unwrap().session.item_id.unwrap()
}
fn variant(photo: &Photo) -> String {
    format!("v{}", photo.id)
}
fn last_issued(a: &TestApp) -> String {
    a.fake
        .state
        .lock()
        .unwrap()
        .issued
        .last()
        .unwrap()
        .0
        .clone()
}

/// A RAW with its real hash, known to the fake document, with a 2560×1707 first preview (the
/// composition's frame), classified.
fn photo(a: &TestApp, s: i64, name: &str, class: C) -> Photo {
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(format!("{name}.ARW"));
    fs::write(&raw, format!("raw bytes of {name}")).unwrap();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id: s,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(NOW_MS),
            source: PhotoSource::Capture,
            created_at: NOW_MS,
        })
        .unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&variant(&photo)], false);
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.app.previews.request_initial(&photo.raw_path).unwrap();
    a.complete_render_with(
        &last_issued(a),
        &SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]),
    )
    .unwrap();
    if class != C::Unreviewed {
        a.review().classify(photo.id, class).unwrap();
    }
    a.store().photo(photo.id).unwrap()
}

fn values(kind: CompositionKind, shape: Shape, rect: Rect) -> CompositionValues {
    CompositionValues {
        kind,
        shape,
        orientation: Orientation::Named,
        quarter_turns: 0,
        flip: Flip::None,
        straighten: 0.0,
        rect,
    }
}
/// The photo's current frame (the preview's, or the master's once one was rendered).
fn frame(a: &TestApp, p: &Photo) -> (f64, f64) {
    let f = a.composition(p.id).unwrap().frame().unwrap();
    (f.w as f64, f.h as f64)
}
/// `split(a, id)`: a 2:1 split, the default rect of a split in the photo's frame.
fn split(a: &TestApp, p: &Photo) {
    let f = a.composition(p.id).unwrap().frame().unwrap();
    let rect = default_rect(
        CompositionKind::Split,
        Shape::Square,
        Orientation::Named,
        turned_frame(f, 0),
    );
    a.app
        .compositions
        .set(p.id, values(CompositionKind::Split, Shape::Square, rect))
        .unwrap();
}
/// A different valid split: 90 % of the width, 2:1.
fn other_split(a: &TestApp, p: &Photo) {
    let (w, h) = frame(a, p);
    let height = 0.9 * w / (2.0 * h);
    a.app
        .compositions
        .set(
            p.id,
            values(
                CompositionKind::Split,
                Shape::Square,
                Rect {
                    x: 0.05,
                    y: (1.0 - height) / 2.0,
                    w: 0.9,
                    h: height,
                },
            ),
        )
        .unwrap();
}
/// A different valid square (half the width, top-left).
fn other_square(a: &TestApp, p: &Photo) {
    let (w, h) = frame(a, p);
    a.app
        .compositions
        .set(
            p.id,
            values(
                CompositionKind::Normal,
                Shape::Square,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 0.5,
                    h: 0.5 * w / h,
                },
            ),
        )
        .unwrap();
}
/// Back to a normal photo with the default square.
fn default_square(a: &TestApp, p: &Photo) {
    let f = a.composition(p.id).unwrap().frame().unwrap();
    let rect = default_rect(
        CompositionKind::Normal,
        Shape::Square,
        Orientation::Named,
        turned_frame(f, 0),
    );
    a.app
        .compositions
        .set(p.id, values(CompositionKind::Normal, Shape::Square, rect))
        .unwrap();
}

fn condition(a: &TestApp) {
    let def = Definition {
        integration: "shopify".into(),
        key: "custom.damage_report".into(),
        label: "Damage report".into(),
        core_type: CoreType::MediaList,
        choices: Vec::new(),
        validations: json!({}),
        group: AttributeGroup::Condition,
        description: None,
        writable: true,
    };
    a.fake_shop().state.lock().unwrap().definitions = vec![(
        def,
        DefinitionLink {
            platform_type: "list.file_reference".into(),
            platform_meta: json!({}),
        },
    )];
    AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
        .refresh_schema(a.fake_shop())
        .unwrap();
}

fn drain(a: &TestApp) {
    for _ in 0..12 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
}
fn confirm(a: &TestApp, s: i64) -> psm_core::Result<i64> {
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "Publish images")
}
fn publish(a: &TestApp, s: i64) {
    confirm(a, s).unwrap();
    drain(a);
}
fn latest(a: &TestApp, s: i64) -> psm_core::domain::publish::PublishItem {
    let id: i64 = rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT id FROM publish_items WHERE session_id=?1 ORDER BY id DESC LIMIT 1",
            [s],
            |r| r.get(0),
        )
        .unwrap();
    a.store().publish_item(id).unwrap()
}
fn product(a: &TestApp) -> String {
    a.fake_shop().state.lock().unwrap().products[0]
        .external_id
        .clone()
}
fn media(a: &TestApp) -> Vec<(String, String)> {
    a.fake_shop()
        .item_get(&product(a))
        .unwrap()
        .media
        .into_iter()
        .map(|m| (m.id, m.filename.unwrap_or_default()))
        .collect()
}
fn media_names(a: &TestApp) -> Vec<String> {
    media(a).into_iter().map(|(_, name)| name).collect()
}
fn file_names(a: &TestApp) -> Vec<String> {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .files
        .iter()
        .map(|f| f.filename.clone().unwrap_or_default())
        .collect()
}
fn file_id(a: &TestApp, name: &str) -> String {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .files
        .iter()
        .find(|f| f.filename.as_deref() == Some(name))
        .unwrap()
        .id
        .clone()
}
fn damage_report(a: &TestApp) -> Option<Json> {
    let product = product(a);
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .values
        .get(&product)
        .and_then(|rows| {
            rows.iter()
                .find(|(k, _)| k == "custom.damage_report")
                .map(|(_, v)| v.clone())
        })
}
fn name(p: &Photo, suffix: &str) -> String {
    format!("K-{}{suffix}.jpg", uid12(&p.uid))
}
fn link(a: &TestApp, s: i64, p: &Photo, role: &str, slot: OutputSlot) -> Option<PhotoLink> {
    a.store()
        .photo_link(p.id, item_id(a, s), role, slot)
        .unwrap()
}
fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}
fn sorted(list: &[&str]) -> Vec<String> {
    let mut list: Vec<String> = list.iter().map(|s| s.to_string()).collect();
    list.sort();
    list
}
fn json_file(path: &Path) -> Json {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}
fn mutations(a: &TestApp) -> Vec<String> {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .mutations
        .iter()
        .map(|m| m.op.clone())
        .collect()
}
fn render_output_open(a: &TestApp) -> i64 {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE kind='render_output' AND status IN ('queued','running')",
            [],
            |r| r.get(0),
        )
        .unwrap()
}

/// A Capture One geometry change arriving with the next preview (P28 C5(a) shape), delivered
/// without running the publish worker.
fn flag_needs_review(a: &TestApp, p: &Photo) {
    a.fake.set_variant_geometry(
        &variant(p),
        EditorGeometry {
            rotation: 3.0,
            ..Default::default()
        },
    );
    a.clock.advance(31_000);
    a.deliver_hook(
        &json!({"hook":"variant_adjusted","variant_id":variant(p),"raw":p.raw_path}).to_string(),
    )
    .unwrap();
    a.clock.advance(20_000);
    a.app.arrival.tick().unwrap();
    a.app.jobs.tick(a.clock.now_ms()).unwrap();
    a.app.previews.tick().unwrap();
    a.complete_render_with(
        &last_issued(a),
        &SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]),
    )
    .unwrap();
    assert_eq!(
        a.composition(p.id).unwrap().state,
        CompositionState::NeedsReview
    );
}

// C1 — export numbering per output

/// `K` main `[a, b, c]`, condition `[d]`, `a` and `d` split, export started.
fn split_export(a: &TestApp) -> (Photo, Photo, Photo, Photo) {
    let s = session(a);
    let pa = photo(a, s, "a", C::Main);
    let pb = photo(a, s, "b", C::Main);
    let pc = photo(a, s, "c", C::Main);
    let pd = photo(a, s, "d", C::Condition);
    split(a, &pa);
    split(a, &pd);
    a.app.export.run(s).unwrap();
    (pa, pb, pc, pd)
}
fn finish_export(a: &TestApp) {
    for _ in 0..10 {
        a.complete_all_renders().unwrap();
        a.tick(0).unwrap();
        if render_output_open(a) == 0 && a.complete_all_renders().unwrap().is_empty() {
            return;
        }
    }
    panic!("export did not finish");
}

#[test]
fn c1a_split_main_and_condition_photos_export_one_file_per_output() {
    let a = document_app();
    let (pa, pb, _, pd) = split_export(&a);
    finish_export(&a);
    let dir = a.export_root().join("K");
    assert_eq!(
        names(&dir),
        sorted(&[
            "K_01_main.jpg",
            "K_02_main.jpg",
            "K_03_main.jpg",
            "K_04_main.jpg",
            "K_01_condition.jpg",
            "K_02_condition.jpg",
            "item.json",
            "manifest.json",
        ])
    );
    let item = json_file(&dir.join("item.json"));
    assert_eq!(
        item["main"],
        json!([
            "K_01_main.jpg",
            "K_02_main.jpg",
            "K_03_main.jpg",
            "K_04_main.jpg"
        ])
    );
    assert_eq!(
        item["condition"],
        json!(["K_01_condition.jpg", "K_02_condition.jpg"])
    );
    let outputs = &json_file(&dir.join("manifest.json"))["outputs"];
    let entry = |n: &str| {
        (
            outputs[n]["photo_id"].as_i64().unwrap(),
            outputs[n]["slot"].as_str().unwrap().to_string(),
        )
    };
    assert_eq!(entry("K_01_main.jpg"), (pa.id, "left".into()));
    assert_eq!(entry("K_02_main.jpg"), (pa.id, "right".into()));
    assert_eq!(entry("K_03_main.jpg"), (pb.id, "single".into()));
    assert_eq!(entry("K_01_condition.jpg"), (pd.id, "left".into()));
    assert_eq!(entry("K_02_condition.jpg"), (pd.id, "right".into()));
}

#[test]
fn c1b_a_photo_deleted_mid_run_renumbers_the_outputs() {
    let a = document_app();
    let (pa, pb, pc, _) = split_export(&a);
    a.complete_all_renders().unwrap();
    a.review().delete(pb.id).unwrap();
    finish_export(&a);
    let dir = a.export_root().join("K");
    let main: Vec<String> = names(&dir)
        .into_iter()
        .filter(|n| n.ends_with("_main.jpg"))
        .collect();
    assert_eq!(
        main,
        sorted(&["K_01_main.jpg", "K_02_main.jpg", "K_03_main.jpg"])
    );
    assert_eq!(
        json_file(&dir.join("item.json"))["main"],
        json!(["K_01_main.jpg", "K_02_main.jpg", "K_03_main.jpg"])
    );
    let outputs = &json_file(&dir.join("manifest.json"))["outputs"];
    let entry = |n: &str| {
        (
            outputs[n]["photo_id"].as_i64().unwrap(),
            outputs[n]["slot"].as_str().unwrap().to_string(),
        )
    };
    assert_eq!(entry("K_01_main.jpg"), (pa.id, "left".into()));
    assert_eq!(entry("K_02_main.jpg"), (pa.id, "right".into()));
    assert_eq!(entry("K_03_main.jpg"), (pc.id, "single".into()));
}

// C2 — upload filenames

/// `K` main `[a, b]`, `a` split, published.
fn split_published() -> (TestApp, i64, Photo, Photo) {
    let a = shop_app();
    let s = session(&a);
    let pa = photo(&a, s, "a", C::Main);
    let pb = photo(&a, s, "b", C::Main);
    split(&a, &pa);
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    (a, s, pa, pb)
}

#[test]
fn c2a_split_halves_upload_with_l_and_r() {
    let (a, _, pa, pb) = split_published();
    let expected = vec![name(&pa, "-L"), name(&pa, "-R"), name(&pb, "")];
    assert_eq!(a.fake_shop().staged_filenames(), expected);
    assert_eq!(media_names(&a), expected);
}

#[test]
fn c2b_a_split_revision_uploads_both_halves_with_r1() {
    let (a, s, pa, pb) = split_published();
    other_split(&a, &pa);
    assert_eq!(a.composition(pa.id).unwrap().revision, 2);
    let before = a.fake_shop().staged_filenames().len();
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert_eq!(
        a.fake_shop().staged_filenames()[before..].to_vec(),
        vec![name(&pa, "-L-r1"), name(&pa, "-R-r1")]
    );
    assert_eq!(
        media_names(&a),
        vec![name(&pa, "-L-r1"), name(&pa, "-R-r1"), name(&pb, "")]
    );
}

#[test]
fn c2c_a_split_condition_photo_creates_two_files() {
    let a = shop_app();
    let s = session(&a);
    let pd = photo(&a, s, "d", C::Condition);
    split(&a, &pd);
    condition(&a);
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    let (left, right) = (name(&pd, "-L"), name(&pd, "-R"));
    assert_eq!(file_names(&a), vec![left.clone(), right.clone()]);
    assert_eq!(
        damage_report(&a),
        Some(json!([file_id(&a, &left), file_id(&a, &right)]))
    );
}

// C3 — links per slot, migration 005, replacement on a composition revision

#[test]
fn c3a_links_are_per_slot_and_record_the_composition_revision() {
    let (a, s, pa, pb) = split_published();
    let rows: Vec<(i64, String, OutputSlot, String, i64, i64)> = a
        .store()
        .photo_links_for_item(item_id(&a, s))
        .unwrap()
        .into_iter()
        .map(|l| {
            (
                l.photo_id,
                l.role,
                l.slot,
                l.upload_state,
                l.composition_rev,
                l.revision,
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (
                pa.id,
                "gallery".into(),
                OutputSlot::Left,
                "ready".into(),
                1,
                0
            ),
            (
                pa.id,
                "gallery".into(),
                OutputSlot::Right,
                "ready".into(),
                1,
                0
            ),
            (
                pb.id,
                "gallery".into(),
                OutputSlot::Single,
                "ready".into(),
                0,
                0
            ),
        ]
    );
    let left = link(&a, s, &pa, "gallery", OutputSlot::Left).unwrap();
    assert_eq!(
        (left.photo_id, left.slot, left.composition_rev),
        (pa.id, OutputSlot::Left, 1)
    );
    assert_eq!(link(&a, s, &pa, "gallery", OutputSlot::Single), None);
}

#[test]
fn c3b_migration_005_keeps_existing_links_as_single() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    paths.create().unwrap();
    {
        let mut conn = rusqlite::Connection::open(paths.db()).unwrap();
        rusqlite_migration::Migrations::new(
            SqliteStore::migrations().into_iter().take(4).collect(),
        )
        .to_version(&mut conn, 4)
        .unwrap();
        conn.execute_batch(
            "INSERT INTO items(id,sku,folder_key,folder_key_ci,item_type,title,origin,created_at,updated_at) VALUES (7,'K','K','k','Sideboard','Oak','local',1,1);
             INSERT INTO sessions(id,item_id,last_activity_at,created_at) VALUES (8,7,1,1);
             INSERT INTO photos(id,uid,session_id,raw_path,source,created_at) VALUES (9,'u-9',8,'/raws/a.ARW','capture',1);
             INSERT INTO photo_links(id,photo_id,integration,item_id,role,external_id,revision,retouch_gen,upload_state,updated_at) VALUES (42,9,'shopify',7,'gallery','m1',2,1,'ready',1);",
        )
        .unwrap();
    }
    let store = SqliteStore::open(&paths).unwrap();
    let link =
        psm_core::ports::store::Store::photo_link(&store, 9, 7, "gallery", OutputSlot::Single)
            .unwrap()
            .unwrap();
    assert_eq!(
        (
            link.slot,
            link.composition_rev,
            link.external_id.as_deref(),
            link.upload_state.as_str(),
            link.revision,
            link.retouch_gen
        ),
        (OutputSlot::Single, 0, Some("m1"), "ready", 2, 1)
    );
    drop(store);
    let conn = rusqlite::Connection::open(paths.db()).unwrap();
    let id: i64 = conn
        .query_row("SELECT id FROM photo_links WHERE photo_id=9", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(id, 42);
    let indexes: Vec<(String, i64)> = conn
        .prepare("PRAGMA index_list(photo_links)")
        .unwrap()
        .query_map([], |r| Ok((r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let unique: Vec<Vec<String>> = indexes
        .into_iter()
        .filter(|(_, unique)| *unique == 1)
        .map(|(index, _)| {
            conn.prepare(&format!("PRAGMA index_info({index})"))
                .unwrap()
                .query_map([], |r| r.get(2))
                .unwrap()
                .map(Result::unwrap)
                .collect()
        })
        .collect();
    assert!(
        unique.contains(&vec![
            "photo_id".to_string(),
            "integration".into(),
            "item_id".into(),
            "role".into(),
            "slot".into()
        ]),
        "{unique:?}"
    );
}

/// `K` main `[a, b]`, both normal, published.
fn normal_published() -> (TestApp, i64, Photo, Photo) {
    let a = shop_app();
    let s = session(&a);
    let pa = photo(&a, s, "a", C::Main);
    let pb = photo(&a, s, "b", C::Main);
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    (a, s, pa, pb)
}

#[test]
fn c3c_a_composition_revision_replaces_the_media_in_place() {
    let (a, s, pa, pb) = normal_published();
    let old = link(&a, s, &pa, "gallery", OutputSlot::Single)
        .unwrap()
        .external_id
        .unwrap();
    let staged_before = a.fake_shop().staged_filenames().len();
    other_square(&a, &pa);
    assert_eq!(a.composition(pa.id).unwrap().revision, 1);
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    let la = link(&a, s, &pa, "gallery", OutputSlot::Single).unwrap();
    assert_eq!(
        (
            la.revision,
            la.composition_rev,
            la.superseded_ids.clone(),
            la.upload_state.as_str()
        ),
        (1, 1, vec![old], "ready")
    );
    let uid = uid12(&pa.uid);
    let of_a: Vec<String> = media_names(&a)
        .into_iter()
        .filter(|n| n.contains(&uid))
        .collect();
    assert_eq!(of_a, vec![name(&pa, "-r1")]);
    assert_eq!(media_names(&a)[0], name(&pa, "-r1"));
    assert_eq!(
        media(&a)[0].0,
        la.external_id.unwrap(),
        "the new media takes a's place"
    );
    let lb = link(&a, s, &pb, "gallery", OutputSlot::Single).unwrap();
    assert_eq!(lb.revision, 0);
    assert_eq!(
        a.fake_shop().staged_filenames()[staged_before..].to_vec(),
        vec![name(&pa, "-r1")]
    );
}

#[test]
fn c3d_an_identical_save_uploads_nothing() {
    let a = shop_app();
    let s = session(&a);
    let pa = photo(&a, s, "a", C::Main);
    // A user composition (revision 1) is published first: P28 counts a default → user origin
    // change as a change, so only a user composition can be re-saved identically.
    other_square(&a, &pa);
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    let staged = a.fake_shop().staged_filenames();
    let current = a.composition(pa.id).unwrap().values().unwrap();
    let saved = a.app.compositions.set(pa.id, current).unwrap();
    assert_eq!(saved.revision, 1);
    assert!(!a.store().session(s).unwrap().dirty);
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert_eq!(a.fake_shop().staged_filenames(), staged);
    let la = link(&a, s, &pa, "gallery", OutputSlot::Single).unwrap();
    assert_eq!((la.revision, la.composition_rev), (0, 1));
}

// C4 — retirement on Normal ↔ Split

#[test]
fn c4a_normal_to_split_retires_the_single_link() {
    let (a, s, pa, pb) = normal_published();
    let single = link(&a, s, &pa, "gallery", OutputSlot::Single)
        .unwrap()
        .external_id
        .unwrap();
    split(&a, &pa);
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    let retired = link(&a, s, &pa, "gallery", OutputSlot::Single).unwrap();
    assert_eq!(
        (
            retired.upload_state.as_str(),
            retired.external_id.as_deref()
        ),
        ("retired", Some(single.as_str()))
    );
    assert!(!media(&a).iter().any(|(id, _)| *id == single));
    for slot in [OutputSlot::Left, OutputSlot::Right] {
        assert_eq!(
            link(&a, s, &pa, "gallery", slot).unwrap().upload_state,
            "ready"
        );
    }
    assert_eq!(
        media_names(&a),
        vec![name(&pa, "-L"), name(&pa, "-R"), name(&pb, "")]
    );
}

/// C4(a) then back to normal (C4(b)'s state).
fn split_and_back() -> (TestApp, i64, Photo, Photo, Vec<String>) {
    let (a, s, pa, pb) = normal_published();
    let single = link(&a, s, &pa, "gallery", OutputSlot::Single)
        .unwrap()
        .external_id
        .unwrap();
    split(&a, &pa);
    publish(&a, s);
    let halves: Vec<String> = [OutputSlot::Left, OutputSlot::Right]
        .into_iter()
        .map(|slot| {
            link(&a, s, &pa, "gallery", slot)
                .unwrap()
                .external_id
                .unwrap()
        })
        .collect();
    default_square(&a, &pa);
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    let mut retired = vec![single];
    retired.extend(halves);
    (a, s, pa, pb, retired)
}

#[test]
fn c4b_split_to_normal_retires_the_halves() {
    let (a, s, pa, pb, retired) = split_and_back();
    for slot in [OutputSlot::Left, OutputSlot::Right] {
        assert_eq!(
            link(&a, s, &pa, "gallery", slot).unwrap().upload_state,
            "retired"
        );
    }
    for id in &retired[1..] {
        assert!(!media(&a).iter().any(|(m, _)| m == id));
    }
    let single = link(&a, s, &pa, "gallery", OutputSlot::Single).unwrap();
    assert_eq!(
        (single.upload_state.as_str(), single.revision),
        ("ready", 0)
    );
    assert!(!media(&a).iter().any(|(m, _)| *m == retired[0]));
    assert_eq!(media_names(&a), vec![name(&pa, ""), name(&pb, "")]);
    assert_eq!(
        a.fake_shop().staged_filenames().last(),
        Some(&name(&pa, ""))
    );
    assert_eq!(
        media(&a)[0].0,
        single.external_id.unwrap(),
        "a fresh upload, not the first single media"
    );
}

#[test]
fn c4c_retired_rows_stay_as_own_evidence() {
    let (a, s, pa, _, retired) = split_and_back();
    let rows: Vec<(OutputSlot, String, Option<String>)> = a
        .store()
        .photo_links_for_item(item_id(&a, s))
        .unwrap()
        .into_iter()
        .filter(|l| l.photo_id == pa.id && l.slot != OutputSlot::Single)
        .map(|l| (l.slot, l.upload_state, l.external_id))
        .collect();
    assert_eq!(
        rows,
        vec![
            (OutputSlot::Left, "retired".into(), Some(retired[1].clone())),
            (
                OutputSlot::Right,
                "retired".into(),
                Some(retired[2].clone())
            ),
        ]
    );
    let single = link(&a, s, &pa, "gallery", OutputSlot::Single).unwrap();
    assert!(single.superseded_ids.contains(&retired[0]));
    let view = SessionsService::new(a.store(), a.clock.as_ref())
        .with_commerce(a.commerce.as_deref().map(|c| c as &dyn CommerceAdapter))
        .view(s)
        .unwrap();
    let index = view.photos.iter().position(|p| p.id == pa.id).unwrap();
    assert_eq!(view.upload_states[index], Some("ready".to_string()));
}

// C5 — gallery and cover

#[test]
fn c5a_gallery_is_the_main_outputs_and_the_cover_is_the_left_half() {
    let (a, _, pa, pb) = split_published();
    assert_eq!(
        media_names(&a),
        vec![name(&pa, "-L"), name(&pa, "-R"), name(&pb, "")]
    );
    let cover = a.fake_shop().item_get(&product(&a)).unwrap().media[0].clone();
    assert_eq!((cover.position, cover.filename), (1, Some(name(&pa, "-L"))));
}

#[test]
fn c5b_a_reorder_keeps_the_halves_adjacent_left_first() {
    let (a, s, pa, pb) = split_published();
    a.review().reorder(pb.id, 1).unwrap();
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert_eq!(
        media_names(&a),
        vec![name(&pb, ""), name(&pa, "-L"), name(&pa, "-R")]
    );
}

// C6 — damage report

#[test]
fn c6a_damage_report_lists_condition_outputs_in_order() {
    let a = shop_app();
    let s = session(&a);
    let pm = photo(&a, s, "m", C::Main);
    let pd = photo(&a, s, "d", C::Condition);
    let pe = photo(&a, s, "e", C::Condition);
    split(&a, &pd);
    condition(&a);
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    let ids = vec![
        file_id(&a, &name(&pd, "-L")),
        file_id(&a, &name(&pd, "-R")),
        file_id(&a, &name(&pe, "")),
    ];
    assert_eq!(damage_report(&a), Some(json!(ids)));
    let gallery: Vec<String> = media(&a).into_iter().map(|(id, _)| id).collect();
    assert!(ids.iter().all(|id| !gallery.contains(id)));
    assert_eq!(media_names(&a), vec![name(&pm, "")]);
}

// C7 — needs_review refusals

fn publish_runs(a: &TestApp) -> i64 {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row("SELECT COUNT(*) FROM publish_runs", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn c7a_confirmation_refuses_while_a_photo_needs_review() {
    let a = shop_app();
    let s = session(&a);
    photo(&a, s, "a", C::Main);
    let pb = photo(&a, s, "b", C::Main);
    flag_needs_review(&a, &pb);
    let runs = publish_runs(&a);
    let result = confirm(&a, s);
    assert!(
        matches!(&result, Err(Error::NeedsReview { photos }) if photos == &vec!["Main 02".to_string()]),
        "{result:?}"
    );
    assert_eq!(publish_runs(&a), runs);
    assert!(mutations(&a).is_empty());
}

#[test]
fn c7b_needs_review_after_confirmation_fails_prepare_then_retry_publishes() {
    let a = shop_app();
    let s = session(&a);
    let pa = photo(&a, s, "a", C::Main);
    let pb = photo(&a, s, "b", C::Main);
    confirm(&a, s).unwrap();
    flag_needs_review(&a, &pb);
    drain(&a);
    let item = latest(&a, s);
    assert_eq!(
        (item.step, item.failed_step),
        (PublishStep::Failed, Some(PublishStep::Prepare))
    );
    assert!(
        item.error
            .as_deref()
            .is_some_and(|e| e.contains("Re-frame first: Main 02")),
        "{:?}",
        item.error
    );
    assert!(mutations(&a).is_empty());
    other_square(&a, &pb);
    assert_eq!(a.composition(pb.id).unwrap().state, CompositionState::Ok);
    a.app.publish.as_ref().unwrap().retry(item.id).unwrap();
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert_eq!(media_names(&a), vec![name(&pa, ""), name(&pb, "")]);
}

// C8 — stale staging, masters after publish

#[test]
fn c8a_a_stale_staging_from_a_failed_attempt_is_rendered_again() {
    let a = shop_app();
    let s = session(&a);
    let pa = photo(&a, s, "a", C::Main);
    a.fake_shop().fail_before_effect("product_create_media");
    publish(&a, s);
    let failed = latest(&a, s);
    assert_eq!(failed.failed_step, Some(PublishStep::Upload));
    let old_run = failed.export_run_id.unwrap();
    other_square(&a, &pa);
    assert_eq!(a.composition(pa.id).unwrap().revision, 1);
    a.app.publish.as_ref().unwrap().retry(failed.id).unwrap();
    drain(&a);
    let done = latest(&a, s);
    assert_eq!(done.step, PublishStep::Done);
    let g = a.store().adjusted_gen(&pa.raw_path).unwrap();
    let output = a.outputs_dir(&pa.raw_path).join(format!(
        "{}-single-g{g}-r1-jpeg-2048-q90.jpg",
        uid12(&pa.uid)
    ));
    let expected = sha256_file(&output).unwrap();
    assert_eq!(
        a.fake_shop().uploaded_sha(&name(&pa, "")),
        Some(expected.clone())
    );
    assert_eq!(
        link(&a, s, &pa, "gallery", OutputSlot::Single)
            .unwrap()
            .export_sha256,
        Some(expected)
    );
    assert!(
        !a.export_root()
            .join(format!(".K.partial-{old_run}"))
            .exists()
    );
    assert_ne!(done.export_run_id, Some(old_run));
    let manifest = json_file(&a.export_root().join("K/manifest.json"));
    assert_eq!(
        manifest["outputs"]["K_01_main.jpg"]["composition_rev"],
        json!(1)
    );
}

#[test]
fn c8b_masters_are_released_after_a_publish_of_a_clean_session() {
    let a = shop_app();
    let s = session(&a);
    let pa = photo(&a, s, "a", C::Main);
    publish(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert!(!a.store().session(s).unwrap().dirty);
    assert!(!a.masters_dir(&pa.raw_path).exists());
    assert!(a.outputs_dir(&pa.raw_path).is_dir());
}

#[test]
fn c8c_a_session_dirty_at_done_keeps_its_masters() {
    let a = shop_app();
    let s = session(&a);
    let pa = photo(&a, s, "a", C::Main);
    let pb = photo(&a, s, "b", C::Main);
    confirm(&a, s).unwrap();
    let commerce = a.commerce.as_ref().unwrap().clone();
    commerce.hold_media_poll();
    // prepare runs on the publish worker until upload holds at the first media poll; the
    // photo order changes then (a photo change during the run).
    let mut reached = false;
    for _ in 0..8 {
        a.clock.advance(1100);
        a.app.jobs.tick(a.clock.now_ms()).unwrap();
        a.app.render_jobs.tick(a.clock.now_ms()).unwrap();
        a.complete_all_renders().unwrap();
        a.app.render_jobs.tick(a.clock.now_ms()).unwrap();
        let jobs = a.app.publish_jobs.clone();
        let now = a.clock.now_ms();
        let worker = std::thread::spawn(move || jobs.tick(now).unwrap());
        while !worker.is_finished() && !commerce.media_poll_entered() {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        if commerce.media_poll_entered() {
            assert_eq!(latest(&a, s).step, PublishStep::Upload);
            a.review().reorder(pb.id, 1).unwrap();
            commerce.release_media_poll();
            worker.join().unwrap();
            reached = true;
            break;
        }
        worker.join().unwrap();
    }
    commerce.release_media_poll();
    assert!(reached, "publish worker did not reach the media poll");
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert!(a.store().session(s).unwrap().dirty);
    let g = a.store().adjusted_gen(&pa.raw_path).unwrap();
    assert!(
        a.masters_dir(&pa.raw_path)
            .join(format!("master-{g}.tif"))
            .is_file()
    );
}
