use psm_core::ports::editing::RenderCompletion;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::Path,
    time::{Duration, Instant, SystemTime},
};

#[derive(Clone)]
pub struct RenderEntry {
    pub completion: RenderCompletion,
    pub issued_mono: Instant,
    pub attempts: u8,
}

pub struct PendingRender {
    pub raw: String,
    pub purpose: psm_core::domain::editing::RenderPurpose,
    pub generation: i64,
    pub tracking_job_id: i64,
}

#[derive(Default)]
pub struct RenderRegistry {
    pub jobs: HashMap<String, RenderEntry>,
    pub busy: HashSet<String>,
    pub pending: VecDeque<PendingRender>,
}

pub fn base_name(raw: &str) -> String {
    Path::new(raw)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase()
}

pub fn accept_output(output_path: &Path, issued_at: SystemTime) -> bool {
    output_path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .is_some_and(|mtime| {
            mtime
                .checked_add(Duration::from_secs(2))
                .is_some_and(|v| v >= issued_at)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn mc16_output_is_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.jpg");
        assert!(!accept_output(&path, SystemTime::now()));
        fs::write(&path, b"new").unwrap();
        assert!(accept_output(&path, SystemTime::now()));
        assert!(!accept_output(
            &path,
            SystemTime::now() + Duration::from_secs(5)
        ));
    }
}
