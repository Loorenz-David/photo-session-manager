use psm_core::ports::clock::Clock;
use std::{
    sync::atomic::{AtomicI64, Ordering},
    time::Instant,
};

pub struct ManualClock {
    ms: AtomicI64,
    start: Instant,
}
impl ManualClock {
    pub fn new(ms: i64) -> Self {
        Self {
            ms: AtomicI64::new(ms),
            start: Instant::now(),
        }
    }
    pub fn advance(&self, ms: i64) {
        self.ms.fetch_add(ms, Ordering::SeqCst);
    }
}
impl Clock for ManualClock {
    fn now_ms(&self) -> i64 {
        self.ms.load(Ordering::SeqCst)
    }
    fn mono(&self) -> Instant {
        self.start
    }
}
