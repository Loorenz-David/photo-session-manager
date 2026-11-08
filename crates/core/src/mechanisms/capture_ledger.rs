//! 10E MC-33 — the capture ledger's pure rules (file key, association order with the FIFO
//! window, duplicate decision, completion, target folder, watchdog count, numbering gaps).
//! The service (`services::camera_ledger`) feeds every `expecting` exposure and every `done`
//! prior with the same key or stem; every filter below is applied here.
use crate::domain::camera::{Association, DeliveryOutcome, ExposureStatus, FIFO_WINDOW_MS};
use crate::domain::camera::{WATCHDOG_MAX_AGE_MS, WATCHDOG_MIN_AGE_MS};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FifoCandidate {
    pub exposure_id: i64,
    pub exposed_at: i64,
    pub session_id: Option<i64>,
    pub expected_files: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorDelivery {
    pub id: i64,
    pub file_key: String,
    pub stem_key: String,
    pub sha256: String,
    pub exposure_id: Option<i64>,
    pub session_id: Option<i64>,
    pub outcome: DeliveryOutcome,
    pub completed_at: i64,
    pub final_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociationDecision {
    pub exposure_id: Option<i64>,
    pub association: Association,
    pub session_id: Option<i64>,
    /// Other sessions in the FIFO candidate set (distinct, in candidate order); `None` = Unassigned.
    pub ambiguous_with: Vec<Option<i64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateDecision {
    Photo,
    Duplicate { of: i64 },
    RenamedPhoto { n: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gap {
    pub prefix: String,
    pub after: u64,
    pub before: u64,
    pub missing: u64,
}

fn split_extension(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(dot) if dot > 0 => (&name[..dot], &name[dot..]),
        _ => (name, ""),
    }
}

/// `stem` without a trailing `__dup<n>` and that `n` (R-I26: the app's own collision suffix).
fn strip_dup(stem: &str) -> (&str, Option<u32>) {
    if let Some(at) = stem.rfind("__dup") {
        let digits = &stem[at + 5..];
        if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
            return (&stem[..at], digits.parse().ok());
        }
    }
    (stem, None)
}

fn base_name(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

/// R-I26: the camera file name without directories and without `__dup<n>`, upper-cased,
/// extension included. Camera re-send suffixes (`_N`) are the adapter's business (H7).
pub fn file_key(name: &str) -> String {
    let (stem, extension) = split_extension(base_name(name));
    let (stem, _) = strip_dup(stem);
    format!("{stem}{extension}").to_uppercase()
}

/// The file key without its extension (R-I26 rule (2): RAW + JPEG of one shot).
pub fn stem_key(file_key: &str) -> String {
    split_extension(file_key).0.to_owned()
}

/// R-I26 association order: (1) same key and SHA-256, (2) same stem within the window,
/// (3) FIFO over the windowed candidates still expecting a file, (4) none.
pub fn associate(
    file_key: &str,
    stem_key: &str,
    sha256: &str,
    completed_at: i64,
    prior: &[PriorDelivery],
    candidates: &[FifoCandidate],
) -> AssociationDecision {
    let from_prior = |p: &PriorDelivery, association| AssociationDecision {
        exposure_id: p.exposure_id,
        association,
        session_id: p.session_id,
        ambiguous_with: Vec::new(),
    };
    if let Some(p) = prior
        .iter()
        .find(|p| p.file_key == file_key && p.sha256 == sha256)
    {
        return from_prior(p, Association::Filename);
    }
    if let Some(p) = prior
        .iter()
        .find(|p| p.stem_key == stem_key && (completed_at - p.completed_at).abs() <= FIFO_WINDOW_MS)
    {
        return from_prior(p, Association::Stem);
    }
    let mut open: Vec<&FifoCandidate> = candidates
        .iter()
        .filter(|c| c.exposed_at <= completed_at && completed_at - c.exposed_at <= FIFO_WINDOW_MS)
        .filter(|c| {
            let received = prior
                .iter()
                .filter(|p| {
                    p.exposure_id == Some(c.exposure_id) && p.outcome != DeliveryOutcome::Duplicate
                })
                .count();
            received < c.expected_files.unwrap_or(1) as usize
        })
        .collect();
    open.sort_by_key(|c| (c.exposed_at, c.exposure_id));
    let Some(pick) = open.first() else {
        return AssociationDecision {
            exposure_id: None,
            association: Association::None,
            session_id: None,
            ambiguous_with: Vec::new(),
        };
    };
    let mut ambiguous_with = Vec::new();
    for c in &open {
        if c.session_id != pick.session_id && !ambiguous_with.contains(&c.session_id) {
            ambiguous_with.push(c.session_id);
        }
    }
    AssociationDecision {
        exposure_id: Some(pick.exposure_id),
        association: Association::Fifo,
        // Every candidate agreeing gives their common session, which is the pick's; when they
        // differ the pick decides (R-I26).
        session_id: pick.session_id,
        ambiguous_with,
    }
}

/// MC-33: a shot is complete only when an expected count is known and met.
pub fn is_complete(expected_files: Option<u32>, non_duplicate_count: u32) -> bool {
    expected_files.is_some_and(|n| non_duplicate_count >= n)
}

/// R-I27: an identical earlier copy (same key and SHA-256) → duplicate; same key, other bytes →
/// a new photo under the lowest `__dupN` no earlier photo of that key carries; else a photo.
pub fn duplicate_decision(
    file_key: &str,
    sha256: &str,
    prior: &[PriorDelivery],
) -> DuplicateDecision {
    let same_key: Vec<&PriorDelivery> = prior.iter().filter(|p| p.file_key == file_key).collect();
    if let Some(p) = same_key.iter().find(|p| p.sha256 == sha256) {
        return DuplicateDecision::Duplicate { of: p.id };
    }
    if same_key.is_empty() {
        return DuplicateDecision::Photo;
    }
    let used: Vec<u32> = same_key
        .iter()
        .filter(|p| p.outcome != DeliveryOutcome::Duplicate)
        .filter_map(|p| strip_dup(split_extension(base_name(&p.final_name)).0).1)
        .collect();
    let n = (1..).find(|n| !used.contains(n)).unwrap_or(1);
    DuplicateDecision::RenamedPhoto { n }
}

/// `DSC01234.ARW`, 1 → `DSC01234__dup1.ARW`.
pub fn dup_name(camera_filename: &str, n: u32) -> String {
    let (stem, extension) = split_extension(base_name(camera_filename));
    format!("{stem}__dup{n}{extension}")
}

/// R-I28: `<capture root>/<folder key>/`, `_Unassigned` when there is no session.
pub fn target_folder(capture_root: &Path, folder_key: Option<&str>) -> PathBuf {
    capture_root.join(folder_key.unwrap_or("_Unassigned"))
}

/// R-I29: `expecting` exposures older than 60 s and newer than 30 min (both bounds exclusive).
pub fn pending_count(exposures: &[(ExposureStatus, i64)], now_ms: i64) -> usize {
    exposures
        .iter()
        .filter(|(status, exposed_at)| {
            let age = now_ms - exposed_at;
            *status == ExposureStatus::Expecting
                && age > WATCHDOG_MIN_AGE_MS
                && age < WATCHDOG_MAX_AGE_MS
        })
        .count()
}

/// The lab's `seq_of`: `_<1–2 digits>` stripped (here only), then `<prefix><4+ digits>`.
pub fn seq_of(stem: &str) -> Option<(String, u64)> {
    let mut s = stem;
    if let Some(at) = s.rfind('_') {
        let tail = &s[at + 1..];
        if (1..=2).contains(&tail.len()) && tail.bytes().all(|b| b.is_ascii_digit()) {
            s = &s[..at];
        }
    }
    let digits = s.bytes().rev().take_while(u8::is_ascii_digit).count();
    if digits < 4 {
        return None;
    }
    let (prefix, number) = s.split_at(s.len() - digits);
    Some((prefix.to_owned(), number.parse().ok()?))
}

/// MC-33 / R-I29: gaps in each numbering prefix (prefixes in first-seen order).
pub fn sequence_gaps(stems: &[&str]) -> Vec<Gap> {
    let mut by_prefix: Vec<(String, Vec<u64>)> = Vec::new();
    for stem in stems {
        let Some((prefix, number)) = seq_of(stem) else {
            continue;
        };
        match by_prefix.iter_mut().find(|(p, _)| *p == prefix) {
            Some((_, numbers)) => {
                if numbers.iter().min().is_some_and(|first| number < *first) {
                    numbers.clear(); // camera counter restarted
                }
                numbers.push(number);
            }
            None => by_prefix.push((prefix, vec![number])),
        }
    }
    let mut gaps = Vec::new();
    for (prefix, mut numbers) in by_prefix {
        numbers.sort_unstable();
        numbers.dedup();
        for pair in numbers.windows(2) {
            if pair[1] - pair[0] > 1 {
                gaps.push(Gap {
                    prefix: prefix.clone(),
                    after: pair[0],
                    before: pair[1],
                    missing: pair[1] - pair[0] - 1,
                });
            }
        }
    }
    gaps
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exposure(id: i64, at: i64, session: Option<i64>, expected: Option<u32>) -> FifoCandidate {
        FifoCandidate {
            exposure_id: id,
            exposed_at: at,
            session_id: session,
            expected_files: expected,
        }
    }

    /// Feeds `files` one by one, every call given all `exposures` and the decisions so far as
    /// `photo` priors; returns each decision.
    fn run(exposures: &[FifoCandidate], files: &[(&str, &str, i64)]) -> Vec<AssociationDecision> {
        let mut prior = Vec::new();
        let mut decisions = Vec::new();
        for (i, (name, sha, at)) in files.iter().enumerate() {
            let key = file_key(name);
            let stem = stem_key(&key);
            let decision = associate(&key, &stem, sha, *at, &prior, exposures);
            prior.push(PriorDelivery {
                id: i as i64 + 1,
                file_key: key,
                stem_key: stem,
                sha256: (*sha).into(),
                exposure_id: decision.exposure_id,
                session_id: decision.session_id,
                outcome: DeliveryOutcome::Photo,
                completed_at: *at,
                final_name: (*name).into(),
            });
            decisions.push(decision);
        }
        decisions
    }

    fn picks(decisions: &[AssociationDecision]) -> Vec<(Option<i64>, Association)> {
        decisions
            .iter()
            .map(|d| (d.exposure_id, d.association))
            .collect()
    }

    #[test]
    fn mc33_file_key_and_stem() {
        assert_eq!(file_key("DSC01234.ARW"), "DSC01234.ARW");
        assert_eq!(file_key("dsc01234.arw"), "DSC01234.ARW");
        assert_eq!(file_key("DSC01234__dup2.ARW"), "DSC01234.ARW");
        assert_eq!(file_key("DSC01234_1.ARW"), "DSC01234_1.ARW");
        assert_eq!(file_key("sub/DSC01234__dup1.ARW"), "DSC01234.ARW");
    }

    #[test]
    fn mc33_sequence_gaps() {
        assert_eq!(
            sequence_gaps(&["DSC00001", "DSC00002", "DSC00004", "DSC00005_1", "IMG"]),
            vec![Gap {
                prefix: "DSC".into(),
                after: 2,
                before: 4,
                missing: 1
            }]
        );
    }

    #[test]
    fn mc33_one_exposure_one_file_is_provisional() {
        // e1 is camera-initiated: a shot like any other, not an orphan.
        let exposures = [exposure(1, 0, Some(7), None)];
        let decision = associate("DSC00001.ARW", "DSC00001", "h1", 6_000, &[], &exposures);
        assert_eq!(
            decision,
            AssociationDecision {
                exposure_id: Some(1),
                association: Association::Fifo,
                session_id: Some(7),
                ambiguous_with: vec![]
            }
        );
        assert!(!is_complete(None, 1));
    }

    #[test]
    fn mc33_expected_count_completes() {
        assert!(is_complete(Some(1), 1));
        assert!(!is_complete(Some(2), 1));
        assert!(is_complete(Some(2), 2));
        assert!(!is_complete(None, 5));
    }

    #[test]
    fn mc33_raw_and_jpeg_grouped_by_stem() {
        let exposures = [
            exposure(1, 0, Some(7), None),
            exposure(2, 2_000, Some(7), None),
        ];
        let decisions = run(
            &exposures,
            &[
                ("DSC00001.JPG", "a", 3_000),
                ("DSC00001.ARW", "b", 8_000),
                ("DSC00002.JPG", "c", 9_000),
                ("DSC00002.ARW", "d", 14_000),
            ],
        );
        assert_eq!(
            picks(&decisions),
            vec![
                (Some(1), Association::Fifo),
                (Some(1), Association::Stem),
                (Some(2), Association::Fifo),
                (Some(2), Association::Stem)
            ]
        );
    }

    #[test]
    fn mc33_burst_with_lagging_uploads_in_order() {
        let exposures: Vec<_> = (0..5)
            .map(|i| exposure(i + 1, i * 1_500, Some(7), None))
            .collect();
        let decisions = run(
            &exposures,
            &[
                ("DSC00010.ARW", "a", 8_000),
                ("DSC00011.ARW", "b", 14_000),
                ("DSC00012.ARW", "c", 20_000),
                ("DSC00013.ARW", "d", 26_000),
                ("DSC00014.ARW", "e", 32_000),
            ],
        );
        assert_eq!(
            decisions.iter().map(|d| d.exposure_id).collect::<Vec<_>>(),
            vec![Some(1), Some(2), Some(3), Some(4), Some(5)]
        );
    }

    #[test]
    fn mc33_bracketing_with_expected_files() {
        let exposures = [
            exposure(1, 0, Some(7), Some(3)),
            exposure(2, 5_000, Some(7), None),
        ];
        let decisions = run(
            &exposures,
            &[
                ("DSC00100.ARW", "a", 10_000),
                ("DSC00101.ARW", "b", 11_000),
                ("DSC00102.ARW", "c", 12_000),
                ("DSC00103.ARW", "d", 13_000),
            ],
        );
        assert_eq!(
            picks(&decisions),
            vec![
                (Some(1), Association::Fifo),
                (Some(1), Association::Fifo),
                (Some(1), Association::Fifo),
                (Some(2), Association::Fifo)
            ]
        );
    }

    #[test]
    fn mc33_file_before_any_exposure_is_unassociated() {
        let exposures = [exposure(1, 5_000, Some(7), None)];
        let decision = associate("DSC00001.ARW", "DSC00001", "a", 1_000, &[], &exposures);
        assert_eq!(decision.exposure_id, None);
        assert_eq!(decision.association, Association::None);
    }

    #[test]
    fn mc33_fifo_window_bounds() {
        let both = [
            exposure(1, 0, Some(7), None),
            exposure(2, 400_000, Some(7), None),
        ];
        let only_e1 = [exposure(1, 0, Some(7), None)];
        let at = |t, exposures: &[FifoCandidate]| {
            associate("X.ARW", "X", "h", t, &[], exposures).exposure_id
        };
        assert_eq!(at(405_000, &both), Some(2));
        assert_eq!(at(200_000, &only_e1), Some(1));
        assert_eq!(at(300_000, &only_e1), Some(1));
        assert_eq!(at(300_001, &only_e1), None);
        assert_eq!(at(0, &only_e1), Some(1));
    }

    #[test]
    fn mc33_candidate_set_decides_the_session() {
        let (s, t) = (10, 11);
        let decide = |second: Option<i64>| {
            associate(
                "X.ARW",
                "X",
                "h",
                20_000,
                &[],
                &[
                    exposure(1, 0, Some(s), None),
                    exposure(2, 10_000, second, None),
                ],
            )
        };
        assert_eq!(
            decide(Some(s)),
            AssociationDecision {
                exposure_id: Some(1),
                association: Association::Fifo,
                session_id: Some(s),
                ambiguous_with: vec![]
            }
        );
        assert_eq!(
            decide(Some(t)),
            AssociationDecision {
                exposure_id: Some(1),
                association: Association::Fifo,
                session_id: Some(s),
                ambiguous_with: vec![Some(t)]
            }
        );
        assert_eq!(decide(None).ambiguous_with, vec![None]);
    }

    #[test]
    fn mc33_pending_count_bounds() {
        let now = 10_000_000;
        let expecting = |age: i64| (ExposureStatus::Expecting, now - age);
        let exposures = [
            expecting(59_000),
            expecting(60_000),
            expecting(60_001),
            expecting(29 * 60_000),
            expecting(30 * 60_000),
            expecting(30 * 60_000 + 1_000),
            (ExposureStatus::Received, now - 5 * 60_000),
        ];
        assert_eq!(pending_count(&exposures, now), 2);
    }
}
