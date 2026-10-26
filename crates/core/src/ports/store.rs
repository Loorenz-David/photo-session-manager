use crate::Result;
use crate::domain::{
    attribute::{AttributeGroup, Definition, Preset, Value as AttributeValue},
    editing::Job,
    item::{Item, NewItem},
    photo::{Classification, Integrity, NewPhoto, Photo, UndoSlot},
    publish::{DefinitionLink, Notification, PhotoLink, PublishItem},
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
    /// Sets the item's title (its one home, ch05) and its `updated_at`.
    fn update_item_title(&mut self, id: i64, title: &str, at: i64) -> Result<()>;
    fn item_has_photos(&self, item_id: i64) -> Result<bool>;
    fn insert_session(&mut self, item_id: i64, at: i64) -> Result<i64>;
    fn insert_item_link(
        &mut self,
        item_id: i64,
        integration: &str,
        external_id: &str,
        external_url: Option<&str>,
    ) -> Result<()>;
    fn replace_attribute_definition_links(
        &mut self,
        integration: &str,
        links: &[(String, DefinitionLink)],
    ) -> Result<()>;
    fn insert_publish_run(&mut self, integration: &str, at: i64, summary: &str) -> Result<i64>;
    fn insert_publish_item(&mut self, item: &PublishItem) -> Result<i64>;
    /// The stored definitions of one integration, in import order.
    fn attribute_definitions(&self, integration: &str) -> Result<Vec<Definition>>;
    /// Replaces the whole stored definition set of `integration` with `definitions` (in this order).
    fn replace_attribute_definitions(
        &mut self,
        integration: &str,
        definitions: &[Definition],
    ) -> Result<()>;
    /// Every stored value row of the item, cleared rows included, ordered by key.
    fn attribute_values(&self, item_id: i64) -> Result<Vec<AttributeValue>>;
    /// Inserts or replaces the row (`item_id`, `key`).
    fn put_attribute_value(&mut self, value: &AttributeValue) -> Result<()>;
    /// Sets `last_activity_at` of the item's sessions; `change_seq` is left as it is (MC-6).
    fn set_item_activity(&mut self, item_id: i64, at: i64) -> Result<()>;
    fn attribute_preset(&self, item_type: &str, group: AttributeGroup) -> Result<Option<Preset>>;
    fn put_attribute_preset(&mut self, preset: &Preset) -> Result<()>;
}

pub trait Store: Send + Sync {
    fn preview_states(&self) -> Result<Vec<RawPreviewState>>;
    fn preview_state(&self, raw: &str) -> Result<Option<RawPreviewState>>;
    fn ensure_initial_generation(&self, raw: &str) -> Result<Option<i64>>;
    fn adjusted_event(&self, raw: &str, retouch: bool) -> Result<Option<Vec<i64>>>;
    fn complete_preview(&self, raw: &str, generation: i64, path: &str) -> Result<PreviewCommit>;
    fn set_missing_preview_paths(&self, raw: &str, path: &str) -> Result<Vec<(i64, i64)>>;
    fn new_render_job(&self, raw: &str, now_ms: i64) -> Result<i64>;
    fn finish_render_job(&self, id: i64, error: Option<&str>, now_ms: i64) -> Result<()>;
    /// Records an export run (a running `render_export` job row whose payload is the frozen photo
    /// lists) and returns its id, the export run id.
    fn new_export_run(&self, session_id: i64, snapshot: &str, now_ms: i64) -> Result<i64>;
    /// The tracking row of one export render (`render_export`, target = the RAW).
    fn new_export_render_job(
        &self,
        raw: &str,
        run_id: i64,
        photo_id: i64,
        now_ms: i64,
    ) -> Result<i64>;
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
    /// The run's `confirmed_at` when the `publish_runs` row exists and is confirmed (07A MC-10).
    fn confirmed_run(&self, run_id: i64) -> Result<Option<i64>>;
    fn item_link(
        &self,
        item_id: i64,
        integration: &str,
    ) -> Result<Option<crate::domain::item::ItemLink>>;
    fn all_item_links(&self, integration: &str) -> Result<Vec<crate::domain::item::ItemLink>>;
    fn definition_links(&self, integration: &str) -> Result<Vec<(String, DefinitionLink)>>;
    fn publish_item(&self, item_id: i64) -> Result<PublishItem>;
    fn active_publish_items(&self) -> Result<Vec<i64>>;
    fn set_publish_step(
        &self,
        item_id: i64,
        step: crate::domain::session::PublishStep,
        failed_step: Option<crate::domain::session::PublishStep>,
        error: Option<&str>,
        at: i64,
    ) -> Result<()>;
    fn set_publish_progress(&self, item_id: i64, uploaded: i64, total: i64) -> Result<()>;
    fn enqueue_publish_item(&self, item_id: i64, at: i64) -> Result<()>;
    fn set_publish_export(&self, item_id: i64, export_run_id: i64) -> Result<()>;
    fn photo_link(&self, photo_id: i64, item_id: i64, role: &str) -> Result<Option<PhotoLink>>;
    fn upsert_photo_link(&self, link: &PhotoLink, at: i64) -> Result<()>;
    fn photo_links_for_item(&self, item_id: i64) -> Result<Vec<PhotoLink>>;
    fn complete_publish_item(&self, item_id: i64, at: i64, text: &str) -> Result<()>;
    fn notifications(&self, session_id: Option<i64>) -> Result<Vec<Notification>>;
    fn record_hook(
        &self,
        spool_name: &str,
        hook: &str,
        payload: &str,
        received_at: i64,
    ) -> Result<()>;
}
