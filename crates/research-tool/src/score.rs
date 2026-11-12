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
