//! P65 (owner 2026-11-11, changelog 45): a photo filed into an item folder makes the app ask
//! Capture One to synchronise that folder; exports and publishes wait for Capture One instead of
//! failing; a product the app creates gets stock tracking with the chosen location and quantity,
//! remembered as the default. One test per criterion row of
//! `plans/P65_capture_sync_waiting_jobs_and_inventory.md`.

use psm_core::{
    domain::{
        photo::{Classification as C, NewPhoto, Photo, PhotoSource},
        publish::{StockChoice, StockLocation, StockOutcome},
        session::{ListFilter, PublishStep, RowLabel},
    },
    mechanisms::hash::sha256_file,
    ports::{camera::CaptureSink, clock::Clock, commerce::CommerceAdapter, images::uid12},
    services::publish::PublishService,
};
use psm_integrations::commerce::shopify::{
    PollConfig, ShopifyAdapter, ShopifyConfig, transport::HttpReply,
};
use psm_testkit::{
    TestApp,
    clock::ManualClock,
    fake_transport::{FakeTransport, Recorded},
    fixtures::shopify::fixture,
};
use serde_json::{Value as Json, json};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

const NOW_MS: i64 = 1_762_840_000_000;
const TOKEN_URL: &str = "https://example.myshopify.com/admin/oauth/access_token";
const NO_ACCESS: &str = "Your Shopify app needs the read_locations and write_inventory permissions";

// ---------- logs ----------

#[derive(Clone)]
struct LogBytes(Arc<Mutex<Vec<u8>>>);
impl Write for LogBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
/// Runs `f` with the log lines it writes collected; returns its result and the log text.
fn logged<T>(f: impl FnOnce() -> T) -> (T, String) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let output = LogBytes(buffer.clone());
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || output.clone())
        .without_time()
        .with_ansi(false)
        .finish();
    let value = tracing::subscriber::with_default(subscriber, f);
    let text = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    (value, text)
}

// ---------- capture (C1) ----------

fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn count(ops: &[String], op: &str) -> usize {
    ops.iter().filter(|o| o.as_str() == op).count()
}
/// An item session made active (the camera files into its folder); returns (session, folder).
fn active_session(a: &TestApp, sku: &str) -> (i64, PathBuf) {
    let (item, session) = a.items().create_local(sku, sku, "object", None).unwrap();
    a.sessions().activate(session.session.id).unwrap();
    (session.session.id, a.capture_root().join(item.folder_key))
}
/// The camera delivers `name` over FTP and the ledger files it into the active item's folder;
/// the watcher sees the file (it is accepted at the next tick).
fn deliver(a: &TestApp, name: &str) -> String {
    use sha2::{Digest, Sha256};
    let bytes = name.as_bytes();
    let staged = a
        .capture_root()
        .join(".incoming")
        .join(format!(".{name}.part"));
    fs::create_dir_all(staged.parent().unwrap()).unwrap();
    fs::write(&staged, bytes).unwrap();
    a.ledger()
        .delivery(psm_core::domain::camera::Delivery {
            camera_filename: name.into(),
            written_path: staged,
            sha256: Sha256::digest(bytes)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
            size: bytes.len() as u64,
            via: psm_core::domain::camera::Link::Wireless,
            completed_at: a.clock.now_ms(),
        })
        .unwrap();
    let final_path = a
        .ledger()
        .deliveries()
        .unwrap()
        .last()
        .unwrap()
        .final_path
        .clone();
    a.app
        .arrival
        .watcher_event(std::path::Path::new(&final_path))
        .unwrap();
    final_path
}

#[test]
fn c1a_filed_photo_is_synced_and_styled_within_the_sync_delay() {
    let mut a = TestApp::with_camera().unwrap();
    a.fake.state.lock().unwrap().styles = vec!["Bright Contrast".into()];
    a.settings_set(&json!({"style_name": "Bright Contrast"}))
        .unwrap();
    let (session, folder) = active_session(&a, "P65A");
    // Capture One lists the photo 3 s after a sync of its folder, else 60 s after first seeing it.
    a.fake.list_after_sync("DSC05161.ARW", "v-5161");
    assert_eq!(a.fake.state.lock().unwrap().sync_delay_ms, 3_000);
    a.fake.state.lock().unwrap().ops.clear();
    let ((raw, filed_at, issued_at), log) = logged(|| {
        let raw = a.arrival_photo(session, "DSC05161.ARW").unwrap();
        let filed_at = a.clock.now_ms();
        let mut issued_at = None;
        for _ in 0..20 {
            a.tick(1_000).unwrap();
            if !a.fake.state.lock().unwrap().issued.is_empty() {
                issued_at = Some(a.clock.now_ms());
                break;
            }
        }
        (
            raw,
            filed_at,
            issued_at.expect("a preview render was issued"),
        )
    });
    let ops = ops(&a);
    assert_eq!(
        count(&ops, &format!("sync_folder:{}", folder.display())),
        1,
        "{ops:?}"
    );
    // Styled before its first render, rendered within the sync delay (not after 60 s).
    let styled = ops
        .iter()
        .position(|o| o == "apply_style:v-5161:Bright Contrast")
        .expect("style applied");
    let processed = ops
        .iter()
        .position(|o| o == "process:v-5161:PSM Preview")
        .expect("preview rendered");
    assert!(styled < processed);
    let waited = issued_at - filed_at;
    assert!(
        (3_000..=4_000).contains(&waited),
        "preview asked for {waited} ms after the photo was filed"
    );
    let job = a.fake.state.lock().unwrap().issued[0].0.clone();
    a.complete_render(&job).unwrap();
    let photo = &a.store().photos_for_raw(&raw).unwrap()[0];
    assert!(photo.adjusted_preview_path.is_some());
    let seen = log
        .lines()
        .find(|l| l.contains("capture_one.variant_seen"))
        .expect("variant_seen logged");
    let raw_field = format!("raw={raw}");
    assert!(seen.contains(&raw_field), "{seen}");
    assert!(
        seen.contains("after_ms=3000") || seen.contains("after_ms=4000"),
        "{seen}"
    );
    let sync = log
        .lines()
        .find(|l| l.contains("capture_one.sync"))
        .expect("sync logged");
    assert!(
        sync.contains(&format!("folder={}", folder.display())),
        "{sync}"
    );
    assert!(sync.contains(" ms="), "{sync}");
}

#[test]
fn c1b_syncs_are_coalesced_per_folder_and_held_back_by_the_take_back() {
    let a = TestApp::with_camera().unwrap();
    let (_, folder) = active_session(&a, "P65B");
    let sync_op = format!("sync_folder:{}", folder.display());
    a.fake.state.lock().unwrap().ops.clear();
    // Three photos filed within 200 ms.
    for name in ["DSC1.ARW", "DSC2.ARW", "DSC3.ARW"] {
        deliver(&a, name);
        a.clock.advance(60);
    }
    for _ in 0..5 {
        a.tick(1_000).unwrap();
    }
    let syncs = count(&ops(&a), &sync_op);
    assert!((1..=2).contains(&syncs), "{syncs} syncs for 3 arrivals");
    assert_eq!(a.store().photos_in_session(1).unwrap().len(), 0);

    // One arrival while "Take the camera back" holds Capture One: no sync until it ends, then one.
    a.fake.state.lock().unwrap().ops.clear();
    let guard = a.launch_gate().hold_quiet();
    deliver(&a, "DSC4.ARW");
    for _ in 0..3 {
        a.tick(1_000).unwrap();
    }
    assert_eq!(count(&ops(&a), &sync_op), 0, "synced during the take-back");
    drop(guard);
    for _ in 0..3 {
        a.tick(1_000).unwrap();
    }
    assert_eq!(count(&ops(&a), &sync_op), 1);
}

// ---------- publish waits (C2) ----------

fn shop_app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(NOW_MS);
    a
}
fn session(a: &TestApp, sku: &str) -> (i64, i64) {
    let (item, session) = a
        .items()
        .create_local(sku, "Oak", "Sideboard", None)
        .unwrap();
    (session.session.id, item.id)
}
/// A main photo whose RAW Capture One has not loaded yet (no variant, no preview).
fn unloaded_photo(a: &TestApp, s: i64, name: &str) -> Photo {
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(name);
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
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.review().classify(photo.id, C::Main).unwrap();
    a.store().photo(photo.id).unwrap()
}
fn loaded_photo(a: &TestApp, s: i64, name: &str) -> Photo {
    let photo = unloaded_photo(a, s, name);
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    photo
}
fn service(a: &TestApp) -> Arc<PublishService> {
    a.app.publish.clone().unwrap()
}
fn step(a: &TestApp, s: i64) -> (PublishStep, Option<String>) {
    let item = a
        .store()
        .publish_item(a.store().latest_publish(s).unwrap().unwrap().item_id)
        .unwrap();
    (item.step, item.error)
}
fn row_label(a: &TestApp, s: i64) -> RowLabel {
    a.sessions()
        .list(ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == s)
        .unwrap()
        .label
}
fn tick_render(a: &TestApp, ms: i64) {
    a.tick(ms).unwrap();
    a.complete_all_renders().unwrap();
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

#[test]
fn c2a_publish_waits_for_capture_one_then_completes() {
    let a = shop_app();
    let (s, _) = session(&a, "W1");
    let first = unloaded_photo(&a, s, "DSC05162.ARW");
    let second = unloaded_photo(&a, s, "DSC05163.ARW");
    // Pressed while the previews are still pending: accepted.
    service(&a).confirm(&[s], "Publish images").unwrap();
    let started = a.clock.now_ms();
    while a.clock.now_ms() - started < 20_000 {
        tick_render(&a, 1_000);
        assert_eq!(
            step(&a, s).0,
            PublishStep::Prepare,
            "the run failed or moved on"
        );
    }
    let activity = service(&a).activity(s).unwrap().unwrap();
    assert_eq!(activity.text, "Waiting for Capture One…");
    assert_eq!(activity.error, None);
    assert!(activity.can_cancel);
    assert_eq!(row_label(&a, s), RowLabel::WaitingForEditor);
    // Capture One loads both photos after 20 s.
    for photo in [&first, &second] {
        a.fake
            .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    }
    for _ in 0..60 {
        tick_render(&a, 1_000);
        if step(&a, s).0 == PublishStep::Done {
            break;
        }
    }
    assert_eq!(step(&a, s), (PublishStep::Done, None));
    assert_eq!(
        gallery_names(&a),
        [&first, &second]
            .iter()
            .map(|p| format!("W1-{}.jpg", uid12(&p.uid)))
            .collect::<Vec<_>>()
    );
    assert_eq!(row_label(&a, s), RowLabel::Published);
}

#[test]
fn c2b_the_wait_ends_after_ten_minutes_and_other_errors_fail_at_once() {
    let a = shop_app();
    let (s, _) = session(&a, "W2");
    unloaded_photo(&a, s, "DSC05164.ARW");
    service(&a).confirm(&[s], "Publish images").unwrap();
    tick_render(&a, 1_000);
    let started = a.clock.now_ms();
    while a.clock.now_ms() - started < 9 * 60_000 + 45_000 {
        tick_render(&a, 15_000);
        assert_eq!(step(&a, s).0, PublishStep::Prepare);
    }
    for _ in 0..4 {
        tick_render(&a, 15_000);
    }
    assert_eq!(
        step(&a, s),
        (
            PublishStep::Failed,
            Some("Capture One has not loaded DSC05164.ARW after 10 minutes".into())
        )
    );
    assert_eq!(row_label(&a, s), RowLabel::PublishFailed);

    // Capture One refuses the render for another reason: failed at once, as before.
    let b = shop_app();
    let (t, _) = session(&b, "W3");
    loaded_photo(&b, t, "DSC05165.ARW");
    b.fake.state.lock().unwrap().fail_process = true;
    service(&b).confirm(&[t], "Publish images").unwrap();
    tick_render(&b, 1_000);
    tick_render(&b, 1_000);
    assert_eq!(
        step(&b, t),
        (PublishStep::Failed, Some("provider: process failed".into()))
    );
}

#[test]
fn c2c_cancel_while_waiting_for_capture_one() {
    let a = shop_app();
    let (s, _) = session(&a, "W4");
    unloaded_photo(&a, s, "DSC05166.ARW");
    let svc = service(&a);
    svc.confirm(&[s], "Publish images").unwrap();
    for _ in 0..5 {
        tick_render(&a, 1_000);
    }
    assert_eq!(
        svc.activity(s).unwrap().unwrap().text,
        "Waiting for Capture One…"
    );
    let item = a.store().latest_publish(s).unwrap().unwrap().item_id;
    svc.cancel(item).unwrap();
    for _ in 0..16 {
        tick_render(&a, 1_000);
    }
    assert_eq!(step(&a, s), (PublishStep::Cancelled, None));
    assert_eq!(svc.activity(s).unwrap().unwrap().text, "Publish cancelled");
    assert_eq!(row_label(&a, s), RowLabel::PublishCancelled);
    // The export stopped waiting with it.
    assert!(!a.app.export.running());
}

// ---------- inventory on create (C3) ----------

fn l1() -> StockChoice {
    StockChoice {
        location_id: "gid://shopify/Location/111".into(),
        location_name: "Warehouse".into(),
        quantity: 1,
    }
}
fn shop_locations(a: &TestApp) {
    a.fake_shop().state.lock().unwrap().locations = vec![
        StockLocation {
            id: "gid://shopify/Location/111".into(),
            name: "Warehouse".into(),
        },
        StockLocation {
            id: "gid://shopify/Location/222".into(),
            name: "Shop floor".into(),
        },
    ];
}
fn publish_all(a: &TestApp, s: i64) {
    service(a).confirm(&[s], "Publish images").unwrap();
    for _ in 0..12 {
        tick_render(a, 1_100);
    }
    assert_eq!(step(a, s), (PublishStep::Done, None));
}
fn settings_json(a: &TestApp) -> Json {
    serde_json::from_slice(&fs::read(a.app.paths.settings()).unwrap()).unwrap()
}

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
fn created_reply() -> HttpReply {
    HttpReply::new(
        200,
        json!({ "data": { "productSet": { "product": {
            "id": "gid://shopify/Product/900", "title": "Oak", "status": "DRAFT",
            "variants": { "nodes": [{ "id": "gid://shopify/ProductVariant/901", "sku": "S-1",
                "barcodes": { "nodes": [] }, "inventoryItem": { "sku": "S-1" } }] }
        }, "userErrors": [] } } }),
    )
}
fn access_denied_reply(field: &str, scope: &str) -> HttpReply {
    HttpReply::new(
        200,
        json!({ "errors": [{ "message": format!("Access denied for {field} field. Required access: `{scope}` access scope."),
            "extensions": { "code": "ACCESS_DENIED" } }] }),
    )
}

#[test]
fn c3a_created_product_is_tracked_with_the_quantity_at_the_location_and_remembered() {
    // The Shopify request: tracked, 1 available at L1, in the same productSet; then read back.
    let (fake, adapter) = adapter();
    fake.on_graphql("ProductSet", None, vec![created_reply()]);
    fake.on_graphql(
        "InventoryReadback",
        Some(&json!({ "id": "gid://shopify/Product/900", "locationId": "gid://shopify/Location/111" })),
        vec![HttpReply::new(
            200,
            json!({ "data": { "product": { "variants": { "nodes": [{ "inventoryItem": {
                "tracked": true,
                "inventoryLevel": { "quantities": [{ "name": "available", "quantity": 1 }] }
            } }] } } } }),
        )],
    );
    let app = TestApp::new().unwrap();
    let run = app.confirmed_run().unwrap();
    let (id, outcome) = adapter
        .item_create_stocked(&run, "S-1", None, "Oak", "", &l1())
        .unwrap();
    assert_eq!(id, "gid://shopify/Product/900");
    assert_eq!(outcome, StockOutcome::Tracked { verified: true });
    let sent = &fake.graphql("ProductSet")[0].1["input"]["variants"][0];
    assert_eq!(sent["inventoryItem"], json!({ "tracked": true }));
    assert_eq!(
        sent["inventoryQuantities"],
        json!([{ "locationId": "gid://shopify/Location/111", "name": "available", "quantity": 1 }])
    );
    assert_eq!(sent["sku"], json!("S-1"));
    assert_eq!(fake.graphql("InventoryReadback").len(), 1);

    // The publish: a new item with L1 / 1 is created with them, and L1 / 1 becomes the default.
    let a = shop_app();
    shop_locations(&a);
    let (s, item) = session(&a, "INV1");
    loaded_photo(&a, s, "a.ARW");
    service(&a).set_item_stock(item, Some(&l1())).unwrap();
    assert_eq!(service(&a).stock_default(), None);
    publish_all(&a, s);
    let shop = a.fake_shop().state.lock().unwrap();
    assert_eq!(
        shop.stock_requests,
        vec![("item_create_stocked".to_string(), l1())]
    );
    let product = shop.products[0].external_id.clone();
    assert_eq!(shop.stock.get(&product), Some(&l1()));
    drop(shop);
    assert_eq!(service(&a).stock_default(), Some(l1()));
    assert_eq!(
        settings_json(&a)["commerce"]["inventory_default"],
        json!({ "location_id": "gid://shopify/Location/111", "location_name": "Warehouse", "quantity": 1 })
    );
}

#[test]
fn c3b_found_products_and_republishes_never_touch_stock() {
    let a = shop_app();
    shop_locations(&a);
    service(&a).set_stock_default(&l1()).unwrap();
    // Found in the shop by its SKU: linked, never created, so no stock fields are sent.
    a.fake_shop().add_product("FOUND-1", "Oak", None);
    let (s, item) = session(&a, "FOUND-1");
    loaded_photo(&a, s, "f.ARW");
    publish_all(&a, s);
    assert!(!service(&a).is_new_item(item).unwrap());
    assert!(
        a.fake_shop()
            .state
            .lock()
            .unwrap()
            .stock_requests
            .is_empty()
    );
    assert!(a.fake_shop().state.lock().unwrap().stock.is_empty());

    // Created with stock, then published again: the second publish sends no stock fields.
    let (t, item) = session(&a, "NEW-1");
    loaded_photo(&a, t, "n1.ARW");
    publish_all(&a, t);
    assert_eq!(a.fake_shop().state.lock().unwrap().stock_requests.len(), 1);
    assert!(!service(&a).is_new_item(item).unwrap());
    assert_eq!(service(&a).item_stock(item).unwrap(), None);
    loaded_photo(&a, t, "n2.ARW");
    publish_all(&a, t);
    assert_eq!(a.fake_shop().state.lock().unwrap().stock_requests.len(), 1);
    let creates = a
        .fake_shop()
        .mutation_ops()
        .iter()
        .filter(|op| op.as_str() == "item_create")
        .count();
    assert_eq!(creates, 1);

    // At the Shopify adapter, only the create carries tracking or quantities.
    let (fake, adapter) = adapter();
    fake.replay_any(&fixture("sv7_create"));
    let app = TestApp::new().unwrap();
    let run = app.confirmed_run().unwrap();
    adapter
        .item_create(&run, "PSM-PROBE-SKU-1", None, "Plain", "")
        .unwrap();
    for request in fake.requests() {
        if let Recorded::Graphql { variables, .. } = request {
            let text = variables.to_string();
            assert!(
                !text.contains("inventoryQuantities") && !text.contains("tracked"),
                "{text}"
            );
        }
    }
}

#[test]
fn c3c_form_default_settings_and_a_shop_without_location_access() {
    let a = shop_app();
    shop_locations(&a);
    let svc = service(&a);
    // The shop's locations are read once per app run.
    let locations = svc.stock_locations().unwrap();
    assert_eq!(
        locations
            .locations
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>(),
        ["Warehouse", "Shop floor"]
    );
    assert_eq!(locations.missing_access, None);
    svc.stock_locations().unwrap();
    assert_eq!(a.fake_shop().state.lock().unwrap().location_reads, 1);
    // A new item without a choice: nothing to prefill; Settings sets a default; the form shows it.
    let (s, item) = session(&a, "FORM-1");
    assert!(svc.is_new_item(item).unwrap());
    assert_eq!(svc.item_stock(item).unwrap(), None);
    let shop_floor = StockChoice {
        location_id: "gid://shopify/Location/222".into(),
        location_name: "Shop floor".into(),
        quantity: 3,
    };
    svc.set_stock_default(&shop_floor).unwrap();
    assert_eq!(svc.stock_default(), Some(shop_floor.clone()));
    assert_eq!(svc.item_stock(item).unwrap(), Some(shop_floor.clone()));
    // "No location" in the form: created untracked even with a default.
    svc.set_item_stock(item, None).unwrap();
    assert_eq!(svc.item_stock(item).unwrap(), None);
    loaded_photo(&a, s, "x.ARW");
    publish_all(&a, s);
    assert!(
        a.fake_shop()
            .state
            .lock()
            .unwrap()
            .stock_requests
            .is_empty()
    );
    // A quantity below 0 is refused.
    assert!(
        svc.set_stock_default(&StockChoice {
            quantity: -1,
            ..shop_floor.clone()
        })
        .is_err()
    );

    // A shop whose app lacks read_locations / write_inventory: the sentence, and the create goes
    // through untracked; the publish does not fail.
    let b = shop_app();
    b.fake_shop().state.lock().unwrap().stock_access_denied = true;
    let svc = service(&b);
    assert_eq!(
        svc.stock_locations().unwrap().missing_access.as_deref(),
        Some(NO_ACCESS)
    );
    let (t, item) = session(&b, "NOACCESS-1");
    loaded_photo(&b, t, "y.ARW");
    svc.set_item_stock(item, Some(&l1())).unwrap();
    publish_all(&b, t);
    assert!(b.fake_shop().state.lock().unwrap().stock.is_empty());

    // The same at the Shopify adapter: locations refused → the sentence; productSet with stock
    // refused (ACCESS_DENIED) → sent again without stock fields, created untracked.
    let (fake, adapter) = adapter();
    fake.on_graphql(
        "Locations",
        None,
        vec![access_denied_reply("locations", "read_locations")],
    );
    assert_eq!(
        adapter.stock_locations().unwrap().missing_access.as_deref(),
        Some(NO_ACCESS)
    );
    fake.on_graphql(
        "ProductSet",
        None,
        vec![
            access_denied_reply("inventoryQuantities", "write_inventory"),
            created_reply(),
        ],
    );
    let app = TestApp::new().unwrap();
    let run = app.confirmed_run().unwrap();
    let (id, outcome) = adapter
        .item_create_stocked(&run, "S-1", None, "Oak", "", &l1())
        .unwrap();
    assert_eq!(id, "gid://shopify/Product/900");
    assert_eq!(
        outcome,
        StockOutcome::Untracked {
            reason: NO_ACCESS.into()
        }
    );
    let sets = fake.graphql("ProductSet");
    assert_eq!(sets.len(), 2);
    assert!(sets[0].1.to_string().contains("inventoryQuantities"));
    assert!(!sets[1].1.to_string().contains("inventoryQuantities"));
    assert!(!sets[1].1.to_string().contains("tracked"));
}
