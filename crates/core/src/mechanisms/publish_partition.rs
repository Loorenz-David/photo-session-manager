//! MC-A10, MC-A1: which of an item's values a publish freezes, withholds or holds as a conflict.

use serde_json::Value as Json;

use crate::domain::attribute::{AiStatus, Definition, Provenance, Value};
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
