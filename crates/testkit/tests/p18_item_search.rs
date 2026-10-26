use psm_core::{
    domain::{item::ItemOrigin, publish::RemoteItem},
    services::items::ItemsService,
};
use psm_testkit::TestApp;

#[test]
fn c3a_local_search_never_queries_shop_and_shop_search_returns_only_remote_hit() {
    let app = TestApp::with_fake_shop().unwrap();
    app.items()
        .create_local("SKU-1", "Local chair", "Chair", None)
        .unwrap();
    app.fake_shop()
        .state
        .lock()
        .unwrap()
        .products
        .push(RemoteItem {
            external_id: "shop-9".into(),
            title: "Shop chair".into(),
            sku: Some("SKU-9".into()),
            barcode: None,
            thumbnail_url: None,
            media: vec![],
        });
    let service =
        ItemsService::new(app.store(), app.clock.as_ref()).with_commerce(Some(app.fake_shop()));

    let local = service.search_local("sku").unwrap();
    assert_eq!(
        local
            .iter()
            .map(|item| item.sku.as_str())
            .collect::<Vec<_>>(),
        vec!["SKU-1"]
    );
    assert!(
        !app.fake_shop()
            .state
            .lock()
            .unwrap()
            .reads
            .iter()
            .any(|read| read == "item_find")
    );

    let shop = service.search_shop("sku").unwrap();
    assert_eq!(shop.len(), 1);
    assert_eq!(shop[0].sku, "SKU-9");
    assert_eq!(shop[0].id, 0);
    assert_eq!(shop[0].origin, ItemOrigin::Linked);
    assert_eq!(
        app.fake_shop()
            .state
            .lock()
            .unwrap()
            .reads
            .iter()
            .filter(|read| *read == "item_find")
            .count(),
        1
    );
}
