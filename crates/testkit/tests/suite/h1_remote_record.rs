//! H1·U2 behaviour tests (`plans/H1_points_and_remote_record.md`, rows H1·C8–C10, C12): the
//! record of remote pictures at the store, `RemoteItems` and the storage clean-up on
//! `TestApp::with_fake_shop()`. Outcomes: stored rows, the state, the reads and mutations the fake
//! shop recorded, emitted events. "Archived item" = published, then cleared by the real clearing
//! (`p62`'s helpers); "adopted item" = `ItemsService::adopt` of a fake shop product, then
//! `SessionsService::create`. H1·C7 and H1·C11 are inline unit tests of their rules.

use crate::p62_retention_after_publish::{Item, clear_all, item, product_of, published, shop_app};
use psm_core::{
    Error,
    domain::{
        publish::{RemoteMedia, RemoteMediaStatus},
        remote::{CallBudget, RemoteKind, RemoteState},
        storage::ArchivedImage,
    },
    events::AppEvent,
    ports::{clock::Clock, store::Store},
    services::{
        items::ItemsService, remote::Pending, retention::archived_file_name,
        storage::RETENTION_NOT_CLEARED,
    },
};
use psm_infra::{paths::AppPaths, sqlite::store::SqliteStore};
use psm_testkit::{TestApp, fake_commerce::fake_image_url};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

const DAY: i64 = 86_400_000;
const SHOP_ONLY: RemoteState = RemoteState::Remote(RemoteKind::ShopOnly);

fn image(role: &str, position: i64, media_id: &str, bytes: Option<i64>) -> ArchivedImage {
    ArchivedImage {
        role: role.into(),
        position,
        media_id: media_id.into(),
        url: fake_image_url(media_id),
        bytes,
    }
}

fn reads(a: &TestApp, op: &str) -> usize {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .reads
        .iter()
        .filter(|r| r.as_str() == op)
        .count()
}

fn readbacks(a: &TestApp) -> usize {
    reads(a, "media_readback")
}

fn session_changes(a: &TestApp, session: i64) -> usize {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter(|e| matches!(e, AppEvent::SessionChanged { session_id } if *session_id == session))
        .count()
}

fn budget(a: &TestApp, available: f64, maximum: f64) {
    a.fake_shop()
        .set_budget(Some(CallBudget { available, maximum }));
}

/// The fake shop's gallery media `ids` (ready) of `product`.
fn set_gallery(a: &TestApp, product: &str, ids: &[&str]) {
    let mut s = a.fake_shop().state.lock().unwrap();
    let p = s
        .products
        .iter_mut()
        .find(|p| p.external_id == product)
        .unwrap();
    p.media = ids
        .iter()
        .enumerate()
        .map(|(i, id)| RemoteMedia {
            id: (*id).into(),
            filename: Some(format!("{id}.jpg")),
            status: RemoteMediaStatus::Ready,
            position: i as i64 + 1,
        })
        .collect();
}

/// The fake shop's condition files `ids` of `product` (its condition-photos field value).
fn set_condition(a: &TestApp, product: &str, ids: &[&str]) {
    let mut s = a.fake_shop().state.lock().unwrap();
    for id in ids {
        if !s.files.iter().any(|f| f.id == *id) {
            s.files.push(RemoteMedia {
                id: (*id).into(),
                filename: Some(format!("{id}.jpg")),
                status: RemoteMediaStatus::Ready,
                position: 0,
            });
        }
    }
    let key = a.fake_shop().condition_key.clone();
    let values = s.values.entry(product.into()).or_default();
    values.retain(|(k, _)| *k != key);
    values.push((key, json!(ids)));
}

/// A shop product adopted from the search, with its session: (session id, item id, product id).
fn adopted(a: &TestApp, sku: &str, gallery: &[&str], condition: &[&str]) -> (i64, i64, String) {
    let product = a
        .fake_shop()
        .add_product(sku, &format!("{sku} title"), None);
    set_gallery(a, &product, gallery);
    set_condition(a, &product, condition);
    let item = ItemsService::new(a.store(), a.clock.as_ref())
        .with_commerce(Some(a.fake_shop()))
        .adopt(&product)
        .unwrap();
    let session = a.sessions().create(item.id).unwrap().session.id;
    (session, item.id, product)
}

/// The three gallery media and two condition files of H1·C9(a), with sizes.
fn adopted_with_pictures(a: &TestApp) -> (i64, i64) {
    let (session, item, _) = adopted(a, "SHOP1", &["m1", "m2", "m3"], &["c1", "c2"]);
    for (id, bytes) in [
        ("m1", 1001),
        ("m2", 1002),
        ("m3", 1003),
        ("c1", 2001),
        ("c2", 2002),
    ] {
        a.fake_shop().set_media_bytes(id, bytes);
    }
    (session, item)
}

fn expected_shop_record() -> Vec<ArchivedImage> {
    vec![
        image("gallery", 1, "m1", Some(1001)),
        image("gallery", 2, "m2", Some(1002)),
        image("gallery", 3, "m3", Some(1003)),
        image("condition", 1, "c1", Some(2001)),
        image("condition", 2, "c2", Some(2002)),
    ]
}

/// An item published (3 main, 2 condition) and then cleared by the real clearing.
fn archived(a: &TestApp, sku: &str) -> Item {
    let it = published(a, sku, 3, 2);
    a.clock.advance(4 * DAY + 60_000);
    clear_all(a);
    assert!(
        a.store()
            .item_retention(it.item)
            .unwrap()
            .cleared_at
            .is_some(),
        "{sku} cleared"
    );
    it
}

/// The fake shop's gallery media ids and condition file ids of `product`, in order.
fn shop_ids(a: &TestApp, product: &str) -> (Vec<String>, Vec<String>) {
    let s = a.fake_shop().state.lock().unwrap();
    let gallery = s
        .products
        .iter()
        .find(|p| p.external_id == product)
        .unwrap()
        .media
        .iter()
        .map(|m| m.id.clone())
        .collect();
    let key = &a.fake_shop().condition_key;
    let condition = s
        .values
        .get(product)
        .and_then(|values| values.iter().find(|(k, _)| k == key))
        .and_then(|(_, v)| v.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();
    (gallery, condition)
}

fn record_ids(a: &TestApp, item: i64) -> Vec<(String, i64, String)> {
    a.store()
        .archived_images(item)
        .unwrap()
        .into_iter()
        .map(|i| (i.role, i.position, i.media_id))
        .collect()
}

fn previews(a: &TestApp) -> PathBuf {
    a.app.paths.root.join("previews")
}

fn place(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"cached").unwrap();
}

// ---------------------------------------------------------------------------------------------
// H1·C8 the record at the store (MC-H3.2, M41)

#[test]
fn h1_c8a_record_round_trip() {
    let a = shop_app();
    let (_, session) = a
        .items()
        .create_local("REC1", "Record", "Sideboard", None)
        .unwrap();
    let item = session.session.item_id.unwrap();
    let mut gallery = image("gallery", 1, "m1", Some(1000));
    gallery.url = "u1".into();
    let mut condition = image("condition", 1, "c1", None);
    condition.url = "u2".into();
    // Handed condition first: the store answers gallery first.
    a.store()
        .set_remote_record(item, &[condition.clone(), gallery.clone()], 77)
        .unwrap();
    assert_eq!(
        a.store().archived_images(item).unwrap(),
        vec![gallery, condition]
    );
    let retention = a.store().item_retention(item).unwrap();
    assert_eq!(
        (
            retention.remote_checked_at,
            retention.cleared_at,
            retention.restored_at
        ),
        (Some(77), None, None)
    );
}

#[test]
fn h1_c8b_an_older_database_keeps_its_retention_record() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    paths.create().unwrap();
    {
        let mut conn = rusqlite::Connection::open(paths.db()).unwrap();
        rusqlite_migration::Migrations::new(
            SqliteStore::migrations().into_iter().take(19).collect(),
        )
        .to_version(&mut conn, 19)
        .unwrap();
        conn.execute_batch(
            "INSERT INTO items(id,sku,folder_key,folder_key_ci,item_type,title,origin,created_at,updated_at) VALUES (7,'K','K','k','Sideboard','Oak','local',1,1);
             INSERT INTO item_retention(item_id,keep_files,cleared_at) VALUES (7,1,5);
             INSERT INTO archived_images(item_id,role,position,media_id,url) VALUES (7,'gallery',1,'m1','https://cdn.fake-shop.test/files/m1.jpg');
             INSERT INTO archived_images(item_id,role,position,media_id,url) VALUES (7,'condition',1,'c1','https://cdn.fake-shop.test/files/c1.jpg');",
        )
        .unwrap();
    }
    let store = SqliteStore::open(&paths).unwrap();
    let retention = store.item_retention(7).unwrap();
    assert_eq!(
        (
            retention.keep_files,
            retention.cleared_at,
            retention.restored_at,
            retention.remote_checked_at
        ),
        (true, Some(5), None, None)
    );
    assert_eq!(
        store.archived_images(7).unwrap(),
        vec![
            image("gallery", 1, "m1", None),
            image("condition", 1, "c1", None)
        ]
    );
}

// ---------------------------------------------------------------------------------------------
// H1·C9 the first-open read (MC-H3.2, MC-H5.3)

#[test]
fn h1_c9a_first_open_records_the_shop_pictures() {
    let a = shop_app();
    let (session, item) = adopted_with_pictures(&a);
    a.events.0.lock().unwrap().clear();
    let before = readbacks(&a);
    let state = a.remote().ensure_record(session).unwrap();
    assert_eq!(state, SHOP_ONLY);
    assert_eq!(
        a.store().archived_images(item).unwrap(),
        expected_shop_record()
    );
    assert_eq!(
        a.store().item_retention(item).unwrap().remote_checked_at,
        Some(a.clock.now_ms())
    );
    assert_eq!(a.remote().state(session).unwrap(), SHOP_ONLY);
    assert_eq!(readbacks(&a) - before, 1);
    assert_eq!(session_changes(&a, session), 1);
}

#[test]
fn h1_c9b_a_product_without_pictures_is_local() {
    let a = shop_app();
    let (session, item, _) = adopted(&a, "SHOP2", &[], &[]);
    let before = readbacks(&a);
    a.remote().ensure_record(session).unwrap();
    assert_eq!(a.store().archived_images(item).unwrap(), vec![]);
    assert_eq!(
        a.store().item_retention(item).unwrap().remote_checked_at,
        Some(a.clock.now_ms())
    );
    assert_eq!(a.remote().state(session).unwrap(), RemoteState::Local);
    assert_eq!(readbacks(&a) - before, 1);
}

#[test]
fn h1_c9c_later_opens_use_the_record() {
    let a = shop_app();
    let (session, _) = adopted_with_pictures(&a);
    let before = readbacks(&a);
    a.remote().ensure_record(session).unwrap();
    a.remote().ensure_record(session).unwrap();
    assert_eq!(readbacks(&a) - before, 1);
}

#[test]
fn h1_c9d_two_callers_make_one_read() {
    let a = shop_app();
    let (session, item) = adopted_with_pictures(&a);
    let before = readbacks(&a);
    a.fake_shop().set_hook(Some(Box::new(|op: &str| {
        if op == "media_readback" {
            std::thread::sleep(Duration::from_millis(100));
        }
    })));
    std::thread::scope(|scope| {
        let first = scope.spawn(|| a.remote().ensure_record(session));
        let second = scope.spawn(|| a.remote().ensure_record(session));
        first.join().unwrap().unwrap();
        second.join().unwrap().unwrap();
    });
    a.fake_shop().set_hook(None);
    assert_eq!(readbacks(&a) - before, 1);
    assert_eq!(
        a.store().archived_images(item).unwrap(),
        expected_shop_record()
    );
}

#[test]
fn h1_c9e_an_unreachable_shop_writes_nothing() {
    let a = shop_app();
    let (session, item) = adopted_with_pictures(&a);
    a.fake_shop().state.lock().unwrap().unreachable = true;
    let error = a.remote().ensure_record(session).unwrap_err();
    a.fake_shop().state.lock().unwrap().unreachable = false;
    assert!(matches!(error, Error::Provider(_)), "{error:?}");
    assert_eq!(a.store().archived_images(item).unwrap(), vec![]);
    assert_eq!(
        a.store().item_retention(item).unwrap().remote_checked_at,
        None
    );
    assert_eq!(a.remote().state(session).unwrap(), RemoteState::Unknown);
}

#[test]
fn h1_c9f_a_picture_still_processing_writes_nothing() {
    let a = shop_app();
    let (session, item, product) = adopted(&a, "SHOP3", &["m1", "m2", "m3"], &[]);
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .products
        .iter_mut()
        .find(|p| p.external_id == product)
        .unwrap()
        .media[1]
        .status = RemoteMediaStatus::Processing;
    match a.remote().ensure_record(session).unwrap_err() {
        Error::Provider(text) => assert!(text.contains("still processing"), "{text}"),
        other => panic!("{other:?}"),
    }
    assert_eq!(a.store().archived_images(item).unwrap(), vec![]);
    assert_eq!(
        a.store().item_retention(item).unwrap().remote_checked_at,
        None
    );
    assert_eq!(a.remote().state(session).unwrap(), RemoteState::Unknown);
}

#[test]
fn h1_c9g_the_queued_read_waits_for_the_budget() {
    let a = shop_app();
    let (session, _) = adopted_with_pictures(&a);
    let before = readbacks(&a);
    a.remote().queue_record(session).unwrap();
    budget(&a, 100.0, 1000.0);
    assert_eq!(a.remote().run_pending().unwrap(), Pending::Waiting);
    assert_eq!(readbacks(&a) - before, 0);
    budget(&a, 900.0, 1000.0);
    assert_eq!(a.remote().run_pending().unwrap(), Pending::Worked);
    assert_eq!(readbacks(&a) - before, 1);
    assert_eq!(a.remote().state(session).unwrap(), SHOP_ONLY);
}

// ---------------------------------------------------------------------------------------------
// H1·C10 Refresh from the shop (MC-H3.3, MC-H5.4, M-H1)

/// H1·C10(a)'s change in the shop: the gallery reordered to `[m2, m1, m3]`, a condition file
/// `c3` added. Returns the expected record ids.
fn change_the_shop(a: &TestApp, product: &str) -> Vec<(String, i64, String)> {
    let (gallery, mut condition) = shop_ids(a, product);
    assert_eq!((gallery.len(), condition.len()), (3, 2));
    let reordered = [&gallery[1], &gallery[0], &gallery[2]];
    set_gallery(
        a,
        product,
        &reordered.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
    );
    condition.push("c3".into());
    set_condition(
        a,
        product,
        &condition.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    reordered
        .iter()
        .enumerate()
        .map(|(i, id)| ("gallery".to_owned(), i as i64 + 1, (*id).clone()))
        .chain(
            condition
                .iter()
                .enumerate()
                .map(|(i, id)| ("condition".to_owned(), i as i64 + 1, id.clone())),
        )
        .collect()
}

/// H1·C10(a); with `low_budget`, the budget is `{100, 1000}` from after the setup on (H1·C10(b)).
fn refresh_after_change(a: &TestApp, low_budget: bool) {
    let it = archived(a, "ARCH");
    let product = product_of(a, &it);
    let expected = change_the_shop(a, &product);
    if low_budget {
        budget(a, 100.0, 1000.0);
    }
    let cleared_at = a.store().item_retention(it.item).unwrap().cleared_at;
    let before = readbacks(a);
    a.remote().refresh(it.session).unwrap();
    assert_eq!(record_ids(a, it.item), expected);
    assert_eq!(
        a.store().item_retention(it.item).unwrap().cleared_at,
        cleared_at
    );
    assert_eq!(readbacks(a) - before, 1);
}

#[test]
fn h1_c10a_refresh_replaces_the_record() {
    let a = shop_app();
    refresh_after_change(&a, false);
}

#[test]
fn h1_c10b_a_press_never_waits_for_the_budget() {
    let a = shop_app();
    refresh_after_change(&a, true);
}

#[test]
fn h1_c10c_a_local_item_is_not_remote() {
    let a = shop_app();
    let it = item(&a, "LOC", 1, 0);
    let before = readbacks(&a);
    let error = a.remote().refresh(it.session).unwrap_err();
    assert!(matches!(error, Error::NotRemote), "{error:?}");
    assert_eq!(readbacks(&a) - before, 0);
}

#[test]
fn h1_c10d_refresh_deletes_stale_cache_files() {
    let a = shop_app();
    let it = archived(&a, "PRUNE");
    let item = it.item.to_string();
    let cover = previews(&a)
        .join("remote")
        .join(&item)
        .join("cover-000000000000.jpg");
    let stale = previews(&a)
        .join("archived")
        .join(&item)
        .join("gallery-01-000000000000.jpg");
    let first = a.store().archived_images(it.item).unwrap()[0].clone();
    let current = previews(&a)
        .join("archived")
        .join(&item)
        .join(archived_file_name(&first));
    for file in [&cover, &stale, &current] {
        place(file);
    }
    a.remote().refresh(it.session).unwrap();
    assert_eq!(
        (cover.exists(), stale.exists(), current.is_file()),
        (false, false, true)
    );
}

#[test]
fn h1_c10e_reading_and_refreshing_make_no_shop_change() {
    let a = shop_app();
    let it = archived(&a, "READONLY");
    let (shop_session, _) = adopted_with_pictures(&a);
    let after_setup = a.fake_shop().mutation_ops();
    a.remote().ensure_record(shop_session).unwrap();
    a.remote().refresh(it.session).unwrap();
    budget(&a, 100.0, 1000.0);
    a.remote().refresh(it.session).unwrap();
    a.remote().refresh(shop_session).unwrap();
    assert_eq!(a.fake_shop().mutation_ops(), after_setup);
}

// ---------------------------------------------------------------------------------------------
// H1·C12 the clearing keeps sizes and waits for the budget (MC-H4.1, MC-H5.3, 10F)

fn not_cleared_health(a: &TestApp) -> Option<String> {
    a.app
        .health
        .report("", false, false)
        .items
        .into_iter()
        .find(|i| i.code == RETENTION_NOT_CLEARED && !i.ok)
        .map(|i| i.detail)
}

#[test]
fn h1_c12a_the_clearing_records_picture_sizes() {
    let a = shop_app();
    let it = published(&a, "SIZED", 2, 1);
    let (gallery, condition) = shop_ids(&a, &product_of(&a, &it));
    let mut sizes = Vec::new();
    for (n, id) in gallery.iter().chain(condition.iter()).enumerate() {
        let bytes = 1_000_000 + n as i64;
        a.fake_shop().set_media_bytes(id, bytes);
        sizes.push(Some(bytes));
    }
    a.clock.advance(4 * DAY + 60_000);
    clear_all(&a);
    assert!(
        a.store()
            .item_retention(it.item)
            .unwrap()
            .cleared_at
            .is_some()
    );
    let recorded: Vec<Option<i64>> = a
        .store()
        .archived_images(it.item)
        .unwrap()
        .into_iter()
        .map(|i| i.bytes)
        .collect();
    assert_eq!(recorded, sizes);
}

#[test]
fn h1_c12bc_the_clearing_waits_while_the_budget_is_low() {
    let a = shop_app();
    let first = published(&a, "WAIT1", 1, 0);
    let second = published(&a, "WAIT2", 1, 0);
    a.clock.advance(4 * DAY + 60_000);
    budget(&a, 100.0, 1000.0);
    let before = readbacks(&a);
    // H1·C12(b)
    clear_all(&a);
    let cleared = |it: &Item| {
        a.store()
            .item_retention(it.item)
            .unwrap()
            .cleared_at
            .is_some()
    };
    assert_eq!((cleared(&first), cleared(&second)), (false, false));
    assert_eq!(readbacks(&a) - before, 0);
    assert_eq!(not_cleared_health(&a), None);
    // H1·C12(c)
    budget(&a, 900.0, 1000.0);
    clear_all(&a);
    assert_eq!((cleared(&first), cleared(&second)), (true, true));
}
