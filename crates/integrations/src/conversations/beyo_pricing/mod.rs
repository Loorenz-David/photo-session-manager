//! The Beyo Pricing adapter of the conversations port (book 09B; `master_plan_conversations.md`
//! §4 Adapter). Pricing's API is owned by its integration package (`API.md`,
//! `EVENTS_AND_WEBHOOKS.md`, `openapi-integration.json`); this module is the only place that
//! knows its addresses, its wire words and its sentences.

pub mod adapter;
pub mod label_types;
pub mod labels;
pub mod transport;
pub mod types;

pub use adapter::BeyoPricingAdapter;

/// The server used when the settings name none (09B §4).
pub const DEFAULT_SERVER: &str = "https://pricing.beyoworkaroundtheclock.com";

/// The server address as stored (09B §4, master plan §4 Settings): trimmed, one trailing `/`
/// removed, `https://` with a host and no path, query, fragment or user info. `None` = refused.
pub fn checked_server(raw: &str) -> Option<String> {
    let text = raw.trim();
    let text = text.strip_suffix('/').unwrap_or(text);
    let host = text.strip_prefix("https://")?;
    let (name, port) = match host.split_once(':') {
        Some((name, port)) => (name, Some(port)),
        None => (host, None),
    };
    let name_ok = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.'));
    let port_ok = port.is_none_or(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
    (name_ok && port_ok).then(|| text.to_owned())
}

/// The effective server for a stored setting: the checked address, or [`DEFAULT_SERVER`].
pub fn effective_server(stored: Option<&str>) -> String {
    stored
        .and_then(checked_server)
        .unwrap_or_else(|| DEFAULT_SERVER.to_owned())
}

#[cfg(test)]
mod tests {
    use super::checked_server;

    #[test]
    fn c1_c6a_checked_server_keeps_only_bare_https_hosts() {
        assert_eq!(
            checked_server("https://pricing.example/").as_deref(),
            Some("https://pricing.example")
        );
        assert_eq!(
            checked_server(" https://pricing.example ").as_deref(),
            Some("https://pricing.example")
        );
        for refused in [
            "http://pricing.example",
            "https://pricing.example/x",
            "https://u:p@pricing.example",
            "pricing.example",
            "https://",
        ] {
            assert_eq!(checked_server(refused), None, "{refused}");
        }
    }
}
