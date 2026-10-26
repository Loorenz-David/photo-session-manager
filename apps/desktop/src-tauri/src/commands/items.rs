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
    pub external_id: Option<String>,
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

#[derive(Serialize, Deserialize, Type)]
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
            Error::SkuInShop { sku } => {
                data.insert("sku".into(), sku.clone());
                "sku_in_shop"
            }
            Error::NoIntegration => "no_integration",
            Error::SeveralProductsCarrySku { sku } => {
                data.insert("sku".into(), sku.clone());
                "several_products_carry_sku"
            }
            Error::ForeignMediaAdded { n } => {
                data.insert("n".into(), n.to_string());
                "foreign_media_added"
            }
            Error::GalleryVerification => "gallery_verification",
            Error::UploadIncomplete => "upload_incomplete",
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
            Error::AttributeVerification { keys } => {
                data.insert("keys".into(), keys.join(","));
                "attribute_verification"
            }
            Error::NotConfirmed => "not_confirmed",
            Error::ShopNotPermitted => "shop_not_permitted",
            Error::AppNotInstalled => "app_not_installed",
            Error::CredentialsRejected => "credentials_rejected",
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
            external_id: item.external_id,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn item_create(
    state: State<'_, AppState>,
    sku: String,
    title: String,
    item_type: String,
    barcode: Option<String>,
) -> Result<ItemDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<ItemDto, CommandError> {
        let service = ItemsService::new(app.store.as_ref(), &SystemClock)
            .with_commerce(app.commerce.as_deref());
        service
            .create_local(&sku, &title, &item_type, barcode.as_deref())
            .map(|(item, _)| item.into())
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn item_update_identity(
    state: State<'_, AppState>,
    item_id: CommandId,
    sku: Option<String>,
    barcode: Option<String>,
) -> Result<ItemDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<ItemDto, CommandError> {
        ItemsService::new(app.store.as_ref(), &SystemClock)
            .with_commerce(app.commerce.as_deref())
            .update_identity(item_id.0, sku.as_deref(), barcode.as_deref())
            .map(Into::into)
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn items_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<ItemDto>, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<ItemDto>, CommandError> {
        ItemsService::new(app.store.as_ref(), &SystemClock)
            .search_local(&query)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn items_search_shop(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<ItemDto>, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<ItemDto>, CommandError> {
        ItemsService::new(app.store.as_ref(), &SystemClock)
            .with_commerce(app.commerce.as_deref())
            .search_shop(&query)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn items_next_sku(state: State<'_, AppState>) -> Result<Option<String>, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<Option<String>, CommandError> {
        ItemsService::new(app.store.as_ref(), &SystemClock)
            .with_commerce(app.commerce.as_deref())
            .next_sku(app.settings.sku_prefix.as_deref())
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn item_adopt(
    state: State<'_, AppState>,
    external_id: String,
) -> Result<ItemDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<ItemDto, CommandError> {
        ItemsService::new(app.store.as_ref(), &SystemClock)
            .with_commerce(app.commerce.as_deref())
            .adopt(&external_id)
            .map(Into::into)
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}
