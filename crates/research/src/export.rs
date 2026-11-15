//! MC-A11 export: the research record of the chosen runs as one folder, then one zip.
//!
//! Layout (field names fixed in `master_plan_research.md` §6.7; read by the R9 tool):
//! `research-export-<local YYYYMMDD-HHMMSS>/<item id>-<folder key>/run-<id>/` with `run.json`,
//! `steps.json`, `writes.json`, `feedback.json`, `calls/<call id>.json`, `images/<sha256>.jpg`.
//! Exclusions are decided at export time: the image of a photo that is now `deleted` (or whose
//! row is gone) is not written and every request that referenced it says
//! `"withheld": "photo deleted"`. Before zipping, every written file is scanned for every value
//! of the secrets port, plain and URL-encoded; a hit deletes the partial folder, writes no zip and
//! names the file (fail closed).

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::{Datelike, TimeZone};
use psm_core::{Error, Result};
use serde_json::{Value as Json, json};

use crate::{
    api::ExportSelection,
    domain::Stage,
    retention::{DAY_MS, image_refs},
    rules::calls::redact,
    service::ResearchService,
    store::{self, CallRow, RunRow},
};

/// What `run.json` says when the run's call records were purged.
pub const CALL_RECORD_EXPIRED: &str = "call record expired";
/// What replaces the image reference of a photo deleted at export time.
pub const PHOTO_DELETED: &str = "photo deleted";

fn local(ms: i64) -> Option<chrono::DateTime<chrono::Local>> {
    chrono::Local.timestamp_millis_opt(ms).single()
}

/// The local date of `ms` as `YYYYMMDD`.
fn local_date(ms: i64) -> Option<i64> {
    local(ms)
        .map(|t| i64::from(t.year()) * 10_000 + i64::from(t.month()) * 100 + i64::from(t.day()))
}

fn parsed(text: Option<&str>) -> Json {
    text.and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or(Json::Null)
}

fn io(error: std::io::Error) -> Error {
    Error::InvalidData(format!("research export: {error}"))
}

fn write_json(path: &Path, value: &Json) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io)?;
    }
    let text =
        serde_json::to_string_pretty(value).map_err(|e| Error::InvalidData(e.to_string()))?;
    fs::write(path, text).map_err(io)
}

/// Replaces each `{"image_sha256": <withheld sha>}` in a recorded text by `{"withheld": "photo
/// deleted"}`; a text that references none is returned as recorded, byte for byte.
fn withhold(text: &str, withheld: &BTreeSet<String>) -> String {
    if !image_refs(text).iter().any(|sha| withheld.contains(sha)) {
        return text.to_owned();
    }
    fn walk(value: &mut Json, withheld: &BTreeSet<String>) {
        match value {
            Json::Object(map) => {
                let hit = map
                    .get("image_sha256")
                    .and_then(Json::as_str)
                    .is_some_and(|sha| withheld.contains(sha));
                if hit {
                    *value = json!({ "withheld": PHOTO_DELETED });
                } else {
                    map.values_mut().for_each(|inner| walk(inner, withheld));
                }
            }
            Json::Array(items) => items.iter_mut().for_each(|item| walk(item, withheld)),
            _ => {}
        }
    }
    let mut value: Json = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(_) => return text.to_owned(),
    };
    walk(&mut value, withheld);
    value.to_string()
}

/// Every file under `root`, as (path, path relative to `root` with `/` separators), sorted.
fn files_under(root: &Path) -> Result<Vec<(PathBuf, String)>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(io)? {
            let path = entry.map_err(io)?.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|e| Error::InvalidData(e.to_string()))?
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push((path, relative));
            }
        }
    }
    out.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(out)
}

/// Zips `folder` (entries prefixed with its own name) into `zip_path`, through a temporary file.
fn zip_folder(folder: &Path, zip_path: &Path) -> Result<()> {
    let name = folder
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temp = zip_path.with_extension("zip.partial");
    let file = fs::File::create(&temp).map_err(io)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (path, relative) in files_under(folder)? {
        zip.start_file(format!("{name}/{relative}"), options)
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        zip.write_all(&fs::read(&path).map_err(io)?).map_err(io)?;
    }
    zip.finish()
        .map_err(|e| Error::InvalidData(e.to_string()))?;
    fs::rename(&temp, zip_path).map_err(io)
}

impl ResearchService {
    fn selected_runs(&self, selection: &ExportSelection) -> Result<Vec<RunRow>> {
        let all = self
            .store
            .research_transaction(|tx| store::runs(tx.conn(), None))?;
        Ok(all
            .into_iter()
            .filter(|run| match selection {
                ExportSelection::Range { from, to } => run
                    .started_at
                    .and_then(local_date)
                    .is_some_and(|date| (*from..=*to).contains(&date)),
                ExportSelection::Items(ids) => ids.contains(&run.item_id),
            })
            .collect())
    }

    /// Writes one run's folder; returns nothing, the files are the result.
    fn export_run(&self, run: &RunRow, root: &Path, now_ms: i64, record_days: i64) -> Result<()> {
        let (folder_key, steps, calls, outbound, decide, feedback, deleted) =
            self.store.research_transaction(|tx| {
                let conn = tx.conn();
                let outbound = store::outbound(conn, Some(run.id))?;
                let mut deleted: BTreeSet<i64> = BTreeSet::new();
                for row in &outbound {
                    if store::photo_deleted(conn, row.photo_id)? {
                        deleted.insert(row.photo_id);
                    }
                }
                Ok((
                    store::folder_key(conn, run.item_id)?,
                    store::steps(conn, run.id)?,
                    store::calls(conn, Some(run.id))?,
                    outbound,
                    store::output(conn, run.id, Stage::Decide.as_str(), "decide")?,
                    store::feedback_since(
                        conn,
                        run.item_id,
                        run.started_at.unwrap_or(run.queued_at),
                    )?,
                    deleted,
                ))
            })?;
        let dir = root
            .join(format!("{}-{folder_key}", run.item_id))
            .join(format!("run-{}", run.id));
        fs::create_dir_all(&dir).map_err(io)?;
        let withheld: BTreeSet<String> = outbound
            .iter()
            .filter(|o| deleted.contains(&o.photo_id))
            .map(|o| o.sha256.clone())
            .collect();
        let call_ids: BTreeSet<i64> = calls.iter().map(|c| c.id).collect();
        let expired = outbound.iter().any(|o| !call_ids.contains(&o.call_id))
            || (calls.is_empty()
                && run
                    .started_at
                    .is_some_and(|at| at.saturating_add(record_days * DAY_MS) <= now_ms));
        write_json(
            &dir.join("run.json"),
            &json!({
                "run": {
                    "id": run.id,
                    "item_id": run.item_id,
                    "session_id": run.session_id,
                    "trigger": run.trigger.as_str(),
                    "state": run.state.as_str(),
                    "stage_reached": run.stage_reached,
                    "queued_at": run.queued_at,
                    "started_at": run.started_at,
                    "finished_at": run.finished_at,
                    "cost_micro_usd": run.cost_micro_usd,
                    "error": run.error,
                },
                "settings_snapshot": parsed(run.settings_snapshot.as_deref()),
                "form_snapshot": parsed(run.form_snapshot.as_deref()),
                "input_snapshot": parsed(run.input_snapshot.as_deref()),
                "call_record": if expired { CALL_RECORD_EXPIRED } else { "kept" },
            }),
        )?;
        write_json(
            &dir.join("steps.json"),
            &Json::Array(
                steps
                    .iter()
                    .map(|s| {
                        json!({
                            "seq": s.seq,
                            "stage": s.stage,
                            "kind": s.kind,
                            "summary": s.summary,
                            "detail": s.detail,
                            "source_url": s.source_url,
                            "source_title": s.source_title,
                            "photo_ids": s.photo_ids,
                            "field_keys": s.field_keys,
                            "created_at": s.created_at,
                        })
                    })
                    .collect(),
            ),
        )?;
        write_json(
            &dir.join("writes.json"),
            &parsed(decide.as_ref().map(|o| o.output.as_str())),
        )?;
        write_json(
            &dir.join("feedback.json"),
            &Json::Array(
                feedback
                    .iter()
                    .map(|f| {
                        json!({
                            "id": f.id,
                            "key": f.key,
                            "action": f.action,
                            "suggested_value": parsed(Some(&f.suggested_value)),
                            "confidence": f.confidence,
                            "final_value": parsed(f.final_value.as_deref()),
                            "at": f.at,
                        })
                    })
                    .collect(),
            ),
        )?;
        let mut images: BTreeMap<String, ()> = BTreeMap::new();
        for call in &calls {
            let file = self.call_file(call, &withheld);
            for text in [file["request"].as_str(), file["response"].as_str()]
                .into_iter()
                .flatten()
            {
                for sha in image_refs(text) {
                    images.insert(sha, ());
                }
            }
            write_json(&dir.join("calls").join(format!("{}.json", call.id)), &file)?;
        }
        for sha in images.keys() {
            let source = self.files.path(sha);
            if source.is_file() {
                let target = dir.join("images").join(format!("{sha}.jpg"));
                fs::create_dir_all(dir.join("images")).map_err(io)?;
                fs::copy(&source, &target).map_err(io)?;
            }
        }
        Ok(())
    }

    /// A call record as exported: the recorded request and response texts (byte for byte, but
    /// for withheld images) and the record's other fields.
    fn call_file(&self, call: &CallRow, withheld: &BTreeSet<String>) -> Json {
        json!({
            "id": call.id,
            "run_id": call.run_id,
            "stage": call.stage,
            "unit_key": call.unit_key,
            "attempt": call.attempt,
            "provider": call.provider,
            "model": call.model,
            "effort": call.effort,
            "state": call.state,
            "request": withhold(&call.request, withheld),
            "response": call.response.as_deref().map(|r| withhold(r, withheld)),
            "usage": call.usage,
            "cost_micro_usd": call.cost_micro_usd,
            "started_at": call.started_at,
            "finished_at": call.finished_at,
            "duration_ms": call.duration_ms,
            "error_class": call.error_class,
            "error_text": call.error_text,
            "redaction_hit": call.redaction_hit,
        })
    }

    /// MC-A11 Export research: the selected runs' folder under `dest`, scanned for secrets, then
    /// zipped beside it. Returns the zip's path.
    pub fn export(&self, selection: ExportSelection, dest: &Path) -> Result<PathBuf> {
        let now = self.clock.now_ms();
        let stamp = local(now)
            .map(|t| t.format("%Y%m%d-%H%M%S").to_string())
            .unwrap_or_else(|| now.to_string());
        let name = format!("research-export-{stamp}");
        let partial = dest.join(format!(".{name}.partial"));
        let folder = dest.join(&name);
        let zip_path = dest.join(format!("{name}.zip"));
        if folder.exists() || zip_path.exists() {
            return Err(Error::InvalidData(format!("{name} already exists")));
        }
        let runs = self.selected_runs(&selection)?;
        let record_days = self.settings().record_days.max(2);
        let written = (|| -> Result<()> {
            fs::create_dir_all(&partial).map_err(io)?;
            for run in &runs {
                self.export_run(run, &partial, now, record_days)?;
            }
            Ok(())
        })();
        if let Err(error) = written {
            let _ = fs::remove_dir_all(&partial);
            return Err(error);
        }
        // Fail closed: no file may hold a value of the secrets port.
        let secrets = self.gateway.secret_values();
        let secrets: Vec<&str> = secrets.iter().map(String::as_str).collect();
        for (path, relative) in files_under(&partial)? {
            let bytes = fs::read(&path).map_err(io)?;
            if redact(&String::from_utf8_lossy(&bytes), &secrets).hit {
                let _ = fs::remove_dir_all(&partial);
                return Err(Error::InvalidData(format!(
                    "Export research stopped: {relative} contains a key"
                )));
            }
        }
        fs::rename(&partial, &folder).map_err(io)?;
        if let Err(error) = zip_folder(&folder, &zip_path) {
            let _ = fs::remove_file(zip_path.with_extension("zip.partial"));
            return Err(error);
        }
        Ok(zip_path)
    }
}
