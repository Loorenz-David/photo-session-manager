//! P64 publish speed and visibility (owner 2026-11-10, changelog 44): files are looked up by name
//! (never by listing the shop), uploads are reserved together and run in parallel, the progress
//! explains its step and errors, a publish can be cancelled, and unreviewed photos are published
//! after the main photos. One test per criterion row of `plans/P64_publish_speed_and_visibility.md`.

use psm_core::{
    domain::{
        attribute::{AttributeGroup, CoreType, Definition},
        photo::{Classification as C, NewPhoto, Photo, PhotoSource},
        publish::{DefinitionLink, RemoteMediaStatus},
        session::{ListFilter, PublishStep, RowLabel, SessionStatus},
    },
    mechanisms::hash::sha256_file,
    ports::{commerce::CommerceAdapter, images::uid12},
    services::{attributes::AttributesService, publish::PublishService},
};
use psm_integrations::commerce::shopify::{
    PollConfig, ShopifyAdapter, ShopifyConfig, transport::HttpReply,
};
use psm_testkit::{
    TestApp,
    clock::ManualClock,
    fake_transport::{FakeTransport, Recorded},
    fixtures::{shopify::fixture, synthetic_jpeg::SyntheticJpeg},
};
use serde_json::{Value as Json, json};
use std::{
    collections::HashSet,
    fs,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

const NOW_MS: i64 = 1_762_250_400_000;
const TOKEN_URL: &str = "https://example.myshopify.com/admin/oauth/access_token";

// ---------- adapter over FakeTransport (C1) ----------

fn adapter() -> (Arc<FakeTransport>, ShopifyAdapter) {
    let fake = Arc::new(FakeTransport::new());
    let mut token = fixture("sv0_token").reply();
    token.body["access_token"] = json!("T1-psm-access-token");
    fake.on_form_always(TOKEN_URL, token);
    let adapter = ShopifyAdapter::new(
        ShopifyConfig {
            shop_domain: "example.myshopify.com".into(),
            api_version: "2026-10".into(),
            client_id: "psm-test-client-id".into(),
            client_secret: "SECRET-XYZ".into(),
            condition_photos_key: "custom.damage_report".into(),
        },
        fake.clone(),
        Arc::new(ManualClock::new(1_000_000)),
        PollConfig {
            interval_ms: 0,
            attempts: 5,
            retries: 3,
            retry_wait_ms: 0,
        },
    );
    (fake, adapter)
}

fn file_node(id: usize, name: Option<&str>, alt: &str) -> Json {
    json!({
        "id": format!("gid://shopify/MediaImage/{id}"),
        "fileStatus": if name.is_some() { "READY" } else { "PROCESSING" },
        "alt": alt,
        "image": name.map(|n| json!({ "url": format!("https://cdn.shopify.com/s/files/1/0000/0000/0000/files/{n}?v=1") })),
    })
}

fn files_reply(nodes: Vec<Json>, next: Option<String>) -> HttpReply {
    HttpReply::new(
        200,
        json!({ "data": { "files": {
            "pageInfo": { "hasNextPage": next.is_some(), "endCursor": next },
            "nodes": nodes,
        } } }),
    )
}

/// Every GraphQL request about the shop's files, with whether it carried a search query.
fn file_requests(fake: &FakeTransport) -> Vec<bool> {
    fake.requests()
        .into_iter()
        .filter_map(|r| match r {
            Recorded::Graphql {
                query, variables, ..
            } if query.contains("files(") => Some(
                query.contains("query: $query")
                    && variables["query"].as_str().is_some_and(|q| !q.is_empty()),
            ),
            _ => None,
        })
        .collect()
}

#[test]
fn c1a_lookup_by_name_in_a_shop_of_50000_files() {
    let (fake, adapter) = adapter();
    // The shop: 50 000 files in 500 pages of 100; the wanted one is on the last page. A lookup
    // that lists the shop would page through all of them.
    for page in 0..500 {
        let nodes = (0..100)
            .map(|i| {
                let n = page * 100 + i;
                let name = if n == 49_999 {
                    "SKU-c01.jpg".to_string()
                } else {
                    format!("OTHER-{n}.jpg")
                };
                file_node(n, Some(&name), &name)
            })
            .collect();
        let next = (page < 499).then(|| format!("cursor-{page}"));
        let after = if page == 0 {
            Json::Null
        } else {
            json!(format!("cursor-{}", page - 1))
        };
        fake.on_graphql(
            "FilesAll",
            Some(&json!({ "after": after })),
            vec![files_reply(nodes, next)],
        );
    }
    // Shopify's file search by name answers with the matching file.
    fake.on_graphql(
        "FilesByName",
        Some(&json!({ "query": "filename:SKU-c01" })),
        vec![files_reply(
            vec![file_node(49_999, Some("SKU-c01.jpg"), "SKU-c01.jpg")],
            None,
        )],
    );
    let found = adapter.list_files_by_name("SKU-c01").unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, "gid://shopify/MediaImage/49999");
    assert_eq!(found[0].filename.as_deref(), Some("SKU-c01.jpg"));
    assert_eq!(found[0].status, RemoteMediaStatus::Ready);
    let requests = file_requests(&fake);
    assert!(
        !requests.is_empty() && requests.len() <= 2,
        "files requests: {requests:?}"
    );
    assert!(
        requests.iter().all(|with_query| *with_query),
        "a files request without a search query was sent"
    );
}

#[test]
fn c1b_lookup_keeps_the_match_rules() {
    let (fake, adapter) = adapter();
    // A uniquified CDN name that starts with the stem, plus an unrelated hit of the search.
    fake.on_graphql(
        "FilesByName",
        Some(&json!({ "query": "filename:SKU-c01" })),
        vec![files_reply(
            vec![
                file_node(1, Some("SKU-c01_a1b2.jpg"), "SKU-c01.jpg"),
                file_node(2, Some("SKU-c03.jpg"), "SKU-c03.jpg"),
            ],
            None,
        )],
    );
    // A file still processing: no CDN name yet, recognised by its alt.
    fake.on_graphql(
        "FilesByName",
        Some(&json!({ "query": "filename:SKU-c02" })),
        vec![files_reply(vec![file_node(3, None, "SKU-c02.jpg")], None)],
    );
    let first = adapter.list_files_by_name("SKU-c01").unwrap();
    assert_eq!(
        first
            .iter()
            .map(|m| (m.id.as_str(), m.filename.as_deref()))
            .collect::<Vec<_>>(),
        vec![("gid://shopify/MediaImage/1", Some("SKU-c01_a1b2.jpg"))]
    );
    let second = adapter.list_files_by_name("SKU-c02").unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].id, "gid://shopify/MediaImage/3");
    assert_eq!(second[0].filename, None);
    assert_eq!(second[0].status, RemoteMediaStatus::Processing);
    assert_eq!(file_requests(&fake), vec![true, true]);
}

// ---------- publish service over FakeCommerce (C2–C4) ----------

fn shop_app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(NOW_MS);
    a
}
fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, "Oak", "Sideboard", None)
        .unwrap()
        .1
        .session
        .id
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
/// A RAW known to the fake document; its adjusted preview is rendered when `preview` is true.
fn photo_with(a: &TestApp, s: i64, name: &str, class: C, preview: bool) -> Photo {
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
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    if preview {
        a.app.previews.request_initial(&photo.raw_path).unwrap();
        a.complete_render_with(
            &last_issued(a),
            &SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]),
        )
        .unwrap();
    }
    if class != C::Unreviewed {
        a.review().classify(photo.id, class).unwrap();
    }
    a.store().photo(photo.id).unwrap()
}
fn photo(a: &TestApp, s: i64, name: &str, class: C) -> Photo {
    photo_with(a, s, name, class, true)
}
fn condition_field(a: &TestApp) {
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
fn service(a: &TestApp) -> Arc<PublishService> {
    a.app.publish.clone().unwrap()
}
fn drain(a: &TestApp) {
    for _ in 0..12 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
}
fn publish(a: &TestApp, s: i64) {
    service(a).confirm(&[s], "Publish images").unwrap();
    drain(a);
}
fn latest_item(a: &TestApp, s: i64) -> i64 {
    a.store().latest_publish(s).unwrap().unwrap().item_id
}
fn upload_name(p: &Photo, key: &str) -> String {
    format!("{key}-{}.jpg", uid12(&p.uid))
}
fn gallery_names(a: &TestApp) -> Vec<String> {
    let product = a.fake_shop().state.lock().unwrap().products[0]
        .external_id
        .clone();
    a.fake_shop()
        .item_get(&product)
        .unwrap()
        .media
        .into_iter()
        .map(|m| m.filename.unwrap_or_default())
        .collect()
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
fn count(ops: &[String], op: &str) -> usize {
    ops.iter().filter(|o| o.as_str() == op).count()
}
fn row(a: &TestApp, s: i64) -> psm_core::domain::session::SessionRow {
    a.sessions()
        .list(ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == s)
        .unwrap()
}

#[test]
fn c2a_reservations_batched_and_at_most_four_uploads_in_flight() {
    let a = shop_app();
    condition_field(&a);
    let s = session(&a, "K");
    let main: Vec<Photo> = (0..8)
        .map(|i| photo(&a, s, &format!("m{i}"), C::Main))
        .collect();
    let cond: Vec<Photo> = (0..3)
        .map(|i| photo(&a, s, &format!("c{i}"), C::Condition))
        .collect();
    a.fake_shop().set_upload_delay(40);
    publish(&a, s);
    assert_eq!(
        a.store().publish_item(latest_item(&a, s)).unwrap().step,
        PublishStep::Done
    );
    let ops = a.fake_shop().mutation_ops();
    assert_eq!(count(&ops, "upload"), 11);
    let reservations = count(&ops, "staged_uploads_create");
    assert!(
        (1..=2).contains(&reservations),
        "{reservations} stagedUploadsCreate calls"
    );
    assert_eq!(a.fake_shop().staged_filenames().len(), 11);
    let in_flight = a.fake_shop().max_uploads_in_flight();
    assert!(
        (2..=4).contains(&in_flight),
        "{in_flight} uploads were in flight at once"
    );
    // The outcome is unchanged: the gallery in the frozen order, the condition files uploaded.
    assert_eq!(
        gallery_names(&a),
        main.iter().map(|p| upload_name(p, "K")).collect::<Vec<_>>()
    );
    assert_eq!(
        file_names(&a),
        cond.iter().map(|p| upload_name(p, "K")).collect::<Vec<_>>()
    );
}

#[test]
fn c2b_crash_mid_batch_resumes_without_uploading_a_file_twice() {
    let mut a = shop_app();
    condition_field(&a);
    let s = session(&a, "K");
    let main: Vec<Photo> = (0..8)
        .map(|i| photo(&a, s, &format!("m{i}"), C::Main))
        .collect();
    let cond: Vec<Photo> = (0..3)
        .map(|i| photo(&a, s, &format!("c{i}"), C::Condition))
        .collect();
    service(&a).confirm(&[s], "Publish images").unwrap();
    // The process dies during the 6th upload: 5 uploads have finished.
    let uploads = Arc::new(AtomicUsize::new(0));
    let seen = uploads.clone();
    a.fake_shop().set_hook(Some(Box::new(move |op| {
        if op == "upload" && seen.fetch_add(1, Ordering::SeqCst) + 1 == 6 {
            panic!("simulated process kill after 5 uploads");
        }
    })));
    let killed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drain(&a)));
    assert!(killed.is_err());
    assert!(uploads.load(Ordering::SeqCst) >= 6);
    a.fake_shop().set_hook(None);
    let media_before: Vec<String> = gallery_names(&a);
    a.kill_and_restart().unwrap();
    drain(&a);
    assert_eq!(
        a.store().publish_item(latest_item(&a, s)).unwrap().step,
        PublishStep::Done
    );
    let gallery = gallery_names(&a);
    // No media or file exists twice, and the gallery is the frozen order.
    assert_eq!(
        gallery,
        main.iter().map(|p| upload_name(p, "K")).collect::<Vec<_>>()
    );
    assert_eq!(
        file_names(&a),
        cond.iter().map(|p| upload_name(p, "K")).collect::<Vec<_>>()
    );
    // Photos already on Shopify before the crash were neither staged nor uploaded again.
    let staged = a.fake_shop().staged_filenames();
    for name in &media_before {
        assert_eq!(
            staged.iter().filter(|n| *n == name).count(),
            1,
            "{name} was staged again after the crash"
        );
    }
    assert_eq!(
        count(&a.fake_shop().mutation_ops(), "product_delete_media"),
        0
    );
}

#[test]
fn c3a_progress_names_the_step_then_the_error_and_its_retry_state() {
    let a = shop_app();
    condition_field(&a);
    let s = session(&a, "K");
    photo(&a, s, "m0", C::Main);
    for i in 0..3 {
        photo(&a, s, &format!("c{i}"), C::Condition);
    }
    let svc = service(&a);
    svc.confirm(&[s], "Publish images").unwrap();
    // While the third condition file is checked for readiness, two of three are done.
    let seen: Arc<Mutex<Vec<psm_core::domain::publish::PublishActivity>>> = Arc::default();
    let polls = Arc::new(AtomicUsize::new(0));
    {
        let (seen, polls, svc, clock) = (seen.clone(), polls.clone(), svc.clone(), a.clock.clone());
        a.fake_shop().set_hook(Some(Box::new(move |op| {
            if op == "poll_file" && polls.fetch_add(1, Ordering::SeqCst) + 1 == 3 {
                clock.advance(65_000);
                seen.lock().unwrap().push(svc.activity(s).unwrap().unwrap());
            }
        })));
    }
    drain(&a);
    a.fake_shop().set_hook(None);
    let during = seen.lock().unwrap().pop().expect("the third file poll ran");
    assert_eq!(during.step, PublishStep::Upload);
    assert_eq!(during.text, "Uploading condition photos 2 of 3");
    assert!(
        during.elapsed_ms >= 65_000,
        "elapsed {} ms",
        during.elapsed_ms
    );
    assert_eq!(during.error, None);
    assert_eq!(during.retry, None);
    assert!(during.can_cancel);

    // A second session whose upload fails: the error is shown, waiting for Retry; once retried,
    // the error is shown as being retried.
    let t = session(&a, "L");
    photo(&a, t, "n0", C::Main);
    svc.confirm(&[t], "Publish images").unwrap();
    a.fake_shop().fail_before_effect("upload");
    drain(&a);
    let failed = svc.activity(t).unwrap().unwrap();
    assert_eq!(failed.step, PublishStep::Failed);
    assert_eq!(failed.text, "Publish failed");
    assert_eq!(
        failed.error.as_deref(),
        Some("provider: upload unavailable")
    );
    assert_eq!(failed.retry.as_deref(), Some("retry_available"));
    assert!(!failed.can_cancel);
    svc.retry(failed.item_id).unwrap();
    let retrying = svc.activity(t).unwrap().unwrap();
    assert_eq!(retrying.step, PublishStep::Upload);
    assert_eq!(
        retrying.error.as_deref(),
        Some("provider: upload unavailable")
    );
    assert_eq!(retrying.retry.as_deref(), Some("retrying"));
    drain(&a);
    let done = svc.activity(t).unwrap().unwrap();
    assert_eq!(done.step, PublishStep::Done);
    assert_eq!((done.error, done.retry), (None, None));
}

#[test]
fn c3b_cancel_stops_after_the_request_in_flight_and_the_next_publish_continues() {
    let a = shop_app();
    let s = session(&a, "K");
    let main: Vec<Photo> = (0..8)
        .map(|i| photo(&a, s, &format!("m{i}"), C::Main))
        .collect();
    let svc = service(&a);
    svc.confirm(&[s], "Publish images").unwrap();
    let item = latest_item(&a, s);
    // The photographer cancels while the bar reads 3 of 8 (the 4th photo's readiness check is
    // the request in flight) and confirms.
    let polls = Arc::new(AtomicUsize::new(0));
    let at_cancel: Arc<Mutex<Option<(String, usize)>>> = Arc::default();
    {
        let (polls, at_cancel, svc, fake) = (
            polls.clone(),
            at_cancel.clone(),
            svc.clone(),
            a.commerce.clone().unwrap(),
        );
        a.fake_shop().set_hook(Some(Box::new(move |op| {
            if op == "poll_media_ready" && polls.fetch_add(1, Ordering::SeqCst) + 1 == 4 {
                let text = svc.activity(s).unwrap().unwrap().text;
                svc.cancel(item).unwrap();
                let mutations = fake.state.lock().unwrap().mutations.len();
                *at_cancel.lock().unwrap() = Some((text, mutations));
            }
        })));
    }
    drain(&a);
    a.fake_shop().set_hook(None);
    let (text, mutations_at_cancel) = at_cancel.lock().unwrap().clone().expect("cancelled");
    assert_eq!(text, "Uploading main photos 3 of 8");
    // No upload or mutation started after the request in flight.
    assert_eq!(
        a.fake_shop().state.lock().unwrap().mutations.len(),
        mutations_at_cancel
    );
    let cancelled = a.store().publish_item(item).unwrap();
    assert_eq!(cancelled.step, PublishStep::Cancelled);
    assert_eq!(cancelled.error, None);
    assert_eq!(svc.activity(s).unwrap().unwrap().text, "Publish cancelled");
    let r = row(&a, s);
    assert_eq!(r.label, RowLabel::PublishCancelled);
    assert_ne!(r.status, SessionStatus::Published);
    assert_eq!(a.store().session(s).unwrap().session.published_at, None);
    // Nothing on Shopify was deleted; the product created by this publish stays linked.
    let ops = a.fake_shop().mutation_ops();
    assert_eq!(count(&ops, "product_delete_media"), 0);
    let on_shop = gallery_names(&a);
    assert_eq!(on_shop.len(), 4);
    let ready_before: HashSet<String> = on_shop.iter().cloned().collect();
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    assert!(a.store().item_link(item_id, "shopify").unwrap().is_some());

    // The next Publish uploads only the photos without media on Shopify and ends in the full
    // frozen order, with no photo uploaded twice.
    let staged_before = a.fake_shop().staged_filenames().len();
    let uploads_before = count(&ops, "upload");
    publish(&a, s);
    assert_eq!(
        a.store().publish_item(latest_item(&a, s)).unwrap().step,
        PublishStep::Done
    );
    let ops = a.fake_shop().mutation_ops();
    let newly_staged = a.fake_shop().staged_filenames()[staged_before..].to_vec();
    assert_eq!(
        newly_staged,
        main[4..]
            .iter()
            .map(|p| upload_name(p, "K"))
            .collect::<Vec<_>>()
    );
    assert_eq!(count(&ops, "upload") - uploads_before, 4);
    assert!(newly_staged.iter().all(|n| !ready_before.contains(n)));
    assert_eq!(count(&ops, "item_create"), 1);
    assert_eq!(
        gallery_names(&a),
        main.iter().map(|p| upload_name(p, "K")).collect::<Vec<_>>()
    );
}

#[test]
fn c4a_unreviewed_photos_are_published_after_the_main_photos() {
    let a = shop_app();
    condition_field(&a);
    let s = session(&a, "K");
    // In the workspace's order: B, C, A, D, E, F. Main B and A are positions 1 and 2 of Main;
    // unreviewed C and D have no position and keep their arrival order (C before D).
    let b = photo(&a, s, "b", C::Main);
    let c = photo(&a, s, "c", C::Unreviewed);
    let a_ = photo(&a, s, "a", C::Main);
    let d = photo(&a, s, "d", C::Unreviewed);
    let e = photo(&a, s, "e", C::Condition);
    let f = photo(&a, s, "f", C::Main);
    a.review().classify(f.id, C::Deleted).unwrap();
    let position = |p: &Photo| a.store().photo(p.id).unwrap().position;
    assert!(position(&b) < position(&a_));
    let preview = service(&a).preview(&[s]).unwrap();
    assert_eq!(preview.items[0].main, vec![b.id, a_.id, c.id, d.id]);
    assert_eq!(preview.items[0].condition, vec![e.id]);
    publish(&a, s);
    let item = a.store().publish_item(latest_item(&a, s)).unwrap();
    assert_eq!(item.frozen_main, vec![b.id, a_.id, c.id, d.id]);
    assert_eq!(item.frozen_condition, vec![e.id]);
    assert_eq!(item.step, PublishStep::Done);
    assert_eq!(
        gallery_names(&a),
        [&b, &a_, &c, &d]
            .iter()
            .map(|p| upload_name(p, "K"))
            .collect::<Vec<_>>()
    );
    assert_eq!(file_names(&a), vec![upload_name(&e, "K")]);
    let f_name = upload_name(&f, "K");
    assert!(!a.fake_shop().staged_filenames().contains(&f_name));
}

#[test]
fn c4b_ready_without_every_photo_reviewed() {
    let a = shop_app();
    let s = session(&a, "K");
    photo(&a, s, "a", C::Main);
    photo(&a, s, "c", C::Unreviewed);
    assert_eq!(row(&a, s).status, SessionStatus::Ready);
    assert_eq!(row(&a, s).label, RowLabel::Ready);

    let t = session(&a, "L");
    photo(&a, t, "a2", C::Main);
    photo_with(&a, t, "c2", C::Unreviewed, false);
    assert_eq!(row(&a, t).status, SessionStatus::InProgress);
}
