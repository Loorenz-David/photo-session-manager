//! MC-A10, MC-A1: which of an item's values a publish freezes, withholds or holds as a conflict.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value as Json;

use crate::domain::attribute::{AiConfidence, AiStatus, Definition, Provenance, Value};
use crate::mechanisms::sameness::same;

fn parsed(text: &str) -> Json {
    serde_json::from_str(text).unwrap_or_else(|_| Json::String(text.to_owned()))
}

/// MC-A1 conflict: `ai` + `suggested` (any confidence), with an `ai_prior`, and the value not the
/// same as the prior's under `S`.
pub fn is_conflict(definition: &Definition, row: &Value) -> bool {
    row.provenance == Provenance::Ai
        && row.ai_status == Some(AiStatus::Suggested)
        && row
            .ai_prior
            .as_ref()
            .is_some_and(|prior| !same(definition, &parsed(&row.value), &parsed(&prior.value)))
}

/// How a confirmation splits an item's rows (MC-A10).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Partition {
    /// Every live, valid row except the withheld and pending ones: key → canonical value.
    pub frozen: BTreeMap<String, Json>,
    /// `ai` + `suggested` + `low` with no prior: left out for good.
    pub withheld: Vec<String>,
    /// Conflicts whose key is in the item's current layout: left out until resolved.
    pub pending: BTreeMap<String, Value>,
    /// Per frozen, withheld or pending key: `{class, provenance, status, confidence, prior}`.
    pub origin: BTreeMap<String, Json>,
}

fn origin(class: &str, row: &Value) -> Json {
    serde_json::json!({
        "class": class,
        "provenance": row.provenance,
        "status": row.ai_status,
        "confidence": row.ai_confidence,
        "prior": row.ai_prior,
    })
}

/// MC-A10 at confirmation. `values` are the item's stored rows, `definitions` the integration's
/// active definitions, `layout` every key of the item's current layout (all three steps).
/// - withheld: live `ai` + `suggested` + `low` with no `ai_prior`;
/// - pending: a conflict (MC-A1) whose key is in the layout and has a definition;
/// - left out entirely: a conflict whose key is not in the layout, and any `ai` + `suggested`
///   row with an `ai_prior` whose key has no definition (no type for `S`: fail closed);
/// - frozen: every other live, valid row (a prior that is not a conflict freezes at any
///   confidence; a row with no definition is frozen as before).
pub fn partition(
    values: &[Value],
    definitions: &[Definition],
    layout: &BTreeSet<String>,
) -> Partition {
    let mut out = Partition::default();
    for row in values.iter().filter(|v| v.is_live()) {
        let suggested =
            row.provenance == Provenance::Ai && row.ai_status == Some(AiStatus::Suggested);
        let definition = definitions.iter().find(|d| d.key == row.key);
        if suggested && row.ai_prior.is_some() {
            let Some(definition) = definition else {
                continue;
            };
            if is_conflict(definition, row) {
                if layout.contains(&row.key) {
                    out.origin.insert(row.key.clone(), origin("pending", row));
                    out.pending.insert(row.key.clone(), row.clone());
                }
                continue;
            }
        } else if suggested && row.ai_confidence == Some(AiConfidence::Low) {
            out.origin.insert(row.key.clone(), origin("withheld", row));
            out.withheld.push(row.key.clone());
            continue;
        }
        if row.valid
            && let Some(json) = row.json()
        {
            out.origin.insert(row.key.clone(), origin("frozen", row));
            out.frozen.insert(row.key.clone(), json);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::attribute::{AiPrior, AttributeGroup, CoreType};

    fn def(key: &str) -> Definition {
        Definition {
            integration: "shop".into(),
            key: key.into(),
            label: key.into(),
            core_type: CoreType::Text,
            choices: vec![],
            validations: serde_json::json!({}),
            group: AttributeGroup::Details,
            description: None,
            writable: true,
        }
    }
    fn row(key: &str, value: &str, provenance: Provenance) -> Value {
        Value {
            item_id: 1,
            key: key.into(),
            value: Json::String(value.into()).to_string(),
            valid: true,
            provenance,
            ai_confidence: None,
            ai_status: None,
            ai_reason: None,
            ai_sources: None,
            ai_prior: None,
            updated_at: 0,
        }
    }
    fn ai(key: &str, value: &str, confidence: AiConfidence, prior: Option<&str>) -> Value {
        Value {
            ai_confidence: Some(confidence),
            ai_status: Some(AiStatus::Suggested),
            ai_prior: prior.map(|p| AiPrior {
                value: Json::String(p.into()).to_string(),
                valid: true,
                provenance: Provenance::Default,
            }),
            ..row(key, value, Provenance::Ai)
        }
    }
    fn keys(list: &[&str]) -> BTreeSet<String> {
        list.iter().map(|k| k.to_string()).collect()
    }

    #[test]
    fn is_conflict_needs_a_differing_prior_on_a_suggestion() {
        let d = def("k");
        assert!(is_conflict(
            &d,
            &ai("k", "Walnut", AiConfidence::High, Some("Oak"))
        ));
        assert!(is_conflict(
            &d,
            &ai("k", "Walnut", AiConfidence::Low, Some("Oak"))
        ));
        assert!(!is_conflict(
            &d,
            &ai("k", "oak ", AiConfidence::Low, Some("Oak"))
        ));
        assert!(!is_conflict(
            &d,
            &ai("k", "Walnut", AiConfidence::High, None)
        ));
        let accepted = Value {
            ai_status: Some(AiStatus::Accepted),
            ..ai("k", "Walnut", AiConfidence::High, Some("Oak"))
        };
        assert!(!is_conflict(&d, &accepted));
    }

    #[test]
    fn partition_splits_frozen_withheld_pending() {
        let defs: Vec<_> = ["a", "b", "c", "d", "e", "f", "g"].map(def).to_vec();
        let invalid = Value {
            valid: false,
            ..row("f", "x", Provenance::Human)
        };
        let cleared = Value {
            ai_status: Some(AiStatus::Cleared),
            ..row("g", "y", Provenance::Ai)
        };
        let values = vec![
            row("a", "H", Provenance::Human),
            ai("b", "M", AiConfidence::Medium, None),
            ai("c", "L", AiConfidence::Low, None),
            ai("d", "C", AiConfidence::Low, Some("D")),
            ai("e", "same", AiConfidence::Low, Some("Same")),
            invalid,
            cleared,
        ];
        let p = partition(&values, &defs, &keys(&["a", "b", "c", "d", "e", "f", "g"]));
        assert_eq!(
            p.frozen.keys().cloned().collect::<Vec<_>>(),
            vec!["a", "b", "e"]
        );
        assert_eq!(p.withheld, vec!["c"]);
        assert_eq!(p.pending.keys().cloned().collect::<Vec<_>>(), vec!["d"]);
        assert_eq!(
            p.origin.keys().cloned().collect::<Vec<_>>(),
            vec!["a", "b", "c", "d", "e"]
        );
        assert_eq!(p.origin["d"]["class"], "pending");
        assert_eq!(p.origin["d"]["prior"]["value"], "\"D\"");
        assert_eq!(p.origin["c"]["confidence"], "low");
    }

    #[test]
    fn partition_leaves_out_conflicts_outside_layout_or_without_definition() {
        let defs = vec![def("in"), def("out")];
        let values = vec![
            ai("in", "X", AiConfidence::High, Some("D")),
            ai("out", "X", AiConfidence::High, Some("D")),
            ai("gone", "X", AiConfidence::High, Some("D")),
            ai("gone_low", "X", AiConfidence::Low, None),
            row("plain", "P", Provenance::Default),
        ];
        let p = partition(&values, &defs, &keys(&["in"]));
        assert_eq!(p.pending.keys().cloned().collect::<Vec<_>>(), vec!["in"]);
        assert_eq!(p.withheld, vec!["gone_low"]);
        assert_eq!(p.frozen.keys().cloned().collect::<Vec<_>>(), vec!["plain"]);
        assert!(!p.origin.contains_key("out") && !p.origin.contains_key("gone"));
    }
}
