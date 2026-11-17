//! R4·U3: `ResearchService::execute` on the harness with scripted fakes and a `running` run row.
//! Shop host `beyo.example`; trusted `[e1.example, e2.example]`; layout: designer (text, the
//! attribution key), woods (multi-choice), width (dimension), scratches (severity, Condition
//! step), title (Product step).

use psm_core::domain::{
    attribute::{AiConfidence, AiStatus, Provenance, Value},
    photo::Classification,
    research::FeedbackAction,
};
use psm_research::{
    config::ResearchConfig,
    domain::{RunState, Stage},
    fakes::{self, ModelScript, PageScript},
    ports::SearchCandidate,
    rules::{fingerprint::fingerprint, sites::page_identity},
    stages::{ExecuteOutcome, OBSERVE_UNIT},
};
use psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg;
use serde_json::{Value as Json, json};

use crate::harness::{DESIGNER, Harness, SCRATCHES, TITLE, WIDTH, WOODS};

pub(crate) const P_E1: &str = "https://www.e1.example/p/1";
pub(crate) const P_E2: &str = "https://e2.example/item/2";
pub(crate) const P_UNTRUSTED: &str = "https://other.example/x";
const P_SHOP: &str = "https://beyo.example/products/another-chair";

fn u(url: &str) -> String {
    page_identity(url).unwrap()
}

fn image(n: u8) -> Vec<u8> {
    SyntheticJpeg::rgb(32, 24, [n, 255 - n, n / 2])
}

/// Search results in the given order, ranked 1, 2, …
fn results(h: &Harness, urls: &[&str]) {
    h.search.set_results(
        urls.iter()
            .enumerate()
            .map(|(i, url)| SearchCandidate {
                url: (*url).to_owned(),
                rank: u32::try_from(i + 1).unwrap(),
                title: String::new(),
            })
            .collect(),
    );
}

/// A readable page with one image of its own.
fn page(h: &Harness, url: &str, n: u8) {
    h.reader
        .set_page(url, PageScript::page(&format!("Page {n}"), vec![image(n)]));
}

/// The page's read reply states `stated` for `key` (one key per page in these rows).
fn states(h: &Harness, url: &str, key: &str, stated: Json) {
    h.model
        .reply(Stage::Read, &u(url), json!({ "keys": { key: stated } }));
}

fn designer(name: &str) -> Json {
    json!({"stated": name, "value": name})
}

fn verdict(h: &Harness, url: &str, verdict: &str) {
    h.model
        .reply(Stage::Compare, &u(url), json!({ "verdict": verdict }));
}

/// The reply of the run's one Observe unit (R12·U1).
pub(crate) fn observe(h: &Harness, reply: Json) {
    h.model.reply(Stage::Observe, OBSERVE_UNIT, reply);
}

fn row(h: &Harness, key: &str) -> Option<Value> {
    h.row(key)
}

fn ai(row: &Option<Value>) -> (Json, AiConfidence, Provenance, AiStatus) {
    let row = row.as_ref().expect("a row");
    (
        serde_json::from_str(&row.value).unwrap(),
        row.ai_confidence.unwrap(),
        row.provenance,
        row.ai_status.unwrap(),
    )
}

/// C1's run: two main photos and one condition photo; e1 and e2 state designer A and match;
/// an untrusted page states nothing; observe reads Teak (sure); condition reads Minor (sure).
struct Full {
    h: Harness,
    run: i64,
    mains: Vec<i64>,
    condition: i64,
}

fn full() -> Full {
    let h = Harness::chair();
    let (mains, condition) = full_script(&h);
    let run = h.start_run(&h.stage_config());
    Full {
        h,
        run,
        mains,
        condition,
    }
}

/// C1's photos, search results, pages and scripted fake replies on a Chair harness, without the
/// run (R8: the adapter rows run this script with a real adapter in place of one fake). Returns
/// the two main photos and the condition photo.
pub(crate) fn full_script(h: &Harness) -> (Vec<i64>, i64) {
    let mains = vec![
        h.add_main_photo_with_preview(h.session_id),
        h.add_main_photo_with_preview(h.session_id),
    ];
    let condition = h.add_photo_with_preview(h.session_id, Classification::Condition);
    results(h, &[P_E1, P_E2, P_UNTRUSTED]);
    for (n, url) in [P_E1, P_E2, P_UNTRUSTED].iter().enumerate() {
        page(h, url, u8::try_from(n + 1).unwrap() * 40);
    }
    states(h, P_E1, DESIGNER, designer("Designer A"));
    states(h, P_E2, DESIGNER, designer("Designer A"));
    verdict(h, P_E1, "match");
    verdict(h, P_E2, "match");
    verdict(h, P_UNTRUSTED, "different");
    observe(
        h,
        json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}, "marking_photos": []}),
    );
    h.model.reply(
        Stage::Condition,
        "condition",
        json!({"reads": {SCRATCHES: {"value": "Minor", "sure": true}}}),
    );
    h.model.reply(
        Stage::Decide,
        "decide",
        json!({"reasons": {DESIGNER: "Two trusted dealers name Designer A."}}),
    );
    (mains, condition)
}

#[test]
fn a12_c1a_a_full_run_fills_the_form() {
    let f = full();
    assert_eq!(f.h.execute(f.run), ExecuteOutcome::Finished);
    assert_form_filled(&f.h, f.run);
}

/// What C1's run writes: designer A (medium, from e1 and e2), Teak, Minor; the run finished at
/// Decide.
pub(crate) fn assert_form_filled(h: &Harness, run_id: i64) {
    let designer_row = row(h, DESIGNER);
    assert_eq!(
        ai(&designer_row),
        (
            json!("Designer A"),
            AiConfidence::Medium,
            Provenance::Ai,
            AiStatus::Suggested
        )
    );
    assert_eq!(
        designer_row.unwrap().ai_sources.unwrap(),
        vec![u(P_E1), u(P_E2)]
    );
    assert_eq!(
        ai(&row(h, WOODS)),
        (
            json!(["Teak"]),
            AiConfidence::Medium,
            Provenance::Ai,
            AiStatus::Suggested
        )
    );
    assert_eq!(
        ai(&row(h, SCRATCHES)),
        (
            json!("Minor"),
            AiConfidence::Medium,
            Provenance::Ai,
            AiStatus::Suggested
        )
    );
    let run = h.run_row(run_id);
    assert_eq!(run.state, RunState::Finished);
    assert_eq!(run.stage_reached.as_deref(), Some("decide"));
}

#[test]
fn a12_c1b_steps_are_gapless_and_in_stage_order() {
    let f = full();
    f.h.execute(f.run);
    let steps = f.h.steps(f.run);
    assert_eq!(
        steps.iter().map(|s| s.seq).collect::<Vec<_>>(),
        (1..=i64::try_from(steps.len()).unwrap()).collect::<Vec<_>>()
    );
    let kinds: Vec<&str> = steps.iter().map(|s| s.kind.as_str()).collect();
    assert_eq!(
        kinds,
        vec![
            "looked",
            "searched",
            "read",
            "read",
            "read",
            "compared",
            "compared",
            "compared",
            "looked",
            "concluded",
            "concluded",
            "concluded",
            "not_determined",
        ]
    );
    assert_eq!(steps[0].photo_ids, f.mains);
    assert_eq!(steps[8].photo_ids, vec![f.condition]);
    for key in [DESIGNER, WOODS, SCRATCHES] {
        assert!(steps.iter().any(|s| s.field_keys.iter().any(|k| k == key)));
    }
}

#[test]
fn a12_c1c_stage_outputs_one_per_unit_with_fingerprints() {
    let f = full();
    f.h.execute(f.run);
    let config = f.h.stage_config();
    let outputs = f.h.outputs(f.run);
    let keys: Vec<(String, String)> = outputs
        .iter()
        .map(|o| (o.stage.clone(), o.unit_key.clone()))
        .collect();
    let mut expected: Vec<(String, String)> = vec![("observe".into(), OBSERVE_UNIT.into())];
    expected.push(("find".into(), "cover".into()));
    for url in [P_E1, P_E2, P_UNTRUSTED] {
        expected.push(("read".into(), u(url)));
    }
    for url in [P_E1, P_E2, P_UNTRUSTED] {
        expected.push(("compare".into(), u(url)));
    }
    expected.push(("condition".into(), "condition".into()));
    expected.push(("decide".into(), "decide".into()));
    assert_eq!(keys, expected);
    // Each fingerprint is R3·U2's over the unit's MC-A7 identity.
    let revision = |p: i64| f.h.app.composition(p).unwrap().revision;
    let observe = config.stage(Stage::Observe);
    let first = &outputs[0];
    assert_eq!(
        first.fingerprint,
        fingerprint(
            1,
            &json!({
                "photos": [
                    [f.mains[0], revision(f.mains[0]), "wide"],
                    [f.mains[1], revision(f.mains[1]), "closer"],
                ],
                "long_edge": 1024,
                "model": observe.model,
                "effort": observe.effort,
                "instructions_version": 2,
                "language": "English",
            })
        )
        .unwrap()
    );
    assert_eq!(
        outputs[1].fingerprint,
        fingerprint(
            2,
            &json!({
                "cover_photo_id": f.mains[0],
                "composition_rev": revision(f.mains[0]),
                "provider": "fake-search",
            })
        )
        .unwrap()
    );
    let page_images: Vec<String> =
        serde_json::from_str::<Json>(&outputs[2].output).unwrap()["images"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_owned())
            .collect();
    assert_eq!(
        outputs[5].fingerprint,
        fingerprint(
            4,
            &json!({
                "page": u(P_E1),
                "images": page_images,
                "cover": [f.mains[0], revision(f.mains[0])],
            })
        )
        .unwrap()
    );
    let decide: Json = serde_json::from_str(&outputs[9].output).unwrap();
    let designer = &decide["writes"][DESIGNER];
    assert_eq!(designer["value"], json!("Designer A"));
    assert_eq!(designer["confidence"], json!("medium"));
    assert_eq!(
        designer["reason"],
        json!("Two trusted dealers name Designer A.")
    );
    assert_eq!(designer["sources"], json!([u(P_E1), u(P_E2)]));
    assert_eq!(designer["outcome"], json!("written"));
    let mut fps: Vec<&str> = outputs[..9]
        .iter()
        .map(|o| o.fingerprint.as_str())
        .collect();
    fps.sort();
    let form: psm_research::service::FormSnapshot =
        serde_json::from_str(f.h.run_row(f.run).form_snapshot.as_deref().unwrap()).unwrap();
    assert_eq!(
        outputs[9].fingerprint,
        fingerprint(
            7,
            &json!({
                "units": fps,
                "layout": form.fields.iter().map(|f| json!({
                    "key": f.definition.key,
                    "core_type": f.definition.core_type,
                    "choices": f.definition.choices,
                    "step": f.step,
                })).collect::<Vec<_>>(),
                "attribution_keys": [DESIGNER],
                "fill_keys": config.fill_keys,
                "detailed_condition": config.detailed_condition,
            })
        )
        .unwrap()
    );
}

fn reader_urls(h: &Harness) -> Vec<String> {
    h.reader
        .requests()
        .into_iter()
        .map(|c| c.url.unwrap())
        .collect()
}

#[test]
fn a12_c2a_reading_order_shop_trusted_untrusted() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    results(&h, &[P_UNTRUSTED, P_E2, P_E1, P_SHOP]);
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    assert_eq!(reader_urls(&h), vec![P_SHOP, P_E1, P_E2, P_UNTRUSTED]);
}

#[test]
fn a12_c2b_page_limit_counts_obtained_pages() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    let fourth = "https://other.example/y";
    results(&h, &[P_E1, P_E2, P_UNTRUSTED, fourth]);
    h.reader.set_page(P_E1, PageScript::unobtainable());
    let mut config = h.stage_config();
    config.max_pages = 2;
    let run = h.start_run(&config);
    h.execute(run);
    assert_eq!(reader_urls(&h), vec![P_E1, P_E2, P_UNTRUSTED]);
    let reads: Vec<_> = h
        .steps(run)
        .into_iter()
        .filter(|s| s.kind == "read")
        .collect();
    assert_eq!(reads[0].summary, "could not be read");
    assert_eq!(reads[0].source_url.as_deref(), Some(P_E1));
    // A7: the status view marks exactly that step `unread` (shown as a warning).
    let view = psm_research::api::ResearchApi::status(h.service.as_ref(), h.session_id).unwrap();
    let unread: Vec<_> = view
        .steps
        .iter()
        .filter(|s| s.unread)
        .map(|s| (s.kind.as_str(), s.source_url.as_deref()))
        .collect();
    assert_eq!(unread, vec![("read", Some(P_E1))]);
    let obtained = h
        .outputs(run)
        .iter()
        .filter(|o| o.stage == "read")
        .filter(|o| serde_json::from_str::<Json>(&o.output).unwrap()["obtained"] == json!(true))
        .count();
    assert_eq!(obtained, 2);
}

#[test]
fn a12_c2c_own_listing_is_never_read_nor_evidence() {
    let h = Harness::chair();
    h.link_item("gid://shopify/Product/77");
    h.app.fake_shop().set_public_listing(
        vec!["beyo.example".into(), "beyo-store.myshopify.com".into()],
        Some("https://beyo.example/products/oak-chair".into()),
        "/products/oak-chair".into(),
    );
    h.add_main_photo_with_preview(h.session_id);
    let own = "https://beyo.example/products/oak-chair?variant=3";
    results(&h, &[own]);
    page(&h, own, 9);
    states(&h, own, DESIGNER, designer("Designer A"));
    verdict(&h, own, "match");
    let run = h.start_run(&h.stage_config());
    assert_eq!(h.execute(run), ExecuteOutcome::Finished);
    assert!(h.reader.requests().is_empty());
    let steps = h.steps(run);
    assert!(
        steps
            .iter()
            .any(|s| s.summary == "this item's own listing" && s.source_url.as_deref() == Some(own))
    );
    assert!(row(&h, DESIGNER).is_none());
}

#[test]
fn a12_c2d_redirected_page_counts_as_its_final_site() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    let candidate = "https://redirector.example/go/1";
    let final_url = "https://shop.e1.example/item/9";
    results(&h, &[candidate, P_E2]);
    h.reader.set_page(
        candidate,
        PageScript {
            final_url: Some(final_url.into()),
            ..PageScript::page("Redirected", vec![image(3)])
        },
    );
    page(&h, P_E2, 4);
    states(&h, candidate, DESIGNER, designer("Designer A"));
    states(&h, P_E2, DESIGNER, designer("Designer A"));
    verdict(&h, candidate, "match");
    verdict(&h, P_E2, "match");
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    let read = h
        .outputs(run)
        .into_iter()
        .find(|o| o.stage == "read" && o.unit_key == u(candidate))
        .unwrap();
    let output: Json = serde_json::from_str(&read.output).unwrap();
    assert_eq!(
        output["site"],
        json!({"class": "trusted", "identity": "e1.example"})
    );
    let designer_row = row(&h, DESIGNER).unwrap();
    assert_eq!(designer_row.ai_confidence, Some(AiConfidence::Medium));
    assert_eq!(
        designer_row.ai_sources.unwrap(),
        // Read in candidate order (the redirector is untrusted, so after e2); counted as e1.
        vec![u(P_E2), u(final_url)]
    );
}

#[test]
fn a12_c3a_units_and_image_sizes() {
    let h = Harness::chair();
    let cover = h.add_main_photo_with_preview(h.session_id);
    let marked = h.add_main_photo_with_preview(h.session_id);
    let unreviewed = h.add_photo_with_preview(h.session_id, Classification::Unreviewed);
    let c1 = h.add_photo_with_preview(h.session_id, Classification::Condition);
    let c2 = h.add_photo_with_preview(h.session_id, Classification::Condition);
    observe(&h, json!({"reads": {}, "marking_photos": [2]}));
    results(&h, &[P_E1]);
    page(&h, P_E1, 7);
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    let log = h.model.requests();
    let of = |stage: Stage| -> Vec<&psm_research::fakes::FakeCall> {
        log.iter().filter(|c| c.stage == Some(stage)).collect()
    };
    let observed = of(Stage::Observe);
    assert_eq!(observed.len(), 1);
    assert_eq!(observed[0].unit_key, OBSERVE_UNIT);
    let rendered = |p: i64| h.images.render(p, run, 1024).unwrap().sha256().to_owned();
    assert_eq!(
        observed[0].images,
        vec![
            (rendered(cover), 1024),
            (rendered(marked), 1024),
            (rendered(unreviewed), 1024)
        ]
    );
    let labels = of(Stage::Labels);
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].unit_key, format!("photo:{marked}"));
    assert_eq!(labels[0].images[0].1, 2048);
    let condition = of(Stage::Condition);
    assert_eq!(condition.len(), 1);
    assert_eq!(condition[0].images.len(), 2);
    assert!(condition[0].images.iter().all(|(_, edge)| *edge == 2048));
    let cover_1024 = h.images.render(cover, run, 1024).unwrap();
    let searched = h.search.requests();
    assert_eq!(
        searched[0].images,
        vec![(cover_1024.sha256().to_owned(), 1024)]
    );
    let compared = of(Stage::Compare);
    assert_eq!(compared.len(), 1);
    assert_eq!(
        compared[0].images,
        vec![(cover_1024.sha256().to_owned(), 1024)]
    );
    let _ = (c1, c2);
}

#[test]
fn a12_c3b_label_gives_high_without_sources() {
    let h = Harness::chair();
    let photo = h.add_main_photo_with_preview(h.session_id);
    observe(&h, json!({"reads": {}, "marking_photos": [1]}));
    h.model.reply(
        Stage::Labels,
        &format!("photo:{photo}"),
        json!({"labels": {DESIGNER: designer("Designer A")}, "text": "Designed by Designer A"}),
    );
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    let designer_row = row(&h, DESIGNER).unwrap();
    assert_eq!(designer_row.value, "\"Designer A\"");
    assert_eq!(designer_row.ai_confidence, Some(AiConfidence::High));
    assert_eq!(designer_row.ai_sources, Some(vec![]));
}

#[test]
fn a12_c3c_no_condition_photo_no_stage_6() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    assert!(
        h.model
            .requests()
            .iter()
            .all(|c| c.stage != Some(Stage::Condition))
    );
    assert!(row(&h, SCRATCHES).is_none());
    let nd = h.steps(run).pop().unwrap();
    assert_eq!(nd.kind, "not_determined");
    assert!(nd.field_keys.contains(&SCRATCHES.to_owned()));
    assert!(nd.detail.contains("no Condition photo"));
}

#[test]
fn a12_c4a_not_determined_creates_no_row() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    results(&h, &[P_E1]);
    page(&h, P_E1, 5);
    verdict(&h, P_E1, "match");
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    assert!(row(&h, DESIGNER).is_none());
    let nd = h.steps(run).pop().unwrap();
    assert_eq!(nd.kind, "not_determined");
    assert!(nd.field_keys.contains(&DESIGNER.to_owned()));
    assert!(
        nd.detail
            .contains(&format!("{DESIGNER}: no label or listing states it"))
    );
}

#[test]
fn a12_c4b_protected_values_are_untouched() {
    let f = full();
    let h = &f.h;
    let human = Value {
        item_id: h.item_id,
        key: DESIGNER.into(),
        value: "\"Someone Else\"".into(),
        valid: true,
        provenance: Provenance::Human,
        ai_confidence: None,
        ai_status: None,
        ai_reason: None,
        ai_sources: None,
        ai_prior: None,
        updated_at: 5,
    };
    let accepted = Value {
        item_id: h.item_id,
        key: WOODS.into(),
        value: "[\"Oak\"]".into(),
        valid: true,
        provenance: Provenance::Ai,
        ai_confidence: Some(AiConfidence::Medium),
        ai_status: Some(AiStatus::Accepted),
        ai_reason: Some("earlier".into()),
        ai_sources: Some(vec![]),
        ai_prior: None,
        updated_at: 6,
    };
    h.put(&human);
    h.put(&accepted);
    h.execute(f.run);
    assert_eq!(row(h, DESIGNER), Some(human));
    assert_eq!(row(h, WOODS), Some(accepted));
    let last = h.steps(f.run).pop().unwrap();
    assert!(
        last.detail
            .contains(&format!("{DESIGNER}: the photographer wrote this value"))
    );
    assert!(
        last.detail
            .contains(&format!("{WOODS}: the photographer accepted this value"))
    );
}

#[test]
fn a12_c4c_cleared_value_is_not_written_again() {
    let f = full();
    let h = &f.h;
    h.feedback(DESIGNER, FeedbackAction::Cleared, json!("Designer A"));
    h.execute(f.run);
    assert!(row(h, DESIGNER).is_none());
    let last = h.steps(f.run).pop().unwrap();
    assert!(
        last.detail
            .contains(&format!("{DESIGNER}: matches a value you cleared"))
    );
}

#[test]
fn a12_c4d_unsupported_suggestion_is_lowered() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    h.suggest_with(
        DESIGNER,
        json!("Designer A"),
        AiConfidence::Medium,
        Some("earlier run"),
        &[u(P_E1)],
    );
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    let designer_row = row(&h, DESIGNER).unwrap();
    assert_eq!(designer_row.value, "\"Designer A\"");
    assert_eq!(designer_row.ai_confidence, Some(AiConfidence::Low));
    assert!(
        designer_row
            .ai_reason
            .unwrap()
            .starts_with("No longer supported:")
    );
    assert_eq!(designer_row.ai_sources, Some(vec![u(P_E1)]));
}

/// R7·U1, carried (owner 2026-11-12; 08B changelog 7): a run that ends `stopped_at_cap` lowers
/// nothing. The first run suggests the designer at medium confidence from two trusted pages; a
/// follow-up (the cover re-framed) with a smaller cap reaches its reserve at the new photo's
/// observe call, so it reads no page and decides from the photos alone.
#[test]
fn r7_u1_a_capped_run_lowers_nothing() {
    let f = full();
    assert_eq!(f.h.execute(f.run), ExecuteOutcome::Finished);
    let h = &f.h;
    let before = row(h, DESIGNER).unwrap();
    assert_eq!(before.ai_confidence, Some(AiConfidence::Medium));
    h.recompose(f.mains[0]);
    h.model.script(
        Stage::Observe,
        OBSERVE_UNIT,
        ModelScript {
            output: json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}, "marking_photos": []}),
            usage: Some(vec![("input_token".into(), 199_500)]),
            response: None,
        },
    );
    let mut config = h.stage_config();
    config.max_cost_per_run = 200_000;
    let follow_up = h.start_run(&config);
    h.query(|c| {
        psm_research::store::set_trigger(c, follow_up, psm_research::domain::RunTrigger::FollowUp)
            .unwrap()
    });
    h.reader.clear_requests();
    assert_eq!(h.execute(follow_up), ExecuteOutcome::StoppedAtCap);
    assert!(h.reader.requests().is_empty(), "the follow-up read a page");
    let after = row(h, DESIGNER).unwrap();
    assert_eq!(
        (
            &after.value,
            after.ai_confidence,
            &after.ai_reason,
            &after.ai_sources
        ),
        (
            &before.value,
            before.ai_confidence,
            &before.ai_reason,
            &before.ai_sources
        )
    );
}

#[test]
fn a12_c4e_product_step_is_never_written() {
    let f = full();
    let h = &f.h;
    observe(
        h,
        json!({"reads": {
            WOODS: {"value": ["Teak"], "sure": true},
            TITLE: {"value": "A great chair", "sure": true},
        }, "marking_photos": []}),
    );
    h.execute(f.run);
    assert_eq!(h.title(), "Oak chair");
    assert!(row(h, TITLE).is_none());
    let last = h.steps(f.run).pop().unwrap();
    assert!(last.field_keys.contains(&TITLE.to_owned()));
    // The built-in title definition's label is "Title".
    assert!(
        last.detail.lines().any(|l| l == "Title: not researched"),
        "{}",
        last.detail
    );
}

#[test]
fn a12_c4f_measured_from_listings_not_photos() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    observe(
        &h,
        json!({"reads": {WIDTH: {"value": {"value": 118, "unit": "cm"}, "sure": true}}}),
    );
    results(&h, &[P_E1, P_E2]);
    page(&h, P_E1, 1);
    page(&h, P_E2, 2);
    let width = json!({"stated": "120 cm", "value": {"value": 120, "unit": "cm"}});
    states(&h, P_E1, WIDTH, width.clone());
    states(&h, P_E2, WIDTH, width);
    verdict(&h, P_E1, "match");
    verdict(&h, P_E2, "match");
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    let width_row = row(&h, WIDTH).unwrap();
    let value: Json = serde_json::from_str(&width_row.value).unwrap();
    assert_eq!(value["unit"], json!("CENTIMETERS"));
    assert_eq!(
        value["value"].as_str().unwrap().parse::<f64>().unwrap(),
        120.0
    );
    assert_eq!(width_row.ai_confidence, Some(AiConfidence::Medium));
}

/// C5(a)'s run: the read of the second page reports enough usage that cost + reserve reaches
/// the cap.
fn reserve_run() -> (Harness, i64) {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    h.add_main_photo_with_preview(h.session_id);
    observe(
        &h,
        json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}}),
    );
    let third = "https://other.example/z";
    results(&h, &[P_E1, P_E2, third]);
    page(&h, P_E1, 1);
    page(&h, P_E2, 2);
    page(&h, third, 3);
    // Before the second page's model call: 1 observe + find + read e1 (2 calls) + reader e2 =
    // 5 fallbacks of 1 000. 494 500 more makes 499 500: cost + reserve (1 000) ≥ cap (500 000).
    h.model.script(
        Stage::Read,
        &u(P_E2),
        ModelScript {
            output: json!({"keys": {DESIGNER: designer("Designer A")}}),
            usage: Some(vec![("input_token".into(), 494_500)]),
            response: None,
        },
    );
    let run = h.start_run(&h.stage_config());
    (h, run)
}

#[test]
fn a12_c5a_reserve_reached_skips_reading_and_decides() {
    let (h, run) = reserve_run();
    assert_eq!(h.execute(run), ExecuteOutcome::StoppedAtCap);
    assert_eq!(reader_urls(&h), vec![P_E1, P_E2]);
    assert!(
        h.model
            .requests()
            .iter()
            .all(|c| c.stage != Some(Stage::Compare))
    );
    assert!(
        h.model
            .requests()
            .iter()
            .any(|c| c.stage == Some(Stage::Decide))
    );
    assert_eq!(h.run_row(run).state, RunState::StoppedAtCap);
    assert_eq!(ai(&row(&h, WOODS)).0, json!(["Teak"]));
}

#[test]
fn a12_c5c_cost_bound_is_cap_plus_one_call() {
    let (h, run) = reserve_run();
    h.execute(run);
    let calls = h.calls(run);
    let mut before = 0;
    for call in &calls {
        assert!(before < 500_000, "a call started at cost {before}");
        before += call.cost_micro_usd;
    }
    let cost = h.run_row(run).cost_micro_usd;
    assert_eq!(cost, before);
    let crossing = calls.last().unwrap().cost_micro_usd;
    assert!(cost > 500_000 && cost <= 500_000 + crossing, "{cost}");
}

#[test]
fn a12_c5b_cap_reached_before_decide_writes_nothing() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    h.model.script(
        Stage::Observe,
        OBSERVE_UNIT,
        ModelScript {
            output: json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}}),
            usage: Some(vec![("input_token".into(), 500_000)]),
            response: None,
        },
    );
    let run = h.start_run(&h.stage_config());
    assert_eq!(h.execute(run), ExecuteOutcome::StoppedAtCap);
    let row = h.run_row(run);
    assert_eq!(row.state, RunState::StoppedAtCap);
    assert_eq!(row.cost_micro_usd, 500_000);
    assert!(h.row(WOODS).is_none());
    assert!(
        h.model
            .requests()
            .iter()
            .all(|c| c.stage != Some(Stage::Decide))
    );
}

#[test]
fn a12_c6a_failure_ends_the_run_and_keeps_earlier_values() {
    let h = Harness::chair();
    let earlier = h.suggest_with(
        WOODS,
        json!(["Oak"]),
        AiConfidence::Medium,
        Some("an earlier run"),
        &[],
    );
    h.add_main_photo_with_preview(h.session_id);
    observe(
        &h,
        json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}}),
    );
    results(&h, &[P_E1, P_E2]);
    for _ in 0..3 {
        h.reader.fault_for(P_E2, fakes::transient("timed out"));
    }
    let before = h.row(WOODS);
    let run = h.start_run(&h.stage_config());
    assert_eq!(h.execute(run), ExecuteOutcome::Failed);
    let row = h.run_row(run);
    assert_eq!(row.state, RunState::Failed);
    assert_eq!(row.error.as_deref(), Some("read failed: timed out"));
    assert_eq!(h.row(WOODS), before);
    assert!(matches!(
        earlier,
        psm_research::writes::WriteOutcome::Written(_)
    ));
    assert!(h.outputs(run).iter().all(|o| o.stage != "decide"));
}

#[test]
fn a12_c6b_refused_unit_is_not_determined_and_the_run_finishes() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    h.add_main_photo_with_preview(h.session_id);
    h.model.fault_for(
        Stage::Observe,
        OBSERVE_UNIT,
        fakes::refusal("I can't describe this photo"),
    );
    observe(
        &h,
        json!({"reads": {WOODS: {"value": ["Oak"], "sure": true}}}),
    );
    let run = h.start_run(&h.stage_config());
    assert_eq!(h.execute(run), ExecuteOutcome::Finished);
    let unit = h
        .outputs(run)
        .into_iter()
        .find(|o| o.unit_key == OBSERVE_UNIT)
        .unwrap();
    let output: Json = serde_json::from_str(&unit.output).unwrap();
    assert_eq!(
        output["not_determined"],
        json!("I can't describe this photo")
    );
    // The one Observe unit was refused: no photo read, so woods is not written.
    assert!(row(&h, WOODS).is_none());
}

#[test]
fn a12_c7a_language_and_choices_reach_every_request() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    observe(
        &h,
        json!({"reads": {WOODS: {"value": ["teak"], "sure": true}}}),
    );
    results(&h, &[P_E1]);
    page(&h, P_E1, 1);
    let mut config = h.stage_config();
    config.language = "Swedish".into();
    let run = h.start_run(&config);
    h.execute(run);
    let log = h.model.requests();
    assert!(!log.is_empty());
    for call in &log {
        assert_eq!(call.language, "Swedish");
        assert_eq!(call.input["language"], json!("Swedish"));
    }
    for call in log
        .iter()
        .filter(|c| matches!(c.stage, Some(Stage::Observe | Stage::Read)))
    {
        let woods = call.input["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["key"] == json!(WOODS))
            .unwrap();
        assert_eq!(woods["choices"], json!(["Oak", "Teak"]));
    }
    assert_eq!(row(&h, WOODS).unwrap().value, "[\"Teak\"]");
}

#[test]
fn a12_c7b_live_settings_change_does_not_reach_a_running_run() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    results(&h, &[P_UNTRUSTED, P_E2, P_E1]);
    for (n, url) in [P_E1, P_E2, P_UNTRUSTED].iter().enumerate() {
        page(&h, url, u8::try_from(n).unwrap() + 10);
    }
    states(&h, P_E1, DESIGNER, designer("Designer A"));
    states(&h, P_E2, DESIGNER, designer("Designer A"));
    verdict(&h, P_E1, "match");
    verdict(&h, P_E2, "match");
    h.set_research_settings(json!({"trusted_sites": ["e1.example", "e2.example"]}));
    let paths = h.app.app.paths.clone();
    h.search.before_call(move |_| {
        crate::harness::save_research(&paths, json!({"trusted_sites": []}));
    });
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    assert_eq!(h.research_settings().unwrap()["trusted_sites"], json!([]));
    assert_eq!(reader_urls(&h), vec![P_E1, P_E2, P_UNTRUSTED]);
    assert_eq!(
        row(&h, DESIGNER).unwrap().ai_confidence,
        Some(AiConfidence::Medium)
    );
}

#[test]
fn a12_c8a_confidence_is_computed_not_taken_from_the_model() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    results(&h, &[P_E1]);
    page(&h, P_E1, 1);
    states(&h, P_E1, DESIGNER, designer("Designer A"));
    verdict(&h, P_E1, "similar");
    h.model.reply(
        Stage::Decide,
        "decide",
        json!({"reasons": {DESIGNER: "Certainly Designer A."}, "confidence": {DESIGNER: "high"}}),
    );
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    let designer_row = row(&h, DESIGNER).unwrap();
    assert_eq!(designer_row.ai_confidence, Some(AiConfidence::Low));
    let _ = ResearchConfig::default();
}
