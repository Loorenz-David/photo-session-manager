use crate::{
    commands::{
        items::{CommandError, CommandId},
        photos::ReviewResponse,
    },
    state::AppState,
};
use psm_core::{
    Result,
    domain::{
        composition::Shape,
        template::{Presets, Template, TemplateSummary, TemplateView},
    },
    services::templates::TemplatesService,
};
use psm_infra::imaging::ImageOpsImpl;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

#[derive(Serialize, Type)]
pub struct PresetsDto {
    pub split: bool,
    pub shape: Option<String>,
    pub zoom: u8,
}
#[derive(Deserialize, Type)]
pub struct PresetsInput {
    pub split: bool,
    pub shape: Option<String>,
    pub zoom: u8,
}
#[derive(Serialize, Type)]
pub struct ContainerDto {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = specta_typescript::Number)]
    pub final_no: i64,
    #[specta(type = specta_typescript::Number)]
    pub shoot_no: i64,
    pub example_file: Option<String>,
    pub example_path: Option<String>,
    pub presets: PresetsDto,
    pub name: Option<String>,
    pub description: Option<String>,
}
#[derive(Serialize, Type)]
pub struct TemplateDto {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub item_type: String,
    pub containers: Vec<ContainerDto>,
}
#[derive(Serialize, Type)]
pub struct TemplateSummaryDto {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub item_type: String,
    #[specta(type = specta_typescript::Number)]
    pub container_count: i64,
}
#[derive(Serialize, Type)]
pub struct TemplatePhotoViewDto {
    #[specta(type = specta_typescript::Number)]
    pub photo_id: i64,
    #[specta(type = Option<specta_typescript::Number>)]
    pub slot: Option<i64>,
    pub by_hand: bool,
    pub differs: bool,
}
#[derive(Serialize, Type)]
pub struct TemplateViewDto {
    pub templated: bool,
    #[specta(type = Option<specta_typescript::Number>)]
    pub template_id: Option<i64>,
    pub containers: Vec<ContainerDto>,
    pub photos: Vec<TemplatePhotoViewDto>,
    #[specta(type = specta_typescript::Number)]
    pub differs: i64,
    pub can_shift_back: bool,
    #[specta(type = Option<specta_typescript::Number>)]
    pub next_shot: Option<i64>,
}
impl From<Template> for TemplateDto {
    fn from(t: Template) -> Self {
        Self {
            id: t.id,
            item_type: t.item_type,
            containers: t
                .containers
                .into_iter()
                .map(|c| ContainerDto {
                    id: c.id,
                    final_no: c.final_no,
                    shoot_no: c.shoot_no,
                    example_file: c.example_file,
                    example_path: c.example_path,
                    presets: PresetsDto {
                        split: c.presets.split,
                        shape: c.presets.shape.map(|s| s.as_str().to_owned()),
                        zoom: c.presets.zoom,
                    },
                    name: c.name,
                    description: c.description,
                })
                .collect(),
        }
    }
}
impl From<TemplateSummary> for TemplateSummaryDto {
    fn from(s: TemplateSummary) -> Self {
        Self {
            id: s.id,
            item_type: s.item_type,
            container_count: s.container_count,
        }
    }
}
impl From<TemplateView> for TemplateViewDto {
    fn from(v: TemplateView) -> Self {
        Self {
            templated: v.templated,
            template_id: v.template_id,
            containers: TemplateDto::from(Template {
                id: 0,
                item_type: String::new(),
                containers: v.containers,
            })
            .containers,
            photos: v
                .photos
                .into_iter()
                .map(|p| TemplatePhotoViewDto {
                    photo_id: p.photo_id,
                    slot: p.slot,
                    by_hand: p.by_hand,
                    differs: p.differs,
                })
                .collect(),
            differs: v.differs,
            can_shift_back: v.can_shift_back,
            next_shot: v.next_shot,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn template_session_view(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> std::result::Result<TemplateViewDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut view = psm_core::services::template_placement::session_view_with_current(
            app.store.as_ref(),
            app.camera_runtime.current_shots().as_ref(),
            app.clock.now_ms(),
            session_id.0,
        )?;
        for c in &mut view.containers {
            c.example_path = c.example_file.as_ref().map(|file| {
                app.paths
                    .root
                    .join("templates")
                    .join(view.template_id.expect("templated view id").to_string())
                    .join(file)
                    .to_string_lossy()
                    .into_owned()
            });
        }
        Ok::<_, psm_core::Error>(TemplateViewDto::from(view))
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map_err(CommandError::from)
}
/// Runs a person's template action (drop, Back to template) on the wired app.
async fn hand_action(
    state: State<'_, AppState>,
    op: impl FnOnce(
        &psm_core::services::template_placement::HandAction<'_>,
    ) -> Result<psm_core::services::review::ReviewResult>
    + Send
    + 'static,
) -> std::result::Result<ReviewResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        op(&psm_core::services::template_placement::HandAction {
            store: app.store.as_ref(),
            clock: app.clock.as_ref(),
            events: app.events.as_ref(),
            history: app.history.as_ref(),
            shots: app.camera_runtime.current_shots(),
        })
        .map(ReviewResponse::from)
        .map_err(CommandError::from)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

/// T3: drop a photo onto container `container_no` (its final number); the touched session.
#[tauri::command]
#[specta::specta]
pub async fn template_drop(
    state: State<'_, AppState>,
    photo_id: CommandId,
    container_no: CommandId,
) -> std::result::Result<ReviewResponse, CommandError> {
    hand_action(state, move |h| {
        psm_core::services::template_placement::drop_on_container(h, photo_id.0, container_no.0)
    })
    .await
}

/// T3: Back to template for the session's differing photos; the touched session.
#[tauri::command]
#[specta::specta]
pub async fn template_back(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> std::result::Result<ReviewResponse, CommandError> {
    hand_action(state, move |h| {
        psm_core::services::template_placement::back_to_template(h, session_id.0)
    })
    .await
}

/// T4: Shift the rest back for the session; the touched session.
#[tauri::command]
#[specta::specta]
pub async fn template_shift_back(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> std::result::Result<ReviewResponse, CommandError> {
    hand_action(state, move |h| {
        psm_core::services::template_placement::shift_back(h, session_id.0)
    })
    .await
}

async fn run<T: Send + 'static>(
    state: State<'_, AppState>,
    op: impl FnOnce(&TemplatesService<'_>) -> Result<T> + Send + 'static,
) -> std::result::Result<T, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        let imaging = ImageOpsImpl;
        let service = TemplatesService::new(
            app.store.as_ref(),
            app.clock.as_ref(),
            &imaging,
            app.paths.root.join("templates"),
        );
        op(&service).map_err(CommandError::from)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn templates_list(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<TemplateSummaryDto>, CommandError> {
    run(state, |s| {
        s.list().map(|v| v.into_iter().map(Into::into).collect())
    })
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn template_get(
    state: State<'_, AppState>,
    item_type: String,
) -> std::result::Result<Option<TemplateDto>, CommandError> {
    run(state, move |s| s.get(&item_type).map(|v| v.map(Into::into))).await
}
#[tauri::command]
#[specta::specta]
pub async fn template_create(
    state: State<'_, AppState>,
    item_type: String,
) -> std::result::Result<TemplateDto, CommandError> {
    run(state, move |s| s.create(&item_type).map(Into::into)).await
}
#[tauri::command]
#[specta::specta]
pub async fn template_delete(
    state: State<'_, AppState>,
    template_id: CommandId,
) -> std::result::Result<(), CommandError> {
    run(state, move |s| s.delete(template_id.0)).await
}
#[tauri::command]
#[specta::specta]
pub async fn template_container_add(
    state: State<'_, AppState>,
    template_id: CommandId,
    source: Option<String>,
) -> std::result::Result<TemplateDto, CommandError> {
    run(state, move |s| {
        s.container_add(template_id.0, source.as_deref().map(std::path::Path::new))
            .map(Into::into)
    })
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn template_container_remove(
    state: State<'_, AppState>,
    container_id: CommandId,
) -> std::result::Result<TemplateDto, CommandError> {
    run(state, move |s| {
        s.container_remove(container_id.0).map(Into::into)
    })
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn template_container_example_set(
    state: State<'_, AppState>,
    container_id: CommandId,
    source: String,
) -> std::result::Result<TemplateDto, CommandError> {
    run(state, move |s| {
        s.container_example_set(container_id.0, std::path::Path::new(&source))
            .map(Into::into)
    })
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn template_containers_reorder(
    state: State<'_, AppState>,
    template_id: CommandId,
    container_ids: Vec<CommandId>,
) -> std::result::Result<TemplateDto, CommandError> {
    run(state, move |s| {
        s.containers_reorder(
            template_id.0,
            &container_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
        )
        .map(Into::into)
    })
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn template_shooting_order_set(
    state: State<'_, AppState>,
    template_id: CommandId,
    container_ids: Vec<CommandId>,
) -> std::result::Result<TemplateDto, CommandError> {
    run(state, move |s| {
        s.shooting_order_set(
            template_id.0,
            &container_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
        )
        .map(Into::into)
    })
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn template_container_label_set(
    state: State<'_, AppState>,
    container_id: CommandId,
    name: String,
    description: String,
) -> std::result::Result<TemplateDto, CommandError> {
    run(state, move |s| {
        s.container_label_set(container_id.0, &name, &description)
            .map(Into::into)
    })
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn template_container_presets_set(
    state: State<'_, AppState>,
    container_id: CommandId,
    presets: PresetsInput,
) -> std::result::Result<TemplateDto, CommandError> {
    run(state, move |s| {
        let shape = presets
            .shape
            .as_deref()
            .map(Shape::parse)
            .transpose_shape()?;
        s.container_presets_set(
            container_id.0,
            Presets {
                split: presets.split,
                shape,
                zoom: presets.zoom,
            },
        )
        .map(Into::into)
    })
    .await
}
trait ParseShape {
    fn transpose_shape(self) -> Result<Option<Shape>>;
}
impl ParseShape for Option<Option<Shape>> {
    fn transpose_shape(self) -> Result<Option<Shape>> {
        match self {
            None => Ok(None),
            Some(Some(v)) => Ok(Some(v)),
            Some(None) => Err(psm_core::Error::InvalidValue("unknown shape".into())),
        }
    }
}
