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
    /// The photographer's name for the shot angle ("Front, low"); `None` shows "Container n".
    pub name: Option<String>,
    /// How to take the shot, shown behind an info button. Display only.
    pub description: Option<String>,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceRow {
    pub photo_id: i64,
    pub slot: Option<i64>,
    pub place_by_hand: bool,
    pub frame_by_hand: bool,
    pub released: bool,
    pub key_at: Option<i64>,
    pub key_name: Option<String>,
    pub pending_zoom: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct ShotKey {
    pub photo_id: i64,
    pub at: i64,
    pub name: Option<String>,
    pub released: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplatePhotoView {
    pub photo_id: i64,
    pub slot: Option<i64>,
    pub by_hand: bool,
    pub differs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateView {
    pub templated: bool,
    pub template_id: Option<i64>,
    pub containers: Vec<Container>,
    pub photos: Vec<TemplatePhotoView>,
    pub differs: i64,
    pub can_shift_back: bool,
    /// The shooting number (`shoot_no`) of the container the next shot goes to, or `None` when the
    /// next shot lands after the last container (Condition). Display only (10G §5 guidance).
    pub next_shot: Option<i64>,
}
