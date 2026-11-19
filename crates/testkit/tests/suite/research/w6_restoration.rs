//! W6 (rows W6·C1, C2, C3(b), C4): "Restoration done" (`local.restoration`, WC8-R). The field
//! in the form (`AttributesService` through `TestApp`), a publish with the fake commerce adapter
//! and the local export, a research run and a `write` run on the wired app with the research
//! fakes. Rows C3(a) and C5 are unit tests of `never_written` and `postcheck::check`.

use std::fs;

use psm_core::{
    domain::{
        attribute::{AttributeGroup, CoreType, Definition, Preset, PresetField, Provenance},
        photo::{Classification as C, NewPhoto, PhotoSource},
        publish::DefinitionLink,
        session::PublishStep,
    },
    mechanisms::hash::sha256_file,
    services::attributes::AttributesService,
};
use psm_research::{
    domain::{RunState, RunTrigger, Stage},
    stages::OBSERVE_UNIT,
    store::{self as rstore, RunRow},
    writer::run::Origin,
};
use psm_testkit::{
    TestApp,
    fixtures::{
        definitions::{INTEGRATION, shop_definitions},
        synthetic_jpeg::SyntheticJpeg,
    },
    research_test_settings,
};
use serde_json::{Value as Json, json};

const NOW_MS: i64 = 1_763_590_000_000;
const RESTORATION: &str = "local.restoration";
const CHOICES: [&str; 14] = [
    "Cleaned",
    "Sanded",
    "Oiled",
    "Waxed",
    "Lacquered",
    "Joints re-glued",
    "Veneer repaired",
    "New foam",
    "New webbing",
    "New springs",
    "New papercord",
    "New fabric",
    "New leather",
    "Rewired",
];

fn service(a: &TestApp) -> AttributesService<'_> {
    AttributesService::new(a.store(), a.clock.as_ref(), Some(INTEGRATION))
}

fn shop_app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(NOW_MS);
    service(&a)
        .import_definitions(INTEGRATION, shop_definitions())
        .unwrap();
    a
}

fn restoration_row(a: &TestApp, item: i64) -> Option<psm_core::domain::attribute::Value> {
    service(a)
        .values(item)
        .unwrap()
        .into_iter()
        .find(|v| v.key == RESTORATION)
}

fn choose(a: &TestApp, item: i64, choices: &[&str]) {
    service(a)
        .set_value(item, RESTORATION, json!(choices), Provenance::Human)
        .unwrap();
}

// ---------------------------------------------------------------- C1: the field in the form

#[test]
fn w6_c1a_a_stored_condition_preset_ends_with_restoration_done() {
    let a = shop_app();
    let stored = ["custom.condition_cracks", "custom.condition_odor"];
    a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_preset(&Preset {
                item_type: "Chair".into(),
                group: AttributeGroup::Condition,
                layout: stored
                    .iter()
                    .map(|k| PresetField {
                        key: (*k).to_owned(),
                        default: None,
                    })
                    .collect(),
            })
        })
        .unwrap();
    let item = a
        .items()
        .create_local("W6-C1A", "", "Chair", None)
        .unwrap()
        .0
        .id;
    let step = service(&a)
        .form_get(item)
        .unwrap()
        .groups
        .into_iter()
        .find(|g| g.group == AttributeGroup::Condition)
        .unwrap();
    let keys: Vec<&str> = step
        .fields
        .iter()
        .map(|f| f.definition.key.as_str())
        .collect();
    assert_eq!(keys, [stored[0], stored[1], RESTORATION]);
    let last = &step.fields.last().unwrap().definition;
    assert_eq!(last.label, "Restoration done");
    assert_eq!(last.core_type, CoreType::MultiChoice);
    assert_eq!(last.choices, CHOICES);
}

#[test]
fn w6_c1b_two_choices_are_stored_as_a_set_by_a_human() {
    let a = shop_app();
    let item = a
        .items()
        .create_local("W6-C1B", "", "Chair", None)
        .unwrap()
        .0
        .id;
    choose(&a, item, &["Sanded", "Oiled"]);
    let row = restoration_row(&a, item).expect("a stored row");
    assert_eq!(row.value, r#"["Sanded","Oiled"]"#);
    assert!(row.valid);
    assert_eq!(row.provenance, Provenance::Human);
}

// ---------------------------------------------------------------- C2: never published, exported

/// A local item with one session and one Main photo, linked to a fake product: `(session, item)`.
fn linked_session(a: &TestApp, sku: &str) -> (i64, i64) {
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
            captured_at: Some(NOW_MS),
            source: PhotoSource::Capture,
            created_at: NOW_MS,
        })
        .unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.review().classify(photo.id, C::Main).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    let id = a.fake_shop().add_product(sku, "Oak", None);
    a.items().link(item.id, "shopify", &id, None).unwrap();
    (s, item.id)
}

fn drain(a: &TestApp) {
    for _ in 0..15 {
        a.tick(1_100).unwrap();
        a.complete_all_renders().unwrap();
    }
}

/// The fake shop's schema: one text metafield, linked (so a publish writes it).
fn shop_schema(a: &TestApp) {
    a.fake_shop().state.lock().unwrap().definitions = vec![(
        Definition {
            integration: INTEGRATION.into(),
            key: MATERIAL.into(),
            label: "Material".into(),
            core_type: CoreType::Text,
            choices: vec![],
            validations: json!({}),
            group: AttributeGroup::Details,
            description: None,
            writable: true,
        },
        DefinitionLink {
            platform_type: "single_line_text_field".into(),
            platform_meta: json!({}),
        },
    )];
    service(a).refresh_schema(a.fake_shop()).unwrap();
}

const MATERIAL: &str = "custom.material";

#[test]
fn w6_c2a_a_publish_never_sends_restoration_done() {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(NOW_MS);
    shop_schema(&a);
    let (s, item) = linked_session(&a, "W6-C2A");
    choose(&a, item, &["Sanded", "Oiled"]);
    // A shop field too, so the publish does write attributes.
    service(&a)
        .set_value(item, MATERIAL, json!("Teak"), Provenance::Human)
        .unwrap();
    a.confirm_as_shown(&[s], "Publish").unwrap();
    drain(&a);
    let p = a
        .store()
        .publish_item(a.store().latest_publish(s).unwrap().unwrap().item_id)
        .unwrap();
    assert_eq!(p.step, PublishStep::Done);
    let writes = a.fake_shop().state.lock().unwrap().attribute_writes.clone();
    let keys: Vec<String> = writes.iter().flatten().map(|w| w.key.clone()).collect();
    assert_eq!(keys, [MATERIAL]);
}

#[test]
fn w6_c2b_the_local_export_holds_restoration_done() {
    let a = shop_app();
    let (s, item) = linked_session(&a, "W6-C2B");
    choose(&a, item, &["Sanded", "Oiled"]);
    a.app.export.run(s).unwrap();
    a.complete_all_renders().unwrap();
    a.tick(0).unwrap();
    let text = fs::read_to_string(a.export_root().join("W6-C2B/item.json")).unwrap();
    let json: Json = serde_json::from_str(&text).unwrap();
    assert_eq!(json["attributes"][RESTORATION], json!(["Sanded", "Oiled"]));
}

// ---------------------------------------------------------------- C3(b), C4: research and writer

/// The wired research app with the writer's default model priced for the fake provider, and
/// `local.restoration` among the fields research is asked to fill.
fn research_app() -> TestApp {
    let a = TestApp::with_research().unwrap();
    a.clock.advance(NOW_MS);
    let mut settings = research_test_settings();
    settings["fill_keys"]
        .as_array_mut()
        .unwrap()
        .push(json!(RESTORATION));
    settings["prices"].as_array_mut().unwrap().push(json!({
        "provider": psm_research::fakes::FakeModel::ID, "model": "gpt-6.1-sol",
        "unit": "input_token", "usd_per_million": "2", "fallback_call_usd": "0.001"
    }));
    a.set_research_settings(settings).unwrap();
    a
}

/// An item with one session and one Main photo: `(session, item)`.
fn research_session(a: &TestApp, sku: &str) -> (i64, i64) {
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
    // The adjusted preview research reads.
    a.store().adjusted_event(&photo.raw_path, false).unwrap();
    let generation = a.store().adjusted_gen(&photo.raw_path).unwrap();
    let path = a
        .dir
        .path()
        .join(format!("adjusted-{}-g{generation}.jpg", photo.id));
    fs::write(&path, SyntheticJpeg::rgb(1600, 1200, [180, 90, 40])).unwrap();
    a.store()
        .complete_preview(
            &photo.raw_path,
            generation,
            path.to_str().unwrap(),
            None,
            None,
        )
        .unwrap();
    (s, item.id)
}

fn runs(a: &TestApp, s: i64) -> Vec<RunRow> {
    a.app
        .store
        .research_transaction(|tx| rstore::all_runs(tx.conn(), Some(s)))
        .unwrap()
}

#[test]
fn w6_c3b_a_research_run_never_writes_restoration_done() {
    let a = research_app();
    let (s, item) = research_session(&a, "W6-C3B");
    // The fakes propose Oiled for the field.
    let model = &a.research_fakes().model;
    model.reply(
        Stage::Observe,
        OBSERVE_UNIT,
        json!({"reads": {RESTORATION: {"value": ["Oiled"], "sure": true}},
               "marking_photos": [], "note": []}),
    );
    assert!(a.app.research.as_ref().unwrap().arranged(s).unwrap());
    for _ in 0..3 {
        a.tick(1_100).unwrap();
    }
    let run = runs(&a, s)
        .into_iter()
        .rfind(|r| r.trigger != RunTrigger::Write)
        .expect("a research run");
    assert_eq!(run.state, RunState::Finished);
    assert!(restoration_row(&a, item).is_none());
    let steps = a
        .app
        .store
        .research_transaction(|tx| rstore::steps(tx.conn(), run.id))
        .unwrap();
    let last_decide = steps
        .iter()
        .rfind(|s| s.stage == Stage::Decide.as_str())
        .expect("a decide step");
    assert!(
        last_decide.field_keys.contains(&RESTORATION.to_owned())
            && last_decide
                .detail
                .contains("Restoration done: not researched"),
        "{last_decide:?}"
    );
}

#[test]
fn w6_c4a_the_writer_sees_restoration_done_as_a_typed_fact() {
    let a = research_app();
    let (s, item) = research_session(&a, "W6-C4A");
    a.attributes(Some("shopify"))
        .set_value(item, RESTORATION, json!(["New foam"]), Provenance::Human)
        .unwrap();
    let answer = json!({
        "texts": {"title": {"text": "Danish Chair", "facts": [], "reason": "From the fields."}},
        "findings": [], "questions": [],
    });
    let model = &a.research_fakes().model;
    model.reply(Stage::Write, "texts", answer.clone());
    model.reply(Stage::Write, "retry", answer);
    let writer = a
        .app
        .handles()
        .writer
        .expect("the wired app hands the writer");
    assert!(writer.request(s, Origin::WriteTexts).unwrap().queued);
    for _ in 0..3 {
        a.tick(1_100).unwrap();
    }
    let request = a
        .research_fakes()
        .model
        .requests()
        .into_iter()
        .find(|r| r.stage == Some(Stage::Write) && r.unit_key == "texts")
        .expect("a writer call");
    let field = request.input["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["key"] == RESTORATION)
        .unwrap_or_else(|| panic!("no {RESTORATION} in {}", request.input["fields"]))
        .clone();
    assert_eq!(field["decision"], "typed", "{field}");
    assert_eq!(field["fact"], true, "{field}");
    assert_eq!(field["value"], "New foam", "{field}");
}
