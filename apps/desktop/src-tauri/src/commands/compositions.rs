use crate::{
    commands::items::{CommandError, CommandId},
    state::AppState,
};
use psm_core::{
    Error,
    domain::composition::{
        Composition, CompositionKind, CompositionValues, Flip, Orientation, Rect, Shape,
    },
    services::composition::ApplyResult,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RectDto {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FrameDto {
    #[specta(type = specta_typescript::Number)]
    pub w: i64,
    #[specta(type = specta_typescript::Number)]
    pub h: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CompositionValuesDto {
    pub kind: String,
    pub shape: String,
    pub orientation: String,
    pub quarter_turns: u8,
    pub flip: String,
    pub straighten: f64,
    pub rect: RectDto,
}
#[derive(Serialize, Deserialize, Type)]
pub struct CompositionApplyItemDto {
    pub photo_id: CommandId,
    pub values: CompositionValuesDto,
}
#[derive(Serialize, Type)]
pub struct ApplyRefusalDto {
    #[specta(type = specta_typescript::Number)]
    pub photo_id: i64,
    pub reason: String,
}
#[derive(Serialize, Type)]
pub struct ApplyResultDto {
    #[specta(type = Vec<specta_typescript::Number>)]
    pub applied: Vec<i64>,
    pub refused: Vec<ApplyRefusalDto>,
}
impl From<ApplyResult> for ApplyResultDto {
    fn from(result: ApplyResult) -> Self {
        Self {
            applied: result.applied,
            refused: result
                .refused
                .into_iter()
                .map(|r| ApplyRefusalDto {
                    photo_id: r.photo_id,
                    reason: r.reason,
                })
                .collect(),
        }
    }
}
impl TryFrom<CompositionValuesDto> for CompositionValues {
    type Error = Error;
    fn try_from(dto: CompositionValuesDto) -> Result<Self, Self::Error> {
        let parse = |value: &str| {
            serde_json::from_value::<serde_json::Value>(serde_json::Value::String(value.into()))
                .unwrap()
        };
        let kind: CompositionKind =
            serde_json::from_value(parse(&dto.kind)).map_err(|_| Error::CompositionInvalid {
                reason: "kind".into(),
            })?;
        let shape = Shape::parse(&dto.shape).ok_or_else(|| Error::CompositionInvalid {
            reason: "shape".into(),
        })?;
        let orientation: Orientation =
            serde_json::from_value(parse(&dto.orientation)).map_err(|_| {
                Error::CompositionInvalid {
                    reason: "orientation".into(),
                }
            })?;
        let flip: Flip =
            serde_json::from_value(parse(&dto.flip)).map_err(|_| Error::CompositionInvalid {
                reason: "flip".into(),
            })?;
        Ok(Self {
            kind,
            shape,
            orientation,
            quarter_turns: dto.quarter_turns,
            flip,
            straighten: dto.straighten,
            rect: Rect {
                x: dto.rect.x,
                y: dto.rect.y,
                w: dto.rect.w,
                h: dto.rect.h,
            },
        })
    }
}
#[derive(Serialize, Type)]
pub struct CompositionDto {
    #[specta(type = specta_typescript::Number)]
    pub photo_id: i64,
    pub kind: String,
    pub shape: String,
    pub orientation: String,
    pub quarter_turns: u8,
    pub flip: String,
    pub straighten: f64,
    pub rect: Option<RectDto>,
    pub frame: Option<FrameDto>,
    pub origin: String,
    #[specta(type = specta_typescript::Number)]
    pub revision: i64,
    pub state: String,
    pub review_reason: Option<String>,
    #[specta(type = specta_typescript::Number)]
    pub updated_at: i64,
}
impl From<Composition> for CompositionDto {
    fn from(c: Composition) -> Self {
        Self {
            photo_id: c.photo_id,
            kind: c.kind.as_str().into(),
            shape: c.shape.as_str().into(),
            orientation: c.orientation.as_str().into(),
            quarter_turns: c.quarter_turns,
            flip: c.flip.as_str().into(),
            straighten: c.straighten,
            rect: c.rect.map(|r| RectDto {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
            }),
            frame: c.frame().map(|f| FrameDto { w: f.w, h: f.h }),
            origin: c.origin.as_str().into(),
            revision: c.revision,
            state: c.state.as_str().into(),
            review_reason: c.review_reason.map(|r| r.as_str().into()),
            updated_at: c.updated_at,
        }
    }
}
#[tauri::command]
#[specta::specta]
pub async fn composition_set(
    state: State<'_, AppState>,
    photo_id: CommandId,
    values: CompositionValuesDto,
) -> Result<CompositionDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        app.compositions
            .set(photo_id.0, values.try_into()?)
            .map(Into::into)
            .map_err(CommandError::from)
    })
    .await
    .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?
}
#[tauri::command]
#[specta::specta]
pub async fn composition_reset(
    state: State<'_, AppState>,
    photo_id: CommandId,
) -> Result<CompositionDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        app.compositions
            .reset(photo_id.0)
            .map(Into::into)
            .map_err(CommandError::from)
    })
    .await
    .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?
}
#[tauri::command]
#[specta::specta]
pub async fn composition_apply(
    state: State<'_, AppState>,
    items: Vec<CompositionApplyItemDto>,
) -> Result<ApplyResultDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut result = ApplyResult::default();
        for item in items {
            match item.values.try_into() {
                Ok(values) => {
                    let one = app.compositions.apply(vec![(item.photo_id.0, values)]);
                    result.applied.extend(one.applied);
                    result.refused.extend(one.refused);
                }
                Err(error) => result
                    .refused
                    .push(psm_core::services::composition::ApplyRefusal {
                        photo_id: item.photo_id.0,
                        reason: error.to_string(),
                    }),
            }
        }
        Ok(ApplyResultDto::from(result))
    })
    .await
    .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?
}
