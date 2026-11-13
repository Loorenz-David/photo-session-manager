//! The reported usage → price-table units (08B MC-A8, changelog 11; captures
//! `fixtures/openai/*.json`, RESEARCH-VERIFY-1).

use serde_json::Value as Json;

/// `input_token` (net of cached and cache-write tokens), `cached_input_token`,
/// `cache_write_token`, `output_token` (reasoning included), `web_search_call`
/// (`tool_usage.web_search.num_requests`), in that order, each only when greater than 0. `None`
/// when the reply carries no `usage` at all.
pub fn usage_of(body: &Json) -> Option<Vec<(String, i64)>> {
    let usage = body.get("usage").filter(|u| u.is_object())?;
    let count = |value: &Json| value.as_i64().unwrap_or(0).max(0);
    let input = count(&usage["input_tokens"]);
    let cached = count(&usage["input_tokens_details"]["cached_tokens"]);
    let written = count(&usage["input_tokens_details"]["cache_write_tokens"]);
    let output = count(&usage["output_tokens"]);
    let searches = count(&body["tool_usage"]["web_search"]["num_requests"]);
    let units = [
        (
            "input_token",
            input.saturating_sub(cached).saturating_sub(written).max(0),
        ),
        ("cached_input_token", cached),
        ("cache_write_token", written),
        ("output_token", output),
        ("web_search_call", searches),
    ];
    Some(
        units
            .into_iter()
            .filter(|(_, n)| *n > 0)
            .map(|(unit, n)| (unit.to_owned(), n))
            .collect(),
    )
}
