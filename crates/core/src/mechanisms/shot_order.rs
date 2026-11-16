//! Shutter order with places reserved by unfulfilled camera exposures (10G P4).
use crate::domain::camera::FIFO_WINDOW_MS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shot {
    pub photo_id: i64,
    pub at: i64,
    pub counter: Option<(String, u64)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Shot(Shot),
    Hold { at: i64, id: i64 },
}

impl Entry {
    fn at(&self) -> i64 {
        match self {
            Self::Shot(s) => s.at,
            Self::Hold { at, .. } => *at,
        }
    }
    fn id(&self) -> i64 {
        match self {
            Self::Shot(s) => s.photo_id,
            Self::Hold { id, .. } => *id,
        }
    }
    fn participates(&self) -> bool {
        matches!(self, Self::Hold { .. })
            || matches!(
                self,
                Self::Shot(Shot {
                    counter: Some(_),
                    ..
                })
            )
    }
    fn prefix(&self) -> Option<&str> {
        match self {
            Self::Shot(Shot {
                counter: Some((prefix, _)),
                ..
            }) => Some(prefix),
            _ => None,
        }
    }
}

/// P4: time makes the slots; a common file-counter run assigns received photos to those slots.
pub fn order(shots: Vec<Shot>, holds: Vec<(i64, i64)>) -> Vec<Entry> {
    let mut entries: Vec<Entry> = shots
        .into_iter()
        .map(Entry::Shot)
        .chain(holds.into_iter().map(|(at, id)| Entry::Hold { at, id }))
        .collect();
    entries.sort_by_key(|e| (e.at(), matches!(e, Entry::Hold { .. }), e.id()));
    let participating: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| e.participates().then_some(i))
        .collect();
    let mut start = 0;
    while start < participating.len() {
        let mut end = start + 1;
        let mut prefix = entries[participating[start]].prefix().map(str::to_owned);
        while end < participating.len() {
            let prev = &entries[participating[end - 1]];
            let next = &entries[participating[end]];
            let close = next.at().saturating_sub(prev.at()) <= FIFO_WINDOW_MS;
            let same = next
                .prefix()
                .is_none_or(|p| prefix.as_deref().is_none_or(|q| p == q));
            if !close || !same {
                break;
            }
            if prefix.is_none() {
                prefix = next.prefix().map(str::to_owned);
            }
            end += 1;
        }
        let slots = &participating[start..end];
        let mut run_shots: Vec<Shot> = slots
            .iter()
            .filter_map(|i| match &entries[*i] {
                Entry::Shot(s) => Some(s.clone()),
                Entry::Hold { .. } => None,
            })
            .collect();
        if !run_shots.is_empty() {
            let largest = run_shots
                .iter()
                .filter_map(|s| s.counter.as_ref().map(|c| c.1))
                .max()
                .unwrap_or(0);
            let adjusted = |s: &Shot| {
                let n = s.counter.as_ref().map(|c| c.1).unwrap_or(0);
                if largest.saturating_sub(n) > 5000 {
                    n.saturating_add(10000)
                } else {
                    n
                }
            };
            run_shots.sort_by_key(|s| (adjusted(s), s.at, s.photo_id));
            let m = slots.len();
            let k = run_shots.len();
            let mut assigned: Vec<Option<Entry>> = vec![None; m];
            let mut previous_slot = 0usize;
            let mut previous_counter = 0u64;
            for (j, shot) in run_shots.into_iter().enumerate() {
                let counter = adjusted(&shot);
                let slot = if j == 0 {
                    0
                } else {
                    let advance = counter.saturating_sub(previous_counter).max(1);
                    previous_slot
                        .saturating_add(usize::try_from(advance).unwrap_or(usize::MAX))
                        .min(m - (k - j))
                };
                assigned[slot] = Some(Entry::Shot(shot));
                previous_slot = slot;
                previous_counter = counter;
            }
            let holds: Vec<Entry> = slots
                .iter()
                .filter_map(|i| match &entries[*i] {
                    Entry::Hold { .. } => Some(entries[*i].clone()),
                    Entry::Shot(_) => None,
                })
                .collect();
            let mut remaining = holds.into_iter();
            for (offset, slot) in slots.iter().enumerate() {
                entries[*slot] = assigned[offset]
                    .take()
                    .unwrap_or_else(|| remaining.next().expect("run has enough holds"));
            }
        }
        start = end;
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanisms::capture_ledger::{seq_of, stem_key};

    fn shot(id: i64, at: i64, name: Option<&str>) -> Shot {
        Shot {
            photo_id: id,
            at,
            counter: name.and_then(|n| seq_of(&stem_key(n))),
        }
    }
    fn ids(shots: Vec<Shot>, holds: Vec<(i64, i64)>) -> Vec<i64> {
        order(shots, holds)
            .iter()
            .map(|e| match e {
                Entry::Shot(s) => s.photo_id,
                Entry::Hold { id, .. } => -*id,
            })
            .collect()
    }
    #[test]
    fn t2_c1a_time_and_id() {
        assert_eq!(
            ids(
                vec![
                    shot(1, 10, None),
                    shot(2, 30, None),
                    shot(3, 20, None),
                    shot(7, 40, None),
                    shot(5, 40, None)
                ],
                vec![]
            ),
            vec![1, 3, 2, 5, 7]
        );
    }
    #[test]
    fn t2_c1b_counter_corrects_time() {
        assert_eq!(
            ids(
                vec![
                    shot(1, 10, Some("DSC00001.ARW")),
                    shot(2, 20, Some("DSC00003.ARW")),
                    shot(3, 30, Some("DSC00002.ARW"))
                ],
                vec![]
            ),
            vec![1, 3, 2]
        );
    }
    #[test]
    fn t2_c1c_gap_reserves_hold() {
        assert_eq!(
            ids(
                vec![
                    shot(1, 10, Some("DSC00001.ARW")),
                    shot(2, 20, Some("DSC00003.ARW"))
                ],
                vec![(30, 9)]
            ),
            vec![1, -9, 2]
        );
    }
    #[test]
    fn t2_c1d_no_gap_ends_with_hold() {
        assert_eq!(
            ids(
                vec![
                    shot(1, 10, Some("DSC00001.ARW")),
                    shot(2, 20, Some("DSC00002.ARW"))
                ],
                vec![(15, 9)]
            ),
            vec![1, 2, -9]
        );
    }
    #[test]
    fn t2_c1e_first_shot_late() {
        assert_eq!(
            ids(vec![shot(2, 10, Some("DSC00002.ARW"))], vec![(20, 9)]),
            vec![2, -9]
        );
    }
    #[test]
    fn t2_c1f_multiple_gaps() {
        assert_eq!(
            ids(
                vec![
                    shot(1, 10, Some("DSC00001.ARW")),
                    shot(2, 20, Some("DSC00003.ARW")),
                    shot(3, 30, Some("DSC00005.ARW"))
                ],
                vec![(40, 9)]
            ),
            vec![1, -9, 2, 3]
        );
    }
    #[test]
    fn t2_c1g_counter_restart() {
        assert_eq!(
            ids(
                vec![
                    shot(1, 20, Some("DSC09999.ARW")),
                    shot(2, 10, Some("DSC00001.ARW"))
                ],
                vec![]
            ),
            vec![1, 2]
        );
    }
    #[test]
    fn t2_c1h_outside_window() {
        assert_eq!(
            ids(
                vec![
                    shot(1, 0, Some("DSC00005.ARW")),
                    shot(2, FIFO_WINDOW_MS + 1, Some("DSC00004.ARW"))
                ],
                vec![]
            ),
            vec![1, 2]
        );
    }
    #[test]
    fn t2_c1i_no_counter_keeps_its_slot() {
        assert_eq!(
            ids(
                vec![
                    shot(1, 10, Some("DSC00002.ARW")),
                    shot(3, 15, None),
                    shot(2, 20, Some("DSC00001.ARW"))
                ],
                vec![]
            ),
            vec![2, 3, 1]
        );
    }
    #[test]
    fn t2_c1j_prefix_starts_new_run() {
        assert_eq!(
            ids(
                vec![
                    shot(1, 10, Some("DSC00002.ARW")),
                    shot(2, 20, Some("IMG00001.ARW"))
                ],
                vec![]
            ),
            vec![1, 2]
        );
    }
}
