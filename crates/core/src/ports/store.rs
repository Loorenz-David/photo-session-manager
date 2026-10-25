use crate::Result;
use crate::domain::{
    editing::Job,
    item::{Item, NewItem},
    photo::{Classification, Integrity, NewPhoto, Photo, UndoSlot},
    session::{Session, SessionDto, SessionListRecord},
};

#[derive(Debug, Clone)]
pub struct RawPreviewState {
    pub raw_path: String,
    pub adjusted_gen: i64,
    pub rendered_gen: i64,
    pub retouch_gen: i64,
    pub last_own_write_at: Option<i64>,
    pub adjusted_preview_path: Option<String>,
}

pub struct PreviewCommit {
    pub previous_path: Option<String>,
    pub photos: Vec<(i64, i64)>,
}

pub trait StoreTx {
    fn enqueue_mirror_tag(&mut self, raw_path: &str, photo_id: i64, now_ms: i64) -> Result<()>;
    fn session(&self, id: i64) -> Result<Session>;
    fn photo(&self, id: i64) -> Result<Photo>;
    fn photos(&self, session_id: i64) -> Result<Vec<Photo>>;
    fn set_photo_review(
        &mut self,
        id: i64,
        session_id: i64,
        classification: Classification,
        position: Option<i64>,
    ) -> Result<()>;
    fn set_positions(&mut self, positions: &[(i64, i64)]) -> Result<()>;
    fn copy_photo(
        &mut self,
        photo: &Photo,
        target_session: i64,
        position: Option<i64>,
    ) -> Result<i64>;
    fn set_undo_slot(&mut self, session_id: i64, slot: Option<&UndoSlot>) -> Result<()>;
    fn touch_session(&mut self, session_id: i64, at: i64) -> Result<()>;
    fn items(&self) -> Result<Vec<Item>>;
    fn item(&self, id: i64) -> Result<Item>;
    fn insert_item(&mut self, item: NewItem<'_>) -> Result<i64>;
    fn update_item_identity(
        &mut self,
        id: i64,
        sku: &str,
        folder_key: &str,
        barcode: Option<&str>,
        at: i64,
    ) -> Result<()>;
    fn item_has_photos(&self, item_id: i64) -> Result<bool>;
    fn insert_session(&mut self, item_id: i64, at: i64) -> Result<i64>;
    fn insert_item_link(
        &mut self,
        item_id: i64,
        integration: &str,
        external_id: &str,
        external_url: Option<&str>,
    ) -> Result<()>;
}

pub trait Store: Send + Sync {
    fn preview_states(&self) -> Result<Vec<RawPreviewState>>;
    fn preview_state(&self, raw: &str) -> Result<Option<RawPreviewState>>;
    fn ensure_initial_generation(&self, raw: &str) -> Result<Option<i64>>;
    fn adjusted_event(&self, raw: &str, retouch: bool) -> Result<Option<Vec<i64>>>;
    fn complete_preview(&self, raw: &str, generation: i64, path: &str) -> Result<PreviewCommit>;
    fn new_render_job(&self, raw: &str, now_ms: i64) -> Result<i64>;
    fn finish_render_job(&self, id: i64, error: Option<&str>, now_ms: i64) -> Result<()>;
    fn active_session_id(&self) -> Result<Option<i64>>;
    fn set_active_session_id(&self, id: Option<i64>) -> Result<()>;
    fn job_due(&self, now_ms: i64) -> Result<Option<Job>>;
    fn job_done(&self, id: i64, now_ms: i64) -> Result<()>;
    fn job_retry(&self, id: i64, code: &str, next_attempt_at: i64, now_ms: i64) -> Result<()>;
    fn requeue_running_jobs(&self) -> Result<()>;
    fn mark_own_write(&self, raw_path: &str, now_ms: i64) -> Result<()>;
    fn job(&self, id: i64) -> Result<Job>;
    fn transaction(&self, op: &mut dyn FnMut(&mut dyn StoreTx) -> Result<()>) -> Result<()>;
    fn item(&self, id: i64) -> Result<Item>;
    fn session(&self, id: i64) -> Result<SessionDto>;
    fn session_list_records(&self) -> Result<Vec<SessionListRecord>>;
    fn photo_insert_or_get(&self, photo: NewPhoto) -> Result<Photo>;
    fn arrival_insert(&self, photo: NewPhoto, path_key: &str) -> Result<(Photo, bool)>;
    fn session_folders(&self) -> Result<Vec<(String, i64)>>;
    fn session_for_item(&self, item_id: i64) -> Result<Option<i64>>;
    fn photos_in_session(&self, session_id: i64) -> Result<Vec<Photo>>;
    fn record_arrival_event(
        &self,
        source: &str,
        payload: &str,
        at: i64,
        error: Option<&str>,
    ) -> Result<i64>;
    fn set_capture_event_error(&self, id: i64, error: &str) -> Result<()>;
    fn set_raw_hash(&self, raw_path: &str, hash: &str) -> Result<()>;
    fn set_original_preview(
        &self,
        raw_path: &str,
        path: Option<&str>,
        orientation: Option<i64>,
    ) -> Result<()>;
    fn photo(&self, id: i64) -> Result<Photo>;
    fn set_photo_integrity(&self, id: i64, integrity: Integrity) -> Result<()>;
    fn upsert_raw_state(&self, raw_path: &str, path_key: Option<&str>) -> Result<()>;
    fn mechanism_version(&self) -> Result<String>;
    fn record_hook(
        &self,
        spool_name: &str,
        hook: &str,
        payload: &str,
        received_at: i64,
    ) -> Result<()>;
}
