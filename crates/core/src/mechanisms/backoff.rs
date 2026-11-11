use std::time::Duration;

/// Retry delay after a deferred preview render attempt.
pub fn render_backoff(attempts: u32) -> Duration {
    let exponent = attempts.saturating_sub(1).min(6);
    Duration::from_secs((1_u64 << exponent).min(60))
}

/// Changelog 45 (P65): an export or publish render that meets an editor that is not ready waits at
/// most this long before the run fails.
pub const EDITOR_WAIT_MAX_MS: i64 = 10 * 60 * 1000;
/// The text shown while a run waits for the editor (popover step, session row).
pub const WAITING_FOR_EDITOR: &str = "Waiting for Capture One…";
/// The prefix of a failed export run's reason when its wait for the editor ran out; the rest is
/// the sentence shown to the photographer.
pub const EDITOR_WAIT_EXPIRED: &str = "editor_wait: ";

/// Why a render could not start or finish because the editor was not ready (changelog 45).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorWait {
    /// The editor has not listed the photo yet (`variant_not_found`).
    NotLoaded,
    /// The editor did not answer in time, or a render produced nothing in time.
    Busy,
    /// The editor is closed or held ("Take the camera back").
    Unavailable,
}

/// Classifies a render error (its text, as errors reach the export) as "the editor is not ready"
/// (`Some`: wait and retry) or anything else (`None`: fail as before).
pub fn editor_wait(error: &str) -> Option<EditorWait> {
    let text = error.to_ascii_lowercase();
    if text.contains("variant_not_found") {
        Some(EditorWait::NotLoaded)
    } else if text.contains("editor_launch_deferred")
        || text.contains("document_not_open")
        || text.contains("not running")
        || text.contains("-600")
    {
        Some(EditorWait::Unavailable)
    } else if text == "timeout"
        || text.contains("timed out")
        || text.contains("-1712")
        || text.contains("busy")
    {
        Some(EditorWait::Busy)
    } else {
        None
    }
}

/// The delay before retry `attempt` (1-based) of a waiting render: 1, 2, 4, 8, then 15 s.
pub fn editor_wait_backoff_ms(attempt: u32) -> i64 {
    (1_000_i64 << attempt.saturating_sub(1).min(4)).min(15_000)
}

/// The sentence a run fails with after waiting `EDITOR_WAIT_MAX_MS` for `photo` (a file name).
pub fn editor_wait_expired(reason: EditorWait, photo: &str) -> String {
    match reason {
        EditorWait::NotLoaded => format!("Capture One has not loaded {photo} after 10 minutes"),
        EditorWait::Busy => format!("Capture One did not render {photo} within 10 minutes"),
        EditorWait::Unavailable => {
            format!("Capture One was not available to render {photo} for 10 minutes")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mc19_render_backoff() {
        let actual: Vec<_> = (1..=9)
            .map(|attempt| render_backoff(attempt).as_secs())
            .collect();
        assert_eq!(actual, [1, 2, 4, 8, 16, 32, 60, 60, 60]);
    }
}
