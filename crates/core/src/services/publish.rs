//! Confirmed publish runs over the commerce port (07A MC-10…MC-12).
use crate::{
    Error, Result,
    domain::{
        attribute::CoreType,
        composition::{CompositionState, OutputSlot},
        output::slot_name,
        photo::{Classification, Integrity},
        publish::{
            ConfirmedRun, Notification, PhotoLink, PublishActivity, PublishItem, PublishPreview,
            PublishPreviewItem, StockChoice, StockLocations, StockOutcome,
        },
        session::PublishStep,
    },
    events::AppEvent,
    mechanisms::{
        attribute_write::{merge_tags, write_set},
        backoff::{EDITOR_WAIT_EXPIRED, WAITING_FOR_EDITOR},
        hash::sha256_file,
        naming::{ExportGroup, folder_key, output_names, review_label},
        publish::{frozen_main, publish_summary, replacement_candidates, upload_waves},
        reconcile,
        status::progress_percent,
        upload_name,
        values::canonicalize,
    },
    ports::{
        clock::Clock,
        commerce::{CommerceAdapter, StockDefaults},
        events::EventSink,
        store::Store,
    },
    services::{export::ExportService, integrity::IntegrityCheck},
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
};

/// One output of a photo list (10D MC-29): its group, photo, slot and staged file name.
struct Output {
    group: ExportGroup,
    photo_id: i64,
    slot: OutputSlot,
    name: String,
}

/// One `manifest.json` entry of a staging folder (P29): what the file was rendered from.
struct Staged {
    photo_id: i64,
    slot: String,
    adjusted_gen: i64,
    composition_rev: i64,
}

/// `manifest.json` of a staging folder: the output format key and the entries by file name.
fn read_manifest(staging: &Path) -> Option<(String, HashMap<String, Staged>)> {
    let text = std::fs::read_to_string(staging.join("manifest.json")).ok()?;
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    let format_key = json.get("format_key")?.as_str()?.to_string();
    let mut entries = HashMap::new();
    for (name, entry) in json.get("outputs")?.as_object()? {
        entries.insert(
            name.clone(),
            Staged {
                photo_id: entry.get("photo_id")?.as_i64()?,
                slot: entry.get("slot")?.as_str()?.to_string(),
                adjusted_gen: entry.get("adjusted_gen")?.as_i64()?,
                composition_rev: entry.get("composition_rev")?.as_i64()?,
            },
        );
    }
    Some((format_key, entries))
}

pub struct PublishService {
    store: Arc<dyn Store>,
    commerce: Arc<dyn CommerceAdapter>,
    export: Arc<ExportService>,
    clock: Arc<dyn Clock>,
    events: Arc<dyn EventSink>,
    exports: Mutex<HashMap<i64, i64>>,
    /// P64: what each running publish item is doing, its last error and a requested cancel
    /// (in memory: after a restart the persisted step and counts are shown).
    live: Mutex<HashMap<i64, Live>>,
    /// Changelog 45 (P65): the remembered stock default (Settings → Shopify); `None` in bare
    /// services.
    stock_defaults: Mutex<Option<Arc<dyn StockDefaults>>>,
    /// The platform's stock locations, read once per app run.
    stock_locations: Mutex<Option<StockLocations>>,
}

/// The integration whose links and stock this service writes.
const INTEGRATION: &str = "shopify";

/// P64: the in-memory activity of one publish item.
#[derive(Debug, Clone, Default)]
struct Live {
    started_at: i64,
    text: String,
    error: Option<String>,
    retried: bool,
    cancel: bool,
}

fn active(step: PublishStep) -> bool {
    matches!(
        step,
        PublishStep::Prepare | PublishStep::Upload | PublishStep::Update | PublishStep::Describe
    )
}

impl PublishService {
    pub fn new(
        store: Arc<dyn Store>,
        commerce: Arc<dyn CommerceAdapter>,
        export: Arc<ExportService>,
        clock: Arc<dyn Clock>,
        events: Arc<dyn EventSink>,
    ) -> Self {
        Self {
            store,
            commerce,
            export,
            clock,
            events,
            exports: Mutex::new(HashMap::new()),
            live: Mutex::new(HashMap::new()),
            stock_defaults: Mutex::new(None),
            stock_locations: Mutex::new(None),
        }
    }

    /// Changelog 45 (P65): where the stock default is read and remembered.
    pub fn set_stock_defaults(&self, defaults: Arc<dyn StockDefaults>) {
        *self
            .stock_defaults
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(defaults);
    }
    fn defaults(&self) -> Option<Arc<dyn StockDefaults>> {
        self.stock_defaults
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    /// The remembered stock default, `None` without one (or when it cannot be read).
    pub fn stock_default(&self) -> Option<StockChoice> {
        self.defaults().and_then(|d| match d.get() {
            Ok(choice) => choice,
            Err(error) => {
                tracing::warn!(%error, "stock default not read");
                None
            }
        })
    }
    /// Settings → Shopify: sets the remembered stock default.
    pub fn set_stock_default(&self, choice: &StockChoice) -> Result<()> {
        validate_stock(choice)?;
        self.defaults()
            .ok_or_else(|| Error::InvalidValue("stock default".into()))?
            .remember(choice)
    }
    /// The platform's active stock locations, read once per app run (a read that fails is asked
    /// again next time).
    pub fn stock_locations(&self) -> Result<StockLocations> {
        let mut cached = self
            .stock_locations
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(locations) = cached.as_ref() {
            return Ok(locations.clone());
        }
        let locations = self.commerce.stock_locations()?;
        *cached = Some(locations.clone());
        Ok(locations)
    }
    /// Whether the item is not on the platform yet (a publish will create it).
    pub fn is_new_item(&self, item_id: i64) -> Result<bool> {
        Ok(self.store.item_link(item_id, INTEGRATION)?.is_none())
    }
    /// The item form's stock for a new item: its own choice, else the remembered default.
    /// `None` for an item already on the platform (its stock is the platform's).
    pub fn item_stock(&self, item_id: i64) -> Result<Option<StockChoice>> {
        if self.store.item_link(item_id, INTEGRATION)?.is_some() {
            return Ok(None);
        }
        Ok(match self.store.item_stock(item_id, INTEGRATION)? {
            // "No location" chosen in the form: created untracked, whatever the default.
            Some(choice) if choice.location_id.is_empty() => None,
            Some(choice) => Some(choice),
            None => self.stock_default(),
        })
    }
    /// The item form sets the stock a new item will be created with; `None` = no location (the
    /// item is created without stock tracking).
    pub fn set_item_stock(&self, item_id: i64, choice: Option<&StockChoice>) -> Result<()> {
        let none = StockChoice {
            location_id: String::new(),
            location_name: String::new(),
            quantity: 0,
        };
        let stored = match choice {
            Some(choice) => {
                validate_stock(choice)?;
                choice
            }
            None => &none,
        };
        self.store.set_item_stock(item_id, INTEGRATION, Some(stored))
    }

    fn with_live<T>(&self, item_id: i64, f: impl FnOnce(&mut Live) -> T) -> T {
        let mut live = self.live.lock().unwrap_or_else(|e| e.into_inner());
        let now = self.clock.now_ms();
        f(live.entry(item_id).or_insert_with(|| Live {
            started_at: now,
            ..Live::default()
        }))
    }
    fn set_text(&self, item_id: i64, text: impl Into<String>) {
        let text = text.into();
        self.with_live(item_id, |live| live.text = text);
    }
    fn cancel_requested(&self, item_id: i64) -> bool {
        self.live
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&item_id)
            .is_some_and(|live| live.cancel)
    }
    /// Changelog 44 (P64): asks a running publish item to stop at its next safe point: after
    /// the request in flight, never between a mutation and recording its result (H4). Nothing
    /// already on the platform is deleted; the next publish reuses it.
    pub fn cancel(&self, item_id: i64) -> Result<()> {
        let item = self.store.publish_item(item_id)?;
        if !active(item.step) {
            return Err(Error::InvalidValue("publish item is not running".into()));
        }
        self.with_live(item_id, |live| live.cancel = true);
        self.emit(&item, item.step);
        Ok(())
    }
    /// P64: what the progress popover shows for the session's latest publish item.
    pub fn activity(&self, session_id: i64) -> Result<Option<PublishActivity>> {
        let Some(latest) = self.store.latest_publish(session_id)? else {
            return Ok(None);
        };
        let item = self.store.publish_item(latest.item_id)?;
        let live = self
            .live
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&item.id)
            .cloned();
        let running = active(item.step);
        let waiting = running
            && item.step == PublishStep::Prepare
            && item
                .export_run_id
                .is_some_and(|export| self.export.waiting(export));
        let text = match item.step {
            _ if running && live.as_ref().is_some_and(|l| l.cancel) => "Cancelling…".to_string(),
            // Changelog 45 (P65): a render waits for Capture One; the publish is queued, not failed.
            _ if waiting => WAITING_FOR_EDITOR.to_string(),
            _ if running && live.as_ref().is_some_and(|l| !l.text.is_empty()) => {
                live.as_ref().map(|l| l.text.clone()).unwrap_or_default()
            }
            PublishStep::Prepare => "Preparing outputs".into(),
            PublishStep::Upload => format!("Uploading photos {} of {}", item.uploaded, item.total),
            PublishStep::Update => "Updating the product".into(),
            PublishStep::Describe => "Writing the description".into(),
            PublishStep::Done => "Published".into(),
            PublishStep::Failed => "Publish failed".into(),
            PublishStep::Cancelled => "Publish cancelled".into(),
        };
        let started_at = match &live {
            Some(live) => live.started_at,
            None => self.store.confirmed_run(item.run_id)?.unwrap_or(0),
        };
        let error = item
            .error
            .clone()
            .or_else(|| live.as_ref().and_then(|l| l.error.clone()));
        let retry = if item.step == PublishStep::Failed {
            Some("retry_available".to_string())
        } else if running && error.is_some() && live.as_ref().is_some_and(|l| l.retried) {
            Some("retrying".to_string())
        } else {
            None
        };
        Ok(Some(PublishActivity {
            item_id: item.id,
            step: item.step,
            text,
            percent: progress_percent(item.step, item.uploaded, item.total),
            elapsed_ms: (self.clock.now_ms() - started_at).max(0),
            error,
            retry,
            can_cancel: running && !live.as_ref().is_some_and(|l| l.cancel),
        }))
    }
    /// Ends a cancelled item: step `cancelled` (not an error, not published, 07 / changelog 44).
    fn finish_cancel(&self, item: &PublishItem) -> Result<()> {
        if item.step == PublishStep::Prepare {
            // Changelog 45 (P65): an export still rendering (or waiting for Capture One) for this
            // publish is ended with it.
            let known = self
                .exports
                .lock()
                .map_err(|e| Error::InvalidData(e.to_string()))?
                .remove(&item.id);
            for export in known.into_iter().chain(item.export_run_id) {
                if self.export.has_run(export) {
                    self.export.cancel_run(export)?;
                }
            }
        }
        self.store.set_publish_step(
            item.id,
            PublishStep::Cancelled,
            None,
            None,
            self.clock.now_ms(),
        )?;
        self.with_live(item.id, |live| {
            live.cancel = false;
            live.text = "Publish cancelled".into();
        });
        self.emit(item, PublishStep::Cancelled);
        Ok(())
    }

    fn selected(&self, session_id: i64, ids: &[i64]) -> Result<Vec<i64>> {
        let mut selected = Vec::new();
        for id in ids {
            let photo = self.store.photo(*id)?;
            if photo.session_id == session_id && photo.classification != Classification::Deleted {
                selected.push(*id);
            }
        }
        Ok(selected)
    }
    /// The outputs of the main then condition photos, in order, by their current composition
    /// kind (10D MC-29 §Outputs; names as the export stages them).
    fn outputs(&self, key: &str, main: &[i64], condition: &[i64]) -> Result<Vec<Output>> {
        let mut out = Vec::new();
        for (group, ids) in [
            (ExportGroup::Main, main),
            (ExportGroup::Condition, condition),
        ] {
            let mut kinds = Vec::new();
            for &id in ids {
                kinds.push((id, self.store.composition(id)?.kind.slots()));
            }
            out.extend(output_names(key, group, &kinds).into_iter().map(
                |(photo_id, slot, name)| Output {
                    group,
                    photo_id,
                    slot,
                    name,
                },
            ));
        }
        Ok(out)
    }
    /// 10D MC-27 §Revisions: the labels (`Main 02`, `Condition 01`) of the photos that need
    /// re-framing, in display order.
    fn needs_review(&self, main: &[i64], condition: &[i64]) -> Result<Vec<String>> {
        let mut flagged = Vec::new();
        for (group, ids) in [
            (ExportGroup::Main, main),
            (ExportGroup::Condition, condition),
        ] {
            for (index, &id) in ids.iter().enumerate() {
                if self.store.composition(id)?.state == CompositionState::NeedsReview {
                    flagged.push(review_label(group, index + 1, ids.len()));
                }
            }
        }
        Ok(flagged)
    }
    /// MC-28 §Cache key: a staging folder is reused only when its `manifest.json` holds exactly
    /// the current outputs, each rendered from the RAW's current `adjusted_gen` and the photo's
    /// current composition revision with the current output format.
    fn staging_current(
        &self,
        staging: &Path,
        key: &str,
        main: &[i64],
        condition: &[i64],
    ) -> Result<bool> {
        if !staging.join("item.json").exists() {
            return Ok(false);
        }
        let Some((format_key, entries)) = read_manifest(staging) else {
            return Ok(false);
        };
        if format_key != self.export.format().format_key() {
            return Ok(false);
        }
        let outputs = self.outputs(key, main, condition)?;
        if outputs.len() != entries.len() {
            return Ok(false);
        }
        for output in outputs {
            let Some(entry) = entries.get(&output.name) else {
                return Ok(false);
            };
            let raw = self.store.photo(output.photo_id)?.raw_path;
            if entry.photo_id != output.photo_id
                || entry.slot != slot_name(output.slot)
                || entry.adjusted_gen != self.store.adjusted_gen(&raw)?
                || entry.composition_rev != self.store.composition(output.photo_id)?.revision
                || !staging.join(&output.name).is_file()
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
    /// The session's frozen-main candidates (changelog 44: main then unreviewed, each in the order
    /// the workspace shows them), its condition photos in position order, and how many unreviewed photos are included.
    fn split(&self, session_id: i64) -> Result<(Vec<i64>, Vec<i64>, usize)> {
        let photos = self.store.photos_in_session(session_id)?;
        let classes: Vec<_> = photos
            .iter()
            .map(|p| (p.classification, p.position, p.captured_at, p.id))
            .collect();
        let mut condition: Vec<(i64, i64)> = photos
            .iter()
            .filter(|p| p.classification == Classification::Condition)
            .map(|p| (p.position.unwrap_or(0), p.id))
            .collect();
        condition.sort();
        let unreviewed = photos
            .iter()
            .filter(|p| p.classification == Classification::Unreviewed)
            .count();
        Ok((
            frozen_main(&classes),
            condition.into_iter().map(|(_, id)| id).collect(),
            unreviewed,
        ))
    }
    fn frozen_attributes(&self, item_id: i64) -> Result<BTreeMap<String, serde_json::Value>> {
        let mut out = BTreeMap::new();
        self.store.transaction(&mut |tx| {
            for value in tx.attribute_values(item_id)? {
                if value.valid
                    && value.is_live()
                    && let Some(json) = value.json()
                {
                    out.insert(value.key, json);
                }
            }
            Ok(())
        })?;
        Ok(out)
    }
    fn condition_key(&self) -> Result<Option<String>> {
        let mut key = None;
        self.store.transaction(&mut |tx| {
            key = tx
                .attribute_definitions("shopify")?
                .into_iter()
                .find(|d| {
                    d.core_type == CoreType::MediaList
                        && d.key == self.commerce.condition_photos_key()
                })
                .map(|d| d.key);
            Ok(())
        })?;
        Ok(key)
    }
    fn filtered_condition(&self, item_id: i64, ids: &[i64]) -> Result<Vec<i64>> {
        let key = self.commerce.condition_photos_key();
        let attributes = self.frozen_attributes(item_id)?;
        let excluded: HashSet<i64> = attributes
            .get(key)
            .and_then(|v| v.get("excluded"))
            .and_then(|v| serde_json::from_value::<Vec<i64>>(v.clone()).ok())
            .unwrap_or_default()
            .into_iter()
            .collect();
        Ok(ids
            .iter()
            .copied()
            .filter(|id| !excluded.contains(id))
            .collect())
    }
    pub fn preview(&self, session_ids: &[i64]) -> Result<PublishPreview> {
        let mut items = Vec::new();
        for &session_id in session_ids {
            let session = self.store.session(session_id)?;
            let item_id = session
                .session
                .item_id
                .ok_or_else(|| Error::InvalidValue("unassigned cannot publish".into()))?;
            let (main, condition, unreviewed) = self.split(session_id)?;
            let condition = self.filtered_condition(item_id, &condition)?;
            let link = self.store.item_link(item_id, "shopify")?;
            let mut announced_removals = Vec::new();
            if let Some(link) = &link {
                let own: HashSet<String> = self
                    .store
                    .photo_links_for_item(item_id)?
                    .into_iter()
                    .filter(|p| p.role == "gallery")
                    .flat_map(|p| p.external_id.into_iter().chain(p.superseded_ids))
                    .collect();
                announced_removals = self
                    .commerce
                    .item_get(&link.external_id)?
                    .media
                    .into_iter()
                    .filter(|m| !own.contains(&m.id))
                    .map(|m| m.id)
                    .collect();
            }
            let mut main_thumbnails = Vec::new();
            let mut condition_thumbnails = Vec::new();
            for id in &main {
                main_thumbnails.push(self.store.photo(*id)?.adjusted_preview_path);
            }
            for id in &condition {
                condition_thumbnails.push(self.store.photo(*id)?.adjusted_preview_path);
            }
            let mut definitions = Vec::new();
            self.store.transaction(&mut |tx| {
                definitions = tx.attribute_definitions("shopify")?;
                Ok(())
            })?;
            let attributes = self.frozen_attributes(item_id)?;
            let details_total = definitions
                .iter()
                .filter(|d| d.group == crate::domain::attribute::AttributeGroup::Details)
                .count();
            let details_filled = definitions
                .iter()
                .filter(|d| {
                    d.group == crate::domain::attribute::AttributeGroup::Details
                        && attributes.contains_key(&d.key)
                })
                .count();
            items.push(PublishPreviewItem {
                session_id,
                main,
                condition,
                main_thumbnails,
                condition_thumbnails,
                details_filled,
                details_total,
                unreviewed,
                announced_removals,
                create_item: link.is_none(),
            });
        }
        let mut preview = PublishPreview {
            items,
            summary: String::new(),
        };
        preview.summary = publish_summary(&preview);
        Ok(preview)
    }
    pub fn confirm(&self, session_ids: &[i64], summary_text: &str) -> Result<i64> {
        // 10F Q3: a cleared item is never republished (its photos would need a new session).
        for session_id in session_ids {
            crate::services::retention::ensure_session_not_cleared(
                self.store.as_ref(),
                *session_id,
            )?;
        }
        let preview = self.preview(session_ids)?;
        // 10D MC-27 §Revisions: refused, naming the photos, while any of them needs re-framing.
        let mut flagged = Vec::new();
        for p in &preview.items {
            flagged.extend(self.needs_review(&p.main, &p.condition)?);
        }
        if !flagged.is_empty() {
            return Err(Error::NeedsReview { photos: flagged });
        }
        let mut totals = HashMap::new();
        for p in &preview.items {
            let mut total = 0;
            for id in p.main.iter().chain(&p.condition) {
                total += self.store.composition(*id)?.kind.slots().len() as i64;
            }
            totals.insert(p.session_id, total);
        }
        let condition_key = self.condition_key()?;
        // Changelog 45 (P65): the stock a new item is created with, frozen with the confirmation.
        let mut stocks = HashMap::new();
        for p in &preview.items {
            if !p.create_item {
                continue;
            }
            if let Some(item_id) = self.store.session(p.session_id)?.session.item_id
                && let Some(choice) = self.item_stock(item_id)?
            {
                stocks.insert(p.session_id, choice);
            }
        }
        let at = self.clock.now_ms();
        let mut run_id = 0;
        let mut item_ids = Vec::new();
        self.store.transaction(&mut |tx| {
            run_id = tx.insert_publish_run("shopify", at, summary_text)?;
            for p in &preview.items {
                let session = tx.session(p.session_id)?;
                let item = tx.item(
                    session
                        .item_id
                        .ok_or_else(|| Error::InvalidValue("unassigned".into()))?,
                )?;
                let mut attrs = BTreeMap::new();
                for v in tx.attribute_values(item.id)? {
                    if v.valid
                        && v.is_live()
                        && let Some(json) = v.json()
                    {
                        attrs.insert(v.key, json);
                    }
                }
                let photos = tx.photos(p.session_id)?;
                let classes: Vec<_> = photos
                    .iter()
                    .map(|photo| {
                        (
                            photo.classification,
                            photo.position,
                            photo.captured_at,
                            photo.id,
                        )
                    })
                    .collect();
                let current_main = frozen_main(&classes);
                let mut current_condition: Vec<(i64, i64)> = photos
                    .iter()
                    .filter(|photo| photo.classification == Classification::Condition)
                    .map(|photo| (photo.position.unwrap_or(0), photo.id))
                    .collect();
                current_condition.sort();
                let excluded: HashSet<i64> = condition_key
                    .as_ref()
                    .and_then(|key| {
                        attrs
                            .get(key)
                            .and_then(|v| v.get("excluded"))
                            .and_then(|v| serde_json::from_value::<Vec<i64>>(v.clone()).ok())
                    })
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
                let current_condition: Vec<i64> = current_condition
                    .into_iter()
                    .map(|(_, id)| id)
                    .filter(|id| !excluded.contains(id))
                    .collect();
                if current_main != p.main || current_condition != p.condition {
                    return Err(Error::InvalidValue(
                        "publish preview changed; review again".into(),
                    ));
                }
                let row = PublishItem {
                    id: 0,
                    run_id,
                    session_id: p.session_id,
                    create_item: p.create_item,
                    title: item.title,
                    frozen_item_type: item.item_type,
                    frozen_main: p.main.clone(),
                    frozen_condition: p.condition.clone(),
                    frozen_attributes: attrs,
                    announced_removals: p.announced_removals.clone(),
                    confirmed_change_seq: session.change_seq,
                    step: PublishStep::Prepare,
                    failed_step: None,
                    uploaded: 0,
                    total: totals.get(&p.session_id).copied().unwrap_or(0),
                    error: None,
                    export_run_id: None,
                };
                item_ids.push((tx.insert_publish_item(&row)?, p.session_id));
            }
            Ok(())
        })?;
        for (id, session_id) in item_ids {
            if let Some(choice) = stocks.get(&session_id) {
                self.store.set_publish_item_stock(id, choice)?;
            }
            self.store.enqueue_publish_item(id, at)?;
        }
        Ok(run_id)
    }
    pub fn retry(&self, item_id: i64) -> Result<()> {
        let item = self.store.publish_item(item_id)?;
        if item.step != PublishStep::Failed {
            return Err(Error::InvalidValue("publish item is not failed".into()));
        }
        let step = item
            .failed_step
            .ok_or_else(|| Error::InvalidData("missing failed step".into()))?;
        let now = self.clock.now_ms();
        self.with_live(item_id, |live| {
            live.started_at = now;
            live.text.clear();
            live.error = item.error.clone();
            live.retried = true;
            live.cancel = false;
        });
        self.store
            .set_publish_step(item_id, step, None, None, self.clock.now_ms())?;
        self.store
            .enqueue_publish_item(item_id, self.clock.now_ms())
    }
    pub fn notifications(&self, session_id: Option<i64>) -> Result<Vec<Notification>> {
        self.store.notifications(session_id)
    }

    fn emit(&self, item: &PublishItem, step: PublishStep) {
        self.events.emit(AppEvent::PublishProgress {
            session_id: item.session_id,
            item_id: item.id,
            step,
            uploaded: item.uploaded,
            total: item.total,
        });
    }
    pub fn run_item(&self, item_id: i64) -> Result<()> {
        let item = self.store.publish_item(item_id)?;
        if !active(item.step) {
            return Ok(());
        }
        if self.cancel_requested(item.id) {
            return self.finish_cancel(&item);
        }
        match item.step {
            PublishStep::Prepare => self.set_text(item.id, "Preparing outputs"),
            PublishStep::Update => self.set_text(item.id, "Updating the product"),
            PublishStep::Describe => self.set_text(item.id, "Writing the description"),
            _ => self.with_live(item.id, |_| ()),
        }
        if item.step == PublishStep::Prepare {
            self.emit(&item, PublishStep::Prepare);
        }
        if item.step == PublishStep::Upload && !self.upload_staging_current(&item)? {
            // M35: a staging folder from an earlier attempt that no longer matches the photos'
            // current outputs is rendered again (prepare) before anything is uploaded.
            let now = self.clock.now_ms();
            self.store
                .set_publish_step(item.id, PublishStep::Prepare, None, None, now)?;
            self.store.enqueue_publish_item(item.id, now)?;
            return Ok(());
        }
        let token = ConfirmedRun::from_persisted(self.store.as_ref(), item.run_id)?;
        let outcome = match item.step {
            PublishStep::Prepare => self.prepare(&item, &token),
            PublishStep::Upload => self.upload(&item, &token),
            PublishStep::Update => self.update(&item, &token),
            PublishStep::Describe => Ok(true),
            _ => Ok(true),
        };
        let next_is_done = matches!(item.step, PublishStep::Update | PublishStep::Describe);
        if self.cancel_requested(item.id) && !(next_is_done && matches!(outcome, Ok(true))) {
            // The step stopped at a safe point (or finished one); a cancel ends the item here.
            return self.finish_cancel(&item);
        }
        match outcome {
            Ok(false) => Err(Error::Timeout),
            Ok(true) => {
                let next = match item.step {
                    PublishStep::Prepare => PublishStep::Upload,
                    PublishStep::Upload => PublishStep::Update,
                    PublishStep::Update => PublishStep::Done,
                    _ => PublishStep::Done,
                };
                if next == PublishStep::Done {
                    // The staging export is swapped only after the confirmed run reaches done.
                    if let Some(export_id) = item.export_run_id {
                        let local = self.store.item(
                            self.store
                                .session(item.session_id)?
                                .session
                                .item_id
                                .unwrap(),
                        )?;
                        let key = folder_key(&local.sku)?;
                        let staging = self.export.staging_folder(&key, export_id);
                        if staging.exists()
                            && let Err(error) = self.export.publish_swap(&staging, &key, export_id)
                        {
                            self.store.set_publish_step(
                                item.id,
                                PublishStep::Failed,
                                Some(item.step),
                                Some(&error.to_string()),
                                self.clock.now_ms(),
                            )?;
                            self.emit(&item, PublishStep::Failed);
                            return Ok(());
                        }
                    }
                    let mut text = String::from("Published");
                    if self.condition_key()?.is_none() && !item.frozen_condition.is_empty() {
                        text.push_str(&format!(
                            ". Condition photos not published: field {} not found in the shop",
                            self.commerce.condition_photos_key()
                        ));
                    }
                    self.store
                        .complete_publish_item(item.id, self.clock.now_ms(), &text)?;
                    self.with_live(item.id, |live| {
                        live.error = None;
                        live.retried = false;
                        live.text.clear();
                    });
                    // MC-28 §Master: the session's masters are released once its publish
                    // completes and the session is not dirty.
                    if !self.store.session(item.session_id)?.dirty
                        && let Err(error) = self.export.release_masters(item.session_id)
                    {
                        tracing::warn!(%error, "masters not released after publish");
                    }
                } else {
                    self.store
                        .set_publish_step(item.id, next, None, None, self.clock.now_ms())?;
                    self.store
                        .enqueue_publish_item(item.id, self.clock.now_ms())?;
                }
                self.emit(&item, next);
                Ok(())
            }
            Err(error) => {
                let text = error.to_string();
                self.with_live(item.id, |live| live.error = Some(text.clone()));
                self.store.set_publish_step(
                    item.id,
                    PublishStep::Failed,
                    Some(item.step),
                    Some(&text),
                    self.clock.now_ms(),
                )?;
                self.emit(&item, PublishStep::Failed);
                Ok(())
            }
        }
    }
    /// Creates the platform item: with the frozen stock when the confirmation carried one
    /// (changelog 45), whose values then become the remembered default.
    fn create_item(
        &self,
        run: &ConfirmedRun,
        item: &PublishItem,
        local: &crate::domain::item::Item,
    ) -> Result<String> {
        let Some(stock) = self.store.publish_item_stock(item.id)? else {
            return self.commerce.item_create(
                run,
                &local.sku,
                local.barcode.as_deref(),
                &item.title,
                &item.frozen_item_type,
            );
        };
        let (remote, outcome) = self.commerce.item_create_stocked(
            run,
            &local.sku,
            local.barcode.as_deref(),
            &item.title,
            &item.frozen_item_type,
            &stock,
        )?;
        match &outcome {
            StockOutcome::Tracked { verified } => tracing::info!(
                location = stock.location_name,
                quantity = stock.quantity,
                verified,
                "item created with stock"
            ),
            StockOutcome::Untracked { reason } => {
                tracing::warn!(reason, "item created without stock tracking")
            }
        }
        if let Some(defaults) = self.defaults()
            && let Err(error) = defaults.remember(&stock)
        {
            tracing::warn!(%error, "stock default not remembered");
        }
        Ok(remote)
    }
    /// Whether the staging folder the upload step would read still holds the current outputs.
    fn upload_staging_current(&self, item: &PublishItem) -> Result<bool> {
        let Some(export_id) = item.export_run_id else {
            return Ok(true);
        };
        let session = self.store.session(item.session_id)?;
        let Some(item_id) = session.session.item_id else {
            return Ok(true);
        };
        let key = folder_key(&self.store.item(item_id)?.sku)?;
        let main = self.selected(item.session_id, &item.frozen_main)?;
        let condition = self.selected(item.session_id, &item.frozen_condition)?;
        self.staging_current(
            &self.export.staging_folder(&key, export_id),
            &key,
            &main,
            &condition,
        )
    }
    fn prepare(&self, item: &PublishItem, run: &ConfirmedRun) -> Result<bool> {
        let local = self.store.item(
            self.store
                .session(item.session_id)?
                .session
                .item_id
                .unwrap(),
        )?;
        let main = self.selected(item.session_id, &item.frozen_main)?;
        let condition = self.selected(item.session_id, &item.frozen_condition)?;
        // 10D MC-27 §Revisions: a photo flagged after confirmation fails the item here, before
        // any remote write; Retry re-runs prepare.
        let flagged = self.needs_review(&main, &condition)?;
        if !flagged.is_empty() {
            return Err(Error::NeedsReview { photos: flagged });
        }
        if self.store.item_link(local.id, "shopify")?.is_none() {
            let matches = self.commerce.item_search(&local.sku)?;
            let remote = match matches.len() {
                0 => self.create_item(run, item, &local)?,
                1 => matches[0].external_id.clone(),
                _ => return Err(Error::SeveralProductsCarrySku { sku: local.sku }),
            };
            self.store
                .transaction(&mut |tx| tx.insert_item_link(local.id, "shopify", &remote, None))?;
        }
        for id in main.iter().chain(&condition) {
            match IntegrityCheck::new(self.store.as_ref()).verify(*id)? {
                Integrity::Ok => {}
                Integrity::RawChanged => return Err(Error::RawChanged { photo_id: *id }),
                Integrity::RawMissing => return Err(Error::RawMissing { photo_id: *id }),
            }
        }
        let key = folder_key(&local.sku)?;
        let export_id = {
            let mut exports = self
                .exports
                .lock()
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            let known = exports.get(&item.id).copied();
            if let Some(id) = known.filter(|id| self.export.has_run(*id)) {
                // The export of this attempt is still rendering.
                id
            } else if let Some(id) =
                known.filter(|id| self.store.job(*id).is_ok_and(|job| job.status == "failed"))
            {
                exports.remove(&item.id);
                let reason = self.store.job(id)?.error.unwrap_or_default();
                // Changelog 45 (P65): waited 10 minutes for Capture One: the sentence as is.
                if let Some(sentence) = reason.strip_prefix(EDITOR_WAIT_EXPIRED) {
                    return Err(Error::EditorWaitExpired(sentence.to_string()));
                }
                return Err(Error::Provider(format!("export failed: {reason}")));
            } else if let Some(id) = known.or(item.export_run_id).filter(|id| {
                self.staging_current(
                    &self.export.staging_folder(&key, *id),
                    &key,
                    &main,
                    &condition,
                )
                .unwrap_or(false)
            }) {
                exports.insert(item.id, id);
                id
            } else {
                for old in known.into_iter().chain(item.export_run_id) {
                    let _ = self
                        .export
                        .discard_staging(&self.export.staging_folder(&key, old));
                }
                let id = self
                    .export
                    .run_staged(item.session_id, &main, &condition, &item.frozen_attributes)?
                    .0;
                self.store.set_publish_export(item.id, id)?;
                exports.insert(item.id, id);
                id
            }
        };
        if self.store.job(export_id)?.status != "done" {
            return Ok(false);
        }
        let staging = self.export.staging_folder(&key, export_id);
        let manifest = read_manifest(&staging)
            .map(|(_, entries)| entries)
            .unwrap_or_default();
        let condition_role = format!("attribute:{}", self.condition_key()?.unwrap_or_default());
        let now = self.clock.now_ms();
        for (ids, role) in [(&main, "gallery"), (&condition, condition_role.as_str())] {
            for id in ids.iter() {
                // 10D MC-29 §Links: a slot the photo no longer has is retired (its id stays own).
                let slots = self.store.composition(*id)?.kind.slots();
                self.store
                    .retire_photo_links(local.id, *id, role, slots, now)?;
            }
        }
        for output in self.outputs(&key, &main, &condition)? {
            let role = match output.group {
                ExportGroup::Main => "gallery",
                ExportGroup::Condition => condition_role.as_str(),
            };
            let id = output.photo_id;
            let hash = sha256_file(&staging.join(&output.name))?;
            let composition_rev = manifest
                .get(&output.name)
                .map(|entry| entry.composition_rev)
                .ok_or(Error::UploadIncomplete)?;
            let export_gen = self.store.export_retouch_gen(export_id, id)?.unwrap_or(0);
            let mut link = self
                .store
                .photo_link(id, local.id, role, output.slot)?
                .unwrap_or(PhotoLink {
                    photo_id: id,
                    item_id: local.id,
                    role: role.to_string(),
                    slot: output.slot,
                    external_id: None,
                    upload_state: "none".into(),
                    export_sha256: None,
                    superseded_ids: Vec::new(),
                    revision: 0,
                    retouch_gen: export_gen,
                    composition_rev,
                });
            if link.upload_state == "retired" {
                // A slot the photo has again (Normal → Split → Normal) is uploaded anew under its
                // revision; the retired media id stays own as a superseded id, never adopted.
                if let Some(old_id) = link.external_id.take() {
                    link.superseded_ids.push(old_id);
                }
                link.upload_state = "none".into();
            }
            let replaced = replacement_candidates(
                &[link.clone()],
                &HashMap::from([(id, export_gen)]),
                &HashMap::from([(id, composition_rev)]),
            )
            .contains(&id)
                // A link whose remote media may already exist from an earlier staging (staged,
                // uploaded, attached) and whose recorded generation or revision is older than
                // the staged file's holds a stale rendering: replace it like a ready link, so
                // the stale media is never adopted and reconciliation removes it.
                || (matches!(link.upload_state.as_str(), "staged" | "uploaded" | "attached")
                    && (link.composition_rev < composition_rev || link.retouch_gen < export_gen));
            if replaced {
                if let Some(old_id) = link.external_id.take() {
                    link.superseded_ids.push(old_id);
                }
                link.revision += 1;
                link.upload_state = "none".into();
            }
            if link.upload_state != "ready" {
                // What this link will upload: the staged file's generation and revision.
                link.retouch_gen = export_gen;
                link.composition_rev = composition_rev;
            }
            link.export_sha256 = Some(hash);
            if link.upload_state == "none" && !replaced {
                link.upload_state = "exported".into();
            }
            self.store.upsert_photo_link(&link, now)?;
        }
        Ok(true)
    }
    /// 07A MC-11 upload (P64): per group (main, then condition), the links not yet `ready` are
    /// adopted when the platform already holds their file (by name), polled when attached, and
    /// otherwise reserved together, uploaded in waves of `PARALLEL_UPLOADS` in parallel, each
    /// wave registered in one request and polled to ready. Every remote effect is recorded before
    /// the next request; a cancel stops before the next upload or mutation (changelog 44).
    fn upload(&self, item: &PublishItem, run: &ConfirmedRun) -> Result<bool> {
        let local = self.store.item(
            self.store
                .session(item.session_id)?
                .session
                .item_id
                .unwrap(),
        )?;
        let remote = self
            .store
            .item_link(local.id, "shopify")?
            .ok_or(Error::UploadIncomplete)?
            .external_id;
        let key = folder_key(&local.sku)?;
        let export_id = item.export_run_id.ok_or(Error::UploadIncomplete)?;
        let staging = self.export.staging_folder(&key, export_id);
        let main = self.selected(item.session_id, &item.frozen_main)?;
        let condition = self.selected(item.session_id, &item.frozen_condition)?;
        let outputs = self.outputs(&key, &main, &condition)?;
        let total = outputs.len() as i64;
        let condition_key = self.condition_key()?;
        let condition_role = format!("attribute:{}", condition_key.clone().unwrap_or_default());
        let mut uploaded = 0;
        // 10D MC-29: one upload per output, main then condition, a split's Left then Right;
        // retired links are not outputs, so they are never uploaded or counted.
        for group in [ExportGroup::Main, ExportGroup::Condition] {
            if group == ExportGroup::Condition && condition_key.is_none() {
                continue;
            }
            let (role, words) = match group {
                ExportGroup::Main => ("gallery", "main"),
                ExportGroup::Condition => (condition_role.as_str(), "condition"),
            };
            let group_outputs: Vec<&Output> = outputs.iter().filter(|o| o.group == group).collect();
            let group_total = group_outputs.len();
            let mut done = 0;
            let mut work: Vec<(&Output, PhotoLink, String)> = Vec::new();
            for output in group_outputs {
                // Safety recheck immediately before any remote write.
                let photo = self.store.photo(output.photo_id)?;
                if photo.session_id != item.session_id
                    || photo.classification == Classification::Deleted
                {
                    continue;
                }
                let link = self
                    .store
                    .photo_link(output.photo_id, local.id, role, output.slot)?
                    .ok_or(Error::UploadIncomplete)?;
                if link.upload_state == "ready" {
                    uploaded += 1;
                    done += 1;
                    continue;
                }
                let stem =
                    upload_name::stem(&local.folder_key, &photo.uid, output.slot, link.revision);
                work.push((output, link, stem));
            }
            if work.is_empty() {
                continue;
            }
            if self.cancel_requested(item.id) {
                return Ok(false);
            }
            // MC-11 adoption: a file the platform already holds under the link's upload name
            // (created before a crash recorded it) is adopted, never uploaded again.
            self.set_text(item.id, "Checking Shopify");
            let gallery = if group == ExportGroup::Main {
                let media = self.commerce.list_media(&remote)?;
                if media.iter().any(Self::unnamed_processing) {
                    return Ok(false);
                }
                media
            } else {
                Vec::new()
            };
            for (_, link, stem) in work.iter_mut() {
                if link.external_id.is_some() {
                    continue;
                }
                let files;
                let candidates = if group == ExportGroup::Main {
                    &gallery
                } else if matches!(link.upload_state.as_str(), "staged" | "uploaded") {
                    // P64: only a link whose upload reached the platform can have a file there;
                    // it is looked up by name (never by listing the shop's files).
                    files = self.commerce.list_files_by_name(stem)?;
                    if files.iter().any(Self::unnamed_processing) {
                        return Ok(false);
                    }
                    &files
                } else {
                    continue;
                };
                if let Some(found) = candidates.iter().find(|m| {
                    !link.superseded_ids.contains(&m.id)
                        && m.filename
                            .as_deref()
                            .is_some_and(|name| upload_name::matches(stem, name))
                }) {
                    link.external_id = Some(found.id.clone());
                    link.upload_state = "attached".into();
                    self.store.upsert_photo_link(link, self.clock.now_ms())?;
                }
            }
            let progress_text =
                |done: usize| format!("Uploading {words} photos {done} of {group_total}");
            // Attached before this attempt (or adopted above): poll to ready.
            for (_, link, _) in work.iter_mut() {
                if link.external_id.is_none() {
                    continue;
                }
                if self.cancel_requested(item.id) {
                    return Ok(false);
                }
                self.set_text(item.id, progress_text(done));
                self.mark_ready(item, link, group, &mut uploaded, total)?;
                done += 1;
            }
            let pending: Vec<usize> = (0..work.len())
                .filter(|&i| work[i].1.external_id.is_none())
                .collect();
            if pending.is_empty() {
                continue;
            }
            if self.cancel_requested(item.id) {
                return Ok(false);
            }
            self.set_text(item.id, progress_text(done));
            let mut files = Vec::with_capacity(pending.len());
            for &i in &pending {
                let size = std::fs::metadata(staging.join(&work[i].0.name))?.len();
                files.push((work[i].2.clone(), size));
            }
            let targets = self.commerce.staged_uploads_create_many(run, &files)?;
            if targets.len() != pending.len() {
                return Err(Error::Provider(format!(
                    "{} upload targets reserved for {} files",
                    targets.len(),
                    pending.len()
                )));
            }
            for &i in &pending {
                work[i].1.upload_state = "staged".into();
                self.store
                    .upsert_photo_link(&work[i].1, self.clock.now_ms())?;
            }
            for wave in upload_waves(pending.len()) {
                if self.cancel_requested(item.id) {
                    return Ok(false);
                }
                let results: Vec<Result<()>> = std::thread::scope(|scope| {
                    let handles: Vec<_> = wave
                        .clone()
                        .map(|k| {
                            let path = staging.join(&work[pending[k]].0.name);
                            let target = &targets[k];
                            let commerce = &self.commerce;
                            scope.spawn(move || -> Result<()> {
                                let bytes = std::fs::read(path)?;
                                commerce.upload(run, target, &bytes)
                            })
                        })
                        .collect();
                    handles
                        .into_iter()
                        .map(|h| h.join().unwrap_or_else(|p| std::panic::resume_unwind(p)))
                        .collect()
                });
                let mut sent = Vec::new();
                let mut failure = None;
                for (k, result) in wave.clone().zip(results) {
                    match result {
                        Ok(()) => {
                            let link = &mut work[pending[k]].1;
                            link.upload_state = "uploaded".into();
                            self.store.upsert_photo_link(link, self.clock.now_ms())?;
                            sent.push(k);
                        }
                        Err(error) => {
                            failure.get_or_insert(error);
                        }
                    }
                }
                // One registration request for the wave's uploaded files; none after a cancel
                // (the next publish uploads them again; nothing exists on the platform yet).
                if !sent.is_empty() && !self.cancel_requested(item.id) {
                    let ids = if group == ExportGroup::Main {
                        let urls: Vec<String> = sent
                            .iter()
                            .map(|&k| targets[k].resource_url.clone())
                            .collect();
                        self.commerce.product_create_media_many(
                            run,
                            &remote,
                            &urls,
                            &local.title,
                        )?
                    } else {
                        let files: Vec<(String, String)> = sent
                            .iter()
                            .map(|&k| {
                                (targets[k].resource_url.clone(), targets[k].filename.clone())
                            })
                            .collect();
                        self.commerce.file_create_many(run, &files)?
                    };
                    if ids.len() != sent.len() {
                        return Err(Error::Provider(format!(
                            "{} files registered for {} uploads",
                            ids.len(),
                            sent.len()
                        )));
                    }
                    for (&k, id) in sent.iter().zip(ids) {
                        let link = &mut work[pending[k]].1;
                        link.external_id = Some(id);
                        link.upload_state = "attached".into();
                        self.store.upsert_photo_link(link, self.clock.now_ms())?;
                    }
                }
                if let Some(error) = failure {
                    return Err(error);
                }
                for &k in &sent {
                    if self.cancel_requested(item.id) {
                        return Ok(false);
                    }
                    let link = &mut work[pending[k]].1;
                    self.mark_ready(item, link, group, &mut uploaded, total)?;
                    done += 1;
                    self.set_text(item.id, progress_text(done));
                }
            }
        }
        Ok(true)
    }
    /// A platform file still processing whose name is not readable yet (SV-2): wait for it.
    fn unnamed_processing(m: &crate::domain::publish::RemoteMedia) -> bool {
        m.filename.is_none()
            && matches!(
                m.status,
                crate::domain::publish::RemoteMediaStatus::Processing
                    | crate::domain::publish::RemoteMediaStatus::Uploaded
            )
    }
    /// Polls an attached link's media or file to ready, records it and the item's progress.
    fn mark_ready(
        &self,
        item: &PublishItem,
        link: &mut PhotoLink,
        group: ExportGroup,
        uploaded: &mut i64,
        total: i64,
    ) -> Result<()> {
        let external = link.external_id.as_deref().ok_or(Error::UploadIncomplete)?;
        if group == ExportGroup::Main {
            self.commerce.poll_media_ready(external)?;
        } else {
            self.commerce.poll_file(external)?;
        }
        link.upload_state = "ready".into();
        self.store.upsert_photo_link(link, self.clock.now_ms())?;
        *uploaded += 1;
        self.store.set_publish_progress(item.id, *uploaded, total)?;
        self.events.emit(AppEvent::PublishProgress {
            session_id: item.session_id,
            item_id: item.id,
            step: PublishStep::Upload,
            uploaded: *uploaded,
            total,
        });
        Ok(())
    }
    fn update(&self, item: &PublishItem, run: &ConfirmedRun) -> Result<bool> {
        let local = self.store.item(
            self.store
                .session(item.session_id)?
                .session
                .item_id
                .unwrap(),
        )?;
        let external = self
            .store
            .item_link(local.id, "shopify")?
            .ok_or(Error::UploadIncomplete)?
            .external_id;
        // MC-12 `D`: one ready link per main output, in order (10D MC-29 §Gallery and cover).
        let mut desired = Vec::new();
        for id in self.selected(item.session_id, &item.frozen_main)? {
            for slot in self.store.composition(id)?.kind.slots() {
                let link = self
                    .store
                    .photo_link(id, local.id, "gallery", *slot)?
                    .ok_or(Error::UploadIncomplete)?;
                if link.upload_state != "ready" {
                    return Err(Error::UploadIncomplete);
                }
                desired.push(link.external_id.ok_or(Error::UploadIncomplete)?);
            }
        }
        let current_media = self.commerce.list_media(&external)?;
        let current: Vec<String> = current_media.iter().map(|m| m.id.clone()).collect();
        let mut own = HashSet::new();
        let mut stems = Vec::new();
        for link in self.store.photo_links_for_item(local.id)? {
            if link.role == "gallery" {
                // Retired links included: their media is own and is removed here.
                own.extend(link.external_id);
                own.extend(link.superseded_ids);
                let uid = self.store.photo(link.photo_id)?.uid;
                for revision in 0..=link.revision {
                    stems.push(upload_name::stem(
                        &local.folder_key,
                        &uid,
                        link.slot,
                        revision,
                    ));
                }
            }
        }
        for media in &current_media {
            if media
                .filename
                .as_deref()
                .is_some_and(|name| stems.iter().any(|stem| upload_name::matches(stem, name)))
            {
                own.insert(media.id.clone());
            }
        }
        let announced: HashSet<String> = item.announced_removals.iter().cloned().collect();
        let plan = reconcile::plan(&desired, &current, &own, &announced)
            .map_err(|reconcile::ReconcileError::Foreign(n)| Error::ForeignMediaAdded { n })?;
        if self.cancel_requested(item.id) {
            return Ok(false);
        }
        if !plan.deletes.is_empty() {
            self.commerce.product_delete_media(
                run,
                &external,
                &plan.deletes.into_iter().collect::<Vec<_>>(),
            )?;
        }
        if self.cancel_requested(item.id) {
            return Ok(false);
        }
        let job = self
            .commerce
            .product_reorder_media(run, &external, &desired)?;
        self.commerce.poll_job(&job)?;
        let readback: Vec<_> = self
            .commerce
            .list_media(&external)?
            .into_iter()
            .map(|m| m.id)
            .collect();
        if readback != desired {
            return Err(Error::GalleryVerification);
        }
        // MC-13: compare typed values, write only confirmed differences, then verify the same keys.
        let mut definitions = Vec::new();
        self.store.transaction(&mut |tx| {
            definitions = tx.attribute_definitions("shopify")?;
            Ok(())
        })?;
        let links: HashMap<_, _> = self
            .store
            .definition_links("shopify")?
            .into_iter()
            .collect();
        let remote_values = self.commerce.attributes_read(&external)?;
        let mut changes = write_set(&item.frozen_attributes, &definitions, &remote_values);
        if let Some(key) = self.condition_key()? {
            let selected = self.selected(item.session_id, &item.frozen_condition)?;
            if !selected.is_empty() || item.frozen_attributes.contains_key(&key) {
                // 10D MC-29 §Damage report: the selected condition outputs in order.
                let mut ids = Vec::new();
                for photo_id in selected {
                    for slot in self.store.composition(photo_id)?.kind.slots() {
                        let link = self
                            .store
                            .photo_link(photo_id, local.id, &format!("attribute:{key}"), *slot)?
                            .ok_or(Error::UploadIncomplete)?;
                        if link.upload_state != "ready" {
                            return Err(Error::UploadIncomplete);
                        }
                        ids.push(link.external_id.ok_or(Error::UploadIncomplete)?);
                    }
                }
                let value = serde_json::json!(ids);
                if !remote_values.iter().any(|(k, v)| k == &key && v == &value) {
                    changes.push((key, value));
                }
            }
        }
        let writes: Vec<_> = changes
            .iter()
            .filter_map(|(key, value)| {
                links
                    .get(key)
                    .map(|link| crate::domain::publish::WriteValue {
                        key: key.clone(),
                        link: link.clone(),
                        value: value.clone(),
                    })
            })
            .collect();
        if self.cancel_requested(item.id) {
            return Ok(false);
        }
        for batch in writes.chunks(25) {
            self.commerce.attributes_write(run, &external, batch)?;
        }
        if !writes.is_empty() {
            let readback: HashMap<_, _> = self
                .commerce
                .attributes_read(&external)?
                .into_iter()
                .collect();
            let condition_readback = if writes.iter().any(|write| {
                definitions.iter().any(|definition| {
                    definition.key == write.key && definition.core_type == CoreType::MediaList
                })
            }) {
                self.commerce.condition_photos_readback(&external)?
            } else {
                None
            };
            let failed: Vec<_> = writes
                .iter()
                .filter(|write| {
                    let actual = if definitions.iter().any(|definition| {
                        definition.key == write.key && definition.core_type == CoreType::MediaList
                    }) {
                        condition_readback.as_ref()
                    } else {
                        readback.get(&write.key)
                    };
                    let Some(actual) = actual else {
                        return true;
                    };
                    let Some(definition) = definitions.iter().find(|d| d.key == write.key) else {
                        return true;
                    };
                    if definition.core_type == CoreType::MediaList {
                        return actual != &write.value;
                    }
                    canonicalize(definition.core_type, definition, actual.clone()).ok()
                        != canonicalize(definition.core_type, definition, write.value.clone()).ok()
                })
                .map(|write| write.key.clone())
                .collect();
            if !failed.is_empty() {
                return Err(Error::AttributeVerification { keys: failed });
            }
        }
        let remote = self.commerce.item_get(&external)?;
        let type_base = self
            .store
            .item_link(local.id, "shopify")?
            .and_then(|link| link.item_type_base);
        let type_untouched = type_base
            .as_deref()
            .map_or(local.item_type.is_empty(), |base| base == local.item_type);
        let sync_remote_type = type_untouched
            && remote
                .item_type
                .as_deref()
                .is_some_and(|kind| kind != local.item_type);
        if sync_remote_type {
            let remote_type = remote.item_type.as_deref().unwrap();
            crate::services::items::ItemsService::new(self.store.as_ref(), self.clock.as_ref())
                .set_type(local.id, remote_type)?;
            self.store
                .set_item_type_base(local.id, "shopify", remote_type)?;
        }
        if self.cancel_requested(item.id) {
            return Ok(false);
        }
        if remote.title != item.title {
            self.commerce.update_title(run, &external, &item.title)?;
        }
        if !sync_remote_type
            && !item.frozen_item_type.is_empty()
            && remote.item_type.as_deref() != Some(item.frozen_item_type.as_str())
        {
            self.commerce
                .update_item_type(run, &external, &item.frozen_item_type)?;
            if self.commerce.item_get(&external)?.item_type.as_deref()
                != Some(item.frozen_item_type.as_str())
            {
                return Err(Error::InvalidData("item type read-back mismatch".into()));
            }
        }
        if !sync_remote_type && !item.frozen_item_type.is_empty() {
            self.store
                .set_item_type_base(local.id, "shopify", &item.frozen_item_type)?;
        }
        if remote.sku.as_deref() != Some(&local.sku)
            || local
                .barcode
                .as_ref()
                .is_some_and(|barcode| remote.barcode.as_ref() != Some(barcode))
        {
            let identity = self.commerce.item_set_identity(
                run,
                &external,
                &local.sku,
                local.barcode.as_deref(),
            )?;
            if identity.sku != local.sku
                || local
                    .barcode
                    .as_ref()
                    .is_some_and(|barcode| identity.barcode.as_ref() != Some(barcode))
            {
                return Err(Error::InvalidData("identity read-back mismatch".into()));
            }
        }
        let (remote_tags, remote_description) = self.commerce.product_fields(&external)?;
        if let Some(description) = item
            .frozen_attributes
            .get("product.description")
            .and_then(|v| v.as_str())
            && remote_description.as_deref()
                != Some(self.commerce.render_description(description).as_str())
        {
            self.set_text(item.id, "Writing the description");
            self.commerce
                .update_description(run, &external, description)?;
        }
        let local_tags = item
            .frozen_attributes
            .get("product.tags")
            .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok());
        if local_tags.is_some() {
            let link = self
                .store
                .item_link(local.id, "shopify")?
                .ok_or(Error::UploadIncomplete)?;
            let base: Vec<String> = link
                .tags_base
                .as_deref()
                .and_then(|text| serde_json::from_str(text).ok())
                .unwrap_or_default();
            let local_tags = local_tags.unwrap_or_default();
            if let Some(tags) = merge_tags(&base, &local_tags, &remote_tags) {
                self.commerce.update_tags(run, &external, &tags)?;
            }
            self.store
                .set_item_tags_base(local.id, "shopify", &local_tags)?;
        } else {
            let link = self
                .store
                .item_link(local.id, "shopify")?
                .ok_or(Error::UploadIncomplete)?;
            let base: Vec<String> = link
                .tags_base
                .as_deref()
                .and_then(|text| serde_json::from_str(text).ok())
                .unwrap_or_default();
            if !base.is_empty() {
                let local_tags = Vec::new();
                if let Some(tags) = merge_tags(&base, &local_tags, &remote_tags) {
                    self.commerce.update_tags(run, &external, &tags)?;
                }
                self.store
                    .set_item_tags_base(local.id, "shopify", &local_tags)?;
            }
        }
        Ok(true)
    }
}

/// A stock choice names a location and a quantity of 0 or more.
fn validate_stock(choice: &StockChoice) -> Result<()> {
    if choice.location_id.trim().is_empty() || choice.quantity < 0 {
        return Err(Error::InvalidValue("stock".into()));
    }
    Ok(())
}
