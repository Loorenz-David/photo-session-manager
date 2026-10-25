use std::time::{Duration, Instant};

pub fn is_stale(adjusted_gen: i64, rendered_gen: i64) -> bool {
    adjusted_gen > rendered_gen
}

pub fn due_at(last_adjusted: Instant, delay: Duration) -> Instant {
    last_adjusted + delay
}

pub fn clamp_delay(setting: Option<&str>) -> Duration {
    let seconds = setting.and_then(|v| v.parse::<u64>().ok()).unwrap_or(15);
    Duration::from_secs(seconds.clamp(10, 30))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mc17_staleness_and_due_time() {
        let start = Instant::now();
        assert!(is_stale(2, 1));
        assert!(!is_stale(1, 1));
        assert_eq!(
            due_at(start, Duration::from_secs(15)),
            start + Duration::from_secs(15)
        );
    }

    #[test]
    fn mc17_delay_clamp() {
        for (input, expected) in [
            (Some("5"), 10),
            (Some("40"), 30),
            (None, 15),
            (Some("abc"), 15),
        ] {
            assert_eq!(clamp_delay(input).as_secs(), expected);
        }
    }
}
