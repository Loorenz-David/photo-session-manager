//! P07 backend rows: item search (C5(c)), SKU proposal (C5(e), MC-22) and one session per item (C5(d)).
use psm_core::domain::item::ItemOrigin;
use psm_testkit::TestApp;

#[test]
fn c5c_search_returns_local_oak_item_only() {
    let app = TestApp::new().unwrap();
    app.items()
        .create_local("OAK-1", "Oak sideboard", "Sideboard", None)
        .unwrap();
    app.items()
        .create_local("TEAK-1", "Teak desk", "Desk", None)
        .unwrap();

    let found = app.items().search("oak").unwrap();

    let skus: Vec<&str> = found.iter().map(|item| item.sku.as_str()).collect();
    assert_eq!(skus, vec!["OAK-1"]);
    assert_eq!(found[0].title, "Oak sideboard");
    assert_eq!(found[0].origin, ItemOrigin::Local);
}

#[test]
fn c5e_next_sku_follows_largest_exact_prefix_number() {
    let app = TestApp::new().unwrap();
    for sku in ["VF-0419", "VF-0420", "vf-0999", "VF-12a"] {
        app.items().create_local(sku, sku, "Chair", None).unwrap();
    }

    assert_eq!(
        app.items().next_sku(Some("VF")).unwrap().as_deref(),
        Some("VF-0421")
    );
    assert_eq!(app.items().next_sku(None).unwrap(), None);
    assert_eq!(app.items().next_sku(Some("")).unwrap(), None);
}

#[test]
fn c5d_session_create_returns_the_existing_session_of_an_item() {
    let app = TestApp::new().unwrap();
    let (item, first) = app
        .items()
        .create_local("OAK-1", "Oak sideboard", "Sideboard", None)
        .unwrap();

    let again = app.sessions().create(item.id).unwrap();
    let third = app.sessions().create(item.id).unwrap();

    assert_eq!(again.session.id, first.session.id);
    assert_eq!(third.session.id, first.session.id);
    assert_eq!(
        app.store().session_for_item(item.id).unwrap(),
        Some(first.session.id)
    );
}
