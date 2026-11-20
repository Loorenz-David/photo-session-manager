//! W1: the product note (book 08C §4; `master_plan_writer.md` §5 WC1 items 5–9 and WC2).
//!
//! Which note counts (WC1.5), the photo numbers of the condition observations (WC1.6), the
//! owner's decision over each tagged key and the replacement it makes (WC2), staleness (WC1.7,
//! computed at read time, never stored) and **Look again** (WC1.8). The pure rules are
//! [`decision`], [`reviewed`] and [`stale`].

use psm_core::{
    Error, Result,
    domain::{
        attribute::{AiConfidence, AiStatus, Provenance, Value},
        photo::Classification,
        research::{Feedback, FeedbackAction},
    },
    ports::store::StoreTx,
    services::attributes::AttributesService,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{
    api::Reason,
    config::{ResearchConfig, ResearchSettings},
    domain::{FieldClass, RunState, RunTrigger, Stage},
    rules::fingerprint::fingerprint,
    service::{FormSnapshot, MODEL_ACCOUNT, ResearchService, form_snapshot},
    shop::ShopFacts,
    stages::{self, OBSERVE_UNIT, UsablePhoto},
    store::{self, OutputRow, RunRow},
    writer::api::Requested,
};

/// The built-in title key (its stored state is WC4.3's).
pub const PRODUCT_TITLE: &str = "product.title";
/// The integration whose link title base decides a title's provenance (the rule `form_get` uses).
use psm_core::services::attributes::LINK_INTEGRATION as TITLE_LINK_INTEGRATION;
/// How the panel marks a condition observation whose photo left the condition photos (WC1.6).
pub const PHOTO_GONE: &str = "photo no longer in the condition photos";

/// The owner's standing on one key (WC2 "Decision of a key", first match wins).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Accepted,
    Corrected,
    KeptPrevious,
    Dismissed,
    Typed,
    /// An unchecked research suggestion, with its confidence.
    Unchecked(Option<AiConfidence>),
    ShopDefault,
    Imported,
    Empty,
}

impl Decision {
    pub fn code(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Corrected => "corrected",
            Self::KeptPrevious => "kept_previous",
            Self::Dismissed => "dismissed",
            Self::Typed => "typed",
            Self::Unchecked(_) => "unchecked",
            Self::ShopDefault => "shop_default",
            Self::Imported => "imported",
            Self::Empty => "empty",
        }
    }

    /// The owner's decision (08C §4 list): accepted, corrected, kept previous, dismissed, typed.
    pub fn is_owner(self) -> bool {
        matches!(
            self,
            Self::Accepted | Self::Corrected | Self::KeptPrevious | Self::Dismissed | Self::Typed
        )
    }

    /// Undecided (08C §6): typed values research never looked at, unchecked suggestions, shop
    /// defaults, imported values.
    pub fn is_undecided(self) -> bool {
        matches!(
            self,
            Self::Typed | Self::Unchecked(_) | Self::ShopDefault | Self::Imported
        )
    }
}

/// The latest feedback row by `(at, id)`.
fn latest(feedback: &[Feedback]) -> Option<&Feedback> {
    feedback.iter().max_by_key(|f| (f.at, f.id))
}

/// WC2: the decision of a key from its stored row (for `product.title` the stored state of
/// WC4.3) and its `research_feedback` rows.
pub fn decision(row: Option<&Value>, feedback: &[Feedback]) -> Decision {
    let Some(row) = row else {
        return Decision::Empty;
    };
    if row.ai_status == Some(AiStatus::Cleared) {
        return Decision::Dismissed;
    }
    if !row.valid {
        return Decision::Empty;
    }
    let action = latest(feedback).map(|f| f.action);
    match row.provenance {
        Provenance::Ai if row.ai_status == Some(AiStatus::Accepted) => Decision::Accepted,
        Provenance::Ai => Decision::Unchecked(row.ai_confidence),
        Provenance::Human if action == Some(FeedbackAction::Edited) => Decision::Corrected,
        Provenance::Human => Decision::Typed,
        Provenance::Default | Provenance::Integration
            if action == Some(FeedbackAction::KeptPrevious) =>
        {
            Decision::KeptPrevious
        }
        Provenance::Default => Decision::ShopDefault,
        Provenance::Integration => Decision::Imported,
    }
}

/// WC2 / HW3, the never-re-raise predicate: the row is cleared, or an accepted `ai` value, or
/// some feedback row of the key exists and the row is not an unchecked `ai` suggestion.
pub fn reviewed(row: Option<&Value>, feedback: &[Feedback]) -> bool {
    let suggested = row.is_some_and(|r| {
        r.provenance == Provenance::Ai && r.ai_status == Some(AiStatus::Suggested)
    });
    row.is_some_and(|r| {
        r.ai_status == Some(AiStatus::Cleared)
            || (r.provenance == Provenance::Ai && r.ai_status == Some(AiStatus::Accepted))
    }) || (!feedback.is_empty() && !suggested)
}

/// What staleness is read from (WC1.7): for each photo look, `None` when there is no note, else
/// whether the photos it would look at now are the ones it looked at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaleInput {
    pub main_same: Option<bool>,
    pub condition_same: Option<bool>,
    pub usable_main: bool,
    pub usable_condition: bool,
    /// A Condition field is chosen (`research.fill_keys`).
    pub condition_chosen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Staleness {
    pub stale_main: bool,
    pub stale_condition: bool,
    pub missing_main: bool,
}

/// WC1.7 (amended 2026-11-19, W5·U4: `stale_condition` needs a chosen Condition field, so no
/// banner shows that Look again cannot clear).
pub fn stale(input: &StaleInput) -> Staleness {
    Staleness {
        stale_main: input.main_same == Some(false),
        stale_condition: input.condition_chosen
            && match input.condition_same {
                Some(same) => !same,
                None => input.usable_condition,
            },
        missing_main: input.main_same.is_none() && input.usable_main,
    }
}

/// One key of an observation and its decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyDecision {
    pub key: String,
    /// [`Decision::code`].
    pub decision: String,
}

/// One observation of the note as the panel shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    /// `p1.<i>` (stage 1) or `c1.<i>` (stage 6), i from 1 in the stored note.
    #[serde(rename = "ref")]
    pub reference: String,
    /// The text research wrote.
    pub original: String,
    /// The text shown and given to the writer (replaced by the owner's decisions, WC2).
    pub text: String,
    pub keys: Vec<String>,
    /// "condition photo n", [`PHOTO_GONE`], or none (stage 1).
    pub photo: Option<String>,
    /// The text states the owner's decision.
    pub owner: bool,
    pub decisions: Vec<KeyDecision>,
    /// False only for a condition observation whose photo left the condition photos (WC1.6).
    pub in_writer_input: bool,
}

/// The original text of an observation for a key the owner has not decided (WC2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchSaw {
    pub key: String,
    pub text: String,
}

/// The session's product note (WC1, WC2).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteView {
    pub observations: Vec<Observation>,
    pub research_saw: Vec<ResearchSaw>,
    pub stale_main: bool,
    pub stale_condition: bool,
    pub missing_main: bool,
}

/// Why Look again queued nothing (WC1.8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookBlock {
    NoIntegration,
    NoModelKey,
    PriceMissing { provider: String, model: String },
    Busy,
    NotStale,
}

impl LookBlock {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoIntegration => "NoIntegration",
            Self::NoModelKey => "NoModelKey",
            Self::PriceMissing { .. } => "PriceMissing",
            Self::Busy => "Busy",
            Self::NotStale => "NotStale",
        }
    }

    pub fn sentence(&self) -> String {
        match self {
            Self::NoIntegration => "Connect a shop to use research".to_owned(),
            Self::NoModelKey => "Add the model key in Settings → Research".to_owned(),
            Self::PriceMissing { provider, model } => {
                format!("No price is set for {provider} {model}")
                    .trim_end()
                    .to_owned()
            }
            Self::Busy => "Research is still working on this session".to_owned(),
            Self::NotStale => "The photos have not changed since they were looked at".to_owned(),
        }
    }

    pub fn reason(&self) -> Reason {
        Reason {
            code: self.code().to_owned(),
            sentence: self.sentence(),
        }
    }
}

/// A photo look's note: the run, its committed unit and the stored note.
struct Look {
    run: RunRow,
    unit: OutputRow,
    note: Vec<Json>,
}

/// WC1.5: the latest run of the session that looks at photos and has a committed unit of
/// `stage` whose output has the key `note`.
fn latest_note(runs: &[RunRow], conn: &rusqlite::Connection, stage: Stage) -> Result<Option<Look>> {
    let unit_key = match stage {
        Stage::Observe => OBSERVE_UNIT,
        _ => "condition",
    };
    for run in runs.iter().rev() {
        if !run.trigger.looks_at_photos() {
            continue;
        }
        let Some(unit) = store::output(conn, run.id, stage.as_str(), unit_key)? else {
            continue;
        };
        let output: Json = serde_json::from_str(&unit.output).unwrap_or(Json::Null);
        if let Some(note) = output.get("note").and_then(Json::as_array) {
            return Ok(Some(Look {
                run: run.clone(),
                note: note.clone(),
                unit,
            }));
        }
    }
    Ok(None)
}

/// A queued run of the item, as `usable_photos` reads eligibility for it.
fn probe(item_id: i64, session_id: i64) -> RunRow {
    RunRow {
        id: 0,
        item_id,
        session_id,
        trigger: RunTrigger::Look,
        state: RunState::Queued,
        stage_reached: None,
        queued_at: 0,
        started_at: None,
        finished_at: None,
        cost_micro_usd: 0,
        settings_snapshot: None,
        form_snapshot: None,
        input_snapshot: None,
        error: None,
    }
}

/// The unit's fingerprint equals the one `identity` gives.
fn same_unit(stage: Stage, identity: &Json, unit: &OutputRow) -> bool {
    fingerprint(stage.number(), identity).is_ok_and(|f| f == unit.fingerprint)
}

/// WC4.3 (read here for W1): the stored state of `key`, as the writer's door reads it
/// ([`crate::writer::writes::stored_state`], A29: a title that is the SKU is none).
fn stored_state(
    tx: &dyn StoreTx,
    item_id: i64,
    rows: &[Value],
    key: &str,
) -> Result<Option<Value>> {
    let row = rows.iter().find(|r| r.key == key);
    if key != PRODUCT_TITLE {
        return Ok(row.cloned());
    }
    let item = tx.item(item_id)?;
    let title_base = if row.is_none() {
        tx.item_title_base(item_id, TITLE_LINK_INTEGRATION)?
    } else {
        None
    };
    Ok(crate::writer::writes::stored_state(
        row,
        item_id,
        key,
        &item.title,
        title_base.as_deref(),
        &item.sku,
    ))
}

/// One key's decision and what the replaced text says for it.
struct Decided {
    decision: Decision,
    /// "<label>: <value> (owner's decision)" or the dismissed form; for owner's decisions only.
    owner_text: Option<String>,
}

impl ResearchService {
    /// `writer::note::assemble`: the session's product note as the owner's decisions show it
    /// (WC1.5–7, WC2).
    pub fn assemble(&self, session_id: i64) -> Result<NoteView> {
        let Some(item_id) = self.session_item(session_id)? else {
            return Ok(NoteView::default());
        };
        let (main, condition) = self.store.research_transaction(|tx| {
            let runs = store::all_runs(tx.conn(), Some(session_id))?;
            Ok((
                latest_note(&runs, tx.conn(), Stage::Observe)?,
                latest_note(&runs, tx.conn(), Stage::Condition)?,
            ))
        })?;
        let settings = self.settings();
        let usable = stages::usable_photos(self, &probe(item_id, session_id))?;
        let staleness = stale(&StaleInput {
            main_same: main
                .as_ref()
                .map(|look| self.main_same(look, &usable, &settings)),
            condition_same: condition
                .as_ref()
                .map(|look| condition_same(look, &usable))
                .transpose()?,
            usable_main: usable
                .iter()
                .any(|p| p.classification == Classification::Main),
            usable_condition: usable
                .iter()
                .any(|p| p.classification == Classification::Condition),
            condition_chosen: self.condition_chosen(item_id, &settings)?,
        });
        let form = AttributesService::new(
            self.store.as_ref(),
            self.clock.as_ref(),
            self.integration.as_deref(),
        )
        .form_get(item_id)?;
        let label = |key: &str| -> String {
            form.groups
                .iter()
                .flat_map(|g| g.fields.iter())
                .find(|f| f.definition.key == key)
                .map(|f| f.definition.label.clone())
                .unwrap_or_else(|| key.to_owned())
        };
        // WC1.6: the selected condition photos (Condition by (position, id), minus the damage
        // report's excluded): the first media-list field of the form, else every Condition photo.
        let selected: Vec<i64> = match form
            .groups
            .iter()
            .flat_map(|g| g.fields.iter())
            .find_map(|f| f.selected.clone())
        {
            Some(selected) => selected,
            None => self.store.research_transaction(|tx| {
                let mut photos: Vec<_> = tx
                    .photos(session_id)?
                    .into_iter()
                    .filter(|p| p.classification == Classification::Condition)
                    .collect();
                photos.sort_by_key(|p| (p.position, p.id));
                Ok(photos.into_iter().map(|p| p.id).collect())
            })?,
        };
        let mut entries: Vec<(String, &Json, Option<String>, bool)> = Vec::new();
        for (i, entry) in main.iter().flat_map(|l| l.note.iter()).enumerate() {
            entries.push((format!("p1.{}", i + 1), entry, None, true));
        }
        for (i, entry) in condition.iter().flat_map(|l| l.note.iter()).enumerate() {
            let (photo, kept) = match entry["photo_id"].as_i64() {
                Some(id) => match selected.iter().position(|p| *p == id) {
                    Some(n) => (Some(format!("condition photo {}", n + 1)), true),
                    None => (Some(PHOTO_GONE.to_owned()), false),
                },
                None => (None, true),
            };
            entries.push((format!("c1.{}", i + 1), entry, photo, kept));
        }
        let mut keys: Vec<String> = Vec::new();
        for (_, entry, _, _) in &entries {
            for key in entry["keys"].as_array().into_iter().flatten() {
                if let Some(key) = key.as_str()
                    && !keys.iter().any(|k| k == key)
                {
                    keys.push(key.to_owned());
                }
            }
        }
        let decided: Vec<(String, Decided)> = self.store.research_transaction(|tx| {
            let rows = tx.attribute_values(item_id)?;
            let mut out = Vec::new();
            for key in &keys {
                let row = stored_state(tx, item_id, &rows, key)?;
                let feedback = tx.feedback_by_key(item_id, key)?;
                let decision = decision(row.as_ref(), &feedback);
                let owner_text = match decision {
                    Decision::Dismissed => {
                        let mut values: Vec<String> = Vec::new();
                        for value in crate::writes::cleared_set(tx, row.as_ref(), item_id, key)? {
                            let text = stages::display(&value);
                            if !values.contains(&text) {
                                values.push(text);
                            }
                        }
                        let dismissed = if values.is_empty() {
                            "it".to_owned()
                        } else {
                            values.join(", ")
                        };
                        Some(format!(
                            "{}: not stated (the owner dismissed {dismissed})",
                            label(key)
                        ))
                    }
                    d if d.is_owner() => {
                        let value = row
                            .as_ref()
                            .and_then(Value::json)
                            .map(|v| stages::display(&v))
                            .unwrap_or_default();
                        Some(format!("{}: {value} (owner's decision)", label(key)))
                    }
                    _ => None,
                };
                out.push((
                    key.clone(),
                    Decided {
                        decision,
                        owner_text,
                    },
                ));
            }
            Ok(out)
        })?;
        let of = |key: &str| decided.iter().find(|(k, _)| k == key).map(|(_, d)| d);
        let mut view = NoteView {
            stale_main: staleness.stale_main,
            stale_condition: staleness.stale_condition,
            missing_main: staleness.missing_main,
            ..NoteView::default()
        };
        for (reference, entry, photo, kept) in entries {
            let original = entry["text"].as_str().unwrap_or_default().to_owned();
            let keys: Vec<String> = entry["keys"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|k| k.as_str().map(str::to_owned))
                .collect();
            let owner_texts: Vec<String> = keys
                .iter()
                .filter_map(|k| of(k).and_then(|d| d.owner_text.clone()))
                .collect();
            let owner = !owner_texts.is_empty();
            if kept {
                // WC2 `research_saw`: the original words for every key the owner has not decided
                // (a typed value included: research never looked at it).
                for key in &keys {
                    if of(key).is_some_and(|d| {
                        !matches!(
                            d.decision,
                            Decision::Accepted
                                | Decision::Corrected
                                | Decision::KeptPrevious
                                | Decision::Dismissed
                        )
                    }) {
                        view.research_saw.push(ResearchSaw {
                            key: key.clone(),
                            text: original.clone(),
                        });
                    }
                }
            }
            view.observations.push(Observation {
                reference,
                text: if owner {
                    owner_texts.join("; ")
                } else {
                    original.clone()
                },
                original,
                decisions: keys
                    .iter()
                    .map(|k| KeyDecision {
                        key: k.clone(),
                        decision: of(k)
                            .map_or(Decision::Empty, |d| d.decision)
                            .code()
                            .to_owned(),
                    })
                    .collect(),
                keys,
                photo,
                owner,
                in_writer_input: kept,
            });
        }
        Ok(view)
    }

    /// WC1.7 `stale_main`: the photos stage 1 would look at now (under the live photo limit), as
    /// `N1`'s run would have identified them, are the ones its unit looked at.
    fn main_same(&self, look: &Look, usable: &[UsablePhoto], settings: &ResearchSettings) -> bool {
        let Ok(config) = ResearchConfig::from_snapshot(look.run.settings_snapshot.as_deref())
        else {
            return false;
        };
        let plan = stages::photo_plan(usable, settings.observe_max_photos);
        same_unit(
            Stage::Observe,
            &stages::observe_identity(&config, &plan),
            &look.unit,
        )
    }

    /// Whether a Condition field is chosen now (the rule stage 6 runs by).
    fn condition_chosen(&self, item_id: i64, settings: &ResearchSettings) -> Result<bool> {
        let config = ResearchConfig::from_settings(settings, &ShopFacts::default());
        let form = form_snapshot(
            &self.store,
            self.clock.as_ref(),
            self.integration.as_deref(),
            item_id,
        )?;
        Ok(!stages::fields_in(&config, &form, &[FieldClass::Condition]).is_empty())
    }

    /// WC1.8: the first missing prerequisite of a look run's calls.
    pub(crate) fn look_block(&self, settings: &ResearchSettings) -> Option<LookBlock> {
        if self.shop.is_none() {
            return Some(LookBlock::NoIntegration);
        }
        let Some(model) = self
            .providers
            .model
            .as_ref()
            .filter(|_| self.key_set(MODEL_ACCOUNT))
        else {
            return Some(LookBlock::NoModelKey);
        };
        let fallbacks = settings.fallbacks();
        let provider = model.id();
        [Stage::Observe, Stage::Condition]
            .into_iter()
            .map(|stage| settings.stage(stage).model)
            .find(|m| {
                !fallbacks
                    .iter()
                    .any(|f| f.provider == provider && &f.model == m)
            })
            .map(|model| LookBlock::PriceMissing {
                provider: provider.clone(),
                model,
            })
    }

    /// WC1.8 **Look again**: queues a `look` run when nothing else runs on the session, the
    /// photos changed (or were never looked at) and the two looks' prerequisites hold.
    pub fn look_again(&self, session_id: i64) -> Result<Requested> {
        let Some(item_id) = self.session_item(session_id)? else {
            return Err(Error::InvalidData(
                "the Unassigned session is never researched".into(),
            ));
        };
        let busy = self.store.research_transaction(|tx| {
            Ok(store::all_runs(tx.conn(), Some(session_id))?
                .iter()
                .any(|r| {
                    r.trigger != RunTrigger::Browse
                        && matches!(r.state, RunState::Queued | RunState::Running)
                }))
        })?;
        let block = if busy {
            Some(LookBlock::Busy)
        } else {
            let note = self.assemble(session_id)?;
            if note.stale_main || note.stale_condition || note.missing_main {
                self.look_block(&self.settings())
            } else {
                Some(LookBlock::NotStale)
            }
        };
        if block.is_none() {
            self.insert_queued(item_id, session_id, RunTrigger::Look)?;
        }
        tracing::info!(
            target: "psm::writer",
            "writer.look_again session={session_id} queued={} reason={}",
            block.is_none(),
            block.as_ref().map_or("", LookBlock::code)
        );
        Ok(Requested {
            queued: block.is_none(),
            reason: block.map(|b| b.reason()),
        })
    }
}

/// WC1.7 `stale_condition`: the usable Condition photos now, identified as `N6`'s run did, are
/// the ones its unit looked at.
fn condition_same(look: &Look, usable: &[UsablePhoto]) -> Result<bool> {
    let config = ResearchConfig::from_snapshot(look.run.settings_snapshot.as_deref())?;
    let form = FormSnapshot::from_text(look.run.form_snapshot.as_deref())?;
    let keys = stages::fields_in(&config, &form, &[FieldClass::Condition]);
    let now: Vec<(i64, i64)> = usable
        .iter()
        .filter(|p| p.classification == Classification::Condition)
        .map(|p| (p.id, p.revision))
        .collect();
    Ok(same_unit(
        Stage::Condition,
        &stages::condition_identity(&now, &keys),
        &look.unit,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(provenance: Provenance, status: Option<AiStatus>) -> Value {
        Value {
            item_id: 1,
            key: "custom.wood".into(),
            value: "\"Teak\"".into(),
            valid: true,
            provenance,
            ai_confidence: status.map(|_| AiConfidence::Medium),
            ai_status: status,
            ai_reason: None,
            ai_sources: None,
            ai_prior: None,
            updated_at: 0,
        }
    }

    fn feedback(action: FeedbackAction, at: i64) -> Feedback {
        Feedback {
            id: at,
            item_id: 1,
            key: "custom.wood".into(),
            suggested_value: "\"Oak\"".into(),
            confidence: AiConfidence::Medium,
            final_value: None,
            action,
            at,
            step_token: None,
        }
    }

    use AiStatus::{Accepted, Cleared, Suggested};
    use Provenance::{Ai, Default, Human, Integration};

    #[test]
    fn w1_c6a_a_cleared_ai_row_is_dismissed() {
        assert_eq!(
            decision(Some(&row(Ai, Some(Cleared))), &[]),
            Decision::Dismissed
        );
    }

    #[test]
    fn w1_c6b_a_cleared_human_row_is_dismissed() {
        assert_eq!(
            decision(Some(&row(Human, Some(Cleared))), &[]),
            Decision::Dismissed
        );
    }

    #[test]
    fn w1_c6c_an_invalid_live_row_is_empty() {
        let mut invalid = row(Default, None);
        invalid.valid = false;
        assert_eq!(decision(Some(&invalid), &[]), Decision::Empty);
    }

    #[test]
    fn w1_c6d_a_suggestion_is_unchecked_with_its_confidence() {
        let mut low = row(Ai, Some(Suggested));
        low.ai_confidence = Some(AiConfidence::Low);
        assert_eq!(
            decision(Some(&low), &[]),
            Decision::Unchecked(Some(AiConfidence::Low))
        );
    }

    #[test]
    fn w1_c6e_an_accepted_suggestion_is_accepted() {
        assert_eq!(
            decision(Some(&row(Ai, Some(Accepted))), &[]),
            Decision::Accepted
        );
    }

    #[test]
    fn w1_c6f_a_human_row_whose_latest_feedback_is_an_edit_is_corrected() {
        let rows = [
            feedback(FeedbackAction::KeptPrevious, 1),
            feedback(FeedbackAction::Edited, 2),
        ];
        assert_eq!(
            decision(Some(&row(Human, None)), &rows),
            Decision::Corrected
        );
        // The latest by time decides, not the order given.
        let reversed = [rows[1].clone(), rows[0].clone()];
        assert_eq!(
            decision(Some(&row(Human, None)), &reversed),
            Decision::Corrected
        );
    }

    #[test]
    fn w1_c6g_a_human_row_without_feedback_is_typed() {
        assert_eq!(decision(Some(&row(Human, None)), &[]), Decision::Typed);
    }

    #[test]
    fn w1_c6h_a_default_row_kept_previous_is_kept_previous() {
        assert_eq!(
            decision(
                Some(&row(Default, None)),
                &[feedback(FeedbackAction::KeptPrevious, 1)]
            ),
            Decision::KeptPrevious
        );
    }

    #[test]
    fn w1_c6i_a_default_row_without_feedback_is_a_shop_default() {
        assert_eq!(
            decision(Some(&row(Default, None)), &[]),
            Decision::ShopDefault
        );
    }

    #[test]
    fn w1_c6j_an_integration_row_read_again_after_a_clear_is_imported() {
        assert_eq!(
            decision(
                Some(&row(Integration, None)),
                &[feedback(FeedbackAction::Cleared, 1)]
            ),
            Decision::Imported
        );
    }

    #[test]
    fn w1_c6k_no_row_is_empty() {
        assert_eq!(decision(None, &[]), Decision::Empty);
    }

    #[test]
    fn w1_c7a_an_accepted_suggestion_is_reviewed() {
        assert!(reviewed(
            Some(&row(Ai, Some(Accepted))),
            &[feedback(FeedbackAction::Accepted, 1)]
        ));
    }

    #[test]
    fn w1_c7b_a_later_suggestion_after_a_clear_is_not_reviewed() {
        assert!(!reviewed(
            Some(&row(Ai, Some(Suggested))),
            &[feedback(FeedbackAction::Cleared, 1)]
        ));
    }

    #[test]
    fn w1_c7c_a_typed_value_without_feedback_is_not_reviewed() {
        assert!(!reviewed(Some(&row(Human, None)), &[]));
    }

    #[test]
    fn w1_c7d_a_cleared_human_row_is_reviewed() {
        assert!(reviewed(Some(&row(Human, Some(Cleared))), &[]));
    }

    fn input() -> StaleInput {
        StaleInput {
            main_same: Some(true),
            condition_same: Some(true),
            usable_main: true,
            usable_condition: true,
            condition_chosen: true,
        }
    }

    /// The rules of WC1.7 behind rows W1·C10(a)–(e).
    #[test]
    fn w1_c10_stale_rules() {
        let none = Staleness::default();
        assert_eq!(stale(&input()), none);
        let changed = StaleInput {
            main_same: Some(false),
            ..input()
        };
        assert_eq!(
            stale(&changed),
            Staleness {
                stale_main: true,
                ..none
            }
        );
        let condition = StaleInput {
            condition_same: Some(false),
            ..input()
        };
        assert!(stale(&condition).stale_condition);
        let no_n6 = StaleInput {
            condition_same: None,
            ..input()
        };
        assert!(stale(&no_n6).stale_condition);
        assert!(
            !stale(&StaleInput {
                condition_chosen: false,
                ..no_n6
            })
            .stale_condition
        );
        // W5·U4 (WC1.7 amended): an N6 whose photos changed, with no Condition field chosen.
        assert!(
            !stale(&StaleInput {
                condition_chosen: false,
                ..condition
            })
            .stale_condition
        );
        let no_n1 = StaleInput {
            main_same: None,
            ..input()
        };
        assert_eq!(
            stale(&no_n1),
            Staleness {
                missing_main: true,
                ..none
            }
        );
        assert!(
            !stale(&StaleInput {
                usable_main: false,
                ..no_n1
            })
            .missing_main
        );
    }
}
