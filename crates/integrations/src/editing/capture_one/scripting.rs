use psm_core::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub kind: String,
    pub captures: String,
    pub capture_root: String,
    pub camera: Option<String>,
}

pub trait CaptureOneScripting: Send + Sync {
    fn document_info(&self, id: &str) -> Result<DocumentInfo>;
    fn save(&self, id: &str) -> Result<()>;
    fn set_captures(&self, id: &str, path: &str) -> Result<()>;
    fn get_captures(&self, id: &str) -> Result<String>;
    fn set_hook(&self, name: &str, path: &str) -> Result<()>;
    fn get_hook(&self, name: &str) -> Result<Option<String>>;
    fn recipe_exists(&self, name: &str) -> Result<bool>;
    fn set_recipe_props(
        &self,
        name: &str,
        existing_files: &str,
        output_sub_folder: &str,
    ) -> Result<()>;
    fn find_image(&self, id: &str, raw: &str) -> Result<Option<String>>;
    fn find_image_in_folder(&self, id: &str, folder: &str, raw: &str) -> Result<Option<String>>;
    fn get_color_tag(&self, variant: &str) -> Result<i32>;
    fn set_color_tag(&self, variant: &str, tag: i32) -> Result<()>;
    fn process(&self, variant: &str, recipe: &str) -> Result<String>;
    fn capture(&self) -> Result<()>;
    fn import(&self, id: &str, paths: &[String]) -> Result<()>;
}
