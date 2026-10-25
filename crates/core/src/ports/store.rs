use crate::Result;
use crate::domain::{
    item::{Item, NewItem},
    photo::{Classification, Integrity, NewPhoto, Photo, UndoSlot},
    session::{Session, SessionDto, SessionListRecord},
};

pub trait StoreTx {
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
    fn transaction(&self, op: &mut dyn FnMut(&mut dyn StoreTx) -> Result<()>) -> Result<()>;
    fn item(&self, id: i64) -> Result<Item>;
    fn session(&self, id: i64) -> Result<SessionDto>;
    fn session_list_records(&self) -> Result<Vec<SessionListRecord>>;
    fn photo_insert_or_get(&self, photo: NewPhoto) -> Result<Photo>;
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
