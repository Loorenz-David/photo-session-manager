//! H2·U1 behaviour tests (`plans/H2_list_pictures_pages_search.md`, rows H2·C1–C5): the row
//! picture rule at `SessionsService::list` with `RemoteItems`, and the cover queue at
//! `RemoteItems::{request_covers, run_pending}` on `TestApp::with_fake_shop()`. Outcomes: the
//! row's `cover_preview_path` / `cover_pending`, files on disk, the fetches / URL reads /
//! mutations the fake shop recorded, emitted events, Health.
//!
//! "Archived item" = published, then cleared by the real clearing (`p62`'s helpers, as in H1).
//! Where a row needs several archived items (H2·C3, C4) they are made the way `a17` makes one: a
//! local item linked to a fake shop product, marked cleared with its record of remote pictures
//! (the same stored state the clearing leaves, without a publish per item).

use crate::p62_retention_after_publish::{Item, clear_all, item, product_of, published, shop_app};
use psm_core::{
    domain::{
        publish::{RemoteMedia, RemoteMediaStatus},
        remote::{CallBudget, PictureTarget},
        session::ListFilter,
        storage::ArchivedImage,
    },
    events::AppEvent,
    ports::clock::Clock,
    services::{
        items::ItemsService,
        remote::{CoverSource, Pending},
    },
};
use psm_integrations::session_delete;
use psm_testkit::{
    TestApp,
    fake_commerce::{ImageFault, fake_image_bytes, fake_image_url, fake_refreshed_url},
};
use sha2::Digest;
use std::{
    fs,
    path::{Path, PathBuf},
};

const DAY: i64 = 86_400_000;
const MINUTE: i64 = 60_000;

// ---------------------------------------------------------------------------------------------
// helpers

fn previews(a: &TestApp) -> PathBuf {
    a.app.paths.root.join("previews")
}

/// `cover-` + the first 12 hex of the SHA-256 of `media_id` + `.jpg`, computed here.
fn expected_cover_name(media_id: &str) -> String {
    let hex: String = sha2::Sha256::digest(media_id.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("cover-{}.jpg", &hex[..12])
}

fn cover_file(a: &TestApp, item: i64, media_id: &str) -> PathBuf {
    previews(a)
        .join("remote")
        .join(item.to_string())
        .join(expected_cover_name(media_id))
}

fn place(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"cached").unwrap();
}

/// The list row of `session`: (its picture path, `cover_pending`).
fn row(a: &TestApp, session: i64) -> (Option<String>, bool) {
    let list = a
        .sessions()
        .with_remote(a.remote())
        .list(ListFilter::All, None)
        .unwrap();
    let row = list
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == session)
        .expect("row listed");
    (row.cover_preview_path, row.cover_pending)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// An item published and then cleared by the real clearing (as H1's tests).
fn archived(a: &TestApp, sku: &str) -> Item {
    let it = published(a, sku, 3, 2);
    a.clock.advance(4 * DAY + MINUTE);
    clear_all(a);
    assert!(
        a.store()
            .item_retention(it.item)
            .unwrap()
            .cleared_at
            .is_some()
    );
    it
}

/// The archived item's cover picture (its record's first gallery picture).
fn cover_of(a: &TestApp, item: i64) -> ArchivedImage {
    a.store()
        .archived_images(item)
        .unwrap()
        .into_iter()
        .find(|i| i.role == "gallery")
        .unwrap()
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

fn gallery_ids(a: &TestApp, product: &str) -> Vec<String> {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .products
        .iter()
        .find(|p| p.external_id == product)
        .unwrap()
        .media
        .iter()
        .map(|m| m.id.clone())
        .collect()
}

/// A shelved item as `a17` makes one: a local item linked to a fake shop product holding the
/// gallery media `<sku>-m1`, `<sku>-m2`, marked cleared with that record. (session, item, cover
/// media id).
fn shelved(a: &TestApp, sku: &str) -> (i64, i64, String) {
    let (item, session) = a
        .items()
        .create_local(sku, &format!("{sku} title"), "Sideboard", None)
        .unwrap();
    let product = a
        .fake_shop()
        .add_product(sku, &format!("{sku} title"), None);
    let ids = [format!("{sku}-m1"), format!("{sku}-m2")];
    set_gallery(a, &product, &[&ids[0], &ids[1]]);
    a.store()
        .transaction(&mut |tx| tx.insert_item_link(item.id, "shopify", &product, None))
        .unwrap();
    let images: Vec<ArchivedImage> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| ArchivedImage {
            role: "gallery".into(),
            position: i as i64 + 1,
            media_id: id.clone(),
            url: fake_image_url(id),
            bytes: None,
        })
        .collect();
    a.store()
        .mark_item_cleared(item.id, a.clock.now_ms(), &images)
        .unwrap();
    (session.session.id, item.id, ids[0].clone())
}

fn fetches(a: &TestApp) -> Vec<(String, u32)> {
    a.fake_shop().image_fetches()
}

fn fetched_urls(a: &TestApp) -> Vec<String> {
    let mut urls: Vec<String> = fetches(a).into_iter().map(|(url, _)| url).collect();
    urls.sort();
    urls
}

fn cover_events(a: &TestApp) -> Vec<(i64, String)> {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::PictureReady {
                target: PictureTarget::Cover { session_id },
                path,
            } => Some((*session_id, path.clone())),
            _ => None,
        })
        .collect()
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

fn request_and_run(a: &TestApp, sessions: &[i64]) {
    a.remote()
        .request_covers(CoverSource::Page, sessions)
        .unwrap();
    a.remote().run_pending().unwrap();
}

fn budget(a: &TestApp, available: f64, maximum: f64) {
    a.fake_shop()
        .set_budget(Some(CallBudget { available, maximum }));
}

/// Sets main photo 1's two stored preview paths of `session` (the files are the test's).
fn set_previews(a: &TestApp, session: i64, adjusted: &Path, original: &Path) {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .execute(
            "UPDATE photos SET adjusted_preview_path=?1, original_preview_path=?2 WHERE session_id=?3 AND classification='main' AND position=1",
            rusqlite::params![path_string(adjusted), path_string(original), session],
        )
        .unwrap();
}

// ---------------------------------------------------------------------------------------------
// H2·C1 the row picture rule (MC-H1.1, M-H4)

/// A local item with one main photo whose stored previews are two files of the test.
fn local_with_previews(a: &TestApp) -> (Item, PathBuf, PathBuf) {
    let it = item(a, "LOCAL", 1, 0);
    let dir = a.dir.path().join("test-previews");
    let adjusted = dir.join("adjusted.jpg");
    let original = dir.join("original.jpg");
    place(&adjusted);
    place(&original);
    set_previews(a, it.session, &adjusted, &original);
    (it, adjusted, original)
}

#[test]
fn h2_c1a_the_adjusted_preview_when_its_file_exists() {
    let a = shop_app();
    let (it, adjusted, _) = local_with_previews(&a);
    assert_eq!(row(&a, it.session), (Some(path_string(&adjusted)), false));
}

#[test]
fn h2_c1b_the_original_preview_when_the_adjusted_file_is_gone() {
    let a = shop_app();
    let (it, adjusted, original) = local_with_previews(&a);
    fs::remove_file(&adjusted).unwrap();
    assert_eq!(row(&a, it.session), (Some(path_string(&original)), false));
}

#[test]
fn h2_c1c_a_local_item_without_preview_files_has_no_picture() {
    let a = shop_app();
    let (it, adjusted, original) = local_with_previews(&a);
    fs::remove_file(&adjusted).unwrap();
    fs::remove_file(&original).unwrap();
    assert_eq!(row(&a, it.session), (None, false));
}

#[test]
fn h2_c1d_an_archived_item_shows_its_cover_file() {
    let a = shop_app();
    let it = archived(&a, "ARCH");
    let stored: Vec<Option<String>> = a
        .store()
        .photos_in_session(it.session)
        .unwrap()
        .into_iter()
        .map(|p| p.adjusted_preview_path.or(p.original_preview_path))
        .collect();
    assert!(stored.iter().any(Option::is_some), "the rows name previews");
    let cover = cover_file(&a, it.item, &cover_of(&a, it.item).media_id);
    place(&cover);
    assert_eq!(row(&a, it.session), (Some(path_string(&cover)), false));
}

#[test]
fn h2_c1e_an_archived_item_without_its_cover_file_is_pending() {
    let a = shop_app();
    let it = archived(&a, "ARCH");
    assert_eq!(row(&a, it.session), (None, true));
}

#[test]
fn h2_c1f_adopted_items_read_and_never_read() {
    let a = shop_app();
    let adopt = |sku: &str| {
        let product = a
            .fake_shop()
            .add_product(sku, &format!("{sku} title"), None);
        let id = format!("{sku}-m1");
        set_gallery(&a, &product, &[&id]);
        let item = ItemsService::new(a.store(), a.clock.as_ref())
            .with_commerce(Some(a.fake_shop()))
            .adopt(&product)
            .unwrap();
        (
            a.sessions().create(item.id).unwrap().session.id,
            item.id,
            id,
        )
    };
    let (read, read_item, media) = adopt("SHOP1");
    let (never_read, _, _) = adopt("SHOP2");
    a.remote().ensure_record(read).unwrap();
    let cover = cover_file(&a, read_item, &media);
    place(&cover);
    assert_eq!(
        (row(&a, read), row(&a, never_read)),
        ((Some(path_string(&cover)), false), (None, false))
    );
}

#[test]
fn h2_c1g_a_record_with_only_condition_pictures_has_no_cover() {
    let a = shop_app();
    let it = archived(&a, "ARCH");
    let condition: Vec<ArchivedImage> = a
        .store()
        .archived_images(it.item)
        .unwrap()
        .into_iter()
        .filter(|i| i.role == "condition")
        .collect();
    a.store()
        .set_remote_record(it.item, &condition, a.clock.now_ms())
        .unwrap();
    assert_eq!(row(&a, it.session), (None, false));
}

// ---------------------------------------------------------------------------------------------
// H2·C2 the cover file (MC-H1.2, MC-H1.3)

/// H2·C2(a): an archived item's cover requested and downloaded. (item, cover media id).
fn cover_downloaded(a: &TestApp) -> (Item, String) {
    let it = archived(a, "ARCH");
    let media = cover_of(a, it.item).media_id;
    request_and_run(a, &[it.session]);
    (it, media)
}

fn cover_files(a: &TestApp, item: i64) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(previews(a).join("remote").join(item.to_string()))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn h2_c2a_the_cover_is_written_from_the_shop_picture() {
    let a = shop_app();
    let (it, media) = cover_downloaded(&a);
    let url = fake_image_url(&media);
    let file = cover_file(&a, it.item, &media);
    assert_eq!(fs::read(&file).unwrap(), fake_image_bytes(&url));
    assert!(fetches(&a).contains(&(url, 320)));
    assert_eq!(cover_files(&a, it.item), vec![expected_cover_name(&media)]);
}

#[test]
fn h2_c2b_a_new_cover_media_replaces_the_file() {
    let a = shop_app();
    let (it, _) = cover_downloaded(&a);
    let product = product_of(&a, &it);
    let mut ids = gallery_ids(&a, &product);
    ids.swap(0, 1);
    set_gallery(
        &a,
        &product,
        &ids.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    a.remote().refresh(it.session).unwrap();
    request_and_run(&a, &[it.session]);
    assert_eq!(cover_files(&a, it.item), vec![expected_cover_name(&ids[0])]);
}

#[test]
fn h2_c2c_a_cached_cover_is_not_fetched_again() {
    let a = shop_app();
    let (it, _) = cover_downloaded(&a);
    let before = fetches(&a).len();
    request_and_run(&a, &[it.session]);
    assert_eq!(fetches(&a).len(), before);
}

#[test]
fn h2_c2d_one_picture_ready_event_with_the_file() {
    let a = shop_app();
    let (it, media) = cover_downloaded(&a);
    assert_eq!(
        cover_events(&a),
        vec![(it.session, path_string(&cover_file(&a, it.item, &media)))]
    );
}

// ---------------------------------------------------------------------------------------------
// H2·C3 fetching never delays the list, four at a time, per source (MC-H1.3)

#[test]
fn h2_c3a_requesting_covers_downloads_nothing() {
    let a = shop_app();
    let sessions: Vec<i64> = ["A", "B", "C"].iter().map(|s| shelved(&a, s).0).collect();
    let pending: Vec<i64> = sessions
        .iter()
        .filter(|s| row(&a, **s).1)
        .copied()
        .collect();
    assert_eq!(pending, sessions);
    a.remote()
        .request_covers(CoverSource::Page, &pending)
        .unwrap();
    assert!(fetches(&a).is_empty());
}

#[test]
fn h2_c3b_at_most_four_downloads_at_once() {
    let a = shop_app();
    let items: Vec<(i64, i64, String)> = (0..10).map(|i| shelved(&a, &format!("S{i}"))).collect();
    a.fake_shop().set_fetch_delay(50);
    let sessions: Vec<i64> = items.iter().map(|i| i.0).collect();
    request_and_run(&a, &sessions);
    let written = items
        .iter()
        .filter(|(_, item, media)| cover_file(&a, *item, media).is_file())
        .count();
    assert_eq!((written, a.fake_shop().max_fetches_in_flight()), (10, 4));
}

#[test]
fn h2_c3c_a_new_page_replaces_the_page_not_started() {
    let a = shop_app();
    let [a1, b, c, d] = ["A", "B", "C", "D"].map(|s| shelved(&a, s));
    a.remote()
        .request_covers(CoverSource::Page, &[a1.0, b.0])
        .unwrap();
    a.remote()
        .request_covers(CoverSource::Page, &[c.0, d.0])
        .unwrap();
    a.remote().run_pending().unwrap();
    let mut want = vec![fake_image_url(&c.2), fake_image_url(&d.2)];
    want.sort();
    assert_eq!(fetched_urls(&a), want);
}

#[test]
fn h2_c3d_a_source_replaces_only_its_own() {
    let a = shop_app();
    let [a1, b, c] = ["A", "B", "C"].map(|s| shelved(&a, s));
    a.remote()
        .request_covers(CoverSource::Page, &[a1.0, b.0])
        .unwrap();
    a.remote()
        .request_covers(CoverSource::Search, &[c.0])
        .unwrap();
    a.remote().run_pending().unwrap();
    let mut want = vec![
        fake_image_url(&a1.2),
        fake_image_url(&b.2),
        fake_image_url(&c.2),
    ];
    want.sort();
    assert_eq!(fetched_urls(&a), want);
}

// ---------------------------------------------------------------------------------------------
// H2·C4 stale addresses and gone pictures (MC-H1.4, MC-H1.5, MC-H5.3)

#[test]
fn h2_c4a_gone_addresses_are_refreshed_in_one_read_and_retried_once() {
    let a = shop_app();
    let items: Vec<(i64, i64, String)> = ["A", "B", "C"].iter().map(|s| shelved(&a, s)).collect();
    for (_, _, media) in &items {
        a.fake_shop()
            .fail_image(&fake_image_url(media), vec![ImageFault::Gone]);
    }
    let sessions: Vec<i64> = items.iter().map(|i| i.0).collect();
    request_and_run(&a, &sessions);
    let reads = a.fake_shop().url_reads();
    assert_eq!(reads.len(), 1, "{reads:?}");
    let mut asked = reads[0].clone();
    asked.sort();
    let mut want: Vec<String> = items.iter().map(|i| i.2.clone()).collect();
    want.sort();
    assert_eq!(asked, want);
    for (_, item, media) in &items {
        assert_eq!(cover_of(&a, *item).url, fake_refreshed_url(media, 1));
        assert!(cover_file(&a, *item, media).is_file());
    }
    assert_eq!(fetches(&a).len(), 6);
}

/// H2·C4(b): one shelved item whose cover media was deleted from the shop and whose address
/// answers gone, requested and run. (session, item, cover media id).
fn cover_gone(a: &TestApp) -> (i64, i64, String) {
    let (session, item, media) = shelved(a, "GONE");
    let product = a
        .store()
        .item_link(item, "shopify")
        .unwrap()
        .unwrap()
        .external_id;
    let rest: Vec<String> = gallery_ids(a, &product)
        .into_iter()
        .filter(|id| *id != media)
        .collect();
    set_gallery(
        a,
        &product,
        &rest.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    a.fake_shop()
        .fail_image(&fake_image_url(&media), vec![ImageFault::Gone]);
    request_and_run(a, &[session]);
    (session, item, media)
}

#[test]
fn h2_c4b_a_picture_the_shop_no_longer_has_is_gone() {
    let a = shop_app();
    let (session, item, media) = cover_gone(&a);
    assert_eq!(cover_of(&a, item).url, "");
    assert!(!cover_file(&a, item, &media).exists());
    assert_eq!(
        health(&a, "remote_picture_gone").as_deref(),
        Some("1 picture is no longer in the shop: GONE")
    );
    let (fetched, read) = (fetches(&a).len(), a.fake_shop().url_reads().len());
    request_and_run(&a, &[session]);
    assert_eq!(
        (fetches(&a).len(), a.fake_shop().url_reads().len()),
        (fetched, read)
    );
}

#[test]
fn h2_c4c_an_address_is_not_asked_for_again_within_ten_minutes() {
    let a = shop_app();
    let (session, item, media) = shelved(&a, "COOL");
    a.fake_shop()
        .fail_image(&fake_image_url(&media), vec![ImageFault::Gone]);
    a.fake_shop().fail_image(
        &fake_refreshed_url(&media, 1),
        vec![ImageFault::Gone, ImageFault::Gone, ImageFault::Gone],
    );
    request_and_run(&a, &[session]);
    assert!(!cover_file(&a, item, &media).exists());
    assert_eq!(a.fake_shop().url_reads().len(), 1);
    a.clock.advance(9 * MINUTE);
    request_and_run(&a, &[session]);
    assert_eq!(a.fake_shop().url_reads().len(), 1);
    a.clock.advance(2 * MINUTE);
    request_and_run(&a, &[session]);
    assert_eq!(a.fake_shop().url_reads().len(), 2);
}

#[test]
fn h2_c4d_a_failed_download_is_not_refreshed() {
    let a = shop_app();
    let (session, item, media) = shelved(&a, "FAIL");
    a.fake_shop()
        .fail_image(&fake_image_url(&media), vec![ImageFault::Fail]);
    request_and_run(&a, &[session]);
    assert!(!cover_file(&a, item, &media).exists());
    assert!(a.fake_shop().url_reads().is_empty());
}

#[test]
fn h2_c4e_the_refresh_waits_for_the_budget() {
    let a = shop_app();
    let gone = shelved(&a, "LOW");
    let fine = shelved(&a, "FINE");
    a.fake_shop()
        .fail_image(&fake_image_url(&gone.2), vec![ImageFault::Gone]);
    budget(&a, 100.0, 1000.0);
    a.remote()
        .request_covers(CoverSource::Page, &[gone.0, fine.0])
        .unwrap();
    assert_eq!(a.remote().run_pending().unwrap(), Pending::Worked);
    assert!(cover_file(&a, fine.1, &fine.2).is_file());
    assert!(a.fake_shop().url_reads().is_empty());
    budget(&a, 900.0, 1000.0);
    a.remote().run_pending().unwrap();
    assert_eq!(a.fake_shop().url_reads().len(), 1);
    assert!(cover_file(&a, gone.1, &gone.2).is_file());
}

#[test]
fn h2_c4f_refresh_from_the_shop_clears_the_health_item() {
    let a = shop_app();
    let (session, _, _) = cover_gone(&a);
    assert!(health(&a, "remote_picture_gone").is_some());
    a.remote().refresh(session).unwrap();
    assert_eq!(health(&a, "remote_picture_gone"), None);
}

// ---------------------------------------------------------------------------------------------
// H2·C5 no shop change; Delete session (M-H1, M-H5, MC-H1.2, MC-H1.5)

#[test]
fn h2_c5a_no_step_changes_the_shop() {
    // H2·C2(a): its setup publishes, so only what follows it is compared.
    let a = shop_app();
    let it = archived(&a, "ARCH");
    let after_setup = a.fake_shop().mutation_ops();
    request_and_run(&a, &[it.session]);
    assert_eq!(a.fake_shop().mutation_ops(), after_setup);
    // H2·C3(b), C4(a), C4(b): nothing was published.
    let a = shop_app();
    let sessions: Vec<i64> = (0..10).map(|i| shelved(&a, &format!("S{i}")).0).collect();
    a.fake_shop().set_fetch_delay(50);
    request_and_run(&a, &sessions);
    let three: Vec<(i64, i64, String)> = ["A", "B", "C"].iter().map(|s| shelved(&a, s)).collect();
    for (_, _, media) in &three {
        a.fake_shop()
            .fail_image(&fake_image_url(media), vec![ImageFault::Gone]);
    }
    request_and_run(&a, &three.iter().map(|i| i.0).collect::<Vec<_>>());
    cover_gone(&a);
    assert!(a.fake_shop().mutation_ops().is_empty());
}

#[test]
fn h2_c5b_delete_session_removes_the_picture_caches() {
    let a = shop_app();
    let it = archived(&a, "DEL");
    let cover = cover_file(&a, it.item, &cover_of(&a, it.item).media_id);
    let view = previews(&a)
        .join("archived")
        .join(it.item.to_string())
        .join("gallery-01-000000000000.jpg");
    place(&cover);
    place(&view);
    let reads = a.fake_shop().state.lock().unwrap().reads.len();
    let mutations = a.fake_shop().mutation_ops().len();
    session_delete::delete(&a.app.handles(), it.session).unwrap();
    assert_eq!(
        (
            cover.parent().unwrap().exists(),
            view.parent().unwrap().exists()
        ),
        (false, false)
    );
    assert_eq!(a.fake_shop().state.lock().unwrap().reads.len(), reads);
    assert_eq!(a.fake_shop().mutation_ops().len(), mutations);
}

#[test]
fn h2_c5c_delete_session_clears_the_health_item() {
    let a = shop_app();
    let (session, _, _) = cover_gone(&a);
    assert!(health(&a, "remote_picture_gone").is_some());
    session_delete::delete(&a.app.handles(), session).unwrap();
    assert_eq!(health(&a, "remote_picture_gone"), None);
}
