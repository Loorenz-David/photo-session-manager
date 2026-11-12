//! Stable log-site interest for thread-scoped log captures (P70).
//!
//! Several tests read the app's log lines through a thread-scoped subscriber
//! (`tracing::subscriber::with_default`). `tracing` caches, per log site, whether any subscriber
//! wants it, and recomputes that cache whenever a scoped subscriber is created or dropped. With
//! every phase's tests in one process, another test thread can hit the same log site during such a
//! recompute and the capturing test misses its line. A process-wide subscriber that wants every
//! site (and records nothing) keeps every site's interest at "always", so a capture never misses a
//! line written on its own thread. It writes nothing anywhere; the app's behaviour is unchanged.

use std::sync::Once;
use tracing::{
    Event, Metadata, Subscriber,
    level_filters::LevelFilter,
    span::{Attributes, Id, Record},
    subscriber::Interest,
};

struct EverySite;
impl Subscriber for EverySite {
    fn register_callsite(&self, _: &'static Metadata<'static>) -> Interest {
        Interest::always()
    }
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }
    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(LevelFilter::TRACE)
    }
    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }
    fn record(&self, _: &Id, _: &Record<'_>) {}
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn event(&self, _: &Event<'_>) {}
    fn enter(&self, _: &Id) {}
    fn exit(&self, _: &Id) {}
}

/// Installs the process-wide "every site" subscriber once, unless the process already has a
/// global subscriber (an isolated test that installs its own keeps it).
pub fn keep_log_sites_interested() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = tracing::subscriber::set_global_default(EverySite);
    });
}
