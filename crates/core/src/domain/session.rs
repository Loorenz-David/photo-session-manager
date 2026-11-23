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
    pub arranged_at: Option<i64>,
    pub undo_slot: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDto {
    pub session: Session,
    pub dirty: bool,
    /// A10: why a published session is open again (set only while `dirty`).
    pub changes: Option<PublishedChanges>,
    pub photo_count: i64,
    pub unreviewed_count: i64,
    pub main_count: i64,
    pub condition_count: i64,
}

/// Owner correction A10 (2026-11-18): what changed in a published session since it was
/// published, so it never reopens without a reason on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PublishedChanges {
    /// Photos (not denied) that arrived after the publish.
    pub new_photos: i64,
    /// Photos of the session whose adjustments changed in Capture One after the publish.
    pub editor_edits: i64,
    /// Something was changed in the app (review, move, crop by hand, template action, undo).
    pub app_edits: bool,
    /// Changed before this version recorded reasons: the reason is not known.
    pub unrecorded: bool,
}

/// What the session workspace shows: the session, its item (none for Unassigned) and its visible photos
/// (every classification except `deleted`, which is hidden, ch06).
#[derive(Debug, Clone, PartialEq)]
pub struct SessionView {
    pub session: SessionDto,
    pub item: Option<crate::domain::item::Item>,
    pub photos: Vec<crate::domain::photo::Photo>,
    pub compositions: Vec<crate::domain::composition::Composition>,
    pub publish: Option<LatestPublish>,
    pub upload_states: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatestPublish {
    pub item_id: i64,
    pub step: PublishStep,
    pub failed_step: Option<PublishStep>,
    pub error: Option<String>,
    pub uploaded: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishStep {
    Held,
    Prepare,
    Upload,
    Update,
    Describe,
    Done,
    Failed,
    /// Changelog 44: the photographer cancelled the publish; nothing on the platform was undone.
    Cancelled,
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
    WaitingForYou(i64),
    Publishing(u8),
    PublishFailed,
    /// Changelog 44: `Publish cancelled · Publish`.
    PublishCancelled,
    DescriptionFailed,
    Published,
    Ready,
    InProgress(i64),
    /// P62 (10F §3): published, local files cleared, photos shown from Shopify.
    Archived,
    /// Changelog 45 (P65): a publish waits for Capture One to load its photos.
    WaitingForEditor,
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
    /// When the session was created: the list's order and time groups (changelog 46 (1)).
    pub created_at: i64,
    pub arranged: bool,
    /// MC-H1.1: the row picture; never a path whose file is missing (M-H4).
    pub cover_preview_path: Option<String>,
    /// MC-H1.1: the row's item is remote and its cover is not on this Mac yet.
    pub cover_pending: bool,
    pub publish: Option<LatestPublish>,
    /// A10: why a published session is open again.
    pub changes: Option<PublishedChanges>,
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

/// MC-H2.1: one group of a page: its rows on the page and its row count before the cut.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageGroup {
    pub label: String,
    pub total: usize,
    pub rows: Vec<SessionRow>,
}

/// MC-H2.1: one page of the sessions list (`counts` and `unassigned_count` over all sessions).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPage {
    pub groups: Vec<PageGroup>,
    pub counts: ListCounts,
    pub unassigned_count: i64,
    pub page: u32,
    pub pages: u32,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct SessionListRecord {
    pub session: SessionDto,
    pub sku: String,
    pub item_type: String,
    pub title: String,
    pub non_deleted_missing_preview_count: i64,
    /// The stored preview paths of the main photo at position 1 (MC-H1.1(i); the files may be
    /// gone).
    pub cover_adjusted_preview_path: Option<String>,
    pub cover_original_preview_path: Option<String>,
    pub latest_step: Option<PublishStep>,
    pub failed_step: Option<PublishStep>,
    pub uploaded: i64,
    pub total: i64,
    pub unresolved_conflicts: i64,
    pub publish: Option<LatestPublish>,
    /// Changelog 45 (P65): the session's export run waits for the editor.
    pub waiting_for_editor: bool,
}
