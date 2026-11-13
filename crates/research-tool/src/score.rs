//! Scoring of a model's answer against the answer key (R1·U2; MC-A2 mapping). An answer that
//! maps to no choice is *not determined*, as MC-A2 says for a failed mapping.

use crate::answer_key::{Bucket, KeyEntry};
use psm_core::mechanisms::name_key::name_key;
use serde_json::Value as Json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Outcome {
    Correct,
    Wrong,
    NotDetermined,
}

/// The core type of a scored key.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyKind {
    Choice(Vec<String>),
    MultiChoice(Vec<String>),
    Text,
}

/// MC-A2 choice mapping: the byte-equal choice, else the unique choice with the same name key.
pub fn map_choice(choices: &[String], answer: &str) -> Option<String> {
    if let Some(c) = choices.iter().find(|c| c.as_str() == answer) {
        return Some(c.clone());
    }
    let key = name_key(answer);
    let matches: Vec<&String> = choices.iter().filter(|c| name_key(c) == key).collect();
    match matches.as_slice() {
        [one] => Some((*one).clone()),
        _ => None,
    }
}

fn strings(value: &Json) -> Vec<String> {
    match value {
        Json::String(s) => vec![s.clone()],
        Json::Array(items) => items
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

/// Scores `answer` (`None` or JSON null / empty = the model determined nothing) against `key`.
pub fn score(kind: &KeyKind, key: &Json, answer: Option<&Json>) -> Outcome {
    let answer = match answer {
        None | Some(Json::Null) => return Outcome::NotDetermined,
        Some(a) => a,
    };
    match kind {
        KeyKind::Choice(choices) => {
            let Some(a) = answer.as_str().filter(|s| !s.trim().is_empty()) else {
                return Outcome::NotDetermined;
            };
            let Some(mapped) = map_choice(choices, a) else {
                return Outcome::NotDetermined;
            };
            let expected = key
                .as_str()
                .and_then(|k| map_choice(choices, k))
                .unwrap_or_default();
            if mapped == expected {
                Outcome::Correct
            } else {
                Outcome::Wrong
            }
        }
        KeyKind::MultiChoice(choices) => {
            let mut mapped: Vec<String> = strings(answer)
                .iter()
                .filter_map(|a| map_choice(choices, a))
                .collect();
            if mapped.is_empty() {
                return Outcome::NotDetermined;
            }
            let mut expected: Vec<String> = strings(key)
                .iter()
                .filter_map(|k| map_choice(choices, k))
                .collect();
            mapped.sort();
            mapped.dedup();
            expected.sort();
            expected.dedup();
            if mapped == expected {
                Outcome::Correct
            } else {
                Outcome::Wrong
            }
        }
        KeyKind::Text => {
            let Some(a) = answer.as_str().filter(|s| !s.trim().is_empty()) else {
                return Outcome::NotDetermined;
            };
            let k = key.as_str().unwrap_or("");
            if name_key(a) == name_key(k) {
                Outcome::Correct
            } else {
                Outcome::Wrong
            }
        }
    }
}

/// Scores one field of one piece in the bucket the answer key puts it in (a corrected field
/// against its corrected value, in the corrected bucket only). `None` when the key has no value.
pub fn score_field(
    entries: &[KeyEntry],
    n: u32,
    field: &str,
    kind: &KeyKind,
    answer: Option<&Json>,
) -> Option<(Bucket, Outcome)> {
    let entry = entries.iter().find(|e| e.n == n && e.field == field)?;
    Some((entry.bucket, score(kind, &entry.value, answer)))
}

/// Read stage: a page that does not name the key states nothing, so *none* is correct and any
/// name is wrong; a page that names the key is scored as a text answer.
pub fn score_read(page_names_key: bool, key: &str, answer: Option<&str>) -> Outcome {
    let answer = answer.filter(|a| !a.trim().is_empty());
    if !page_names_key {
        return match answer {
            None => Outcome::Correct,
            Some(_) => Outcome::Wrong,
        };
    }
    match answer {
        None => Outcome::NotDetermined,
        Some(a) if name_key(a) == name_key(key) => Outcome::Correct,
        Some(_) => Outcome::Wrong,
    }
}

// ---------------------------------------------------------------------------------------------
// Acceptance runs (R9·U2 task 2): each run's written values against the answer key.
// ---------------------------------------------------------------------------------------------

/// The confidences at which a hedged attribution counts as correct (`answer_key_corrections.json`
/// `scoring_rules.hedged_attribution`: medium or low; high is wrong).
pub const HEDGED_CORRECT_AT: [&str; 2] = ["medium", "low"];

/// What a run wrote for a field: the value and its confidence (`writes.json`).
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    pub value: Json,
    pub confidence: String,
    pub sources: Vec<String>,
}

/// An attribution against the key. A key value: the same name (MC-A2 name key) is correct, but
/// for a hedged piece only at medium or low; another name is wrong; nothing written is not
/// determined. No key value: any suggestion is wrong, nothing written is correct.
pub fn score_attribution(entry: Option<&KeyEntry>, suggestion: Option<&Suggestion>) -> Outcome {
    match (entry, suggestion) {
        (None, None) => Outcome::Correct,
        (None, Some(_)) => Outcome::Wrong,
        (Some(_), None) => Outcome::NotDetermined,
        (Some(entry), Some(s)) => match score(&KeyKind::Text, &entry.value, Some(&s.value)) {
            Outcome::Correct
                if entry.hedged && !HEDGED_CORRECT_AT.contains(&s.confidence.as_str()) =>
            {
                Outcome::Wrong
            }
            other => other,
        },
    }
}

/// The class of a scored field in the tables of 08A §11.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldGroup {
    Attribution,
    Visible,
    Condition,
}

/// One scored field of one run.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FieldScore {
    pub n: u32,
    pub key: String,
    pub group: FieldGroup,
    /// `None`: an attribution the key leaves empty.
    pub corrected: Option<bool>,
    pub hedged: bool,
    pub expected: Json,
    pub suggested: Json,
    pub confidence: Option<String>,
    pub outcome: Outcome,
    /// A disagreement with the key: listed for a human look with its evidence.
    pub review_candidate: bool,
    pub sources: Vec<String>,
    /// The run's steps that name the field (`stage: summary` and the source, if any).
    pub evidence: Vec<String>,
}

impl serde::Serialize for Outcome {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(match self {
            Outcome::Correct => "correct",
            Outcome::Wrong => "wrong",
            Outcome::NotDetermined => "not_determined",
        })
    }
}

/// The key's field name of a definition key (`custom.designer` → `designer`).
pub fn field_of(key: &str) -> &str {
    key.split_once('.').map_or(key, |(_, field)| field)
}

fn kind_of(definition: &psm_core::domain::attribute::Definition) -> KeyKind {
    use psm_core::domain::attribute::CoreType;
    match definition.core_type {
        CoreType::Choice => KeyKind::Choice(definition.choices.clone()),
        CoreType::MultiChoice => KeyKind::MultiChoice(definition.choices.clone()),
        _ => KeyKind::Text,
    }
}

/// What a run's `writes.json` suggests for `key` (an entry that was written, kept or lowered).
pub fn suggestion_of(writes: &Json, key: &str) -> Option<Suggestion> {
    let write = writes["writes"].get(key)?;
    let outcome = write["outcome"].as_str().unwrap_or_default();
    if !matches!(outcome, "written" | "unchanged" | "lowered") {
        return None;
    }
    Some(Suggestion {
        value: write["value"].clone(),
        confidence: write["confidence"].as_str().unwrap_or_default().to_owned(),
        sources: write["sources"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|s| s.as_str().map(str::to_owned))
            .collect(),
    })
}

/// Scores one run of piece `n`: every attribution key, and every other definition the key holds
/// a value for (fields the key does not hold are not scored).
pub fn score_run(
    n: u32,
    writes: &Json,
    steps: &[Json],
    entries: &[KeyEntry],
    definitions: &[psm_core::domain::attribute::Definition],
    attribution_keys: &[String],
) -> Vec<FieldScore> {
    use psm_core::domain::attribute::AttributeGroup;
    let mut out = Vec::new();
    for definition in definitions {
        let key = definition.key.as_str();
        let entry = entries
            .iter()
            .find(|e| e.n == n && e.field == field_of(key));
        let attribution = attribution_keys.iter().any(|k| k == key);
        if !attribution && entry.is_none() {
            continue;
        }
        let suggestion = suggestion_of(writes, key);
        let (group, outcome) = if attribution {
            (
                FieldGroup::Attribution,
                score_attribution(entry, suggestion.as_ref()),
            )
        } else {
            let group = if definition.group == AttributeGroup::Condition {
                FieldGroup::Condition
            } else {
                FieldGroup::Visible
            };
            let expected = entry.map(|e| e.value.clone()).unwrap_or(Json::Null);
            (
                group,
                score(
                    &kind_of(definition),
                    &expected,
                    suggestion.as_ref().map(|s| &s.value),
                ),
            )
        };
        let evidence = steps
            .iter()
            .filter(|s| {
                s["field_keys"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|k| k.as_str() == Some(key))
            })
            .map(|s| {
                let mut line = format!(
                    "{}: {}",
                    s["stage"].as_str().unwrap_or_default(),
                    s["summary"].as_str().unwrap_or_default()
                );
                if let Some(url) = s["source_url"].as_str() {
                    line.push_str(&format!(" ({url})"));
                }
                line
            })
            .collect();
        out.push(FieldScore {
            n,
            key: key.to_owned(),
            group,
            corrected: entry.map(|e| e.bucket == Bucket::Corrected),
            hedged: entry.is_some_and(|e| e.hedged),
            expected: entry.map(|e| e.value.clone()).unwrap_or(Json::Null),
            suggested: suggestion
                .as_ref()
                .map(|s| s.value.clone())
                .unwrap_or(Json::Null),
            confidence: suggestion.as_ref().map(|s| s.confidence.clone()),
            outcome,
            review_candidate: outcome == Outcome::Wrong,
            sources: suggestion.map(|s| s.sources).unwrap_or_default(),
            evidence,
        });
    }
    out
}
