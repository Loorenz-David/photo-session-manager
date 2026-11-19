//! W2 unit 4 (contracts WC4, WC5 "Regenerate once", WC7 of `master_plan_writer.md` §5): a writer
//! run's call, its one rewrite and its writes (rows W2·C16–C20). Runs on the research harness with
//! `FakeModel` replying for stage `write`; the outcome is the requests the fake received, the call
//! records, the attribute rows, the steps and the unit outputs.

use psm_core::domain::attribute::{
    AiConfidence, AiPrior, AiStatus, AttributeGroup, CoreType, Provenance,
};
use psm_research::{
    domain::{RunState, Stage},
    writer::{
        api::WriterApi,
        defaults::{INSTRUCTION_EN, TITLE_FORMULA_EN},
        run::{NOT_RECHECKED, Origin},
    },
};
use serde_json::{Value as Json, json};

use crate::{
    harness::{DESIGNER, Harness, WOOD},
    w1_note::{add_definitions, definition, layout},
    w2_writer_run::{DESCRIPTION, ERA, put, write_runs},
};

pub(crate) const REPORT: &str = "custom.condition_report_description";
pub(crate) const TITLE: &str = "product.title";

/// The writer's default model, priced for the fake provider: USD 2 per million input tokens,
/// USD 10 per million output tokens, USD 0.001 per call whose usage is unknown.
fn price_default_model(h: &Harness) {
    let mut prices = h.research_settings().unwrap()["prices"].clone();
    let rows = prices.as_array_mut().unwrap();
    rows.push(
        json!({"provider": "fake-model", "model": "gpt-6.1-sol", "unit": "input_token",
                     "usd_per_million": "2", "fallback_call_usd": "0.001"}),
    );
    rows.push(
        json!({"provider": "fake-model", "model": "gpt-6.1-sol", "unit": "output_token",
                     "usd_per_million": "10"}),
    );
    h.patch_settings(json!({ "prices": prices }));
}

/// An item of type Texts with a blank title: Details designer, era, wood; Condition the condition
/// report text field (when `report`); the default Product step (title, description, …). The
/// writer runs on its default settings.
pub(crate) fn texts_harness(report: bool) -> Harness {
    let mut h = Harness::new();
    add_definitions(
        &h,
        vec![
            definition(ERA, "Era", CoreType::Text, AttributeGroup::Details),
            definition(
                REPORT,
                "Condition report",
                CoreType::Textarea,
                AttributeGroup::Condition,
            ),
        ],
    );
    layout(&h, "Texts", AttributeGroup::Details, &[DESIGNER, ERA, WOOD]);
    let condition: &[&str] = if report { &[REPORT] } else { &[] };
    layout(&h, "Texts", AttributeGroup::Condition, condition);
    let (item, session) = h
        .app
        .items()
        .create_local("W2-4", "", "Texts", None)
        .unwrap();
    h.item_id = item.id;
    h.session_id = session.session.id;
    price_default_model(&h);
    h
}

pub(crate) const CLEAN_TITLE: &str = "Danish Teak Chair with Curved Back";
pub(crate) const CLEAN_PARAGRAPHS: [&str; 4] = [
    "A dining chair made in Denmark with a frame of solid teak and a seat of woven papercord.",
    "The back rail is curved and the legs taper toward the floor. The seat is square with rounded \
     corners and sits on four rails.",
    "We cleaned the frame, oiled the teak and rewove the seat with new natural papercord. It works \
     at a dining table or a desk.",
    "Dimensions: W 49 × D 46 × H 78 cm, seat height 45 cm.",
];
pub(crate) const CLEAN_REPORT: &str = "Good. The frame was cleaned and oiled. Light wear on the seat rails. Stable, with no loose joints.";

/// An answer with the three texts (the description as paragraphs), each naming its facts.
pub(crate) fn answer(title: &str, paragraphs: &[&str], report: &str) -> Json {
    json!({
        "texts": {
            "title": {"text": title, "facts": [DESIGNER, ERA], "reason": "From the fields."},
            "description": {"paragraphs": paragraphs, "facts": [WOOD], "reason": "From the note."},
            "condition_report": {"text": report, "facts": [], "reason": "From the condition."},
        },
        "findings": [],
        "questions": [],
    })
}

pub(crate) fn clean() -> Json {
    answer(CLEAN_TITLE, &CLEAN_PARAGRAPHS, CLEAN_REPORT)
}

/// A description with "stunning" in it (otherwise as the clean one).
fn stunning() -> Vec<&'static str> {
    let mut paragraphs = CLEAN_PARAGRAPHS.to_vec();
    paragraphs[0] = "A stunning dining chair made in Denmark with a frame of solid teak and a seat \
                     of woven papercord.";
    paragraphs
}

pub(crate) fn joined(paragraphs: &[&str]) -> String {
    paragraphs.join("\n\n")
}

/// Asks for texts, then starts the run and runs its job; returns the run.
pub(crate) fn write(h: &Harness) -> i64 {
    let answer = WriterApi::request(h.service.as_ref(), h.session_id, Origin::WriteTexts).unwrap();
    assert!(answer.queued, "{answer:?}");
    h.tick(0);
    write_runs(h, h.session_id).last().unwrap().id
}

/// One writer run whose first call answers `first` (and the rewrite `retry`, when scripted).
pub(crate) fn run_with(h: &Harness, first: Json, retry: Option<Json>) -> i64 {
    h.model.reply(Stage::Write, "texts", first);
    if let Some(retry) = retry {
        h.model.reply(Stage::Write, "retry", retry);
    }
    write(h)
}

pub(crate) fn write_requests(h: &Harness) -> Vec<psm_research::fakes::FakeCall> {
    h.model
        .requests()
        .into_iter()
        .filter(|c| c.stage == Some(Stage::Write))
        .collect()
}

fn text_of(h: &Harness, key: &str) -> String {
    serde_json::from_str::<Json>(&h.row(key).unwrap().value)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned()
}

/// The steps of a run that name `key`, as (kind, summary + detail).
fn steps_of(h: &Harness, run: i64, key: &str) -> Vec<(String, String)> {
    h.steps(run)
        .into_iter()
        .filter(|s| s.field_keys.iter().any(|k| k == key))
        .map(|s| (s.kind, format!("{} {}", s.summary, s.detail)))
        .collect()
}

fn result_of(h: &Harness, run: i64) -> Json {
    let row = h
        .outputs(run)
        .into_iter()
        .find(|o| o.stage == "write" && o.unit_key == "result")
        .expect("no result unit");
    serde_json::from_str(&row.output).unwrap()
}

fn item_title(h: &Harness) -> String {
    h.app.store().item(h.item_id).unwrap().title
}

#[test]
fn w2_c16a_one_request_with_the_guide_the_input_and_no_image() {
    let h = texts_harness(true);
    let run = run_with(&h, clean(), None);
    assert_eq!(h.run_row(run).state, RunState::Finished);
    let requests = write_requests(&h);
    assert_eq!(requests.len(), 1);
    let call = &h.calls(run)[0];
    let recorded: Json = serde_json::from_str(&call.request).unwrap();
    let instructions = recorded["instructions"].as_str().unwrap();
    assert!(instructions.starts_with(INSTRUCTION_EN));
    assert!(instructions.contains(TITLE_FORMULA_EN));
    assert!(instructions.contains("Example A"));
    let snapshot: Json =
        serde_json::from_str(h.run_row(run).input_snapshot.as_deref().unwrap()).unwrap();
    assert_eq!(requests[0].input, snapshot);
    assert!(requests[0].images.is_empty());
    assert!(requests[0].listing_images.is_empty());
    assert_eq!(recorded["images"], json!([]));
}

#[test]
fn w2_c16b_clean_texts_arrive_as_medium_suggestions_with_their_facts() {
    let h = texts_harness(true);
    run_with(&h, clean(), None);
    for (key, facts) in [
        (TITLE, "Facts: custom.designer, Era"),
        (DESCRIPTION, "Facts: custom.wood"),
        (REPORT, "Facts: "),
    ] {
        let row = h.row(key).unwrap_or_else(|| panic!("no {key} row"));
        assert_eq!(row.provenance, Provenance::Ai, "{key}");
        assert_eq!(row.ai_status, Some(AiStatus::Suggested), "{key}");
        assert_eq!(row.ai_confidence, Some(AiConfidence::Medium), "{key}");
        assert!(
            row.ai_reason.as_deref().unwrap().ends_with(facts),
            "{key}: {:?}",
            row.ai_reason
        );
        assert_eq!(row.ai_sources, Some(Vec::new()), "{key}");
    }
    assert_eq!(text_of(&h, DESCRIPTION), joined(&CLEAN_PARAGRAPHS));
    assert_eq!(text_of(&h, TITLE), CLEAN_TITLE);
}

#[test]
fn w2_c17a_a_text_with_a_hit_is_rewritten_once_and_the_clean_ones_kept() {
    let h = texts_harness(true);
    let retry = json!({
        "texts": {"description": {"paragraphs": CLEAN_PARAGRAPHS, "facts": [WOOD],
                                  "reason": "Rewritten."}},
        "findings": [], "questions": [],
    });
    let first = answer("Teak Chair, First Answer", &stunning(), CLEAN_REPORT);
    let run = run_with(&h, first, Some(retry));
    let requests = write_requests(&h);
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[1].unit_key, "retry");
    assert_eq!(requests[1].input["texts"], json!(["description"]));
    assert!(requests[1].input["hits"].to_string().contains("stunning"));
    let second: Json = serde_json::from_str(&h.calls(run)[1].request).unwrap();
    assert!(
        second["instructions"]
            .as_str()
            .unwrap()
            .contains("stunning")
    );
    let description = h.row(DESCRIPTION).unwrap();
    assert_eq!(text_of(&h, DESCRIPTION), joined(&CLEAN_PARAGRAPHS));
    assert_eq!(description.ai_confidence, Some(AiConfidence::Medium));
    assert_eq!(text_of(&h, TITLE), "Teak Chair, First Answer");
    assert_eq!(text_of(&h, REPORT), CLEAN_REPORT);
}

#[test]
fn w2_c17b_a_text_that_fails_twice_arrives_flagged_low() {
    let h = texts_harness(true);
    let mut retried = stunning();
    retried[1] = "The stunning back rail is curved and the legs taper toward the floor.";
    let retry = json!({
        "texts": {"description": {"paragraphs": retried, "facts": [WOOD], "reason": "Again."}},
        "findings": [], "questions": [],
    });
    let run = run_with(
        &h,
        answer(CLEAN_TITLE, &stunning(), CLEAN_REPORT),
        Some(retry),
    );
    let description = h.row(DESCRIPTION).unwrap();
    assert_eq!(text_of(&h, DESCRIPTION), joined(&retried));
    assert_eq!(description.ai_confidence, Some(AiConfidence::Low));
    let reason = description.ai_reason.unwrap();
    assert!(
        reason.contains("Check:") && reason.contains("stunning"),
        "{reason}"
    );
    let attempts = result_of(&h, run)["hits"].clone();
    let listed: Vec<(i64, String)> = attempts
        .as_array()
        .unwrap()
        .iter()
        .map(|a| (a["attempt"].as_i64().unwrap(), a["hits"].to_string()))
        .collect();
    assert_eq!(listed.len(), 2);
    assert_eq!((listed[0].0, listed[1].0), (1, 2));
    assert!(listed.iter().all(|(_, hits)| hits.contains("stunning")));
}

#[test]
fn w2_c17c_at_the_run_cost_limit_the_text_is_flagged_not_rechecked() {
    let h = texts_harness(true);
    // One call costs 1000 × 2 + 500 × 10 = 7000 µUSD: the limit after the first call.
    h.model.set_default_usage(Some(vec![
        ("input_token".into(), 1000),
        ("output_token".into(), 500),
    ]));
    h.patch_settings(json!({"max_cost_per_run": "0.007"}));
    let run = run_with(&h, answer(CLEAN_TITLE, &stunning(), CLEAN_REPORT), None);
    assert_eq!(write_requests(&h).len(), 1);
    let description = h.row(DESCRIPTION).unwrap();
    assert_eq!(description.ai_confidence, Some(AiConfidence::Low));
    assert!(description.ai_reason.unwrap().contains(NOT_RECHECKED));
    assert_eq!(h.run_row(run).state, RunState::StoppedAtCap);
}

/// One writer run of the clean answer over a description stored as given.
fn over_description(
    value: &str,
    provenance: Provenance,
    status: Option<AiStatus>,
) -> (Harness, i64) {
    let h = texts_harness(true);
    put(
        &h,
        DESCRIPTION,
        json!(value),
        provenance,
        status,
        status.map(|_| AiConfidence::Medium),
    );
    let run = run_with(&h, clean(), None);
    (h, run)
}

#[test]
fn w2_c18a_a_description_with_no_row_is_written() {
    let h = texts_harness(true);
    run_with(&h, clean(), None);
    assert_eq!(text_of(&h, DESCRIPTION), joined(&CLEAN_PARAGRAPHS));
}

#[test]
fn w2_c18b_the_shops_description_is_kept_as_the_prior() {
    let (h, _) = over_description("Shop text", Provenance::Integration, None);
    let row = h.row(DESCRIPTION).unwrap();
    assert_eq!(text_of(&h, DESCRIPTION), joined(&CLEAN_PARAGRAPHS));
    assert_eq!(
        row.ai_prior,
        Some(AiPrior {
            value: json!("Shop text").to_string(),
            valid: true,
            provenance: Provenance::Integration,
        })
    );
}

#[test]
fn w2_c18c_a_typed_description_is_never_overwritten() {
    let (h, run) = over_description("My text", Provenance::Human, None);
    let row = h.row(DESCRIPTION).unwrap();
    assert_eq!(
        (row.provenance, text_of(&h, DESCRIPTION).as_str()),
        (Provenance::Human, "My text")
    );
    let steps = steps_of(&h, run, DESCRIPTION);
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].0, "not_determined");
    assert!(
        steps[0].1.contains("the photographer wrote this value"),
        "{steps:?}"
    );
}

#[test]
fn w2_c18d_an_accepted_description_is_never_overwritten() {
    let (h, run) = over_description("Accepted text", Provenance::Ai, Some(AiStatus::Accepted));
    let row = h.row(DESCRIPTION).unwrap();
    assert_eq!(row.ai_status, Some(AiStatus::Accepted));
    assert_eq!(text_of(&h, DESCRIPTION), "Accepted text");
    let steps = steps_of(&h, run, DESCRIPTION);
    assert!(
        steps[0].1.contains("the photographer accepted this value"),
        "{steps:?}"
    );
}

#[test]
fn w2_c18e_an_earlier_writer_text_is_replaced_and_its_prior_kept() {
    let h = texts_harness(true);
    let prior = AiPrior {
        value: json!("Shop text").to_string(),
        valid: true,
        provenance: Provenance::Integration,
    };
    let mut row = psm_core::domain::attribute::Value {
        item_id: h.item_id,
        key: DESCRIPTION.into(),
        value: json!("An earlier writer text").to_string(),
        valid: true,
        provenance: Provenance::Ai,
        ai_confidence: Some(AiConfidence::Medium),
        ai_status: Some(AiStatus::Suggested),
        ai_reason: Some("Earlier.".into()),
        ai_sources: Some(Vec::new()),
        ai_prior: Some(prior.clone()),
        updated_at: h.now(),
    };
    h.put(&row);
    run_with(&h, clean(), None);
    row = h.row(DESCRIPTION).unwrap();
    assert_eq!(text_of(&h, DESCRIPTION), joined(&CLEAN_PARAGRAPHS));
    assert_eq!(row.ai_prior, Some(prior));
}

#[test]
fn w2_c18f_a_dismissed_text_is_not_suggested_again_but_a_new_one_is() {
    let h = texts_harness(true);
    // Short descriptions pass the length check here, so no rewrite is asked for.
    h.set_writer_settings(json!({"lengths": {"description_min_words": 1}}));
    put(
        &h,
        DESCRIPTION,
        json!("Old"),
        Provenance::Ai,
        Some(AiStatus::Cleared),
        Some(AiConfidence::Medium),
    );
    let first = run_with(&h, answer(CLEAN_TITLE, &["Old"], CLEAN_REPORT), None);
    let row = h.row(DESCRIPTION).unwrap();
    assert_eq!(row.ai_status, Some(AiStatus::Cleared));
    assert_eq!(text_of(&h, DESCRIPTION), "Old");
    let steps = steps_of(&h, first, DESCRIPTION);
    assert!(
        steps[0].1.contains("matches a text you dismissed"),
        "{steps:?}"
    );
    run_with(&h, answer(CLEAN_TITLE, &["New text"], CLEAN_REPORT), None);
    let row = h.row(DESCRIPTION).unwrap();
    assert_eq!(row.ai_status, Some(AiStatus::Suggested));
    assert_eq!(text_of(&h, DESCRIPTION), "New text");
}

#[test]
fn w2_c18g_a_blank_title_gets_a_title_suggestion_and_stays_blank() {
    let h = texts_harness(true);
    run_with(&h, clean(), None);
    let row = h.row(TITLE).unwrap();
    assert_eq!(
        (row.provenance, row.ai_status),
        (Provenance::Ai, Some(AiStatus::Suggested))
    );
    assert_eq!(text_of(&h, TITLE), CLEAN_TITLE);
    assert_eq!(item_title(&h), "");
}

fn set_title(h: &Harness, title: &str) {
    let item = h.item_id;
    h.query(|c| {
        c.execute(
            "UPDATE items SET title=?1 WHERE id=?2",
            rusqlite::params![title, item],
        )
        .unwrap()
    });
}

#[test]
fn w2_c18h_a_typed_title_is_never_suggested_over() {
    let h = texts_harness(true);
    set_title(&h, "Chair");
    let run = run_with(&h, clean(), None);
    assert_eq!(h.row(TITLE), None);
    let steps = steps_of(&h, run, TITLE);
    assert!(
        steps[0].1.contains("the photographer wrote this value"),
        "{steps:?}"
    );
    assert_eq!(item_title(&h), "Chair");
}

#[test]
fn w2_c18i_the_shops_title_becomes_the_prior_of_the_suggestion() {
    let h = texts_harness(true);
    set_title(&h, "Shop chair");
    h.app
        .store()
        .transaction(&mut |tx| tx.set_item_title_base(h.item_id, "shopify", "Shop chair"))
        .unwrap();
    run_with(&h, clean(), None);
    let row = h.row(TITLE).unwrap();
    assert_eq!(text_of(&h, TITLE), CLEAN_TITLE);
    assert_eq!(
        row.ai_prior,
        Some(AiPrior {
            value: json!("Shop chair").to_string(),
            valid: true,
            provenance: Provenance::Integration,
        })
    );
    assert_eq!(item_title(&h), "Shop chair");
}

#[test]
fn w2_c19a_texts_that_were_not_requested_are_never_written() {
    // W5 (WC8.1): the search texts are built-in and always present, so they are requested; the
    // condition report without its field is the text that is not.
    let h = texts_harness(false);
    let run = run_with(&h, clean(), None);
    assert_eq!(h.row(REPORT), None);
    let writes = result_of(&h, run)["writes"].clone();
    assert!(
        writes
            .as_array()
            .unwrap()
            .contains(&json!({"key": REPORT, "outcome": "not requested"})),
        "{writes}"
    );
}

fn has_unit(store: &psm_infra::sqlite::store::SqliteStore, unit: &str) -> bool {
    store
        .research_transaction(|tx| {
            Ok(tx
                .conn()
                .query_row(
                    "SELECT COUNT(*) FROM research_stage_outputs WHERE stage='write' AND unit_key=?1",
                    [unit],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap()
                > 0)
        })
        .unwrap()
}

#[test]
fn w2_c20a_a_kill_between_the_call_and_the_writes_resumes_without_a_new_call() {
    let reference = texts_harness(true);
    let reference_run = run_with(&reference, clean(), None);
    let mut h = texts_harness(true);
    h.model.reply(Stage::Write, "texts", clean());
    assert!(
        WriterApi::request(h.service.as_ref(), h.session_id, Origin::WriteTexts)
            .unwrap()
            .queued
    );
    let store = h.store().clone();
    h.kill_when(move || has_unit(&store, "texts") && !has_unit(&store, "result"));
    h.tick_killed(0);
    let run = write_runs(&h, h.session_id)[0].id;
    assert_eq!(h.run_row(run).state, RunState::Running);
    h.tick(0);
    assert_eq!(h.run_row(run).state, RunState::Finished);
    assert_eq!(write_requests(&h).len(), 1);
    assert_eq!(h.end_state(run), reference.end_state(reference_run));
    for key in [TITLE, DESCRIPTION, REPORT] {
        assert_eq!(steps_of(&h, run, key).len(), 1, "{key}");
    }
}

#[test]
fn w2_c20b_the_same_texts_written_again_change_no_row() {
    let h = texts_harness(true);
    run_with(&h, clean(), None);
    let stamps = |h: &Harness| -> Vec<(String, i64)> {
        h.query(|c| {
            let mut stmt = c
                .prepare(
                    "SELECT key, updated_at FROM attribute_values WHERE item_id=?1 ORDER BY key",
                )
                .unwrap();
            stmt.query_map([h.item_id], |r| Ok((r.get(0)?, r.get(1)?)))
                .unwrap()
                .map(Result::unwrap)
                .collect()
        })
    };
    let before = stamps(&h);
    assert_eq!(before.len(), 3);
    h.advance(60_000);
    let again = run_with(&h, clean(), None);
    assert_eq!(h.run_row(again).state, RunState::Finished);
    assert_eq!(stamps(&h), before);
    let outcomes: Vec<String> = result_of(&h, again)["writes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["outcome"].as_str().unwrap().to_owned())
        .collect();
    // W5: the two search texts are requested too; this answer has none for them.
    assert_eq!(
        outcomes,
        ["unchanged", "unchanged", "unchanged", "no text", "no text"]
    );
}
