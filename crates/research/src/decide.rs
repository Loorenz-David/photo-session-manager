//! Stage 7, Decide (08A §5, MC-A4, MC-A1, MC-A7). For every layout field: its class, the
//! evidence from the run's committed outputs, the decision of the confidence rules; then one model
//! call writes the reasons; then one transaction applies the writes (MC-A1 through `writes`) and
//! commits the `concluded` and `not_determined` steps and the stage 7 output together.

use std::collections::BTreeMap;

use psm_core::{
    Result,
    domain::attribute::{AiConfidence, AiStatus, AttributeGroup, CoreType, Definition, Provenance},
    ports::store::StoreTx,
};
use serde_json::{Map, Value as Json, json};

use crate::{
    domain::{FieldClass, LabelItem, ListingItem, PhotoRead, RunState, Stage, StepKind, Verdict},
    gateway::Answer,
    rules::{
        confidence::{
            Decision, decide_attribution_without_shop, decide_condition, decide_measured,
            decide_visible,
        },
        derived_condition::{LevelValue, derive},
        fingerprint::fingerprint,
        sites::Site,
        value_map::Stated,
    },
    stages::{
        Asked, Committed, Ctx, DECIDE, ExecuteOutcome, Work, class_of, display, stated_from, step,
    },
    store::{self, NewStep, OutputRow},
    writes::{self, WriteOutcome},
};

const UNIT: &str = "decide";

/// The committed stage 7 output of the run, if any.
pub(crate) fn committed(ctx: &Ctx<'_>) -> Result<Option<Json>> {
    let row = ctx.svc.store.research_transaction(|tx| {
        store::output(tx.conn(), ctx.run.id, Stage::Decide.as_str(), UNIT)
    })?;
    Ok(row.map(|r| serde_json::from_str(&r.output).unwrap_or(Json::Null)))
}

/// Ends the run after its stage 7 commit: `finished`, or `stopped_at_cap` when reading stopped.
pub(crate) fn finish(ctx: &Ctx<'_>, output: Json) -> Result<ExecuteOutcome> {
    let (state, outcome) = if output["ending"].as_str() == Some(RunState::StoppedAtCap.as_str()) {
        (RunState::StoppedAtCap, ExecuteOutcome::StoppedAtCap)
    } else {
        (RunState::Finished, ExecuteOutcome::Finished)
    };
    end(ctx, state)?;
    Ok(outcome)
}

pub(crate) fn end(ctx: &Ctx<'_>, state: RunState) -> Result<()> {
    let now = ctx.svc.clock.now_ms();
    ctx.svc.store.research_transaction(|tx| {
        store::set_run_state(tx.conn(), ctx.run.id, state, None, Some(now))
    })?;
    ctx.svc.changed(ctx.run.session_id);
    Ok(())
}

/// One field decided to a value.
struct Decided {
    definition: Definition,
    value: Json,
    confidence: AiConfidence,
    sources: Vec<String>,
    named: Vec<Stated>,
}

fn named_text(named: &[Stated]) -> String {
    named
        .iter()
        .map(|s| match s {
            Stated::Mapped(v) => display(v),
            Stated::Unmapped(k) => k.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The evidence of the run, by stage.
struct Evidence<'a> {
    /// The one stage 1 output (R12·U1): one read per visible field over the chosen photos.
    observe: Option<&'a Json>,
    pages: Vec<&'a Json>,
    verdicts: BTreeMap<String, Verdict>,
    labels: Vec<&'a Json>,
    condition: Option<&'a Json>,
}

impl<'a> Evidence<'a> {
    fn of(used: &'a [Committed]) -> Self {
        let of = |stage: Stage| -> Vec<&'a Json> {
            used.iter()
                .filter(|c| c.stage == stage)
                .map(|c| &c.output)
                .collect()
        };
        let verdicts = of(Stage::Compare)
            .into_iter()
            .filter_map(|c| {
                let verdict: Verdict = serde_json::from_value(c["verdict"].clone()).ok()?;
                Some((c["unit"].as_str()?.to_owned(), verdict))
            })
            .collect();
        Self {
            observe: of(Stage::Observe).into_iter().next(),
            pages: of(Stage::Read)
                .into_iter()
                .filter(|p| p["obtained"].as_bool() == Some(true))
                .collect(),
            verdicts,
            labels: of(Stage::Labels),
            condition: of(Stage::Condition).into_iter().next(),
        }
    }

    fn listings(&self, key: &str) -> Vec<ListingItem> {
        self.pages
            .iter()
            .filter_map(|page| {
                let stated = stated_from(page["statements"].get(key)?)?;
                let site: Site = Site {
                    class: serde_json::from_value(page["site"]["class"].clone()).ok()?,
                    identity: page["site"]["identity"].as_str()?.to_owned(),
                };
                Some(ListingItem {
                    page: page["page"].as_str()?.to_owned(),
                    site,
                    verdict: page["unit"]
                        .as_str()
                        .and_then(|u| self.verdicts.get(u).copied()),
                    stated,
                })
            })
            .collect()
    }

    fn labels(&self, key: &str) -> Vec<LabelItem> {
        self.labels
            .iter()
            .filter_map(|l| {
                Some(LabelItem {
                    photo_id: l["photo_id"].as_i64()?,
                    stated: stated_from(l["statements"].get(key)?)?,
                })
            })
            .collect()
    }

    /// The photo read of a visible field: the Observe unit's read for it, else a mapped label
    /// statement.
    fn photo_read(&self, key: &str) -> Option<PhotoRead> {
        self.observe
            .and_then(|o| {
                let read = o["reads"].get(key)?;
                Some(PhotoRead {
                    value: read.get("value")?.clone(),
                    sure: read["sure"].as_bool().unwrap_or(false),
                })
            })
            .or_else(|| {
                self.labels(key).into_iter().find_map(|l| match l.stated {
                    Stated::Mapped(value) => Some(PhotoRead { value, sure: true }),
                    Stated::Unmapped(_) => None,
                })
            })
    }

    fn condition_read(&self, key: &str) -> Option<PhotoRead> {
        let read = self.condition?["reads"].get(key)?;
        Some(PhotoRead {
            value: read.get("value")?.clone(),
            sure: read["sure"].as_bool().unwrap_or(false),
        })
    }
}

/// Stage 7 for the run, then its end.
pub(crate) fn decide(ctx: &Ctx<'_>, used: &[Committed], stopped: bool) -> Result<ExecuteOutcome> {
    let evidence = Evidence::of(used);
    let mut decided: Vec<Decided> = Vec::new();
    let mut not_determined: Vec<(Definition, String)> = Vec::new();
    let mut derived: Vec<Definition> = Vec::new();
    for field in &ctx.form.fields {
        let definition = &field.definition;
        let key = definition.key.as_str();
        let class = class_of(ctx, definition, field.step);
        let decision: Decision = match class {
            FieldClass::NeverWritten => {
                not_determined.push((definition.clone(), "not researched".to_owned()));
                continue;
            }
            FieldClass::Derived => {
                derived.push(definition.clone());
                continue;
            }
            FieldClass::Condition => decide_condition(
                evidence.condition_read(key).as_ref(),
                evidence.condition.is_some(),
            ),
            FieldClass::Attribution => decide_attribution_without_shop(
                definition,
                &evidence.labels(key),
                &evidence.listings(key),
            ),
            FieldClass::Measured => decide_measured(definition, &evidence.listings(key)),
            FieldClass::Visible => decide_visible(
                definition,
                evidence.photo_read(key).as_ref(),
                &evidence.listings(key),
            ),
        };
        match decision.written {
            Some((value, confidence)) => decided.push(Decided {
                definition: definition.clone(),
                value,
                confidence,
                sources: decision.sources,
                named: decision.named,
            }),
            None => {
                let reason = if !decision.named.is_empty() {
                    format!("the evidence disagrees: {}", named_text(&decision.named))
                } else {
                    match class {
                        FieldClass::Condition if evidence.condition.is_some() => {
                            "the condition photos do not show it".to_owned()
                        }
                        FieldClass::Condition => "no Condition photo".to_owned(),
                        FieldClass::Visible => "not seen in the photos or listings".to_owned(),
                        _ => "no label or listing states it".to_owned(),
                    }
                };
                not_determined.push((definition.clone(), reason));
            }
        }
    }
    // Reasons: one model call for every decided field (none decided: no call).
    let mut reasons: Map<String, Json> = Map::new();
    if !decided.is_empty() {
        let input = json!({
            "fields": decided.iter().map(|d| json!({
                "key": d.definition.key,
                "label": d.definition.label,
                "value": d.value,
                "confidence": d.confidence.as_str(),
                "sources": d.sources,
                "others": named_text(&d.named),
            })).collect::<Vec<_>>(),
        });
        let request = ctx.model_request(Stage::Decide, UNIT, DECIDE, input, Vec::new(), Vec::new());
        let (asked, _) = ctx.ask(Stage::Decide, UNIT, request)?;
        match asked {
            Asked::Answer(Answer::Model(reply)) => {
                if let Some(map) = reply.output["reasons"].as_object() {
                    reasons = map.clone();
                }
            }
            Asked::Answer(_) | Asked::NotDetermined(_) => {}
            Asked::Work(Work::Skip) => {
                end(ctx, RunState::StoppedAtCap)?;
                return Ok(ExecuteOutcome::StoppedAtCap);
            }
            Asked::Work(Work::Halt(outcome)) => return Ok(outcome),
            Asked::Work(Work::Commit { .. }) => {}
        }
    } else {
        let cost = ctx
            .svc
            .store
            .research_transaction(|tx| store::run(tx.conn(), ctx.run.id))?
            .map(|r| r.cost_micro_usd)
            .unwrap_or(0);
        if cost >= ctx.config.max_cost_per_run && derived.is_empty() {
            end(ctx, RunState::StoppedAtCap)?;
            return Ok(ExecuteOutcome::StoppedAtCap);
        }
    }
    let ending = if stopped {
        RunState::StoppedAtCap
    } else {
        RunState::Finished
    };
    let mut fingerprints: Vec<&str> = used.iter().map(|c| c.fingerprint.as_str()).collect();
    fingerprints.sort();
    let identity = json!({
        "units": fingerprints,
        "layout": ctx.form.fields.iter().map(|f| json!({
            "key": f.definition.key,
            "core_type": f.definition.core_type,
            "choices": f.definition.choices,
            "step": f.step,
        })).collect::<Vec<_>>(),
        "attribution_keys": ctx.config.attribution_keys,
        "fill_keys": ctx.config.fill_keys,
        "detailed_condition": ctx.config.detailed_condition,
    });
    let unit_fingerprint = fingerprint(Stage::Decide.number(), &identity)
        .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
    let now = ctx.svc.clock.now_ms();
    let integration = ctx.svc.integration.clone();
    let item_id = ctx.run.item_id;
    ctx.svc.store.research_transaction(|tx| {
        let mut writes_out: Map<String, Json> = Map::new();
        let mut steps: Vec<NewStep> = Vec::new();
        let mut not_written: Vec<(Definition, String)> = Vec::new();
        for d in &decided {
            let mut reason = reasons
                .get(&d.definition.key)
                .and_then(Json::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    format!(
                        "Decided at {} confidence from the recorded evidence.",
                        d.confidence.as_str()
                    )
                });
            if !d.named.is_empty() {
                reason = format!("{reason} Other values seen: {}.", named_text(&d.named));
            }
            let outcome = writes::suggest_in(
                tx,
                now,
                integration.as_deref(),
                item_id,
                &d.definition.key,
                &d.value,
                d.confidence,
                Some(&reason),
                &d.sources,
            )?;
            writes_out.insert(
                d.definition.key.clone(),
                json!({
                    "value": d.value,
                    "confidence": d.confidence.as_str(),
                    "reason": reason,
                    "sources": d.sources,
                    "outcome": outcome.text(),
                }),
            );
            match outcome {
                WriteOutcome::Written(_) | WriteOutcome::Unchanged => {
                    let mut concluded = step(
                        Stage::Decide,
                        StepKind::Concluded,
                        format!(
                            "{}: {} ({})",
                            d.definition.label,
                            display(&d.value),
                            d.confidence.as_str()
                        ),
                    );
                    concluded.detail = reason.clone();
                    concluded.field_keys = vec![d.definition.key.clone()];
                    steps.push(concluded);
                }
                other => not_written.push((d.definition.clone(), other.text())),
            }
        }
        // The level rows here include condition suggestions written just above, in this same
        // transaction. A person-filled level counts even when the field is not chosen.
        let level_fields: Vec<_> = ctx
            .form
            .fields
            .iter()
            .filter(|f| {
                f.step == AttributeGroup::Condition
                    && f.definition.group == AttributeGroup::Condition
                    && f.definition.core_type == CoreType::Choice
            })
            .collect();
        let current = tx.attribute_values(item_id)?;
        for definition in &derived {
            let values: Vec<LevelValue> = level_fields
                .iter()
                .map(|field| {
                    let row = current.iter().find(|v| {
                        v.key == field.definition.key && v.ai_status != Some(AiStatus::Cleared)
                    });
                    LevelValue {
                        label: field.definition.label.clone(),
                        value: row.and_then(|v| serde_json::from_str::<String>(&v.value).ok()),
                        ai_low: row.is_some_and(|v| {
                            v.provenance == Provenance::Ai
                                && v.ai_confidence == Some(AiConfidence::Low)
                        }),
                    }
                })
                .collect();
            let grade = derive(&ctx.config.detailed_condition.levels, &values, definition);
            match (grade.value, grade.confidence) {
                (Some(value), Some(confidence)) => {
                    let outcome = writes::suggest_in(
                        tx,
                        now,
                        integration.as_deref(),
                        item_id,
                        &definition.key,
                        &value,
                        confidence,
                        Some(&grade.reason),
                        &[],
                    )?;
                    writes_out.insert(definition.key.clone(), json!({
                        "value": value, "confidence": confidence.as_str(), "reason": grade.reason,
                        "sources": [], "outcome": outcome.text(),
                    }));
                    match outcome {
                        WriteOutcome::Written(_) | WriteOutcome::Unchanged => {
                            let mut concluded = step(
                                Stage::Decide,
                                StepKind::Concluded,
                                format!(
                                    "{}: {} ({})",
                                    definition.label,
                                    display(&value),
                                    confidence.as_str()
                                ),
                            );
                            concluded.detail = grade.reason;
                            concluded.field_keys = vec![definition.key.clone()];
                            steps.push(concluded);
                        }
                        other => not_written.push((definition.clone(), other.text())),
                    }
                }
                _ => not_written.push((definition.clone(), grade.reason)),
            }
        }
        let mut nd_map: Map<String, Json> = Map::new();
        let mut nd_lines: Vec<String> = Vec::new();
        let mut nd_keys: Vec<String> = Vec::new();
        for (definition, reason) in not_determined.iter().chain(not_written.iter()) {
            // 08B changelog 7 (owner 2026-11-12): a run that stopped at its cap read less than
            // a complete run, so its silence is no evidence against an earlier suggestion.
            let lowered = if reason != "not researched" && ending != RunState::StoppedAtCap {
                writes::lower_in(tx, now, item_id, &definition.key, reason)?
            } else {
                WriteOutcome::Unchanged
            };
            if let WriteOutcome::Written(row) = &lowered {
                let mut concluded = step(
                    Stage::Decide,
                    StepKind::Concluded,
                    format!("{}: no longer supported", definition.label),
                );
                concluded.detail = row.ai_reason.clone().unwrap_or_default();
                concluded.field_keys = vec![definition.key.clone()];
                steps.push(concluded);
                writes_out.insert(
                    definition.key.clone(),
                    json!({
                        "value": serde_json::from_str::<Json>(&row.value).unwrap_or(Json::Null),
                        "confidence": AiConfidence::Low.as_str(),
                        "reason": row.ai_reason,
                        "sources": row.ai_sources,
                        "outcome": "lowered",
                    }),
                );
                continue;
            }
            nd_map.insert(definition.key.clone(), json!(reason));
            nd_lines.push(format!("{}: {reason}", definition.label));
            nd_keys.push(definition.key.clone());
        }
        let mut nd_step = step(Stage::Decide, StepKind::NotDetermined, "Not determined");
        nd_step.detail = nd_lines.join("\n");
        nd_step.field_keys = nd_keys;
        steps.push(nd_step);
        let output = json!({
            "writes": writes_out,
            "not_determined": nd_map,
            "ending": ending.as_str(),
        });
        let conn = tx.conn();
        store::insert_output(
            conn,
            &OutputRow {
                run_id: ctx.run.id,
                stage: Stage::Decide.as_str().to_owned(),
                unit_key: UNIT.to_owned(),
                fingerprint: unit_fingerprint.clone(),
                output: output.to_string(),
                reused_from_run_id: None,
                committed_at: now,
            },
        )?;
        for s in &steps {
            store::insert_step(conn, ctx.run.id, s, now)?;
        }
        store::set_stage_reached(conn, ctx.run.id, Stage::Decide.as_str())
    })?;
    ctx.svc.changed(ctx.run.session_id);
    let output = committed(ctx)?.unwrap_or(Json::Null);
    finish(ctx, output)
}
