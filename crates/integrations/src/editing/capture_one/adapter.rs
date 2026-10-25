use super::{
    renders::{PendingRender, RenderEntry, RenderRegistry, accept_output, base_name},
    scripting::CaptureOneScripting,
};
use psm_core::{
    Error, Result,
    domain::{
        editing::{DocumentState, EditingCapabilities, RenderPurpose},
        photo::Classification,
    },
    mechanisms::canon::canon,
    ports::{
        editing::{EditingProvider, RenderCompletion},
        store::Store,
    },
    services::health::HealthService,
};
use psm_infra::paths::AppPaths;
use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant, UNIX_EPOCH},
};

pub struct CaptureOneAdapter {
    pub scripting: Arc<dyn CaptureOneScripting>,
    pub store: Arc<dyn Store>,
    pub health: Arc<HealthService>,
    pub document_id: String,
    pub preview_recipe: String,
    pub web_recipe: String,
    pub paths: AppPaths,
    renders: Mutex<RenderRegistry>,
}
impl CaptureOneAdapter {
    pub fn new(
        scripting: Arc<dyn CaptureOneScripting>,
        store: Arc<dyn Store>,
        health: Arc<HealthService>,
        document_id: String,
        preview_recipe: String,
        web_recipe: String,
        paths: AppPaths,
    ) -> Self {
        Self {
            scripting,
            store,
            health,
            document_id,
            preview_recipe,
            web_recipe,
            paths,
            renders: Mutex::new(RenderRegistry::default()),
        }
    }
    fn variant(&self, raw: &str) -> Result<String> {
        if let Some(id) = self.scripting.find_image(&self.document_id, raw)? {
            return Ok(id);
        }
        let folder = Path::new(raw)
            .parent()
            .ok_or(Error::VariantNotFound)?
            .to_string_lossy();
        self.scripting
            .find_image_in_folder(&self.document_id, &folder, raw)?
            .ok_or(Error::VariantNotFound)
    }
    fn apply_folder(&self, target: &str) -> Result<()> {
        self.scripting.save(&self.document_id)?;
        fs::create_dir_all(target)?;
        self.scripting.set_captures(&self.document_id, target)?;
        let readback = self.scripting.get_captures(&self.document_id)?;
        if !same_path(&readback, target) {
            return Err(Error::CaptureFolderNotSet);
        }
        Ok(())
    }
}
fn same_path(a: &str, b: &str) -> bool {
    match (canon(Path::new(a)), canon(Path::new(b))) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
pub fn tag_for(classification: Classification) -> i32 {
    match classification {
        Classification::Main => 4,
        Classification::Condition => 5,
        Classification::Deleted => 1,
        Classification::Unreviewed => 0,
    }
}
impl EditingProvider for CaptureOneAdapter {
    fn inspect_document(&self) -> Result<DocumentState> {
        if self.document_id.is_empty() {
            return Err(Error::DocumentNotOpen);
        }
        let info = self
            .scripting
            .document_info(&self.document_id)
            .map_err(|_| Error::DocumentNotOpen)?;
        if info.kind != "session" {
            return Err(Error::DocumentNotSession);
        }
        Ok(DocumentState {
            capture_root: info.capture_root,
            capture_folder: info.captures,
        })
    }
    fn set_capture_folder(&self, target: &str) -> Result<()> {
        let prior = self
            .scripting
            .document_info(&self.document_id)
            .map_err(|_| Error::CaptureFolderNotSet)?
            .captures;
        if self.apply_folder(target).is_err() {
            let _ = self.apply_folder(&prior);
            self.health.set(
                "capture_folder_not_set",
                "Capture folder could not be verified",
            );
            return Err(Error::CaptureFolderNotSet);
        }
        self.health.clear("capture_folder_not_set");
        Ok(())
    }
    fn install_hooks(&self) -> Result<()> {
        let result = (|| {
            for name in ["capture_done", "processing_done", "variant_adjusted"] {
                let expected = self.paths.hooks().join(format!("{name}.scpt"));
                let path = expected.to_string_lossy();
                self.scripting.set_hook(name, &path)?;
                let read = self.scripting.get_hook(name)?;
                if !read.as_deref().is_some_and(|value| same_path(value, &path)) {
                    return Err(Error::HooksNotInstalled);
                }
            }
            Ok(())
        })();
        if result.is_err() {
            self.health
                .set("hooks_not_installed", "Hook path read-back mismatch");
        } else {
            self.health.clear("hooks_not_installed");
        }
        result
    }
    fn check_recipes(&self) -> Result<Vec<String>> {
        let mut missing = Vec::new();
        for name in [&self.preview_recipe, &self.web_recipe] {
            if self.scripting.recipe_exists(name)? {
                self.scripting
                    .set_recipe_props(name, "overwrite", "_session_manager")?;
            } else {
                missing.push(name.clone());
            }
        }
        self.health.clear("recipe_missing");
        for name in &missing {
            self.health.set("recipe_missing", name);
        }
        Ok(missing)
    }
    fn render(&self, raw: &str, purpose: RenderPurpose) -> Result<String> {
        let recipe = match purpose {
            RenderPurpose::Preview => &self.preview_recipe,
            RenderPurpose::Export => &self.web_recipe,
        };
        if !self.scripting.recipe_exists(recipe)? {
            return Err(Error::RecipeMissing {
                name: recipe.clone(),
            });
        }
        let variant = self.variant(raw)?;
        self.scripting.process(&variant, recipe)
    }
    fn request_render(
        &self,
        raw: &str,
        purpose: RenderPurpose,
        generation: i64,
        tracking_job_id: i64,
        now: Instant,
        now_ms: i64,
    ) -> Result<()> {
        let mut state = self
            .renders
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        if state.jobs.values().any(|e| e.completion.raw == raw)
            || state.pending.iter().any(|e| e.raw == raw)
        {
            return Ok(());
        }
        if !state.busy.insert(base_name(raw)) {
            state.pending.push_back(PendingRender {
                raw: raw.into(),
                purpose,
                generation,
                tracking_job_id,
            });
            return Ok(());
        }
        match self.render(raw, purpose) {
            Ok(job_id) => {
                state.jobs.insert(
                    job_id.clone(),
                    RenderEntry {
                        completion: RenderCompletion {
                            job_id,
                            raw: raw.into(),
                            purpose,
                            generation,
                            issued_at_ms: now_ms,
                            tracking_job_id,
                        },
                        issued_mono: now,
                        attempts: 0,
                    },
                );
                Ok(())
            }
            Err(error) => {
                state.busy.remove(&base_name(raw));
                Err(error)
            }
        }
    }
    fn render_completion(&self, job: &str) -> Option<RenderCompletion> {
        self.renders
            .lock()
            .ok()?
            .jobs
            .get(job)
            .map(|e| e.completion.clone())
    }
    fn accept_render_output(&self, output: &Path, issued_at_ms: i64) -> bool {
        accept_output(
            output,
            UNIX_EPOCH + Duration::from_millis(issued_at_ms.max(0) as u64),
        )
    }
    fn finish_render(&self, job: &str, now: Instant, now_ms: i64) -> Result<()> {
        let mut state = self
            .renders
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let Some(entry) = state.jobs.remove(job) else {
            return Ok(());
        };
        state.busy.remove(&base_name(&entry.completion.raw));
        let key = base_name(&entry.completion.raw);
        if let Some(index) = state.pending.iter().position(|p| base_name(&p.raw) == key) {
            let next = state.pending.remove(index).expect("pending index");
            state.busy.insert(key);
            match self.render(&next.raw, next.purpose) {
                Ok(job_id) => {
                    state.jobs.insert(
                        job_id.clone(),
                        RenderEntry {
                            completion: RenderCompletion {
                                job_id,
                                raw: next.raw,
                                purpose: next.purpose,
                                generation: next.generation,
                                issued_at_ms: now_ms,
                                tracking_job_id: next.tracking_job_id,
                            },
                            issued_mono: now,
                            attempts: 0,
                        },
                    );
                }
                Err(e) => {
                    state.busy.remove(&base_name(&next.raw));
                    return Err(e);
                }
            }
        }
        Ok(())
    }
    fn expire_renders(&self, now: Instant, now_ms: i64) -> Result<Vec<(i64, i64)>> {
        let mut failed = Vec::new();
        let expired: Vec<_> = {
            let state = self
                .renders
                .lock()
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            state
                .jobs
                .iter()
                .filter(|(_, e)| {
                    now.duration_since(e.issued_mono).as_secs()
                        >= match e.completion.purpose {
                            RenderPurpose::Preview => 60,
                            RenderPurpose::Export => 120,
                        }
                })
                .map(|(id, _)| id.clone())
                .collect()
        };
        for old_id in expired {
            let mut state = self
                .renders
                .lock()
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            let Some(mut entry) = state.jobs.remove(&old_id) else {
                continue;
            };
            if entry.attempts == 0 {
                match self.render(&entry.completion.raw, entry.completion.purpose) {
                    Ok(new_id) => {
                        entry.completion.job_id = new_id.clone();
                        entry.completion.issued_at_ms = now_ms;
                        entry.issued_mono = now;
                        entry.attempts = 1;
                        state.jobs.insert(new_id, entry);
                    }
                    Err(error) => {
                        state.jobs.insert(old_id, entry);
                        return Err(error);
                    }
                }
            } else {
                failed.push((
                    entry.completion.tracking_job_id,
                    entry.completion.generation,
                ));
                let key = base_name(&entry.completion.raw);
                state.busy.remove(&key);
                if let Some(index) = state.pending.iter().position(|p| base_name(&p.raw) == key) {
                    let next = state.pending.remove(index).expect("pending index");
                    state.busy.insert(key);
                    let id = self.render(&next.raw, next.purpose)?;
                    state.jobs.insert(
                        id.clone(),
                        RenderEntry {
                            completion: RenderCompletion {
                                job_id: id,
                                raw: next.raw,
                                purpose: next.purpose,
                                generation: next.generation,
                                issued_at_ms: now_ms,
                                tracking_job_id: next.tracking_job_id,
                            },
                            issued_mono: now,
                            attempts: 0,
                        },
                    );
                }
            }
        }
        Ok(failed)
    }
    fn render_in_flight(&self, raw: &str) -> bool {
        self.renders.lock().is_ok_and(|s| {
            s.jobs.values().any(|e| e.completion.raw == raw)
                || s.pending.iter().any(|e| e.raw == raw)
        })
    }
    fn mirror_tag(&self, raw: &str, classification: Classification, now_ms: i64) -> Result<()> {
        let variant = self.variant(raw)?;
        let desired = tag_for(classification);
        if self.scripting.get_color_tag(&variant)? == desired {
            return Ok(());
        }
        self.store.mark_own_write(raw, now_ms)?;
        let write = self.scripting.set_color_tag(&variant, desired);
        if matches!(write, Err(Error::Timeout)) {
            return Err(Error::Timeout);
        }
        write?;
        if self.scripting.get_color_tag(&variant)? != desired {
            return Err(Error::TagVerification);
        }
        Ok(())
    }
    fn import(&self, paths: &[String]) -> Result<()> {
        self.scripting.import(&self.document_id, paths)
    }
    fn capabilities(&self) -> EditingCapabilities {
        EditingCapabilities {
            capture_folder: true,
            hooks: true,
            render: true,
            mirror_tag: true,
            import: true,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mc19_tag_mapping() {
        assert_eq!(
            [
                Classification::Unreviewed,
                Classification::Deleted,
                Classification::Main,
                Classification::Condition
            ]
            .map(tag_for),
            [0, 1, 4, 5]
        );
    }
}
