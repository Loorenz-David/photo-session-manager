//! P67 (changelog 46 (3)–(5)): the item form shows the linked Shopify product's values, defaults
//! and saved values per item type, Fill defaults, and no product type chosen for the photographer.
//! Boundaries: `AttributesService` calls and the form they produce (`form_get`), the stored values,
//! and the calls the fake shop received. Row C4(a) (batch step memory) is a UI row: its test is in
//! `apps/desktop/src/features/form/__tests__/p67_form_values_ui.test.tsx`.

use psm_core::{
    domain::attribute::{AttributeGroup, Form, FormField, Preset, PresetField, Provenance},
    ports::commerce::CommerceAdapter,
    services::attributes::{AttributesService, RemoteRead},
};
use psm_integrations::commerce::shopify::{
    PollConfig, ShopifyAdapter, ShopifyConfig, adapter::description_html, transport::HttpReply,
};
use psm_testkit::{
    TestApp,
    clock::ManualClock,
    fake_transport::FakeTransport,
    fixtures::definitions::{INTEGRATION, shop_definitions},
};
use serde_json::{Value as Json, json};
use std::sync::Arc;

fn app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    service(&a)
        .import_definitions(INTEGRATION, shop_definitions())
        .unwrap();
    a
}

fn service(a: &TestApp) -> AttributesService<'_> {
    AttributesService::new(a.store(), a.clock.as_ref(), Some(INTEGRATION))
}

/// A product in the fake shop with a type, tags, a description and metafield values.
fn product(a: &TestApp, sku: &str, title: &str, kind: &str, values: Vec<(&str, Json)>) -> String {
    let id = a.fake_shop().add_product(sku, title, None);
    let mut shop = a.fake_shop().state.lock().unwrap();
    shop.products
        .iter_mut()
        .find(|p| p.external_id == id)
        .unwrap()
        .item_type = Some(kind.into());
    shop.values.insert(
        id.clone(),
        values.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
    );
    id
}

fn linked(a: &TestApp, external_id: &str) -> i64 {
    a.items()
        .with_commerce(Some(a.fake_shop()))
        .adopt(external_id)
        .unwrap()
        .id
}

fn field<'f>(form: &'f Form, key: &str) -> &'f FormField {
    form.groups
        .iter()
        .flat_map(|g| g.fields.iter())
        .find(|f| f.definition.key == key)
        .unwrap_or_else(|| panic!("{key} not in the form"))
}

/// The field's value and provenance, `None` when it has none.
fn shown(form: &Form, key: &str) -> Option<(Json, Provenance)> {
    field(form, key)
        .value
        .as_ref()
        .map(|v| (v.json().unwrap(), v.provenance))
}

fn product_reads(a: &TestApp) -> Vec<String> {
    a.fake_shop().state.lock().unwrap().reads.clone()
}

#[test]
fn c1a_linked_item_form_shows_the_products_values_from_shopify() {
    let a = app();
    let id = product(
        &a,
        "SOFA-1",
        "Velvet sofa",
        "Sofa",
        vec![
            ("custom.color_furniture", json!("Green")),
            ("custom.country", json!("Sweden")),
            ("custom.condition_scratch", json!("Minor")),
        ],
    );
    {
        let mut shop = a.fake_shop().state.lock().unwrap();
        shop.tags
            .insert(id.clone(), vec!["velvet".into(), "green".into()]);
        shop.descriptions.insert(
            id.clone(),
            description_html("Deep seat & soft arms.\n\nNew velvet."),
        );
    }
    let item_id = linked(&a, &id);
    a.fake_shop().state.lock().unwrap().reads.clear();

    let read = service(&a)
        .read_integration_values(Some(a.fake_shop()), item_id)
        .unwrap();
    assert_eq!(read, RemoteRead::Read);
    // One request for the whole product, and nothing written to the shop.
    assert_eq!(product_reads(&a), vec!["product_values".to_owned()]);
    assert!(a.fake_shop().mutation_ops().is_empty());

    let form = service(&a).form_get(item_id).unwrap();
    let from_shopify = Provenance::Integration;
    assert_eq!(
        shown(&form, "product.title"),
        Some((json!("Velvet sofa"), from_shopify))
    );
    assert_eq!(
        shown(&form, "product.description"),
        Some((json!("Deep seat & soft arms.\n\nNew velvet."), from_shopify))
    );
    assert_eq!(
        shown(&form, "product.tags"),
        Some((json!(["velvet", "green"]), from_shopify))
    );
    assert_eq!(
        shown(&form, "custom.color_furniture"),
        Some((json!("Green"), from_shopify))
    );
    assert_eq!(
        shown(&form, "custom.country"),
        Some((json!("Sweden"), from_shopify))
    );
    assert_eq!(
        shown(&form, "custom.condition_scratch"),
        Some((json!("Minor"), from_shopify))
    );
    assert!(form.item_type_from_integration);
    assert_eq!(a.store().item(item_id).unwrap().item_type, "Sofa");
}

/// The real Shopify adapter: the form's values come from one GraphQL request per product.
#[test]
fn c1a_shopify_adapter_reads_the_form_values_in_one_request() {
    let fake = Arc::new(FakeTransport::new());
    fake.on_form_always(
        "https://example.myshopify.com/admin/oauth/access_token",
        HttpReply::new(
            200,
            json!({"access_token":"test-token","expires_in":3600,"scope":"read_products"}),
        ),
    );
    fake.on_graphql_always(
        "ProductValues",
        None,
        HttpReply::new(
            200,
            json!({"data":{"product":{
                "id":"gid://shopify/Product/1","title":"Velvet sofa","productType":"Sofa",
                "tags":["velvet","green"],
                "descriptionHtml":"<p>Deep seat &amp; soft arms.</p><p>New velvet.</p>",
                "metafields":{"pageInfo":{"hasNextPage":false,"endCursor":null},"nodes":[
                    {"namespace":"custom","key":"color_furniture","type":"single_line_text_field","value":"Green"},
                    {"namespace":"custom","key":"condition_scratch","type":"single_line_text_field","value":"Minor"},
                    {"namespace":"custom","key":"damage_report","type":"list.file_reference","value":"[\"gid://shopify/MediaImage/9\"]"}
                ]}
            }}}),
        ),
    );
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
    let values = adapter.product_values("gid://shopify/Product/1").unwrap();
    assert_eq!(fake.graphql_count(), 1);
    assert_eq!(values.title, "Velvet sofa");
    assert_eq!(values.item_type.as_deref(), Some("Sofa"));
    assert_eq!(values.tags, vec!["velvet", "green"]);
    assert_eq!(
        values.description.as_deref(),
        Some("Deep seat & soft arms.\n\nNew velvet.")
    );
    assert_eq!(
        values.attributes,
        vec![
            ("custom.color_furniture".to_owned(), json!("Green")),
            ("custom.condition_scratch".to_owned(), json!("Minor")),
        ]
    );
}

#[test]
fn c1b_local_edit_wins_on_reopen_and_a_failed_read_keeps_the_form_editable() {
    let a = app();
    let id = product(
        &a,
        "SOFA-1",
        "Velvet sofa",
        "Sofa",
        vec![
            ("custom.color_furniture", json!("Green")),
            ("custom.country", json!("Sweden")),
        ],
    );
    let item_id = linked(&a, &id);
    let attrs = service(&a);
    attrs
        .read_integration_values(Some(a.fake_shop()), item_id)
        .unwrap();
    attrs
        .set_value(
            item_id,
            "custom.color_furniture",
            json!("Blue"),
            Provenance::Human,
        )
        .unwrap();
    a.fake_shop().state.lock().unwrap().values.insert(
        id.clone(),
        vec![
            ("custom.color_furniture".into(), json!("Red")),
            ("custom.country".into(), json!("Denmark")),
        ],
    );

    // Reopen: the edited field keeps the photographer's value; the untouched one follows the shop.
    assert_eq!(
        attrs
            .read_integration_values(Some(a.fake_shop()), item_id)
            .unwrap(),
        RemoteRead::Read
    );
    let form = attrs.form_get(item_id).unwrap();
    assert_eq!(
        shown(&form, "custom.color_furniture"),
        Some((json!("Blue"), Provenance::Human))
    );
    assert_eq!(
        shown(&form, "custom.country"),
        Some((json!("Denmark"), Provenance::Integration))
    );

    // The product cannot be read: the local values stay and the form can still be edited.
    a.fake_shop().state.lock().unwrap().unreachable = true;
    let failed = attrs
        .read_integration_values(Some(a.fake_shop()), item_id)
        .unwrap();
    assert!(matches!(failed, RemoteRead::Failed(_)), "{failed:?}");
    let form = attrs.form_get(item_id).unwrap();
    assert_eq!(
        shown(&form, "custom.color_furniture"),
        Some((json!("Blue"), Provenance::Human))
    );
    assert_eq!(
        shown(&form, "custom.country"),
        Some((json!("Denmark"), Provenance::Integration))
    );
    attrs
        .set_value(
            item_id,
            "custom.country",
            json!("Norway"),
            Provenance::Human,
        )
        .unwrap();
    assert_eq!(
        shown(&attrs.form_get(item_id).unwrap(), "custom.country"),
        Some((json!("Norway"), Provenance::Human))
    );
}

#[test]
fn c2a_save_as_default_fills_new_items_of_that_type_and_empty_linked_values() {
    let a = app();
    let attrs = service(&a);
    attrs
        .default_set("Sofa", "custom.color_furniture", Some(json!("Oak")))
        .unwrap();

    let sofa = a
        .items()
        .create_local("SOFA-2", "Sofa", "Sofa", None)
        .unwrap()
        .0;
    assert_eq!(
        shown(&attrs.form_get(sofa.id).unwrap(), "custom.color_furniture"),
        Some((json!("Oak"), Provenance::Default))
    );

    let chair = a
        .items()
        .create_local("CHAIR-2", "Chair", "Chair", None)
        .unwrap()
        .0;
    assert_eq!(
        shown(&attrs.form_get(chair.id).unwrap(), "custom.color_furniture"),
        None
    );

    let empty = product(&a, "SOFA-3", "Linen sofa", "Sofa", vec![]);
    let empty_id = linked(&a, &empty);
    a.sessions().create(empty_id).unwrap();
    attrs
        .read_integration_values(Some(a.fake_shop()), empty_id)
        .unwrap();
    assert_eq!(
        shown(&attrs.form_get(empty_id).unwrap(), "custom.color_furniture"),
        Some((json!("Oak"), Provenance::Default))
    );

    let set = product(
        &a,
        "SOFA-4",
        "Walnut sofa",
        "Sofa",
        vec![("custom.color_furniture", json!("Walnut"))],
    );
    let set_id = linked(&a, &set);
    a.sessions().create(set_id).unwrap();
    attrs
        .read_integration_values(Some(a.fake_shop()), set_id)
        .unwrap();
    assert_eq!(
        shown(&attrs.form_get(set_id).unwrap(), "custom.color_furniture"),
        Some((json!("Walnut"), Provenance::Integration))
    );
}

#[test]
fn c2b_saved_values_most_recent_first_pick_remove_and_customize_shows_the_default() {
    let a = app();
    let attrs = service(&a);
    let sofa = a
        .items()
        .create_local("SOFA-5", "Sofa", "Sofa", None)
        .unwrap()
        .0;
    attrs
        .saved_value_add("Sofa", "custom.country", json!("Sweden"))
        .unwrap();
    a.clock.advance(1);
    attrs
        .saved_value_add("Sofa", "custom.country", json!("Denmark"))
        .unwrap();
    attrs
        .saved_value_add("Sofa", "custom.color_furniture", json!("Oak"))
        .unwrap();
    a.clock.advance(1);
    attrs
        .saved_value_add("Sofa", "custom.color_furniture", json!("Walnut"))
        .unwrap();

    let form = attrs.form_get(sofa.id).unwrap();
    assert_eq!(
        field(&form, "custom.country").saved,
        vec![json!("Denmark"), json!("Sweden")]
    );
    assert_eq!(
        field(&form, "custom.color_furniture").saved,
        vec![json!("Walnut"), json!("Oak")]
    );

    // Picking a saved value fills the field as the photographer's own value.
    attrs
        .set_value(
            sofa.id,
            "custom.country",
            json!("Sweden"),
            Provenance::Human,
        )
        .unwrap();
    attrs
        .saved_value_remove("Sofa", "custom.color_furniture", json!("Oak"))
        .unwrap();
    let form = attrs.form_get(sofa.id).unwrap();
    assert_eq!(
        shown(&form, "custom.country"),
        Some((json!("Sweden"), Provenance::Human))
    );
    assert_eq!(
        field(&form, "custom.color_furniture").saved,
        vec![json!("Walnut")]
    );

    // Save as default from the form is the default Customize fields shows (same data).
    attrs
        .default_set("Sofa", "custom.country", Some(json!("Denmark")))
        .unwrap();
    let customize = attrs.presets_get("Sofa", AttributeGroup::Details).unwrap();
    assert_eq!(
        customize
            .layout
            .iter()
            .find(|f| f.key == "custom.country")
            .unwrap()
            .default,
        Some(json!("Denmark"))
    );
    assert_eq!(
        field(&attrs.form_get(sofa.id).unwrap(), "custom.country").default,
        Some(json!("Denmark"))
    );
}

#[test]
fn c2c_fill_defaults_fills_only_the_empty_fields_that_have_a_default() {
    let a = app();
    let attrs = service(&a);
    let sofa = a
        .items()
        .create_local("SOFA-6", "Sofa", "Sofa", None)
        .unwrap()
        .0;
    let keys = [
        "custom.color_furniture",
        "custom.designer",
        "custom.manufacturer",
        "custom.shape",
        "custom.mark_stamp",
        "custom.country",
    ];
    let defaults: [Option<Json>; 6] = [
        Some(json!("Oak")),
        Some(json!("Wegner")),
        Some(json!("Fritz Hansen")),
        None,
        None,
        Some(json!("Denmark")),
    ];
    attrs
        .presets_set(Preset {
            item_type: "Sofa".into(),
            group: AttributeGroup::Details,
            layout: keys
                .iter()
                .zip(defaults)
                .map(|(key, default)| PresetField {
                    key: (*key).into(),
                    default,
                })
                .collect(),
        })
        .unwrap();
    attrs
        .set_value(
            sofa.id,
            "custom.country",
            json!("Sweden"),
            Provenance::Human,
        )
        .unwrap();

    assert_eq!(attrs.fill_defaults(sofa.id).unwrap(), 3);
    let form = attrs.form_get(sofa.id).unwrap();
    assert_eq!(
        shown(&form, "custom.color_furniture"),
        Some((json!("Oak"), Provenance::Default))
    );
    assert_eq!(
        shown(&form, "custom.designer"),
        Some((json!("Wegner"), Provenance::Default))
    );
    assert_eq!(
        shown(&form, "custom.manufacturer"),
        Some((json!("Fritz Hansen"), Provenance::Default))
    );
    assert_eq!(shown(&form, "custom.shape"), None);
    assert_eq!(shown(&form, "custom.mark_stamp"), None);
    assert_eq!(
        shown(&form, "custom.country"),
        Some((json!("Sweden"), Provenance::Human))
    );
}

#[test]
fn c3a_new_item_without_a_type_keeps_an_empty_type_when_its_form_opens() {
    let a = app();
    // The shop has types to offer; none may be chosen for the photographer.
    product(&a, "SOFA-7", "Sofa", "Sofa", vec![]);
    let item = a
        .items()
        .create_local("LAMP-1", "Lamp", "", None)
        .unwrap()
        .0;
    a.sessions().create(item.id).unwrap();
    a.fake_shop().state.lock().unwrap().reads.clear();
    let attrs = service(&a);

    // Opening the form: the read finds no linked product, the form is fetched.
    assert_eq!(
        attrs
            .read_integration_values(Some(a.fake_shop()), item.id)
            .unwrap(),
        RemoteRead::NotLinked
    );
    let form = attrs.form_get(item.id).unwrap();

    assert_eq!(a.store().item(item.id).unwrap().item_type, "");
    assert!(!form.item_type_from_integration);
    assert!(attrs.values(item.id).unwrap().is_empty());
    assert!(product_reads(&a).is_empty());
}
