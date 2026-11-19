//! W5 (rows W5·C1–C3 and W5·C6–C9): the two search texts. U1: the built-in fields, always present
//! in the Product step (`form_get`, `layout_keys`, `presets_set`), read from the linked product
//! (the P67 remote read), and requested and written by the writer (a `write` run on the wired app
//! with the research fakes). U3: the publish step (`PublishService` through `TestApp` with the
//! fake commerce adapter).

use std::{
    fs,
    sync::{Arc, Mutex},
};

use psm_core::{
    domain::{
        attribute::{
            AiConfidence, AiPrior, AiStatus, AttributeGroup, Preset, PresetField, Provenance,
        },
        photo::{Classification as C, NewPhoto, PhotoSource},
        publish::{PublishItem, Seo},
        session::PublishStep,
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

// ---------------------------------------------------------------- U3: the publish step

const PUBLISH_NOW_MS: i64 = 1_763_580_000_000;

fn publish_app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(PUBLISH_NOW_MS);
    a
}

/// A local item with one session and one Main photo, linked to a fake product:
/// `(session, item, product id)`.
fn linked_session(a: &TestApp, sku: &str) -> (i64, i64, String) {
    let (item, session) = a.items().create_local(sku, "Oak", "Chair", None).unwrap();
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
            captured_at: Some(PUBLISH_NOW_MS),
            source: PhotoSource::Capture,
            created_at: PUBLISH_NOW_MS,
        })
        .unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.review().classify(photo.id, C::Main).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    let id = a.fake_shop().add_product(sku, "Oak", None);
    a.items().link(item.id, "shopify", &id, None).unwrap();
    (s, item.id, id)
}

fn human(a: &TestApp, item: i64, key: &str, text: &str) {
    a.attributes(Some("shopify"))
        .set_value(item, key, json!(text), Provenance::Human)
        .unwrap();
}

/// An `ai` + `suggested` row stored as the writer leaves one.
fn suggestion(
    a: &TestApp,
    item: i64,
    key: &str,
    text: &str,
    confidence: AiConfidence,
    prior: Option<&str>,
) {
    a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_value(&psm_core::domain::attribute::Value {
                item_id: item,
                key: key.into(),
                value: json!(text).to_string(),
                valid: true,
                provenance: Provenance::Ai,
                ai_confidence: Some(confidence),
                ai_status: Some(AiStatus::Suggested),
                ai_reason: Some("For search.".into()),
                ai_sources: Some(Vec::new()),
                ai_prior: prior.map(|p| AiPrior {
                    value: json!(p).to_string(),
                    valid: true,
                    provenance: Provenance::Integration,
                }),
                updated_at: PUBLISH_NOW_MS,
            })
        })
        .unwrap();
}

fn drain(a: &TestApp) {
    for _ in 0..15 {
        a.tick(1_100).unwrap();
        a.complete_all_renders().unwrap();
    }
}

fn latest(a: &TestApp, s: i64) -> PublishItem {
    a.store()
        .publish_item(a.store().latest_publish(s).unwrap().unwrap().item_id)
        .unwrap()
}

fn seo_writes(a: &TestApp) -> Vec<Seo> {
    a.fake_shop().state.lock().unwrap().seo_writes.clone()
}

fn seo_reads(a: &TestApp) -> usize {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .reads
        .iter()
        .filter(|op| *op == "product_seo")
        .count()
}

fn seo(title: Option<&str>, description: Option<&str>) -> Seo {
    Seo {
        title: title.map(Into::into),
        description: description.map(Into::into),
    }
}

#[test]
fn w5_c6a_a_frozen_search_title_is_written_with_the_remote_description_kept() {
    let a = publish_app();
    let (s, item, id) = linked_session(&a, "W5-C6A");
    human(&a, item, SEO_TITLE, "Teak chair, 1960s");
    a.fake_shop().set_seo(&id, None, Some("Shop meta"));
    // At each `product_seo` read: how many `update_seo` calls the shop had received.
    let writes_at_read = Arc::new(Mutex::new(Vec::new()));
    let seen = writes_at_read.clone();
    let shop = a.commerce.clone().unwrap();
    a.fake_shop().set_hook(Some(Box::new(move |op| {
        if op == "product_seo" {
            let writes = shop.state.lock().unwrap().seo_writes.len();
            seen.lock().unwrap().push(writes);
        }
    })));
    a.confirm_as_shown(&[s], "Publish").unwrap();
    drain(&a);
    a.fake_shop().set_hook(None);
    assert_eq!(
        seo_writes(&a),
        [seo(Some("Teak chair, 1960s"), Some("Shop meta"))]
    );
    assert_eq!(*writes_at_read.lock().unwrap(), [0, 1]);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
}

#[test]
fn w5_c6b_search_texts_equal_after_trimming_are_not_written() {
    let a = publish_app();
    let (s, item, id) = linked_session(&a, "W5-C6B");
    human(&a, item, SEO_TITLE, "Teak chair, 1960s");
    human(&a, item, SEO_DESCRIPTION, "Shop meta");
    a.fake_shop()
        .set_seo(&id, Some("  Teak chair, 1960s "), Some("Shop meta\n"));
    a.confirm_as_shown(&[s], "Publish").unwrap();
    drain(&a);
    assert!(seo_writes(&a).is_empty());
    assert!(
        !a.fake_shop()
            .mutation_ops()
            .contains(&"update_seo".to_owned())
    );
    assert_eq!(latest(&a, s).step, PublishStep::Done);
}

#[test]
fn w5_c6c_no_frozen_search_text_reads_and_writes_nothing() {
    let a = publish_app();
    let (s, _, id) = linked_session(&a, "W5-C6C");
    a.fake_shop()
        .set_seo(&id, Some("Shop title"), Some("Shop meta"));
    a.confirm_as_shown(&[s], "Publish").unwrap();
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert_eq!(seo_reads(&a), 0);
    assert!(seo_writes(&a).is_empty());
}

#[test]
fn w5_c7a_a_frozen_description_alone_sends_the_unset_title_as_null() {
    let a = publish_app();
    let (s, item, _) = linked_session(&a, "W5-C7A");
    human(&a, item, SEO_DESCRIPTION, CLEAN_SEO_DESCRIPTION);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    drain(&a);
    assert_eq!(seo_writes(&a), [seo(None, Some(CLEAN_SEO_DESCRIPTION))]);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
}

#[test]
fn w5_c8a_a_search_title_read_back_different_fails_the_update_naming_it() {
    let a = publish_app();
    let (s, item, _) = linked_session(&a, "W5-C8A");
    let title = "Danish Teak Lounge Chair with a New Papercord Seat and an Oiled Frame 1960s";
    assert_eq!(title.chars().count(), 75);
    human(&a, item, SEO_TITLE, title);
    a.fake_shop().state.lock().unwrap().seo_title_cut = Some(70);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    drain(&a);
    let p = latest(&a, s);
    assert_eq!(p.step, PublishStep::Failed);
    assert_eq!(p.failed_step, Some(PublishStep::Update));
    let error = p.error.unwrap();
    assert!(error.contains(SEO_TITLE), "{error}");
    assert!(!error.contains(SEO_DESCRIPTION), "{error}");
}

#[test]
fn w5_c9a_a_kill_after_the_search_write_resumes_without_writing_again() {
    let mut a = publish_app();
    let (s, item, id) = linked_session(&a, "W5-C9A");
    human(&a, item, SEO_TITLE, "Teak chair, 1960s");
    human(&a, item, SEO_DESCRIPTION, CLEAN_SEO_DESCRIPTION);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    a.fake_shop().fail_after_effect("update_seo");
    let killed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drain(&a)));
    assert!(killed.is_err(), "the process was not killed at update_seo");
    assert_eq!(seo_writes(&a).len(), 1);
    a.kill_and_restart().unwrap();
    drain(&a);
    assert_eq!(seo_writes(&a).len(), 1, "a second update_seo after resume");
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert_eq!(
        a.fake_shop().seo_of(&id),
        seo(Some("Teak chair, 1960s"), Some(CLEAN_SEO_DESCRIPTION))
    );
}

#[test]
fn w5_c9b_a_low_search_title_without_prior_is_withheld_and_not_sent() {
    let a = publish_app();
    let (s, item, _) = linked_session(&a, "W5-C9B");
    suggestion(
        &a,
        item,
        SEO_TITLE,
        CLEAN_SEO_TITLE,
        AiConfidence::Low,
        None,
    );
    let preview = a.app.publish.as_ref().unwrap().preview(&[s]).unwrap();
    assert_eq!(preview.items[0].n_withheld, 1);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    drain(&a);
    let p = latest(&a, s);
    assert_eq!(p.frozen_attribute_origin[SEO_TITLE]["class"], "withheld");
    assert_eq!(p.step, PublishStep::Done);
    assert!(seo_writes(&a).is_empty());
}

#[test]
fn w5_c9c_a_search_description_with_a_prior_holds_until_use_suggestion() {
    let a = publish_app();
    let (s, item, id) = linked_session(&a, "W5-C9C");
    a.fake_shop().set_seo(&id, None, Some("Shop meta"));
    suggestion(
        &a,
        item,
        SEO_DESCRIPTION,
        CLEAN_SEO_DESCRIPTION,
        AiConfidence::Medium,
        Some("Shop meta"),
    );
    let preview = a.app.publish.as_ref().unwrap().preview(&[s]).unwrap();
    let conflicts: Vec<&str> = preview.items[0]
        .conflicts
        .iter()
        .map(|c| c.key.as_str())
        .collect();
    assert_eq!(conflicts, [SEO_DESCRIPTION]);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    assert_eq!(latest(&a, s).step, PublishStep::Held);
    drain(&a);
    assert!(seo_writes(&a).is_empty());
    a.attributes(Some("shopify"))
        .accept(item, SEO_DESCRIPTION)
        .unwrap();
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert_eq!(seo_writes(&a), [seo(None, Some(CLEAN_SEO_DESCRIPTION))]);
}
