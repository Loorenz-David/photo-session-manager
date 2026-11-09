//! The clean-up decision of 10F §1, §2 and §4 levels 1–3 (P61 core mechanism): which files go,
//! in which order, and until which limit. Pure: the storage service gathers the candidates (only
//! verified duplicates, preview exports already copied into the cache, and masters no running
//! publish or export needs ever become candidates) and deletes what this returns.

use std::path::PathBuf;

/// A verified duplicate goes 24 hours after it was diverted (10F §1).
pub const DUPLICATE_AGE_MS: i64 = 24 * 3_600_000;

/// The three kinds part 1 may clear, in the budget's removal order (10F §4 levels 1–3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CleanupKind {
    Duplicate,
    PreviewExport,
    Master,
}

/// One file that may be cleared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub kind: CleanupKind,
    pub path: PathBuf,
    pub bytes: u64,
    /// When the file became what it is (a duplicate diverted, an export written, a master
    /// rendered); the budget clears the oldest first within a level.
    pub since_ms: i64,
}

/// The rule that removed a file, as logged (`storage.cleared rule=…`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// 10F §1: a verified duplicate 24 h after it was diverted.
    Duplicate24h,
    /// 10F §2: a preview export the app has copied into its cache.
    PreviewCopied,
    /// 10F §4 level 1: a verified duplicate before 24 h, free space below the minimum.
    BudgetDuplicate,
    /// 10F §4 level 3: a master no running publish or export needs, free space below the minimum.
    BudgetMaster,
}
impl Rule {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Duplicate24h => "duplicate_24h",
            Self::PreviewCopied => "preview_copied",
            Self::BudgetDuplicate => "budget_duplicate",
            Self::BudgetMaster => "budget_master",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deletion {
    pub kind: CleanupKind,
    pub path: PathBuf,
    pub bytes: u64,
    pub rule: Rule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupPlan {
    /// Ordered duplicates → preview exports → masters, oldest first within each kind.
    pub deletions: Vec<Deletion>,
    /// Free space once the deletions are done (the free space read plus what they free).
    pub free_after: u64,
    /// Still below the minimum after every level this phase may clear (10F §4: warn).
    pub short: bool,
}

/// Decides what to clear now.
///
/// 1. Routine rules, whatever the free space: verified duplicates diverted at least 24 h ago
///    (`duplicate_24h`) and every copied preview export (`preview_copied`).
/// 2. If free space (plus what step 1 frees) is still below `min_free_bytes`: level 1 the other
///    verified duplicates, level 2 (preview exports: all already taken by step 1), level 3
///    masters, each level oldest first, stopping as soon as free space reaches the minimum.
///
/// The result lists duplicates, then preview exports, then masters.
pub fn plan_cleanup(
    candidates: &[Candidate],
    now_ms: i64,
    free_bytes: u64,
    min_free_bytes: u64,
) -> CleanupPlan {
    let mut sorted: Vec<&Candidate> = candidates.iter().collect();
    sorted.sort_by(|a, b| (a.kind, a.since_ms, &a.path).cmp(&(b.kind, b.since_ms, &b.path)));
    let mut chosen: Vec<Option<Rule>> = vec![None; sorted.len()];
    let mut free = free_bytes;
    for (index, candidate) in sorted.iter().enumerate() {
        let rule = match candidate.kind {
            CleanupKind::Duplicate
                if now_ms >= candidate.since_ms.saturating_add(DUPLICATE_AGE_MS) =>
            {
                Some(Rule::Duplicate24h)
            }
            CleanupKind::PreviewExport => Some(Rule::PreviewCopied),
            _ => None,
        };
        if let Some(rule) = rule {
            chosen[index] = Some(rule);
            free = free.saturating_add(candidate.bytes);
        }
    }
    for (index, candidate) in sorted.iter().enumerate() {
        if free >= min_free_bytes {
            break;
        }
        if chosen[index].is_some() {
            continue;
        }
        let rule = match candidate.kind {
            CleanupKind::Duplicate => Rule::BudgetDuplicate,
            CleanupKind::PreviewExport => Rule::PreviewCopied,
            CleanupKind::Master => Rule::BudgetMaster,
        };
        chosen[index] = Some(rule);
        free = free.saturating_add(candidate.bytes);
    }
    let deletions = sorted
        .iter()
        .zip(chosen)
        .filter_map(|(candidate, rule)| {
            rule.map(|rule| Deletion {
                kind: candidate.kind,
                path: candidate.path.clone(),
                bytes: candidate.bytes,
                rule,
            })
        })
        .collect();
    CleanupPlan {
        deletions,
        free_after: free,
        short: free < min_free_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: i64 = 3_600_000;
    const T: i64 = 1_000 * H;

    fn c(kind: CleanupKind, name: &str, bytes: u64, since_ms: i64) -> Candidate {
        Candidate {
            kind,
            path: PathBuf::from(name),
            bytes,
            since_ms,
        }
    }
    fn names(plan: &CleanupPlan) -> Vec<(&str, &str)> {
        plan.deletions
            .iter()
            .map(|d| (d.path.to_str().unwrap(), d.rule.as_str()))
            .collect()
    }

    #[test]
    fn mc_cleanup_duplicate_goes_at_24_hours_not_before() {
        let dup = [c(CleanupKind::Duplicate, "d", 10, T)];
        assert!(
            plan_cleanup(&dup, T + 23 * H, 1_000, 0)
                .deletions
                .is_empty()
        );
        assert_eq!(
            names(&plan_cleanup(&dup, T + 24 * H, 1_000, 0)),
            vec![("d", "duplicate_24h")]
        );
    }

    #[test]
    fn mc_cleanup_copied_exports_go_at_once_masters_never_without_pressure() {
        let all = [
            c(CleanupKind::Master, "m", 10, T),
            c(CleanupKind::PreviewExport, "e", 10, T),
            c(CleanupKind::Duplicate, "d", 10, T),
        ];
        let plan = plan_cleanup(&all, T + H, 1_000, 500);
        assert_eq!(names(&plan), vec![("e", "preview_copied")]);
        assert!(!plan.short);
        assert_eq!(plan.free_after, 1_010);
    }

    #[test]
    fn mc_cleanup_budget_order_and_stop() {
        let all = [
            c(CleanupKind::Master, "m2", 100, T + 2),
            c(CleanupKind::Master, "m1", 100, T + 1),
            c(CleanupKind::PreviewExport, "e", 100, T),
            c(CleanupKind::Duplicate, "d", 100, T + 5),
        ];
        // 250 short: export (routine) + duplicate + the older master reach the limit.
        let plan = plan_cleanup(&all, T + H, 1_000, 1_250);
        assert_eq!(
            names(&plan),
            vec![
                ("d", "budget_duplicate"),
                ("e", "preview_copied"),
                ("m1", "budget_master")
            ]
        );
        assert_eq!((plan.free_after, plan.short), (1_300, false));
        // 150 short: the export and the duplicate are enough; no master goes.
        let plan = plan_cleanup(&all, T + H, 1_000, 1_150);
        assert_eq!(
            names(&plan),
            vec![("d", "budget_duplicate"), ("e", "preview_copied")]
        );
    }

    #[test]
    fn mc_cleanup_short_when_every_level_is_not_enough() {
        let all = [
            c(CleanupKind::Duplicate, "d", 100, T),
            c(CleanupKind::Master, "m", 100, T),
        ];
        let plan = plan_cleanup(&all, T + H, 1_000, 5_000);
        assert_eq!(
            names(&plan),
            vec![("d", "budget_duplicate"), ("m", "budget_master")]
        );
        assert_eq!((plan.free_after, plan.short), (1_200, true));
    }
}
