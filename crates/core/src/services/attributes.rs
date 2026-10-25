//! Attributes (ch08): definitions imported from an integration (or only the built-in Product fields
//! without one), values in canonical form with provenance (05A MC-9), presets per item type and
//! step, the form (`form_get`) with the damage-report selection (07A MC-14). Every write sets the
//! item's sessions' `last_activity_at` (MC-4) and never touches `change_seq` (MC-6).

use crate::{
    Error, Result,
    domain::{
        attribute::{
            AiConfidence, AiStatus, AttributeGroup, CoreType, Definition, Form, FormField,
            FormGroup, Preset, PresetField, Provenance, Value,
        },
        item::Item,
        photo::Classification,
    },
    mechanisms::values::{
        PhotoId, canonical_json, canonicalize, is_severity_set, selected_condition_photos,
    },
    ports::{
        clock::Clock,
        store::{Store, StoreTx},
    },
};
use serde_json::Value as Json;
use std::collections::{BTreeMap, HashSet};

/// The integration name of the built-in definitions.
pub const CORE_INTEGRATION: &str = "core";
pub const PRODUCT_TITLE: &str = "product.title";
pub const PRODUCT_DESCRIPTION: &str = "product.description";
pub const PRODUCT_TAGS: &str = "product.tags";
const PRODUCT_FIELDS: [&str; 3] = [PRODUCT_TITLE, PRODUCT_DESCRIPTION, PRODUCT_TAGS];

/// Title, description and tags: always present, never removable (ch08 §Groups and presets).
pub fn builtin_definitions() -> Vec<Definition> {
    [
        (PRODUCT_TITLE, "Title", CoreType::Text),
        (PRODUCT_DESCRIPTION, "Description", CoreType::RichText),
        (PRODUCT_TAGS, "Tags", CoreType::Tags),
    ]
    .into_iter()
    .map(|(key, label, core_type)| Definition {
        integration: CORE_INTEGRATION.into(),
        key: key.into(),
        label: label.into(),
        core_type,
        choices: Vec::new(),
        validations: serde_json::json!({}),
        group: AttributeGroup::Product,
        description: None,
        writable: true,
    })
    .collect()
}

fn active_definitions(tx: &dyn StoreTx, integration: Option<&str>) -> Result<Vec<Definition>> {
    let mut all = builtin_definitions();
    if let Some(integration) = integration {
        all.extend(tx.attribute_definitions(integration)?);
    }
    Ok(all)
}

fn unknown(key: &str) -> Error {
    Error::InvalidValue(format!("unknown attribute {key}"))
}

/// The stored form of `raw` for `definition`: canonical and valid, or verbatim and invalid (MC-9).
fn stored_form(definition: &Definition, raw: Json) -> (String, bool) {
    let verbatim = raw.to_string();
    match canonicalize(definition.core_type, definition, raw) {
        Ok(canonical) => (canonical.0, true),
        Err(_) => (verbatim, false),
    }
}

/// Same stored content (everything but `updated_at`): writing it again would change nothing.
fn same_content(a: &Value, b: &Value) -> bool {
    a.value == b.value
        && a.valid == b.valid
        && a.provenance == b.provenance
        && a.ai_confidence == b.ai_confidence
        && a.ai_status == b.ai_status
        && a.ai_reason == b.ai_reason
        && a.ai_sources == b.ai_sources
}

/// Writes the preset defaults of `item_type` as `default` values for keys the item has no value
/// for yet. Called in the transaction that creates a session (ch08: presets pre-fill new sessions).
pub fn prefill_defaults(
    tx: &mut dyn StoreTx,
    item_id: i64,
    item_type: &str,
    at: i64,
) -> Result<()> {
    let existing: HashSet<String> = tx
        .attribute_values(item_id)?
        .into_iter()
        .map(|v| v.key)
        .collect();
    for group in AttributeGroup::ALL {
        let Some(preset) = tx.attribute_preset(item_type, group)? else {
            continue;
        };
        for field in preset.layout {
            let Some(default) = field.default else {
                continue;
            };
            if field.key == PRODUCT_TITLE {
                continue; // the title is the item's own (ch05), never a stored row
            }
            if existing.contains(&field.key) {
                continue;
            }
            tx.put_attribute_value(&Value {
                item_id,
                key: field.key,
                value: canonical_json(&default),
                valid: true,
                provenance: Provenance::Default,
                ai_confidence: None,
                ai_status: None,
                ai_reason: None,
                ai_sources: None,
                updated_at: at,
            })?;
        }
    }
    Ok(())
}

/// The item's title presented as the value of `product.title`: the title has one home, `items.title`
/// (ch05), so this is the single source of the field. An empty title is no value.
fn title_value(item: &Item) -> Option<Value> {
    (!item.title.trim().is_empty()).then(|| Value {
        item_id: item.id,
        key: PRODUCT_TITLE.to_owned(),
        value: canonical_json(&Json::String(item.title.clone())),
        valid: true,
        provenance: Provenance::Human,
        ai_confidence: None,
        ai_status: None,
        ai_reason: None,
        ai_sources: None,
        updated_at: item.updated_at,
    })
}

/// Computes the row to store from the definition, the stored row and the current time.
type NextRow<'n> = dyn FnMut(&Definition, Option<&Value>, i64) -> Result<Option<Value>> + 'n;

pub struct AttributesService<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
    integration: Option<&'a str>,
}

impl<'a> AttributesService<'a> {
    /// `integration` = the configured integration (its definitions are the active set), or `None`.
    pub fn new(store: &'a dyn Store, clock: &'a dyn Clock, integration: Option<&'a str>) -> Self {
        Self {
            store,
            clock,
            integration,
        }
    }

    /// Replaces the definition set of `integration`. `reference` definitions are never writable
    /// (MC-9 "never written").
    pub fn import_definitions(
        &self,
        integration: &str,
        definitions: Vec<Definition>,
    ) -> Result<()> {
        if integration == CORE_INTEGRATION {
            return Err(Error::InvalidValue(
                "the built-in definitions cannot be replaced".into(),
            ));
        }
        let mut seen: HashSet<String> = PRODUCT_FIELDS.iter().map(|k| k.to_string()).collect();
        let mut prepared = Vec::with_capacity(definitions.len());
        for mut definition in definitions {
            if !seen.insert(definition.key.clone()) {
                return Err(Error::InvalidValue(format!(
                    "attribute {} defined twice",
                    definition.key
                )));
            }
            definition.integration = integration.to_owned();
            if definition.core_type == CoreType::Reference {
                definition.writable = false;
            }
            prepared.push(definition);
        }
        self.store
            .transaction(&mut |tx| tx.replace_attribute_definitions(integration, &prepared))
    }

    /// The built-in Product fields, then the definitions of `integration` in import order.
    pub fn definitions(&self, integration: Option<&str>) -> Result<Vec<Definition>> {
        let mut out = Vec::new();
        self.store.transaction(&mut |tx| {
            out = active_definitions(tx, integration)?;
            Ok(())
        })?;
        Ok(out)
    }

    /// One read-modify-write of the value (`item_id`, `key`) in a transaction. `next` gets the
    /// definition and the stored row and returns the row to store; a row with the same content as
    /// the stored one is not written and leaves `updated_at` and `last_activity_at` alone (M16).
    fn write(&self, item_id: i64, key: &str, next: &mut NextRow<'_>) -> Result<Option<Value>> {
        let now = self.clock.now_ms();
        let mut result = None;
        self.store.transaction(&mut |tx| {
            tx.item(item_id)?;
            let definition = active_definitions(tx, self.integration)?
                .into_iter()
                .find(|d| d.key == key)
                .ok_or_else(|| unknown(key))?;
            let stored = tx
                .attribute_values(item_id)?
                .into_iter()
                .find(|v| v.key == key);
            result = stored.clone().filter(Value::is_live);
            let Some(row) = next(&definition, stored.as_ref(), now)? else {
                return Ok(());
            };
            if stored.as_ref().is_some_and(|s| same_content(s, &row)) {
                return Ok(());
            }
            tx.put_attribute_value(&row)?;
            tx.set_item_activity(item_id, now)?;
            result = Some(row).filter(Value::is_live);
            Ok(())
        })?;
        Ok(result)
    }

    /// Sets the item's title, the single home of `product.title` (ch05): `items.title`, not a value
    /// row. `None` empties it. A title equal to the stored one is not written (H4); a write sets the
    /// item's sessions' activity and leaves `change_seq` alone (MC-4, MC-6).
    fn write_title(&self, item_id: i64, title: Option<String>) -> Result<Value> {
        let now = self.clock.now_ms();
        let title = title.unwrap_or_default();
        let mut result = None;
        self.store.transaction(&mut |tx| {
            let item = tx.item(item_id)?;
            if item.title != title {
                tx.update_item_title(item_id, &title, now)?;
                tx.set_item_activity(item_id, now)?;
            }
            let stored = tx.item(item_id)?;
            result = Some(Value {
                item_id,
                key: PRODUCT_TITLE.to_owned(),
                value: canonical_json(&Json::String(stored.title)),
                valid: true,
                provenance: Provenance::Human,
                ai_confidence: None,
                ai_status: None,
                ai_reason: None,
                ai_sources: None,
                updated_at: stored.updated_at,
            });
            Ok(())
        })?;
        result.ok_or_else(|| Error::InvalidData("title not stored".into()))
    }

    /// Stores `raw` in canonical form (`valid = true`), or verbatim with `valid = false` when it
    /// does not fit (MC-9). A value equal to the stored one is not written. A human value is never
    /// replaced by a non-human one. `product.title` is the item's title: a human text sets
    /// `items.title` and no value row is stored.
    pub fn set_value(
        &self,
        item_id: i64,
        key: &str,
        raw: Json,
        provenance: Provenance,
    ) -> Result<Value> {
        if key == PRODUCT_TITLE {
            if provenance != Provenance::Human {
                return Err(Error::InvalidValue(format!(
                    "{key} is the item's title; only a human edit sets it"
                )));
            }
            let definition = builtin_definitions()
                .into_iter()
                .find(|d| d.key == PRODUCT_TITLE)
                .ok_or_else(|| unknown(key))?;
            let canonical = canonicalize(definition.core_type, &definition, raw)
                .map_err(|e| Error::InvalidValue(format!("{key}: {}", e.0)))?;
            let title: String = serde_json::from_str(&canonical.0)
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            return self.write_title(item_id, Some(title));
        }
        let mut raw = Some(raw);
        self.write(item_id, key, &mut |definition, stored, now| {
            if provenance != Provenance::Human
                && stored.is_some_and(|s| s.is_live() && s.provenance == Provenance::Human)
            {
                return Err(Error::InvalidValue(format!(
                    "{key} holds a human value; only a human edit replaces it"
                )));
            }
            let (value, valid) = stored_form(definition, raw.take().unwrap_or(Json::Null));
            Ok(Some(Value {
                item_id,
                key: key.to_owned(),
                value,
                valid,
                provenance,
                ai_confidence: None,
                ai_status: None,
                ai_reason: None,
                ai_sources: None,
                updated_at: now,
            }))
        })?
        .ok_or_else(|| Error::InvalidData(format!("value {key} not stored")))
    }

    /// An AI suggestion (ch08 §Values and provenance): refused on a field a human owns; otherwise
    /// stored with provenance `ai`, status `suggested`.
    pub fn ai_suggest(
        &self,
        item_id: i64,
        key: &str,
        raw: Json,
        confidence: AiConfidence,
        reason: Option<&str>,
        sources: Vec<String>,
    ) -> Result<Value> {
        let mut raw = Some(raw);
        self.write(item_id, key, &mut |definition, stored, now| {
            if stored.is_some_and(|s| s.provenance == Provenance::Human) {
                return Err(Error::InvalidValue(format!(
                    "{key} holds a human value; AI never overwrites it"
                )));
            }
            let (value, valid) = stored_form(definition, raw.take().unwrap_or(Json::Null));
            Ok(Some(Value {
                item_id,
                key: key.to_owned(),
                value,
                valid,
                provenance: Provenance::Ai,
                ai_confidence: Some(confidence),
                ai_status: Some(AiStatus::Suggested),
                ai_reason: reason.map(str::to_owned),
                ai_sources: Some(sources.clone()),
                updated_at: now,
            }))
        })?
        .ok_or_else(|| Error::InvalidData(format!("value {key} not stored")))
    }

    /// "Accept all": every suggested AI value except low-confidence ones becomes `accepted`.
    /// Returns how many were accepted.
    pub fn accept_all(&self, item_id: i64) -> Result<usize> {
        let now = self.clock.now_ms();
        let mut count = 0;
        self.store.transaction(&mut |tx| {
            tx.item(item_id)?;
            count = 0;
            for mut value in tx.attribute_values(item_id)? {
                if value.provenance == Provenance::Ai
                    && value.ai_status == Some(AiStatus::Suggested)
                    && value.ai_confidence != Some(AiConfidence::Low)
                {
                    value.ai_status = Some(AiStatus::Accepted);
                    value.updated_at = now;
                    tx.put_attribute_value(&value)?;
                    count += 1;
                }
            }
            if count > 0 {
                tx.set_item_activity(item_id, now)?;
            }
            Ok(())
        })?;
        Ok(count)
    }

    /// Clears the field: the row is kept with `ai_status = cleared` and is no longer one of the
    /// item's values. Clearing an empty or already cleared field changes nothing.
    pub fn clear(&self, item_id: i64, key: &str) -> Result<()> {
        if key == PRODUCT_TITLE {
            self.write_title(item_id, None)?;
            return Ok(());
        }
        self.write(item_id, key, &mut |_, stored, now| {
            Ok(stored.filter(|s| s.is_live()).map(|s| Value {
                ai_status: Some(AiStatus::Cleared),
                updated_at: now,
                ..s.clone()
            }))
        })?;
        Ok(())
    }

    /// The item's values (cleared ones excluded), valid or not, ordered by key.
    pub fn values(&self, item_id: i64) -> Result<Vec<Value>> {
        let mut out = Vec::new();
        self.store.transaction(&mut |tx| {
            tx.item(item_id)?;
            out = tx
                .attribute_values(item_id)?
                .into_iter()
                .filter(Value::is_live)
                .collect();
            Ok(())
        })?;
        Ok(out)
    }

    /// The layout of one step for one item type: the stored preset, else every active definition
    /// of the group in definition order, without defaults.
    pub fn presets_get(&self, item_type: &str, group: AttributeGroup) -> Result<Preset> {
        let mut out = None;
        self.store.transaction(&mut |tx| {
            out = Some(match tx.attribute_preset(item_type, group)? {
                Some(preset) => preset,
                None => Preset {
                    item_type: item_type.to_owned(),
                    group,
                    layout: active_definitions(tx, self.integration)?
                        .into_iter()
                        .filter(|d| d.group == group)
                        .map(|d| PresetField {
                            key: d.key,
                            default: None,
                        })
                        .collect(),
                },
            });
            Ok(())
        })?;
        out.ok_or_else(|| Error::InvalidData("preset".into()))
    }

    /// Stores a layout (Customize fields). Refused with `invalid_value` when a key is unknown or
    /// listed twice, a default does not fit its definition, or a Product layout lacks title,
    /// description or tags. Defaults are stored in canonical form.
    pub fn presets_set(&self, preset: Preset) -> Result<()> {
        if preset.group == AttributeGroup::Product {
            for key in PRODUCT_FIELDS {
                if !preset.layout.iter().any(|f| f.key == key) {
                    return Err(Error::InvalidValue(format!(
                        "{key} is never removable from the Product step"
                    )));
                }
            }
        }
        self.store.transaction(&mut |tx| {
            let definitions = active_definitions(tx, self.integration)?;
            let mut seen = HashSet::new();
            let mut layout = Vec::with_capacity(preset.layout.len());
            for field in &preset.layout {
                if !seen.insert(field.key.as_str()) {
                    return Err(Error::InvalidValue(format!("{} listed twice", field.key)));
                }
                let definition = definitions
                    .iter()
                    .find(|d| d.key == field.key)
                    .ok_or_else(|| unknown(&field.key))?;
                let default = match &field.default {
                    None => None,
                    Some(raw) => {
                        let canonical = canonicalize(definition.core_type, definition, raw.clone())
                            .map_err(|e| {
                                Error::InvalidValue(format!("default of {}: {}", field.key, e.0))
                            })?;
                        Some(
                            serde_json::from_str(&canonical.0)
                                .map_err(|e| Error::InvalidData(e.to_string()))?,
                        )
                    }
                };
                layout.push(PresetField {
                    key: field.key.clone(),
                    default,
                });
            }
            tx.put_attribute_preset(&Preset {
                item_type: preset.item_type.clone(),
                group: preset.group,
                layout,
            })
        })
    }

    /// The form of an item: Details, Condition, Product, each with its ordered fields (preset
    /// layout, else every definition of the group), their values, and `filled/total`. A
    /// `media_list` field carries the selected condition photos of the item's session (MC-14).
    pub fn form_get(&self, item_id: i64) -> Result<Form> {
        let condition: Vec<PhotoId> = match self.store.session_for_item(item_id)? {
            Some(session_id) => {
                let mut photos: Vec<_> = self
                    .store
                    .photos_in_session(session_id)?
                    .into_iter()
                    .filter(|p| p.classification == Classification::Condition)
                    .collect();
                photos.sort_by_key(|p| (p.position, p.id));
                photos.into_iter().map(|p| p.id).collect()
            }
            None => Vec::new(),
        };
        let mut groups = Vec::new();
        self.store.transaction(&mut |tx| {
            let item = tx.item(item_id)?;
            let definitions = active_definitions(tx, self.integration)?;
            let values: BTreeMap<String, Value> = tx
                .attribute_values(item_id)?
                .into_iter()
                .filter(Value::is_live)
                .map(|v| (v.key.clone(), v))
                .collect();
            groups.clear();
            for group in AttributeGroup::ALL {
                let ordered: Vec<&Definition> = match tx.attribute_preset(&item.item_type, group)? {
                    Some(preset) => preset
                        .layout
                        .iter()
                        .filter_map(|f| definitions.iter().find(|d| d.key == f.key))
                        .collect(),
                    None => definitions.iter().filter(|d| d.group == group).collect(),
                };
                let fields: Vec<FormField> = ordered
                    .into_iter()
                    .map(|definition| {
                        let value = if definition.key == PRODUCT_TITLE {
                            title_value(&item)
                        } else {
                            values.get(&definition.key).cloned()
                        };
                        let selected = (definition.core_type == CoreType::MediaList).then(|| {
                            let excluded: Vec<PhotoId> = value
                                .as_ref()
                                .filter(|v| v.valid)
                                .and_then(Value::json)
                                .and_then(|j| j.get("excluded").cloned())
                                .and_then(|e| serde_json::from_value(e).ok())
                                .unwrap_or_default();
                            selected_condition_photos(&condition, &excluded)
                        });
                        FormField {
                            is_default: value
                                .as_ref()
                                .is_some_and(|v| v.provenance == Provenance::Default),
                            is_severity: definition.core_type == CoreType::Choice
                                && is_severity_set(&definition.choices),
                            definition: definition.clone(),
                            value,
                            selected,
                        }
                    })
                    .collect();
                groups.push(FormGroup {
                    group,
                    filled: fields
                        .iter()
                        .filter(|f| f.value.as_ref().is_some_and(|v| v.valid))
                        .count(),
                    total: fields.len(),
                    fields,
                });
            }
            Ok(())
        })?;
        Ok(Form { item_id, groups })
    }
}
