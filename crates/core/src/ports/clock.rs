use std::time::Instant;

pub trait Clock: Send + Sync {
    fn now_ms(&self) -> i64;
    fn mono(&self) -> Instant;
}

impl<T: Clock + ?Sized> Clock for std::sync::Arc<T> {
    fn now_ms(&self) -> i64 {
        self.as_ref().now_ms()
    }

    fn mono(&self) -> Instant {
        self.as_ref().mono()
    }
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
