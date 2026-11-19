//! W3·U1 (rows W3·C1–C3): the title suggestion in core (WC4.7) and the publish partition with the
//! built-in fields (WC4.8). Boundary: `AttributesService` and `PublishService` through `TestApp`;
//! the outcome is the form, `items.title`, the rows, the feedback rows, undo results, the publish
//! preview and items, and the calls the fake commerce adapter received. Suggestions are stored
//! through the store (as the writer leaves them) or written by a real writer run (C2(b)).

use std::fs;

use psm_core::{
    domain::{
        attribute::{AiConfidence, AiPrior, AiStatus, Provenance, Value},
        photo::{Classification as C, NewPhoto, PhotoSource},
        publish::PublishItem,
        research::FeedbackAction,
        session::PublishStep,
    },
    mechanisms::hash::sha256_file,
    services::attributes::AttributesService,
};
use psm_research::{domain::Stage, writer::run::Origin};
use psm_testkit::{RESEARCH_INTEGRATION, TestApp, research_test_settings};
use serde_json::{Value as Json, json};

const NOW_MS: i64 = 1_763_560_000_000;
const TITLE: &str = "product.title";
const DESCRIPTION: &str = "product.description";
const SHOP_TITLE: &str = "Shop chair";
const SUGGESTION: &str = "Børge Mogensen J39 Chair in Beech, 1960s";

fn shop_app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(NOW_MS);
    a
}
fn attrs(a: &TestApp) -> AttributesService<'_> {
    a.attributes(Some(RESEARCH_INTEGRATION))
}
fn text(s: &str) -> String {
    Json::String(s.into()).to_string()
}
fn photo(a: &TestApp, s: i64, name: &str) {
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(name);
    fs::write(&raw, format!("raw bytes of {name}")).unwrap();
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
}
/// An item whose title "Shop chair" came from its linked product (the link's title base), with
/// one session and one Main photo: `(session, item, product id)`.
fn linked(a: &TestApp, sku: &str) -> (i64, i64, String) {
    let (item, session) = a
        .items()
        .create_local(sku, SHOP_TITLE, "Chair", None)
        .unwrap();
    let product = a.fake_shop().add_product(sku, SHOP_TITLE, None);
    a.items().link(item.id, "shopify", &product, None).unwrap();
    a.store()
        .transaction(&mut |tx| tx.set_item_title_base(item.id, "shopify", SHOP_TITLE))
        .unwrap();
    photo(a, session.session.id, &format!("{sku}.ARW"));
    (session.session.id, item.id, product)
}
/// An unlinked item with a blank title, one session, one Main photo: `(session, item)`.
fn blank(a: &TestApp, sku: &str) -> (i64, i64) {
    let (item, session) = a.items().create_local(sku, "", "Chair", None).unwrap();
    photo(a, session.session.id, &format!("{sku}.ARW"));
    (session.session.id, item.id)
}
/// A writer suggestion as `writer::writes` stores it: `ai` + `suggested`, with `prior`.
fn suggestion(
    a: &TestApp,
    item: i64,
    key: &str,
    value: &str,
    confidence: AiConfidence,
    prior: Option<(&str, Provenance)>,
) {
    a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_value(&Value {
                item_id: item,
                key: key.into(),
                value: text(value),
                valid: true,
                provenance: Provenance::Ai,
                ai_confidence: Some(confidence),
                ai_status: Some(AiStatus::Suggested),
                ai_reason: Some("From the fields.\nFacts: Designer".into()),
                ai_sources: Some(Vec::new()),
                ai_prior: prior.map(|(value, provenance)| AiPrior {
                    value: text(value),
                    valid: true,
                    provenance,
                }),
                updated_at: NOW_MS,
            })
        })
        .unwrap();
}
/// The C1(a) suggestion: the writer's title over the shop's title.
fn title_suggestion(a: &TestApp, item: i64) {
    suggestion(
        a,
        item,
        TITLE,
        SUGGESTION,
        AiConfidence::Medium,
        Some((SHOP_TITLE, Provenance::Integration)),
    );
}
fn row(a: &TestApp, item: i64, key: &str) -> Option<Value> {
    let mut out = None;
    a.store()
        .transaction(&mut |tx| {
            out = tx
                .attribute_values(item)?
                .into_iter()
                .find(|v| v.key == key);
            Ok(())
        })
        .unwrap();
    out
}
fn feedback(a: &TestApp, item: i64, key: &str) -> Vec<psm_core::domain::research::Feedback> {
    let mut out = Vec::new();
    a.store()
        .transaction(&mut |tx| {
            out = tx.feedback_by_key(item, key)?;
            Ok(())
        })
        .unwrap();
    out
}
fn item_title(a: &TestApp, item: i64) -> String {
    a.store().item(item).unwrap().title
}
fn form_title(a: &TestApp, item: i64) -> Option<Value> {
    attrs(a)
        .form_get(item)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.fields)
        .find(|f| f.definition.key == TITLE)
        .expect("the title is in the form")
        .value
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
fn ops(a: &TestApp, op: &str) -> usize {
    a.fake_shop()
        .mutation_ops()
        .iter()
        .filter(|o| *o == op)
        .count()
}
fn remote_title(a: &TestApp, product: &str) -> String {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .products
        .iter()
        .find(|p| p.external_id == product)
        .unwrap()
        .title
        .clone()
}

#[test]
fn w3_c1a_the_form_shows_the_title_suggestion_over_the_shops_title() {
    let a = shop_app();
    let (_, item, _) = linked(&a, "W3-C1A");
    title_suggestion(&a, item);
    let shown = form_title(&a, item).expect("the title has a value");
    assert_eq!(shown.value, text(SUGGESTION));
    assert_eq!(shown.provenance, Provenance::Ai);
    assert_eq!(shown.ai_status, Some(AiStatus::Suggested));
    assert_eq!(shown.ai_prior.unwrap().value, text(SHOP_TITLE));
    assert_eq!(item_title(&a, item), SHOP_TITLE);
}

#[test]
fn w3_c1b_typing_over_the_suggestion_removes_it_with_one_edited_feedback() {
    let a = shop_app();
    let (_, item, _) = linked(&a, "W3-C1B");
    title_suggestion(&a, item);
    attrs(&a)
        .set_value(item, TITLE, json!("My title"), Provenance::Human)
        .unwrap();
    let shown = form_title(&a, item).unwrap();
    assert_eq!(shown.value, text("My title"));
    assert_eq!(shown.provenance, Provenance::Human);
    assert_eq!(item_title(&a, item), "My title");
    assert!(row(&a, item, TITLE).is_none());
    let rows = feedback(&a, item, TITLE);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].action, FeedbackAction::Edited);
    assert_eq!(rows[0].suggested_value, text(SUGGESTION));
    assert_eq!(
        rows[0].final_value.as_deref(),
        Some(text("My title").as_str())
    );
}

#[test]
fn w3_c2a_accept_sets_the_items_title_and_undo_redo_move_both_together() {
    let a = shop_app();
    let (s, item, _) = linked(&a, "W3-C2A");
    title_suggestion(&a, item);
    attrs(&a).accept(item, TITLE).unwrap();
    let after_accept = |a: &TestApp| {
        assert_eq!(item_title(a, item), SUGGESTION);
        let r = row(a, item, TITLE).unwrap();
        assert_eq!(
            (r.provenance, r.ai_status),
            (Provenance::Ai, Some(AiStatus::Accepted))
        );
        let rows = feedback(a, item, TITLE);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0].action, FeedbackAction::Accepted);
    };
    after_accept(&a);
    a.history().undo(s).unwrap();
    assert_eq!(item_title(&a, item), SHOP_TITLE);
    let r = row(&a, item, TITLE).unwrap();
    assert_eq!(
        (r.provenance, r.ai_status),
        (Provenance::Ai, Some(AiStatus::Suggested))
    );
    assert_eq!(r.ai_prior.unwrap().value, text(SHOP_TITLE));
    assert!(feedback(&a, item, TITLE).is_empty());
    a.history().redo(s).unwrap();
    after_accept(&a);
}

/// The price of the writer's default model for the fake provider (the research test settings
/// price only `m1`).
fn price_writer_model(a: &TestApp) {
    let mut settings = research_test_settings();
    settings["prices"].as_array_mut().unwrap().push(json!({
        "provider": psm_research::fakes::FakeModel::ID, "model": "gpt-6.1-sol",
        "unit": "input_token", "usd_per_million": "2", "fallback_call_usd": "0.001"
    }));
    a.set_research_settings(settings).unwrap();
}
const CLEAN_PARAGRAPHS: [&str; 4] = [
    "A dining chair made in Denmark with a frame of solid beech and a seat of woven papercord.",
    "The back rail is curved and the legs taper toward the floor. The seat is square with rounded \
     corners and sits on four rails.",
    "We cleaned the frame, oiled the wood and rewove the seat with new natural papercord. It works \
     at a dining table or a desk.",
    "Dimensions: W 49 × D 46 × H 78 cm, seat height 45 cm.",
];
fn writer_answer(title: &str) -> Json {
    json!({
        "texts": {
            "title": {"text": title, "facts": [], "reason": "From the fields."},
            "description": {"paragraphs": CLEAN_PARAGRAPHS, "facts": [], "reason": "From the note."},
        },
        "findings": [],
        "questions": [],
    })
}

#[test]
fn w3_c2b_keep_previous_deletes_the_suggestion_and_the_writer_never_writes_it_again() {
    let a = TestApp::with_research().unwrap();
    a.clock.advance(NOW_MS);
    price_writer_model(&a);
    let (s, item, _) = linked(&a, "W3-C2B");
    title_suggestion(&a, item);
    attrs(&a).keep_previous(item, TITLE).unwrap();
    assert!(row(&a, item, TITLE).is_none());
    assert_eq!(item_title(&a, item), SHOP_TITLE);
    let rows = feedback(&a, item, TITLE);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].action, FeedbackAction::KeptPrevious);
    assert_eq!(rows[0].suggested_value, text(SUGGESTION));
    assert_eq!(form_title(&a, item).unwrap().value, text(SHOP_TITLE));

    // A later writer run proposes the same title: not written (it is in the cleared set), while
    // its description is.
    let fakes = a.research_fakes();
    fakes
        .model
        .reply(Stage::Write, "texts", writer_answer(SUGGESTION));
    fakes
        .model
        .reply(Stage::Write, "retry", writer_answer(SUGGESTION));
    let writer = a
        .app
        .handles()
        .writer
        .expect("the wired app hands the writer");
    let answer = writer.request(s, Origin::WriteTexts).unwrap();
    assert!(answer.queued, "{answer:?}");
    for _ in 0..3 {
        a.tick(1_100).unwrap();
    }
    assert_eq!(
        row(&a, item, DESCRIPTION).map(|r| (r.provenance, r.ai_status)),
        Some((Provenance::Ai, Some(AiStatus::Suggested))),
        "the writer run wrote its texts"
    );
    assert!(row(&a, item, TITLE).is_none());
    assert_eq!(item_title(&a, item), SHOP_TITLE);
}

#[test]
fn w3_c2c_dismissing_a_title_suggestion_on_a_blank_title_clears_it() {
    let a = shop_app();
    let (_, item) = blank(&a, "W3-C2C");
    suggestion(&a, item, TITLE, SUGGESTION, AiConfidence::Medium, None);
    attrs(&a).clear(item, TITLE).unwrap();
    let r = row(&a, item, TITLE).unwrap();
    assert_eq!(
        (r.provenance, r.ai_status),
        (Provenance::Ai, Some(AiStatus::Cleared))
    );
    assert_eq!(item_title(&a, item), "");
    let rows = feedback(&a, item, TITLE);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].action, FeedbackAction::Cleared);
    assert!(form_title(&a, item).is_none());
}

#[test]
fn w3_c2d_accept_all_takes_the_title_and_skips_a_low_description() {
    let a = shop_app();
    let (_, item) = blank(&a, "W3-C2D");
    suggestion(&a, item, TITLE, SUGGESTION, AiConfidence::Medium, None);
    suggestion(&a, item, DESCRIPTION, "A chair.", AiConfidence::Low, None);
    attrs(&a).accept_all(item).unwrap();
    assert_eq!(item_title(&a, item), SUGGESTION);
    assert_eq!(
        row(&a, item, TITLE).unwrap().ai_status,
        Some(AiStatus::Accepted)
    );
    assert_eq!(
        row(&a, item, DESCRIPTION).unwrap().ai_status,
        Some(AiStatus::Suggested)
    );
}

#[test]
fn w3_c3a_a_writer_description_over_the_shops_is_a_conflict_that_holds_the_item() {
    let a = shop_app();
    let (s, item, _) = linked(&a, "W3-C3A");
    suggestion(
        &a,
        item,
        DESCRIPTION,
        "A beech chair by Børge Mogensen.",
        AiConfidence::Medium,
        Some(("Shop text", Provenance::Integration)),
    );
    let preview = a.app.publish.as_ref().unwrap().preview(&[s]).unwrap();
    let conflicts: Vec<_> = preview.items[0]
        .conflicts
        .iter()
        .map(|c| c.key.as_str())
        .collect();
    assert_eq!(conflicts, [DESCRIPTION]);
    a.confirm_as_shown(&[s], "Publish").unwrap();
    assert_eq!(latest(&a, s).step, PublishStep::Held);
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Held);
    assert!(
        a.fake_shop().mutation_ops().is_empty(),
        "{:?}",
        a.fake_shop().mutation_ops()
    );
}

#[test]
fn w3_c3b_an_unaccepted_title_suggestion_neither_holds_nor_reaches_the_shop() {
    let a = shop_app();
    let (s, item, product) = linked(&a, "W3-C3B");
    suggestion(&a, item, TITLE, SUGGESTION, AiConfidence::Medium, None);
    let preview = a.app.publish.as_ref().unwrap().preview(&[s]).unwrap();
    assert_eq!(preview.items[0].n_withheld, 0);
    assert!(preview.items[0].conflicts.is_empty());
    a.confirm_as_shown(&[s], "Publish").unwrap();
    let p = latest(&a, s);
    assert!(!p.frozen_attributes.contains_key(TITLE));
    assert!(!p.frozen_attribute_origin.contains_key(TITLE));
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert_eq!(ops(&a, "update_title"), 0);
    assert_eq!(remote_title(&a, &product), SHOP_TITLE);
}

#[test]
fn w3_c3c_an_accepted_title_suggestion_is_sent_once() {
    let a = shop_app();
    let (s, item, product) = linked(&a, "W3-C3C");
    suggestion(&a, item, TITLE, SUGGESTION, AiConfidence::Medium, None);
    attrs(&a).accept(item, TITLE).unwrap();
    a.confirm_as_shown(&[s], "Publish").unwrap();
    drain(&a);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert_eq!(ops(&a, "update_title"), 1);
    assert_eq!(remote_title(&a, &product), SUGGESTION);
}
