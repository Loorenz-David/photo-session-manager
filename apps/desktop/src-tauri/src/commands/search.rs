//! Owner correction A12: the "Search this photo" window's commands. Only the app's own pages can
//! call them (Tauri refuses commands from remote pages); the page webview is driven from here.

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use base64::Engine;
use psm_core::{
    Error,
    domain::attribute::{CoreType, Provenance},
    events::AppEvent,
    ports::{clock::Clock, events::EventSink, store::Store},
};
use psm_research::{
    api::ResearchApi,
    browse::{AskMode, AskRequest, VisualMatch},
    rules::value_map::{Mapped, map_output},
};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, State};

use crate::{
    commands::{
        attributes::{AttributeValueDto, integration, with_service},
        items::{CommandError, CommandId},
    },
    search_window::{
        Bounds, GOOGLE_START, LensPhase, MAX_SELECTION_CHARS, SELECTION_SCRIPT, SearchWindows,
        allowed_navigation, next_zoom, parse_selection_report, web_bounds, web_label, window,
    },
    state::AppState,
};

fn refused(code: &str, message: impl Into<String>) -> CommandError {
    CommandError {
        code: code.into(),
        message: message.into(),
        data: BTreeMap::new(),
    }
}

fn joined(e: impl std::fmt::Display) -> CommandError {
    CommandError::from(Error::InvalidData(e.to_string()))
}

fn tauri_error(e: tauri::Error) -> CommandError {
    CommandError::from(Error::InvalidData(e.to_string()))
}

fn engine(state: &State<'_, AppState>) -> Result<Arc<dyn ResearchApi>, CommandError> {
    state
        .handles()
        .map_err(CommandError::from)?
        .research
        .ok_or_else(|| refused("research_unavailable", "Research is not available"))
}

/// A search tab's page webview (A18: one per tab).
fn page(app: &AppHandle, session_id: i64, tab: i64) -> Result<tauri::Webview, CommandError> {
    app.get_webview(&web_label(session_id, tab))
        .ok_or_else(|| refused("search_window_closed", "This search tab is closed"))
}

/// A search tab's page webview, made when the tab is new (A18).
fn tab_page(app: &AppHandle, session_id: i64, tab: i64) -> Result<tauri::Webview, CommandError> {
    window::ensure_tab(app, session_id, tab).map_err(|why| refused("search_tab_refused", why))
}

/// A web page address the page webview may open.
fn web_url(url: &str) -> Result<tauri::Url, CommandError> {
    let parsed: tauri::Url = url
        .trim()
        .parse()
        .map_err(|_| refused("not_a_web_page", "This is not a web page address"))?;
    if !matches!(parsed.scheme(), "http" | "https") || !allowed_navigation(parsed.as_str()) {
        return Err(refused("not_a_web_page", "This is not a web page address"));
    }
    Ok(parsed)
}

/// Opens the session's search window pinned on `photo_id` (or brings it forward on that photo).
#[tauri::command]
#[specta::specta]
pub async fn search_window_open(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: CommandId,
    photo_id: CommandId,
) -> Result<(), CommandError> {
    let handles = state.handles().map_err(CommandError::from)?;
    let item_id = Store::session(handles.store.as_ref(), session_id.0)
        .map_err(CommandError::from)?
        .session
        .item_id
        .ok_or_else(|| refused("no_item", "This session has no item"))?;
    let item = Store::item(handles.store.as_ref(), item_id).map_err(CommandError::from)?;
    let name = if item.item_type.is_empty() {
        format!("{} · {}", item.title, item.sku)
    } else {
        format!("{} · {}", item.item_type, item.sku)
    };
    crate::search_window::window::open(
        &app,
        session_id.0,
        photo_id.0,
        &format!("Search this photo — {name}"),
    )
    .map_err(tauri_error)
}

/// Where the browser area is in the app's page (CSS pixels of the app's page, i.e. logical
/// points), and whether the current tab's page webview shows (hidden while the window's own
/// results list shows). A13: moved by where the app's page sits in the window (below the title
/// bar). A18: `tab` is the current tab; every other tab's page webview is hidden.
#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_arguments)]
pub async fn search_web_place(
    app: AppHandle,
    windows: State<'_, SearchWindows>,
    session_id: CommandId,
    tab: CommandId,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    visible: bool,
) -> Result<(), CommandError> {
    let web = tab_page(&app, session_id.0, tab.0)?;
    let current = web_label(session_id.0, tab.0);
    for other in window::tabs(&app, session_id.0) {
        if other.label() != current {
            let _ = other.hide();
        }
    }
    let ui = windows.layout(session_id.0).unwrap_or(Bounds {
        x: 0.0,
        y: 0.0,
        width: f64::MAX,
        height: f64::MAX,
    });
    let placed = web_bounds(
        Bounds {
            x,
            y,
            width,
            height,
        },
        ui,
    );
    web.set_position(LogicalPosition::new(placed.x, placed.y))
        .map_err(tauri_error)?;
    web.set_size(LogicalSize::new(placed.width, placed.height))
        .map_err(tauri_error)?;
    if visible {
        web.show().map_err(tauri_error)?;
    } else {
        web.hide().map_err(tauri_error)?;
    }
    Ok(())
}

/// Opens a web page in a tab of the window's browser.
#[tauri::command]
#[specta::specta]
pub async fn search_web_go(
    app: AppHandle,
    session_id: CommandId,
    tab: CommandId,
    url: String,
) -> Result<(), CommandError> {
    let url = web_url(&url)?;
    tab_page(&app, session_id.0, tab.0)?
        .navigate(url)
        .map_err(tauri_error)
}

/// Back, forward or reload in a tab of the window's browser.
#[tauri::command]
#[specta::specta]
pub async fn search_web_step(
    app: AppHandle,
    session_id: CommandId,
    tab: CommandId,
    action: String,
) -> Result<(), CommandError> {
    let web = page(&app, session_id.0, tab.0)?;
    match action.as_str() {
        "back" => web.eval("history.back()").map_err(tauri_error),
        "forward" => web.eval("history.forward()").map_err(tauri_error),
        "reload" => web.reload().map_err(tauri_error),
        _ => Err(refused("unknown_action", "Unknown browser action")),
    }
}

/// A18: page zoom of a tab: the step after `action` (`in`, `out`, `reset`) from `current` (the
/// zoom the app's page holds for the tab), applied to the tab's page; answers the new zoom.
#[tauri::command]
#[specta::specta]
pub async fn search_web_zoom(
    app: AppHandle,
    session_id: CommandId,
    tab: CommandId,
    current: f64,
    action: String,
) -> Result<f64, CommandError> {
    let zoom = next_zoom(current, &action);
    page(&app, session_id.0, tab.0)?
        .set_zoom(zoom)
        .map_err(tauri_error)?;
    tracing::debug!(
        target: "psm::search",
        "search_window.zoom session={} tab={} zoom={zoom}",
        session_id.0,
        tab.0
    );
    Ok(zoom)
}

/// A18: closes a search tab (its page and its Google route).
#[tauri::command]
#[specta::specta]
pub async fn search_tab_close(
    app: AppHandle,
    session_id: CommandId,
    tab: CommandId,
) -> Result<(), CommandError> {
    window::close_tab(&app, session_id.0, tab.0);
    Ok(())
}

/// Searches the photo with Google Lens in a search tab (A18: made when new): Google's page opens
/// and the photo is posted from it (`search_lens` events follow).
#[tauri::command]
#[specta::specta]
pub async fn search_lens_start(
    app: AppHandle,
    state: State<'_, AppState>,
    windows: State<'_, SearchWindows>,
    session_id: CommandId,
    tab: CommandId,
    photo_id: CommandId,
) -> Result<(), CommandError> {
    let handles = state.handles().map_err(CommandError::from)?;
    let (sid, pid) = (session_id.0, photo_id.0);
    let jpeg = tauri::async_runtime::spawn_blocking(move || {
        psm_research::browse::lens_image(
            handles.store.as_ref(),
            &psm_infra::imaging::ImageOpsImpl,
            &handles.paths.root,
            sid,
            pid,
        )
    })
    .await
    .map_err(joined)?
    .map_err(CommandError::from)?;
    let web = tab_page(&app, sid, tab.0)?;
    windows.start(
        sid,
        tab.0,
        pid,
        base64::engine::general_purpose::STANDARD.encode(&jpeg),
    );
    tracing::info!(
        target: "psm::search",
        "search_window.lens_route step=start session={sid} tab={} photo={pid} bytes={}",
        tab.0,
        jpeg.len()
    );
    web.navigate(GOOGLE_START.parse().map_err(joined)?)
        .map_err(tauri_error)
}

/// The window gave up waiting for Google (no results in time): the route is over and the window
/// shows its own list.
#[tauri::command]
#[specta::specta]
pub async fn search_lens_give_up(
    windows: State<'_, SearchWindows>,
    session_id: CommandId,
    tab: CommandId,
    why: String,
) -> Result<(), CommandError> {
    let why: String = why.chars().take(120).collect();
    windows.fail(session_id.0, tab.0, &why);
    tracing::info!(
        target: "psm::search",
        "search_window.lens_route result=fallback session={} tab={} why={why:?}",
        session_id.0,
        tab.0
    );
    Ok(())
}

/// Back to Google's results page of the tab's search.
#[tauri::command]
#[specta::specta]
pub async fn search_lens_results(
    app: AppHandle,
    windows: State<'_, SearchWindows>,
    session_id: CommandId,
    tab: CommandId,
) -> Result<bool, CommandError> {
    match windows.phase(session_id.0, tab.0) {
        Some(LensPhase::Results { url }) => {
            page(&app, session_id.0, tab.0)?
                .navigate(web_url(&url)?)
                .map_err(tauri_error)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[derive(Serialize, Type)]
pub struct SelectionDto {
    pub text: String,
    pub url: String,
    pub title: String,
}

/// What is selected on the page: the selection, or why it was refused.
#[derive(Serialize, Type)]
pub struct SelectionAnswerDto {
    pub selection: Option<SelectionDto>,
    pub refused: Option<String>,
}

/// Reads what is selected in a tab of the window's browser (the app's page asks, only of its
/// current tab; the answer is checked).
#[tauri::command]
#[specta::specta]
pub async fn search_web_selection(
    app: AppHandle,
    session_id: CommandId,
    tab: CommandId,
) -> Result<SelectionAnswerDto, CommandError> {
    let web = page(&app, session_id.0, tab.0)?;
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    web.eval_with_callback(SELECTION_SCRIPT, move |raw| {
        let _ = tx.send(raw);
    })
    .map_err(tauri_error)?;
    let raw = tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default()
    })
    .await
    .map_err(joined)?;
    Ok(match parse_selection_report(&raw) {
        Ok(selection) => SelectionAnswerDto {
            selection: selection.map(|s| SelectionDto {
                text: s.text,
                url: s.url,
                title: s.title,
            }),
            refused: None,
        },
        Err(why) => SelectionAnswerDto {
            selection: None,
            refused: Some(why.sentence().to_owned()),
        },
    })
}

#[derive(Serialize, Type)]
pub struct VisualMatchDto {
    pub url: String,
    pub title: String,
    pub source: String,
    pub thumbnail: Option<String>,
}

impl From<VisualMatch> for VisualMatchDto {
    fn from(m: VisualMatch) -> Self {
        Self {
            url: m.url,
            title: m.title,
            source: m.source,
            thumbnail: m.thumbnail,
        }
    }
}

/// The window's own results list: the matches research already found (no cost).
#[tauri::command]
#[specta::specta]
pub async fn search_matches(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<Vec<VisualMatchDto>, CommandError> {
    let research = engine(&state)?;
    let sid = session_id.0;
    tauri::async_runtime::spawn_blocking(move || research.browse_matches(sid))
        .await
        .map_err(joined)?
        .map(|m| m.into_iter().map(Into::into).collect())
        .map_err(CommandError::from)
}

/// The window's own results list from one new visual search of the photo (a counted call).
#[tauri::command]
#[specta::specta]
pub async fn search_visual(
    state: State<'_, AppState>,
    session_id: CommandId,
    photo_id: CommandId,
) -> Result<Vec<VisualMatchDto>, CommandError> {
    let research = engine(&state)?;
    let (sid, pid) = (session_id.0, photo_id.0);
    tauri::async_runtime::spawn_blocking(move || research.browse_search(sid, pid))
        .await
        .map_err(joined)?
        .map(|m| m.into_iter().map(Into::into).collect())
        .map_err(CommandError::from)
}

#[derive(Serialize, Type)]
pub struct SavedPageDto {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub url: String,
    pub title: String,
    #[specta(type = specta_typescript::Number)]
    pub saved_at: i64,
}

impl From<psm_infra::sqlite::saved_pages::SavedPage> for SavedPageDto {
    fn from(p: psm_infra::sqlite::saved_pages::SavedPage) -> Self {
        Self {
            id: p.id,
            url: p.url,
            title: p.title,
            saved_at: p.saved_at,
        }
    }
}

/// The item's saved pages, oldest first.
#[tauri::command]
#[specta::specta]
pub async fn search_pages(
    state: State<'_, AppState>,
    item_id: CommandId,
) -> Result<Vec<SavedPageDto>, CommandError> {
    let handles = state.handles().map_err(CommandError::from)?;
    let id = item_id.0;
    tauri::async_runtime::spawn_blocking(move || handles.store.saved_pages(id))
        .await
        .map_err(joined)?
        .map(|pages| pages.into_iter().map(Into::into).collect())
        .map_err(CommandError::from)
}

/// Saves the page for the item (kept until removed).
#[tauri::command]
#[specta::specta]
pub async fn search_page_save(
    state: State<'_, AppState>,
    item_id: CommandId,
    url: String,
    title: String,
) -> Result<SavedPageDto, CommandError> {
    web_url(&url)?;
    let handles = state.handles().map_err(CommandError::from)?;
    let id = item_id.0;
    let now = handles.clock.now_ms();
    let saved = tauri::async_runtime::spawn_blocking(move || {
        handles.store.saved_page_add(id, &url, &title, now)
    })
    .await
    .map_err(joined)?
    .map_err(CommandError::from)?;
    tracing::info!(
        target: "psm::search",
        "search_window.page_saved item={id} host={}",
        psm_research::rules::sites::host(&saved.url).unwrap_or_default()
    );
    Ok(saved.into())
}

#[tauri::command]
#[specta::specta]
pub async fn search_page_remove(
    state: State<'_, AppState>,
    page_id: CommandId,
) -> Result<(), CommandError> {
    let handles = state.handles().map_err(CommandError::from)?;
    let id = page_id.0;
    tauri::async_runtime::spawn_blocking(move || handles.store.saved_page_remove(id))
        .await
        .map_err(joined)?
        .map_err(CommandError::from)
}

/// The value a field takes from selected text: text as it is for text fields; for other kinds,
/// the text mapped as research maps a stated value (a choice by name, a number, a measurement).
pub fn value_from_text(
    definition: &psm_core::domain::attribute::Definition,
    text: &str,
) -> Result<serde_json::Value, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("Select some text first".into());
    }
    match definition.core_type {
        CoreType::Text | CoreType::Textarea => Ok(serde_json::Value::String(text.to_owned())),
        _ => match map_output(definition, &serde_json::Value::String(text.to_owned())) {
            Mapped::Value { value, .. } => Ok(value),
            Mapped::Fail(_) => Err(format!(
                "“{}” is not a value {} takes",
                text.chars().take(60).collect::<String>(),
                definition.label
            )),
        },
    }
}

/// What "Put in <field>" did: the stored value, and what the field held before (its stored JSON
/// text; `None`: it was empty), so the window's toast can put it back (A14).
#[derive(Serialize, Type)]
pub struct PutTextDto {
    pub stored: AttributeValueDto,
    pub previous: Option<String>,
}

fn session_item(app: &psm_integrations::AppHandles, sid: i64) -> Result<i64, CommandError> {
    Store::session(app.store.as_ref(), sid)
        .map_err(CommandError::from)?
        .session
        .item_id
        .ok_or_else(|| refused("no_item", "This session has no item"))
}

/// What "Put in" writes: the selected text (mapped into the field's kind), or a value an answer in
/// words found (A18; its stored JSON text).
enum PutSource {
    Text(String),
    Value(String),
}

/// "Put in <field>": the selected text into a field, as a person's edit.
#[tauri::command]
#[specta::specta]
pub async fn search_put_text(
    state: State<'_, AppState>,
    session_id: CommandId,
    key: String,
    text: String,
) -> Result<PutTextDto, CommandError> {
    if text.chars().count() > MAX_SELECTION_CHARS {
        return Err(refused("selection_too_long", "The selection is too long"));
    }
    put(&state, session_id.0, key, PutSource::Text(text)).await
}

/// A18: "Put '<value>' in <field>" from an answer in words: the value AI found (its stored JSON
/// text, `AskFoundDto.value_json`) into the field, exactly as "Put in" writes selected text: a
/// person's edit, with what the field held for the toast's Undo.
#[tauri::command]
#[specta::specta]
pub async fn search_put_value(
    state: State<'_, AppState>,
    session_id: CommandId,
    key: String,
    value_json: String,
) -> Result<PutTextDto, CommandError> {
    if value_json.chars().count() > MAX_SELECTION_CHARS {
        return Err(refused("selection_too_long", "The value is too long"));
    }
    put(&state, session_id.0, key, PutSource::Value(value_json)).await
}

async fn put(
    state: &State<'_, AppState>,
    sid: i64,
    key: String,
    source: PutSource,
) -> Result<PutTextDto, CommandError> {
    let handles = state.handles().map_err(CommandError::from)?;
    let app = handles.clone();
    let put = tauri::async_runtime::spawn_blocking(move || -> Result<_, CommandError> {
        let item_id = session_item(&app, sid)?;
        with_service(&app, |service| {
            let definitions = service.definitions(integration(&app).as_deref())?;
            let definition = definitions
                .iter()
                .find(|d| d.key == key)
                .ok_or_else(|| Error::NotFound(format!("field {key}")))?;
            let (value, chars) = match &source {
                PutSource::Text(text) => (
                    value_from_text(definition, text).map_err(Error::InvalidValue)?,
                    text.chars().count(),
                ),
                PutSource::Value(json) => (
                    serde_json::from_str::<serde_json::Value>(json)
                        .map_err(|_| Error::InvalidValue("not a value".into()))?,
                    json.chars().count(),
                ),
            };
            // A14: what the field held as the form shows it (the title is the item's own), for the
            // toast's Undo.
            let previous = service
                .form_get(item_id)?
                .groups
                .into_iter()
                .flat_map(|g| g.fields)
                .find(|f| f.definition.key == key)
                .and_then(|f| f.value)
                .map(|v| v.value);
            let stored = service.set_value(item_id, &key, value, Provenance::Human)?;
            tracing::info!(
                target: "psm::search",
                "search_window.text_to_field session={sid} key={key} chars={chars} from={}",
                match source {
                    PutSource::Text(_) => "selection",
                    PutSource::Value(_) => "ai_answer",
                }
            );
            Ok(PutTextDto {
                stored: stored.into(),
                previous,
            })
        })
    })
    .await
    .map_err(joined)??;
    handles
        .events
        .emit(AppEvent::ResearchChanged { session_id: sid });
    Ok(put)
}

/// A14: Undo of "Put in <field>": the field gets back what it held (`previous`, the stored JSON
/// text `search_put_text` answered), or is emptied when it was empty. A person's edit.
#[tauri::command]
#[specta::specta]
pub async fn search_put_back(
    state: State<'_, AppState>,
    session_id: CommandId,
    key: String,
    previous: Option<String>,
) -> Result<(), CommandError> {
    let handles = state.handles().map_err(CommandError::from)?;
    let sid = session_id.0;
    let app = handles.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), CommandError> {
        let item_id = session_item(&app, sid)?;
        let value = previous
            .as_deref()
            .map(serde_json::from_str::<serde_json::Value>)
            .transpose()
            .map_err(|_| refused("not_restorable", "The previous value cannot be put back"))?;
        with_service(&app, |service| {
            match value {
                Some(value) => {
                    service.set_value(item_id, &key, value, Provenance::Human)?;
                }
                None => service.clear(item_id, &key)?,
            }
            tracing::info!(
                target: "psm::search",
                "search_window.text_to_field_undone session={sid} key={key}"
            );
            Ok(())
        })
    })
    .await
    .map_err(joined)??;
    handles
        .events
        .emit(AppEvent::ResearchChanged { session_id: sid });
    Ok(())
}

/// A14: "Open AI review" from the search window: the main window comes forward and opens the
/// session's details on the AI review tab (event `search_open_review`).
#[tauri::command]
#[specta::specta]
pub async fn search_open_review(app: AppHandle, session_id: CommandId) -> Result<(), CommandError> {
    use tauri::Emitter;
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.unminimize();
        let _ = main.set_focus();
    }
    app.emit(
        "search_open_review",
        serde_json::json!({ "session_id": session_id.0 }),
    )
    .map_err(tauri_error)
}

#[derive(Serialize, Type)]
pub struct AskSuggestionDto {
    pub key: String,
    pub label: String,
    pub value: String,
}

/// A18: one value an answer in words found (offered as "Put in"; nothing written).
#[derive(Serialize, Type)]
pub struct AskFoundDto {
    pub key: String,
    pub label: String,
    pub value: String,
    pub value_json: String,
    pub quote: String,
}

#[derive(Serialize, Type)]
pub struct AskAnswerDto {
    pub suggested: Vec<AskSuggestionDto>,
    /// `[field label, why]` of the fields the answer named but that were not written.
    pub not_written: Vec<(String, String)>,
    /// A18 answer mode: the answer in words.
    pub answer: Option<String>,
    /// A18 answer mode: the values found per field.
    pub found: Vec<AskFoundDto>,
    #[specta(type = specta_typescript::Number)]
    pub cost_micro_usd: i64,
}

/// "Ask AI…": the selected text and the person's instruction to the model. `mode` `suggest`: the
/// answer becomes AI suggestions in the form's AI review tab; `answer` (A18): a short answer in
/// words and the values found per field, nothing written.
#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_arguments)]
pub async fn search_ask(
    state: State<'_, AppState>,
    session_id: CommandId,
    text: String,
    url: String,
    title: String,
    instruction: String,
    mode: String,
) -> Result<AskAnswerDto, CommandError> {
    let research = engine(&state)?;
    let mode = match mode.as_str() {
        "answer" => AskMode::Answer,
        "suggest" => AskMode::Suggest,
        _ => return Err(refused("unknown_mode", "Unknown Ask AI mode")),
    };
    let request = AskRequest {
        session_id: session_id.0,
        page_url: url,
        page_title: title,
        text,
        instruction,
        mode,
    };
    let answer = tauri::async_runtime::spawn_blocking(move || research.browse_ask(&request))
        .await
        .map_err(joined)?
        .map_err(CommandError::from)?;
    Ok(AskAnswerDto {
        suggested: answer
            .suggested
            .into_iter()
            .map(|s| AskSuggestionDto {
                key: s.key,
                label: s.label,
                value: s.value,
            })
            .collect(),
        not_written: answer.not_written,
        answer: answer.answer,
        found: answer
            .found
            .into_iter()
            .map(|f| AskFoundDto {
                key: f.key,
                label: f.label,
                value: f.value,
                value_json: f.value_json,
                quote: f.quote,
            })
            .collect(),
        cost_micro_usd: answer.cost_micro_usd,
    })
}
