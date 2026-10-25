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
