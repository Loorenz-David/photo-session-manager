//! MC-A5: the research copy of a photo and the only code that can make one.
//!
//! A [`ResearchImage`] has private fields and no public constructor: outside this module nothing
//! can build one from a path or from bytes, so a RAW or a master can never reach a provider port
//! (rule 11). [`ResearchImages::render`] makes one from an **eligible** photo's adjusted preview,
//! framed by its current composition (10D MC-27), through `Imaging::render_research`.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use psm_core::{
    Result,
    domain::{
        composition::{CompositionValues, Rect},
        photo::Classification,
    },
    ports::{images::Imaging, store::StoreTx},
};
use psm_infra::sqlite::store::{SqliteStore, SqliteTx};
use rusqlite::{OptionalExtension, params};

use crate::{
    files::ResearchFiles,
    store::{RunRow, db},
};

/// A reduced, metadata-free research copy of one photo, stored under its SHA-256.
///
/// It cannot be built outside the research crate's renderer:
///
/// ```compile_fail
/// let image = psm_research::images::ResearchImage {
///     photo_id: 1,
///     sha256: String::new(),
///     long_edge: 1024,
///     path: std::path::PathBuf::from("/tmp/raw.ARW"),
///     bytes_len: 0,
///     source_gen: 1,
///     composition_rev: 1,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchImage {
    photo_id: i64,
    sha256: String,
    long_edge: u32,
    path: PathBuf,
    bytes_len: u64,
    source_gen: i64,
    composition_rev: i64,
}

impl ResearchImage {
    pub fn photo_id(&self) -> i64 {
        self.photo_id
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn long_edge(&self) -> u32 {
        self.long_edge
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn bytes_len(&self) -> u64 {
        self.bytes_len
    }
    pub fn source_gen(&self) -> i64 {
        self.source_gen
    }
    pub fn composition_rev(&self) -> i64 {
        self.composition_rev
    }
    /// The JPEG bytes an adapter sends.
    pub fn bytes(&self) -> std::io::Result<Vec<u8>> {
        std::fs::read(&self.path)
    }
}

/// Why a photo has no research copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoSource {
    /// Not in the run's session (or its item), or classified `deleted`, or gone.
    NotEligible,
    /// No adjusted preview for the current `adjusted_gen` yet (the camera preview is never used).
    NoAdjustedPreview,
    /// The render or the store failed.
    Failed(String),
}

impl std::fmt::Display for NoSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoSource::NotEligible => write!(f, "the photo may not be used for research"),
            NoSource::NoAdjustedPreview => write!(f, "the photo has no adjusted preview yet"),
            NoSource::Failed(why) => write!(f, "{why}"),
        }
    }
}

/// MC-A5 eligibility, read inside the caller's transaction: the photo is in the run's session,
/// that session's item is the run's item, and it is `main`, `condition` or `unreviewed`.
pub(crate) fn eligible(tx: &SqliteTx<'_>, run: &RunRow, photo_id: i64) -> Result<bool> {
    let Ok(photo) = tx.photo(photo_id) else {
        return Ok(false);
    };
    if photo.session_id != run.session_id || photo.classification == Classification::Deleted {
        return Ok(false);
    }
    let session = tx.session(photo.session_id)?;
    Ok(session.item_id == Some(run.item_id))
}

/// The adjusted preview of the photo's current `adjusted_gen`, with that generation; `None` when
/// the photo has none (or only a preview of an older generation).
pub(crate) fn source(tx: &SqliteTx<'_>, photo_id: i64) -> Result<Option<(PathBuf, i64)>> {
    let row: Option<(Option<String>, i64, i64)> = tx
        .conn()
        .query_row(
            "SELECT p.adjusted_preview_path, COALESCE(r.adjusted_gen,0), COALESCE(r.rendered_gen,0) \
             FROM photos p LEFT JOIN raw_state r ON r.raw_path=p.raw_path WHERE p.id=?1",
            params![photo_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(db)?;
    Ok(match row {
        Some((Some(path), adjusted, rendered)) if rendered >= adjusted => {
            let path = PathBuf::from(path);
            path.is_file().then_some((path, adjusted))
        }
        _ => None,
    })
}

/// The composition values a research copy is framed with; a composition with no rect yet frames
/// the whole preview.
pub(crate) fn values_of(composition: &psm_core::domain::composition::Composition) -> CompositionValues {
    composition.values().unwrap_or(CompositionValues {
        kind: composition.kind,
        shape: composition.shape,
        orientation: composition.orientation,
        quarter_turns: composition.quarter_turns,
        flip: composition.flip,
        straighten: composition.straighten,
        rect: Rect {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        },
    })
}

type CopyKey = (i64, i64, i64, i64, u32);

/// The renderer of research copies: one stored copy per run and
/// `(photo, source generation, composition revision, long edge)`.
pub struct ResearchImages {
    store: Arc<SqliteStore>,
    imaging: Arc<dyn Imaging>,
    files: ResearchFiles,
    copies: Mutex<HashMap<CopyKey, ResearchImage>>,
}

impl ResearchImages {
    pub fn new(store: Arc<SqliteStore>, imaging: Arc<dyn Imaging>, files: ResearchFiles) -> Self {
        Self {
            store,
            imaging,
            files,
            copies: Mutex::new(HashMap::new()),
        }
    }

    /// The research copy of `photo_id` for run `run_id` at `long_edge`, or why there is none.
    pub fn render(
        &self,
        photo_id: i64,
        run_id: i64,
        long_edge: u32,
    ) -> std::result::Result<ResearchImage, NoSource> {
        let failed = |e: psm_core::Error| NoSource::Failed(e.to_string());
        let found = self
            .store
            .research_transaction(|tx| {
                let Some(run) = crate::store::run(tx.conn(), run_id)? else {
                    return Ok(Err(NoSource::NotEligible));
                };
                if !eligible(tx, &run, photo_id)? {
                    return Ok(Err(NoSource::NotEligible));
                }
                let Some((preview, generation)) = source(tx, photo_id)? else {
                    return Ok(Err(NoSource::NoAdjustedPreview));
                };
                let composition = tx.composition(photo_id)?;
                Ok(Ok((preview, generation, composition)))
            })
            .map_err(failed)??;
        let (preview, generation, composition) = found;
        let key = (
            run_id,
            photo_id,
            generation,
            composition.revision,
            long_edge,
        );
        if let Some(copy) = self.copies.lock().unwrap().get(&key)
            && copy.path.is_file()
        {
            return Ok(copy.clone());
        }
        let temp = self.files.temp_path().map_err(failed)?;
        let rendered =
            self.imaging
                .render_research(&preview, &values_of(&composition), long_edge, &temp);
        if let Err(error) = rendered {
            let _ = std::fs::remove_file(&temp);
            return Err(failed(error));
        }
        let (sha256, path, bytes_len) = self.files.commit_temp(&temp).map_err(failed)?;
        let copy = ResearchImage {
            photo_id,
            sha256,
            long_edge,
            path,
            bytes_len,
            source_gen: generation,
            composition_rev: composition.revision,
        };
        self.copies.lock().unwrap().insert(key, copy.clone());
        Ok(copy)
    }

    pub fn files(&self) -> &ResearchFiles {
        &self.files
    }
}

/// Classifications research may use at all (MC-A5).
pub fn usable(classification: Classification) -> bool {
    matches!(
        classification,
        Classification::Main | Classification::Condition | Classification::Unreviewed
    )
}
