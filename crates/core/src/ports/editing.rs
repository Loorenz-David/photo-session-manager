use crate::{
    Result,
    domain::{
        composition::EditorGeometry,
        editing::{
            DocumentState, EditingCapabilities, ProvisionInputs, ProvisionReport, RenderPurpose,
            StyleOutcome,
        },
        photo::Classification,
    },
};
use std::{
    path::Path,
    time::{Instant, SystemTime},
};

#[derive(Debug, Clone)]
pub struct RenderCompletion {
    pub job_id: String,
    pub raw: String,
    pub purpose: RenderPurpose,
    pub generation: i64,
    pub issued_at_ms: i64,
    pub tracking_job_id: i64,
    pub geometry: EditorGeometry,
}

pub trait EditingProvider: Send + Sync {
    fn ensure(&self, inputs: &ProvisionInputs) -> Result<ProvisionReport>;
    fn bring_document_to_front(&self) -> Result<()>;
    fn inspect_document(&self) -> Result<DocumentState>;
    fn set_capture_folder(&self, target: &str) -> Result<()>;
    fn set_current_collection(&self, folder: &str) -> Result<()>;
    fn apply_style(&self, raw: &str, name: &str) -> Result<StyleOutcome>;
    fn find_variant(&self, raw: &str) -> Result<bool>;
    fn install_hooks(&self) -> Result<()>;
    fn render(&self, raw: &str, purpose: RenderPurpose) -> Result<String>;
    fn request_render(
        &self,
        raw: &str,
        purpose: RenderPurpose,
        generation: i64,
        tracking_job_id: i64,
        now: Instant,
        now_ms: i64,
    ) -> Result<()>;
    fn render_completion(&self, job: &str) -> Option<RenderCompletion>;
    fn accept_render_output(&self, output: &Path, issued_at_ms: i64) -> bool;
    fn finish_render(&self, job: &str, now: Instant, now_ms: i64) -> Result<()>;
    fn expire_renders(&self, now: Instant, now_ms: i64) -> Result<Vec<(i64, i64)>>;
    fn render_in_flight(&self, raw: &str) -> bool;
    /// How many renders the provider has issued and not finished (including a start in progress):
    /// "Take the camera back" waits for none (10E MC-34 R-I32).
    fn renders_in_flight(&self) -> usize {
        0
    }
    fn mirror_tag(&self, raw: &str, classification: Classification, now_ms: i64) -> Result<()>;
    fn settings_changed_at(&self, raw: &str) -> Result<Option<SystemTime>>;
    fn capabilities(&self) -> EditingCapabilities;
}
