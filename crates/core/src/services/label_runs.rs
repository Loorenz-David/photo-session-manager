//! L3 of `LabelsService` (book 09C §6; `master_plan_labels.md` §5 MC-L2): the check (what
//! generating would do, chunked and shown as one), the preview picture of one label, the one
//! kept check, generating (one key per chunk, made once and reused by every retry and by Try
//! again) and its result. Nothing here is stored (M-L1): the check, its generation and the keys
//! live in the service's memory and are gone at quit. Platform-neutral (H7): the provider is a
//! port, the files are a port. `provider.run` is called at exactly one place, in `generate`.
//! The download saves a file of the kept result into the labels folder and opens it.

use super::labels::{LabelsFailure, LabelsService};
use crate::{
    Error,
    domain::label::{
        CheckOutcome, LabelArtifact, LabelCheck, LabelPlan, LabelRun, LabelRunResult, LabelState,
        LabelStatus, LabelsError, RunResult, SavedLabel, Unresolved,
    },
    events::AppEvent,
    mechanisms::labels::{
        ChunkOutcome, RUN_CHUNK, RUN_RETRIES, chunks, group_check, merge_plans, merge_runs,
    },
};
use std::{collections::HashMap, path::PathBuf};

const LOG: &str = "psm::labels";

type Answer<T> = std::result::Result<T, LabelsFailure>;

/// MC-L2.3: the one check the service keeps (memory only).
pub struct KeptCheck {
    id: String,
    plan: LabelPlan,
    /// The preview picture made for a product of the check (MC-L2.5).
    previews: HashMap<String, PathBuf>,
    /// MC-L2.6: created by the first `generate`.
    generation: Option<Generation>,
}

/// MC-L2.6–.10: the generation of the kept check.
struct Generation {
    chunks: Vec<RunChunk>,
    /// A `generate` call is working (a second one answers `Busy`, and so does `check`).
    running: bool,
    /// Every chunk has an outcome: the result, answered again without a request.
    result: Option<LabelRunResult>,
}

/// One chunk of ready products with its key (made once) and its outcome.
struct RunChunk {
    product_ids: Vec<String>,
    key: String,
    outcome: Option<ChunkOutcome>,
}

impl KeptCheck {
    fn result(&self) -> Option<LabelRunResult> {
        let generation = self.generation.as_ref()?;
        if let Some(result) = &generation.result {
            return Some(result.clone());
        }
        let chunks: Vec<(Vec<String>, Option<ChunkOutcome>)> = generation
            .chunks
            .iter()
            .map(|c| (c.product_ids.clone(), c.outcome.clone()))
            .collect();
        Some(merge_runs(&self.plan.items, &chunks))
    }

    fn running(&self) -> bool {
        self.generation.as_ref().is_some_and(|g| g.running)
    }
}

/// Clears the generation's `running` flag on every way out of `generate`.
struct Running<'s> {
    service: &'s LabelsService,
    check_id: String,
}

impl Drop for Running<'_> {
    fn drop(&mut self) {
        if let Some(generation) = self
            .service
            .kept()
            .as_mut()
            .filter(|c| c.id == self.check_id)
            .and_then(|c| c.generation.as_mut())
        {
            generation.running = false;
        }
    }
}

/// The reason text of a file failure (projection r0, D2): the inner `io::Error`'s text for an
/// I/O error, else the error's own text.
pub(super) fn reason(error: &Error) -> String {
    match error {
        Error::Io(inner) => inner.to_string(),
        other => other.to_string(),
    }
}

/// A preview picture's name: the check id and the product id, keeping only characters that
/// cannot leave the previews folder.
fn picture_name(check_id: &str, product_id: &str) -> String {
    let safe: String = product_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    format!("{check_id}-{safe}")
}

impl LabelsService {
    fn kept(&self) -> std::sync::MutexGuard<'_, Option<KeptCheck>> {
        self.kept.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// D10: the kept check is dropped and its preview pictures removed.
    fn drop_check(&self) {
        *self.kept() = None;
        if let Err(error) = self.files.clear_previews() {
            tracing::warn!(target: LOG, %error, "label previews not cleared");
        }
    }

    /// MC-L2.1–.4 (a press; changes nothing at the provider): the products without repeats
    /// (first kept), checked by the provider in chunks of `RUN_CHUNK` and merged; one templates
    /// read names each group's product types (a failed read leaves them empty). The new check
    /// replaces the kept one and clears the preview pictures. A chunk the provider found none of
    /// lists its products as not found; when every chunk does, `NoProducts`. Any other provider
    /// error is answered and nothing is kept (D10: an earlier check is dropped too).
    pub fn check(&self, product_ids: &[String]) -> Answer<LabelCheck> {
        let mut ids: Vec<String> = Vec::with_capacity(product_ids.len());
        for id in product_ids {
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }
        if ids.is_empty() {
            return Err(Error::InvalidValue("no products to check".into()).into());
        }
        if self.kept().as_ref().is_some_and(KeptCheck::running) {
            return Err(LabelsFailure::Busy);
        }
        let store = self.gate()?;
        let ranges = chunks(ids.len(), RUN_CHUNK);
        let mut answers = Vec::with_capacity(ranges.len());
        for (index, range) in ranges.iter().enumerate() {
            let chunk = &ids[range.clone()];
            match self.learn(self.provider.plan(&store, chunk)) {
                Ok(plan) => answers.push((index, Ok(plan))),
                Err(LabelsFailure::Provider(LabelsError::NoProducts)) => answers.push((
                    index,
                    Err(chunk
                        .iter()
                        .map(|id| Unresolved {
                            product_id: Some(id.clone()),
                            sku: None,
                            message: String::new(),
                        })
                        .collect()),
                )),
                Err(failure) => {
                    self.drop_check();
                    return Err(failure);
                }
            }
        }
        if answers.iter().all(|(_, answer)| answer.is_err()) {
            self.drop_check();
            return Err(LabelsFailure::Provider(LabelsError::NoProducts));
        }
        let templates = self
            .learn(self.provider.templates(&store))
            .unwrap_or_default();
        let plan = merge_plans(answers);
        let groups = group_check(&plan, &templates);
        let id = uuid::Uuid::new_v4().to_string();
        tracing::info!(
            target: LOG,
            "labels.check products={} chunks={} ready={} failing={}",
            ids.len(),
            ranges.len(),
            plan.summary.ready,
            plan.summary.failing
        );
        let check = LabelCheck {
            id: id.clone(),
            groups,
            unresolved: plan.unresolved.clone(),
            summary: plan.summary,
            sizes: plan.sizes.clone(),
        };
        if self.kept().as_ref().is_some_and(KeptCheck::running) {
            return Err(LabelsFailure::Busy);
        }
        self.drop_check();
        *self.kept() = Some(KeptCheck {
            id,
            plan,
            previews: HashMap::new(),
            generation: None,
        });
        Ok(check)
    }

    /// MC-L2.5: the preview picture of one product of the kept check (else `NotFound`, no
    /// request). The first ask fetches the provider's PDF and turns its page into a picture; a
    /// later ask answers that picture without a request while the file exists. A picture that
    /// cannot be made is `PreviewFailed`; a preview never blocks Generate.
    pub fn preview(&self, check_id: &str, product_id: &str) -> Answer<PathBuf> {
        {
            let kept = self.kept();
            let check = kept
                .as_ref()
                .filter(|c| c.id == check_id)
                .ok_or(LabelsFailure::NotFound)?;
            if !check.plan.items.iter().any(|i| i.product_id == product_id) {
                return Err(LabelsFailure::NotFound);
            }
            if let Some(path) = check.previews.get(product_id).filter(|p| p.is_file()) {
                return Ok(path.clone());
            }
        }
        let store = self.gate()?;
        let pdf = self.learn(self.provider.preview(&store, product_id))?;
        let path = self
            .files
            .raster(&pdf, &picture_name(check_id, product_id))
            .map_err(|error| LabelsFailure::PreviewFailed(reason(&error)))?;
        if let Some(check) = self.kept().as_mut().filter(|c| c.id == check_id) {
            check.previews.insert(product_id.to_owned(), path.clone());
        }
        Ok(path)
    }

    /// MC-L2.6–.10 (the only way a run starts, M-L2): generates the labels of the kept check's
    /// ready products (outcome not `Fail`), else `NotFound` (no request). The first call makes
    /// the chunks of `RUN_CHUNK` and **each chunk's key, once**; every chunk without an outcome
    /// is sent in order with its stored key and its stored products. A chunk that could not be
    /// reached, or whose renderer is busy, is sent again at once, up to `RUN_RETRIES` more times;
    /// still failing, that error is answered and the next call (Try again) continues at that
    /// chunk with the same key. A finished chunk is never sent again; a finished generation is
    /// answered again without a request. While a call works, another `generate` (a double press)
    /// and `check` answer `Busy`.
    pub fn generate(&self, check_id: &str) -> Answer<LabelRunResult> {
        {
            let mut kept = self.kept();
            let check = kept
                .as_mut()
                .filter(|c| c.id == check_id)
                .ok_or(LabelsFailure::NotFound)?;
            if let Some(generation) = &check.generation {
                if generation.running {
                    return Err(LabelsFailure::Busy);
                }
                if let Some(result) = &generation.result {
                    return Ok(result.clone());
                }
            }
            if check.generation.is_none() {
                let ready: Vec<String> = check
                    .plan
                    .items
                    .iter()
                    .filter(|i| i.outcome != CheckOutcome::Fail)
                    .map(|i| i.product_id.clone())
                    .collect();
                if ready.is_empty() {
                    return Err(LabelsFailure::NothingToGenerate);
                }
                check.generation = Some(Generation {
                    chunks: chunks(ready.len(), RUN_CHUNK)
                        .into_iter()
                        .map(|range| RunChunk {
                            product_ids: ready[range].to_vec(),
                            key: format!("sm-{}", uuid::Uuid::new_v4()),
                            outcome: None,
                        })
                        .collect(),
                    running: false,
                    result: None,
                });
            }
            if let Some(generation) = check.generation.as_mut() {
                generation.running = true;
            }
        }
        let _running = Running {
            service: self,
            check_id: check_id.to_owned(),
        };
        let store = self.gate()?;
        let (count, printed_at) = {
            let kept = self.kept();
            let check = kept
                .as_ref()
                .filter(|c| c.id == check_id)
                .ok_or(LabelsFailure::NotFound)?;
            let count = check.generation.as_ref().map_or(0, |g| g.chunks.len());
            let printed_at: HashMap<String, Option<i64>> = check
                .plan
                .items
                .iter()
                .map(|i| (i.product_id.clone(), i.printed_at))
                .collect();
            (count, printed_at)
        };
        for index in 0..count {
            let Some((product_ids, key)) = self.pending_chunk(check_id, index) else {
                continue;
            };
            let started = self.copy().start();
            let mut attempt = 0;
            let answer = loop {
                attempt += 1;
                let answer = self.provider.run(&store, &product_ids, &key);
                tracing::info!(
                    target: LOG,
                    "labels.run chunk={} of={count} items={} attempt={attempt} outcome={}",
                    index + 1,
                    product_ids.len(),
                    match &answer {
                        Ok(_) => "ok".to_owned(),
                        Err(error) => error.to_string(),
                    }
                );
                match answer {
                    Err(LabelsError::Unreachable(_) | LabelsError::RendererUnavailable)
                        if attempt <= RUN_RETRIES => {}
                    answer => break answer,
                }
            };
            let outcome = match self.learn(answer) {
                Ok(run) => {
                    self.follow_run(started, &run, &printed_at);
                    ChunkOutcome::Ran(run)
                }
                Err(LabelsFailure::Provider(LabelsError::NoProducts)) => {
                    ChunkOutcome::NoProducts(self.describe_error(&LabelsError::NoProducts))
                }
                Err(failure) => return Err(failure),
            };
            if let Some(chunk) = self
                .kept()
                .as_mut()
                .filter(|c| c.id == check_id)
                .and_then(|c| c.generation.as_mut())
                .and_then(|g| g.chunks.get_mut(index))
            {
                chunk.outcome = Some(outcome);
            }
        }
        let mut kept = self.kept();
        let check = kept
            .as_mut()
            .filter(|c| c.id == check_id)
            .ok_or(LabelsFailure::NotFound)?;
        let result = check.result().ok_or(LabelsFailure::NotFound)?;
        if let Some(generation) = check.generation.as_mut() {
            generation.result = Some(result.clone());
        }
        Ok(result)
    }

    /// The stored products and key of chunk `index` when it has no outcome yet.
    fn pending_chunk(&self, check_id: &str, index: usize) -> Option<(Vec<String>, String)> {
        let kept = self.kept();
        let chunk = kept
            .as_ref()
            .filter(|c| c.id == check_id)?
            .generation
            .as_ref()?
            .chunks
            .get(index)?;
        chunk
            .outcome
            .is_none()
            .then(|| (chunk.product_ids.clone(), chunk.key.clone()))
    }

    /// MC-L2.8: the display copy follows a finished chunk (`started` was taken before its
    /// request): marked printed → printed at the run's time with a price; no price → not
    /// printed without a price (the reset as it was); reprinted → printed at the time the check
    /// showed (else the run's). Then the screen is told which products changed.
    fn follow_run(&self, started: u64, run: &LabelRun, printed_at: &HashMap<String, Option<i64>>) {
        let now = self.clock.mono();
        let mut changed: Vec<String> = Vec::new();
        {
            let mut copy = self.copy();
            for item in &run.items {
                let Some(result) = item.result else { continue };
                let before = copy.get(&item.product_id);
                let state = match result {
                    RunResult::MarkedPrinted => LabelState {
                        status: LabelStatus::Printed,
                        printed_at: Some(run.created_at),
                        has_price: Some(true),
                        reset: None,
                    },
                    RunResult::NoPrice => LabelState {
                        status: LabelStatus::NotPrinted,
                        printed_at: None,
                        has_price: Some(false),
                        reset: before.and_then(|s| s.reset),
                    },
                    RunResult::Reprinted => LabelState {
                        status: LabelStatus::Printed,
                        printed_at: printed_at
                            .get(&item.product_id)
                            .copied()
                            .flatten()
                            .or(Some(run.created_at)),
                        has_price: before.and_then(|s| s.has_price),
                        reset: None,
                    },
                };
                copy.apply_answer(started, &item.product_id, Some(state), now);
                if !changed.contains(&item.product_id) {
                    changed.push(item.product_id.clone());
                }
            }
        }
        if !changed.is_empty() {
            self.events.emit(AppEvent::LabelsChanged {
                product_ids: changed,
                access: false,
            });
        }
    }

    /// MC-L2.10: the result over the finished chunks of the kept check's generation (for a
    /// dialog the photographer leaves after a failed chunk); `NotFound` without a generation.
    pub fn partial(&self, check_id: &str) -> Answer<LabelRunResult> {
        self.kept()
            .as_ref()
            .filter(|c| c.id == check_id)
            .and_then(KeptCheck::result)
            .ok_or(LabelsFailure::NotFound)
    }

    /// MC-L2.11: a file of the kept check's finished chunks (else `NotFound`, no request) is
    /// downloaded by the address the run named (never changed here), saved in the labels folder
    /// and opened; a failed open is logged and answered as `opened: false` (the file is saved).
    /// `Expired` and `SaveFailed` are answered; a second call tries again.
    pub fn download(&self, check_id: &str, artifact_id: &str) -> Answer<SavedLabel> {
        let artifact: LabelArtifact = {
            let kept = self.kept();
            kept.as_ref()
                .filter(|c| c.id == check_id)
                .and_then(|c| c.generation.as_ref())
                .and_then(|g| {
                    g.chunks.iter().find_map(|chunk| match &chunk.outcome {
                        Some(ChunkOutcome::Ran(run)) => {
                            run.artifacts.iter().find(|a| a.id == artifact_id).cloned()
                        }
                        _ => None,
                    })
                })
                .ok_or(LabelsFailure::NotFound)?
        };
        let store = self.gate()?;
        let bytes = match self.learn(self.provider.download(&store, &artifact.download_url)) {
            Ok(bytes) => bytes,
            Err(failure) => {
                tracing::info!(target: LOG, "labels.download bytes=0 outcome={failure}");
                return Err(failure);
            }
        };
        let path = match self.files.save(&artifact.filename, &bytes) {
            Ok(path) => path,
            Err(error) => {
                tracing::warn!(target: LOG, "labels.download bytes={} outcome=save failed", bytes.len());
                return Err(LabelsFailure::SaveFailed(reason(&error)));
            }
        };
        let opened = match self.files.open(&path) {
            Ok(()) => true,
            Err(error) => {
                tracing::warn!(target: LOG, %error, "label file not opened");
                false
            }
        };
        tracing::info!(
            target: LOG,
            "labels.download bytes={} outcome={}",
            bytes.len(),
            if opened { "opened" } else { "saved" }
        );
        Ok(SavedLabel { path, opened })
    }
}
