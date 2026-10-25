use std::time::Instant;

pub trait Clock: Send + Sync {
    fn now_ms(&self) -> i64;
    fn mono(&self) -> Instant;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        chrono::Utc::now().timestamp_millis()
    }

    fn mono(&self) -> Instant {
        Instant::now()
    }
}
