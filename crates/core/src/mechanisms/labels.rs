//! The pure rules of labels (book 09C §7, §8; `master_plan_labels.md` §4 Constants, Pure rules,
//! §5 MC-L3.2): request chunking and the display copy of label state. Memory only: nothing here
//! is ever stored (M-L1).

use crate::domain::label::{LabelChange, LabelChangeKind, LabelState, LabelStatus};
use std::{collections::HashMap, ops::Range, time::Duration, time::Instant};

/// At most this many product ids per status request (the provider's maximum).
pub const STATUS_CHUNK: usize = 200;
/// A status answer younger than this is not asked again by a refreshing read.
pub const STATUS_FRESH_MS: u64 = 10_000;

/// Consecutive ranges covering `0..n`, each `size` long except the last, which is shorter;
/// none for `n = 0`.
pub fn chunks(n: usize, size: usize) -> Vec<Range<usize>> {
    let size = size.max(1);
    (0..n)
        .step_by(size)
        .map(|start| start..(start + size).min(n))
        .collect()
}

#[derive(Debug, Clone, Default)]
struct Entry {
    /// `None` = held with no state (the provider answered that it does not know the product).
    state: Option<LabelState>,
    /// The sequence number of the last event applied to this product (0 = none).
    event_seq: u64,
    /// When a status or list answer was last applied (monotonic).
    answered_at: Option<Instant>,
}

/// The display copy (MC-L3.2): product id → label state, the sequence number of the last event
/// applied to it, and when it was last answered.
#[derive(Debug, Default)]
pub struct LabelCopy {
    seq: u64,
    /// Answers started before this sequence were asked before the copy was emptied.
    floor: u64,
    entries: HashMap<String, Entry>,
}

impl LabelCopy {
    pub fn new() -> Self {
        Self::default()
    }

    /// The current sequence, taken before a request whose answer will be applied.
    pub fn start(&self) -> u64 {
        self.seq
    }

    /// Applies one answer of a request started at `started`: stored (with no state for `None`,
    /// D10) only when no event was applied to that product after `started` — an event that
    /// arrived while the request was in flight wins over the older answer.
    pub fn apply_answer(
        &mut self,
        started: u64,
        product_id: &str,
        state: Option<LabelState>,
        now: Instant,
    ) {
        if started < self.floor {
            return;
        }
        let entry = self.entries.entry(product_id.to_owned()).or_default();
        if entry.event_seq > started {
            return;
        }
        entry.state = state;
        entry.answered_at = Some(now);
    }

    /// Applies one label event: `Printed` → printed at the event's time with a price and no
    /// reset; `Reset` → not printed with the reason, the price as it was (unknown without an
    /// entry). Creates the entry when there is none and raises the sequence.
    pub fn apply_event(&mut self, change: &LabelChange) {
        self.seq += 1;
        let seq = self.seq;
        let entry = self.entries.entry(change.product_id.clone()).or_default();
        entry.state = Some(match change.kind {
            LabelChangeKind::Printed { .. } => LabelState {
                status: LabelStatus::Printed,
                printed_at: Some(change.at),
                has_price: Some(true),
                reset: None,
            },
            LabelChangeKind::Reset { reason } => LabelState {
                status: LabelStatus::NotPrinted,
                printed_at: None,
                has_price: entry.state.and_then(|s| s.has_price),
                reset: reason,
            },
        });
        entry.event_seq = seq;
    }

    /// The state held for the product.
    pub fn get(&self, product_id: &str) -> Option<LabelState> {
        self.entries.get(product_id).and_then(|e| e.state)
    }

    /// Whether the product was answered less than `max_age` before `now`.
    pub fn fresh(&self, product_id: &str, now: Instant, max_age: Duration) -> bool {
        self.entries
            .get(product_id)
            .and_then(|e| e.answered_at)
            .is_some_and(|at| now.saturating_duration_since(at) < max_age)
    }

    /// Empties the copy; answers to requests started before this are not applied.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.seq += 1;
        self.floor = self.seq;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::label::ResetReason;

    fn printed(at: i64) -> LabelState {
        LabelState {
            status: LabelStatus::Printed,
            printed_at: Some(at),
            has_price: Some(true),
            reset: None,
        }
    }

    fn missing_tag() -> LabelState {
        LabelState {
            status: LabelStatus::NotPrinted,
            printed_at: None,
            has_price: Some(true),
            reset: None,
        }
    }

    fn change(product_id: &str, at: i64, kind: LabelChangeKind) -> LabelChange {
        LabelChange {
            product_id: product_id.into(),
            at,
            kind,
        }
    }

    #[test]
    fn chunks_of_nothing_is_none() {
        assert_eq!(chunks(0, 200), Vec::<Range<usize>>::new());
    }

    #[test]
    fn chunks_of_exactly_one_size() {
        assert_eq!(chunks(200, 200), vec![0..200]);
    }

    #[test]
    fn chunks_with_a_shorter_last_one() {
        assert_eq!(chunks(201, 200), vec![0..200, 200..201]);
    }

    #[test]
    fn an_answer_older_than_an_event_for_the_same_product_is_ignored() {
        let now = Instant::now();
        let mut copy = LabelCopy::new();
        let started = copy.start();
        copy.apply_event(&change(
            "p1",
            5,
            LabelChangeKind::Printed { price_minor: None },
        ));
        copy.apply_answer(started, "p1", Some(missing_tag()), now);
        assert_eq!(copy.get("p1"), Some(printed(5)));
    }

    #[test]
    fn an_answer_for_another_product_than_the_event_is_stored() {
        let now = Instant::now();
        let mut copy = LabelCopy::new();
        let started = copy.start();
        copy.apply_event(&change(
            "p1",
            5,
            LabelChangeKind::Printed { price_minor: None },
        ));
        copy.apply_answer(started, "p2", Some(missing_tag()), now);
        assert_eq!(copy.get("p2"), Some(missing_tag()));
    }

    #[test]
    fn reset_keeps_the_price_it_had() {
        let now = Instant::now();
        let mut copy = LabelCopy::new();
        let mut no_price = printed(1);
        no_price.has_price = Some(false);
        copy.apply_answer(copy.start(), "p1", Some(no_price), now);
        copy.apply_event(&change(
            "p1",
            9,
            LabelChangeKind::Reset {
                reason: Some(ResetReason::PriceChanged),
            },
        ));
        assert_eq!(
            copy.get("p1"),
            Some(LabelState {
                status: LabelStatus::NotPrinted,
                printed_at: None,
                has_price: Some(false),
                reset: Some(ResetReason::PriceChanged),
            })
        );
    }

    #[test]
    fn reset_without_an_entry_creates_one_with_the_price_unknown() {
        let mut copy = LabelCopy::new();
        copy.apply_event(&change(
            "p9",
            9,
            LabelChangeKind::Reset {
                reason: Some(ResetReason::SkuChanged),
            },
        ));
        assert_eq!(
            copy.get("p9"),
            Some(LabelState {
                status: LabelStatus::NotPrinted,
                printed_at: None,
                has_price: None,
                reset: Some(ResetReason::SkuChanged),
            })
        );
    }

    #[test]
    fn fresh_within_the_age_and_not_after_it_nor_without_an_entry() {
        let now = Instant::now();
        let age = Duration::from_millis(STATUS_FRESH_MS);
        let mut copy = LabelCopy::new();
        copy.apply_answer(copy.start(), "p1", Some(missing_tag()), now);
        assert!(copy.fresh("p1", now + Duration::from_secs(9), age));
        assert!(!copy.fresh("p1", now + Duration::from_secs(11), age));
        assert!(!copy.fresh("p2", now, age));
    }
}
