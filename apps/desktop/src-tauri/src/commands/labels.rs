//! L1 (book 09C §4, §7; `master_plan_labels.md` §4 Commands): the label markers of sessions.
//! Thin: one `LabelsService::session_labels` call and `access()` on a blocking worker (a status
//! read may wait on Pricing). The failure words come from the service (the provider's
//! `describe`); `labels_off` carries the connection state in `data.reason` with an empty message
//! (the screen has 09B's words).

use crate::{
    commands::items::{CommandError, CommandId},
    state::AppState,
};
use psm_core::{
    domain::label::{
        LabelState, LabelStatus, LabelsAccess, LabelsError, ResetReason, SessionLabel,
    },
    services::labels::{LabelsFailure, LabelsService},
};
use psm_integrations::AppHandles;
use serde::Serialize;
use specta::Type;
use std::collections::BTreeMap;
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
