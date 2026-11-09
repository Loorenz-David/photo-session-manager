use psm_core::{domain::editing::RenderPurpose, ports::editing::RenderCompletion};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

#[derive(Clone)]
pub struct RenderEntry {
    pub completion: RenderCompletion,
    pub issued_mono: Instant,
    pub attempts: u8,
    /// Re-sent once after its export came out in the wrong format (10F §5).
    pub recipe_retried: bool,
}

pub struct PendingRender {
    pub raw: String,
    pub purpose: RenderPurpose,
    pub generation: i64,
    pub tracking_job_id: i64,
    pub new_arrival: bool,
}

#[derive(Default)]
pub struct RenderRegistry {
    pub jobs: HashMap<String, RenderEntry>,
    pub busy: HashSet<String>,
    pub pending: VecDeque<PendingRender>,
    /// The photo open on the stage or in the editor (P58).
    pub focused_raw: Option<String>,
    /// Queued requests Capture One refused when their turn came: (tracking job, error).
    pub refused: Vec<(i64, String)>,
    /// The recipes read back and repaired in the current render batch (10F §5).
    pub recipes_checked: HashSet<RenderPurpose>,
}

/// The `busy` entry while a preview is with Capture One: previews go one at a time (P58).
pub const PREVIEW_LANE: &str = "\0preview-lane";

impl RenderRegistry {
    /// Whether a render can be sent now: no render of the same RAW is out (outputs are found by
    /// the RAW's base name, MC-16) and, for a preview, no other preview is out.
    pub fn can_start(&self, raw: &str, purpose: RenderPurpose) -> bool {
        !self.busy.contains(&base_name(raw))
            && (purpose == RenderPurpose::Master || !self.busy.contains(PREVIEW_LANE))
    }
    pub fn claim(&mut self, raw: &str, purpose: RenderPurpose) {
        self.busy.insert(base_name(raw));
        if purpose == RenderPurpose::Preview {
            self.busy.insert(PREVIEW_LANE.into());
        }
    }
    pub fn release(&mut self, raw: &str, purpose: RenderPurpose) {
        self.busy.remove(&base_name(raw));
        if purpose == RenderPurpose::Preview {
            self.busy.remove(PREVIEW_LANE);
        }
    }
    /// Renders that are out, or being sent.
    pub fn out(&self) -> usize {
        self.jobs
            .len()
            .max(self.busy.iter().filter(|key| *key != PREVIEW_LANE).count())
    }
    /// The next queued render that can be sent (P58, book changelog 39). A master goes as soon as
    /// its RAW is free, as before. Previews go one at a time: the focused photo's, then the newest
    /// arrival's (the most recently queued first), then the rest in the order they were queued.
    pub fn pop_next(&mut self) -> Option<PendingRender> {
        let ready = |p: &PendingRender| self.can_start(&p.raw, p.purpose);
        let preview = |p: &PendingRender| p.purpose == RenderPurpose::Preview && ready(p);
        let index = self
            .pending
            .iter()
            .position(|p| p.purpose == RenderPurpose::Master && ready(p))
            .or_else(|| {
                self.pending
                    .iter()
                    .position(|p| preview(p) && self.focused_raw.as_deref() == Some(p.raw.as_str()))
            })
            .or_else(|| {
                self.pending
                    .iter()
                    .rposition(|p| preview(p) && p.new_arrival)
            })
            .or_else(|| self.pending.iter().position(preview))?;
        self.pending.remove(index)
    }
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
        .is_some_and(|mtime| fresh(mtime, issued_at))
}

fn fresh(mtime: SystemTime, issued_at: SystemTime) -> bool {
    mtime
        .checked_add(Duration::from_secs(2))
        .is_some_and(|v| v >= issued_at)
}

/// The file a render produced, from the path the processing-done hook reported (10A MC-16
/// §Completion, P56). The reported path when it was written after the request; otherwise Capture
/// One's "add suffix" copy beside it — `<stem> <n>.<ext>`, the hook still reporting the unsuffixed
/// path (MC-16; P52 F12: `DSC04979 14.tif` reported as `DSC04979.tif`) — the newest one written
/// after the request. One render per output base name is in flight at a time (MC-16 §Issuing), so
/// that copy is this render's. `None`: nothing written after the request.
pub fn resolve_output(reported: &Path, issued_at: SystemTime) -> Option<PathBuf> {
    if accept_output(reported, issued_at) {
        return Some(reported.to_path_buf());
    }
    let stem = reported.file_stem()?.to_str()?;
    let ext = reported.extension()?.to_str()?;
    let prefix = format!("{stem} ");
    let mut best: Option<(SystemTime, u64, PathBuf)> = None;
    for entry in fs::read_dir(reported.parent()?).ok()?.flatten() {
        let path = entry.path();
        let (Some(name_stem), Some(name_ext)) = (
            path.file_stem().and_then(|s| s.to_str()),
            path.extension().and_then(|s| s.to_str()),
        ) else {
            continue;
        };
        if !name_ext.eq_ignore_ascii_case(ext) {
            continue;
        }
        let Some(counter) = name_stem
            .strip_prefix(&prefix)
            .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|n| n.parse::<u64>().ok())
        else {
            continue;
        };
        let Some(mtime) = entry.metadata().ok().and_then(|m| m.modified().ok()) else {
            continue;
        };
        if fresh(mtime, issued_at)
            && best
                .as_ref()
                .is_none_or(|(at, n, _)| (mtime, counter) > (*at, *n))
        {
            best = Some((mtime, counter, path));
        }
    }
    best.map(|(_, _, path)| path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn focused_then_newest_arrivals_then_oldest_backlog() {
        let mut queue = RenderRegistry::default();
        let add = |queue: &mut RenderRegistry, id: &str, purpose, fresh| {
            queue.pending.push_back(PendingRender {
                raw: format!("/c/{id}.ARW"),
                purpose,
                generation: 1,
                tracking_job_id: 1,
                new_arrival: fresh,
            })
        };
        let next = |queue: &mut RenderRegistry| queue.pop_next().map(|p| p.raw);
        let raw = |id: &str| Some(format!("/c/{id}.ARW"));
        add(&mut queue, "old-1", RenderPurpose::Preview, false);
        add(&mut queue, "old-2", RenderPurpose::Preview, false);
        add(&mut queue, "new-1", RenderPurpose::Preview, true);
        add(&mut queue, "new-2", RenderPurpose::Preview, true);
        add(&mut queue, "old-1", RenderPurpose::Master, false);
        queue.focused_raw = raw("old-2");
        // A master goes as soon as its RAW is free; a preview only while no preview is out.
        assert_eq!(next(&mut queue), raw("old-1"));
        queue.claim("/c/old-1.ARW", RenderPurpose::Master);
        assert_eq!(next(&mut queue), raw("old-2"));
        queue.claim("/c/old-2.ARW", RenderPurpose::Preview);
        assert_eq!(next(&mut queue), None);
        queue.release("/c/old-2.ARW", RenderPurpose::Preview);
        queue.focused_raw = None;
        assert_eq!(next(&mut queue), raw("new-2"));
        assert_eq!(next(&mut queue), raw("new-1"));
        // old-1's preview waits for its master: same RAW.
        assert_eq!(next(&mut queue), None);
        queue.release("/c/old-1.ARW", RenderPurpose::Master);
        assert_eq!(next(&mut queue), raw("old-1"));
    }
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

    fn write_at(path: &Path, at: SystemTime) {
        fs::write(path, b"x").unwrap();
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(at))
            .unwrap();
    }

    #[test]
    fn p56_completion_resolves_capture_ones_suffixed_copy() {
        let dir = tempfile::tempdir().unwrap();
        let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let issued = t0 + Duration::from_secs(600);
        let reported = dir.path().join("DSC04979.tif");
        // The reported (unsuffixed) file is an old render; Capture One wrote ` 13` and ` 14` since.
        write_at(&reported, t0);
        write_at(&dir.path().join("DSC04979 2.tif"), t0);
        write_at(
            &dir.path().join("DSC04979 13.tif"),
            issued + Duration::from_secs(1),
        );
        write_at(
            &dir.path().join("DSC04979 14.tif"),
            issued + Duration::from_secs(3),
        );
        // Never another photo, another format or Capture One's temp files.
        write_at(
            &dir.path().join("DSC049791 15.tif"),
            issued + Duration::from_secs(9),
        );
        write_at(
            &dir.path().join("DSC04979 16.jpg"),
            issued + Duration::from_secs(9),
        );
        write_at(
            &dir.path().join("DSC04979 17.tif.tmp"),
            issued + Duration::from_secs(9),
        );
        write_at(
            &dir.path().join("DSC04979 x.tif"),
            issued + Duration::from_secs(9),
        );
        assert_eq!(
            resolve_output(&reported, issued),
            Some(dir.path().join("DSC04979 14.tif"))
        );
        // A fresh reported file wins (the `overwrite` recipe the adapter sets).
        write_at(&reported, issued + Duration::from_secs(1));
        assert_eq!(resolve_output(&reported, issued), Some(reported.clone()));
        // Nothing written after the request: no output.
        let later = issued + Duration::from_secs(60);
        assert_eq!(resolve_output(&reported, later), None);
        // The reported file may not exist at all.
        let missing = dir.path().join("DSC05000.jpg");
        write_at(&dir.path().join("DSC05000 1.jpg"), later);
        assert_eq!(
            resolve_output(&missing, later),
            Some(dir.path().join("DSC05000 1.jpg"))
        );
        assert_eq!(
            resolve_output(&dir.path().join("DSC06000.jpg"), later),
            None
        );
    }
}
