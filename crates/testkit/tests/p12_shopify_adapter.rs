//! P12 behaviour tests: the real `ShopifyAdapter` over `FakeTransport`, replaying the sanitized
//! fixtures the grounding probe captured from the test store (`shopify_admin.md`, SV-0…SV-7).
//! One test per criterion row of `plans/P12_shopify_adapter.md`. No network.

use psm_core::{
    Error,
    domain::attribute::{AttributeGroup, CoreType, Definition},
    domain::publish::{
        ConfirmedRun, DefinitionLink, RemoteIdentity, RemoteMedia, RemoteMediaStatus, StagedTarget,
        WriteValue,
    },
    mechanisms::values::{SEVERITY_SET, canonicalize},
    ports::commerce::CommerceAdapter,
};
use psm_integrations::commerce::shopify::{
    PollConfig, ShopifyAdapter, ShopifyConfig,
    adapter::description_html,
    transport::HttpReply,
    type_map::{self, DIMENSION_UNITS, VOLUME_UNITS, WEIGHT_UNITS},
};
use psm_testkit::{
    TestApp,
    clock::ManualClock,
    fake_transport::{FakeTransport, Recorded},
    fixtures::shopify::{Fixture, fixture},
};
use serde_json::{Value as Json, json};
use std::sync::Arc;

const TOKEN_URL: &str = "https://example.myshopify.com/admin/oauth/access_token";
const CLIENT_ID: &str = "psm-test-client-id";
const SECRET: &str = "SECRET-XYZ";
// The plan's tokens `T1`/`T2`, made distinctive so a substring check cannot hit fixture text
// (a fixture date such as `2026-10-26T10:00:00Z` contains "T1").
const T1: &str = "T1-psm-access-token";
const T2: &str = "T2-psm-access-token";
const PRODUCT: &str = "gid://shopify/Product/15444448280729";
const PROBE_SKU: &str = "PSM-PROBE-SKU-1";
const PROBE_BARCODE: &str = "4006381333931";
const LIFETIME_S: i64 = 86_399; // SV-0 `expires_in`

struct Shop {
    fake: Arc<FakeTransport>,
    clock: Arc<ManualClock>,
    adapter: ShopifyAdapter,
}

fn token_reply(token: &str) -> HttpReply {
    let mut reply = fixture("sv0_token").reply();
    reply.body["access_token"] = json!(token);
    reply
}

fn poll() -> PollConfig {
    PollConfig {
        interval_ms: 0,
        attempts: 5,
        retries: 3,
        retry_wait_ms: 0,
    }
}

/// An adapter for the test store with `condition_photos_key`; the token endpoint is not primed.
fn bare_shop(condition_photos_key: &str) -> Shop {
    let fake = Arc::new(FakeTransport::new());
    let clock = Arc::new(ManualClock::new(1_000_000));
    let adapter = ShopifyAdapter::new(
        ShopifyConfig {
            shop_domain: "example.myshopify.com".into(),
            api_version: "2026-10".into(),
            client_id: CLIENT_ID.into(),
            client_secret: SECRET.into(),
            condition_photos_key: condition_photos_key.into(),
        },
        fake.clone(),
        clock.clone(),
        poll(),
    );
    Shop {
        fake,
        clock,
        adapter,
    }
}

/// An adapter whose token endpoint always answers the SV-0 token reply with `T1`.
fn shop() -> Shop {
    let shop = bare_shop("custom.damage_report");
    shop.fake.on_form_always(TOKEN_URL, token_reply(T1));
    shop
}

fn run() -> (TestApp, ConfirmedRun) {
    let app = TestApp::new().unwrap();
    let run = app.confirmed_run().unwrap();
    (app, run)
}

fn schema_of(shop: &Shop) -> Vec<(Definition, DefinitionLink)> {
    shop.fake.replay(&fixture("sv1_definitions_p1"));
    shop.fake.replay(&fixture("sv1_definitions_p2"));
    shop.adapter.attributes_schema().unwrap()
}

fn find<'a>(
    schema: &'a [(Definition, DefinitionLink)],
    key: &str,
) -> &'a (Definition, DefinitionLink) {
    schema
        .iter()
        .find(|(d, _)| d.key == key)
        .unwrap_or_else(|| panic!("no definition {key}"))
}

// ---------------------------------------------------------------------------------------------
// C1 item_search (MC-11 prepare)

#[test]
fn c1a_search_no_match_is_empty() {
    let shop = shop();
    shop.fake.replay(&fixture("sv6_search_none"));
    assert_eq!(
        shop.adapter.item_search("PSM-PROBE-NO-SUCH-SKU").unwrap(),
        vec![]
    );
}

#[test]
fn c1b_search_one_match() {
    let shop = shop();
    shop.fake.replay(&fixture("sv6_search_immediate"));
    let items = shop.adapter.item_search(PROBE_SKU).unwrap();
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item.external_id, PRODUCT);
    assert_eq!(
        item.title,
        "PSM-PROBE product (Session Manager grounding probe)"
    );
    assert_eq!(item.sku.as_deref(), Some(PROBE_SKU));
    assert_eq!(item.barcode.as_deref(), Some(PROBE_BARCODE));
    assert_eq!(item.thumbnail_url, None); // the fixture product has no featured media yet
    assert!(item.media.is_empty());
}

#[test]
fn c1c_search_two_products_returns_both() {
    let shop = shop();
    shop.fake.replay(&fixture("sv6_search_two_products"));
    let items = shop.adapter.item_search(PROBE_SKU).unwrap();
    let ids: Vec<&str> = items.iter().map(|i| i.external_id.as_str()).collect();
    assert_eq!(ids, vec![PRODUCT, "gid://shopify/Product/15444448289999"]);
}

#[test]
fn c1d_search_is_byte_exact_after_nfc() {
    // SV-6: Shopify's search is case-insensitive; the other-case capture returns the variant whose
    // SKU is upper case. The adapter compares byte for byte after NFC, so nothing matches.
    let shop = shop();
    shop.fake.replay(&fixture("sv6_search_other_case"));
    assert_eq!(shop.adapter.item_search("psm-probe-sku-1").unwrap(), vec![]);
}

// ---------------------------------------------------------------------------------------------
// C2 item_get

#[test]
fn c2a_item_get_media_in_gallery_order() {
    let shop = shop();
    shop.fake.replay(&fixture("sv2_item_get"));
    let item = shop.adapter.item_get(PRODUCT).unwrap();
    let media = |id: &str, filename: &str, position: i64| RemoteMedia {
        id: id.into(),
        filename: Some(filename.into()),
        status: RemoteMediaStatus::Ready,
        position,
    };
    assert_eq!(
        item.media,
        vec![
            media(
                "gid://shopify/MediaImage/46644767260825",
                "psm-probe-ab12cd34ef56.jpg",
                1
            ),
            media(
                "gid://shopify/MediaImage/46644767457433",
                "psm-probe-ab12cd34ef56_4fffd4cd-17a1-449a-92f7-2e862798500b.jpg",
                2
            ),
            media(
                "gid://shopify/MediaImage/46644767686809",
                "psm-probe-0123456789ab.jpg",
                3
            ),
        ]
    );
}

// ---------------------------------------------------------------------------------------------
// C3 identity, create, mutation token

#[test]
fn c3x_set_identity_writes_only_the_sku() {
    let shop = shop();
    let (_app, run) = run();
    // Before: the SV-7 read-back (SKU PSM-PROBE-SKU-1, barcode 4006381333931).
    shop.fake.replay(&fixture("sv7_create_readback"));
    shop.fake.replay_any(&fixture("sv7_identity_sku_only"));
    let identity = shop
        .adapter
        .item_set_identity(&run, PRODUCT, "PSM-PROBE-SKU-2", None)
        .unwrap();
    let mutations = shop.fake.graphql("ItemSetIdentity");
    assert_eq!(mutations.len(), 1);
    let variant = &mutations[0].1["variants"][0];
    assert_eq!(variant["inventoryItem"]["sku"], json!("PSM-PROBE-SKU-2"));
    assert!(
        variant.get("barcodes").is_none(),
        "no barcode sent: {variant}"
    );
    assert_eq!(
        identity,
        RemoteIdentity {
            sku: "PSM-PROBE-SKU-2".into(),
            barcode: Some(PROBE_BARCODE.into())
        }
    );
}

#[test]
fn c3a_create_draft_with_sku_and_barcode() {
    let shop = shop();
    let (_app, run) = run();
    let create = fixture("sv7_create");
    shop.fake.replay_any(&create);
    let title = "PSM-PROBE product (Session Manager grounding probe)";
    let id = shop
        .adapter
        .item_create(&run, PROBE_SKU, Some(PROBE_BARCODE), title)
        .unwrap();
    assert_eq!(id, PRODUCT);
    // The recorded request is the SV-7 shape: exactly the variables the probe sent.
    let sent = shop.fake.graphql("ProductSet");
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].1, create.variables());
    // The SV-7 read-back shows DRAFT and the default variant's SKU/barcode equal to the inputs.
    let readback = fixture("sv7_create_readback");
    assert_eq!(readback.data()["product"]["status"], json!("DRAFT"));
    shop.fake.replay(&readback);
    let item = shop.adapter.item_get(&id).unwrap();
    assert_eq!(item.sku.as_deref(), Some(PROBE_SKU));
    assert_eq!(item.barcode.as_deref(), Some(PROBE_BARCODE));
}

#[test]
fn c3b_confirmed_run_only_from_a_confirmed_row() {
    let app = TestApp::new().unwrap();
    let conn = rusqlite::Connection::open(app.app.paths.db()).unwrap();
    conn.execute(
        "INSERT INTO publish_runs(integration,created_at,confirmed_at,confirmed_summary) VALUES ('shopify',1,NULL,'x')",
        [],
    )
    .unwrap();
    let unconfirmed = conn.last_insert_rowid();
    assert!(matches!(
        ConfirmedRun::from_persisted(app.store(), unconfirmed),
        Err(Error::NotConfirmed)
    ));
    assert!(matches!(
        ConfirmedRun::from_persisted(app.store(), 9_999),
        Err(Error::NotConfirmed)
    ));
    conn.execute(
        "INSERT INTO publish_runs(integration,created_at,confirmed_at,confirmed_summary) VALUES ('shopify',1,42,'x')",
        [],
    )
    .unwrap();
    let confirmed = conn.last_insert_rowid();
    let run = ConfirmedRun::from_persisted(app.store(), confirmed).unwrap();
    assert_eq!((run.run_id(), run.confirmed_at()), (confirmed, 42));
}

// ---------------------------------------------------------------------------------------------
// C4 attributes_schema (type map, writability, condition key)

fn assert_type(key: &str, core: CoreType, writable: bool) -> Definition {
    let shop = shop();
    let schema = schema_of(&shop);
    let (definition, _) = find(&schema, key);
    assert_eq!(
        (definition.core_type, definition.writable),
        (core, writable),
        "{key}"
    );
    definition.clone()
}

#[test]
fn c4a_choice() {
    let d = assert_type("custom.extention_type", CoreType::Choice, true);
    assert_eq!(d.choices, vec!["one", "Two Outside"]);
    // The severity-choice condition field and the no-product-group rule over the whole return.
    let shop = shop();
    let schema = schema_of(&shop);
    let (cracks, _) = find(&schema, "custom.condition_psm_probe_cracks");
    assert_eq!(cracks.choices, SEVERITY_SET.to_vec());
    assert_eq!(cracks.group, AttributeGroup::Condition);
    assert!(
        schema
            .iter()
            .all(|(d, _)| d.group != AttributeGroup::Product)
    );
}

#[test]
fn c4b_text() {
    assert_type("beyo.fx_text", CoreType::Text, true);
}

#[test]
fn c4c_multi_choice() {
    let d = assert_type("custom.testing_listing", CoreType::MultiChoice, true);
    assert_eq!(d.choices.len(), 4);
}

#[test]
fn c4d_tags() {
    assert_type("beyo.fx_list_text", CoreType::Tags, true);
}

#[test]
fn c4e_textarea() {
    assert_type("beyo.fx_text_long", CoreType::Textarea, true);
}

#[test]
fn c4f_rich_text() {
    assert_type("custom.psm_probe_rich_text_field", CoreType::RichText, true);
}

#[test]
fn c4g_integer() {
    assert_type("beyo.fx_integer", CoreType::Integer, true);
}

#[test]
fn c4h_decimal() {
    assert_type("beyo.fx_decimal", CoreType::Decimal, true);
}

#[test]
fn c4i_dimension_units() {
    let d = assert_type("beyo.fx_dimension", CoreType::Dimension, true);
    assert_eq!(d.validations, json!({ "units": DIMENSION_UNITS }));
}

#[test]
fn c4j_weight_units() {
    let d = assert_type("beyo.fx_weight", CoreType::Weight, true);
    assert_eq!(d.validations, json!({ "units": WEIGHT_UNITS }));
}

#[test]
fn c4k_volume_is_dimension() {
    let d = assert_type("beyo.fx_volume", CoreType::Dimension, true);
    assert_eq!(d.validations, json!({ "units": VOLUME_UNITS }));
}

#[test]
fn c4l_boolean() {
    assert_type("beyo.fx_boolean", CoreType::Boolean, true);
}

#[test]
fn c4m_date() {
    assert_type("beyo.fx_date", CoreType::Date, true);
}

#[test]
fn c4n_date_time_is_read_only_reference() {
    assert_type("beyo.fx_datetime", CoreType::Reference, false);
}

#[test]
fn c4o_color() {
    assert_type("beyo.fx_color", CoreType::Color, true);
}

#[test]
fn c4p_rating_bounds() {
    let d = assert_type("beyo.fx_rating", CoreType::Rating, true);
    assert_eq!(d.validations, json!({ "min": "1", "max": "5" }));
}

#[test]
fn c4q_money() {
    assert_type("beyo.fx_money", CoreType::Money, true);
}

#[test]
fn c4r_url() {
    assert_type("beyo.fx_url", CoreType::Url, true);
}

#[test]
fn c4s_configured_condition_key_is_media_list() {
    let d = assert_type("custom.damage_report", CoreType::MediaList, true);
    assert_eq!(d.group, AttributeGroup::Condition);
}

#[test]
fn c4t_references_are_read_only() {
    let shop = shop();
    let schema = schema_of(&shop);
    for key in [
        "custom.psm_probe_file_reference",
        "beyo.fx_reference",
        "custom.psm_probe_product_reference",
        "beyo.fx_list_reference",
        "custom.psm_probe_list_product_reference",
    ] {
        let (d, link) = find(&schema, key);
        assert_eq!(
            (d.core_type, d.writable),
            (CoreType::Reference, false),
            "{key}"
        );
        assert!(!link.platform_type.is_empty());
    }
}

#[test]
fn c4u_access_forbidding_writes_makes_read_only() {
    // Declared synthetic edit: the shop's definitions all carry `PUBLIC_READ_WRITE`, so the
    // captured `beyo.fx_integer` definition's `access.admin` is changed to `MERCHANT_READ_WRITE`
    // ("no other apps have access").
    let shop = shop();
    let mut edited = 0;
    for name in ["sv1_definitions_p1", "sv1_definitions_p2"] {
        let mut page = fixture(name);
        let nodes = page.body["data"]["metafieldDefinitions"]["nodes"]
            .as_array_mut()
            .unwrap();
        for node in nodes
            .iter_mut()
            .filter(|n| n["namespace"] == "beyo" && n["key"] == "fx_integer")
        {
            node["access"]["admin"] = json!("MERCHANT_READ_WRITE");
            edited += 1;
        }
        shop.fake.replay(&page);
    }
    assert_eq!(edited, 1);
    let schema = shop.adapter.attributes_schema().unwrap();
    let (d, _) = find(&schema, "beyo.fx_integer");
    assert_eq!((d.core_type, d.writable), (CoreType::Integer, false));
}

#[test]
fn c4v_only_the_configured_file_list_is_media_list() {
    let shop = shop();
    let schema = schema_of(&shop);
    let (other, _) = find(&schema, "custom.psm_probe_list_file_reference");
    assert_eq!(
        (other.core_type, other.writable),
        (CoreType::Reference, false)
    );
    let media_lists: Vec<&str> = schema
        .iter()
        .filter(|(d, _)| d.core_type == CoreType::MediaList)
        .map(|(d, _)| d.key.as_str())
        .collect();
    assert_eq!(media_lists, vec!["custom.damage_report"]);
}

#[test]
fn c4w_missing_condition_key_has_no_media_list() {
    let shop = bare_shop("custom.nope");
    shop.fake.on_form_always(TOKEN_URL, token_reply(T1));
    let schema = schema_of(&shop);
    assert!(
        schema
            .iter()
            .all(|(d, _)| d.core_type != CoreType::MediaList)
    );
    let (d, _) = find(&schema, "custom.damage_report");
    assert_eq!((d.core_type, d.writable), (CoreType::Reference, false));
}

// ---------------------------------------------------------------------------------------------
// C5 attributes_read → canonicalize → to_platform_string; attributes_write batching

/// Reads the SV-1 values, canonicalizes `key` with its schema definition, and checks the round
/// trip through `to_platform_string`. Returns (canonical, platform string, fixture string).
fn round_trip(key: &str, expected_canonical: &str, byte_identity: bool) {
    let shop = shop();
    let schema = schema_of(&shop);
    let (definition, link) = find(&schema, key).clone();
    let readback = fixture("sv1_values_readback");
    shop.fake.replay(&readback);
    let values = shop.adapter.attributes_read(PRODUCT).unwrap();
    let raw = values
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| panic!("no value for {key}"));
    let s = readback.data()["product"]["metafields"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| {
            format!(
                "{}.{}",
                m["namespace"].as_str().unwrap(),
                m["key"].as_str().unwrap()
            ) == key
        })
        .and_then(|m| m["value"].as_str())
        .unwrap()
        .to_owned();
    let c = canonicalize(definition.core_type, &definition, raw)
        .unwrap()
        .0;
    assert_eq!(c, expected_canonical, "{key}");
    let canonical: Json = serde_json::from_str(&c).unwrap();
    let platform = type_map::to_platform_string(definition.core_type, &canonical);
    let back = type_map::from_platform_string(definition.core_type, &link.platform_type, &platform);
    assert_eq!(
        canonicalize(definition.core_type, &definition, back)
            .unwrap()
            .0,
        c,
        "{key}"
    );
    if byte_identity {
        assert_eq!(platform, s, "{key}");
    }
}

#[test]
fn c5a_choice() {
    round_trip("custom.extention_type", r#""Two Outside""#, true);
}

#[test]
fn c5b_text() {
    round_trip("beyo.fx_text", "\"Hej världen ✓\"", true);
}

#[test]
fn c5c_multi_choice() {
    round_trip(
        "custom.testing_listing",
        r#"["Completely restored","Light wear consistent with age and use"]"#,
        true,
    );
}

#[test]
fn c5d_tags() {
    round_trip("beyo.fx_list_text", r#"["teak","oak"]"#, true);
}

#[test]
fn c5e_textarea() {
    round_trip(
        "beyo.fx_text_long",
        r#""line one\nline two\n\nsecond paragraph""#,
        true,
    );
}

#[test]
fn c5f_rich_text() {
    round_trip(
        "custom.psm_probe_rich_text_field",
        r#""First paragraph & more\n\nSecond <b> \"q\"""#,
        false,
    );
}

#[test]
fn c5g_integer() {
    round_trip("beyo.fx_integer", "42", true);
}

#[test]
fn c5h_decimal() {
    round_trip("beyo.fx_decimal", r#""12.5""#, false);
}

#[test]
fn c5i_dimension() {
    round_trip(
        "beyo.fx_dimension",
        r#"{"unit":"CENTIMETERS","value":"45.5"}"#,
        false,
    );
}

#[test]
fn c5j_weight() {
    round_trip(
        "beyo.fx_weight",
        r#"{"unit":"KILOGRAMS","value":"2.5"}"#,
        false,
    );
}

#[test]
fn c5k_boolean() {
    round_trip("beyo.fx_boolean", "true", true);
}

#[test]
fn c5l_date() {
    round_trip("beyo.fx_date", r#""2026-10-26""#, true);
}

#[test]
fn c5m_color() {
    round_trip("beyo.fx_color", r##""#a1b2c3""##, true);
}

#[test]
fn c5n_rating() {
    round_trip(
        "beyo.fx_rating",
        r#"{"max":"5","min":"1","value":"4.5"}"#,
        false,
    );
}

#[test]
fn c5o_money() {
    round_trip(
        "beyo.fx_money",
        r#"{"amount":"12.5","currency":"USD"}"#,
        false,
    );
}

#[test]
fn c5p_url() {
    round_trip("beyo.fx_url", r#""https://example.com/item?x=1""#, true);
}

fn text_values(n: usize) -> Vec<WriteValue> {
    (0..n)
        .map(|i| WriteValue {
            key: format!("psm_probe.limit_{i:02}"),
            link: DefinitionLink {
                platform_type: "single_line_text_field".into(),
                platform_meta: json!({}),
            },
            value: json!(format!("v{i}")),
        })
        .collect()
}

#[test]
fn c5q_write_batched_at_the_limit_without_credentials() {
    let shop = shop();
    let (_app, run) = run();
    shop.fake
        .on_graphql_always("MetafieldsSet", None, fixture("sv5_limit_25").reply());
    shop.adapter
        .attributes_write(&run, PRODUCT, &text_values(30))
        .unwrap();
    let calls = shop.fake.graphql("MetafieldsSet");
    assert_eq!(calls.len(), 2); // ⌈30 / 25⌉
    assert_eq!(calls[0].1["metafields"].as_array().unwrap().len(), 25);
    assert_eq!(calls[1].1["metafields"].as_array().unwrap().len(), 5);
    for r in shop.fake.requests() {
        if let Recorded::Graphql {
            query, variables, ..
        } = r
        {
            let text = format!("{query}{variables}");
            assert!(!text.contains(T1) && !text.contains(SECRET));
        }
    }
}

#[test]
fn c5r_partial_failure_names_the_failing_entries() {
    let shop = shop();
    let (_app, run) = run();
    // Batch 1 (25 entries) succeeds; batch 2 (3 entries) gets the SV-5 reply whose userErrors
    // point at entries 0 and 2 by `field: ["metafields", "<index>", "value"]`.
    shop.fake.on_graphql(
        "MetafieldsSet",
        None,
        vec![
            fixture("sv5_limit_25").reply(),
            fixture("sv5_partial_two_invalid").reply(),
        ],
    );
    let values = text_values(28);
    let error = shop
        .adapter
        .attributes_write(&run, PRODUCT, &values)
        .unwrap_err();
    match error {
        Error::AttributeVerification { keys } => {
            assert_eq!(keys, vec![values[25].key.clone(), values[27].key.clone()]);
        }
        other => panic!("expected attribute_verification, got {other:?}"),
    }
    assert_eq!(shop.fake.graphql("MetafieldsSet").len(), 2);
}

// ---------------------------------------------------------------------------------------------
// C6 media primitives

const UID: &str = "ab12cd34-ef56-4a7b-8c9d-0123456789ab";

#[test]
fn c6a_staged_upload_request() {
    let shop = shop();
    let (_app, run) = run();
    shop.fake.replay_any(&fixture("sv2_staged"));
    let target = shop
        .adapter
        .staged_uploads_create(&run, "k-1", UID, 0, 3029)
        .unwrap();
    let sent = shop.fake.graphql("StagedUploadsCreate");
    assert_eq!(sent.len(), 1);
    let input = &sent[0].1["input"][0];
    assert_eq!(input["filename"], json!("k-1-ab12cd34ef56.jpg"));
    assert_eq!(input["mimeType"], json!("image/jpeg"));
    assert_eq!(input["resource"], json!("IMAGE"));
    assert_eq!(input["httpMethod"], json!("PUT"));
    assert_eq!(target.filename, "k-1-ab12cd34ef56.jpg");
}

fn staged_target() -> StagedTarget {
    let staged = fixture("sv2_staged");
    let t = &staged.data()["stagedUploadsCreate"]["stagedTargets"][0];
    StagedTarget {
        filename: "psm-probe-ab12cd34ef56.jpg".into(),
        url: t["url"].as_str().unwrap().into(),
        resource_url: t["resourceUrl"].as_str().unwrap().into(),
        parameters: t["parameters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                (
                    p["name"].as_str().unwrap().into(),
                    p["value"].as_str().unwrap().into(),
                )
            })
            .collect(),
    }
}

#[test]
fn c6b_upload_puts_with_parameters_as_headers() {
    let shop = shop();
    let (_app, run) = run();
    let target = staged_target();
    let staged = fixture("sv2_staged");
    let t = &staged.data()["stagedUploadsCreate"]["stagedTargets"][0];
    shop.fake.on_put(&target.url, fixture("sv2_put").reply());
    shop.adapter.upload(&run, &target, b"jpeg bytes").unwrap();
    let puts = shop.fake.puts();
    assert_eq!(puts.len(), 1);
    assert_eq!(puts[0].0, t["url"].as_str().unwrap());
    let expected: std::collections::BTreeMap<String, String> = t["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["name"].as_str().unwrap().into(),
                p["value"].as_str().unwrap().into(),
            )
        })
        .collect();
    let sent: std::collections::BTreeMap<String, String> = puts[0].1.iter().cloned().collect();
    assert_eq!(sent, expected);
}

#[test]
fn c6c_create_media_returns_the_new_media_id() {
    let shop = shop();
    let (_app, run) = run();
    let create = fixture("sv2_media_create");
    shop.fake.replay_any(&create);
    let resource = staged_target().resource_url;
    let id = shop
        .adapter
        .product_create_media(&run, PRODUCT, &resource, "psm probe 1")
        .unwrap();
    assert_eq!(id, "gid://shopify/MediaImage/46644767260825");
    let sent = shop.fake.graphql("ProductCreateMedia");
    assert_eq!(sent[0].1["media"][0]["originalSource"], json!(resource));
}

#[test]
fn c6d_poll_media_ready_failed_and_timeout() {
    let media = "gid://shopify/MediaImage/46644767260825";
    let processing = fixture("sv2_media_poll_processing").reply();
    // PROCESSING, PROCESSING, READY → returns after the third poll.
    let shop = shop();
    shop.fake.on_graphql(
        "MediaPoll",
        None,
        vec![
            processing.clone(),
            processing.clone(),
            fixture("sv2_media_poll_ready").reply(),
        ],
    );
    shop.adapter.poll_media_ready(media).unwrap();
    assert_eq!(shop.fake.graphql("MediaPoll").len(), 3);
    // A sequence ending FAILED → Provider(<fixture message>).
    let shop2 = self::shop();
    shop2.fake.on_graphql(
        "MediaPoll",
        None,
        vec![processing.clone(), fixture("sv2_media_poll_failed").reply()],
    );
    match shop2.adapter.poll_media_ready(media) {
        Err(Error::Provider(message)) => assert_eq!(message, "Image could not be processed"),
        other => panic!("expected provider error, got {other:?}"),
    }
    // Never READY within the cap (5 attempts) → Timeout.
    let shop3 = self::shop();
    shop3.fake.on_graphql_always("MediaPoll", None, processing);
    assert!(matches!(
        shop3.adapter.poll_media_ready(media),
        Err(Error::Timeout)
    ));
    assert_eq!(shop3.fake.graphql("MediaPoll").len(), 5);
}

#[test]
fn c6e_reorder_moves_every_id_zero_based() {
    let shop = shop();
    let (_app, run) = run();
    shop.fake.replay_any(&fixture("sv4_reorder"));
    let d: Vec<String> = [
        "gid://shopify/MediaImage/46644767686809",
        "gid://shopify/MediaImage/46644767457433",
        "gid://shopify/MediaImage/46644767260825",
    ]
    .map(String::from)
    .to_vec();
    let job = shop
        .adapter
        .product_reorder_media(&run, PRODUCT, &d)
        .unwrap();
    assert_eq!(
        job,
        "gid://shopify/Job/f05fcf08-5e3f-45f2-bd04-31b7179b98e8"
    );
    let sent = shop.fake.graphql("ProductReorderMedia");
    assert_eq!(sent.len(), 1);
    assert_eq!(
        sent[0].1["moves"],
        json!([
            {"id": d[0], "newPosition": "0"},
            {"id": d[1], "newPosition": "1"},
            {"id": d[2], "newPosition": "2"},
        ])
    );
}

#[test]
fn c6f_poll_job_until_done() {
    let shop = shop();
    shop.fake.on_graphql(
        "JobPoll",
        None,
        vec![
            fixture("sv4_job_poll_pending").reply(),
            fixture("sv4_job_poll_done").reply(),
        ],
    );
    shop.adapter
        .poll_job("gid://shopify/Job/f05fcf08-5e3f-45f2-bd04-31b7179b98e8")
        .unwrap();
    assert_eq!(shop.fake.graphql("JobPoll").len(), 2);
}

#[test]
fn c6g_list_media_keeps_exact_filenames() {
    let shop = shop();
    shop.fake.replay(&fixture("sv2_item_get"));
    let media = shop.adapter.list_media(PRODUCT).unwrap();
    let names: Vec<(Option<&str>, i64)> = media
        .iter()
        .take(2)
        .map(|m| (m.filename.as_deref(), m.position))
        .collect();
    assert_eq!(
        names,
        vec![
            (Some("psm-probe-ab12cd34ef56.jpg"), 1),
            (
                Some("psm-probe-ab12cd34ef56_4fffd4cd-17a1-449a-92f7-2e862798500b.jpg"),
                2
            ),
        ]
    );
}

#[test]
fn c6h_delete_not_found_counts_as_deleted() {
    let shop = shop();
    let (_app, run) = run();
    shop.fake.replay_any(&fixture("sv4_delete_media_not_found"));
    let ids = vec!["gid://shopify/MediaImage/46644767260825".to_string()];
    assert!(
        shop.adapter
            .product_delete_media(&run, PRODUCT, &ids)
            .is_ok()
    );
    let sent = shop.fake.graphql("ProductDeleteMedia");
    assert_eq!(
        sent[0].1,
        json!({"files": [{"id": ids[0], "referencesToRemove": [PRODUCT]}]})
    );
}

// ---------------------------------------------------------------------------------------------
// C7 files and the condition attribute

#[test]
fn c7a_file_create_then_poll_ready() {
    let shop = shop();
    let (_app, run) = run();
    shop.fake.replay_any(&fixture("sv3_file_create"));
    shop.fake.on_graphql(
        "FilePoll",
        None,
        vec![
            fixture("sv3_file_poll_processing").reply(),
            fixture("sv3_file_poll_ready").reply(),
        ],
    );
    let resource = "https://shopify-staged-uploads.storage.googleapis.com/tmp/00000000000/files/x/k-1-ab12cd34ef56.jpg";
    let id = shop
        .adapter
        .file_create(&run, resource, "k-1-ab12cd34ef56.jpg")
        .unwrap();
    assert_eq!(id, "gid://shopify/MediaImage/46644770275481");
    assert!(id.starts_with("gid://shopify/MediaImage/"));
    shop.adapter.poll_file(&id).unwrap();
    let sent = shop.fake.graphql("FileCreate");
    assert_eq!(
        sent[0].1["files"][0]["duplicateResolutionMode"],
        json!("APPEND_UUID")
    );
    assert_eq!(shop.fake.graphql("FilePoll").len(), 2);
}

#[test]
fn c7b_media_list_write_sends_file_ids() {
    let shop = shop();
    let (_app, run) = run();
    let schema = schema_of(&shop);
    let (_, link) = find(&schema, "custom.damage_report").clone();
    shop.fake.replay_any(&fixture("sv3_damage_report_set"));
    shop.adapter
        .attributes_write(
            &run,
            PRODUCT,
            &[WriteValue {
                key: "custom.damage_report".into(),
                link,
                value: json!(["f1", "f2"]),
            }],
        )
        .unwrap();
    let sent = shop.fake.graphql("MetafieldsSet");
    let entry = &sent[0].1["metafields"][0];
    assert_eq!(entry["value"], json!(r#"["f1","f2"]"#));
    assert_eq!(entry["type"], json!("list.file_reference"));
    assert_eq!(
        (entry["namespace"].clone(), entry["key"].clone()),
        (json!("custom"), json!("damage_report"))
    );
}

// ---------------------------------------------------------------------------------------------
// C8 description and tags payloads

#[test]
fn c8a_description_html_escapes() {
    assert_eq!(
        description_html("a & b\n\nc < d \"q\""),
        "<p>a &amp; b</p><p>c &lt; d &quot;q&quot;</p>"
    );
}

#[test]
fn c8b_update_tags_payload() {
    let shop = shop();
    let (_app, run) = run();
    shop.fake.replay_any(&fixture("tags_update"));
    shop.adapter
        .update_tags(&run, PRODUCT, &["a".to_string(), "b".to_string()])
        .unwrap();
    let sent = shop.fake.graphql("ProductUpdateTags");
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].1["product"]["tags"], json!(["a", "b"]));
    assert_eq!(sent[0].1["product"]["id"], json!(PRODUCT));
}

// ---------------------------------------------------------------------------------------------
// C9 access token (09A MC-23)

fn search_fixture() -> Fixture {
    fixture("sv6_search_immediate")
}

#[test]
fn c9a_token_fetched_once_and_reused() {
    let shop = bare_shop("custom.damage_report");
    shop.fake.on_form(TOKEN_URL, vec![token_reply(T1)]);
    shop.fake
        .on_graphql_always("VariantSearch", None, search_fixture().reply());
    shop.adapter.item_search(PROBE_SKU).unwrap();
    shop.adapter.item_search(PROBE_SKU).unwrap();
    let forms = shop.fake.forms();
    assert_eq!(forms.len(), 1);
    assert_eq!(forms[0].0, TOKEN_URL);
    let fields: std::collections::BTreeMap<&str, &str> = forms[0]
        .1
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    assert_eq!(
        fields,
        [
            ("client_id", CLIENT_ID),
            ("client_secret", SECRET),
            ("grant_type", "client_credentials")
        ]
        .into_iter()
        .collect()
    );
    let tokens: Vec<String> = shop
        .fake
        .graphql("VariantSearch")
        .into_iter()
        .map(|(t, _)| t)
        .collect();
    assert_eq!(tokens, vec![T1, T1]);
}

#[test]
fn c9b_token_refreshed_before_expiry() {
    let shop = bare_shop("custom.damage_report");
    shop.fake
        .on_form(TOKEN_URL, vec![token_reply(T1), token_reply(T2)]);
    shop.fake
        .on_graphql_always("VariantSearch", None, search_fixture().reply());
    shop.adapter.item_search(PROBE_SKU).unwrap();
    shop.clock.advance((LIFETIME_S - 59) * 1000);
    shop.adapter.item_search(PROBE_SKU).unwrap();
    assert_eq!(shop.fake.forms().len(), 2);
    let tokens: Vec<String> = shop
        .fake
        .graphql("VariantSearch")
        .into_iter()
        .map(|(t, _)| t)
        .collect();
    assert_eq!(tokens, vec![T1, T2]);
}

#[test]
fn c9c_unauthorized_refetches_and_retries_once() {
    let unauthorized = fixture("sv0_graphql_401").reply();
    // 401 then 200: one new token request, the call retried once, Ok.
    let shop = bare_shop("custom.damage_report");
    shop.fake
        .on_form(TOKEN_URL, vec![token_reply(T1), token_reply(T2)]);
    shop.fake.on_graphql(
        "VariantSearch",
        None,
        vec![unauthorized.clone(), search_fixture().reply()],
    );
    assert_eq!(shop.adapter.item_search(PROBE_SKU).unwrap().len(), 1);
    assert_eq!(shop.fake.forms().len(), 2);
    let tokens: Vec<String> = shop
        .fake
        .graphql("VariantSearch")
        .into_iter()
        .map(|(t, _)| t)
        .collect();
    assert_eq!(tokens, vec![T1, T2]);
    // 401, 401: credentials_rejected after exactly two GraphQL attempts.
    let shop = bare_shop("custom.damage_report");
    shop.fake
        .on_form(TOKEN_URL, vec![token_reply(T1), token_reply(T2)]);
    shop.fake.on_graphql(
        "VariantSearch",
        None,
        vec![unauthorized.clone(), unauthorized],
    );
    assert!(matches!(
        shop.adapter.item_search(PROBE_SKU),
        Err(Error::CredentialsRejected)
    ));
    assert_eq!(shop.fake.graphql_count(), 2);
}

fn first_call_refused(token_fixture: &str) -> (Error, usize) {
    let shop = bare_shop("custom.damage_report");
    shop.fake
        .on_form(TOKEN_URL, vec![fixture(token_fixture).reply()]);
    let error = shop.adapter.item_search(PROBE_SKU).unwrap_err();
    (error, shop.fake.graphql_count())
}

#[test]
fn c9d_shop_not_permitted() {
    let (error, graphql) = first_call_refused("sv0_token_shop_not_permitted");
    assert!(matches!(error, Error::ShopNotPermitted));
    assert_eq!(
        error.to_string(),
        "This Shopify store is not in the app's organization"
    );
    assert_eq!(graphql, 0);
}

#[test]
fn c9e_app_not_installed() {
    let (error, graphql) = first_call_refused("sv0_token_app_not_installed");
    assert!(matches!(error, Error::AppNotInstalled));
    assert_eq!(
        error.to_string(),
        "The Session Manager app is not installed on this store"
    );
    assert_eq!(graphql, 0);
}

#[test]
fn c9f_wrong_secret_is_credentials_rejected() {
    let (error, graphql) = first_call_refused("sv0_token_wrong_secret");
    assert!(matches!(error, Error::CredentialsRejected));
    assert_eq!(error.to_string(), "Shopify credentials rejected");
    assert_eq!(graphql, 0);
}

#[test]
fn c9g_secret_and_tokens_never_leave_the_token_request() {
    let mut errors: Vec<Error> = Vec::new();
    let mut transports: Vec<Arc<FakeTransport>> = Vec::new();
    let search = search_fixture().reply();
    let unauthorized = fixture("sv0_graphql_401").reply();
    // C9(a), (b), (c) sequences.
    {
        let shop = bare_shop("custom.damage_report");
        shop.fake
            .on_form(TOKEN_URL, vec![token_reply(T1), token_reply(T2)]);
        shop.fake
            .on_graphql("VariantSearch", None, vec![search.clone(), search.clone()]);
        shop.adapter.item_search(PROBE_SKU).unwrap();
        shop.clock.advance((LIFETIME_S - 59) * 1000);
        shop.adapter.item_search(PROBE_SKU).unwrap();
        transports.push(shop.fake.clone());
    }
    {
        let shop = bare_shop("custom.damage_report");
        shop.fake
            .on_form(TOKEN_URL, vec![token_reply(T1), token_reply(T2)]);
        shop.fake.on_graphql(
            "VariantSearch",
            None,
            vec![unauthorized.clone(), search.clone()],
        );
        shop.adapter.item_search(PROBE_SKU).unwrap();
        transports.push(shop.fake.clone());
    }
    {
        let shop = bare_shop("custom.damage_report");
        shop.fake
            .on_form(TOKEN_URL, vec![token_reply(T1), token_reply(T2)]);
        shop.fake.on_graphql(
            "VariantSearch",
            None,
            vec![unauthorized.clone(), unauthorized],
        );
        errors.push(shop.adapter.item_search(PROBE_SKU).unwrap_err());
        transports.push(shop.fake.clone());
    }
    // C9(d), (e), (f) sequences.
    for refusal in [
        "sv0_token_shop_not_permitted",
        "sv0_token_app_not_installed",
        "sv0_token_wrong_secret",
    ] {
        let shop = bare_shop("custom.damage_report");
        shop.fake.on_form(TOKEN_URL, vec![fixture(refusal).reply()]);
        errors.push(shop.adapter.item_search(PROBE_SKU).unwrap_err());
        transports.push(shop.fake.clone());
    }
    // C6(a), (b) sequences.
    {
        let shop = bare_shop("custom.damage_report");
        shop.fake.on_form_always(TOKEN_URL, token_reply(T1));
        let (_app, run) = run();
        shop.fake.replay_any(&fixture("sv2_staged"));
        let target = shop
            .adapter
            .staged_uploads_create(&run, "k-1", UID, 0, 10)
            .unwrap();
        shop.fake.on_put(&target.url, fixture("sv2_put").reply());
        shop.adapter.upload(&run, &target, b"bytes").unwrap();
        // A failing upload's error is checked too.
        shop.fake
            .on_put(&target.url, HttpReply::new(403, json!("denied")));
        errors.push(shop.adapter.upload(&run, &target, b"bytes").unwrap_err());
        transports.push(shop.fake.clone());
    }
    let forbidden = [SECRET, T1, T2];
    let clean = |text: &str| forbidden.iter().all(|f| !text.contains(f));
    let mut graphql_tokens = Vec::new();
    for fake in &transports {
        for request in fake.requests() {
            match request {
                Recorded::Graphql {
                    query,
                    variables,
                    access_token,
                    url,
                    ..
                } => {
                    assert!(clean(&query) && clean(&variables.to_string()) && clean(&url));
                    graphql_tokens.push(access_token);
                }
                Recorded::Put { url, headers, .. } => {
                    assert!(clean(&url));
                    assert!(headers.iter().all(|(k, v)| clean(k) && clean(v)));
                }
                Recorded::Form { .. } => {}
            }
        }
    }
    assert!(graphql_tokens.iter().any(|t| t == T1) && graphql_tokens.iter().any(|t| t == T2));
    assert_eq!(errors.len(), 5);
    for error in &errors {
        assert!(clean(&error.to_string()), "{error}");
        assert!(clean(&format!("{error:?}")), "{error:?}");
    }
    let config_debug = format!("{:?}", bare_shop("custom.damage_report").adapter.config());
    assert!(clean(&config_debug), "{config_debug}");
}
