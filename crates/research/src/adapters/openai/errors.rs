//! Failures of the Responses API → `CallFailure` (08B MC-A9; captures
//! `fixtures/openai/error_*.json`, RESEARCH-VERIFY-2). `quota` is never set: no out-of-credit
//! reply was captured (structurally held, R8 carried note 1).

use serde_json::Value as Json;

use crate::{
    adapters::http::HttpReply,
    domain::{CallFailure, FailureKind},
};

/// An HTTP error status: the provider's `error.message`, else the body text, else the status.
pub fn status_failure(reply: &HttpReply) -> CallFailure {
    let body = reply.body_json();
    let text = body["error"]["message"]
        .as_str()
        .map(str::to_owned)
        .or_else(|| body.as_str().map(str::to_owned))
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| format!("HTTP {}", reply.status));
    CallFailure {
        sent: true,
        status: Some(reply.status),
        kind: FailureKind::Status,
        quota: false,
        text,
        usage: None,
    }
}

/// A 200 reply that is not a usable answer (`Schema`) or a refusal (`Refusal`).
pub fn reply_failure(
    kind: FailureKind,
    text: impl Into<String>,
    usage: Option<Vec<(String, i64)>>,
) -> CallFailure {
    CallFailure {
        sent: true,
        status: Some(200),
        kind,
        quota: false,
        text: text.into(),
        usage,
    }
}

/// The refusal text of a reply, if the model refused (documented shape, evidence V3).
pub fn refusal(body: &Json) -> Option<String> {
    body["output"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|item| item["content"].as_array().into_iter().flatten())
        .find(|content| content["type"] == "refusal")
        .map(|content| content["refusal"].as_str().unwrap_or("").to_owned())
}
