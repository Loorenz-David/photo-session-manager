use crate::state::AppState;
use psm_core::{
    Error, domain::item::Item, ports::clock::SystemClock, services::items::ItemsService,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::BTreeMap;
use tauri::State;

#[derive(Serialize, Type)]
pub struct ItemDto {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub sku: String,
    pub folder_key: String,
    pub barcode: Option<String>,
    pub item_type: String,
    pub title: String,
    pub origin: String,
    #[specta(type = specta_typescript::Number)]
    pub created_at: i64,
    #[specta(type = specta_typescript::Number)]
    pub updated_at: i64,
}

#[derive(Serialize, Type)]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub data: BTreeMap<String, String>,
}

#[derive(Deserialize, Type)]
#[specta(type = specta_typescript::Number)]
pub struct CommandId(pub i64);

impl From<Error> for CommandError {
    fn from(error: Error) -> Self {
        let mut data = BTreeMap::new();
        let code = match &error {
            Error::SkuCollision { existing_sku } => {
                data.insert("existing_sku".into(), existing_sku.clone());
                "sku_collision"
            }
            Error::RelativePath => "relative_path",
            Error::InvalidValue(_) => "invalid_value",
            Error::NotFound(_) => "not_found",
            Error::Timeout => "timeout",
            Error::Io(_) => "io",
            Error::InvalidData(_) => "invalid_data",
            Error::Provider(_) => "provider",
            Error::TargetHasRaw => "target_has_raw",
            Error::TargetPublished => "target_published",
            Error::NothingToUndo => "nothing_to_undo",
            Error::CaptureFolderNotSet => "capture_folder_not_set",
            Error::HooksNotInstalled => "hooks_not_installed",
            Error::RecipeMissing { name } => {
                data.insert("name".into(), name.clone());
                "recipe_missing"
            }
            Error::TagVerification => "tag_verification",
            Error::VariantNotFound => "variant_not_found",
            Error::DocumentNotOpen => "document_not_open",
            Error::DocumentNotSession => "document_not_session",
            Error::RawChanged { photo_id } => {
                data.insert("photo_id".into(), photo_id.to_string());
                "raw_changed"
            }
            Error::RawMissing { photo_id } => {
                data.insert("photo_id".into(), photo_id.to_string());
                "raw_missing"
            }
        };
        Self {
            code: code.into(),
            message: error.to_string(),
            data,
        }
    }
}

impl From<Item> for ItemDto {
    fn from(item: Item) -> Self {
        Self {
            id: item.id,
            sku: item.sku,
            folder_key: item.folder_key,
            barcode: item.barcode,
            item_type: item.item_type,
            title: item.title,
            origin: match item.origin {
                psm_core::domain::item::ItemOrigin::Local => "local",
                psm_core::domain::item::ItemOrigin::Linked => "linked",
            }
            .into(),
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn item_create(
    state: State<'_, AppState>,
    sku: String,
    title: String,
    item_type: String,
    barcode: Option<String>,
) -> Result<ItemDto, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?;
    let service = ItemsService::new(app.store.as_ref(), &SystemClock);
    service
        .create_local(&sku, &title, &item_type, barcode.as_deref())
        .map(|(item, _)| item.into())
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub fn item_update_identity(
    state: State<'_, AppState>,
    item_id: CommandId,
    sku: Option<String>,
    barcode: Option<String>,
) -> Result<ItemDto, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?;
    ItemsService::new(app.store.as_ref(), &SystemClock)
        .update_identity(item_id.0, sku.as_deref(), barcode.as_deref())
        .map(Into::into)
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub fn items_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<ItemDto>, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?;
    ItemsService::new(app.store.as_ref(), &SystemClock)
        .search(&query)
        .map(|items| items.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub fn items_next_sku(state: State<'_, AppState>) -> Result<Option<String>, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?;
    ItemsService::new(app.store.as_ref(), &SystemClock)
        .next_sku(app.settings.sku_prefix.as_deref())
        .map_err(Into::into)
}
