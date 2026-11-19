//! R5·U1 (rows A08·C1–C4): feedback rows for the photographer's actions on suggestions, the
//! conflict actions, and the guarded undo and redo of attribute steps (08B MC-A12, MC-A1).

use psm_core::{
    Error,
    domain::{
        attribute::{
            AiConfidence, AiPrior, AiStatus, AttributeGroup, Preset, PresetField, Provenance, Value,
        },
        research::{Feedback, FeedbackAction},
    },
    services::history::StepKind,
};
use psm_research::writes::WriteOutcome;
use psm_testkit::{RESEARCH_INTEGRATION, TestApp};

const NOTICE: &str = "that field was changed by research";

struct Fx {
    a: TestApp,
    item: i64,
    session: i64,
}

fn fx(keys: &[&str]) -> Fx {
    let a = TestApp::new().unwrap();
    a.research_fields(keys).unwrap();
    let (item, created) = a
        .items()
        .create_local("R5-U1", "Oak chair", "object", None)
        .unwrap();
    Fx {
        item: item.id,
        session: created.session.id,
        a,
    }
}

fn text(s: &str) -> String {
    serde_json::Value::String(s.into()).to_string()
}

impl Fx {
    fn attrs(&self) -> psm_core::services::attributes::AttributesService<'_> {
        self.a.attributes(Some(RESEARCH_INTEGRATION))
    }
    fn row(&self, key: &str) -> Option<Value> {
        let mut out = None;
        self.a
            .store()
            .transaction(&mut |tx| {
                out = tx
                    .attribute_values(self.item)?
                    .into_iter()
                    .find(|v| v.key == key);
                Ok(())
            })
            .unwrap();
        out
    }
    fn feedback(&self, key: &str) -> Vec<Feedback> {
        feedback_of(&self.a, self.item, key)
    }
    fn suggest(&self, key: &str, value: &str, confidence: AiConfidence) -> WriteOutcome {
        self.a
            .research_suggest(self.item, key, value, confidence)
            .unwrap()
    }
    /// The token of the session's last undo step.
    fn last_token(&self) -> String {
        let store = self.a.app.history.store();
        let all = store.lock().unwrap();
        match &all.get(&self.session).unwrap().undo.back().unwrap().kind {
            StepKind::Attribute { token, .. } => token.clone(),
            other => panic!("not an attribute step: {other:?}"),
        }
    }
    fn undo_depth(&self) -> usize {
        let store = self.a.app.history.store();
        let all = store.lock().unwrap();
        all.get(&self.session).map_or(0, |l| l.undo.len())
    }
    fn tick(&self) {
        self.a.clock.advance(1_000);
    }
}

fn feedback_of(a: &TestApp, item: i64, key: &str) -> Vec<Feedback> {
    let mut out = Vec::new();
    a.store()
        .transaction(&mut |tx| {
            out = tx.feedback_by_key(item, key)?;
            Ok(())
        })
        .unwrap();
    out
}

fn without_time(v: Option<Value>) -> Option<Value> {
    v.map(|v| Value { updated_at: 0, ..v })
}

fn assert_cannot(result: psm_core::Result<psm_core::services::history::HistoryOutcome>) {
    match result {
        Err(Error::StepCannotApply { reason, .. }) => assert_eq!(reason, NOTICE),
        other => panic!("expected the research notice, got {other:?}"),
    }
}

#[test]
fn a08_c1a_accept_writes_accepted_feedback_with_step_token() {
    let f = fx(&["k"]);
    f.a.research_conflict(f.item, "k", "D", "A", AiConfidence::Medium)
        .unwrap();
    f.tick();
    f.attrs().accept(f.item, "k").unwrap();
    let row = f.row("k").unwrap();
    assert_eq!(row.provenance, Provenance::Ai);
    assert_eq!(row.ai_status, Some(AiStatus::Accepted));
    assert_eq!(row.value, text("A"));
    assert_eq!(row.ai_prior, None);
    let feedback = f.feedback("k");
    assert_eq!(feedback.len(), 1);
    let fb = &feedback[0];
    assert_eq!(fb.action, FeedbackAction::Accepted);
    assert_eq!(fb.suggested_value, text("A"));
    assert_eq!(fb.confidence, AiConfidence::Medium);
    assert_eq!(fb.final_value, Some(text("A")));
    assert_eq!(fb.step_token, Some(f.last_token()));
}

#[test]
fn a08_c1b_accept_all_skips_low_and_conflicts() {
    let f = fx(&["k1", "k2", "k3"]);
    f.suggest("k1", "A", AiConfidence::Medium);
    f.suggest("k2", "B", AiConfidence::Low);
    f.a.research_conflict(f.item, "k3", "D", "C", AiConfidence::High)
        .unwrap();
    assert_eq!(f.attrs().accept_all(f.item).unwrap(), 1);
    assert_eq!(f.row("k1").unwrap().ai_status, Some(AiStatus::Accepted));
    assert_eq!(f.row("k2").unwrap().ai_status, Some(AiStatus::Suggested));
    assert_eq!(f.row("k3").unwrap().ai_status, Some(AiStatus::Suggested));
    let all: Vec<_> = ["k1", "k2", "k3"]
        .iter()
        .flat_map(|k| f.feedback(k))
        .collect();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].key, "k1");
    assert_eq!(all[0].action, FeedbackAction::Accepted);
}

#[test]
fn a18_keep_previous_works_on_any_suggestion_with_a_prior_and_never_without_one() {
    let f = fx(&["k", "e"]);
    // A suggestion holding a prior whose value counts as the same (not a conflict).
    f.a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_value(&Value {
                item_id: f.item,
                key: "k".into(),
                value: text("Teak"),
                valid: true,
                provenance: Provenance::Ai,
                ai_confidence: Some(AiConfidence::Medium),
                ai_status: Some(AiStatus::Suggested),
                ai_reason: None,
                ai_sources: None,
                ai_prior: Some(AiPrior {
                    value: text("Teak"),
                    valid: true,
                    provenance: Provenance::Integration,
                    review: None,
                }),
                updated_at: 1,
            })
        })
        .unwrap();
    f.tick();
    let kept = f.attrs().keep_previous(f.item, "k").unwrap();
    assert_eq!(kept.provenance, Provenance::Integration);
    assert_eq!(kept.ai_status, None);
    assert_eq!(f.feedback("k")[0].action, FeedbackAction::KeptPrevious);
    // A suggestion over an empty field has nothing to keep: refused, nothing written.
    f.suggest("e", "Oak", AiConfidence::High);
    let before = f.row("e").unwrap();
    assert!(matches!(
        f.attrs().keep_previous(f.item, "e"),
        Err(Error::InvalidValue(_))
    ));
    assert_eq!(f.row("e").unwrap(), before);
}

#[test]
fn a08_c1c_keep_previous_restores_prior_and_joins_cleared_set() {
    let f = fx(&["k"]);
    f.a.research_conflict(f.item, "k", "D", "A", AiConfidence::Medium)
        .unwrap();
    f.tick();
    f.attrs().keep_previous(f.item, "k").unwrap();
    let row = f.row("k").unwrap();
    assert_eq!(
        Value {
            updated_at: 0,
            ..row
        },
        Value {
            item_id: f.item,
            key: "k".into(),
            value: text("D"),
            valid: true,
            provenance: Provenance::Default,
            ai_confidence: None,
            ai_status: None,
            ai_reason: None,
            ai_sources: None,
            ai_prior: None,
            updated_at: 0,
        }
    );
    let feedback = f.feedback("k");
    assert_eq!(feedback.len(), 1);
    assert_eq!(feedback[0].action, FeedbackAction::KeptPrevious);
    assert_eq!(feedback[0].suggested_value, text("A"));
    assert_eq!(feedback[0].final_value, Some(text("D")));
    assert_eq!(
        f.suggest("k", "A", AiConfidence::High),
        WriteOutcome::MatchesCleared
    );
}

#[test]
fn a08_c1d_clear_suggested_and_accepted_rows() {
    let f = fx(&["k1", "k2"]);
    f.a.research_conflict(f.item, "k1", "D", "A", AiConfidence::Medium)
        .unwrap();
    // An accepted row, stored directly (an accept would write its own feedback row).
    f.a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_value(&Value {
                item_id: f.item,
                key: "k2".into(),
                value: text("B"),
                valid: true,
                provenance: Provenance::Ai,
                ai_confidence: Some(AiConfidence::High),
                ai_status: Some(AiStatus::Accepted),
                ai_reason: None,
                ai_sources: Some(vec![]),
                ai_prior: Some(AiPrior {
                    value: text("B"),
                    valid: true,
                    provenance: Provenance::Default,
                    review: None,
                }),
                updated_at: 1,
            })
        })
        .unwrap();
    f.tick();
    f.attrs().clear(f.item, "k1").unwrap();
    f.attrs().clear(f.item, "k2").unwrap();
    for (key, suggested) in [("k1", "A"), ("k2", "B")] {
        let row = f.row(key).unwrap();
        assert_eq!(row.ai_status, Some(AiStatus::Cleared), "{key}");
        assert_eq!(row.ai_prior, None, "{key}");
        let feedback = f.feedback(key);
        assert_eq!(feedback.len(), 1, "{key}");
        assert_eq!(feedback[0].action, FeedbackAction::Cleared);
        assert_eq!(feedback[0].suggested_value, text(suggested));
        assert_eq!(feedback[0].final_value, None);
    }
}

#[test]
fn a08_c1e_human_edit_of_suggestion_is_edited_feedback() {
    let f = fx(&["k"]);
    f.a.research_conflict(f.item, "k", "D", "A", AiConfidence::Medium)
        .unwrap();
    f.tick();
    f.attrs()
        .set_value(f.item, "k", serde_json::json!("Walnut"), Provenance::Human)
        .unwrap();
    let row = f.row("k").unwrap();
    assert_eq!(row.provenance, Provenance::Human);
    assert_eq!(row.value, text("Walnut"));
    assert_eq!(row.ai_prior, None);
    let feedback = f.feedback("k");
    assert_eq!(feedback.len(), 1);
    assert_eq!(feedback[0].action, FeedbackAction::Edited);
    assert_eq!(feedback[0].suggested_value, text("A"));
    assert_eq!(feedback[0].final_value, Some(text("Walnut")));
}

#[test]
fn a08_c1f_actions_on_non_suggestion_rows_write_no_feedback() {
    let f = fx(&["human", "def", "gone"]);
    let attrs = f.attrs();
    attrs
        .set_value(f.item, "human", serde_json::json!("H1"), Provenance::Human)
        .unwrap();
    f.tick();
    attrs
        .set_value(f.item, "human", serde_json::json!("H2"), Provenance::Human)
        .unwrap();
    f.a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_value(&Value {
                item_id: f.item,
                key: "def".into(),
                value: text("D"),
                valid: true,
                provenance: Provenance::Default,
                ai_confidence: None,
                ai_status: None,
                ai_reason: None,
                ai_sources: None,
                ai_prior: None,
                updated_at: 1,
            })
        })
        .unwrap();
    attrs.clear(f.item, "def").unwrap();
    assert_eq!(f.row("def").unwrap().ai_status, Some(AiStatus::Cleared));
    f.suggest("gone", "A", AiConfidence::Medium);
    attrs.clear(f.item, "gone").unwrap();
    let before_gone = f.feedback("gone").len();
    f.tick();
    attrs
        .set_value(f.item, "gone", serde_json::json!("H"), Provenance::Human)
        .unwrap();
    assert!(f.feedback("human").is_empty());
    assert!(f.feedback("def").is_empty());
    assert_eq!(f.feedback("gone").len(), before_gone);
    assert_eq!(f.row("gone").unwrap().provenance, Provenance::Human);
}

#[test]
fn a08_c2a_research_write_is_no_feedback_and_no_step() {
    let f = fx(&["k"]);
    let before = f.a.history().state(f.session);
    assert!(matches!(
        f.suggest("k", "A", AiConfidence::High),
        WriteOutcome::Written(_)
    ));
    assert!(f.feedback("k").is_empty());
    assert_eq!(f.a.history().state(f.session), before);
}

#[test]
fn a08_c3a_undo_refused_after_research_write() {
    let f = fx(&["k"]);
    f.suggest("k", "A", AiConfidence::Medium);
    f.attrs().clear(f.item, "k").unwrap();
    assert!(matches!(
        f.suggest("k", "B", AiConfidence::Medium),
        WriteOutcome::Written(_)
    ));
    let before = f.row("k");
    assert_cannot(f.a.history().undo(f.session));
    assert_eq!(f.row("k"), before);
    let row = f.row("k").unwrap();
    assert_eq!(
        (row.value, row.ai_status),
        (text("B"), Some(AiStatus::Suggested))
    );
    assert_eq!(f.a.history().state(f.session).undo, None);
}

#[test]
fn a08_c3b_redo_refused_after_research_replaced_suggestion() {
    let f = fx(&["k"]);
    f.suggest("k", "A", AiConfidence::Medium);
    f.attrs().clear(f.item, "k").unwrap();
    f.a.history().undo(f.session).unwrap();
    assert_eq!(f.row("k").unwrap().ai_status, Some(AiStatus::Suggested));
    assert!(matches!(
        f.suggest("k", "B", AiConfidence::Medium),
        WriteOutcome::Written(_)
    ));
    let before = f.row("k");
    assert_cannot(f.a.history().redo(f.session));
    assert_eq!(f.row("k"), before);
    assert_eq!(f.a.history().state(f.session).redo, None);
}

#[test]
fn a08_c3c_two_field_undo_refused_when_one_field_changed() {
    let f = fx(&["x", "y"]);
    f.attrs()
        .presets_set(Preset {
            item_type: "object".into(),
            group: AttributeGroup::Details,
            layout: vec![
                PresetField {
                    key: "x".into(),
                    default: Some(serde_json::json!("DX")),
                },
                PresetField {
                    key: "y".into(),
                    default: Some(serde_json::json!("DY")),
                },
            ],
        })
        .unwrap();
    assert_eq!(f.attrs().fill_defaults(f.item).unwrap(), 2);
    assert!(matches!(
        f.suggest("y", "S", AiConfidence::Medium),
        WriteOutcome::Written(_)
    ));
    let (x, y) = (f.row("x"), f.row("y"));
    assert_cannot(f.a.history().undo(f.session));
    assert_eq!(f.row("x"), x);
    assert_eq!(f.row("y"), y);
    assert_eq!(x.unwrap().provenance, Provenance::Default);
    assert_eq!(y.unwrap().value, text("S"));
}

#[test]
fn a08_c3d_undo_of_unchanged_field_restores_row() {
    let f = fx(&["k"]);
    f.suggest("k", "A", AiConfidence::Medium);
    let before = f.row("k");
    f.tick();
    f.attrs().clear(f.item, "k").unwrap();
    f.tick();
    f.a.history().undo(f.session).unwrap();
    assert_eq!(without_time(f.row("k")), without_time(before));
}

#[test]
fn a08_c4a_undo_deletes_feedback_and_cleared_value_returns() {
    let f = fx(&["k"]);
    f.suggest("k", "A", AiConfidence::Medium);
    f.attrs().clear(f.item, "k").unwrap();
    assert_eq!(f.feedback("k").len(), 1);
    f.a.history().undo(f.session).unwrap();
    assert!(f.feedback("k").is_empty());
    let restored = f.row("k");
    assert_eq!(
        f.suggest("k", "A", AiConfidence::Medium),
        WriteOutcome::Unchanged
    );
    assert_eq!(f.row("k"), restored);
}

#[test]
fn a08_c4b_redo_reinserts_feedback_with_same_token() {
    let f = fx(&["k"]);
    f.suggest("k", "A", AiConfidence::Medium);
    f.attrs().clear(f.item, "k").unwrap();
    let original = f.feedback("k");
    assert_eq!(original.len(), 1);
    f.a.history().undo(f.session).unwrap();
    // Another session's clear writes a feedback row between the undo and the redo (its id may
    // be the one the undo freed).
    let (other, _) =
        f.a.items()
            .create_local("R5-U1-B", "Pine chair", "object", None)
            .unwrap();
    f.a.research_suggest(other.id, "k", "Z", AiConfidence::Medium)
        .unwrap();
    f.attrs().clear(other.id, "k").unwrap();
    f.a.history().redo(f.session).unwrap();
    let again = f.feedback("k");
    assert_eq!(again.len(), 1);
    assert_eq!(again[0].step_token, original[0].step_token);
    assert_eq!(again[0].action, FeedbackAction::Cleared);
    assert_eq!(again[0].suggested_value, text("A"));
    assert_eq!(f.row("k").unwrap().ai_status, Some(AiStatus::Cleared));
    assert_eq!(feedback_of(&f.a, other.id, "k").len(), 1);
}

#[test]
fn a08_c4c_feedback_outlives_the_step_limit() {
    let keys: Vec<String> = (0..26).map(|n| format!("k{n:02}")).collect();
    let refs: Vec<&str> = keys.iter().map(String::as_str).collect();
    let f = fx(&refs);
    for key in &refs {
        f.suggest(key, "A", AiConfidence::Medium);
    }
    for key in &refs {
        f.attrs().accept(f.item, key).unwrap();
    }
    assert_eq!(f.undo_depth(), 25);
    let total: usize = refs.iter().map(|k| f.feedback(k).len()).sum();
    assert_eq!(total, 26);
}
