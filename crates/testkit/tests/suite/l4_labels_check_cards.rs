//! L4·U2 (`docs/implementation/plans/L4_owner_corrections.md` L4·C4): a picture per checked
//! product, from this Mac only (MC-L2.16; book 09C §14.1), through `App.labels` on
//! `TestApp::with_pricing()`, connected and `Allowed`, with the fake Pricing server and the fake
//! shop. "P1" = a label product linked to a session on this Mac; "P2" = a label product with no
//! session here and an `https://` picture address.

use crate::l3_labels_check::{check, connected, label_requests, product};
use psm_core::{
    domain::{label::LabelFilter, session::ListFilter},
    mechanisms::remote::search_name,
    services::{items::ItemsService, remote::Pending},
};
use psm_testkit::{TestApp, fake_pricing::LabelSpec};
use rusqlite::Connection;
use std::{collections::HashMap, path::PathBuf};

const THUMB: &str = "https://cdn.shopify.com/s/files/1/0000/0000/0000/files/test-2_160x.jpg";

/// P1 linked to a session whose first main photo's preview file exists; answers the session.
fn p1_with_picture(a: &TestApp) -> i64 {
    product(a, "1", Some(890_000), None);
    let external =
        a.fake_shop()
            .add_product_with_id("gid://shopify/Product/1", "TEST-1", "Test table", None);
    let item = ItemsService::new(a.store(), a.clock.as_ref())
        .with_commerce(Some(a.fake_shop()))
        .adopt(&external)
        .unwrap()
        .id;
    let session = a.sessions().create(item).unwrap().session.id;
    let photo = a.add_photo(session, "TEST-1_M0.ARW").unwrap();
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
    session
}

/// P2: no session here, a picture address starting `https://`.
fn p2(a: &TestApp) {
    a.fake_pricing().add_label_product(LabelSpec {
        product_id: "2".into(),
        sku: Some("TEST-2".into()),
        title: "Test chair".into(),
        price_minor: Some(450_000),
        thumb_url: Some(THUMB.into()),
        ..LabelSpec::default()
    });
}

fn search_file(a: &TestApp) -> PathBuf {
    a.app
        .paths
        .root
        .join("previews")
        .join("remote")
        .join("search")
        .join(search_name(THUMB))
}

fn run_pending(a: &TestApp) {
    for _ in 0..20 {
        if matches!(a.remote().run_pending().unwrap(), Pending::Idle) {
            return;
        }
    }
}

fn shop_reads(a: &TestApp) -> Vec<String> {
    a.fake_shop().state.lock().unwrap().reads.clone()
}

fn clear_reads(a: &TestApp) {
    a.fake_shop().state.lock().unwrap().reads.clear();
}

fn list_all(a: &TestApp) {
    a.app.labels.list(LabelFilter::All, None, None).unwrap();
}

fn pictures(a: &TestApp, list: &[&str]) -> HashMap<String, PathBuf> {
    check(a, list).pictures
}

fn only(id: &str, path: PathBuf) -> HashMap<String, PathBuf> {
    HashMap::from([(id.to_owned(), path)])
}

/// The row picture `sessions().with_remote(..).list(All, None)` answers for `session`.
fn row_picture(a: &TestApp, session: i64) -> PathBuf {
    let path = a
        .sessions()
        .with_remote(a.remote())
        .list(ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == session)
        .unwrap()
        .cover_preview_path
        .expect("the session row shows a picture");
    PathBuf::from(path)
}

#[test]
fn l4_c4a_a_matched_product_has_its_sessions_row_picture() {
    let a = connected();
    let session = p1_with_picture(&a);
    a.fake_pricing().clear_requests();
    clear_reads(&a);
    let got = pictures(&a, &["1"]);
    assert_eq!(got, only("1", row_picture(&a, session)));
    let store = crate::l3_labels_check::store_id(&a);
    assert_eq!(
        label_requests(&a),
        vec![
            format!("POST /stores/{store}/labels/plan"),
            format!("GET /stores/{store}/labels/templates"),
        ]
    );
    assert_eq!(shop_reads(&a), Vec::<String>::new());
}

#[test]
fn l4_c4b_an_unmatched_product_has_its_cached_search_picture() {
    let a = connected();
    p2(&a);
    list_all(&a);
    run_pending(&a);
    assert_eq!(pictures(&a, &["2"]), only("2", search_file(&a)));
}

#[test]
fn l4_c4c_the_check_never_downloads_a_picture() {
    let a = connected();
    p2(&a);
    list_all(&a);
    assert_eq!(pictures(&a, &["2"]), HashMap::new());
    assert!(!search_file(&a).exists());
    run_pending(&a);
    assert_eq!(pictures(&a, &["2"]), only("2", search_file(&a)));
}

#[test]
fn l4_c4d_without_a_list_there_is_no_picture_and_no_call() {
    let a = connected();
    p2(&a);
    clear_reads(&a);
    assert_eq!(pictures(&a, &["2"]), HashMap::new());
    assert_eq!(shop_reads(&a), Vec::<String>::new());
    run_pending(&a);
    assert_eq!(a.fake_shop().image_fetches(), Vec::<(String, u32)>::new());
}

#[test]
fn l4_c4e_select_all_remembers_the_picture_address_too() {
    let a = connected();
    p2(&a);
    a.app.labels.select_all(LabelFilter::All, None).unwrap();
    check(&a, &["2"]);
    run_pending(&a);
    assert_eq!(pictures(&a, &["2"]), only("2", search_file(&a)));
}

#[test]
fn l4_c4f_test_connection_forgets_the_picture_addresses() {
    let a = connected();
    p2(&a);
    list_all(&a);
    run_pending(&a);
    assert!(search_file(&a).is_file());
    a.app.conversations.test_connection().unwrap();
    crate::l3_labels_check::connect(&a);
    std::fs::remove_file(search_file(&a)).unwrap();
    let fetches = a.fake_shop().image_fetches();
    assert_eq!(pictures(&a, &["2"]), HashMap::new());
    run_pending(&a);
    assert!(!search_file(&a).exists());
    assert_eq!(a.fake_shop().image_fetches(), fetches);
}

#[test]
fn l4_c4g_the_pictures_make_no_run_and_no_shop_write() {
    let a = connected();
    p1_with_picture(&a);
    p2(&a);
    pictures(&a, &["1"]);
    pictures(&a, &["2"]);
    list_all(&a);
    pictures(&a, &["2"]);
    run_pending(&a);
    pictures(&a, &["1", "2"]);
    a.app.labels.select_all(LabelFilter::All, None).unwrap();
    pictures(&a, &["2"]);
    a.app.conversations.test_connection().unwrap();
    crate::l3_labels_check::connect(&a);
    pictures(&a, &["1", "2"]);
    assert!(
        a.fake_pricing()
            .request_lines()
            .iter()
            .all(|line| !line.contains("/labels/runs"))
    );
    assert_eq!(a.fake_shop().mutation_ops(), Vec::<String>::new());
}
