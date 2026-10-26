//! Confirmed publish runs over the commerce port (07A MC-10…MC-12).
use crate::{
    Error, Result,
    domain::{
        attribute::CoreType,
        photo::{Classification, Integrity},
        publish::{
            ConfirmedRun, Notification, PhotoLink, PublishItem, PublishPreview, PublishPreviewItem,
        },
        session::PublishStep,
    },
    events::AppEvent,
    mechanisms::{
        attribute_write::{merge_tags, write_set},
        hash::sha256_file,
        naming::{ExportGroup, export_file_name, folder_key},
        publish::{publish_summary, replacement_candidates},
        reconcile,
        values::canonicalize,
    },
    ports::{clock::Clock, commerce::CommerceAdapter, events::EventSink, store::Store},
    services::{export::ExportService, integrity::IntegrityCheck},
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{Arc, Mutex},
};

pub struct PublishService {
    store: Arc<dyn Store>,
    commerce: Arc<dyn CommerceAdapter>,
    export: Arc<ExportService>,
    clock: Arc<dyn Clock>,
    events: Arc<dyn EventSink>,
    exports: Mutex<HashMap<i64, i64>>,
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
        }
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
    fn split(&self, session_id: i64) -> Result<(Vec<i64>, Vec<i64>, usize)> {
        let mut main = Vec::new();
        let mut condition = Vec::new();
        let mut unreviewed = 0;
        for p in self.store.photos_in_session(session_id)? {
            match p.classification {
                Classification::Main => main.push((p.position.unwrap_or(0), p.id)),
                Classification::Condition => condition.push((p.position.unwrap_or(0), p.id)),
                Classification::Unreviewed => unreviewed += 1,
                Classification::Deleted => {}
            }
        }
        main.sort();
        condition.sort();
        Ok((
            main.into_iter().map(|(_, id)| id).collect(),
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
        let preview = self.preview(session_ids)?;
        let condition_key = self.condition_key()?;
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
                let mut current_main = Vec::new();
                let mut current_condition = Vec::new();
                for photo in tx.photos(p.session_id)? {
                    match photo.classification {
                        Classification::Main => current_main.push((photo.position, photo.id)),
                        Classification::Condition => {
                            current_condition.push((photo.position, photo.id))
                        }
                        _ => {}
                    }
                }
                current_main.sort();
                current_condition.sort();
                let current_main: Vec<i64> = current_main.into_iter().map(|(_, id)| id).collect();
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
                    frozen_main: p.main.clone(),
                    frozen_condition: p.condition.clone(),
                    frozen_attributes: attrs,
                    announced_removals: p.announced_removals.clone(),
                    confirmed_change_seq: session.change_seq,
                    step: PublishStep::Prepare,
                    failed_step: None,
                    uploaded: 0,
                    total: (p.main.len() + p.condition.len()) as i64,
                    error: None,
                    export_run_id: None,
                };
                item_ids.push(tx.insert_publish_item(&row)?);
            }
            Ok(())
        })?;
        for id in item_ids {
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
        if matches!(item.step, PublishStep::Done | PublishStep::Failed) {
            return Ok(());
        }
        if item.step == PublishStep::Prepare {
            self.emit(&item, PublishStep::Prepare);
        }
        let token = ConfirmedRun::from_persisted(self.store.as_ref(), item.run_id)?;
        let outcome = match item.step {
            PublishStep::Prepare => self.prepare(&item, &token),
            PublishStep::Upload => self.upload(&item, &token),
            PublishStep::Update => self.update(&item, &token),
            PublishStep::Describe => Ok(true),
            _ => Ok(true),
        };
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
                    let mut text = format!("Published · {}", self.clock.now_ms());
                    if self.condition_key()?.is_none() && !item.frozen_condition.is_empty() {
                        text.push_str(&format!(
                            ". Condition photos not published: field {} not found in the shop",
                            self.commerce.condition_photos_key()
                        ));
                    }
                    self.store
                        .complete_publish_item(item.id, self.clock.now_ms(), &text)?;
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
                self.store.set_publish_step(
                    item.id,
                    PublishStep::Failed,
                    Some(item.step),
                    Some(&error.to_string()),
                    self.clock.now_ms(),
                )?;
                self.emit(&item, PublishStep::Failed);
                Ok(())
            }
        }
    }
    fn prepare(&self, item: &PublishItem, run: &ConfirmedRun) -> Result<bool> {
        let local = self.store.item(
            self.store
                .session(item.session_id)?
                .session
                .item_id
                .unwrap(),
        )?;
        if self.store.item_link(local.id, "shopify")?.is_none() {
            let matches = self.commerce.item_search(&local.sku)?;
            let remote = match matches.len() {
                0 => self.commerce.item_create(
                    run,
                    &local.sku,
                    local.barcode.as_deref(),
                    &item.title,
                )?,
                1 => matches[0].external_id.clone(),
                _ => return Err(Error::SeveralProductsCarrySku { sku: local.sku }),
            };
            self.store
                .transaction(&mut |tx| tx.insert_item_link(local.id, "shopify", &remote, None))?;
        }
        let main = self.selected(item.session_id, &item.frozen_main)?;
        let condition = self.selected(item.session_id, &item.frozen_condition)?;
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
            if let Some(id) = exports.get(&item.id) {
                *id
            } else if let Some(id) = item.export_run_id.filter(|id| {
                let staging = self.export.staging_folder(&key, *id);
                staging.join("item.json").exists()
                    && [
                        (ExportGroup::Main, &main),
                        (ExportGroup::Condition, &condition),
                    ]
                    .into_iter()
                    .all(|(group, ids)| {
                        ids.iter().enumerate().all(|(index, _)| {
                            staging
                                .join(export_file_name(&key, index + 1, ids.len(), group))
                                .exists()
                        })
                    })
            }) {
                exports.insert(item.id, id);
                id
            } else {
                if let Some(old) = item.export_run_id {
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
        for (group, ids, role) in [
            (ExportGroup::Main, &main, "gallery".to_string()),
            (
                ExportGroup::Condition,
                &condition,
                format!("attribute:{}", self.condition_key()?.unwrap_or_default()),
            ),
        ] {
            for (index, id) in ids.iter().enumerate() {
                let file = staging.join(export_file_name(&key, index + 1, ids.len(), group));
                let hash = sha256_file(&file)?;
                let mut link = self
                    .store
                    .photo_link(*id, local.id, &role)?
                    .unwrap_or(PhotoLink {
                        photo_id: *id,
                        item_id: local.id,
                        role: role.clone(),
                        external_id: None,
                        upload_state: "none".into(),
                        export_sha256: None,
                        superseded_ids: Vec::new(),
                        revision: 0,
                        retouch_gen: self.store.export_retouch_gen(export_id, *id)?.unwrap_or(0),
                    });
                let export_gen = self.store.export_retouch_gen(export_id, *id)?.unwrap_or(0);
                let replaced =
                    replacement_candidates(&[link.clone()], &HashMap::from([(*id, export_gen)]))
                        .contains(id);
                if replaced {
                    if let Some(old_id) = link.external_id.take() {
                        link.superseded_ids.push(old_id);
                    }
                    link.revision += 1;
                    link.retouch_gen = export_gen;
                    link.upload_state = "none".into();
                }
                link.export_sha256 = Some(hash);
                if link.upload_state == "none" && !replaced {
                    link.upload_state = "exported".into();
                }
                self.store.upsert_photo_link(&link, self.clock.now_ms())?;
            }
        }
        Ok(true)
    }
    fn upload_stem(folder_key: &str, uid: &str) -> String {
        format!(
            "{}-{}",
            folder_key,
            uid.chars()
                .filter(|c| *c != '-')
                .take(12)
                .collect::<String>()
                .to_lowercase()
        )
    }
    fn name_matches(stem: &str, name: &str) -> bool {
        if name == format!("{stem}.jpg") {
            return true;
        }
        let Some(suffix) = name
            .strip_prefix(&format!("{stem}_"))
            .and_then(|s| s.strip_suffix(".jpg"))
        else {
            return false;
        };
        uuid::Uuid::parse_str(suffix).is_ok()
    }
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
        let total = (main.len() + condition.len()) as i64;
        let mut uploaded = 0;
        for (group, ids, role) in [
            (ExportGroup::Main, &main, "gallery".to_string()),
            (
                ExportGroup::Condition,
                &condition,
                format!("attribute:{}", self.condition_key()?.unwrap_or_default()),
            ),
        ] {
            for (index, id) in ids.iter().enumerate() {
                if group == ExportGroup::Condition && self.condition_key()?.is_none() {
                    continue;
                }
                // Safety recheck immediately before any remote write.
                let photo = self.store.photo(*id)?;
                if photo.session_id != item.session_id
                    || photo.classification == Classification::Deleted
                {
                    continue;
                }
                let mut link = self
                    .store
                    .photo_link(*id, local.id, &role)?
                    .ok_or(Error::UploadIncomplete)?;
                if link.upload_state == "ready" {
                    uploaded += 1;
                    continue;
                }
                let stem = format!(
                    "{}{}",
                    Self::upload_stem(&local.folder_key, &photo.uid),
                    if link.revision > 0 {
                        format!("-r{}", link.revision)
                    } else {
                        String::new()
                    }
                );
                let existing = if group == ExportGroup::Main {
                    self.commerce.list_media(&remote)?
                } else {
                    self.commerce.list_files_by_name(&stem)?
                };
                if existing.iter().any(|m| {
                    m.filename.is_none()
                        && matches!(
                            m.status,
                            crate::domain::publish::RemoteMediaStatus::Processing
                                | crate::domain::publish::RemoteMediaStatus::Uploaded
                        )
                }) {
                    return Ok(false);
                }
                if link.external_id.is_none()
                    && let Some(found) = existing.iter().find(|m| {
                        m.filename
                            .as_deref()
                            .is_some_and(|name| Self::name_matches(&stem, name))
                    })
                {
                    link.external_id = Some(found.id.clone());
                    link.upload_state = "attached".into();
                    self.store.upsert_photo_link(&link, self.clock.now_ms())?;
                }
                if link.external_id.is_none() {
                    let file = staging.join(export_file_name(&key, index + 1, ids.len(), group));
                    let bytes = std::fs::read(&file)?;
                    let target = self.commerce.staged_uploads_create(
                        run,
                        &local.folder_key,
                        &photo.uid,
                        link.revision,
                        bytes.len() as u64,
                    )?;
                    link.upload_state = "staged".into();
                    self.store.upsert_photo_link(&link, self.clock.now_ms())?;
                    self.commerce.upload(run, &target, &bytes)?;
                    link.upload_state = "uploaded".into();
                    self.store.upsert_photo_link(&link, self.clock.now_ms())?;
                    let external = if group == ExportGroup::Main {
                        self.commerce.product_create_media(
                            run,
                            &remote,
                            &target.resource_url,
                            &local.title,
                        )?
                    } else {
                        self.commerce
                            .file_create(run, &target.resource_url, &target.filename)?
                    };
                    link.external_id = Some(external);
                    link.upload_state = "attached".into();
                    self.store.upsert_photo_link(&link, self.clock.now_ms())?;
                }
                let external = link.external_id.as_deref().ok_or(Error::UploadIncomplete)?;
                if group == ExportGroup::Main {
                    self.commerce.poll_media_ready(external)?;
                } else {
                    self.commerce.poll_file(external)?;
                }
                link.upload_state = "ready".into();
                self.store.upsert_photo_link(&link, self.clock.now_ms())?;
                uploaded += 1;
                self.store.set_publish_progress(item.id, uploaded, total)?;
                self.events.emit(AppEvent::PublishProgress {
                    session_id: item.session_id,
                    item_id: item.id,
                    step: PublishStep::Upload,
                    uploaded,
                    total,
                });
            }
        }
        Ok(true)
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
        let mut desired = Vec::new();
        for id in self.selected(item.session_id, &item.frozen_main)? {
            let link = self
                .store
                .photo_link(id, local.id, "gallery")?
                .ok_or(Error::UploadIncomplete)?;
            if link.upload_state != "ready" {
                return Err(Error::UploadIncomplete);
            }
            desired.push(link.external_id.ok_or(Error::UploadIncomplete)?);
        }
        let current_media = self.commerce.list_media(&external)?;
        let current: Vec<String> = current_media.iter().map(|m| m.id.clone()).collect();
        let mut own = HashSet::new();
        let mut stems = Vec::new();
        for link in self.store.photo_links_for_item(local.id)? {
            if link.role == "gallery" {
                own.extend(link.external_id);
                own.extend(link.superseded_ids);
                stems.push(Self::upload_stem(
                    &local.folder_key,
                    &self.store.photo(link.photo_id)?.uid,
                ));
            }
        }
        for media in &current_media {
            if media
                .filename
                .as_deref()
                .is_some_and(|name| stems.iter().any(|stem| Self::name_matches(stem, name)))
            {
                own.insert(media.id.clone());
            }
        }
        let announced: HashSet<String> = item.announced_removals.iter().cloned().collect();
        let plan = reconcile::plan(&desired, &current, &own, &announced)
            .map_err(|reconcile::ReconcileError::Foreign(n)| Error::ForeignMediaAdded { n })?;
        if !plan.deletes.is_empty() {
            self.commerce.product_delete_media(
                run,
                &external,
                &plan.deletes.into_iter().collect::<Vec<_>>(),
            )?;
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
                let mut ids = Vec::new();
                for photo_id in selected {
                    let link = self
                        .store
                        .photo_link(photo_id, local.id, &format!("attribute:{key}"))?
                        .ok_or(Error::UploadIncomplete)?;
                    if link.upload_state != "ready" {
                        return Err(Error::UploadIncomplete);
                    }
                    ids.push(link.external_id.ok_or(Error::UploadIncomplete)?);
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
        if remote.title != item.title {
            self.commerce.update_title(run, &external, &item.title)?;
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
            && remote_description.as_deref() != Some(description)
        {
            self.commerce
                .update_description(run, &external, description)?;
        }
        if let Some(local_tags) = item
            .frozen_attributes
            .get("product.tags")
            .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        {
            let link = self
                .store
                .item_link(local.id, "shopify")?
                .ok_or(Error::UploadIncomplete)?;
            let base: Vec<String> = link
                .tags_base
                .as_deref()
                .and_then(|text| serde_json::from_str(text).ok())
                .unwrap_or_default();
            if let Some(tags) = merge_tags(&base, &local_tags, &remote_tags) {
                self.commerce.update_tags(run, &external, &tags)?;
                self.store.set_item_tags_base(local.id, "shopify", &tags)?;
            }
        }
        Ok(true)
    }
}
