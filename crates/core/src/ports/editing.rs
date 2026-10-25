use crate::{
    Result,
    domain::{
        editing::{DocumentState, EditingCapabilities, RenderPurpose},
        photo::Classification,
    },
};

pub trait EditingProvider: Send + Sync {
    fn inspect_document(&self) -> Result<DocumentState>;
    fn set_capture_folder(&self, target: &str) -> Result<()>;
    fn install_hooks(&self) -> Result<()>;
    fn check_recipes(&self) -> Result<Vec<String>>;
    fn render(&self, raw: &str, purpose: RenderPurpose) -> Result<String>;
    fn mirror_tag(&self, raw: &str, classification: Classification, now_ms: i64) -> Result<()>;
    fn import(&self, paths: &[String]) -> Result<()>;
    fn capabilities(&self) -> EditingCapabilities;
}
