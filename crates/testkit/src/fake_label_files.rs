//! `FakeLabelFiles` (`master_plan_labels.md` §4 Files): the label files of a `TestApp`. It saves
//! through a real `DiskLabelFiles` on the test's own home and previews folder, but **never runs
//! `sips` or `open`**: its `raster` writes a fixed 1 × 1 PNG and records the bytes it was given,
//! its `open` records the path. Each of `raster`, `save` and `open` can be told to fail once.

use psm_core::{Error, Result, ports::labels::LabelFiles};
use psm_infra::label_files::DiskLabelFiles;
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Mutex,
};

/// A 1 × 1 transparent PNG.
pub const ONE_PIXEL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

#[derive(Default)]
struct State {
    rasters: Vec<Vec<u8>>,
    opened: Vec<PathBuf>,
    fail_raster: bool,
    fail_save: Option<String>,
    fail_open: bool,
}

pub struct FakeLabelFiles {
    inner: DiskLabelFiles,
    state: Mutex<State>,
}

impl FakeLabelFiles {
    /// `previews_root` = `<data>/previews`; `labels_dir` = the test home's labels folder.
    pub fn new(previews_root: PathBuf, labels_dir: PathBuf) -> Self {
        Self {
            inner: DiskLabelFiles::new(previews_root, labels_dir),
            state: Mutex::new(State::default()),
        }
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The next `raster` fails (nothing written).
    pub fn fail_raster_next(&self) {
        self.state().fail_raster = true;
    }

    /// The next `save` answers `Error::Io(io::Error::other(reason))` (nothing written).
    pub fn fail_save_next(&self, reason: &str) {
        self.state().fail_save = Some(reason.to_owned());
    }

    /// The next `open` fails.
    pub fn fail_open_next(&self) {
        self.state().fail_open = true;
    }

    /// The bytes of every `raster` call that wrote a picture, in order.
    pub fn rasters(&self) -> Vec<Vec<u8>> {
        self.state().rasters.clone()
    }

    /// The paths of every `open` call that succeeded, in order.
    pub fn opened(&self) -> Vec<PathBuf> {
        self.state().opened.clone()
    }

    /// The labels folder.
    pub fn labels_dir(&self) -> PathBuf {
        self.inner.labels_dir().to_path_buf()
    }

    /// `<previews>/labels/`.
    pub fn previews_dir(&self) -> PathBuf {
        self.inner.previews_dir()
    }
}

impl LabelFiles for FakeLabelFiles {
    fn raster(&self, pdf: &[u8], name: &str) -> Result<PathBuf> {
        let mut state = self.state();
        if std::mem::take(&mut state.fail_raster) {
            return Err(Error::Provider("scripted raster failure".into()));
        }
        let dir = self.inner.previews_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{name}.png"));
        fs::write(&path, ONE_PIXEL_PNG)?;
        state.rasters.push(pdf.to_vec());
        Ok(path)
    }

    fn clear_previews(&self) -> Result<()> {
        self.inner.clear_previews()
    }

    fn save(&self, filename: &str, bytes: &[u8]) -> Result<PathBuf> {
        if let Some(reason) = self.state().fail_save.take() {
            return Err(Error::Io(io::Error::other(reason)));
        }
        self.inner.save(filename, bytes)
    }

    fn open(&self, path: &Path) -> Result<()> {
        let mut state = self.state();
        if std::mem::take(&mut state.fail_open) {
            return Err(Error::Provider("scripted open failure".into()));
        }
        state.opened.push(path.to_path_buf());
        Ok(())
    }
}
