//! P10 attribute commands: thin wrappers over `AttributesService` (ch08). Values cross the boundary
//! as JSON text (the stored canonical JSON, MC-9); the UI parses it.

use crate::{
    commands::items::{CommandError, CommandId},
    state::AppState,
};
use psm_core::{
    Error,
    domain::attribute::{AttributeGroup, Definition, Form, Preset, PresetField, Provenance, Value},
    mechanisms::values::canonical_json,
    ports::clock::SystemClock,
    services::attributes::AttributesService,
};
use psm_integrations::App;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

#[derive(Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AttributeGroupDto {
    Details,
    Condition,
    Product,
}

impl From<AttributeGroupDto> for AttributeGroup {
    fn from(value: AttributeGroupDto) -> Self {
        match value {
            AttributeGroupDto::Details => Self::Details,
            AttributeGroupDto::Condition => Self::Condition,
            AttributeGroupDto::Product => Self::Product,
        }
    }
}

impl From<AttributeGroup> for AttributeGroupDto {
    fn from(value: AttributeGroup) -> Self {
        match value {
            AttributeGroup::Details => Self::Details,
            AttributeGroup::Condition => Self::Condition,
            AttributeGroup::Product => Self::Product,
        }
    }
}

/// One definition. `core_type` is one of the 18 core types (ch08); `validations` is JSON text.
#[derive(Serialize, Type)]
pub struct DefinitionDto {
    pub integration: String,
    pub key: String,
    pub label: String,
    pub core_type: String,
    pub choices: Vec<String>,
    pub validations: String,
    pub group: AttributeGroupDto,
    pub description: Option<String>,
    pub writable: bool,
}

impl From<Definition> for DefinitionDto {
    fn from(d: Definition) -> Self {
        Self {
            integration: d.integration,
            key: d.key,
            label: d.label,
            core_type: d.core_type.as_str().into(),
            choices: d.choices,
            validations: canonical_json(&d.validations),
            group: d.group.into(),
            description: d.description,
            writable: d.writable,
        }
    }
}

/// One stored value: `value` is canonical JSON text when `valid`, else the value as received.
/// `provenance` is `human` / `ai` / `default`; `ai_confidence` `high` / `medium` / `low`;
/// `ai_status` `suggested` / `accepted`.
#[derive(Serialize, Type)]
pub struct AttributeValueDto {
    #[specta(type = specta_typescript::Number)]
    pub item_id: i64,
    pub key: String,
    pub value: String,
    pub valid: bool,
    pub provenance: String,
    pub ai_confidence: Option<String>,
    pub ai_status: Option<String>,
    pub ai_reason: Option<String>,
    pub ai_sources: Option<Vec<String>>,
    #[specta(type = specta_typescript::Number)]
    pub updated_at: i64,
}

impl From<Value> for AttributeValueDto {
    fn from(v: Value) -> Self {
        Self {
            item_id: v.item_id,
            key: v.key,
            value: v.value,
            valid: v.valid,
            provenance: v.provenance.as_str().into(),
            ai_confidence: v.ai_confidence.map(|c| c.as_str().into()),
            ai_status: v.ai_status.map(|s| s.as_str().into()),
            ai_reason: v.ai_reason,
            ai_sources: v.ai_sources,
            updated_at: v.updated_at,
        }
    }
}

/// One field of a preset layout; `default` is JSON text.
#[derive(Serialize, Deserialize, Type)]
pub struct PresetFieldDto {
    pub key: String,
    pub default: Option<String>,
}

#[derive(Serialize, Deserialize, Type)]
pub struct PresetDto {
    pub item_type: String,
    pub group: AttributeGroupDto,
    pub layout: Vec<PresetFieldDto>,
}

impl From<Preset> for PresetDto {
    fn from(p: Preset) -> Self {
        Self {
            item_type: p.item_type,
            group: p.group.into(),
            layout: p
                .layout
                .into_iter()
                .map(|f| PresetFieldDto {
                    key: f.key,
                    default: f.default.as_ref().map(canonical_json),
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Type)]
pub struct FormFieldDto {
    pub definition: DefinitionDto,
    pub value: Option<AttributeValueDto>,
    pub is_default: bool,
    pub is_severity: bool,
    /// For a `media_list` field: the selected condition photo ids in condition order (MC-14).
    #[specta(type = Option<Vec<specta_typescript::Number>>)]
    pub selected: Option<Vec<i64>>,
}

#[derive(Serialize, Type)]
pub struct FormGroupDto {
    pub group: AttributeGroupDto,
    pub fields: Vec<FormFieldDto>,
    pub filled: u32,
    pub total: u32,
}

#[derive(Serialize, Type)]
pub struct FormDto {
    #[specta(type = specta_typescript::Number)]
    pub item_id: i64,
    pub groups: Vec<FormGroupDto>,
}

impl From<Form> for FormDto {
    fn from(form: Form) -> Self {
        Self {
            item_id: form.item_id,
            groups: form
                .groups
                .into_iter()
                .map(|g| FormGroupDto {
                    group: g.group.into(),
                    filled: u32::try_from(g.filled).unwrap_or(u32::MAX),
                    total: u32::try_from(g.total).unwrap_or(u32::MAX),
                    fields: g
                        .fields
                        .into_iter()
                        .map(|f| FormFieldDto {
                            definition: f.definition.into(),
                            value: f.value.map(Into::into),
                            is_default: f.is_default,
                            is_severity: f.is_severity,
                            selected: f.selected,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

fn parse(text: &str) -> Result<serde_json::Value, CommandError> {
    serde_json::from_str(text)
        .map_err(|e| CommandError::from(Error::InvalidValue(format!("not JSON: {e}"))))
}

/// The configured integration's kind, whose definitions are the active set (none without one).
fn integration(app: &App) -> Option<String> {
    app.settings
        .integration
        .as_ref()
        .and_then(|value| value.get("kind"))
        .and_then(|kind| kind.as_str())
        .map(str::to_owned)
}

fn with_service<T>(
    state: &State<'_, AppState>,
    op: impl FnOnce(&AttributesService<'_>) -> psm_core::Result<T>,
) -> Result<T, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?;
    let integration = integration(&app);
    let service = AttributesService::new(app.store.as_ref(), &SystemClock, integration.as_deref());
    op(&service).map_err(Into::into)
}

/// The active definitions: title, description, tags, then the integration's (ch08 §Schema).
#[tauri::command]
#[specta::specta]
pub fn attributes_definitions_list(
    state: State<'_, AppState>,
) -> Result<Vec<DefinitionDto>, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?;
    let integration = integration(&app);
    AttributesService::new(app.store.as_ref(), &SystemClock, integration.as_deref())
        .definitions(integration.as_deref())
        .map(|defs| defs.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub fn attribute_values_get(
    state: State<'_, AppState>,
    item_id: CommandId,
) -> Result<Vec<AttributeValueDto>, CommandError> {
    with_service(&state, |s| s.values(item_id.0))
        .map(|values| values.into_iter().map(Into::into).collect())
}

/// A human edit of one field: `value` is JSON text, stored canonical or, if it does not fit the
/// definition, verbatim with `valid = false` (MC-9).
#[tauri::command]
#[specta::specta]
pub fn attribute_values_set(
    state: State<'_, AppState>,
    item_id: CommandId,
    key: String,
    value: String,
) -> Result<AttributeValueDto, CommandError> {
    let raw = parse(&value)?;
    with_service(&state, |s| {
        s.set_value(item_id.0, &key, raw, Provenance::Human)
    })
    .map(Into::into)
}

/// "Accept all": suggested AI values except low-confidence ones become accepted. Returns the count.
#[tauri::command]
#[specta::specta]
pub fn attribute_ai_accept_all(
    state: State<'_, AppState>,
    item_id: CommandId,
) -> Result<u32, CommandError> {
    with_service(&state, |s| s.accept_all(item_id.0)).map(|n| u32::try_from(n).unwrap_or(u32::MAX))
}

#[tauri::command]
#[specta::specta]
pub fn attribute_ai_clear(
    state: State<'_, AppState>,
    item_id: CommandId,
    key: String,
) -> Result<(), CommandError> {
    with_service(&state, |s| s.clear(item_id.0, &key))
}

#[tauri::command]
#[specta::specta]
pub fn presets_get(
    state: State<'_, AppState>,
    item_type: String,
    group: AttributeGroupDto,
) -> Result<PresetDto, CommandError> {
    with_service(&state, |s| s.presets_get(&item_type, group.into())).map(Into::into)
}

/// Customize fields: stores the layout of one step for one item type (refused with
/// `invalid_value` for unknown keys, ill-fitting defaults, or a Product step without title,
/// description and tags).
#[tauri::command]
#[specta::specta]
pub fn presets_set(state: State<'_, AppState>, preset: PresetDto) -> Result<(), CommandError> {
    let mut layout = Vec::with_capacity(preset.layout.len());
    for field in preset.layout {
        layout.push(PresetField {
            key: field.key,
            default: field.default.as_deref().map(parse).transpose()?,
        });
    }
    let preset = Preset {
        item_type: preset.item_type,
        group: preset.group.into(),
        layout,
    };
    with_service(&state, |s| s.presets_set(preset))
}

/// The item's form: Details, Condition, Product with ordered fields, values and fill counts.
#[tauri::command]
#[specta::specta]
pub fn form_get(state: State<'_, AppState>, item_id: CommandId) -> Result<FormDto, CommandError> {
    with_service(&state, |s| s.form_get(item_id.0)).map(Into::into)
}
