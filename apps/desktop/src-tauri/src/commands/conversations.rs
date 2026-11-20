//! Thin conversations commands over `ConversationsService` (C1; `master_plan_conversations.md` §4
//! Commands): Settings → Pricing. Each runs in `spawn_blocking` (a Keychain read or a request to
//! Pricing may take a while); none ever returns the key.

use psm_core::{
    domain::conversation::{
        Attachable, Connection, ConversationList, ConversationRow, ConversationScope,
        ConversationTab, ConversationView, Message, NewMessage, PendingView, ProviderError,
        ReplyRef,
    },
    services::conversations::{ConnectError, ConversationError, ConversationSettingsPatch},
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

// ---------- C3: the conversation view, replies, Mark answered, attachable photos ----------

/// `conversations_off` (`data.reason` = the connection state; the message is the state's
/// sentence when the key is refused or the store is gone), `conversation_resolved`, `provider`
/// (the adapter's words), or a local error (`invalid_value`, `not_found`, …).
fn conversation_error(app: &AppHandles, error: ConversationError) -> CommandError {
    match error {
        ConversationError::Off { state, message } => CommandError {
            code: "conversations_off".into(),
            message,
            data: BTreeMap::from([("reason".to_owned(), state.as_str().to_owned())]),
        },
        ConversationError::Resolved => CommandError {
            code: "conversation_resolved".into(),
            message: app.conversations.describe(&ProviderError::Resolved),
            data: BTreeMap::new(),
        },
        ConversationError::Provider(error) => CommandError {
            code: "provider".into(),
            message: app.conversations.describe(&error),
            data: BTreeMap::new(),
        },
        ConversationError::Storage(error) => error.into(),
    }
}

/// The message a reply answers: its author and the target's first line (at most 80 characters).
#[derive(Debug, Serialize, Type)]
pub struct ReplyRefDto {
    pub message_id: String,
    pub author_name: String,
    pub excerpt: String,
}

/// The quoted field of a message (`field_label` is shown as the chip). C4: `jump_key` = the form
/// field the chip opens, when this Mac's form of the conversation's session has it (view-only).
#[derive(Debug, Serialize, Type)]
pub struct QuoteDto {
    pub field: String,
    pub field_label: String,
    pub text: String,
    /// Absent when the chip opens nothing (as P69's `masters_ready`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jump_key: Option<String>,
}

/// A photo of a message; both addresses null = "Photo not recorded".
#[derive(Debug, Serialize, Type)]
pub struct AttachmentDto {
    pub media_id: String,
    pub thumb_url: Option<String>,
    pub url: Option<String>,
}

/// `kind` = `asker` (the pricer), `me` (this app) or `other`.
#[derive(Debug, Serialize, Type)]
pub struct AuthorDto {
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Serialize, Type)]
pub struct MessageDto {
    pub id: String,
    pub author: AuthorDto,
    pub text: String,
    pub quote: Option<QuoteDto>,
    pub reply_to: Option<ReplyRefDto>,
    pub attachments: Vec<AttachmentDto>,
    #[specta(type = specta_typescript::Number)]
    pub created_at: i64,
    #[specta(type = Option<specta_typescript::Number>)]
    pub edited_at: Option<i64>,
}

/// A photo of a reply not yet confirmed; `preview_path` when this Mac has the photo.
#[derive(Debug, Serialize, Type)]
pub struct PendingAttachmentDto {
    pub media_id: String,
    pub preview_path: Option<String>,
}

/// A reply not yet confirmed: `state` = `sending`, `failed` (with `failure`) or `sent`.
#[derive(Debug, Serialize, Type)]
pub struct PendingViewDto {
    pub client_mutation_id: String,
    pub text: String,
    pub reply_to: Option<ReplyRefDto>,
    pub attachments: Vec<PendingAttachmentDto>,
    pub state: String,
    pub failure: Option<String>,
    #[specta(type = specta_typescript::Number)]
    pub created_at: i64,
}

/// One conversation: `status` = `open`, `answered` or `resolved`; `resolved_by` = `asker`,
/// `activation` or null; `session_id` null = "Not on this Mac"; `can_send` false with
/// `off_reason` when the key is refused or the store is gone.
#[derive(Debug, Serialize, Type)]
pub struct ConversationViewDto {
    pub id: String,
    pub sku: Option<String>,
    pub product_title: String,
    pub topic: String,
    pub blocking: bool,
    pub status: String,
    pub resolved_by: Option<String>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub session_id: Option<i64>,
    pub messages: Vec<MessageDto>,
    pub pending: Vec<PendingViewDto>,
    pub can_send: bool,
    pub off_reason: Option<String>,
}

/// A published photo a reply may attach: `slot` = `single`, `left` or `right`; `role` =
/// `gallery` or `condition`.
#[derive(Debug, Serialize, Type)]
pub struct AttachablePhotoDto {
    pub media_id: String,
    #[specta(type = specta_typescript::Number)]
    pub photo_id: i64,
    pub slot: String,
    pub role: String,
    pub preview_path: Option<String>,
}

/// The photos a reply may attach, or why none is: `reason` = `not_on_this_mac` or
/// `nothing_published`.
#[derive(Debug, Serialize, Type)]
pub struct AttachableDto {
    pub photos: Vec<AttachablePhotoDto>,
    pub reason: Option<String>,
}

/// A reply: `client_mutation_id` is made once by the composer when send is pressed.
#[derive(Debug, Deserialize, Type)]
pub struct ConversationReplyDto {
    pub conversation_id: String,
    pub client_mutation_id: String,
    pub text: String,
    pub reply_to_message_id: Option<String>,
    pub attachment_media_ids: Vec<String>,
}

fn snake<T: Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(text)) => text,
        _ => String::new(),
    }
}

impl From<ReplyRef> for ReplyRefDto {
    fn from(r: ReplyRef) -> Self {
        Self {
            message_id: r.message_id,
            author_name: r.author_name,
            excerpt: r.excerpt,
        }
    }
}

impl From<Message> for MessageDto {
    fn from(m: Message) -> Self {
        Self {
            id: m.id,
            author: AuthorDto {
                kind: m.author.kind.as_str().to_owned(),
                name: m.author.name,
            },
            text: m.text,
            quote: m.quote.map(|q| QuoteDto {
                field: q.field,
                field_label: q.field_label,
                text: q.text,
                jump_key: None,
            }),
            reply_to: m.reply_to.map(Into::into),
            attachments: m
                .attachments
                .into_iter()
                .map(|a| AttachmentDto {
                    media_id: a.media_id,
                    thumb_url: a.thumb_url,
                    url: a.url,
                })
                .collect(),
            created_at: m.created_at,
            edited_at: m.edited_at,
        }
    }
}

impl From<PendingView> for PendingViewDto {
    fn from(p: PendingView) -> Self {
        Self {
            client_mutation_id: p.client_mutation_id,
            text: p.text,
            reply_to: p.reply_to.map(Into::into),
            attachments: p
                .attachments
                .into_iter()
                .map(|a| PendingAttachmentDto {
                    media_id: a.media_id,
                    preview_path: a.preview_path,
                })
                .collect(),
            state: snake(&p.state),
            failure: p.failure,
            created_at: p.created_at,
        }
    }
}

impl From<ConversationView> for ConversationViewDto {
    fn from(v: ConversationView) -> Self {
        Self {
            id: v.id,
            sku: v.sku,
            product_title: v.product_title,
            topic: v.topic,
            blocking: v.blocking,
            status: v.status.as_str().to_owned(),
            resolved_by: v.resolved_by.map(|r| r.as_str().to_owned()),
            session_id: v.session_id,
            messages: v
                .messages
                .into_iter()
                .map(|m| {
                    let jump_key = v.jump_keys.get(&m.id).cloned();
                    let mut dto = MessageDto::from(m);
                    if let Some(quote) = dto.quote.as_mut() {
                        quote.jump_key = jump_key;
                    }
                    dto
                })
                .collect(),
            pending: v.pending.into_iter().map(Into::into).collect(),
            can_send: v.can_send,
            off_reason: v.off_reason,
        }
    }
}

impl From<Attachable> for AttachableDto {
    fn from(a: Attachable) -> Self {
        Self {
            photos: a
                .photos
                .into_iter()
                .map(|p| AttachablePhotoDto {
                    media_id: p.media_id,
                    photo_id: p.photo_id,
                    slot: snake(&p.slot),
                    role: snake(&p.role),
                    preview_path: p.preview_path,
                })
                .collect(),
            reason: a.reason.map(|r| snake(&r)),
        }
    }
}

/// One conversation from the local copy (never waits on the network).
#[tauri::command]
#[specta::specta]
pub async fn conversation_get(
    state: State<'_, AppState>,
    id: String,
) -> Result<ConversationViewDto, CommandError> {
    blocking(state, move |app| {
        Ok(app
            .conversations
            .view(&id)
            .map_err(|e| conversation_error(app, e))?
            .into())
    })
    .await
}

/// The conversation was seen: its read mark moves to its newest message.
#[tauri::command]
#[specta::specta]
pub async fn conversation_mark_read(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), CommandError> {
    blocking(state, move |app| {
        app.conversations.mark_read(&id).map_err(CommandError::from)
    })
    .await
}

/// Saves a reply on this Mac and answers it as `sending` (it leaves at the next quarter
/// second); the same `client_mutation_id` again answers the saved reply, nothing is added.
#[tauri::command]
#[specta::specta]
pub async fn conversation_reply(
    state: State<'_, AppState>,
    input: ConversationReplyDto,
) -> Result<PendingViewDto, CommandError> {
    blocking(state, move |app| {
        let pending = app
            .conversations
            .reply(
                &input.conversation_id,
                NewMessage {
                    client_mutation_id: input.client_mutation_id,
                    text: input.text,
                    reply_to_message_id: input.reply_to_message_id,
                    attachment_media_ids: input.attachment_media_ids,
                },
            )
            .map_err(|e| conversation_error(app, e))?;
        Ok(pending.into())
    })
    .await
}

/// Sends a failed reply again, with the same `client_mutation_id`.
#[tauri::command]
#[specta::specta]
pub async fn conversation_retry(
    state: State<'_, AppState>,
    client_mutation_id: String,
) -> Result<PendingViewDto, CommandError> {
    blocking(state, move |app| {
        Ok(app
            .conversations
            .retry(&client_mutation_id)
            .map_err(|e| conversation_error(app, e))?
            .into())
    })
    .await
}

/// Removes a failed reply from this Mac (no request).
#[tauri::command]
#[specta::specta]
pub async fn conversation_discard(
    state: State<'_, AppState>,
    client_mutation_id: String,
) -> Result<(), CommandError> {
    blocking(state, move |app| {
        app.conversations
            .discard(&client_mutation_id)
            .map_err(|e| conversation_error(app, e))
    })
    .await
}

/// Marks the conversation answered in Pricing without a message.
#[tauri::command]
#[specta::specta]
pub async fn conversation_mark_answered(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), CommandError> {
    blocking(state, move |app| {
        app.conversations
            .mark_answered(&id)
            .map_err(|e| conversation_error(app, e))
    })
    .await
}

/// The published photos of the conversation's product a reply may attach.
#[tauri::command]
#[specta::specta]
pub async fn conversation_attachable(
    state: State<'_, AppState>,
    id: String,
) -> Result<AttachableDto, CommandError> {
    blocking(state, move |app| {
        Ok(app
            .conversations
            .attachable(&id)
            .map_err(|e| conversation_error(app, e))?
            .into())
    })
    .await
}
