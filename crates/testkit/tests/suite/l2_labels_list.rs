//! L2·U1 (`docs/implementation/plans/L2_labels_box.md` L2·C1–C6): the Labels box's list in the
//! service (MC-L1.1–.8): rows, matching to sessions, pictures, opening an unmatched row, the
//! box's number and Select all, through `App.labels` on `TestApp::with_pricing()` with the fake
//! Pricing server, the fake shop and the manual clock. "Pn" = a label product of the fake Pricing
//! with product id `n`; "a linked session for Pn" = the shop product `gid://shopify/Product/<n>`
//! adopted and given a session (as in L1).

use psm_core::{
    Error,
    domain::{
        conversation::ConnectionState,
        item::ItemOrigin,
        label::{
            LabelFilter, LabelPage, LabelState, LabelStatus, LabelsAccess, LabelsError, Selected,
        },
        remote::PictureTarget,
    },
    events::AppEvent,
    mechanisms::remote::search_name,
    ports::{clock::Clock, conversations::ConversationStore},
    services::{items::ItemsService, labels::LabelsFailure, remote::Pending},
};
use psm_testkit::{TestApp, fake_pricing::LabelSpec, test_app::PRICING_KEY};
use rusqlite::Connection;
use std::path::PathBuf;

/// 2026-11-20T09:12:00Z.
const T: i64 = 1_795_165_920_000;
const THUMB: &str = "https://cdn.shopify.com/s/files/1/0000/0000/0000/files/p3_160x.jpg";

fn app() -> TestApp {
    let a = TestApp::with_pricing().unwrap();
    a.clock.advance(1_000_000);
    a
}

fn connect(a: &TestApp) {
    let connection = a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    let mut n = 0;
    while a.app.conversations.tick(a.clock.now_ms()).unwrap().more {
        n += 1;
        assert!(n < 50, "the check never completed");
    }
}

fn connected() -> TestApp {
    let a = app();
    connect(&a);
    clear(&a);
    a
}

fn clear(a: &TestApp) {
    a.fake_pricing().clear_requests();
    a.events.0.lock().unwrap().clear();
}

fn store_id(a: &TestApp) -> String {
    a.app.store.sync().unwrap().store.unwrap().id
}

fn label_requests(a: &TestApp) -> Vec<String> {
    a.fake_pricing()
        .request_lines()
        .into_iter()
        .filter(|line| line.contains("/labels/"))
        .collect()
}

fn products_line(a: &TestApp, query: &str) -> String {
    format!("GET /stores/{}/labels/products?{query}", store_id(a))
}

/// A priced, not printed label product `id` created at `T + created` seconds.
fn product(a: &TestApp, id: &str, sku: Option<&str>, created: i64) {
    a.fake_pricing().add_label_product(LabelSpec {
        product_id: id.into(),
        sku: sku.map(Into::into),
        title: format!("Pricing title {id}"),
        product_type: Some("Test tables".into()),
        price_minor: Some(890_000),
        created_at: Some(T + created * 1_000),
        ..LabelSpec::default()
    });
}

/// A linked session for `Pn`: the shop product adopted and given a session; answers the session.
fn linked(a: &TestApp, n: &str, sku: &str) -> i64 {
    let item = adopted(a, n, sku);
    a.sessions().create(item).unwrap().session.id
}

/// The shop product `gid://shopify/Product/<n>` adopted (no session); answers the item id.
fn adopted(a: &TestApp, n: &str, sku: &str) -> i64 {
    let external = a.fake_shop().add_product_with_id(
        &format!("gid://shopify/Product/{n}"),
        sku,
        &format!("{sku} shop title"),
        None,
    );
    ItemsService::new(a.store(), a.clock.as_ref())
        .with_commerce(Some(a.fake_shop()))
        .adopt(&external)
        .unwrap()
        .id
}

/// A local item (no link) with a session; answers the session.
fn local(a: &TestApp, sku: &str) -> i64 {
    ItemsService::new(a.store(), a.clock.as_ref())
        .create_local(sku, &format!("{sku} local title"), "Chair", None)
        .unwrap()
        .1
        .session
        .id
}

fn shop_reads(a: &TestApp) -> Vec<String> {
    a.fake_shop().state.lock().unwrap().reads.clone()
}

fn count(a: &TestApp, table: &str) -> i64 {
    Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

fn list_all(a: &TestApp) -> LabelPage {
    a.app.labels.list(LabelFilter::All, None, None).unwrap()
}

fn row_session(page: &LabelPage, product_id: &str) -> Option<i64> {
    page.rows
        .iter()
        .find(|r| r.product_id == product_id)
        .expect("row listed")
        .session
        .as_ref()
        .map(|s| s.id)
}

fn missing_tag() -> LabelState {
    LabelState {
        status: LabelStatus::NotPrinted,
        printed_at: None,
        has_price: Some(true),
        reset: None,
    }
}

// ---------------------------------------------------------------------------------------------
// L2·C1 the list (MC-L1.1, .2, .5)

#[test]
fn l2_c1a_one_request_rows_newest_first_with_their_fields() {
    let a = app();
    product(&a, "1", Some("TEST-1"), 1);
    product(&a, "2", Some("TEST-2"), 2);
    product(&a, "3", Some("TEST-3"), 3);
    connect(&a);
    clear(&a);
    let page = a
        .app
        .labels
        .list(LabelFilter::MissingPriceTag, None, None)
        .unwrap();
    assert_eq!(
        label_requests(&a),
        vec![products_line(&a, "filter=missingPriceTag&limit=25")]
    );
    let got: Vec<_> = page
        .rows
        .iter()
        .map(|r| {
            (
                r.product_id.as_str(),
                r.sku.as_deref(),
                r.title.as_str(),
                r.price_minor,
                r.state,
                r.created_at,
            )
        })
        .collect();
    assert_eq!(
        got,
        vec![
            (
                "3",
                Some("TEST-3"),
                "Pricing title 3",
                Some(890_000),
                missing_tag(),
                Some(T + 3_000)
            ),
            (
                "2",
                Some("TEST-2"),
                "Pricing title 2",
                Some(890_000),
                missing_tag(),
                Some(T + 2_000)
            ),
            (
                "1",
                Some("TEST-1"),
                "Pricing title 1",
                Some(890_000),
                missing_tag(),
                Some(T + 1_000)
            ),
        ]
    );
    assert_eq!(page.counts.missing_price_tag, 3);
    assert_eq!(page.next_cursor, None);
}

#[test]
fn l2_c1b_pages_through_the_cursor() {
    let a = connected();
    a.fake_pricing().add_label_products(30);
    let first = a
        .app
        .labels
        .list(LabelFilter::MissingPriceTag, None, None)
        .unwrap();
    assert_eq!(first.rows.len(), 25);
    let cursor = first.next_cursor.clone().expect("a cursor");
    assert_eq!(cursor, "o:25");
    clear(&a);
    let second = a
        .app
        .labels
        .list(LabelFilter::MissingPriceTag, None, Some(&cursor))
        .unwrap();
    assert_eq!(
        label_requests(&a),
        vec![products_line(
            &a,
            "filter=missingPriceTag&cursor=o%3A25&limit=25"
        )]
    );
    assert_eq!(second.rows.len(), 5);
    assert_eq!(second.next_cursor, None);
    let mut ids: Vec<&str> = first
        .rows
        .iter()
        .chain(&second.rows)
        .map(|r| r.product_id.as_str())
        .collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 30);
}

#[test]
fn l2_c1c_the_search_is_trimmed_dropped_when_empty_and_cut() {
    let a = connected();
    let labels = &a.app.labels;
    labels
        .list(LabelFilter::Printed, Some("  teak  "), None)
        .unwrap();
    labels.list(LabelFilter::All, Some("   "), None).unwrap();
    let long = format!("{}{}", "a".repeat(100), "b".repeat(50));
    labels.list(LabelFilter::All, Some(&long), None).unwrap();
    assert_eq!(
        label_requests(&a),
        vec![
            products_line(&a, "filter=printed&q=teak&limit=25"),
            products_line(&a, "limit=25"),
            products_line(&a, &format!("q={}&limit=25", "a".repeat(100))),
        ]
    );
}

#[test]
fn l2_c1d_work_from_is_passed_through() {
    let a = connected();
    assert_eq!(list_all(&a).work_from.as_deref(), Some("2026-11-19"));
}

#[test]
fn l2_c1e_a_session_marker_reads_the_list_state_without_a_status_request() {
    let a = app();
    product(&a, "1", Some("TEST-1"), 1);
    product(&a, "2", Some("SH-2"), 2);
    product(&a, "3", Some("TEST-3"), 3);
    let s = linked(&a, "2", "SH-2");
    connect(&a);
    clear(&a);
    a.app
        .labels
        .list(LabelFilter::MissingPriceTag, None, None)
        .unwrap();
    clear(&a);
    let labels = a.app.labels.session_labels(&[s], false).unwrap();
    assert_eq!(labels[0].state, Some(missing_tag()));
    assert_eq!(label_requests(&a), Vec::<String>::new());
}

#[test]
fn l2_c1f_no_key_or_denied_answer_off_or_denied_without_a_request() {
    let a = app();
    product(&a, "1", Some("TEST-1"), 1);
    assert!(matches!(
        a.app.labels.list(LabelFilter::MissingPriceTag, None, None),
        Err(LabelsFailure::Off(ConnectionState::NoKey))
    ));
    assert_eq!(label_requests(&a), Vec::<String>::new());

    let d = app();
    d.fake_pricing().deny_labels();
    connect(&d);
    assert_eq!(d.app.labels.access().unwrap(), LabelsAccess::Denied);
    clear(&d);
    assert!(matches!(
        d.app.labels.list(LabelFilter::MissingPriceTag, None, None),
        Err(LabelsFailure::Denied)
    ));
    assert_eq!(label_requests(&d), Vec::<String>::new());
}

#[test]
fn l2_c1g_provider_failures_map_to_their_errors() {
    let a = connected();
    let fake = a.fake_pricing();
    fake.fail_matching_with(
        "/labels/products",
        1,
        503,
        "product_list_unavailable",
        "The product list is unavailable.",
    );
    assert!(matches!(
        list_err(&a),
        LabelsFailure::Provider(LabelsError::ListUnavailable)
    ));
    fake.fail_matching("/labels/products", 1, 500);
    assert!(matches!(
        list_err(&a),
        LabelsFailure::Provider(LabelsError::Unreachable(_))
    ));
    fake.fail_matching_with(
        "/labels/products",
        1,
        403,
        "insufficient_scope",
        "This key lacks labels:read.",
    );
    assert!(matches!(
        list_err(&a),
        LabelsFailure::Provider(LabelsError::NotAllowed)
    ));
    assert_eq!(a.app.labels.access().unwrap(), LabelsAccess::Denied);
}

fn list_err(a: &TestApp) -> LabelsFailure {
    a.app
        .labels
        .list(LabelFilter::MissingPriceTag, None, None)
        .unwrap_err()
}

// ---------------------------------------------------------------------------------------------
// L2·C2 matching (MC-L1.3)

#[test]
fn l2_c2a_a_linked_session_is_the_rows_session() {
    let a = app();
    product(&a, "1", Some("SH-1"), 1);
    let s1 = linked(&a, "1", "SH-1");
    connect(&a);
    let page = list_all(&a);
    let session = page.rows[0].session.as_ref().expect("matched");
    assert_eq!(session.id, s1);
    assert_eq!(session.sku, "SH-1");
    assert_eq!(session.title, "SH-1 shop title");
}

#[test]
fn l2_c2b_a_local_item_with_the_same_sku_is_matched() {
    let a = app();
    product(&a, "2", Some("AB-1"), 1);
    let s2 = local(&a, "AB-1");
    connect(&a);
    assert_eq!(row_session(&list_all(&a), "2"), Some(s2));
}

#[test]
fn l2_c2c_the_link_wins_over_the_sku() {
    let a = app();
    let sa = linked(&a, "1", "A-1");
    let _sb = local(&a, "AB-1");
    product(&a, "1", Some("AB-1"), 1);
    connect(&a);
    assert_eq!(row_session(&list_all(&a), "1"), Some(sa));
}

#[test]
fn l2_c2d_an_unmatched_row_has_pricings_title_and_its_shop_id() {
    let a = app();
    product(&a, "3", Some("TEST-3"), 1);
    local(&a, "OTHER-1");
    connect(&a);
    let page = list_all(&a);
    let row = &page.rows[0];
    assert_eq!(row.session, None);
    assert_eq!(row.title, "Pricing title 3");
    assert_eq!(row.commerce_id.as_deref(), Some("gid://shopify/Product/3"));
}

#[test]
fn l2_c2e_no_sku_or_a_sku_in_another_case_matches_nothing() {
    let a = app();
    local(&a, "AB-1");
    product(&a, "8", None, 1);
    product(&a, "9", Some("ab-1"), 2);
    connect(&a);
    let page = list_all(&a);
    assert_eq!(row_session(&page, "8"), None);
    assert_eq!(row_session(&page, "9"), None);
}

#[test]
fn l2_c2f_a_linked_item_without_a_session_gives_no_session() {
    let a = app();
    adopted(&a, "4", "SH-4");
    product(&a, "4", Some("SH-4"), 1);
    connect(&a);
    assert_eq!(row_session(&list_all(&a), "4"), None);
}

// ---------------------------------------------------------------------------------------------
// L2·C3 pictures (MC-L1.4)

#[test]
fn l2_c3a_a_matched_row_has_its_sessions_picture_while_the_file_exists() {
    let a = app();
    product(&a, "1", Some("SH-1"), 1);
    let s1 = linked(&a, "1", "SH-1");
    let photo = a.add_photo(s1, "SH-1_M0.ARW").unwrap();
    let preview = a.dir.path().join("test-previews").join("adjusted.jpg");
    std::fs::create_dir_all(preview.parent().unwrap()).unwrap();
    std::fs::write(&preview, b"cached").unwrap();
    Connection::open(a.app.paths.db())
        .unwrap()
        .execute(
            "UPDATE photos SET classification='main', position=1, adjusted_preview_path=?1 \
             WHERE id=?2",
            rusqlite::params![preview.to_string_lossy(), photo.id],
        )
        .unwrap();
    connect(&a);
    let cover = |page: &LabelPage| page.rows[0].session.clone().unwrap().cover_preview_path;
    assert_eq!(
        cover(&list_all(&a)),
        Some(preview.to_string_lossy().into_owned())
    );
    std::fs::remove_file(&preview).unwrap();
    assert_eq!(cover(&list_all(&a)), None);
}

fn search_file(a: &TestApp, url: &str) -> PathBuf {
    a.app
        .paths
        .root
        .join("previews")
        .join("remote")
        .join("search")
        .join(search_name(url))
}

fn run_pending(a: &TestApp) {
    for _ in 0..20 {
        if matches!(a.remote().run_pending().unwrap(), Pending::Idle) {
            return;
        }
    }
}

#[test]
fn l2_c3b_an_unmatched_rows_picture_comes_through_the_search_cache() {
    let a = app();
    a.fake_pricing().add_label_product(LabelSpec {
        product_id: "3".into(),
        sku: Some("TEST-3".into()),
        title: "Test table".into(),
        price_minor: Some(890_000),
        thumb_url: Some(THUMB.into()),
        created_at: Some(T),
        ..LabelSpec::default()
    });
    connect(&a);
    assert_eq!(list_all(&a).rows[0].picture_path, None);
    run_pending(&a);
    let file = search_file(&a, THUMB);
    assert!(file.is_file());
    assert_eq!(a.fake_shop().image_fetches(), vec![(THUMB.to_owned(), 160)]);
    let ready: Vec<(PictureTarget, String)> = a
        .events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::PictureReady { target, path } => Some((target.clone(), path.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        ready,
        vec![(
            PictureTarget::Search {
                external_id: "gid://shopify/Product/3".into()
            },
            file.to_string_lossy().into_owned()
        )]
    );
    assert_eq!(list_all(&a).rows[0].picture_path, Some(file));
}

#[test]
fn l2_c3c_no_picture_without_an_https_address() {
    let a = app();
    for (id, thumb) in [("3", None), ("5", Some("http://cdn.example/x.jpg"))] {
        a.fake_pricing().add_label_product(LabelSpec {
            product_id: id.into(),
            sku: Some(format!("TEST-{id}")),
            title: "Test table".into(),
            thumb_url: thumb.map(Into::into),
            created_at: Some(T),
            ..LabelSpec::default()
        });
    }
    connect(&a);
    let page = list_all(&a);
    assert!(page.rows.iter().all(|r| r.picture_path.is_none()));
    run_pending(&a);
    assert_eq!(a.fake_shop().image_fetches(), Vec::<(String, u32)>::new());
}

#[test]
fn l2_c3d_the_list_writes_nothing_to_the_shop_or_to_pricing() {
    let a = app();
    product(&a, "1", Some("SH-1"), 1);
    linked(&a, "1", "SH-1");
    a.fake_pricing().add_label_product(LabelSpec {
        product_id: "3".into(),
        sku: Some("TEST-3".into()),
        title: "Test table".into(),
        thumb_url: Some(THUMB.into()),
        created_at: Some(T),
        ..LabelSpec::default()
    });
    a.fake_pricing().add_label_product(LabelSpec {
        product_id: "5".into(),
        title: "Test table".into(),
        thumb_url: Some("http://cdn.example/x.jpg".into()),
        created_at: Some(T),
        ..LabelSpec::default()
    });
    connect(&a);
    list_all(&a);
    run_pending(&a);
    list_all(&a);
    assert_eq!(a.fake_shop().mutation_ops(), Vec::<String>::new());
    assert!(
        a.fake_pricing()
            .request_lines()
            .iter()
            .all(|l| !l.contains("/labels/runs"))
    );
}

// ---------------------------------------------------------------------------------------------
// L2·C4 opening a row (MC-L1.8)

#[test]
fn l2_c4a_a_linked_product_opens_its_session_without_a_shop_call() {
    let a = connected();
    let s1 = linked(&a, "1", "SH-1");
    let reads = shop_reads(&a).len();
    assert_eq!(a.app.labels.open_product("1").unwrap(), s1);
    assert_eq!(shop_reads(&a).len(), reads);
}

#[test]
fn l2_c4b_an_unmatched_product_is_adopted_with_one_read_and_no_write() {
    let a = connected();
    a.fake_shop()
        .add_product_with_id("gid://shopify/Product/4", "SH-4", "Test table", None);
    let reads = shop_reads(&a).len();
    let session = a.app.labels.open_product("4").unwrap();
    let item_id = a.store().session(session).unwrap().session.item_id.unwrap();
    let item = a.store().item(item_id).unwrap();
    assert_eq!(item.sku, "SH-4");
    assert_eq!(item.origin, ItemOrigin::Linked);
    assert_eq!(
        a.store()
            .item_link(item_id, "shopify")
            .unwrap()
            .map(|l| l.external_id),
        Some("gid://shopify/Product/4".into())
    );
    assert_eq!(
        shop_reads(&a)[reads..].to_vec(),
        vec!["item_get".to_owned()]
    );
    assert_eq!(a.fake_shop().mutation_ops(), Vec::<String>::new());
}

#[test]
fn l2_c4c_opening_again_answers_the_same_session_and_the_row_is_matched() {
    let a = connected();
    a.fake_shop()
        .add_product_with_id("gid://shopify/Product/4", "SH-4", "Test table", None);
    product(&a, "4", Some("SH-4"), 1);
    let session = a.app.labels.open_product("4").unwrap();
    let reads = shop_reads(&a).len();
    assert_eq!(a.app.labels.open_product("4").unwrap(), session);
    assert_eq!(shop_reads(&a).len(), reads);
    assert_eq!(row_session(&list_all(&a), "4"), Some(session));
}

#[test]
fn l2_c4d_a_linked_item_without_a_session_gets_one_without_a_shop_call() {
    let a = connected();
    let item = adopted(&a, "5", "SH-5");
    let reads = shop_reads(&a).len();
    let session = a.app.labels.open_product("5").unwrap();
    assert_eq!(
        a.store().session(session).unwrap().session.item_id,
        Some(item)
    );
    assert_eq!(shop_reads(&a).len(), reads);
}

#[test]
fn l2_c4e_a_shop_product_without_a_sku_creates_nothing() {
    let a = connected();
    a.fake_shop()
        .add_product_without_sku("gid://shopify/Product/6", "Test table");
    let (items, sessions) = (count(&a, "items"), count(&a, "sessions"));
    assert!(matches!(
        a.app.labels.open_product("6"),
        Err(LabelsFailure::Storage(Error::InvalidValue(_)))
    ));
    assert_eq!(
        (count(&a, "items"), count(&a, "sessions")),
        (items, sessions)
    );
}

#[test]
fn l2_c4f_a_sku_collision_creates_nothing() {
    let a = connected();
    a.fake_shop()
        .add_product_with_id("gid://shopify/Product/7", "SH-7", "Test table", None);
    local(&a, "sh-7");
    let (items, sessions) = (count(&a, "items"), count(&a, "sessions"));
    assert!(matches!(
        a.app.labels.open_product("7"),
        Err(LabelsFailure::Storage(Error::SkuCollision { .. }))
    ));
    assert_eq!(
        (count(&a, "items"), count(&a, "sessions")),
        (items, sessions)
    );
}

#[test]
fn l2_c4g_a_product_id_that_is_no_shop_id_is_invalid_without_a_shop_call() {
    let a = connected();
    let reads = shop_reads(&a).len();
    assert!(matches!(
        a.app.labels.open_product("abc"),
        Err(LabelsFailure::Storage(Error::InvalidValue(_)))
    ));
    assert_eq!(shop_reads(&a).len(), reads);
}

// ---------------------------------------------------------------------------------------------
// L2·C5 the box's number (MC-L1.6, owner card 1)

fn three_to_print() -> TestApp {
    let a = app();
    product(&a, "1", Some("TEST-1"), 1);
    product(&a, "2", Some("TEST-2"), 2);
    product(&a, "3", Some("TEST-3"), 3);
    connect(&a);
    clear(&a);
    a
}

#[test]
fn l2_c5a_a_refreshing_summary_asks_one_unsearched_row() {
    let a = three_to_print();
    assert_eq!(a.app.labels.summary(true).unwrap(), Some(3));
    assert_eq!(
        label_requests(&a),
        vec![products_line(&a, "filter=missingPriceTag&limit=1")]
    );
}

#[test]
fn l2_c5b_a_summary_without_refresh_never_asks() {
    let a = three_to_print();
    assert_eq!(a.app.labels.summary(false).unwrap(), None);
    assert_eq!(label_requests(&a), Vec::<String>::new());
    a.app.labels.summary(true).unwrap();
    clear(&a);
    assert_eq!(a.app.labels.summary(false).unwrap(), Some(3));
    assert_eq!(label_requests(&a), Vec::<String>::new());
}

#[test]
fn l2_c5c_a_fresh_count_is_not_asked_again() {
    let a = three_to_print();
    a.app.labels.summary(true).unwrap();
    clear(&a);
    a.clock.advance(9_000);
    a.app.labels.summary(true).unwrap();
    assert_eq!(label_requests(&a).len(), 0);
    a.clock.advance(2_000);
    a.app.labels.summary(true).unwrap();
    assert_eq!(label_requests(&a).len(), 1);
}

#[test]
fn l2_c5d_the_number_follows_the_search_until_a_refresh() {
    let a = app();
    let add = |id: &str, title: &str, printed: bool| {
        a.fake_pricing().add_label_product(LabelSpec {
            product_id: id.into(),
            sku: Some(format!("TEST-{id}")),
            title: title.into(),
            price_minor: Some(100_000),
            printed_at: printed.then_some(T),
            created_at: Some(T),
            ..LabelSpec::default()
        });
    };
    add("1", "Oak table", false);
    add("2", "Oak chair", true);
    add("3", "Teak table", false);
    add("4", "Pine shelf", false);
    connect(&a);
    let page = a
        .app
        .labels
        .list(LabelFilter::Printed, Some("oak"), None)
        .unwrap();
    assert_eq!(page.counts.missing_price_tag, 1);
    clear(&a);
    assert_eq!(a.app.labels.summary(false).unwrap(), Some(1));
    assert_eq!(a.app.labels.summary(true).unwrap(), Some(3));
    assert_eq!(
        label_requests(&a),
        vec![products_line(&a, "filter=missingPriceTag&limit=1")]
    );
}

#[test]
fn l2_c5e_no_number_and_no_request_without_access() {
    let a = app();
    product(&a, "1", Some("TEST-1"), 1);
    assert_eq!(a.app.labels.summary(true).unwrap(), None);
    assert_eq!(label_requests(&a), Vec::<String>::new());

    let d = app();
    d.fake_pricing().deny_labels();
    connect(&d);
    clear(&d);
    assert_eq!(d.app.labels.summary(true).unwrap(), None);
    assert_eq!(label_requests(&d), Vec::<String>::new());
}

#[test]
fn l2_c5f_test_connection_forgets_the_number() {
    let a = three_to_print();
    a.app.labels.summary(true).unwrap();
    a.app.conversations.test_connection().unwrap();
    assert_eq!(a.app.labels.summary(false).unwrap(), None);
}

// ---------------------------------------------------------------------------------------------
// L2·C6 Select all (MC-L1.7)

#[test]
fn l2_c6a_select_all_pages_to_the_end_in_the_lists_order() {
    let a = connected();
    let mut ids = a.fake_pricing().add_label_products(620);
    ids.reverse();
    let selected = a
        .app
        .labels
        .select_all(LabelFilter::MissingPriceTag, None)
        .unwrap();
    assert_eq!(
        label_requests(&a),
        vec![
            products_line(&a, "filter=missingPriceTag&limit=500"),
            products_line(&a, "filter=missingPriceTag&cursor=o%3A500&limit=500"),
        ]
    );
    let got: Vec<&str> = selected.iter().map(|s| s.product_id.as_str()).collect();
    assert_eq!(got, ids.iter().map(String::as_str).collect::<Vec<_>>());
}

#[test]
fn l2_c6b_select_all_keeps_only_products_with_a_sku() {
    let a = connected();
    product(&a, "1", Some("TEST-1"), 1);
    product(&a, "2", None, 2);
    product(&a, "3", Some("TEST-3"), 3);
    let selected = a
        .app
        .labels
        .select_all(LabelFilter::MissingPriceTag, None)
        .unwrap();
    assert_eq!(
        selected,
        vec![
            Selected {
                product_id: "3".into(),
                sku: "TEST-3".into()
            },
            Selected {
                product_id: "1".into(),
                sku: "TEST-1".into()
            },
        ]
    );
}

#[test]
fn l2_c6c_every_request_carries_the_filter_and_the_search() {
    let a = connected();
    for id in a.fake_pricing().add_label_products(600) {
        a.fake_pricing().set_label(&id, |l| {
            l.title = format!("Oak {}", l.title);
            l.printed_at = Some(T);
        });
    }
    let selected = a
        .app
        .labels
        .select_all(LabelFilter::Printed, Some("oak"))
        .unwrap();
    assert_eq!(selected.len(), 600);
    let lines = label_requests(&a);
    assert_eq!(lines.len(), 2);
    assert!(
        lines
            .iter()
            .all(|l| l.contains("filter=printed") && l.contains("q=oak")),
        "{lines:?}"
    );
}

#[test]
fn l2_c6d_a_repeated_cursor_is_a_bad_reply_not_an_endless_walk() {
    let a = connected();
    a.fake_pricing().add_label_products(620);
    a.fake_pricing().repeat_cursor_next();
    assert!(matches!(
        a.app.labels.select_all(LabelFilter::MissingPriceTag, None),
        Err(LabelsFailure::Provider(LabelsError::BadReply(_)))
    ));
    assert!(label_requests(&a).len() <= 2);
}

#[test]
fn l2_c6e_a_failed_page_is_the_error_and_no_partial_selection() {
    let a = connected();
    a.fake_pricing().add_label_products(620);
    a.fake_pricing()
        .fail_matching_after("/labels/products", 1, 1, 500);
    assert!(matches!(
        a.app.labels.select_all(LabelFilter::MissingPriceTag, None),
        Err(LabelsFailure::Provider(LabelsError::Unreachable(_)))
    ));
    assert_eq!(label_requests(&a).len(), 2);
}

#[test]
fn l2_c6f_select_all_without_a_key_is_off_without_a_request() {
    let a = app();
    a.fake_pricing().add_label_products(3);
    assert!(matches!(
        a.app.labels.select_all(LabelFilter::MissingPriceTag, None),
        Err(LabelsFailure::Off(ConnectionState::NoKey))
    ));
    assert_eq!(label_requests(&a), Vec::<String>::new());
}
