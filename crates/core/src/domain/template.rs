use crate::domain::composition::Shape;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presets {
    pub split: bool,
    pub shape: Option<Shape>,
    pub zoom: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Container {
    pub id: i64,
    pub final_no: i64,
    pub shoot_no: i64,
    pub example_file: Option<String>,
    pub example_path: Option<String>,
    pub presets: Presets,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    pub id: i64,
    pub item_type: String,
    pub containers: Vec<Container>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateSummary {
    pub id: i64,
    pub item_type: String,
    pub container_count: i64,
}
