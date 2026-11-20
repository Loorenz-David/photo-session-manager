//! Thin conversations commands over `ConversationsService` (C1; `master_plan_conversations.md` §4
//! Commands): Settings → Pricing. Each runs in `spawn_blocking` (a Keychain read or a request to
//! Pricing may take a while); none ever returns the key.

use psm_core::{
    domain::conversation::{
        Connection, ConversationList, ConversationRow, ConversationScope, ConversationTab,
    },
    services::conversations::{ConnectError, ConversationSettingsPatch},
};
use psm_integrations::{AppHandles, conversations::beyo_pricing::DEFAULT_SERVER};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::BTreeMap;
use tauri::State;

use crate::{
    commands::items::{CommandError, CommandId},
    state::AppState,
};

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

// ---------- C2: the panel's list, the header counts, the sessions-list badges ----------

/// Which conversations to list: `{kind: "all"}` or `{kind: "session", session_id}`.
#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConversationScopeDto {
    All,
    Session {
        #[specta(type = specta_typescript::Number)]
        session_id: i64,
    },
}

/// The panel's tab: `open` (unresolved) or `resolved` (the last 30 days).
#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ConversationTabDto {
    Open,
    Resolved,
}

/// One row of the panel (`status` = `open`, `answered` or `resolved`; `session_id` null = "Not on
/// this Mac").
#[derive(Debug, Serialize, Type)]
pub struct ConversationRowDto {
    pub id: String,
    pub sku: Option<String>,
    pub product_title: String,
    pub topic: String,
    pub status: String,
    pub blocking: bool,
    pub waiting: bool,
    pub last_line: Option<String>,
    #[specta(type = specta_typescript::Number)]
    pub last_activity_at: i64,
    pub has_news: bool,
    #[specta(type = Option<specta_typescript::Number>)]
    pub session_id: Option<i64>,
}

/// The panel's list: the connection `state`, its `problem` sentence when the key is refused or the
/// store is gone, the rows, and the conversations with news outside the scope.
#[derive(Debug, Serialize, Type)]
pub struct ConversationListDto {
    pub state: String,
    pub problem: Option<String>,
    pub rows: Vec<ConversationRowDto>,
    #[specta(type = specta_typescript::Number)]
    pub news_elsewhere: i64,
}

/// The header count: conversations with news here and in other sessions.
#[derive(Debug, Serialize, Type)]
pub struct ConversationCountsDto {
    #[specta(type = specta_typescript::Number)]
    pub here: i64,
    #[specta(type = specta_typescript::Number)]
    pub elsewhere: i64,
}

/// The sessions-list badge: "N open", or "Pricing is waiting" when `waiting`.
#[derive(Debug, Serialize, Type)]
pub struct SessionBadgeDto {
    #[specta(type = specta_typescript::Number)]
    pub session_id: i64,
    #[specta(type = specta_typescript::Number)]
    pub open: i64,
    pub waiting: bool,
}

impl From<ConversationScopeDto> for ConversationScope {
    fn from(scope: ConversationScopeDto) -> Self {
        match scope {
            ConversationScopeDto::All => Self::All,
            ConversationScopeDto::Session { session_id } => Self::Session(session_id),
        }
    }
}

impl From<ConversationTabDto> for ConversationTab {
    fn from(tab: ConversationTabDto) -> Self {
        match tab {
            ConversationTabDto::Open => Self::Open,
            ConversationTabDto::Resolved => Self::Resolved,
        }
    }
}

impl From<ConversationRow> for ConversationRowDto {
    fn from(r: ConversationRow) -> Self {
        Self {
            id: r.id,
            sku: r.sku,
            product_title: r.product_title,
            topic: r.topic,
            status: r.status.as_str().to_owned(),
            blocking: r.blocking,
            waiting: r.waiting,
            last_line: r.last_line,
            last_activity_at: r.last_activity_at,
            has_news: r.has_news,
            session_id: r.session_id,
        }
    }
}

impl From<ConversationList> for ConversationListDto {
    fn from(l: ConversationList) -> Self {
        Self {
            state: l.state.as_str().to_owned(),
            problem: l.problem,
            rows: l.rows.into_iter().map(Into::into).collect(),
            news_elsewhere: l.news_elsewhere,
        }
    }
}

/// The panel's rows from the local copy (never waits on the network).
#[tauri::command]
#[specta::specta]
pub async fn conversations_list(
    state: State<'_, AppState>,
    scope: ConversationScopeDto,
    tab: ConversationTabDto,
) -> Result<ConversationListDto, CommandError> {
    blocking(state, move |app| {
        let now = app.clock.now_ms();
        Ok(app
            .conversations
            .list(scope.into(), tab.into(), now)
            .map_err(CommandError::from)?
            .into())
    })
    .await
}

/// The header count for the session on screen (`session_id` null = all).
#[tauri::command]
#[specta::specta]
pub async fn conversations_counts(
    state: State<'_, AppState>,
    session_id: Option<CommandId>,
) -> Result<ConversationCountsDto, CommandError> {
    blocking(state, move |app| {
        let counts = app
            .conversations
            .counts(session_id.map(|id| id.0))
            .map_err(CommandError::from)?;
        Ok(ConversationCountsDto {
            here: counts.here,
            elsewhere: counts.elsewhere,
        })
    })
    .await
}

/// One badge per session with an unresolved conversation.
#[tauri::command]
#[specta::specta]
pub async fn conversations_session_badges(
    state: State<'_, AppState>,
) -> Result<Vec<SessionBadgeDto>, CommandError> {
    blocking(state, |app| {
        Ok(app
            .conversations
            .session_badges()
            .map_err(CommandError::from)?
            .into_iter()
            .map(|b| SessionBadgeDto {
                session_id: b.session_id,
                open: b.open,
                waiting: b.waiting,
            })
            .collect())
    })
    .await
}
