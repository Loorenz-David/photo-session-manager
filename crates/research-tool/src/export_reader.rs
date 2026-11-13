//! Reads what `report` and `replay` work on (R9·U1 task 1): an export of the app's research record
//! (MC-A11 layout, `master_plan_research.md` §6.7), as its folder or its zip, or a folder in the
//! benchmark's recording layout (R1·U1: `…/calls/<id>.json`, `…/images/<sha256>.jpg`).
//!
//! A zip is unpacked into a private temporary folder that is removed when the [`Opened`] value is
//! dropped. Nothing is ever written next to the export.

use serde_json::Value as Json;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::recording::call_files;

/// One run folder of an export: `<item id>-<folder key>/run-<id>/`.
#[derive(Debug, Clone)]
pub struct RunDir {
    pub dir: PathBuf,
    /// `<item id>-<folder key>`.
    pub item_dir: String,
    /// `run.json` as written (`{run, settings_snapshot, form_snapshot, input_snapshot, call_record}`).
    pub run: Json,
    pub steps: Vec<Json>,
    /// `writes.json`: the stage 7 output (`{writes, not_determined, ending}`) or `null`.
    pub writes: Json,
    pub feedback: Vec<Json>,
    /// Every `calls/<id>.json`, by call id.
    pub calls: Vec<Json>,
}

impl RunDir {
    pub fn id(&self) -> i64 {
        self.run["run"]["id"].as_i64().unwrap_or(0)
    }

    /// `run.json`'s `call_record` is "call record expired".
    pub fn expired(&self) -> bool {
        self.run["call_record"].as_str() == Some(psm_research::export::CALL_RECORD_EXPIRED)
    }

    /// The exported copy of a stored image, if the export holds it.
    pub fn image(&self, sha256: &str) -> Option<PathBuf> {
        let safe = !sha256.is_empty() && sha256.bytes().all(|b| b.is_ascii_hexdigit());
        let path = self.dir.join("images").join(format!("{sha256}.jpg"));
        (safe && path.is_file()).then_some(path)
    }
}

/// What a path holds.
#[derive(Debug)]
pub enum Contents {
    /// An export of the app's research record, runs in folder order.
    Export(Vec<RunDir>),
    /// A benchmark recording folder: every call record, in path order.
    Bench(Vec<Json>),
}

/// An opened export or recording; a zip's temporary folder lives as long as this value.
#[derive(Debug)]
pub struct Opened {
    pub contents: Contents,
    temp: Option<PathBuf>,
}

impl Drop for Opened {
    fn drop(&mut self) {
        if let Some(temp) = &self.temp {
            let _ = fs::remove_dir_all(temp);
        }
    }
}

impl Opened {
    pub fn runs(&self) -> &[RunDir] {
        match &self.contents {
            Contents::Export(runs) => runs,
            Contents::Bench(_) => &[],
        }
    }

    pub fn run(&self, id: i64) -> Option<&RunDir> {
        self.runs().iter().find(|r| r.id() == id)
    }
}

fn read_json(path: &Path) -> Result<Json, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn array(value: Json) -> Vec<Json> {
    match value {
        Json::Array(items) => items,
        _ => Vec::new(),
    }
}

/// Every `run.json` under `root`, sorted.
fn run_files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().is_some_and(|n| n == "run.json")
                && path
                    .parent()
                    .and_then(Path::file_name)
                    .is_some_and(|n| n.to_string_lossy().starts_with("run-"))
            {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

fn read_run(run_file: &Path) -> Result<RunDir, String> {
    let dir = run_file
        .parent()
        .ok_or_else(|| format!("{} has no folder", run_file.display()))?
        .to_path_buf();
    let item_dir = dir
        .parent()
        .and_then(Path::file_name)
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let optional = |name: &str| -> Result<Json, String> {
        let path = dir.join(name);
        if path.is_file() {
            read_json(&path)
        } else {
            Ok(Json::Null)
        }
    };
    let mut calls = Vec::new();
    for file in call_files(&dir) {
        calls.push(read_json(&file)?);
    }
    calls.sort_by_key(|c| c["id"].as_i64().unwrap_or(i64::MAX));
    Ok(RunDir {
        run: read_json(run_file)?,
        steps: array(optional("steps.json")?),
        writes: optional("writes.json")?,
        feedback: array(optional("feedback.json")?),
        calls,
        item_dir,
        dir,
    })
}

fn read_folder(root: &Path) -> Result<Contents, String> {
    let runs = run_files(root);
    if !runs.is_empty() {
        return runs
            .iter()
            .map(|f| read_run(f))
            .collect::<Result<Vec<_>, _>>()
            .map(Contents::Export);
    }
    let calls = call_files(root);
    if calls.is_empty() {
        return Err(format!(
            "{} holds neither a research export nor benchmark recordings",
            root.display()
        ));
    }
    calls
        .iter()
        .map(|f| read_json(f))
        .collect::<Result<Vec<_>, _>>()
        .map(Contents::Bench)
}

/// A fresh private folder under the system's temporary folder.
fn temp_dir() -> Result<PathBuf, String> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "psm-research-export-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).map_err(|e| format!("temporary folder: {e}"))?;
    Ok(dir)
}

/// Unpacks `zip` into `dest`; an entry whose name would leave `dest` is refused.
fn unzip(zip: &Path, dest: &Path) -> Result<(), String> {
    let file = fs::File::open(zip).map_err(|e| format!("cannot open {}: {e}", zip.display()))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("{}: {e}", zip.display()))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("{}: {e}", zip.display()))?;
        let Some(relative) = entry.enclosed_name() else {
            return Err(format!("{}: unsafe entry {}", zip.display(), entry.name()));
        };
        let target = dest.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out = fs::File::create(&target).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Opens an export folder, an export zip, or a benchmark recording folder.
pub fn open(path: &Path) -> Result<Opened, String> {
    if path.is_dir() {
        return Ok(Opened {
            contents: read_folder(path)?,
            temp: None,
        });
    }
    if !path.is_file() {
        return Err(format!("{} does not exist", path.display()));
    }
    let temp = temp_dir()?;
    let opened = unzip(path, &temp).and_then(|()| read_folder(&temp));
    match opened {
        Ok(contents) => Ok(Opened {
            contents,
            temp: Some(temp),
        }),
        Err(e) => {
            let _ = fs::remove_dir_all(&temp);
            Err(e)
        }
    }
}
