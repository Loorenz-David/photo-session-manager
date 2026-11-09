//! Files deleted outside Session Manager (book changelog 43, P60): the "missing file" decision and
//! the render-request eligibility that keeps such photos, current previews and the take-back's hold
//! away from Capture One.
use crate::{domain::photo::Integrity, mechanisms::staleness::is_stale};
use std::{io, path::Path};

/// Whether the RAW at `path` is gone: nothing there, or something that is not a file. Any other
/// read error (permissions, an I/O fault) is not a decision and is handed back.
pub fn file_missing(path: &Path) -> io::Result<bool> {
    match path.metadata() {
        Ok(meta) => Ok(!meta.is_file()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error),
    }
}

/// What a photo's recorded integrity becomes after its RAW was looked at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingDecision {
    /// Unchanged.
    Keep,
    /// The file is gone: the photo is missing (`Integrity::RawMissing`).
    Missing,
    /// The file is back where it was: verify it again (hash), the photo returns.
    Returned,
}

pub fn missing_decision(file_missing: bool, recorded: Integrity) -> MissingDecision {
    match (file_missing, recorded) {
        (true, Integrity::RawMissing) => MissingDecision::Keep,
        (true, _) => MissingDecision::Missing,
        (false, Integrity::RawMissing) => MissingDecision::Returned,
        (false, _) => MissingDecision::Keep,
    }
}

/// Whether a preview render may be requested for a RAW now, in check order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderEligibility {
    Request,
    /// The RAW is gone: never requested.
    FileMissing,
    /// The preview is current (no adjustment since the last render): not requested again.
    Current,
    /// "Take the camera back" holds Capture One: the request waits (it neither fails nor counts).
    Held,
}

pub fn render_eligibility(
    file_missing: bool,
    adjusted_gen: i64,
    rendered_gen: i64,
    take_back_hold: bool,
) -> RenderEligibility {
    if file_missing {
        RenderEligibility::FileMissing
    } else if !is_stale(adjusted_gen, rendered_gen) {
        RenderEligibility::Current
    } else if take_back_hold {
        RenderEligibility::Held
    } else {
        RenderEligibility::Request
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_decision() {
        let dir = std::env::temp_dir().join(format!("psm-missing-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let raw = dir.join("DSC0001.ARW");
        assert!(file_missing(&raw).unwrap());
        std::fs::write(&raw, b"raw").unwrap();
        assert!(!file_missing(&raw).unwrap());
        assert!(file_missing(&dir).unwrap());
        assert!(file_missing(&dir.join("gone/DSC0002.ARW")).unwrap());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_decision_transitions() {
        use Integrity::*;
        assert_eq!(missing_decision(true, Ok), MissingDecision::Missing);
        assert_eq!(missing_decision(true, RawChanged), MissingDecision::Missing);
        assert_eq!(missing_decision(true, RawMissing), MissingDecision::Keep);
        assert_eq!(
            missing_decision(false, RawMissing),
            MissingDecision::Returned
        );
        assert_eq!(missing_decision(false, Ok), MissingDecision::Keep);
        assert_eq!(missing_decision(false, RawChanged), MissingDecision::Keep);
    }

    #[test]
    fn render_eligibility_in_check_order() {
        use RenderEligibility::*;
        assert_eq!(render_eligibility(false, 2, 1, false), Request);
        assert_eq!(render_eligibility(false, 1, 0, true), Held);
        assert_eq!(render_eligibility(false, 2, 2, false), Current);
        assert_eq!(render_eligibility(false, 2, 2, true), Current);
        assert_eq!(render_eligibility(true, 2, 1, false), FileMissing);
        assert_eq!(render_eligibility(true, 2, 2, true), FileMissing);
    }
}
