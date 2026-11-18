//! R8·U3 (rows A17·C4–C6): the shop facts research needs from the real `ShopifyAdapter` over
//! `FakeTransport` (08B MC-A3, MC-A2): the shop's public hosts and the item's own listing, the
//! unit names in imported definitions, and the wired app excluding the item's own listing.
//! Fixtures `r8_*.json` are schema-grounded, not observed (see their `source`).

use std::{fs, sync::Arc};

use psm_core::{
    domain::{
        attribute::{CoreType, Definition},
        photo::{Classification as C, NewPhoto, PhotoSource},
        publish::DefinitionLink,
    },
    mechanisms::hash::sha256_file,
    ports::{clock::Clock, commerce::CommerceAdapter},
};
use psm_integrations::commerce::shopify::{
    PollConfig, ShopifyAdapter, ShopifyConfig, transport::HttpReply as ShopReply,
};
use psm_research::{
    adapters::page_reader::HttpPageReader,
    domain::RunState,
    fakes::{FakeModel, FakeResearchHttp, FakeVisualSearch, reply_from_capture},
    ports::{Providers, SearchCandidate},
    rules::value_map::map_output,
    store as rstore,
};
use psm_testkit::{
    TestApp,
    clock::ManualClock,
    fake_transport::{FakeTransport, Recorded},
    fixtures::{shopify::fixture, synthetic_jpeg::SyntheticJpeg},
    research_test_settings,
};
use serde_json::{Value as Json, json};

const TOKEN_URL: &str = "https://example.myshopify.com/admin/oauth/access_token";
const PRODUCT: &str = "gid://shopify/Product/16185797640522";
const OWN: &str = "https://beyovintage.se/en/products/swedish-bookshelf-in-oak-5";
const OTHER: &str = "https://beyovintage.se/en/products/teak-sideboard-2";

fn shop() -> (Arc<FakeTransport>, ShopifyAdapter) {
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

/// The GraphQL documents the fake received.
fn queries(fake: &FakeTransport) -> Vec<String> {
    fake.requests()
        .into_iter()
        .filter_map(|r| match r {
            Recorded::Graphql { query, .. } => Some(query),
            _ => None,
        })
        .collect()
}

#[test]
fn a17_c4a_hosts_and_the_items_own_listing() {
    let (fake, adapter) = shop();
    fake.replay(&fixture("r8_public_listing"));
    let listing = adapter.public_listing(PRODUCT).unwrap();
    assert_eq!(
        listing.hosts,
        vec!["beyovintage.se", "example.myshopify.com"]
    );
    assert_eq!(
        listing.item_url.as_deref(),
        Some("https://beyovintage.se/products/swedish-bookshelf-in-oak-5")
    );
    assert_eq!(
        listing.item_path_tail,
        "products/swedish-bookshelf-in-oak-5"
    );
    let sent = queries(&fake);
    assert_eq!(sent.len(), 1);
    assert!(!sent[0].contains("webPresences"), "{}", sent[0]);

    let (fake, adapter) = shop();
    fake.replay(&fixture("r8_public_listing_unpublished"));
    let listing = adapter.public_listing(PRODUCT).unwrap();
    assert_eq!(listing.item_url, None);
    assert_eq!(
        listing.hosts,
        vec!["beyovintage.se", "example.myshopify.com"]
    );
    assert_eq!(
        listing.item_path_tail,
        "products/swedish-bookshelf-in-oak-5"
    );
}

#[test]
fn a17_c4b_an_unlinked_item_gets_the_hosts_only() {
    let (fake, adapter) = shop();
    fake.replay(&fixture("r8_shop_hosts"));
    let listing = adapter.public_listing("").unwrap();
    assert_eq!(listing.hosts, vec!["example.myshopify.com"]);
    assert_eq!(listing.item_url, None);
    assert_eq!(listing.item_path_tail, "");
    let sent = queries(&fake);
    assert_eq!(sent.len(), 1);
    assert!(!sent[0].contains("product"), "{}", sent[0]);
}

#[test]
fn a17_c5a_a_dimension_converts_by_the_adapters_unit_names() {
    let (fake, adapter) = shop();
    fake.replay(&fixture("sv1_definitions_p1"));
    fake.replay(&fixture("sv1_definitions_p2"));
    let schema: Vec<(Definition, DefinitionLink)> = adapter.attributes_schema().unwrap();
    let (definition, _) = schema
        .iter()
        .find(|(d, _)| d.key == "beyo.fx_dimension")
        .unwrap();
    assert_eq!(definition.core_type, CoreType::Dimension);
    assert_eq!(definition.validations["units"][0], json!("MILLIMETERS"));
    let mapped = map_output(definition, &json!({"value": 10, "unit": "in"}));
    assert_eq!(
        mapped.value(),
        Some(&json!({"value": "254", "unit": "MILLIMETERS"}))
    );
}

/// A Main photo with its research source in a new Chair session of a linked item.
fn linked_session(a: &TestApp) -> i64 {
    let (item, session) = a
        .items()
        .create_local("R8-C6A", "Oak bookshelf", "Chair", None)
        .unwrap();
    a.app
        .store
        .research_transaction(|tx| {
            tx.conn()
                .execute(
                    "INSERT INTO item_links(item_id,integration,external_id) VALUES (?1,'shopify',?2)",
                    rusqlite::params![item.id, PRODUCT],
                )
                .map(|_| ())
                .map_err(|e| psm_core::Error::InvalidData(e.to_string()))
        })
        .unwrap();
    let s = session.session.id;
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join("R8-C6A.ARW");
    fs::write(&raw, "raw bytes of R8-C6A").unwrap();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id: s,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    a.review().classify(photo.id, C::Main).unwrap();
    a.store().adjusted_event(&photo.raw_path, false).unwrap();
    let generation = a.store().adjusted_gen(&photo.raw_path).unwrap();
    let preview = a.dir.path().join(format!("adjusted-{}.jpg", photo.id));
    fs::write(&preview, SyntheticJpeg::rgb(1600, 1200, [90, 60, 30])).unwrap();
    a.store()
        .complete_preview(
            &photo.raw_path,
            generation,
            preview.to_str().unwrap(),
            None,
            None,
        )
        .unwrap();
    s
}

/// The shop's answer to A9's `ShopProduct` for the other product (schema-grounded: the query was
/// accepted by a 2026-10 shop on 2026-11-18; the values are made up).
fn other_product() -> ShopReply {
    ShopReply::new(
        200,
        json!({"data": {"productByIdentifier": {
            "id": "gid://shopify/Product/2",
            "title": "Teak sideboard",
            "handle": "teak-sideboard-2",
            "productType": "Sideboard",
            "vendor": "Beyo Vintage",
            "onlineStoreUrl": "https://beyovintage.se/products/teak-sideboard-2",
            "descriptionHtml": "<p>A teak sideboard from the 1960s.</p>",
            "metafields": {"nodes": [
                {"namespace": "custom", "key": "designer", "type": "single_line_text_field",
                 "value": "Gunnar Nilsson", "definition": {"name": "Designer"},
                 "reference": null, "references": null},
                {"namespace": "custom", "key": "woods", "type": "list.metaobject_reference",
                 "value": "[\"gid://shopify/Metaobject/1\"]", "definition": {"name": "Wood"},
                 "reference": null, "references": {"nodes": [{"displayName": "Teak"}]}},
                {"namespace": "custom", "key": "width", "type": "dimension",
                 "value": "{\"value\":180.0,\"unit\":\"CENTIMETERS\"}",
                 "definition": null, "reference": null, "references": null},
                {"namespace": "custom", "key": "damage_report", "type": "list.file_reference",
                 "value": "[\"gid://shopify/MediaImage/9\"]", "definition": {"name": "Damage"},
                 "reference": null, "references": {"nodes": [{}]}}
            ]},
            "media": {"nodes": []}
        }}}),
    )
}

#[test]
fn a17_c6a_the_items_own_listing_is_no_evidence_in_the_wired_app() {
    let (fake, adapter) = shop();
    let listing = fixture("r8_public_listing");
    fake.on_graphql_always(
        "PublicListing",
        Some(&listing.variables()),
        ShopReply::new(listing.status, listing.body.clone()),
    );
    // A9: the reader asks the shop's hosts once, then reads the other product through the API.
    fake.on_graphql_always(
        "ShopHosts",
        None,
        ShopReply::new(
            200,
            json!({"data": {"shop": {
                "primaryDomain": {"host": "beyovintage.se", "url": "https://beyovintage.se"},
                "myshopifyDomain": "example.myshopify.com"
            }}}),
        ),
    );
    fake.on_graphql_always(
        "ShopProduct",
        Some(&json!({"handle": "teak-sideboard-2"})),
        other_product(),
    );
    let model = Arc::new(FakeModel::default());
    let search = Arc::new(FakeVisualSearch::default());
    search.set_results(vec![
        SearchCandidate {
            url: OWN.into(),
            rank: 1,
            title: "Swedish bookshelf in oak".into(),
        },
        SearchCandidate {
            url: OTHER.into(),
            rank: 2,
            title: "Teak sideboard".into(),
        },
    ]);
    let http = Arc::new(FakeResearchHttp::default());
    // Derived: the captured shop page's envelope answers the other product's address.
    let page: Json = serde_json::from_str(
        &fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../research/fixtures/pages/shop_listing_page.json"),
        )
        .unwrap(),
    )
    .unwrap();
    http.always("GET", OTHER, Ok(reply_from_capture(&page)));
    let reader = Arc::new(HttpPageReader::new(http.clone(), Arc::new(|_| {}), 60));
    let a = TestApp::with_research_using(
        Arc::new(adapter) as Arc<dyn CommerceAdapter>,
        Providers {
            model: Some(model.clone()),
            search: Some(search),
            reader: Some(reader),
        },
    )
    .unwrap();
    let mut settings = research_test_settings();
    settings["prices"]
        .as_array_mut()
        .unwrap()
        .push(json!({"provider": "page_http", "model": "", "fallback_call_usd": "0.001"}));
    a.set_research_settings(settings).unwrap();
    a.clock.advance(1_762_900_000_000);
    let s = linked_session(&a);
    a.app.research.as_ref().unwrap().arranged(s).unwrap();
    a.tick(0).unwrap();
    a.tick(1_100).unwrap();
    let run = a
        .app
        .store
        .research_transaction(|tx| rstore::runs(tx.conn(), Some(s)))
        .unwrap()
        .pop()
        .expect("a run");
    assert_eq!(run.state, RunState::Finished, "{:?}", run.error);
    // A9: neither shop page went to the storefront: the own listing is set apart, the other
    // product is read through the API.
    let read_urls: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
    assert!(read_urls.is_empty(), "{read_urls:?}");
    assert_eq!(
        fake.graphql("ShopProduct")
            .into_iter()
            .map(|(_, vars)| vars)
            .collect::<Vec<_>>(),
        vec![json!({"handle": "teak-sideboard-2"})]
    );
    let page_text = model
        .requests()
        .into_iter()
        .find(|c| c.stage == Some(psm_research::domain::Stage::Read))
        .map(|c| {
            c.input["page"]["text"]
                .as_str()
                .unwrap_or_default()
                .to_owned()
        })
        .expect("a read request");
    for line in [
        "Teak sideboard",
        "Type: Sideboard",
        "Vendor: Beyo Vintage",
        "Designer: Gunnar Nilsson",
        "Wood: Teak",
        "custom.width: 180.0 centimeters",
        "A teak sideboard from the 1960s.",
    ] {
        assert!(page_text.contains(line), "{line} in {page_text}");
    }
    assert!(!page_text.contains("Damage"), "{page_text}");
    let steps = a
        .app
        .store
        .research_transaction(|tx| rstore::steps(tx.conn(), run.id))
        .unwrap();
    assert!(
        steps
            .iter()
            .any(|st| st.summary == "this item's own listing"
                && st.source_url.as_deref() == Some(OWN)),
        "{steps:?}"
    );
    let outputs = a
        .app
        .store
        .research_transaction(|tx| rstore::outputs(tx.conn(), run.id))
        .unwrap();
    let reads: Vec<Json> = outputs
        .iter()
        .filter(|o| o.stage == "read")
        .map(|o| serde_json::from_str(&o.output).unwrap())
        .collect();
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert_eq!(reads[0]["url"], json!(OTHER));
    assert_eq!(reads[0]["obtained"], json!(true));
    assert_eq!(reads[0]["site"]["class"], json!("shop"));
}

/// A9: a shop page whose handle the shop does not know is "could not be read", with the reason,
/// and nothing is asked of the storefront.
#[test]
fn a9_a_shop_handle_the_shop_does_not_know_is_could_not_be_read() {
    let (fake, adapter) = shop();
    fake.on_graphql_always(
        "ShopHosts",
        None,
        ShopReply::new(
            200,
            json!({"data": {"shop": {
                "primaryDomain": {"host": "beyovintage.se", "url": "https://beyovintage.se"},
                "myshopifyDomain": "example.myshopify.com"
            }}}),
        ),
    );
    fake.on_graphql_always(
        "ShopProduct",
        None,
        ShopReply::new(200, json!({"data": {"productByIdentifier": null}})),
    );
    let http = Arc::new(FakeResearchHttp::default());
    let inner = Arc::new(HttpPageReader::new(http.clone(), Arc::new(|_| {}), 60));
    let reader = psm_research::adapters::page_reader::ShopPageReader::new(
        inner,
        Arc::new(adapter) as Arc<dyn CommerceAdapter>,
    );
    use psm_research::ports::PageReader;
    let reply = reader
        .read("https://beyovintage.se/fr/collections/x/products/gone-1")
        .unwrap();
    assert!(!reply.obtained && !reply.blocked);
    assert!(reply.note.unwrap().contains("not found in your shop"));
    assert!(http.requests().is_empty());
    assert_eq!(
        fake.graphql("ShopProduct")[0].1,
        json!({"handle": "gone-1"})
    );
    // Another site's page still goes to the storefront reader.
    let _ = reader
        .read("https://dealer.example/products/gone-1")
        .unwrap();
    assert_eq!(http.requests().len(), 1);
}
