use std::time::Duration;

/// Retry delay after a deferred preview render attempt.
pub fn render_backoff(attempts: u32) -> Duration {
    let exponent = attempts.saturating_sub(1).min(6);
    Duration::from_secs((1_u64 << exponent).min(60))
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
