//! MC-A9, the decision half: the error class of a failed call, what to do about it, and the
//! secret scan applied to every text a call record or log line carries.

use crate::domain::{CallFailure, ErrorClass, FailureKind};

/// Attempts per call (R-IA20).
pub const RESEARCH_CALL_ATTEMPTS: u32 = 3;

/// MC-A9 error classes. A status the contract does not name (for example 505) is `Request`.
pub fn classify(failure: &CallFailure) -> ErrorClass {
    if !failure.sent {
        return ErrorClass::Offline;
    }
    match failure.kind {
        FailureKind::Connect | FailureKind::Timeout | FailureKind::Reset => ErrorClass::Transient,
        FailureKind::Schema => ErrorClass::Schema,
        FailureKind::Refusal => ErrorClass::Refusal,
        FailureKind::Status => match failure.status {
            Some(401 | 403) => ErrorClass::Key,
            _ if failure.quota => ErrorClass::Quota,
            Some(408 | 429 | 500..=504) => ErrorClass::Transient,
            _ => ErrorClass::Request,
        },
    }
}

/// What the gateway does after a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallAction {
    /// No request byte left: hold the run in `running`, no attempt counted (MC-A6 Offline).
    HoldOffline,
    /// Try again after `wait_ms`.
    Retry { wait_ms: u64 },
    /// The run ends `failed`.
    FailRun,
    /// The unit's output is "not determined" with the refusal text.
    NotDetermined,
}

/// MC-A9 action for a class at a 1-based attempt; waits follow `jobs::backoff` (1 s, then 2 s).
pub fn action(class: ErrorClass, attempt: u32) -> CallAction {
    match class {
        ErrorClass::Offline => CallAction::HoldOffline,
        ErrorClass::Transient | ErrorClass::Schema => {
            if attempt < RESEARCH_CALL_ATTEMPTS {
                let wait = psm_infra::jobs::backoff(i64::from(attempt));
                CallAction::Retry {
                    wait_ms: u64::try_from(wait.as_millis()).unwrap_or(u64::MAX),
                }
            } else {
                CallAction::FailRun
            }
        }
        ErrorClass::Key | ErrorClass::Quota | ErrorClass::Request => CallAction::FailRun,
        ErrorClass::Refusal => CallAction::NotDetermined,
    }
}

/// A scanned text and whether any secret was found in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redacted {
    pub text: String,
    pub hit: bool,
}

/// Percent-encoding of every byte outside the URL unreserved set, with upper- or lowercase hex.
fn url_encoded(secret: &str, upper: bool) -> String {
    let mut out = String::new();
    for byte in secret.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else if upper {
            out.push_str(&format!("%{byte:02X}"));
        } else {
            out.push_str(&format!("%{byte:02x}"));
        }
    }
    out
}

/// The secret as `serde_json` writes it inside a JSON string, without the outer quotes (R8·U1).
fn json_escaped(secret: &str) -> String {
    let quoted = serde_json::to_string(secret).unwrap_or_default();
    quoted
        .strip_prefix('"')
        .and_then(|q| q.strip_suffix('"'))
        .unwrap_or(secret)
        .to_owned()
}

/// Every occurrence of each secret, plain, URL-encoded (upper- or lowercase hex) or JSON-escaped,
/// becomes `<redacted>`. An empty secret is ignored.
pub fn redact(text: &str, secrets: &[&str]) -> Redacted {
    let mut forms: Vec<String> = Vec::new();
    for secret in secrets.iter().filter(|s| !s.is_empty()) {
        for form in [
            (*secret).to_owned(),
            url_encoded(secret, true),
            url_encoded(secret, false),
            json_escaped(secret),
        ] {
            if !forms.contains(&form) {
                forms.push(form);
            }
        }
    }
    forms.sort_by_key(|f| std::cmp::Reverse(f.len()));
    let mut out = text.to_owned();
    let mut hit = false;
    for form in &forms {
        if out.contains(form.as_str()) {
            hit = true;
            out = out.replace(form.as_str(), "<redacted>");
        }
    }
    Redacted { text: out, hit }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failure(sent: bool, kind: FailureKind, status: Option<u16>, quota: bool) -> CallFailure {
        CallFailure {
            sent,
            status,
            kind,
            quota,
            text: String::new(),
            usage: None,
        }
    }

    fn status(code: u16, quota: bool) -> CallFailure {
        failure(true, FailureKind::Status, Some(code), quota)
    }

    #[test]
    fn a06_c5a_not_sent_is_offline() {
        assert_eq!(
            classify(&failure(false, FailureKind::Connect, None, false)),
            ErrorClass::Offline
        );
    }

    #[test]
    fn a06_c5b_timeout_and_reset_after_send() {
        assert_eq!(
            classify(&failure(true, FailureKind::Timeout, None, false)),
            ErrorClass::Transient
        );
        assert_eq!(
            classify(&failure(true, FailureKind::Reset, None, false)),
            ErrorClass::Transient
        );
    }

    #[test]
    fn a06_c5c_transient_statuses() {
        for code in [408, 429, 500, 502, 503, 504] {
            assert_eq!(
                classify(&status(code, false)),
                ErrorClass::Transient,
                "{code}"
            );
        }
    }

    #[test]
    fn a06_c5d_key_statuses() {
        assert_eq!(classify(&status(401, false)), ErrorClass::Key);
        assert_eq!(classify(&status(403, false)), ErrorClass::Key);
    }

    #[test]
    fn a06_c5e_quota() {
        assert_eq!(classify(&status(429, true)), ErrorClass::Quota);
        assert_eq!(classify(&status(402, true)), ErrorClass::Quota);
    }

    #[test]
    fn a06_c5f_other_client_errors() {
        for code in [400, 404, 422] {
            assert_eq!(
                classify(&status(code, false)),
                ErrorClass::Request,
                "{code}"
            );
        }
    }

    #[test]
    fn a06_c5g_schema_and_refusal() {
        assert_eq!(
            classify(&failure(true, FailureKind::Schema, None, false)),
            ErrorClass::Schema
        );
        assert_eq!(
            classify(&failure(true, FailureKind::Refusal, None, false)),
            ErrorClass::Refusal
        );
    }

    #[test]
    fn a06_c6a_offline_holds() {
        for attempt in [1, 2, 3, 7] {
            assert_eq!(
                action(ErrorClass::Offline, attempt),
                CallAction::HoldOffline
            );
        }
    }

    #[test]
    fn a06_c6b_transient_retries_then_fails() {
        assert_eq!(
            action(ErrorClass::Transient, 1),
            CallAction::Retry { wait_ms: 1000 }
        );
        assert_eq!(
            action(ErrorClass::Transient, 2),
            CallAction::Retry { wait_ms: 2000 }
        );
        assert_eq!(action(ErrorClass::Transient, 3), CallAction::FailRun);
    }

    #[test]
    fn a06_c6c_schema_retries_then_fails() {
        assert_eq!(
            action(ErrorClass::Schema, 1),
            CallAction::Retry { wait_ms: 1000 }
        );
        assert_eq!(action(ErrorClass::Schema, 3), CallAction::FailRun);
    }

    #[test]
    fn a06_c6d_key_quota_request_fail_at_once() {
        for class in [ErrorClass::Key, ErrorClass::Quota, ErrorClass::Request] {
            assert_eq!(action(class, 1), CallAction::FailRun, "{class:?}");
        }
    }

    #[test]
    fn a06_c6e_refusal_not_determined() {
        assert_eq!(action(ErrorClass::Refusal, 1), CallAction::NotDetermined);
    }

    #[test]
    fn a06_c7a_plain_and_url_encoded() {
        let secrets = ["sk-ab+c/d="];
        assert_eq!(
            redact("Bearer sk-ab+c/d=", &secrets),
            Redacted {
                text: "Bearer <redacted>".to_owned(),
                hit: true
            }
        );
        assert_eq!(
            redact("?api_key=sk-ab%2Bc%2Fd%3D&q=1", &secrets),
            Redacted {
                text: "?api_key=<redacted>&q=1".to_owned(),
                hit: true
            }
        );
        assert_eq!(
            redact("?api_key=sk-ab%2bc%2fd%3d&q=1", &secrets).text,
            "?api_key=<redacted>&q=1"
        );
    }

    #[test]
    fn a06_c7b_every_occurrence_of_every_secret() {
        let out = redact("a=key-one b=key-two c=key-one", &["key-one", "key-two"]);
        assert_eq!(out.text, "a=<redacted> b=<redacted> c=<redacted>");
        assert!(out.hit);
    }

    #[test]
    fn a06_c7c_nothing_to_redact() {
        assert_eq!(
            redact("no secrets here", &["key-one"]),
            Redacted {
                text: "no secrets here".to_owned(),
                hit: false
            }
        );
        assert_eq!(
            redact("no secrets here", &[""]),
            Redacted {
                text: "no secrets here".to_owned(),
                hit: false
            }
        );
    }
}
