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
        publish::RemoteValues,
    },
    mechanisms::values::{
        PhotoId, canonical_json, canonicalize, is_severity_set, selected_condition_photos,
    },
    ports::{
        clock::Clock,
        commerce::CommerceAdapter,
        store::{Store, StoreTx},
    },
    services::history::{FieldChange, FieldState, History, Step, StepKind},
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
        let Some(preset) = resolve_preset(tx, item_type, group)? else {
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

fn resolve_preset(
    tx: &dyn StoreTx,
    item_type: &str,
    group: AttributeGroup,
) -> Result<Option<Preset>> {
    let own = tx.attribute_preset(item_type, group)?;
    if own.is_some() || item_type.is_empty() {
        return Ok(own);
    }
    Ok(tx.attribute_preset("", group)?.map(|mut preset| {
        preset.item_type = item_type.to_owned();
        preset
    }))
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

/// The link integration of the linked product the form reads (the store keys links by it).
const LINK_INTEGRATION: &str = "shopify";

/// No value: null, a blank string, an empty list or object.
fn is_empty_json(value: &Json) -> bool {
    match value {
        Json::Null => true,
        Json::String(text) => text.trim().is_empty(),
        Json::Array(items) => items.is_empty(),
        Json::Object(map) => map.is_empty(),
        _ => false,
    }
}

/// Kinds that never take a default or a saved value: the damage report is per session, a
/// reference is never written (MC-9).
fn takes_no_default(definition: &Definition) -> bool {
    matches!(
        definition.core_type,
        CoreType::MediaList | CoreType::Reference
    ) || definition.key == PRODUCT_TITLE
}

/// The default of every field of `item_type`'s form (the resolved preset of each step).
fn defaults_for(tx: &dyn StoreTx, item_type: &str) -> Result<BTreeMap<String, Json>> {
    let mut out = BTreeMap::new();
    for group in AttributeGroup::ALL {
        if let Some(preset) = resolve_preset(tx, item_type, group)? {
            for field in preset.layout {
                if let Some(default) = field.default {
                    out.insert(field.key, default);
                }
            }
        }
    }
    Ok(out)
}

/// P67: the outcome of reading one item's linked product into its form (changelog 46 (3)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteRead {
    /// The item has no linked product: nothing was read.
    NotLinked,
    Read,
    /// The read failed; the item keeps its local values.
    Failed(String),
}

/// Computes the row to store from the definition, the stored row and the current time.
type NextRow<'n> = dyn FnMut(&Definition, Option<&Value>, i64) -> Result<Option<Value>> + 'n;

pub struct AttributesService<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
    integration: Option<&'a str>,
    history: Option<&'a History>,
}

impl<'a> AttributesService<'a> {
    /// Read the remote schema before opening the transaction, then replace both tables together.
    pub fn refresh_schema(&self, adapter: &dyn CommerceAdapter) -> Result<()> {
        let schema = adapter.attributes_schema()?;
        let definitions: Vec<_> = schema
            .iter()
            .map(|(definition, _)| definition.clone())
            .collect();
        let links: Vec<_> = schema
            .into_iter()
            .map(|(definition, link)| (definition.key, link))
            .collect();
        self.store.transaction(&mut |tx| {
            tx.replace_attribute_definitions("shopify", &definitions)?;
            tx.replace_attribute_definition_links("shopify", &links)
        })
    }
    /// `integration` = the configured integration (its definitions are the active set), or `None`.
    pub fn new(store: &'a dyn Store, clock: &'a dyn Clock, integration: Option<&'a str>) -> Self {
        Self {
            store,
            clock,
            integration,
            history: None,
        }
    }
    pub fn with_history(mut self, history: &'a History) -> Self {
        self.history = Some(history);
        self
    }
    fn record(&self, item_id: i64, label: String, fields: Vec<FieldChange>) -> Result<()> {
        if fields.is_empty() {
            return Ok(());
        }
        if let (Some(history), Some(session_id)) =
            (self.history, self.store.session_for_item(item_id)?)
        {
            history.record(
                session_id,
                Step {
                    label,
                    kind: StepKind::Attribute {
                        session_id,
                        item_id,
                        fields,
                    },
                },
            );
        }
        Ok(())
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
    fn write(
        &self,
        item_id: i64,
        key: &str,
        action: Option<&str>,
        next: &mut NextRow<'_>,
    ) -> Result<Option<Value>> {
        let now = self.clock.now_ms();
        let mut result = None;
        let mut change = None;
        let mut label = String::new();
        self.store.transaction(&mut |tx| {
            tx.item(item_id)?;
            let definition = active_definitions(tx, self.integration)?
                .into_iter()
                .find(|d| d.key == key)
                .ok_or_else(|| unknown(key))?;
            label = definition.label.clone();
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
            change = Some(FieldChange {
                key: key.into(),
                before: FieldState::Row(stored),
                after: FieldState::Row(Some(row.clone())),
            });
            result = Some(row).filter(Value::is_live);
            Ok(())
        })?;
        if let (Some(action), Some(change)) = (action, change) {
            self.record(item_id, format!("{action} {label}"), vec![change])?;
        }
        Ok(result)
    }

    /// Sets the item's title, the single home of `product.title` (ch05): `items.title`, not a value
    /// row. `None` empties it. A title equal to the stored one is not written (H4); a write sets the
    /// item's sessions' activity and leaves `change_seq` alone (MC-4, MC-6).
    fn write_title(&self, item_id: i64, title: Option<String>, action: &str) -> Result<Value> {
        let now = self.clock.now_ms();
        let title = title.unwrap_or_default();
        let mut result = None;
        let mut before = None;
        self.store.transaction(&mut |tx| {
            let item = tx.item(item_id)?;
            if item.title != title {
                before = Some(item.title.clone());
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
        if let Some(old) = before {
            self.record(
                item_id,
                format!("{action} Title"),
                vec![FieldChange {
                    key: PRODUCT_TITLE.into(),
                    before: FieldState::Title(old),
                    after: FieldState::Title(title),
                }],
            )?;
        }
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
            return self.write_title(item_id, Some(title), "Edit");
        }
        let mut raw = Some(raw);
        // 10F Q3: a cleared item keeps only the attributes that need no images (read before the
        // write's transaction holds the store).
        let cleared = self.store.item_retention(item_id)?.cleared_at.is_some();
        self.write(
            item_id,
            key,
            Some("Edit"),
            &mut |definition, stored, now| {
                if cleared && definition.core_type == CoreType::MediaList {
                    return Err(Error::FilesCleared);
                }
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
            },
        )?
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
        self.write(item_id, key, None, &mut |definition, stored, now| {
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
        let mut changes = Vec::new();
        self.store.transaction(&mut |tx| {
            tx.item(item_id)?;
            count = 0;
            for mut value in tx.attribute_values(item_id)? {
                if value.provenance == Provenance::Ai
                    && value.ai_status == Some(AiStatus::Suggested)
                    && value.ai_confidence != Some(AiConfidence::Low)
                {
                    let before = value.clone();
                    value.ai_status = Some(AiStatus::Accepted);
                    value.updated_at = now;
                    tx.put_attribute_value(&value)?;
                    changes.push(FieldChange {
                        key: value.key.clone(),
                        before: FieldState::Row(Some(before)),
                        after: FieldState::Row(Some(value)),
                    });
                    count += 1;
                }
            }
            if count > 0 {
                tx.set_item_activity(item_id, now)?;
            }
            Ok(())
        })?;
        self.record(item_id, "Accept all suggestions".into(), changes)?;
        Ok(count)
    }

    /// Clears the field: the row is kept with `ai_status = cleared` and is no longer one of the
    /// item's values. Clearing an empty or already cleared field changes nothing.
    pub fn clear(&self, item_id: i64, key: &str) -> Result<()> {
        if key == PRODUCT_TITLE {
            self.write_title(item_id, None, "Clear")?;
            return Ok(());
        }
        self.write(item_id, key, Some("Clear"), &mut |_, stored, now| {
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
            out = Some(match resolve_preset(tx, item_type, group)? {
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

    /// P67 (changelog 46 (3)): reads the item's linked product once (one adapter request) and
    /// shows its values in the form. A field the photographer owns locally (a human or AI value,
    /// or one they cleared) is never replaced; a default or an earlier read is replaced by a
    /// non-empty product value; a field the product leaves empty gets the item type's default
    /// (changelog 46 (4)). The type follows the product while the local type is the one last read
    /// (changelog 42); so does the title. Nothing is written to the platform.
    pub fn read_integration_values(
        &self,
        adapter: Option<&dyn CommerceAdapter>,
        item_id: i64,
    ) -> Result<RemoteRead> {
        let Some(link) = self.store.item_link(item_id, LINK_INTEGRATION)? else {
            return Ok(RemoteRead::NotLinked);
        };
        let Some(adapter) = adapter else {
            return Ok(RemoteRead::Failed(Error::NoIntegration.to_string()));
        };
        let remote = match adapter.product_values(&link.external_id) {
            Ok(remote) => remote,
            Err(error) => return Ok(RemoteRead::Failed(error.to_string())),
        };
        if let Some(item_type) = remote.item_type.as_deref() {
            let local = self.store.item(item_id)?;
            if link
                .item_type_base
                .as_deref()
                .is_none_or(|base| base == local.item_type)
            {
                crate::services::items::ItemsService::new(self.store, self.clock)
                    .set_type(item_id, item_type)?;
            }
            self.store
                .set_item_type_base(item_id, LINK_INTEGRATION, item_type)?;
        }
        self.apply_remote(item_id, &remote)?;
        Ok(RemoteRead::Read)
    }

    fn apply_remote(&self, item_id: i64, remote: &RemoteValues) -> Result<()> {
        let now = self.clock.now_ms();
        let mut incoming: BTreeMap<String, Json> = remote.attributes.iter().cloned().collect();
        incoming.insert(PRODUCT_TAGS.into(), Json::from(remote.tags.clone()));
        incoming.insert(
            PRODUCT_DESCRIPTION.into(),
            remote.description.clone().map_or(Json::Null, Json::String),
        );
        self.store.transaction(&mut |tx| {
            let item = tx.item(item_id)?;
            let title = remote.title.trim();
            if !title.is_empty() {
                let base = tx.item_title_base(item_id, LINK_INTEGRATION)?;
                let untouched =
                    item.title.trim().is_empty() || base.as_deref() == Some(item.title.as_str());
                if untouched && item.title != remote.title {
                    tx.update_item_title(item_id, &remote.title, now)?;
                }
                tx.set_item_title_base(item_id, LINK_INTEGRATION, &remote.title)?;
            }
            let stored: BTreeMap<String, Value> = tx
                .attribute_values(item_id)?
                .into_iter()
                .map(|v| (v.key.clone(), v))
                .collect();
            let defaults = defaults_for(tx, &item.item_type)?;
            for definition in active_definitions(tx, self.integration)? {
                if takes_no_default(&definition) {
                    continue;
                }
                let current = stored.get(&definition.key);
                if current.is_some_and(|v| {
                    !v.is_live() || matches!(v.provenance, Provenance::Human | Provenance::Ai)
                }) {
                    continue;
                }
                let row = |value: String, valid: bool, provenance: Provenance| Value {
                    item_id,
                    key: definition.key.clone(),
                    value,
                    valid,
                    provenance,
                    ai_confidence: None,
                    ai_status: None,
                    ai_reason: None,
                    ai_sources: None,
                    updated_at: now,
                };
                let next = match incoming.get(&definition.key).filter(|v| !is_empty_json(v)) {
                    Some(raw) => {
                        let (value, valid) = stored_form(&definition, raw.clone());
                        Some(row(value, valid, Provenance::Integration))
                    }
                    None => defaults
                        .get(&definition.key)
                        .map(|default| row(canonical_json(default), true, Provenance::Default)),
                };
                match (current, next) {
                    (Some(current), Some(next)) if same_content(current, &next) => {}
                    (_, Some(next)) => tx.put_attribute_value(&next)?,
                    (Some(current), None) if current.provenance == Provenance::Integration => {
                        tx.delete_attribute_value(item_id, &definition.key)?
                    }
                    _ => {}
                }
            }
            Ok(())
        })
    }

    /// P67 (changelog 46 (4)): "Save as default" — the field's default for every item of
    /// `item_type`, stored in the step layout of that type (the default Customize fields shows).
    /// `None` or an empty value removes it. Returns the stored default (canonical).
    pub fn default_set(
        &self,
        item_type: &str,
        key: &str,
        value: Option<Json>,
    ) -> Result<Option<Json>> {
        let definition = self
            .definitions(self.integration)?
            .into_iter()
            .find(|d| d.key == key)
            .ok_or_else(|| unknown(key))?;
        if takes_no_default(&definition) {
            return Err(Error::InvalidValue(format!("{key} takes no default")));
        }
        let mut target = None;
        for group in AttributeGroup::ALL {
            let preset = self.presets_get(item_type, group)?;
            if preset.layout.iter().any(|f| f.key == key) {
                target = Some(preset);
                break;
            }
        }
        let mut preset = match target {
            Some(preset) => preset,
            None => {
                let mut preset = self.presets_get(item_type, definition.group)?;
                preset.layout.push(PresetField {
                    key: key.to_owned(),
                    default: None,
                });
                preset
            }
        };
        preset.item_type = item_type.to_owned();
        let group = preset.group;
        let value = value.filter(|v| !is_empty_json(v));
        for field in &mut preset.layout {
            if field.key == key {
                field.default = value.clone();
            }
        }
        self.presets_set(preset)?;
        Ok(self
            .presets_get(item_type, group)?
            .layout
            .into_iter()
            .find(|f| f.key == key)
            .and_then(|f| f.default))
    }

    fn saved_definition(&self, key: &str) -> Result<Definition> {
        let definition = self
            .definitions(self.integration)?
            .into_iter()
            .find(|d| d.key == key)
            .ok_or_else(|| unknown(key))?;
        if takes_no_default(&definition) {
            return Err(Error::InvalidValue(format!("{key} takes no saved values")));
        }
        Ok(definition)
    }

    /// P67 (changelog 46 (4)): the saved values of one field for one item type, most recently
    /// saved first.
    pub fn saved_values(&self, item_type: &str, key: &str) -> Result<Vec<Json>> {
        let mut out = Vec::new();
        self.store.transaction(&mut |tx| {
            out = tx
                .saved_attribute_values(item_type, key)?
                .iter()
                .filter_map(|text| serde_json::from_str(text).ok())
                .collect();
            Ok(())
        })?;
        Ok(out)
    }

    /// "Save value": adds the value (canonical; refused when it does not fit the field) to the
    /// field's saved values for `item_type`, or moves it to the top. Returns the saved values.
    pub fn saved_value_add(&self, item_type: &str, key: &str, raw: Json) -> Result<Vec<Json>> {
        let definition = self.saved_definition(key)?;
        if is_empty_json(&raw) {
            return Err(Error::InvalidValue(format!("{key}: nothing to save")));
        }
        let canonical = canonicalize(definition.core_type, &definition, raw)
            .map_err(|e| Error::InvalidValue(format!("{key}: {}", e.0)))?;
        let now = self.clock.now_ms();
        self.store.transaction(&mut |tx| {
            tx.put_saved_attribute_value(item_type, key, &canonical.0, now)
        })?;
        self.saved_values(item_type, key)
    }

    /// Removes one saved value of the field for `item_type`. Returns the saved values left.
    pub fn saved_value_remove(&self, item_type: &str, key: &str, raw: Json) -> Result<Vec<Json>> {
        let definition = self.saved_definition(key)?;
        let text = canonicalize(definition.core_type, &definition, raw.clone())
            .map(|c| c.0)
            .unwrap_or_else(|_| canonical_json(&raw));
        self.store
            .transaction(&mut |tx| tx.delete_saved_attribute_value(item_type, key, &text))?;
        self.saved_values(item_type, key)
    }

    /// P67 (changelog 46 (4)): "Fill defaults" — every field of the item's form that has no value
    /// and has a default for the item's type gets it (provenance `default`); filled fields are
    /// left alone. One undoable step. Returns how many were filled.
    pub fn fill_defaults(&self, item_id: i64) -> Result<usize> {
        let now = self.clock.now_ms();
        let mut changes = Vec::new();
        self.store.transaction(&mut |tx| {
            changes.clear();
            let item = tx.item(item_id)?;
            let stored: BTreeMap<String, Value> = tx
                .attribute_values(item_id)?
                .into_iter()
                .map(|v| (v.key.clone(), v))
                .collect();
            let definitions = active_definitions(tx, self.integration)?;
            for (key, default) in defaults_for(tx, &item.item_type)? {
                let Some(_definition) = definitions
                    .iter()
                    .find(|d| d.key == key && !takes_no_default(d))
                else {
                    continue;
                };
                let before = stored.get(&key);
                if before.is_some_and(Value::is_live) {
                    continue;
                }
                let row = Value {
                    item_id,
                    key: key.clone(),
                    value: canonical_json(&default),
                    valid: true,
                    provenance: Provenance::Default,
                    ai_confidence: None,
                    ai_status: None,
                    ai_reason: None,
                    ai_sources: None,
                    updated_at: now,
                };
                tx.put_attribute_value(&row)?;
                changes.push(FieldChange {
                    key,
                    before: FieldState::Row(before.cloned()),
                    after: FieldState::Row(Some(row)),
                });
            }
            if !changes.is_empty() {
                tx.set_item_activity(item_id, now)?;
            }
            Ok(())
        })?;
        let count = changes.len();
        self.record(item_id, "Fill defaults".into(), changes)?;
        Ok(count)
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
        let link = self.store.item_link(item_id, LINK_INTEGRATION)?;
        let mut groups = Vec::new();
        let mut item_type_from_integration = false;
        self.store.transaction(&mut |tx| {
            let item = tx.item(item_id)?;
            item_type_from_integration = link.as_ref().is_some_and(|link| {
                !item.item_type.is_empty()
                    && link.item_type_base.as_deref() == Some(item.item_type.as_str())
            });
            let title_from_integration = link.is_some()
                && !item.title.trim().is_empty()
                && tx.item_title_base(item_id, LINK_INTEGRATION)?.as_deref()
                    == Some(item.title.as_str());
            let definitions = active_definitions(tx, self.integration)?;
            let values: BTreeMap<String, Value> = tx
                .attribute_values(item_id)?
                .into_iter()
                .filter(Value::is_live)
                .map(|v| (v.key.clone(), v))
                .collect();
            groups.clear();
            for group in AttributeGroup::ALL {
                let preset = resolve_preset(tx, &item.item_type, group)?;
                let ordered: Vec<(&Definition, Option<Json>)> = match preset {
                    Some(preset) => preset
                        .layout
                        .into_iter()
                        .filter_map(|f| {
                            definitions
                                .iter()
                                .find(|d| d.key == f.key)
                                .map(|d| (d, f.default))
                        })
                        .collect(),
                    None => definitions
                        .iter()
                        .filter(|d| d.group == group)
                        .map(|d| (d, None))
                        .collect(),
                };
                let mut fields = Vec::with_capacity(ordered.len());
                for (definition, default) in ordered {
                    let saved: Vec<Json> = if takes_no_default(definition) {
                        Vec::new()
                    } else {
                        tx.saved_attribute_values(&item.item_type, &definition.key)?
                            .iter()
                            .filter_map(|text| serde_json::from_str(text).ok())
                            .collect()
                    };
                    fields.push((definition, default, saved));
                }
                let fields: Vec<FormField> = fields
                    .into_iter()
                    .map(|(definition, default, saved)| {
                        let value = if definition.key == PRODUCT_TITLE {
                            title_value(&item).map(|mut value| {
                                if title_from_integration {
                                    value.provenance = Provenance::Integration;
                                }
                                value
                            })
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
                            default: default.filter(|_| !takes_no_default(definition)),
                            saved,
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
        Ok(Form {
            item_id,
            groups,
            item_type_from_integration,
        })
    }
}
