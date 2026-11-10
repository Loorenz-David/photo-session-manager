use crate::{
    Result,
    domain::{
        composition::EditorGeometry,
        editing::{
            DocumentState, EditingCapabilities, EditorStyles, ProvisionInputs, ProvisionReport,
            RenderPurpose, StyleOutcome,
        },
        photo::Classification,
    },
};
use std::{
    path::{Path, PathBuf},
    time::{Instant, SystemTime},
};

/// The editor's settings sidecar of a RAW (10A MC-17 §Fallback): when it was last written and a
/// fingerprint of the adjustments it holds, so a rewrite that changes no adjustment (the editor's
/// own processing, a document save) is not taken for an edit (P56, P52 F12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsSidecar {
    pub modified: SystemTime,
    pub fingerprint: String,
}

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
    /// The styles the editor lists (P63). Never launches the editor: `EditorLaunchDeferred` when it
    /// is not running.
    fn styles(&self) -> Result<EditorStyles>;
    fn find_variant(&self, raw: &str) -> Result<bool>;
    fn install_hooks(&self) -> Result<()>;
    fn render(&self, raw: &str, purpose: RenderPurpose) -> Result<String>;
    #[allow(clippy::too_many_arguments)]
    fn request_render(
        &self,
        raw: &str,
        purpose: RenderPurpose,
        generation: i64,
        tracking_job_id: i64,
        now: Instant,
        now_ms: i64,
        // A photo without any adjusted preview yet: it goes before the backlog (P58).
        new_arrival: bool,
    ) -> Result<()>;
    /// The photo open on the stage or in the editor; its preview is rendered next (P58).
    fn set_render_focus(&self, _raw: Option<&str>) {}
    /// Queued renders the provider refused when their turn came: (tracking job, error). Drained.
    fn refused_renders(&self) -> Vec<(i64, String)> {
        Vec::new()
    }
    fn render_completion(&self, job: &str) -> Option<RenderCompletion>;
    /// The file a finished render produced, from the path the editor reported (10A MC-16
    /// §Completion): `None` when no file written after the request can be found for it.
    fn resolve_render_output(&self, output: &Path, issued_at_ms: i64) -> Option<PathBuf>;
    fn finish_render(&self, job: &str, now: Instant, now_ms: i64) -> Result<()>;
    /// 10F §5: render `job` finished in the wrong format (its recipe was changed outside the app).
    /// The provider repairs the recipe and sends the render again once (`true`, the job keeps its
    /// completion under a new id); `false` when it was already re-sent: the render has failed.
    fn recipe_fault(&self, _job: &str, _now: Instant, _now_ms: i64) -> Result<bool> {
        Ok(false)
    }
    fn expire_renders(&self, now: Instant, now_ms: i64) -> Result<Vec<(i64, i64)>>;
    fn render_in_flight(&self, raw: &str) -> bool;
    /// How many renders the provider has issued and not finished (including a start in progress):
    /// "Take the camera back" waits for none (10E MC-34 R-I32).
    fn renders_in_flight(&self) -> usize {
        0
    }
    fn mirror_tag(&self, raw: &str, classification: Classification, now_ms: i64) -> Result<()>;
    fn settings_sidecar(&self, raw: &str) -> Result<Option<SettingsSidecar>>;
    fn capabilities(&self) -> EditingCapabilities;
}
