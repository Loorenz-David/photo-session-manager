//! W5 (rows W5·C1–C3 and W5·C6–C9): the two search texts. U1: the built-in fields, always present
//! in the Product step (`form_get`, `layout_keys`, `presets_set`), read from the linked product
//! (the P67 remote read), and requested and written by the writer (a `write` run on the wired app
//! with the research fakes). U3: the publish step (`PublishService` through `TestApp` with the
//! fake commerce adapter).

use std::fs;

use psm_core::{
    domain::{
        attribute::{
            AiConfidence, AiPrior, AiStatus, AttributeGroup, Preset, PresetField, Provenance,
        },
        photo::{Classification as C, NewPhoto, PhotoSource},
    },
    mechanisms::hash::sha256_file,
    services::attributes::{AttributesService, RemoteRead, layout_keys},
};
use psm_research::{
    domain::{RunState, RunTrigger, Stage},
    store::{self as rstore, RunRow},
    writer::run::Origin,
};
use psm_testkit::{
    TestApp,
    fixtures::definitions::{INTEGRATION, shop_definitions},
    research_test_settings,
};
use serde_json::{Value as Json, json};

const NOW_MS: i64 = 1_763_570_000_000;
const SEO_TITLE: &str = "product.seo_title";
const SEO_DESCRIPTION: &str = "product.seo_description";
const SHOP_SEO_TITLE: &str = "Teak chair | Beyo";

// ---------------------------------------------------------------- U1: fields and the remote read

fn shop_app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    service(&a)
        .import_definitions(INTEGRATION, shop_definitions())
        .unwrap();
    a
}

fn service(a: &TestApp) -> AttributesService<'_> {
    AttributesService::new(a.store(), a.clock.as_ref(), Some(INTEGRATION))
}

fn entries(keys: &[&str]) -> Vec<PresetField> {
    keys.iter()
        .map(|k| PresetField {
            key: (*k).to_owned(),
            default: None,
        })
        .collect()
}

/// A Product preset stored as it was saved before W5 (straight into the store, bypassing
/// `presets_set`), for item type `kind`.
fn stored_preset(a: &TestApp, kind: &str, keys: &[&str]) {
    a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_preset(&Preset {
                item_type: kind.to_owned(),
                group: AttributeGroup::Product,
                layout: entries(keys),
            })
        })
        .unwrap();
}

/// The keys of the Product step of the item's form, in order.
fn product_step(a: &TestApp, item: i64) -> Vec<String> {
    service(a)
        .form_get(item)
        .unwrap()
        .groups
        .into_iter()
        .find(|g| g.group == AttributeGroup::Product)
        .unwrap()
        .fields
        .into_iter()
        .map(|f| f.definition.key)
        .collect()
}

fn local_item(a: &TestApp, sku: &str, kind: &str) -> i64 {
    a.items().create_local(sku, "", kind, None).unwrap().0.id
}

const PRE_W5: [&str; 3] = ["product.title", "product.description", "product.tags"];

#[test]
fn w5_c1a_a_stored_product_preset_shows_both_search_fields_at_the_end() {
    let a = shop_app();
    stored_preset(&a, "Chair", &PRE_W5);
    let item = local_item(&a, "W5-C1A", "Chair");
    assert_eq!(
        product_step(&a, item),
        [
            "product.title",
            "product.description",
            "product.tags",
            SEO_TITLE,
            SEO_DESCRIPTION
        ]
    );
}

#[test]
fn w5_c1b_layout_keys_include_both_search_fields() {
    let a = shop_app();
    stored_preset(&a, "Chair", &PRE_W5);
    let mut keys = None;
    a.store()
        .transaction(&mut |tx| {
            keys = Some(layout_keys(tx, "Chair", Some(INTEGRATION))?);
            Ok(())
        })
        .unwrap();
    let keys = keys.unwrap();
    assert!(keys.contains(SEO_TITLE), "{keys:?}");
    assert!(keys.contains(SEO_DESCRIPTION), "{keys:?}");
}

#[test]
fn w5_c1c_a_layout_with_the_search_fields_placed_keeps_that_order() {
    let a = shop_app();
    let order = [
        "product.title",
        SEO_DESCRIPTION,
        "product.description",
        "product.tags",
        SEO_TITLE,
    ];
    service(&a)
        .presets_set(Preset {
            item_type: "Chair".into(),
            group: AttributeGroup::Product,
            layout: entries(&order),
        })
        .unwrap();
    let stored: Vec<String> = service(&a)
        .presets_get("Chair", AttributeGroup::Product)
        .unwrap()
        .layout
        .into_iter()
        .map(|f| f.key)
        .collect();
    assert_eq!(stored, order);
    let item = local_item(&a, "W5-C1C", "Chair");
    assert_eq!(product_step(&a, item), order);
}

/// A product in the fake shop with the given search texts, adopted as a local item.
fn linked_with_seo(a: &TestApp, sku: &str, title: Option<&str>, description: Option<&str>) -> i64 {
    let id = a.fake_shop().add_product(sku, "Teak chair", None);
    a.fake_shop().set_seo(&id, title, description);
    a.items()
        .with_commerce(Some(a.fake_shop()))
        .adopt(&id)
        .unwrap()
        .id
}

fn row(a: &TestApp, item: i64, key: &str) -> Option<psm_core::domain::attribute::Value> {
    service(a)
        .values(item)
        .unwrap()
        .into_iter()
        .find(|v| v.key == key)
}

#[test]
fn w5_c2a_the_shops_search_title_is_read_and_a_blank_description_is_not() {
    let a = shop_app();
    let item = linked_with_seo(&a, "W5-C2A", Some(SHOP_SEO_TITLE), Some(""));
    let read = service(&a)
        .read_integration_values(Some(a.fake_shop()), item)
        .unwrap();
    assert_eq!(read, RemoteRead::Read);
    let title = row(&a, item, SEO_TITLE).expect("a search title row");
    assert_eq!(title.provenance, Provenance::Integration);
    assert_eq!(title.json(), Some(json!(SHOP_SEO_TITLE)));
    assert!(row(&a, item, SEO_DESCRIPTION).is_none());
}

#[test]
fn w5_c2b_a_human_search_title_is_never_replaced_by_the_shops() {
    let a = shop_app();
    let item = linked_with_seo(&a, "W5-C2B", Some(SHOP_SEO_TITLE), None);
    service(&a)
        .set_value(item, SEO_TITLE, json!("My own title"), Provenance::Human)
        .unwrap();
    let before = row(&a, item, SEO_TITLE).unwrap();
    service(&a)
        .read_integration_values(Some(a.fake_shop()), item)
        .unwrap();
    assert_eq!(row(&a, item, SEO_TITLE), Some(before));
}

// ---------------------------------------------------------------- U1: the writer writes them

/// The wired research app with the writer's default model priced for the fake provider.
fn writer_app() -> TestApp {
    let a = TestApp::with_research().unwrap();
    a.clock.advance(NOW_MS);
    let mut settings = research_test_settings();
    settings["prices"].as_array_mut().unwrap().push(json!({
        "provider": psm_research::fakes::FakeModel::ID, "model": "gpt-6.1-sol",
        "unit": "input_token", "usd_per_million": "2", "fallback_call_usd": "0.001"
    }));
    a.set_research_settings(settings).unwrap();
    a
}

/// An item with one session and one Main photo: `(session, item)`.
fn writer_session(a: &TestApp, sku: &str) -> (i64, i64) {
    let (item, session) = a.items().create_local(sku, "", "Chair", None).unwrap();
    let s = session.session.id;
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(format!("{sku}.ARW"));
    fs::write(&raw, format!("raw bytes of {sku}")).unwrap();
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
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    (s, item.id)
}

const PARAGRAPHS: [&str; 4] = [
    "A dining chair made in Denmark with a frame of solid beech and a seat of woven papercord.",
    "The back rail is curved and the legs taper toward the floor. The seat is square with rounded \
     corners and sits on four rails.",
    "We cleaned the frame, oiled the wood and rewove the seat with new natural papercord. It works \
     at a dining table or a desk.",
    "Dimensions: W 49 × D 46 × H 78 cm, seat height 45 cm.",
];
const TITLE: &str = "Danish Beech Dining Chair with Curved Back";
const CLEAN_SEO_TITLE: &str = "Danish Beech Dining Chair with Curved Back";
const CLEAN_SEO_DESCRIPTION: &str = "Danish dining chair in solid beech with a curved back rail and \
     a woven papercord seat, cleaned and oiled, with the seat newly rewoven.";
/// 61 characters: one over the default `seo_title_max` of 60.
const LONG_SEO_TITLE: &str = "Danish Beech Dining Chairs with a Curved Back and Woven Seats";

fn writer_answer(seo_title: &str) -> Json {
    json!({
        "texts": {
            "title": {"text": TITLE, "facts": [], "reason": "From the fields."},
            "description": {"paragraphs": PARAGRAPHS, "facts": [], "reason": "From the fields."},
            "seo_title": {"text": seo_title, "facts": [], "reason": "For search."},
            "seo_description": {"text": CLEAN_SEO_DESCRIPTION, "facts": [], "reason": "For search."},
        },
        "findings": [],
        "questions": [],
    })
}

/// Scripts both writer calls with `reply`, presses Write texts and lets the run finish.
fn write(a: &TestApp, s: i64, reply: Json) -> RunRow {
    let model = &a.research_fakes().model;
    model.reply(Stage::Write, "texts", reply.clone());
    model.reply(Stage::Write, "retry", reply);
    let writer = a
        .app
        .handles()
        .writer
        .expect("the wired app hands the writer");
    assert!(writer.request(s, Origin::WriteTexts).unwrap().queued);
    for _ in 0..3 {
        a.tick(1_100).unwrap();
    }
    let runs: Vec<RunRow> = a
        .app
        .store
        .research_transaction(|tx| rstore::all_runs(tx.conn(), Some(s)))
        .unwrap()
        .into_iter()
        .filter(|r| r.trigger == RunTrigger::Write)
        .collect();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].state, RunState::Finished);
    runs.into_iter().next().unwrap()
}

fn form_value(a: &TestApp, item: i64, key: &str) -> psm_core::domain::attribute::Value {
    a.attributes(Some("shopify"))
        .form_get(item)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.fields)
        .find(|f| f.definition.key == key)
        .unwrap_or_else(|| panic!("{key} not in the form"))
        .value
        .unwrap_or_else(|| panic!("{key} has no value"))
}

#[test]
fn w5_c3a_the_writer_requests_and_writes_both_search_texts() {
    let a = writer_app();
    let (s, item) = writer_session(&a, "W5-C3A");
    write(&a, s, writer_answer(CLEAN_SEO_TITLE));
    let request = a
        .research_fakes()
        .model
        .requests()
        .into_iter()
        .find(|r| r.stage == Some(Stage::Write) && r.unit_key == "texts")
        .expect("a writer call");
    let texts: Vec<&str> = request.input["texts"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Json::as_str)
        .collect();
    assert!(texts.contains(&"seo_title"), "{texts:?}");
    assert!(texts.contains(&"seo_description"), "{texts:?}");
    for (key, text) in [
        (SEO_TITLE, CLEAN_SEO_TITLE),
        (SEO_DESCRIPTION, CLEAN_SEO_DESCRIPTION),
    ] {
        let value = form_value(&a, item, key);
        assert_eq!(value.json(), Some(json!(text)), "{key}");
        assert_eq!(value.provenance, Provenance::Ai, "{key}");
        assert_eq!(value.ai_status, Some(AiStatus::Suggested), "{key}");
        assert_eq!(value.ai_confidence, Some(AiConfidence::Medium), "{key}");
    }
}

#[test]
fn w5_c3b_a_search_title_over_the_limit_twice_is_written_low_with_its_check() {
    let a = writer_app();
    let (s, item) = writer_session(&a, "W5-C3B");
    assert_eq!(LONG_SEO_TITLE.chars().count(), 61);
    write(&a, s, writer_answer(LONG_SEO_TITLE));
    let value = form_value(&a, item, SEO_TITLE);
    assert_eq!(value.json(), Some(json!(LONG_SEO_TITLE)));
    assert_eq!(value.ai_confidence, Some(AiConfidence::Low));
    let reason = value.ai_reason.unwrap();
    let check = reason
        .split_once("Check: ")
        .unwrap_or_else(|| panic!("no Check: in {reason:?}"))
        .1;
    assert!(check.contains("61 characters (at most 60)"), "{reason:?}");
}

#[test]
fn w5_c3c_a_shop_search_title_becomes_the_suggestions_prior() {
    let a = writer_app();
    let (s, item) = writer_session(&a, "W5-C3C");
    a.attributes(Some("shopify"))
        .set_value(
            item,
            SEO_TITLE,
            json!(SHOP_SEO_TITLE),
            Provenance::Integration,
        )
        .unwrap();
    write(&a, s, writer_answer(CLEAN_SEO_TITLE));
    let value = form_value(&a, item, SEO_TITLE);
    assert_eq!(value.json(), Some(json!(CLEAN_SEO_TITLE)));
    assert_eq!(
        value.ai_prior,
        Some(AiPrior {
            value: json!(SHOP_SEO_TITLE).to_string(),
            valid: true,
            provenance: Provenance::Integration,
        })
    );
}
