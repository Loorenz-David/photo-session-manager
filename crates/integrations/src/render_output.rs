//! The `render_output` job (10D MC-28 §Output render): one output of one photo rendered from the
//! RAW's master by the app's renderer, committed to the output cache under its key, then handed to
//! the export run that asked for it. Runs on its own runner, one job at a time.

use psm_core::{
    Result,
    domain::{composition::OutputSlot, editing::Job},
    mechanisms::composition::{default_rect, turned_frame},
    ports::{
        images::{Imaging, MasterStore, OutputStore, uid12},
        store::Store,
    },
    services::export::ExportService,
};
use psm_infra::jobs::JobHandler;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;

#[derive(Deserialize)]
struct Payload {
    run_id: i64,
    photo_id: i64,
    slot: OutputSlot,
    adjusted_gen: i64,
}

pub struct RenderOutputJob {
    pub store: Arc<dyn Store>,
    pub imaging: Arc<dyn Imaging>,
    pub masters: Arc<dyn MasterStore>,
    pub outputs: Arc<dyn OutputStore>,
    pub export: Arc<ExportService>,
}

impl JobHandler for RenderOutputJob {
    fn run(&self, job: &Job, _now_ms: i64) -> Result<()> {
        let payload: Payload = serde_json::from_str(&job.payload)
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        if !self.export.has_run(payload.run_id) {
            tracing::info!(
                job = job.id,
                run = payload.run_id,
                "render_output for no running export closed"
            );
            return Ok(());
        }
        let photo = self.store.photo(payload.photo_id)?;
        let composition = self.store.composition(payload.photo_id)?;
        let master = self.masters.path(&photo.raw_path, payload.adjusted_gen);
        let values = match composition.values() {
            Some(values) => Some(values),
            None => self.imaging.dimensions(&master)?.map(|frame| {
                let rect = default_rect(
                    composition.kind,
                    composition.shape,
                    composition.orientation,
                    turned_frame(frame, composition.quarter_turns),
                );
                psm_core::domain::composition::CompositionValues {
                    kind: composition.kind,
                    shape: composition.shape,
                    orientation: composition.orientation,
                    quarter_turns: composition.quarter_turns,
                    flip: composition.flip,
                    straighten: composition.straighten,
                    rect,
                }
            }),
        };
        let format = self.export.format();
        let uid = uid12(&photo.uid);
        let dest = self.outputs.path(
            &photo.raw_path,
            &uid,
            payload.slot,
            payload.adjusted_gen,
            composition.revision,
            &format.format_key(),
        );
        let temp = dest.with_extension("jpg.render");
        let started = Instant::now();
        let rendered = values
            .ok_or_else(|| psm_core::Error::InvalidData("master frame unknown".into()))
            .and_then(|values| {
                self.imaging
                    .render_output(&master, &values, payload.slot, &format, &temp)
            })
            .and_then(|_| self.outputs.commit(&dest, &temp));
        if let Err(error) = rendered {
            tracing::warn!(%error, job = job.id, "output render failed");
            let _ = std::fs::remove_file(&temp);
            return self.export.output_failed(payload.run_id, "render");
        }
        tracing::info!(photo_id = payload.photo_id, slot = ?payload.slot, ms = started.elapsed().as_millis() as u64, "render_output done");
        if let Err(error) = self.outputs.prune(&photo.raw_path, &uid, &dest) {
            tracing::warn!(%error, "stale outputs not pruned");
        }
        self.export.output_ready(
            payload.run_id,
            payload.photo_id,
            payload.slot,
            &dest,
            composition.revision,
        )
    }
}
