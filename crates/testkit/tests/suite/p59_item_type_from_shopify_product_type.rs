use psm_core::{
    domain::attribute::{AttributeGroup, Preset, PresetField, Provenance},
    ports::{clock::Clock, commerce::CommerceAdapter},
    services::attributes::AttributesService,
};
use psm_integrations::commerce::shopify::{
    PollConfig, ShopifyAdapter, ShopifyConfig, transport::HttpReply,
};
use psm_testkit::ConfirmAsShown;
use psm_testkit::{
    TestApp,
    clock::ManualClock,
    fake_transport::FakeTransport,
    fixtures::{
        definitions::{INTEGRATION, shop_definitions},
        shopify::fixture,
    },
};
use serde_json::{Value as Json, json};
use std::sync::Arc;

fn app() -> TestApp {
    TestApp::with_fake_shop().unwrap()
}

fn set_remote_type(a: &TestApp, id: &str, kind: &str) {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .products
        .iter_mut()
        .find(|p| p.external_id == id)
        .unwrap()
        .item_type = Some(kind.into());
}

fn shop_adapter() -> (ShopifyAdapter, Arc<FakeTransport>) {
    let fake = Arc::new(FakeTransport::new());
    fake.on_form_always("https://example.myshopify.com/admin/oauth/access_token",
        HttpReply::new(200, json!({"access_token":"test-token","expires_in":3600,"scope":"read_products,write_products"})));
    let adapter = ShopifyAdapter::new(
        ShopifyConfig {
            shop_domain: "example.myshopify.com".into(),
            api_version: "2026-10".into(),
            client_id: "client".into(),
            client_secret: "secret".into(),
            condition_photos_key: "custom.damage_report".into(),
        },
        fake.clone(),
        Arc::new(ManualClock::new(1000)),
        PollConfig {
            interval_ms: 0,
            attempts: 1,
            retries: 1,
            retry_wait_ms: 0,
        },
    );
    (adapter, fake)
}

#[test]
fn c1a_found_item_takes_product_type() {
    let (adapter, fake) = shop_adapter();
    let mut search = fixture("sv6_search_immediate").reply();
    search.body["data"]["productVariants"]["nodes"][0]["product"]["productType"] = json!("Lamps");
    fake.on_graphql_always("VariantSearch", None, search);
    let remote = adapter.item_search("PSM-PROBE-SKU-1").unwrap();
    assert_eq!(remote[0].item_type.as_deref(), Some("Lamps"));
    let a = app();
    let id = a.fake_shop().add_product("LAMP-1", "Floor lamp", None);
    set_remote_type(&a, &id, "Lamps");
    let hit = a
        .items()
        .with_commerce(Some(a.fake_shop()))
        .search_shop("LAMP-1")
        .unwrap();
    assert_eq!(hit[0].item_type, "Lamps");
    let item = a
        .items()
        .with_commerce(Some(a.fake_shop()))
        .adopt(&id)
        .unwrap();
    assert_eq!(item.item_type, "Lamps");
}

#[test]
fn c1b_refresh_fills_linked_empty_type() {
    let a = app();
    let id = a.fake_shop().add_product("CHAIR-1", "Chair", None);
    let item = a
        .items()
        .with_commerce(Some(a.fake_shop()))
        .adopt(&id)
        .unwrap();
    assert_eq!(item.item_type, "");
    set_remote_type(&a, &id, "Chairs");
    assert_eq!(
        a.items()
            .with_commerce(Some(a.fake_shop()))
            .refresh(item.id)
            .unwrap()
            .item_type,
        "Chairs"
    );
    set_remote_type(&a, &id, "Tables");
    assert_eq!(
        a.items()
            .with_commerce(Some(a.fake_shop()))
            .refresh(item.id)
            .unwrap()
            .item_type,
        "Tables"
    );
    a.items().set_type(item.id, "Lamps").unwrap();
    set_remote_type(&a, &id, "Desks");
    assert_eq!(
        a.items()
            .with_commerce(Some(a.fake_shop()))
            .refresh(item.id)
            .unwrap()
            .item_type,
        "Lamps"
    );
}

#[test]
fn c2a_shopify_adapter_returns_sorted_distinct_nonblank_product_types() {
    let (adapter, fake) = shop_adapter();
    fake.on_graphql_always(
        "ProductTypes",
        None,
        HttpReply::new(
            200,
            json!({"data":{"productTypes":{"nodes":["Lamps", "Chairs", "", "Lamps"],"pageInfo":{"hasNextPage":false,"endCursor":null}}}}),
        ),
    );

    assert_eq!(adapter.item_types().unwrap(), vec!["Chairs", "Lamps"]);
}

#[test]
fn c4a_empty_type_preset_is_default_and_own_save_is_independent() {
    let a = app();
    let attrs = AttributesService::new(a.store(), a.clock.as_ref(), Some(INTEGRATION));
    attrs
        .import_definitions(INTEGRATION, shop_definitions())
        .unwrap();
    attrs
        .presets_set(Preset {
            item_type: "".into(),
            group: AttributeGroup::Details,
            layout: vec![PresetField {
                key: "custom.color_furniture".into(),
                default: Some(json!("Oak")),
            }],
        })
        .unwrap();
    let lamps = a
        .items()
        .create_local("LAMP-2", "Lamp", "Lamps", None)
        .unwrap()
        .0;
    let form = attrs.form_get(lamps.id).unwrap();
    let details = form
        .groups
        .iter()
        .find(|g| g.group == AttributeGroup::Details)
        .unwrap();
    assert_eq!(details.fields[0].value.as_ref().unwrap().value, "\"Oak\"");
    let mut own = attrs.presets_get("Lamps", AttributeGroup::Details).unwrap();
    assert_eq!(own.item_type, "Lamps");
    own.layout[0].default = Some(json!("Blue"));
    attrs.presets_set(own).unwrap();
    let chairs = a
        .items()
        .create_local("CHAIR-2", "Chair", "Chairs", None)
        .unwrap()
        .0;
    let chair_form = attrs.form_get(chairs.id).unwrap();
    let chair_details = chair_form
        .groups
        .iter()
        .find(|g| g.group == AttributeGroup::Details)
        .unwrap();
    assert_eq!(
        chair_details.fields[0].value.as_ref().unwrap().value,
        "\"Oak\""
    );
    assert_eq!(
        attrs
            .presets_get("Lamps", AttributeGroup::Details)
            .unwrap()
            .layout[0]
            .default,
        Some(json!("Blue"))
    );
    let second = a
        .items()
        .create_local("LAMP-5", "Lamp", "Lamps", None)
        .unwrap()
        .0;
    assert_eq!(
        attrs.form_get(second.id).unwrap().groups[0].fields[0]
            .value
            .as_ref()
            .unwrap()
            .value,
        "\"Blue\""
    );
    a.items().set_type(second.id, "Chairs").unwrap();
    assert_eq!(
        attrs.form_get(second.id).unwrap().groups[0].fields[0]
            .value
            .as_ref()
            .unwrap()
            .value,
        "\"Oak\""
    );
    attrs
        .set_value(
            second.id,
            "custom.color_furniture",
            json!("Walnut"),
            Provenance::Human,
        )
        .unwrap();
    a.items().set_type(second.id, "Lamps").unwrap();
    assert_eq!(
        attrs.form_get(second.id).unwrap().groups[0].fields[0]
            .value
            .as_ref()
            .unwrap()
            .value,
        "\"Walnut\""
    );
}

#[test]
fn c3a_confirmed_publish_creates_and_updates_product_type() {
    let (adapter, fake) = shop_adapter();
    fake.replay_any(&fixture("sv7_create"));
    let confirmed = TestApp::new().unwrap().confirmed_run().unwrap();
    adapter
        .item_create(&confirmed, "TEST-1", None, "Lamp", "Lamps")
        .unwrap();
    assert_eq!(
        fake.graphql("ProductSet")[0].1["input"]["productType"],
        json!("Lamps")
    );
    fake.on_graphql_always(
        "ProductUpdateTags",
        None,
        HttpReply::new(
            200,
            json!({"data":{"productUpdate":{"product":{"id":"product-1"},"userErrors":[]}}}),
        ),
    );
    adapter
        .update_item_type(&confirmed, "product-1", "Chairs")
        .unwrap();
    assert_eq!(
        fake.graphql("ProductUpdateTags")[0].1["product"]["productType"],
        json!("Chairs")
    );
    let (empty_adapter, empty_fake) = shop_adapter();
    empty_fake.replay_any(&fixture("sv7_create"));
    empty_adapter
        .item_create(&confirmed, "TEST-2", None, "Untyped", "")
        .unwrap();
    assert!(
        empty_fake.graphql("ProductSet")[0].1["input"]
            .get("productType")
            .is_none()
    );
    let a = app();
    let (item, session) = a
        .items()
        .create_local("LAMP-3", "Lamp", "Lamps", None)
        .unwrap();
    let publish = a.app.publish.as_ref().unwrap();
    publish
        .confirm_as_shown(&[session.session.id], "Publish Lamp")
        .unwrap();
    a.items().set_type(item.id, "Chairs").unwrap();
    a.app.publish_jobs.tick(a.clock.now_ms()).unwrap();
    let remote = a.fake_shop().item_search("LAMP-3").unwrap();
    assert_eq!(remote[0].item_type.as_deref(), Some("Lamps"));
    publish
        .confirm_as_shown(&[session.session.id], "Publish Chair")
        .unwrap();
    a.app.publish_jobs.tick(a.clock.now_ms()).unwrap();
    assert_eq!(
        a.fake_shop().item_search("LAMP-3").unwrap()[0]
            .item_type
            .as_deref(),
        Some("Chairs")
    );
    let (_, empty_session) = a
        .items()
        .create_local("LAMP-4", "Untyped", "", None)
        .unwrap();
    publish
        .confirm_as_shown(&[empty_session.session.id], "Publish untyped")
        .unwrap();
    a.app.publish_jobs.tick(a.clock.now_ms()).unwrap();
    assert_eq!(
        a.fake_shop().item_search("LAMP-4").unwrap()[0].item_type,
        None
    );
}

/// A29 (owner 2026-11-20): an adopted product's title is the shop's (its title base), so the
/// writer treats it as the shop's text, never as one the photographer typed.
#[test]
fn a29_adopt_records_the_shop_title_as_the_title_base() {
    let a = app();
    let id = a.fake_shop().add_product("Ch1-201126", "Ch1-201126", None);
    let item = a
        .items()
        .with_commerce(Some(a.fake_shop()))
        .adopt(&id)
        .unwrap();
    assert_eq!(item.title, "Ch1-201126");
    let mut base = None;
    a.store()
        .transaction(&mut |tx| {
            base = tx.item_title_base(item.id, "shopify")?;
            Ok(())
        })
        .unwrap();
    assert_eq!(base.as_deref(), Some("Ch1-201126"));
}

/// A29: the handle read reply for a product with `handle` and SKU `sku`.
fn handle_reply(handle: &str, sku: &str) -> HttpReply {
    HttpReply::new(
        200,
        json!({"data": {"product": {"id": "product-1", "handle": handle,
            "variants": {"nodes": [{"sku": sku}]}}}}),
    )
}

fn title_reply(errors: Json) -> HttpReply {
    HttpReply::new(
        200,
        json!({"data": {"productUpdate": {"product": {"id": "product-1"}, "userErrors": errors}}}),
    )
}

/// A29 (owner 2026-11-20): a title update moves a handle Shopify made from the SKU to the new
/// title's handle, with a 301 redirect from the old address.
#[test]
fn a29_a_sku_handle_moves_with_the_title_and_redirects() {
    let (adapter, fake) = shop_adapter();
    let confirmed = TestApp::new().unwrap().confirmed_run().unwrap();
    fake.on_graphql_always(
        "ProductHandle",
        None,
        handle_reply("ch3-201126", "Ch3-201126"),
    );
    fake.on_graphql_always("ProductUpdateTitle", None, title_reply(json!([])));
    adapter
        .update_title(
            &confirmed,
            "product-1",
            "Henning Kjærnulf Teak Dining Chair for Sorø Stolefabrik",
        )
        .unwrap();
    let sent = fake.graphql("ProductUpdateTitle");
    assert_eq!(sent.len(), 1);
    assert_eq!(
        sent[0].1["product"],
        json!({
            "id": "product-1",
            "title": "Henning Kjærnulf Teak Dining Chair for Sorø Stolefabrik",
            "handle": "henning-kjaernulf-teak-dining-chair-for-soro-stolefabrik",
            "redirectNewHandle": true,
        })
    );
}

/// A29: a handle Shopify reports as taken is tried once more with the SKU appended.
#[test]
fn a29_a_taken_handle_is_tried_again_with_the_sku() {
    let (adapter, fake) = shop_adapter();
    let confirmed = TestApp::new().unwrap().confirmed_run().unwrap();
    fake.on_graphql_always(
        "ProductHandle",
        None,
        handle_reply("ch1-201126-1", "Ch1-201126"),
    );
    fake.on_graphql(
        "ProductUpdateTitle",
        None,
        vec![
            title_reply(json!([{"field": ["handle"], "message": "Handle has already been taken"}])),
            title_reply(json!([])),
        ],
    );
    adapter
        .update_title(
            &confirmed,
            "product-1",
            "Teak Stool with Tan Brown Leather Seat",
        )
        .unwrap();
    let handles: Vec<Json> = fake
        .graphql("ProductUpdateTitle")
        .iter()
        .map(|(_, v)| v["product"]["handle"].clone())
        .collect();
    assert_eq!(
        handles,
        [
            json!("teak-stool-with-tan-brown-leather-seat"),
            json!("teak-stool-with-tan-brown-leather-seat-ch1-201126"),
        ]
    );
}

/// A29: a customised handle is never changed; only the title is sent.
#[test]
fn a29_a_customised_handle_is_left_alone() {
    let (adapter, fake) = shop_adapter();
    let confirmed = TestApp::new().unwrap().confirmed_run().unwrap();
    fake.on_graphql_always(
        "ProductHandle",
        None,
        handle_reply("danish-teak-chair", "Ch2-201126"),
    );
    fake.on_graphql_always("ProductUpdateTitle", None, title_reply(json!([])));
    adapter
        .update_title(
            &confirmed,
            "product-1",
            "Santos Rosewood Ladder-Back Dining Chair",
        )
        .unwrap();
    assert_eq!(
        fake.graphql("ProductUpdateTitle")[0].1["product"],
        json!({"id": "product-1", "title": "Santos Rosewood Ladder-Back Dining Chair"})
    );
}

/// A29: another user error is the provider's error, not a second try.
#[test]
fn a29_a_title_error_is_not_retried() {
    let (adapter, fake) = shop_adapter();
    let confirmed = TestApp::new().unwrap().confirmed_run().unwrap();
    fake.on_graphql_always(
        "ProductHandle",
        None,
        handle_reply("ch9-201126", "Ch9-201126"),
    );
    fake.on_graphql_always(
        "ProductUpdateTitle",
        None,
        title_reply(json!([{"field": ["title"], "message": "Title is too long"}])),
    );
    assert!(
        adapter
            .update_title(&confirmed, "product-1", "Teak and Beech Dining Chair")
            .is_err()
    );
    assert_eq!(fake.graphql("ProductUpdateTitle").len(), 1);
}

#[test]
fn a29_handleize_and_sku_handle() {
    use psm_integrations::commerce::shopify::adapter::{handleize, sku_handle};
    assert_eq!(
        handleize("Ib Kofod-Larsen Teak Dining Chair"),
        "ib-kofod-larsen-teak-dining-chair"
    );
    assert_eq!(
        handleize("Aksel Bender Madsen & Ejner Larsen Teak Armchair"),
        "aksel-bender-madsen-ejner-larsen-teak-armchair"
    );
    assert_eq!(handleize("Fåtölj i teak, Säter"), "fatolj-i-teak-sater");
    assert_eq!(handleize("  Ch1-201126 "), "ch1-201126");
    assert!(sku_handle("ch1-201126", "Ch1-201126"));
    assert!(sku_handle("ch1-201126-2", "Ch1-201126"));
    assert!(!sku_handle("ch1-201126-teak", "Ch1-201126"));
    assert!(!sku_handle("teak-chair", "Ch1-201126"));
    assert!(!sku_handle("", ""));
}
