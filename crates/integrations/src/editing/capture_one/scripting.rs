use psm_core::Result;
use psm_core::domain::composition::EditorGeometry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub kind: String,
    pub id: String,
    pub captures: String,
    pub collections: Vec<CollectionInfo>,
    pub camera: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub id: String,
    pub name: String,
    pub folder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorInfo {
    pub running: bool,
    pub version: Option<String>,
    pub pro: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeProps {
    pub enabled: bool,
    pub output_format: String,
    pub bits: i64,
    pub tiff_compression: String,
    pub export_crop_method: String,
    pub color_profile: String,
    pub scaling_method: String,
    pub scaling_unit: String,
    pub primary_scaling_value: i64,
    pub jpeg_quality: i64,
    pub existing_files: String,
    pub output_sub_folder: String,
    pub root_folder_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeEnsure {
    pub created: bool,
    pub changed: Vec<String>,
    pub props: RecipeProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleEnsure {
    pub listed: bool,
    pub applied: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStarted {
    pub job_id: String,
    pub geometry: EditorGeometry,
}

/// The styles Capture One lists (P63, owner 2026-11-10): the photographer's own (user/custom)
/// styles first, then the built-in ones, each in Capture One's order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleList {
    pub user: Vec<String>,
    pub built_in: Vec<String>,
}

pub trait CaptureOneScripting: Send + Sync {
    /// Whether Capture One runs, read from the system's process list: never an Apple event, so it
    /// can never launch Capture One (P63 #22).
    fn is_running(&self) -> Result<bool>;
    /// The styles Capture One lists (P63). Only ever asked while Capture One runs.
    fn list_styles(&self) -> Result<StyleList>;
    fn editor_info(&self) -> Result<EditorInfo>;
    fn launch(&self) -> Result<()>;
    fn make_document(&self, parent: &str, name: &str) -> Result<String>;
    fn open_document(&self, file: &str) -> Result<()>;
    fn front_document(&self, id: &str) -> Result<String>;
    fn ensure_recipe(&self, id: &str, name: &str, want: &RecipeProps) -> Result<RecipeEnsure>;
    fn ensure_capture_style(&self, id: &str, name: &str) -> Result<StyleEnsure>;
    fn document_info(&self, id: &str) -> Result<DocumentInfo>;
    fn save(&self, id: &str) -> Result<()>;
    fn set_captures(&self, id: &str, path: &str) -> Result<()>;
    fn set_current_collection(&self, id: &str, folder: &str) -> Result<()>;
    fn apply_style(&self, id: &str, variant: &str, name: &str) -> Result<bool>;
    fn get_captures(&self, id: &str) -> Result<String>;
    fn set_hook(&self, name: &str, path: &str) -> Result<()>;
    fn get_hook(&self, name: &str) -> Result<Option<String>>;
    fn recipe_exists(&self, id: &str, name: &str) -> Result<bool>;
    fn find_image(&self, id: &str, raw: &str) -> Result<Option<String>>;
    fn find_image_in_folder(&self, id: &str, folder: &str, raw: &str) -> Result<Option<String>>;
    fn get_color_tag(&self, id: &str, variant: &str) -> Result<i32>;
    fn set_color_tag(&self, id: &str, variant: &str, tag: i32) -> Result<()>;
    fn process(&self, id: &str, variant: &str, recipe: &str) -> Result<ProcessStarted>;
    fn capture(&self) -> Result<()>;
    /// Asks a running Capture One to quit (AppleScript `quit`; never a kill). Does nothing when it
    /// is not running, so it can never launch it (10E MC-34 R-I32 step 3, P48 L13).
    fn quit(&self) -> Result<()>;
}
