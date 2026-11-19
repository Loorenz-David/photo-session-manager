//! A24 (owner 2026-11-19, "perfect, we can build that"): a deliberate request ("Write texts", an
//! answer) offers new texts over reviewed ones, with the reviewed row as the prior that **Keep
//! previous** restores exactly; automatic requests keep MC-A1's refusals. The writer's state shows
//! in the sessions overview, and its status says which texts were not offered and when the run
//! ended. Runs on the research harness with `FakeModel` replying for stage `write`.

use psm_core::domain::attribute::{AiConfidence, AiPrior, AiStatus, Provenance, Value};
use psm_research::{
    api::{ResearchApi, WriterOverview},
    domain::Stage,
    writer::{api::WriterApi, run::Origin},
};
use serde_json::{Value as Json, json};

use crate::{
    harness::{Harness, INTEGRATION},
    w2_writer_run::{DESCRIPTION, write_runs},
    w2_writer_writes::{
        CLEAN_PARAGRAPHS, CLEAN_REPORT, CLEAN_TITLE, REPORT, TITLE, clean, joined, run_auto_with,
        run_with, texts_harness,
    },
};

fn text_of(row: &Value) -> String {
    serde_json::from_str::<Json>(&row.value)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned()
}

/// A reviewed row of `key`: `provenance` and `status`, with a reason and a source.
fn reviewed(
    h: &Harness,
    key: &str,
    value: &str,
    provenance: Provenance,
    status: Option<AiStatus>,
) -> Value {
    let row = Value {
        item_id: h.item_id,
        key: key.into(),
        value: json!(value).to_string(),
        valid: true,
        provenance,
        ai_confidence: status.map(|_| AiConfidence::High),
        ai_status: status,
        ai_reason: status.map(|_| "Earlier reason.".to_owned()),
        ai_sources: status.map(|_| vec!["https://example.test/a".to_owned()]),
        ai_prior: None,
        updated_at: h.now(),
    };
    h.put(&row);
    row
}

/// The row as it would be stored again, without its time.
fn content(row: &Value) -> Value {
    Value {
        updated_at: 0,
        ..row.clone()
    }
}

fn keep_previous(h: &Harness, key: &str) {
    h.app
        .attributes(Some(INTEGRATION))
        .keep_previous(h.item_id, key)
        .unwrap();
}

fn item_title(h: &Harness) -> String {
    h.app.store().item(h.item_id).unwrap().title
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
fn a24_write_texts_offers_a_new_description_over_a_cleared_one() {
    // The work-Mac case: the photographer cleared the description, then pressed Write texts.
    let h = texts_harness(true);
    reviewed(
        &h,
        DESCRIPTION,
        "Old text",
        Provenance::Human,
        Some(AiStatus::Cleared),
    );
    run_with(&h, clean(), None);
    let row = h.row(DESCRIPTION).unwrap();
    assert_eq!(
        (row.provenance, row.ai_status),
        (Provenance::Ai, Some(AiStatus::Suggested))
    );
    assert_eq!(text_of(&row), joined(&CLEAN_PARAGRAPHS));
    // A cleared field holds no value: nothing to keep, Clear empties it again.
    assert_eq!(row.ai_prior, None);
}

#[test]
fn a24_write_texts_offers_over_an_accepted_report_and_keep_previous_restores_it() {
    let h = texts_harness(true);
    let accepted = reviewed(
        &h,
        REPORT,
        "Accepted report.",
        Provenance::Ai,
        Some(AiStatus::Accepted),
    );
    run_with(&h, clean(), None);
    let row = h.row(REPORT).unwrap();
    assert_eq!(row.ai_status, Some(AiStatus::Suggested));
    assert_eq!(text_of(&row), CLEAN_REPORT);
    assert_eq!(row.ai_prior, Some(AiPrior::of(&accepted)));
    keep_previous(&h, REPORT);
    assert_eq!(content(&h.row(REPORT).unwrap()), content(&accepted));
}

#[test]
fn a24_write_texts_offers_over_a_typed_description_and_keep_previous_restores_it() {
    let h = texts_harness(true);
    let typed = reviewed(&h, DESCRIPTION, "My own text", Provenance::Human, None);
    run_with(&h, clean(), None);
    let row = h.row(DESCRIPTION).unwrap();
    assert_eq!(text_of(&row), joined(&CLEAN_PARAGRAPHS));
    assert_eq!(row.ai_prior, Some(AiPrior::of(&typed)));
    keep_previous(&h, DESCRIPTION);
    assert_eq!(content(&h.row(DESCRIPTION).unwrap()), content(&typed));
}

#[test]
fn a24_the_same_text_as_the_accepted_one_is_not_offered() {
    let h = texts_harness(true);
    let accepted = reviewed(
        &h,
        REPORT,
        CLEAN_REPORT,
        Provenance::Ai,
        Some(AiStatus::Accepted),
    );
    run_with(&h, clean(), None);
    assert_eq!(content(&h.row(REPORT).unwrap()), content(&accepted));
    let status = WriterApi::status(h.service.as_ref(), h.session_id).unwrap();
    let skipped: Vec<_> = status
        .not_offered
        .iter()
        .filter(|n| !n.key.starts_with("product.seo_")) // the scripted answer has no search texts
        .map(|n| (n.key.as_str(), n.outcome.as_str()))
        .collect();
    assert_eq!(skipped, vec![(REPORT, "already holds this text")]);
}

#[test]
fn a24_write_texts_offers_over_a_typed_title_which_stays_the_items_title() {
    let h = texts_harness(true);
    set_title(&h, "Chair");
    run_with(&h, clean(), None);
    let row = h.row(TITLE).unwrap();
    assert_eq!(text_of(&row), CLEAN_TITLE);
    assert_eq!(
        row.ai_prior.as_ref().map(|p| p.provenance),
        Some(Provenance::Human)
    );
    // Accept-only: the item's title changes only when the suggestion is accepted.
    assert_eq!(item_title(&h), "Chair");
    keep_previous(&h, TITLE);
    assert_eq!(h.row(TITLE), None);
    assert_eq!(item_title(&h), "Chair");
}

#[test]
fn a24_keep_previous_brings_back_an_accepted_title_row() {
    let h = texts_harness(true);
    set_title(&h, "Accepted chair");
    let accepted = reviewed(
        &h,
        TITLE,
        "Accepted chair",
        Provenance::Ai,
        Some(AiStatus::Accepted),
    );
    run_with(&h, clean(), None);
    assert_eq!(text_of(&h.row(TITLE).unwrap()), CLEAN_TITLE);
    keep_previous(&h, TITLE);
    assert_eq!(content(&h.row(TITLE).unwrap()), content(&accepted));
    assert_eq!(item_title(&h), "Accepted chair");
}

#[test]
fn a24_automatic_runs_keep_the_refusals_and_say_what_was_not_offered() {
    let h = texts_harness(true);
    let accepted = reviewed(
        &h,
        REPORT,
        "Accepted report.",
        Provenance::Ai,
        Some(AiStatus::Accepted),
    );
    reviewed(
        &h,
        DESCRIPTION,
        "Old text",
        Provenance::Human,
        Some(AiStatus::Cleared),
    );
    run_auto_with(&h, clean());
    assert_eq!(content(&h.row(REPORT).unwrap()), content(&accepted));
    assert_eq!(
        h.row(DESCRIPTION).unwrap().ai_status,
        Some(AiStatus::Cleared)
    );
    let status = WriterApi::status(h.service.as_ref(), h.session_id).unwrap();
    let skipped: Vec<_> = status
        .not_offered
        .iter()
        .filter(|n| !n.key.starts_with("product.seo_")) // the scripted answer has no search texts
        .map(|n| (n.key.as_str(), n.outcome.as_str()))
        .collect();
    assert_eq!(
        skipped,
        vec![
            (DESCRIPTION, "the photographer cleared this field"),
            (REPORT, "the photographer accepted this value"),
        ]
    );
    assert!(status.finished_at.is_some());
}

#[test]
fn a24_write_texts_makes_a_queued_automatic_run_deliberate() {
    let h = texts_harness(true);
    let accepted = reviewed(
        &h,
        REPORT,
        "Accepted report.",
        Provenance::Ai,
        Some(AiStatus::Accepted),
    );
    h.model.reply(Stage::Write, "texts", clean());
    let auto = WriterApi::request(h.service.as_ref(), h.session_id, Origin::NextToReview).unwrap();
    assert!(auto.queued);
    let manual = WriterApi::request(h.service.as_ref(), h.session_id, Origin::WriteTexts).unwrap();
    assert!(manual.queued);
    h.tick(0);
    assert_eq!(write_runs(&h, h.session_id).len(), 1);
    let row = h.row(REPORT).unwrap();
    assert_eq!(text_of(&row), CLEAN_REPORT);
    assert_eq!(row.ai_prior, Some(AiPrior::of(&accepted)));
}

#[test]
fn a24_the_overview_shows_the_writer_writing_done_and_failed() {
    let h = texts_harness(true);
    let writer = |h: &Harness| {
        ResearchApi::overview(h.service.as_ref(), Some(h.session_id))
            .unwrap()
            .into_iter()
            .find(|s| s.session_id == h.session_id)
            .and_then(|s| s.writer)
    };
    assert_eq!(writer(&h), None);
    h.model.reply(Stage::Write, "texts", clean());
    WriterApi::request(h.service.as_ref(), h.session_id, Origin::WriteTexts).unwrap();
    let state = |state: &str| {
        Some(WriterOverview {
            state: state.into(),
            sku: "W2-4".into(),
        })
    };
    assert_eq!(writer(&h), state("writing"));
    h.tick(0);
    assert_eq!(writer(&h), state("done"));
    let session = h.session_id;
    h.query(|c| {
        c.execute(
            "UPDATE research_runs SET state='failed', error='the model did not answer' \
             WHERE session_id=?1 AND trigger='write'",
            [session],
        )
        .unwrap()
    });
    assert_eq!(writer(&h), state("failed"));
}
