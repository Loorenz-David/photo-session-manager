use crate::{
    commands::items::{CommandError, CommandId},
    state::AppState,
};
use psm_core::{
    Result,
    domain::{
        composition::Shape,
        template::{Presets, Template, TemplateSummary},
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
