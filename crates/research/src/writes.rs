//! The one door research writes form values through (MC-A1). Every read the decision uses — the
//! stored row, the definition, the layout and the cleared set — happens inside the writing
//! transaction. A research write sets the item's activity time and records no undo step.

use psm_core::{
    Result,
    domain::{
        attribute::{
            AiConfidence, AiStatus, AttributeGroup, Definition, PresetField, Provenance, Value,
        },
        research::FeedbackAction,
    },
    mechanisms::values::canonicalize,
    ports::{
        clock::Clock,
        store::{Store, StoreTx},
    },
    services::attributes::builtin_definitions,
};
use serde_json::Value as Json;

use crate::rules::write_table::{WriteDecision, decide_write, never_written};

/// What a research write did, for the trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteOutcome {
    /// The row now stored.
    Written(Value),
    /// The stored row already has this exact content; nothing was written.
    Unchanged,
    /// The write table forbids writing over the stored row (the table's reason).
    Refused(String),
    /// The value is in the cleared set.
    MatchesCleared,
    /// A live default or integration value is already the same.
    AlreadyHolds,
    /// The field is one research never writes.
    NotResearched,
}

impl WriteOutcome {
    /// The trace text of the outcome.
    pub fn text(&self) -> String {
        match self {
            WriteOutcome::Written(_) => "written".to_owned(),
            WriteOutcome::Unchanged => "unchanged".to_owned(),
            WriteOutcome::Refused(reason) => reason.clone(),
            WriteOutcome::MatchesCleared => "matches a value you cleared".to_owned(),
            WriteOutcome::AlreadyHolds => "already holds this value".to_owned(),
            WriteOutcome::NotResearched => "not researched".to_owned(),
        }
    }
}

/// The definitions research can see: the built-in Product fields, then the integration's.
fn definitions(tx: &dyn StoreTx, integration: Option<&str>) -> Result<Vec<Definition>> {
    let mut all = builtin_definitions();
    if let Some(integration) = integration {
        all.extend(tx.attribute_definitions(integration)?);
    }
    Ok(all)
}

/// The step of the item's current layout that holds `key` (the presets of its item type, all
/// three steps; a step with no stored preset lists every definition of that step).
fn layout_step(
    tx: &dyn StoreTx,
    item_type: &str,
    key: &str,
    all: &[Definition],
) -> Result<Option<AttributeGroup>> {
    for group in AttributeGroup::ALL {
        let own = tx.attribute_preset(item_type, group)?;
        let preset = match own {
            Some(preset) => Some(preset),
            None if item_type.is_empty() => None,
            None => tx.attribute_preset("", group)?,
        };
        let layout: Vec<PresetField> = match preset {
            Some(preset) => preset.layout,
            None => all
                .iter()
                .filter(|d| d.group == group)
                .map(|d| PresetField {
                    key: d.key.clone(),
                    default: None,
                })
                .collect(),
        };
        if layout.iter().any(|f| f.key == key) {
            return Ok(Some(group));
        }
    }
    Ok(None)
}

fn parsed(text: &str) -> Option<Json> {
    serde_json::from_str(text).ok()
}

/// `C(item, key)`: the stored row's value when it is cleared, plus the suggested value of every
/// feedback row of the field with action `cleared` or `kept_previous`.
fn cleared_set(
    tx: &dyn StoreTx,
    stored: Option<&Value>,
    item_id: i64,
    key: &str,
) -> Result<Vec<Json>> {
    let mut out: Vec<Json> = stored
        .filter(|s| !s.is_live())
        .and_then(|s| parsed(&s.value))
        .into_iter()
        .collect();
    for feedback in tx.feedback_by_key(item_id, key)? {
        if matches!(
            feedback.action,
            FeedbackAction::Cleared | FeedbackAction::KeptPrevious
        ) && let Some(value) = parsed(&feedback.suggested_value)
        {
            out.push(value);
        }
    }
    Ok(out)
}

/// Same stored content (everything but `updated_at` and `ai_prior`): writing it would change
/// nothing (the rule of `same_content` in the attributes service).
fn same_content(a: &Value, b: &Value) -> bool {
    a.value == b.value
        && a.valid == b.valid
        && a.provenance == b.provenance
        && a.ai_confidence == b.ai_confidence
        && a.ai_status == b.ai_status
        && a.ai_reason == b.ai_reason
        && a.ai_sources == b.ai_sources
}

/// Writes one research suggestion through MC-A1's table, in one store transaction.
#[allow(clippy::too_many_arguments)]
pub fn suggest(
    store: &dyn Store,
    clock: &dyn Clock,
    integration: Option<&str>,
    item_id: i64,
    key: &str,
    value: &Json,
    confidence: AiConfidence,
    reason: Option<&str>,
    sources: &[String],
) -> Result<WriteOutcome> {
    let now = clock.now_ms();
    let mut outcome = WriteOutcome::Unchanged;
    store.transaction(&mut |tx| {
        let item = tx.item(item_id)?;
        let all = definitions(tx, integration)?;
        let Some(definition) = all.iter().find(|d| d.key == key) else {
            outcome = WriteOutcome::NotResearched;
            return Ok(());
        };
        let step = layout_step(tx, &item.item_type, key, &all)?;
        if never_written(definition, step) {
            outcome = WriteOutcome::NotResearched;
            return Ok(());
        }
        let stored = tx
            .attribute_values(item_id)?
            .into_iter()
            .find(|v| v.key == key);
        let cleared = cleared_set(tx, stored.as_ref(), item_id, key)?;
        let ai_prior = match decide_write(definition, stored.as_ref(), value, &cleared) {
            WriteDecision::Write { ai_prior } => ai_prior,
            WriteDecision::MatchesCleared => {
                outcome = WriteOutcome::MatchesCleared;
                return Ok(());
            }
            WriteDecision::AlreadyHolds => {
                outcome = WriteOutcome::AlreadyHolds;
                return Ok(());
            }
            WriteDecision::Refused(why) => {
                outcome = WriteOutcome::Refused(why.to_owned());
                return Ok(());
            }
        };
        let Ok(canonical) = canonicalize(definition.core_type, definition, value.clone()) else {
            outcome = WriteOutcome::Refused("the value is not valid for this field".to_owned());
            return Ok(());
        };
        let row = Value {
            item_id,
            key: key.to_owned(),
            value: canonical.0,
            valid: true,
            provenance: Provenance::Ai,
            ai_confidence: Some(confidence),
            ai_status: Some(AiStatus::Suggested),
            ai_reason: reason.map(str::to_owned),
            ai_sources: Some(sources.to_vec()),
            ai_prior,
            updated_at: now,
        };
        if stored.as_ref().is_some_and(|s| same_content(s, &row)) {
            outcome = WriteOutcome::Unchanged;
            return Ok(());
        }
        tx.put_attribute_value(&row)?;
        tx.set_item_activity(item_id, now)?;
        outcome = WriteOutcome::Written(row);
        Ok(())
    })?;
    Ok(outcome)
}

/// MC-A1 lowering: on an `ai` + `suggested` row, the same value at `low`, sources kept, reason
/// `No longer supported: <reason>`. Any other row (or none) is left alone (`Unchanged`).
pub fn lower(
    store: &dyn Store,
    clock: &dyn Clock,
    item_id: i64,
    key: &str,
    reason: &str,
) -> Result<WriteOutcome> {
    let now = clock.now_ms();
    let mut outcome = WriteOutcome::Unchanged;
    store.transaction(&mut |tx| {
        tx.item(item_id)?;
        let Some(stored) = tx
            .attribute_values(item_id)?
            .into_iter()
            .find(|v| v.key == key)
        else {
            outcome = WriteOutcome::Unchanged;
            return Ok(());
        };
        if stored.provenance != Provenance::Ai || stored.ai_status != Some(AiStatus::Suggested) {
            outcome = WriteOutcome::Unchanged;
            return Ok(());
        }
        let row = Value {
            ai_confidence: Some(AiConfidence::Low),
            ai_reason: Some(format!("No longer supported: {reason}")),
            updated_at: now,
            ..stored.clone()
        };
        if same_content(&stored, &row) {
            outcome = WriteOutcome::Unchanged;
            return Ok(());
        }
        tx.put_attribute_value(&row)?;
        tx.set_item_activity(item_id, now)?;
        outcome = WriteOutcome::Written(row);
        Ok(())
    })?;
    Ok(outcome)
}
