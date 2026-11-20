//! Thin conversations commands over `ConversationsService` (C1; `master_plan_conversations.md` §4
//! Commands): Settings → Pricing. Each runs in `spawn_blocking` (a Keychain read or a request to
//! Pricing may take a while); none ever returns the key.

use psm_core::{
    domain::conversation::Connection,
    services::conversations::{ConnectError, ConversationSettingsPatch},
};
use psm_integrations::{AppHandles, conversations::beyo_pricing::DEFAULT_SERVER};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::BTreeMap;
use tauri::State;

use crate::{commands::items::CommandError, state::AppState};

/// The connection as Settings shows it (`state` = `no_key`, `no_shop`, `store_unmatched`, `ready`,
/// `refused` or `store_gone`).
#[derive(Debug, Serialize, Type)]
pub struct ConnectionDto {
    pub state: String,
    pub store_name: Option<String>,
    pub shop: Option<String>,
    pub visible_stores: Vec<String>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub last_ok_at: Option<i64>,
    pub last_error: Option<String>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub last_error_at: Option<i64>,
}

/// Settings → Pricing. `key_saved` only: the key itself never leaves the Keychain.
#[derive(Debug, Serialize, Type)]
pub struct ConversationsSettingsDto {
    /// The effective server address.
    pub server: String,
    pub server_default: String,
    pub key_saved: bool,
    pub sound: bool,
    pub banner: bool,
    pub connection: ConnectionDto,
}

/// The fields to save; the others are kept. `server: ""` returns to the default address.
#[derive(Debug, Default, Deserialize, Type)]
pub struct ConversationsSettingsPatchDto {
    pub server: Option<String>,
    pub sound: Option<bool>,
    pub banner: Option<bool>,
}

impl From<Connection> for ConnectionDto {
    fn from(c: Connection) -> Self {
        Self {
            state: c.state.as_str().to_owned(),
            store_name: c.store.map(|s| s.name),
            shop: c.shop,
            visible_stores: c.visible_stores,
            last_ok_at: c.last_ok_at,
            last_error: c.last_error,
            last_error_at: c.last_error_at,
        }
    }
}

fn settings_dto(app: &AppHandles) -> Result<ConversationsSettingsDto, CommandError> {
    let service = &app.conversations;
    let settings = service.settings();
    let connection = service.connection()?;
    Ok(ConversationsSettingsDto {
        server: settings
            .server
            .clone()
            .unwrap_or_else(|| DEFAULT_SERVER.to_owned()),
        server_default: DEFAULT_SERVER.to_owned(),
        key_saved: service.key_saved(),
        sound: settings.sound,
        banner: settings.banner,
        connection: connection.into(),
    })
}

/// `Unreachable`/`BadReply` of `stores()`: the existing code `provider`, in the adapter's words.
fn connect_error(app: &AppHandles, error: ConnectError) -> CommandError {
    match error {
        ConnectError::Provider(error) => CommandError {
            code: "provider".into(),
            message: app.conversations.describe(&error),
            data: BTreeMap::new(),
        },
        ConnectError::Storage(error) => error.into(),
    }
}

async fn blocking<T: Send + 'static>(
    state: State<'_, AppState>,
    work: impl FnOnce(&AppHandles) -> Result<T, CommandError> + Send + 'static,
) -> Result<T, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || work(&app))
        .await
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn conversations_settings_get(
    state: State<'_, AppState>,
) -> Result<ConversationsSettingsDto, CommandError> {
    blocking(state, settings_dto).await
}

#[tauri::command]
#[specta::specta]
pub async fn conversations_settings_save(
    state: State<'_, AppState>,
    patch: ConversationsSettingsPatchDto,
) -> Result<ConversationsSettingsDto, CommandError> {
    blocking(state, move |app| {
        app.conversations
            .settings_save(&ConversationSettingsPatch {
                server: patch.server,
                sound: patch.sound,
                banner: patch.banner,
            })
            .map_err(CommandError::from)?;
        settings_dto(app)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn conversations_key_set(
    state: State<'_, AppState>,
    key: String,
) -> Result<ConversationsSettingsDto, CommandError> {
    blocking(state, move |app| {
        app.conversations
            .key_set(Some(&key))
            .map_err(|e| connect_error(app, e))?;
        settings_dto(app)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn conversations_test(
    state: State<'_, AppState>,
) -> Result<ConversationsSettingsDto, CommandError> {
    blocking(state, |app| {
        app.conversations
            .test_connection()
            .map_err(|e| connect_error(app, e))?;
        settings_dto(app)
    })
    .await
}
