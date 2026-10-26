use psm_core::{
    Error, Result,
    domain::{
        attribute::Definition,
        publish::{
            ConfirmedRun, DefinitionLink, RemoteIdentity, RemoteItem, RemoteMedia,
            RemoteMediaStatus, StagedTarget, WriteValue,
        },
    },
    ports::{
        clock::Clock,
        commerce::{ALL_CAPABILITIES, CommerceAdapter},
    },
};
use serde_json::Value as Json;
use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct Mutation {
    pub op: String,
    pub run_id: i64,
    pub at: i64,
}
#[derive(Default)]
pub struct FakeShop {
    pub products: Vec<RemoteItem>,
    pub files: Vec<RemoteMedia>,
    pub definitions: Vec<(Definition, DefinitionLink)>,
    pub values: HashMap<String, Vec<(String, Json)>>,
    pub attribute_writes: Vec<Vec<WriteValue>>,
    pub mutations: Vec<Mutation>,
    pub reads: Vec<String>,
    pub fail_after: Option<String>,
    pub fail_before: Option<String>,
    pub unreachable: bool,
    pub schema_error: bool,
    pub wrong_order: bool,
    pub delete_not_found: bool,
    pub not_found_delivered: bool,
    pub next_id: i64,
    pub staged: HashMap<String, String>,
    pub tags: HashMap<String, Vec<String>>,
    pub descriptions: HashMap<String, String>,
    pub readback_override: Option<(String, Json)>,
    pub identity_override: Option<RemoteIdentity>,
}
pub struct FakeCommerce {
    pub state: Mutex<FakeShop>,
    pub condition_key: String,
    clock: Arc<dyn Clock>,
    media_poll: (Mutex<MediaPollGate>, Condvar),
}
#[derive(Default)]
struct MediaPollGate {
    held: bool,
    entered: bool,
}
impl FakeCommerce {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self::with_condition_key(clock, "custom.damage_report")
    }
    pub fn with_condition_key(clock: Arc<dyn Clock>, key: &str) -> Self {
        Self {
            state: Mutex::new(FakeShop::default()),
            condition_key: key.into(),
            clock,
            media_poll: (Mutex::new(MediaPollGate::default()), Condvar::new()),
        }
    }
    pub fn hold_media_poll(&self) {
        let mut gate = self.media_poll.0.lock().unwrap();
        gate.held = true;
        gate.entered = false;
    }
    pub fn wait_for_media_poll(&self) -> bool {
        let (lock, ready) = &self.media_poll;
        let gate = lock.lock().unwrap();
        let (gate, _) = ready
            .wait_timeout_while(gate, Duration::from_secs(5), |gate| !gate.entered)
            .unwrap();
        gate.entered
    }
    pub fn release_media_poll(&self) {
        let mut gate = self.media_poll.0.lock().unwrap();
        gate.held = false;
        self.media_poll.1.notify_all();
    }
    pub fn add_product(&self, sku: &str, title: &str, barcode: Option<&str>) -> String {
        let mut s = self.state.lock().unwrap();
        s.next_id += 1;
        let id = format!("product-{}", s.next_id);
        s.products.push(RemoteItem {
            external_id: id.clone(),
            title: title.into(),
            sku: Some(sku.into()),
            barcode: barcode.map(Into::into),
            thumbnail_url: None,
            media: Vec::new(),
        });
        id
    }
    pub fn fail_after_effect(&self, op: &str) {
        self.state.lock().unwrap().fail_after = Some(op.into());
    }
    pub fn fail_before_effect(&self, op: &str) {
        self.state.lock().unwrap().fail_before = Some(op.into());
    }
    fn read(&self, op: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.reads.push(op.into());
        let interrupted = s.fail_after.as_deref() == Some(op);
        if interrupted {
            s.fail_after = None;
        }
        let unreachable = s.unreachable;
        drop(s);
        if interrupted {
            panic!("simulated process kill after {op}")
        }
        if unreachable {
            Err(Error::Provider("shop unavailable".into()))
        } else {
            Ok(())
        }
    }
    fn effect(&self, run: &ConfirmedRun, op: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        if s.fail_before.as_deref() == Some(op) {
            s.fail_before = None;
            return Err(Error::Provider(format!("{op} unavailable")));
        }
        s.mutations.push(Mutation {
            op: op.into(),
            run_id: run.run_id(),
            at: self.clock.now_ms(),
        });
        if s.fail_after.as_deref() == Some(op) {
            s.fail_after = None;
            drop(s);
            panic!("simulated process kill after {op}")
        }
        Ok(())
    }
}
impl CommerceAdapter for FakeCommerce {
    fn condition_photos_key(&self) -> &str {
        &self.condition_key
    }
    fn capabilities(&self) -> Vec<&'static str> {
        ALL_CAPABILITIES.to_vec()
    }
    fn item_search(&self, sku: &str) -> Result<Vec<RemoteItem>> {
        self.read("item_search")?;
        Ok(self
            .state
            .lock()
            .unwrap()
            .products
            .iter()
            .filter(|p| p.sku.as_deref() == Some(sku))
            .cloned()
            .collect())
    }
    fn item_find(&self, query: &str) -> Result<Vec<RemoteItem>> {
        self.read("item_find")?;
        let q = query.to_lowercase();
        Ok(self
            .state
            .lock()
            .unwrap()
            .products
            .iter()
            .filter(|p| {
                p.sku
                    .as_ref()
                    .is_some_and(|s| s.to_lowercase().contains(&q))
                    || p.title.to_lowercase().contains(&q)
            })
            .cloned()
            .collect())
    }
    fn item_get(&self, id: &str) -> Result<RemoteItem> {
        self.read("item_get")?;
        self.state
            .lock()
            .unwrap()
            .products
            .iter()
            .find(|p| p.external_id == id)
            .cloned()
            .ok_or_else(|| Error::NotFound(id.into()))
    }
    fn product_fields(&self, id: &str) -> Result<(Vec<String>, Option<String>)> {
        self.read("product_fields")?;
        let s = self.state.lock().unwrap();
        Ok((
            s.tags.get(id).cloned().unwrap_or_default(),
            s.descriptions.get(id).cloned(),
        ))
    }
    fn item_create(
        &self,
        run: &ConfirmedRun,
        sku: &str,
        barcode: Option<&str>,
        title: &str,
    ) -> Result<String> {
        let id = self.add_product(sku, title, barcode);
        self.effect(run, "item_create")?;
        Ok(id)
    }
    fn item_set_identity(
        &self,
        run: &ConfirmedRun,
        id: &str,
        sku: &str,
        barcode: Option<&str>,
    ) -> Result<RemoteIdentity> {
        let mut s = self.state.lock().unwrap();
        let p = s
            .products
            .iter_mut()
            .find(|p| p.external_id == id)
            .ok_or_else(|| Error::NotFound(id.into()))?;
        p.sku = Some(sku.into());
        if let Some(barcode) = barcode {
            p.barcode = Some(barcode.into())
        };
        let out = RemoteIdentity {
            sku: sku.into(),
            barcode: p.barcode.clone(),
        };
        let out = s.identity_override.clone().unwrap_or(out);
        drop(s);
        self.effect(run, "item_set_identity")?;
        Ok(out)
    }
    fn attributes_schema(&self) -> Result<Vec<(Definition, DefinitionLink)>> {
        self.read("attributes_schema")?;
        let s = self.state.lock().unwrap();
        if s.schema_error {
            return Err(Error::Provider("schema".into()));
        }
        Ok(s.definitions.clone())
    }
    fn attributes_read(&self, id: &str) -> Result<Vec<(String, Json)>> {
        self.read("attributes_read")?;
        let mut values = self
            .state
            .lock()
            .unwrap()
            .values
            .get(id)
            .cloned()
            .unwrap_or_default();
        values.retain(|(key, _)| key != &self.condition_key);
        if let Some((key, value)) = &self.state.lock().unwrap().readback_override
            && let Some(row) = values.iter_mut().find(|(k, _)| k == key)
        {
            row.1 = value.clone();
        }
        Ok(values)
    }
    fn condition_photos_readback(&self, id: &str) -> Result<Option<Json>> {
        self.read("condition_photos_readback")?;
        let state = self.state.lock().unwrap();
        if let Some((key, value)) = &state.readback_override
            && key == &self.condition_key
        {
            return Ok(Some(value.clone()));
        }
        Ok(state
            .values
            .get(id)
            .and_then(|values| values.iter().find(|(key, _)| key == &self.condition_key))
            .map(|(_, value)| value.clone()))
    }
    fn attributes_write(&self, run: &ConfirmedRun, id: &str, values: &[WriteValue]) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.attribute_writes.push(values.to_vec());
        let rows = s.values.entry(id.into()).or_default();
        for v in values {
            if let Some(row) = rows.iter_mut().find(|(key, _)| key == &v.key) {
                row.1 = v.value.clone()
            } else {
                rows.push((v.key.clone(), v.value.clone()))
            }
        }
        drop(s);
        self.effect(run, "attributes_write")
    }
    fn staged_uploads_create(
        &self,
        run: &ConfirmedRun,
        folder_key: &str,
        uid: &str,
        revision: i64,
        _size: u64,
    ) -> Result<StagedTarget> {
        let clean = uid
            .chars()
            .filter(|c| *c != '-')
            .take(12)
            .collect::<String>()
            .to_lowercase();
        let suffix = if revision > 0 {
            format!("-r{revision}")
        } else {
            String::new()
        };
        let filename = format!("{folder_key}-{clean}{suffix}.jpg");
        let mut s = self.state.lock().unwrap();
        s.next_id += 1;
        let resource_url = format!("staged-{}", s.next_id);
        s.staged.insert(resource_url.clone(), filename.clone());
        drop(s);
        self.effect(run, "staged_uploads_create")?;
        Ok(StagedTarget {
            filename,
            url: resource_url.clone(),
            resource_url,
            parameters: Vec::new(),
        })
    }
    fn upload(&self, run: &ConfirmedRun, _target: &StagedTarget, _bytes: &[u8]) -> Result<()> {
        self.effect(run, "upload")
    }
    fn product_create_media(
        &self,
        run: &ConfirmedRun,
        id: &str,
        resource_url: &str,
        _alt: &str,
    ) -> Result<String> {
        let mut s = self.state.lock().unwrap();
        let filename = s
            .staged
            .get(resource_url)
            .cloned()
            .ok_or_else(|| Error::NotFound(resource_url.into()))?;
        s.next_id += 1;
        let media_id = format!("media-{}", s.next_id);
        let p = s
            .products
            .iter_mut()
            .find(|p| p.external_id == id)
            .ok_or_else(|| Error::NotFound(id.into()))?;
        p.media.push(RemoteMedia {
            id: media_id.clone(),
            filename: Some(filename),
            status: RemoteMediaStatus::Ready,
            position: p.media.len() as i64 + 1,
        });
        drop(s);
        self.effect(run, "product_create_media")?;
        Ok(media_id)
    }
    fn poll_media_ready(&self, _id: &str) -> Result<()> {
        self.read("poll_media_ready")?;
        let (lock, ready) = &self.media_poll;
        let mut gate = lock.lock().unwrap();
        if gate.held {
            gate.entered = true;
            ready.notify_all();
            while gate.held {
                gate = ready.wait(gate).unwrap();
            }
        }
        Ok(())
    }
    fn product_reorder_media(
        &self,
        run: &ConfirmedRun,
        id: &str,
        media_ids: &[String],
    ) -> Result<String> {
        let mut s = self.state.lock().unwrap();
        let wrong = s.wrong_order;
        let p = s
            .products
            .iter_mut()
            .find(|p| p.external_id == id)
            .ok_or_else(|| Error::NotFound(id.into()))?;
        if !wrong {
            let mut sorted = Vec::new();
            for media_id in media_ids {
                if let Some(m) = p.media.iter().find(|m| &m.id == media_id) {
                    sorted.push(m.clone())
                }
            }
            for (i, m) in sorted.iter_mut().enumerate() {
                m.position = i as i64 + 1
            }
            p.media = sorted;
        } else {
            p.media.reverse();
        }
        drop(s);
        self.effect(run, "product_reorder_media")?;
        Ok("job-1".into())
    }
    fn poll_job(&self, _job_id: &str) -> Result<()> {
        self.read("poll_job")
    }
    fn list_media(&self, id: &str) -> Result<Vec<RemoteMedia>> {
        self.read("list_media")?;
        Ok(self.item_get(id)?.media)
    }
    fn list_files_by_name(&self, stem: &str) -> Result<Vec<RemoteMedia>> {
        self.read("list_files_by_name")?;
        Ok(self
            .state
            .lock()
            .unwrap()
            .files
            .iter()
            .filter(|f| f.filename.as_deref().is_some_and(|n| n.starts_with(stem)))
            .cloned()
            .collect())
    }
    fn product_delete_media(&self, run: &ConfirmedRun, id: &str, ids: &[String]) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        if s.delete_not_found {
            s.not_found_delivered = true;
            s.delete_not_found = false;
        }
        let p = s
            .products
            .iter_mut()
            .find(|p| p.external_id == id)
            .ok_or_else(|| Error::NotFound(id.into()))?;
        p.media.retain(|m| !ids.contains(&m.id));
        drop(s);
        self.effect(run, "product_delete_media")
    }
    fn file_create(
        &self,
        run: &ConfirmedRun,
        resource_url: &str,
        _filename: &str,
    ) -> Result<String> {
        let mut s = self.state.lock().unwrap();
        let name = s
            .staged
            .get(resource_url)
            .cloned()
            .ok_or_else(|| Error::NotFound(resource_url.into()))?;
        s.next_id += 1;
        let id = format!("file-{}", s.next_id);
        let position = s.files.len() as i64 + 1;
        s.files.push(RemoteMedia {
            id: id.clone(),
            filename: Some(name),
            status: RemoteMediaStatus::Ready,
            position,
        });
        drop(s);
        self.effect(run, "file_create")?;
        Ok(id)
    }
    fn poll_file(&self, _file_id: &str) -> Result<()> {
        self.read("poll_file")
    }
    fn update_title(&self, run: &ConfirmedRun, id: &str, title: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        if let Some(p) = s.products.iter_mut().find(|p| p.external_id == id) {
            p.title = title.into()
        }
        drop(s);
        self.effect(run, "update_title")
    }
    fn update_description(&self, run: &ConfirmedRun, id: &str, paragraphs: &str) -> Result<()> {
        self.state.lock().unwrap().descriptions.insert(
            id.into(),
            psm_integrations::commerce::shopify::adapter::description_html(paragraphs),
        );
        self.effect(run, "update_description")
    }
    fn update_tags(&self, run: &ConfirmedRun, id: &str, tags: &[String]) -> Result<()> {
        self.state
            .lock()
            .unwrap()
            .tags
            .insert(id.into(), tags.to_vec());
        self.effect(run, "update_tags")
    }
}
