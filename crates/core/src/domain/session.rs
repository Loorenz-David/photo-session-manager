use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub item_id: Option<i64>,
    pub published_at: Option<i64>,
    pub published_seq: i64,
    pub change_seq: i64,
    pub last_activity_at: i64,
    pub created_at: i64,
    pub undo_slot: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDto {
    pub session: Session,
    pub dirty: bool,
    pub photo_count: i64,
    pub unreviewed_count: i64,
    pub main_count: i64,
    pub condition_count: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishStep {
    Prepare,
    Upload,
    Update,
    Describe,
    Done,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Publishing,
    Failed,
    Published,
    Ready,
    InProgress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RowLabel {
    Publishing(u8),
    PublishFailed,
    DescriptionFailed,
    Published,
    Ready,
    InProgress(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListFilter {
    Open,
    Published,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListCounts {
    pub open: i64,
    pub published: i64,
    pub all: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRow {
    pub id: i64,
    pub sku: String,
    pub item_type: String,
    pub title: String,
    pub main_count: i64,
    pub condition_count: i64,
    pub unreviewed_count: i64,
    pub status: SessionStatus,
    pub label: RowLabel,
    pub progress: Option<u8>,
    pub last_activity_at: i64,
    pub cover_preview_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListGroup<T> {
    pub label: String,
    pub rows: Vec<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionList {
    pub groups: Vec<ListGroup<SessionRow>>,
    pub counts: ListCounts,
    pub unassigned_count: i64,
}

#[derive(Debug, Clone)]
pub struct SessionListRecord {
    pub session: SessionDto,
    pub sku: String,
    pub item_type: String,
    pub title: String,
    pub non_deleted_missing_preview_count: i64,
    pub cover_preview_path: Option<String>,
    pub latest_step: Option<PublishStep>,
    pub failed_step: Option<PublishStep>,
    pub uploaded: i64,
    pub total: i64,
}
