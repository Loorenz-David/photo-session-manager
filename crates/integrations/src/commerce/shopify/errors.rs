//! Mapping of Shopify replies to core errors. No message built here ever contains a token, the
//! client secret, or a URL (09A MC-23, M30).

use psm_core::Error;
use serde_json::Value as Json;

/// 09A MC-23: a refused client-credentials grant. The reply body is
/// `{"error": <code>, "error_description": <text>}` (SV-0).
pub fn grant_refusal(body: &Json) -> Error {
    match body.get("error").and_then(Json::as_str) {
        Some("shop_not_permitted") => Error::ShopNotPermitted,
        Some("app_not_installed") => Error::AppNotInstalled,
        _ => Error::CredentialsRejected,
    }
}

/// Top-level GraphQL `errors` (a list of `{message, extensions.code}`, or a plain string on 401).
pub fn graphql_errors(body: &Json) -> Option<String> {
    match body.get("errors")? {
        Json::String(s) => Some(s.clone()),
        Json::Array(errors) if !errors.is_empty() => Some(
            errors
                .iter()
                .map(|e| e.get("message").and_then(Json::as_str).unwrap_or("error"))
                .collect::<Vec<_>>()
                .join("; "),
        ),
        _ => None,
    }
}

/// True when the GraphQL reply says the request was throttled (not executed; safe to retry).
pub fn is_throttled(body: &Json) -> bool {
    body.get("errors")
        .and_then(Json::as_array)
        .is_some_and(|errors| {
            errors
                .iter()
                .any(|e| e["extensions"]["code"].as_str() == Some("THROTTLED"))
        })
}

/// `userErrors` of a mutation payload as one message, or `None` when empty.
pub fn user_errors(payload: &Json) -> Option<String> {
    let errors = payload
        .get("userErrors")
        .or_else(|| payload.get("mediaUserErrors"))?
        .as_array()?;
    if errors.is_empty() {
        return None;
    }
    Some(
        errors
            .iter()
            .map(|e| e.get("message").and_then(Json::as_str).unwrap_or("error"))
            .collect::<Vec<_>>()
            .join("; "),
    )
}

/// True when every user error is a not-found code (SV-4: `FILE_DOES_NOT_EXIST`).
pub fn only_not_found(payload: &Json) -> bool {
    payload
        .get("userErrors")
        .and_then(Json::as_array)
        .is_some_and(|errors| {
            !errors.is_empty()
                && errors
                    .iter()
                    .all(|e| e["code"].as_str() == Some("FILE_DOES_NOT_EXIST"))
        })
}

pub fn provider(context: &str, message: impl std::fmt::Display) -> Error {
    Error::Provider(format!("shopify {context}: {message}"))
}
