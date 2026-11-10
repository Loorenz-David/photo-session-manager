//! Retention after publish (10F §3, §4 levels 4–5, Q2, H6, M41; P62 core mechanism): which
//! items' local files may be cleared, and in which order the storage budget clears them. Pure:
//! the storage service gathers the inputs, verifies each chosen item against the commerce
//! platform (M41) and deletes only after that.

use crate::domain::session::PublishStep;

pub const DAY_MS: i64 = 86_400_000;

/// What the decision needs to know about one item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionInput {
    /// A commerce integration is configured (H6: without one nothing is cleared).
    pub integration: bool,
    /// When the item's last publish completed (`sessions.published_at`).
    pub published_at: Option<i64>,
    /// The item changed since that publish (07 dirty).
    pub dirty: bool,
    /// The step of the item's latest publish run (`Done` for a completed publish).
    pub latest_step: Option<PublishStep>,
    /// The "keep files on this Mac" pin (Q2).
    pub keep_files: bool,
    /// Already cleared.
    pub cleared: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eligibility {
    /// Never cleared by retention or by the budget: not published-and-clean, pinned, already
    /// cleared, or no integration.
    Never,
    /// Published and clean, not yet `retention_days` old: only the budget's level 5 may clear it.
    NotYetDue { due_at: i64 },
    /// Published and clean for at least `retention_days` (10F §3 rule, budget level 4).
    Due,
}

/// 10F §3: an item is cleared `retention_days` after its publish completed, if it is published
/// (latest run done), not dirty, not pinned, not cleared yet, and an integration is configured.
pub fn eligibility(input: RetentionInput, now_ms: i64, retention_days: u32) -> Eligibility {
    let Some(published_at) = input.published_at else {
        return Eligibility::Never;
    };
    if !input.integration
        || input.dirty
        || input.keep_files
        || input.cleared
        || input.latest_step != Some(PublishStep::Done)
    {
        return Eligibility::Never;
    }
    let due_at = published_at.saturating_add(i64::from(retention_days).saturating_mul(DAY_MS));
    if now_ms >= due_at {
        Eligibility::Due
    } else {
        Eligibility::NotYetDue { due_at }
    }
}

/// One item that may be cleared, with the bytes its local files take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionCandidate {
    pub item_id: i64,
    pub published_at: i64,
    pub bytes: u64,
    /// `true` for `Eligibility::Due`, `false` for `NotYetDue`.
    pub due: bool,
}

/// The rule that cleared an item's files, as logged (`storage.cleared rule=…`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionRule {
    /// 10F §3 (and budget level 4): published and clean for the retention period.
    Retention,
    /// 10F §4 level 5: published and clean, not yet past retention, free space below the minimum.
    BudgetPublished,
}
impl RetentionRule {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Retention => "retention",
            Self::BudgetPublished => "budget_published",
        }
    }
}

/// Decides which items to clear now, in order: every due item, oldest publish first (10F §3,
/// level 4); then, while `free_bytes` plus what the chosen items free is below `min_free_bytes`,
/// the items not yet due, oldest publish first (level 5). Ties go to the lower item id.
pub fn plan_retention(
    candidates: &[RetentionCandidate],
    free_bytes: u64,
    min_free_bytes: u64,
) -> Vec<(i64, RetentionRule)> {
    let mut sorted: Vec<&RetentionCandidate> = candidates.iter().collect();
    sorted.sort_by_key(|c| (!c.due, c.published_at, c.item_id));
    let mut free = free_bytes;
    let mut out = Vec::new();
    for candidate in sorted {
        if candidate.due {
            out.push((candidate.item_id, RetentionRule::Retention));
        } else if free < min_free_bytes {
            out.push((candidate.item_id, RetentionRule::BudgetPublished));
        } else {
            break;
        }
        free = free.saturating_add(candidate.bytes);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: i64 = 1_762_000_000_000;

    fn published() -> RetentionInput {
        RetentionInput {
            integration: true,
            published_at: Some(T),
            dirty: false,
            latest_step: Some(PublishStep::Done),
            keep_files: false,
            cleared: false,
        }
    }

    #[test]
    fn mc_retention_due_exactly_after_n_days() {
        assert_eq!(
            eligibility(published(), T + 4 * DAY_MS - 1, 4),
            Eligibility::NotYetDue {
                due_at: T + 4 * DAY_MS
            }
        );
        assert_eq!(
            eligibility(published(), T + 4 * DAY_MS, 4),
            Eligibility::Due
        );
        assert_eq!(eligibility(published(), T, 0), Eligibility::Due);
    }

    #[test]
    fn mc_retention_never_for_dirty_unpublished_pinned_cleared_failed_or_without_integration() {
        let late = T + 30 * DAY_MS;
        for input in [
            RetentionInput {
                dirty: true,
                ..published()
            },
            RetentionInput {
                published_at: None,
                latest_step: None,
                ..published()
            },
            RetentionInput {
                keep_files: true,
                ..published()
            },
            RetentionInput {
                cleared: true,
                ..published()
            },
            RetentionInput {
                integration: false,
                ..published()
            },
            RetentionInput {
                latest_step: Some(PublishStep::Failed),
                ..published()
            },
            RetentionInput {
                latest_step: Some(PublishStep::Upload),
                ..published()
            },
        ] {
            assert_eq!(eligibility(input, late, 4), Eligibility::Never, "{input:?}");
        }
    }

    fn c(item_id: i64, published_at: i64, bytes: u64, due: bool) -> RetentionCandidate {
        RetentionCandidate {
            item_id,
            published_at,
            bytes,
            due,
        }
    }

    #[test]
    fn mc_retention_due_items_always_oldest_first_then_budget_until_enough() {
        let all = [
            c(3, T + 3, 100, false),
            c(2, T + 2, 100, false),
            c(9, T + 9, 100, true),
            c(1, T + 1, 100, true),
        ];
        // Enough space: only the due items, oldest publish first.
        assert_eq!(
            plan_retention(&all, 10_000, 1_000),
            vec![(1, RetentionRule::Retention), (9, RetentionRule::Retention)]
        );
        // 250 short: the two due items free 200, then the oldest not-yet-due item.
        assert_eq!(
            plan_retention(&all, 750, 1_000),
            vec![
                (1, RetentionRule::Retention),
                (9, RetentionRule::Retention),
                (2, RetentionRule::BudgetPublished)
            ]
        );
        // Far short: everything, still in order.
        assert_eq!(
            plan_retention(&all, 0, 1_000)
                .into_iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            vec![1, 9, 2, 3]
        );
    }
}
