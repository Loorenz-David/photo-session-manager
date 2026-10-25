use crate::{
    Result,
    domain::{
        editing::{DocumentState, EditingCapabilities, RenderPurpose},
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
}

pub trait EditingProvider: Send + Sync {
    fn inspect_document(&self) -> Result<DocumentState>;
    fn set_capture_folder(&self, target: &str) -> Result<()>;
    fn install_hooks(&self) -> Result<()>;
    fn check_recipes(&self) -> Result<Vec<String>>;
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
    fn mirror_tag(&self, raw: &str, classification: Classification, now_ms: i64) -> Result<()>;
    fn settings_changed_at(&self, raw: &str) -> Result<Option<SystemTime>>;
    fn import(&self, paths: &[String]) -> Result<()>;
    fn capabilities(&self) -> EditingCapabilities;
}
