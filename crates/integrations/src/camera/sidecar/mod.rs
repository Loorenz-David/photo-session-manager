pub mod client;
pub mod protocol;
pub mod supervisor;
pub const RESTART_BACKOFF_S: [u64; 5] = [1, 2, 4, 8, 16];
pub const RESTART_STEADY_S: u64 = 30;
pub const QUIT_GRACE_MS: u64 = 3_000;

/// The sidecar timings a test may shorten (P70). Production always uses `Default`, today's values;
/// only the test harness calls `CameraRuntime::set_sidecar_timings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SidecarTimings {
    /// How long a quitting sidecar gets to exit after `quit` before it is killed.
    pub quit_grace: std::time::Duration,
}
impl Default for SidecarTimings {
    fn default() -> Self {
        Self {
            quit_grace: std::time::Duration::from_millis(QUIT_GRACE_MS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Core mechanism (P70): every timing made test-settable keeps today's production value.
    #[test]
    fn production_timings_unchanged() {
        assert_eq!(
            SidecarTimings::default(),
            SidecarTimings {
                quit_grace: Duration::from_millis(3_000),
            }
        );
    }
}
