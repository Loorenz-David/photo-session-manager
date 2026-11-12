//! R3·U5 (A07·C1–C6): what research may write, through `psm_research::writes` on a real store.
//! Each test reads the stored row back and checks the returned outcome.

use psm_core::domain::{
    attribute::{AiConfidence, AiPrior, AiStatus, Provenance, Value},
    research::FeedbackAction,
};
use psm_research::{rules::write_table::is_conflict, writes::WriteOutcome};
use serde_json::{Value as Json, json};

use crate::harness::*;

fn stored(
    h: &Harness,
    key: &str,
    value: Json,
    provenance: Provenance,
    status: Option<AiStatus>,
) -> Value {
    let row = Value {
        item_id: h.item_id,
        key: key.into(),
        value: value.to_string(),
        valid: true,
        provenance,
        ai_confidence: (provenance == Provenance::Ai).then_some(AiConfidence::Medium),
        ai_status: status,
        ai_reason: None,
        ai_sources: None,
        ai_prior: None,
        updated_at: T0,
    };
    h.put(&row);
    row
}

fn prior(value: Json, valid: bool, provenance: Provenance) -> Option<AiPrior> {
    Some(AiPrior {
        value: value.to_string(),
        valid,
        provenance,
    })
}

/// A suggested row with a prior, written straight through the store.
fn suggested_with_prior(h: &Harness, value: &str, prior_value: &str) -> Value {
    let mut row = stored(
        h,
        MAKER,
        json!(value),
        Provenance::Ai,
        Some(AiStatus::Suggested),
    );
    row.ai_prior = prior(json!(prior_value), true, Provenance::Default);
    h.put(&row);
    row
}

fn assert_suggested(row: &Value, value: Json) {
    assert_eq!(row.value, value.to_string());
    assert!(row.valid);
    assert_eq!(row.provenance, Provenance::Ai);
    assert_eq!(row.ai_status, Some(AiStatus::Suggested));
}

fn refused(outcome: &WriteOutcome) -> bool {
    matches!(outcome, WriteOutcome::Refused(_))
}

#[test]
fn a07_c1a_empty_field_written() {
    let h = Harness::new();
    let outcome = h.suggest(MAKER, json!("A"));
    let row = h.row(MAKER).unwrap();
    assert_eq!(outcome, WriteOutcome::Written(row.clone()));
    assert_suggested(&row, json!("A"));
    assert_eq!(row.ai_confidence, Some(AiConfidence::Medium));
    assert_eq!(row.ai_prior, None);
}

#[test]
fn a07_c1b_default_saved_as_prior() {
    let h = Harness::new();
    stored(&h, MAKER, json!("D"), Provenance::Default, None);
    assert!(matches!(
        h.suggest(MAKER, json!("A")),
        WriteOutcome::Written(_)
    ));
    let row = h.row(MAKER).unwrap();
    assert_suggested(&row, json!("A"));
    assert_eq!(row.ai_prior, prior(json!("D"), true, Provenance::Default));
}

#[test]
fn a07_c1c_integration_saved_as_prior() {
    let h = Harness::new();
    stored(&h, MAKER, json!("I"), Provenance::Integration, None);
    assert!(matches!(
        h.suggest(MAKER, json!("A")),
        WriteOutcome::Written(_)
    ));
    let row = h.row(MAKER).unwrap();
    assert_suggested(&row, json!("A"));
    assert_eq!(
        row.ai_prior,
        prior(json!("I"), true, Provenance::Integration)
    );
}

#[test]
fn a07_c1d_invalid_integration_raw_saved_as_prior() {
    let h = Harness::new();
    let mut raw = stored(&h, WOOD, json!("Pine"), Provenance::Integration, None);
    raw.valid = false;
    h.put(&raw);
    assert!(matches!(
        h.suggest(WOOD, json!("Oak")),
        WriteOutcome::Written(_)
    ));
    let row = h.row(WOOD).unwrap();
    assert_suggested(&row, json!("Oak"));
    assert_eq!(
        row.ai_prior,
        prior(json!("Pine"), false, Provenance::Integration)
    );
}

#[test]
fn a07_c1e_replacing_a_suggestion_keeps_the_prior() {
    let h = Harness::new();
    suggested_with_prior(&h, "A", "D");
    assert!(matches!(
        h.suggest(MAKER, json!("B")),
        WriteOutcome::Written(_)
    ));
    let row = h.row(MAKER).unwrap();
    assert_suggested(&row, json!("B"));
    assert_eq!(row.ai_prior, prior(json!("D"), true, Provenance::Default));
}

#[test]
fn a07_c1f_accepted_never_overwritten() {
    let h = Harness::new();
    let before = stored(
        &h,
        MAKER,
        json!("A"),
        Provenance::Ai,
        Some(AiStatus::Accepted),
    );
    assert!(refused(&h.suggest(MAKER, json!("B"))));
    assert_eq!(h.row(MAKER), Some(before));
}

#[test]
fn a07_c1g_human_never_overwritten() {
    let h = Harness::new();
    let before = stored(&h, MAKER, json!("H"), Provenance::Human, None);
    assert!(refused(&h.suggest(MAKER, json!("B"))));
    assert_eq!(h.row(MAKER), Some(before));
}

#[test]
fn a07_c1h_human_clear_never_overwritten() {
    let h = Harness::new();
    let before = stored(
        &h,
        MAKER,
        json!("H"),
        Provenance::Human,
        Some(AiStatus::Cleared),
    );
    assert!(refused(&h.suggest(MAKER, json!("B"))));
    assert_eq!(h.row(MAKER), Some(before));
}

#[test]
fn a07_c1i_ai_clear_refuses_only_the_cleared_value() {
    let h = Harness::new();
    let before = stored(
        &h,
        MAKER,
        json!("X"),
        Provenance::Ai,
        Some(AiStatus::Cleared),
    );
    let outcome = h.suggest(MAKER, json!("X"));
    assert_eq!(outcome, WriteOutcome::MatchesCleared);
    assert_eq!(outcome.text(), "matches a value you cleared");
    assert_eq!(h.row(MAKER), Some(before));
    assert!(matches!(
        h.suggest(MAKER, json!("Y")),
        WriteOutcome::Written(_)
    ));
    assert_suggested(&h.row(MAKER).unwrap(), json!("Y"));
}

#[test]
fn a07_c1j_default_and_integration_clears_act_like_ai_clears() {
    for provenance in [Provenance::Default, Provenance::Integration] {
        let h = Harness::new();
        let before = stored(&h, MAKER, json!("X"), provenance, Some(AiStatus::Cleared));
        assert_eq!(
            h.suggest(MAKER, json!("X")),
            WriteOutcome::MatchesCleared,
            "{provenance:?}"
        );
        assert_eq!(h.row(MAKER), Some(before));
        assert!(matches!(
            h.suggest(MAKER, json!("Y")),
            WriteOutcome::Written(_)
        ));
        assert_suggested(&h.row(MAKER).unwrap(), json!("Y"));
    }
}

#[test]
fn a07_c1k_ai_without_status_fails_closed() {
    let h = Harness::new();
    let before = stored(&h, MAKER, json!("A"), Provenance::Ai, None);
    assert!(refused(&h.suggest(MAKER, json!("B"))));
    assert_eq!(h.row(MAKER), Some(before));
}

#[test]
fn a07_c2a_replaced_default_is_a_conflict() {
    let h = Harness::new();
    stored(&h, MAKER, json!("D"), Provenance::Default, None);
    h.suggest(MAKER, json!("A"));
    assert!(is_conflict(&h.definition(MAKER), &h.row(MAKER).unwrap()));
}

#[test]
fn a07_c2b_same_value_as_default_not_written() {
    let h = Harness::new();
    let before = stored(&h, MAKER, json!("Teak"), Provenance::Default, None);
    let outcome = h.suggest(MAKER, json!("teak "));
    assert_eq!(outcome, WriteOutcome::AlreadyHolds);
    assert_eq!(outcome.text(), "already holds this value");
    assert_eq!(h.row(MAKER), Some(before));
}

#[test]
fn a07_c2c_suggestion_same_as_prior_is_no_conflict() {
    let h = Harness::new();
    suggested_with_prior(&h, "A", "D");
    assert!(matches!(
        h.suggest(MAKER, json!("d")),
        WriteOutcome::Written(_)
    ));
    let row = h.row(MAKER).unwrap();
    assert_suggested(&row, json!("d"));
    assert_eq!(row.ai_prior, prior(json!("D"), true, Provenance::Default));
    assert!(!is_conflict(&h.definition(MAKER), &row));
}

#[test]
fn a07_c2d_no_prior_no_conflict() {
    let h = Harness::new();
    h.suggest(MAKER, json!("A"));
    assert!(!is_conflict(&h.definition(MAKER), &h.row(MAKER).unwrap()));
}

#[test]
fn a07_c3a_cleared_set_grows() {
    let h = Harness::new();
    h.feedback(MAKER, FeedbackAction::Cleared, json!("X"));
    stored(
        &h,
        MAKER,
        json!("Y"),
        Provenance::Ai,
        Some(AiStatus::Cleared),
    );
    assert_eq!(h.suggest(MAKER, json!("X")), WriteOutcome::MatchesCleared);
    assert_eq!(h.suggest(MAKER, json!("Y")), WriteOutcome::MatchesCleared);
    assert!(matches!(
        h.suggest(MAKER, json!("Z")),
        WriteOutcome::Written(_)
    ));
    assert_suggested(&h.row(MAKER).unwrap(), json!("Z"));
}

#[test]
fn a07_c3b_kept_previous_joins_the_cleared_set() {
    let h = Harness::new();
    let before = stored(&h, MAKER, json!("D"), Provenance::Default, None);
    h.feedback(MAKER, FeedbackAction::KeptPrevious, json!("X"));
    assert_eq!(h.suggest(MAKER, json!("X")), WriteOutcome::MatchesCleared);
    assert_eq!(h.row(MAKER), Some(before));
}

#[test]
fn a07_c3c_membership_by_sameness() {
    let h = Harness::new();
    h.feedback(MAKER, FeedbackAction::Cleared, json!("Hans J. Wegner"));
    assert_eq!(
        h.suggest(MAKER, json!("hans j wegner")),
        WriteOutcome::MatchesCleared
    );
    assert_eq!(h.row(MAKER), None);
}

#[test]
fn a07_c3d_accepted_feedback_is_not_cleared() {
    let h = Harness::new();
    h.feedback(MAKER, FeedbackAction::Accepted, json!("X"));
    assert!(matches!(
        h.suggest(MAKER, json!("X")),
        WriteOutcome::Written(_)
    ));
    assert_suggested(&h.row(MAKER).unwrap(), json!("X"));
}

#[test]
fn a07_c4a_lowering_keeps_value_and_sources() {
    let h = Harness::new();
    h.suggest_with(
        MAKER,
        json!("A"),
        AiConfidence::High,
        Some("label on the back"),
        &["u1".to_owned()],
    );
    assert!(matches!(
        h.lower(MAKER, "the two listings now disagree"),
        WriteOutcome::Written(_)
    ));
    let row = h.row(MAKER).unwrap();
    assert_suggested(&row, json!("A"));
    assert_eq!(row.ai_confidence, Some(AiConfidence::Low));
    assert_eq!(row.ai_sources, Some(vec!["u1".to_owned()]));
    assert_eq!(
        row.ai_reason.as_deref(),
        Some("No longer supported: the two listings now disagree")
    );
}

#[test]
fn a07_c4b_lowering_touches_nothing_else() {
    let h = Harness::new();
    let accepted = stored(
        &h,
        MAKER,
        json!("A"),
        Provenance::Ai,
        Some(AiStatus::Accepted),
    );
    let human = stored(&h, WOOD, json!("Oak"), Provenance::Human, None);
    h.advance(1000);
    assert_eq!(h.lower(MAKER, "gone"), WriteOutcome::Unchanged);
    assert_eq!(h.lower(WOOD, "gone"), WriteOutcome::Unchanged);
    assert_eq!(h.lower(LOCKED, "gone"), WriteOutcome::Unchanged);
    assert_eq!(h.row(MAKER), Some(accepted));
    assert_eq!(h.row(WOOD), Some(human));
    assert_eq!(h.row(LOCKED), None);
}

#[test]
fn a07_c5a_key_outside_layout_not_researched() {
    let h = Harness::new();
    let outcome = h.suggest(OUTSIDE, json!("A"));
    assert_eq!(outcome, WriteOutcome::NotResearched);
    assert_eq!(outcome.text(), "not researched");
    assert_eq!(h.row(OUTSIDE), None);
}

#[test]
fn a07_c5b_never_written_fields_not_researched() {
    let h = Harness::new();
    let title = h.title();
    let cases = [
        (SUBTITLE, json!("A")),
        (MEDIA, json!({"excluded": []})),
        (REFERENCE, json!("gid://x/1")),
        (RICH, json!("A")),
        (MONEY, json!({"amount": "3", "currency": "EUR"})),
        (RATING, json!({"value": "3", "min": "0", "max": "5"})),
        (LOCKED, json!("A")),
        ("product.title", json!("A new title")),
    ];
    for (key, value) in cases {
        assert_eq!(h.suggest(key, value), WriteOutcome::NotResearched, "{key}");
        assert_eq!(h.row(key), None, "{key}");
    }
    assert_eq!(h.title(), title);
}

#[test]
fn a07_c6a_same_suggestion_twice_is_a_no_op() {
    let h = Harness::new();
    let sources = vec!["https://e1.example/p".to_owned()];
    h.advance(1000);
    let written_at = h.now();
    let first = h.suggest_with(
        MAKER,
        json!("A"),
        AiConfidence::Medium,
        Some("why"),
        &sources,
    );
    assert!(matches!(first, WriteOutcome::Written(_)));
    assert_eq!(h.activity(), written_at);
    h.advance(5000);
    let second = h.suggest_with(
        MAKER,
        json!("A"),
        AiConfidence::Medium,
        Some("why"),
        &sources,
    );
    assert_eq!(second, WriteOutcome::Unchanged);
    assert_eq!(h.row(MAKER).unwrap().updated_at, written_at);
    assert_eq!(h.activity(), written_at);
}

#[test]
fn a07_c6b_research_write_records_no_undo_step() {
    let h = Harness::new();
    assert!(matches!(
        h.suggest(MAKER, json!("A")),
        WriteOutcome::Written(_)
    ));
    let state = h.app.history().state(h.session_id);
    assert_eq!(state.undo, None);
    assert_eq!(state.redo, None);
}
