use psm_core::{Result, domain::editing::ShapeOutcome};
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

pub trait CaptureOneScripting: Send + Sync {
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
    fn get_captures(&self, id: &str) -> Result<String>;
    fn set_hook(&self, name: &str, path: &str) -> Result<()>;
    fn get_hook(&self, name: &str) -> Result<Option<String>>;
    fn recipe_exists(&self, id: &str, name: &str) -> Result<bool>;
    fn find_image(&self, id: &str, raw: &str) -> Result<Option<String>>;
    fn find_image_in_folder(&self, id: &str, folder: &str, raw: &str) -> Result<Option<String>>;
    fn get_color_tag(&self, id: &str, variant: &str) -> Result<i32>;
    fn set_color_tag(&self, id: &str, variant: &str, tag: i32) -> Result<()>;
    fn crop_aspect_ratios(&self, id: &str) -> Result<Vec<String>>;
    fn apply_crop_if_untouched(&self, id: &str, variant: &str, ratio: &str)
    -> Result<ShapeOutcome>;
    fn process(&self, id: &str, variant: &str, recipe: &str) -> Result<String>;
    fn capture(&self) -> Result<()>;
    fn import(&self, id: &str, paths: &[String]) -> Result<()>;
}
