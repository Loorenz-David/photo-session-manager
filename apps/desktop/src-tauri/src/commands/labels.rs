//! L1 (book 09C §4, §7; `master_plan_labels.md` §4 Commands): the label markers of sessions.
//! Thin: one `LabelsService::session_labels` call and `access()` on a blocking worker (a status
//! read may wait on Pricing). The failure words come from the service (the provider's
//! `describe`); `labels_off` carries the connection state in `data.reason` with an empty message
//! (the screen has 09B's words).
//!
//! L2 (MC-L1): the Labels box's list, its number, Select all and opening a row; each one service
//! call on a blocking worker.
//!
//! L3 (MC-L2): the check, the preview picture, generating, the result after a failed chunk and
//! the download; each one service call on a blocking worker (`labels_generate` can take minutes
//! for a large batch with retries). No command takes product ids for a run: `labels_generate`
//! takes the id of the check the dialog just showed (M-L2).

use crate::{
    commands::{
        items::{CommandError, CommandId},
        sessions::SessionRowResponse,
    },
    state::AppState,
};
use psm_core::{
    domain::label::{
        CheckGroup, CheckOutcome, LabelArtifact, LabelCheck, LabelCounts, LabelFailure,
        LabelFilter, LabelPage, LabelRow, LabelRunResult, LabelSize, LabelState, LabelStatus,
        LabelsAccess, LabelsError, NoteKind, PlanItem, PlanSummary, PlanTemplate, ResetReason,
        SavedLabel, Selected, SessionLabel, Unresolved,
    },
    services::labels::{LabelsFailure, LabelsService},
};
use psm_integrations::AppHandles;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};
use tauri::State;

/// `printed` | `not_printed`.
#[derive(Debug, Clone, Copy, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LabelStatusDto {
    Printed,
    NotPrinted,
}

/// Why a printed label went back to not printed (the wire words `price_changed`, …).
#[derive(Debug, Clone, Copy, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum ResetReasonDto {
    PriceChanged,
    SkuChanged,
    BarcodeChanged,
}

/// A product's label state (`printed_at` UTC ms; `has_price` null = unknown).
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct LabelMarkerDto {
    pub status: LabelStatusDto,
    #[specta(type = Option<specta_typescript::Number>)]
    pub printed_at: Option<i64>,
    pub has_price: Option<bool>,
    pub reset: Option<ResetReasonDto>,
}

/// Whether label calls may be made: `off` (the connection is not ready), `unknown`, `allowed`,
/// `denied`.
#[derive(Debug, Clone, Copy, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LabelsAccessStateDto {
    Off,
    Unknown,
    Allowed,
    Denied,
}

/// The access with, when `off`, the connection state (`no_key`, `no_shop`, `store_unmatched`,
/// `refused`, `store_gone`).
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct LabelsAccessDto {
    pub state: LabelsAccessStateDto,
    pub reason: Option<String>,
}

/// One session with its product (`null` = not in the shop) and its marker (`null` = none known).
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct SessionLabelDto {
    #[specta(type = specta_typescript::Number)]
    pub session_id: i64,
    pub product_id: Option<String>,
    pub marker: Option<LabelMarkerDto>,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct LabelsStatusDto {
    pub access: LabelsAccessDto,
    pub labels: Vec<SessionLabelDto>,
}

pub fn access_state(access: LabelsAccess) -> LabelsAccessStateDto {
    match access {
        LabelsAccess::Off(_) => LabelsAccessStateDto::Off,
        LabelsAccess::Unknown => LabelsAccessStateDto::Unknown,
        LabelsAccess::Allowed => LabelsAccessStateDto::Allowed,
        LabelsAccess::Denied => LabelsAccessStateDto::Denied,
    }
}

fn access_dto(access: LabelsAccess) -> LabelsAccessDto {
    LabelsAccessDto {
        state: access_state(access),
        reason: match access {
            LabelsAccess::Off(state) => Some(state.as_str().to_owned()),
            _ => None,
        },
    }
}

fn marker(state: LabelState) -> LabelMarkerDto {
    LabelMarkerDto {
        status: match state.status {
            LabelStatus::Printed => LabelStatusDto::Printed,
            LabelStatus::NotPrinted => LabelStatusDto::NotPrinted,
        },
        printed_at: state.printed_at,
        has_price: state.has_price,
        reset: state.reset.map(|r| match r {
            ResetReason::PriceChanged => ResetReasonDto::PriceChanged,
            ResetReason::SkuChanged => ResetReasonDto::SkuChanged,
            ResetReason::BarcodeChanged => ResetReasonDto::BarcodeChanged,
        }),
    }
}

fn session_label(label: SessionLabel) -> SessionLabelDto {
    SessionLabelDto {
        session_id: label.session_id,
        product_id: label.product_id,
        marker: label.state.map(marker),
    }
}

/// `LabelsFailure` → `CommandError` (master plan §4 Commands, error codes).
pub fn labels_error(service: &LabelsService, failure: LabelsFailure) -> CommandError {
    let failure = match failure {
        LabelsFailure::Storage(error) => return error.into(),
        other => other,
    };
    let mut data = BTreeMap::new();
    let code = match &failure {
        LabelsFailure::Off(state) => {
            data.insert("reason".into(), state.as_str().to_owned());
            "labels_off"
        }
        LabelsFailure::Denied | LabelsFailure::Provider(LabelsError::NotAllowed) => "labels_denied",
        LabelsFailure::Provider(error) => match error {
            LabelsError::Unreachable(_) => "labels_unreachable",
            LabelsError::BadReply(_) => "labels_bad_reply",
            LabelsError::StoreGone => "labels_store_gone",
            LabelsError::NoProducts => "labels_no_products",
            LabelsError::RendererUnavailable => "labels_renderer_unavailable",
            LabelsError::ListUnavailable => "labels_list_unavailable",
            LabelsError::Expired => "labels_expired",
            LabelsError::Rejected { .. } => "labels_rejected",
            LabelsError::NotAllowed => "labels_denied",
        },
        LabelsFailure::Busy => "labels_busy",
        LabelsFailure::NothingToGenerate => "labels_nothing",
        LabelsFailure::NotFound => "not_found",
        LabelsFailure::PreviewFailed(_) => "labels_preview_failed",
        LabelsFailure::SaveFailed(_) => "labels_save_failed",
        LabelsFailure::Storage(_) => "storage",
    };
    CommandError {
        code: code.into(),
        message: service.describe(&failure),
        data,
    }
}

/// The body of `labels_status` (MC-L3.3).
pub fn labels_status_body(
    app: &AppHandles,
    session_ids: &[i64],
    refresh: bool,
) -> Result<LabelsStatusDto, CommandError> {
    let service = &app.labels;
    let labels = service
        .session_labels(session_ids, refresh)
        .map_err(|failure| labels_error(service, failure))?;
    let access = service.access().map_err(CommandError::from)?;
    Ok(LabelsStatusDto {
        access: access_dto(access),
        labels: labels.into_iter().map(session_label).collect(),
    })
}

/// The label markers of `session_ids` (in that order), with the labels access. `refresh` asks
/// Pricing for products without a fresh answer; without it only what this Mac already knows is
/// answered.
#[tauri::command]
#[specta::specta]
pub async fn labels_status(
    state: State<'_, AppState>,
    session_ids: Vec<CommandId>,
    refresh: bool,
) -> Result<LabelsStatusDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let session_ids: Vec<i64> = session_ids.into_iter().map(|id| id.0).collect();
    tauri::async_runtime::spawn_blocking(move || labels_status_body(&app, &session_ids, refresh))
        .await
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

// ---------- L2: the Labels box ----------

/// The Labels box's sub-filter.
#[derive(Debug, Clone, Copy, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LabelFilterDto {
    MissingPriceTag,
    NotPrinted,
    Printed,
    All,
}

impl From<LabelFilterDto> for LabelFilter {
    fn from(filter: LabelFilterDto) -> Self {
        match filter {
            LabelFilterDto::MissingPriceTag => Self::MissingPriceTag,
            LabelFilterDto::NotPrinted => Self::NotPrinted,
            LabelFilterDto::Printed => Self::Printed,
            LabelFilterDto::All => Self::All,
        }
    }
}

/// The provider's totals over the searched list (each on its own, the filter not applied).
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct LabelCountsDto {
    #[specta(type = specta_typescript::Number)]
    pub total: i64,
    #[specta(type = specta_typescript::Number)]
    pub has_price: i64,
    #[specta(type = specta_typescript::Number)]
    pub missing_price: i64,
    #[specta(type = specta_typescript::Number)]
    pub missing_price_tag: i64,
    #[specta(type = specta_typescript::Number)]
    pub printed: i64,
    #[specta(type = specta_typescript::Number)]
    pub not_printed: i64,
}

/// One row of the Labels box: the provider's fields, the row's own marker, the matched session
/// (`null` = none on this Mac) and the cached picture of an unmatched row (an existing file).
#[derive(Serialize, Type)]
pub struct LabelRowDto {
    pub product_id: String,
    pub commerce_id: Option<String>,
    pub sku: Option<String>,
    pub title: String,
    pub product_type: Option<String>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub price_minor: Option<i64>,
    pub marker: LabelMarkerDto,
    #[specta(type = Option<specta_typescript::Number>)]
    pub created_at: Option<i64>,
    pub session: Option<SessionRowResponse>,
    pub picture_path: Option<String>,
}

#[derive(Serialize, Type)]
pub struct LabelPageDto {
    pub rows: Vec<LabelRowDto>,
    pub next_cursor: Option<String>,
    pub counts: LabelCountsDto,
    pub work_from: Option<String>,
}

/// The Labels box's number (`null` = none known, or labels are off).
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct LabelsSummaryDto {
    #[specta(type = Option<specta_typescript::Number>)]
    pub missing_price_tag: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct SelectedDto {
    pub product_id: String,
    pub sku: String,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct LabelsOpenDto {
    #[specta(type = specta_typescript::Number)]
    pub session_id: i64,
}

fn counts_dto(c: LabelCounts) -> LabelCountsDto {
    LabelCountsDto {
        total: c.total,
        has_price: c.has_price,
        missing_price: c.missing_price,
        missing_price_tag: c.missing_price_tag,
        printed: c.printed,
        not_printed: c.not_printed,
    }
}

fn row_dto(row: LabelRow) -> LabelRowDto {
    LabelRowDto {
        product_id: row.product_id,
        commerce_id: row.commerce_id,
        sku: row.sku,
        title: row.title,
        product_type: row.product_type,
        price_minor: row.price_minor,
        marker: marker(row.state),
        created_at: row.created_at,
        session: row.session.map(SessionRowResponse::from),
        picture_path: row
            .picture_path
            .map(|path| path.to_string_lossy().into_owned()),
    }
}

fn page_dto(page: LabelPage) -> LabelPageDto {
    LabelPageDto {
        rows: page.rows.into_iter().map(row_dto).collect(),
        next_cursor: page.next_cursor,
        counts: counts_dto(page.counts),
        work_from: page.work_from,
    }
}

fn blocking_error(error: tauri::Error) -> CommandError {
    CommandError::from(psm_core::Error::InvalidData(error.to_string()))
}

/// The body of `labels_list` (MC-L1.1–.5).
pub fn labels_list_body(
    app: &AppHandles,
    filter: LabelFilterDto,
    query: Option<&str>,
    cursor: Option<&str>,
) -> Result<LabelPageDto, CommandError> {
    let service = &app.labels;
    service
        .list(filter.into(), query, cursor)
        .map(page_dto)
        .map_err(|failure| labels_error(service, failure))
}

/// The body of `labels_summary` (MC-L1.6).
pub fn labels_summary_body(
    app: &AppHandles,
    refresh: bool,
) -> Result<LabelsSummaryDto, CommandError> {
    let service = &app.labels;
    service
        .summary(refresh)
        .map(|missing_price_tag| LabelsSummaryDto { missing_price_tag })
        .map_err(|failure| labels_error(service, failure))
}

/// The body of `labels_select_all` (MC-L1.7).
pub fn labels_select_all_body(
    app: &AppHandles,
    filter: LabelFilterDto,
    query: Option<&str>,
) -> Result<Vec<SelectedDto>, CommandError> {
    let service = &app.labels;
    service
        .select_all(filter.into(), query)
        .map(|selected| {
            selected
                .into_iter()
                .map(|Selected { product_id, sku }| SelectedDto { product_id, sku })
                .collect()
        })
        .map_err(|failure| labels_error(service, failure))
}

/// The body of `labels_open` (MC-L1.8).
pub fn labels_open_body(app: &AppHandles, product_id: &str) -> Result<LabelsOpenDto, CommandError> {
    let service = &app.labels;
    service
        .open_product(product_id)
        .map(|session_id| LabelsOpenDto { session_id })
        .map_err(|failure| labels_error(service, failure))
}

/// One page of the Labels box (25 rows) for the filter, the search and the cursor of the page.
#[tauri::command]
#[specta::specta]
pub async fn labels_list(
    state: State<'_, AppState>,
    filter: LabelFilterDto,
    query: Option<String>,
    cursor: Option<String>,
) -> Result<LabelPageDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        labels_list_body(&app, filter, query.as_deref(), cursor.as_deref())
    })
    .await
    .map_err(blocking_error)?
}

/// The Labels box's number; `refresh` asks Pricing when the remembered number is not fresh.
#[tauri::command]
#[specta::specta]
pub async fn labels_summary(
    state: State<'_, AppState>,
    refresh: bool,
) -> Result<LabelsSummaryDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || labels_summary_body(&app, refresh))
        .await
        .map_err(blocking_error)?
}

/// Every product of the filter and the search that has a SKU (Select all N).
#[tauri::command]
#[specta::specta]
pub async fn labels_select_all(
    state: State<'_, AppState>,
    filter: LabelFilterDto,
    query: Option<String>,
) -> Result<Vec<SelectedDto>, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        labels_select_all_body(&app, filter, query.as_deref())
    })
    .await
    .map_err(blocking_error)?
}

/// The session a Labels row opens (an unmatched product is adopted from the shop first).
#[tauri::command]
#[specta::specta]
pub async fn labels_open(
    state: State<'_, AppState>,
    product_id: String,
) -> Result<LabelsOpenDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || labels_open_body(&app, &product_id))
        .await
        .map_err(blocking_error)?
}

// ---------- L3: check, preview, generate, download ----------

/// What generating will do to one product: `mark_printed`, `reprint`, `no_price`, `fail`.
#[derive(Debug, Clone, Copy, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckOutcomeDto {
    MarkPrinted,
    Reprint,
    NoPrice,
    Fail,
}

/// A group's template (millimetres).
#[derive(Debug, Clone, Serialize, Type, PartialEq)]
pub struct PlanTemplateDto {
    pub id: String,
    pub name: String,
    pub width_mm: f64,
    pub height_mm: f64,
}

/// One product of the check, with what will happen to it and Pricing's messages as given.
#[derive(Debug, Clone, Serialize, Type, PartialEq)]
pub struct CheckItemDto {
    pub product_id: String,
    pub sku: Option<String>,
    pub title: Option<String>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub price_minor: Option<i64>,
    pub status: LabelStatusDto,
    #[specta(type = Option<specta_typescript::Number>)]
    pub printed_at: Option<i64>,
    pub outcome: CheckOutcomeDto,
    pub issues: Vec<String>,
    /// L4 (MC-L2.15): Pricing's notes with their kind and the QR code's address.
    pub notes: Vec<PlanNoteDto>,
    /// L4 (MC-L2.16): a picture of the product already on this Mac (inside the data folder).
    pub picture_path: Option<String>,
}

/// What a note says (`not_published`: not on the web shop yet; its QR code opens `link`).
#[derive(Debug, Clone, Copy, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoteKindDto {
    NotPublished,
    Other,
}

/// One note of a check item: its kind, the message as given, the address (or null).
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct PlanNoteDto {
    pub kind: NoteKindDto,
    pub message: String,
    pub link: Option<String>,
}

/// One group of the check (`template` null = the products without a template, always last).
#[derive(Debug, Clone, Serialize, Type, PartialEq)]
pub struct CheckGroupDto {
    pub template: Option<PlanTemplateDto>,
    pub product_types: Vec<String>,
    pub items: Vec<CheckItemDto>,
}

/// A product Pricing did not find ("Not found in Pricing", left out).
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct UnresolvedDto {
    pub product_id: Option<String>,
    pub sku: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct CheckSummaryDto {
    #[specta(type = specta_typescript::Number)]
    pub items: i64,
    #[specta(type = specta_typescript::Number)]
    pub ready: i64,
    #[specta(type = specta_typescript::Number)]
    pub failing: i64,
    #[specta(type = specta_typescript::Number)]
    pub mark_printed: i64,
    #[specta(type = specta_typescript::Number)]
    pub reprint: i64,
    #[specta(type = specta_typescript::Number)]
    pub no_price: i64,
}

/// A label size among the ready products, with how many labels it has.
#[derive(Debug, Clone, Serialize, Type, PartialEq)]
pub struct LabelSizeDto {
    pub width_mm: f64,
    pub height_mm: f64,
    #[specta(type = specta_typescript::Number)]
    pub count: i64,
}

/// The check dialog's content; `id` names the check for preview, generate and download.
#[derive(Debug, Clone, Serialize, Type, PartialEq)]
pub struct LabelCheckDto {
    pub id: String,
    pub groups: Vec<CheckGroupDto>,
    pub unresolved: Vec<UnresolvedDto>,
    pub summary: CheckSummaryDto,
    pub sizes: Vec<LabelSizeDto>,
}

/// The preview picture of one label (a file under the data folder's previews).
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct LabelPreviewDto {
    pub path: String,
}

/// One product not generated, with its message.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct LabelFailureDto {
    pub product_id: Option<String>,
    pub sku: Option<String>,
    pub title: Option<String>,
    pub message: String,
}

/// One label file of the result (`page_count` = its labels).
#[derive(Debug, Clone, Serialize, Type, PartialEq)]
pub struct LabelFileDto {
    pub id: String,
    pub filename: String,
    pub width_mm: f64,
    pub height_mm: f64,
    #[specta(type = specta_typescript::Number)]
    pub page_count: i64,
}

/// The done dialog's content. Fully generated ⇔ `failures` and `not_run` are both empty.
#[derive(Debug, Clone, Serialize, Type, PartialEq)]
pub struct LabelRunResultDto {
    #[specta(type = specta_typescript::Number)]
    pub total: i64,
    #[specta(type = specta_typescript::Number)]
    pub rendered: i64,
    #[specta(type = specta_typescript::Number)]
    pub marked_printed: i64,
    #[specta(type = specta_typescript::Number)]
    pub reprinted: i64,
    #[specta(type = specta_typescript::Number)]
    pub no_price: i64,
    pub failures: Vec<LabelFailureDto>,
    pub not_run: Vec<String>,
    pub artifacts: Vec<LabelFileDto>,
    pub failed_product_ids: Vec<String>,
}

/// A saved label file (in the labels folder, outside the data folder: shown as text only) and
/// whether Preview opened it.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct SavedLabelDto {
    pub path: String,
    pub opened: bool,
}

fn template_dto(t: PlanTemplate) -> PlanTemplateDto {
    PlanTemplateDto {
        id: t.id,
        name: t.name,
        width_mm: t.width_mm,
        height_mm: t.height_mm,
    }
}

fn check_item_dto(item: PlanItem, pictures: &HashMap<String, PathBuf>) -> CheckItemDto {
    let picture_path = pictures
        .get(&item.product_id)
        .map(|path| path.to_string_lossy().into_owned());
    CheckItemDto {
        picture_path,
        product_id: item.product_id,
        sku: item.sku,
        title: item.title,
        price_minor: item.price_minor,
        status: match item.status {
            LabelStatus::Printed => LabelStatusDto::Printed,
            LabelStatus::NotPrinted => LabelStatusDto::NotPrinted,
        },
        printed_at: item.printed_at,
        outcome: match item.outcome {
            CheckOutcome::MarkPrinted => CheckOutcomeDto::MarkPrinted,
            CheckOutcome::Reprint => CheckOutcomeDto::Reprint,
            CheckOutcome::NoPrice => CheckOutcomeDto::NoPrice,
            CheckOutcome::Fail => CheckOutcomeDto::Fail,
        },
        issues: item.issues,
        notes: item
            .notes
            .into_iter()
            .map(|note| PlanNoteDto {
                kind: match note.kind {
                    NoteKind::NotPublished => NoteKindDto::NotPublished,
                    NoteKind::Other => NoteKindDto::Other,
                },
                message: note.message,
                link: note.link,
            })
            .collect(),
    }
}

fn group_dto(group: CheckGroup, pictures: &HashMap<String, PathBuf>) -> CheckGroupDto {
    CheckGroupDto {
        template: group.template.map(template_dto),
        product_types: group.product_types,
        items: group
            .items
            .into_iter()
            .map(|item| check_item_dto(item, pictures))
            .collect(),
    }
}

fn summary_dto(s: PlanSummary) -> CheckSummaryDto {
    CheckSummaryDto {
        items: s.items,
        ready: s.ready,
        failing: s.failing,
        mark_printed: s.mark_printed,
        reprint: s.reprint,
        no_price: s.no_price,
    }
}

fn check_dto(check: LabelCheck) -> LabelCheckDto {
    LabelCheckDto {
        id: check.id,
        groups: check
            .groups
            .into_iter()
            .map(|group| group_dto(group, &check.pictures))
            .collect(),
        unresolved: check
            .unresolved
            .into_iter()
            .map(|u: Unresolved| UnresolvedDto {
                product_id: u.product_id,
                sku: u.sku,
                message: u.message,
            })
            .collect(),
        summary: summary_dto(check.summary),
        sizes: check
            .sizes
            .into_iter()
            .map(|z: LabelSize| LabelSizeDto {
                width_mm: z.width_mm,
                height_mm: z.height_mm,
                count: z.count,
            })
            .collect(),
    }
}

fn result_dto(r: LabelRunResult) -> LabelRunResultDto {
    LabelRunResultDto {
        total: r.total,
        rendered: r.rendered,
        marked_printed: r.marked_printed,
        reprinted: r.reprinted,
        no_price: r.no_price,
        failures: r
            .failures
            .into_iter()
            .map(|f: LabelFailure| LabelFailureDto {
                product_id: f.product_id,
                sku: f.sku,
                title: f.title,
                message: f.message,
            })
            .collect(),
        not_run: r.not_run,
        artifacts: r
            .artifacts
            .into_iter()
            .map(|a: LabelArtifact| LabelFileDto {
                id: a.id,
                filename: a.filename,
                width_mm: a.width_mm,
                height_mm: a.height_mm,
                page_count: a.page_count,
            })
            .collect(),
        failed_product_ids: r.failed_product_ids,
    }
}

/// The body of `labels_check` (MC-L2.1–.4).
pub fn labels_check_body(
    app: &AppHandles,
    product_ids: &[String],
) -> Result<LabelCheckDto, CommandError> {
    let service = &app.labels;
    service
        .check(product_ids)
        .map(check_dto)
        .map_err(|failure| labels_error(service, failure))
}

/// The body of `labels_preview` (MC-L2.5).
pub fn labels_preview_body(
    app: &AppHandles,
    check_id: &str,
    product_id: &str,
) -> Result<LabelPreviewDto, CommandError> {
    let service = &app.labels;
    service
        .preview(check_id, product_id)
        .map(|path| LabelPreviewDto {
            path: path.to_string_lossy().into_owned(),
        })
        .map_err(|failure| labels_error(service, failure))
}

/// The body of `labels_generate` (MC-L2.6–.10).
pub fn labels_generate_body(
    app: &AppHandles,
    check_id: &str,
) -> Result<LabelRunResultDto, CommandError> {
    let service = &app.labels;
    service
        .generate(check_id)
        .map(result_dto)
        .map_err(|failure| labels_error(service, failure))
}

/// The body of `labels_partial` (MC-L2.10; `not_found` without a generation).
pub fn labels_partial_body(
    app: &AppHandles,
    check_id: &str,
) -> Result<LabelRunResultDto, CommandError> {
    let service = &app.labels;
    service
        .partial(check_id)
        .map(result_dto)
        .map_err(|failure| labels_error(service, failure))
}

/// The body of `labels_download` (MC-L2.11).
pub fn labels_download_body(
    app: &AppHandles,
    check_id: &str,
    artifact_id: &str,
) -> Result<SavedLabelDto, CommandError> {
    let service = &app.labels;
    service
        .download(check_id, artifact_id)
        .map(|SavedLabel { path, opened }| SavedLabelDto {
            path: path.to_string_lossy().into_owned(),
            opened,
        })
        .map_err(|failure| labels_error(service, failure))
}

/// What generating the labels of these products would do (changes nothing); a bigger selection
/// is checked in chunks and shown as one.
#[tauri::command]
#[specta::specta]
pub async fn labels_check(
    state: State<'_, AppState>,
    product_ids: Vec<String>,
) -> Result<LabelCheckDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || labels_check_body(&app, &product_ids))
        .await
        .map_err(blocking_error)?
}

/// The preview picture of one product of the check.
#[tauri::command]
#[specta::specta]
pub async fn labels_preview(
    state: State<'_, AppState>,
    check_id: String,
    product_id: String,
) -> Result<LabelPreviewDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || labels_preview_body(&app, &check_id, &product_id))
        .await
        .map_err(blocking_error)?
}

/// Generates the labels of the check's ready products (Generate PDF / Reprint PDF; Try again
/// continues where it stopped).
#[tauri::command]
#[specta::specta]
pub async fn labels_generate(
    state: State<'_, AppState>,
    check_id: String,
) -> Result<LabelRunResultDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || labels_generate_body(&app, &check_id))
        .await
        .map_err(blocking_error)?
}

/// The result over what was generated so far (leaving the dialog after a failed Generate).
#[tauri::command]
#[specta::specta]
pub async fn labels_partial(
    state: State<'_, AppState>,
    check_id: String,
) -> Result<LabelRunResultDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || labels_partial_body(&app, &check_id))
        .await
        .map_err(blocking_error)?
}

/// Saves one label file of the result in Documents › Session Manager Labels and opens it in
/// Preview.
#[tauri::command]
#[specta::specta]
pub async fn labels_download(
    state: State<'_, AppState>,
    check_id: String,
    artifact_id: String,
) -> Result<SavedLabelDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        labels_download_body(&app, &check_id, &artifact_id)
    })
    .await
    .map_err(blocking_error)?
}
