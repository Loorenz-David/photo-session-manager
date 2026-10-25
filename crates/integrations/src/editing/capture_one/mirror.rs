use psm_core::{
    Error, Result,
    domain::editing::Job,
    ports::{editing::EditingProvider, store::Store},
    services::health::HealthService,
};
use psm_infra::jobs::JobHandler;
use std::sync::Arc;

pub struct MirrorTag {
    pub editing: Arc<dyn EditingProvider>,
    pub store: Arc<dyn Store>,
    pub health: Arc<HealthService>,
}
impl JobHandler for MirrorTag {
    fn run(&self, job: &Job, now_ms: i64) -> Result<()> {
        let photo_id = serde_json::from_str::<serde_json::Value>(&job.payload)
            .map_err(|e| Error::InvalidData(e.to_string()))?["photo_id"]
            .as_i64()
            .ok_or_else(|| Error::InvalidData("photo_id".into()))?;
        let photo = self.store.photo(photo_id)?;
        let result = self
            .editing
            .mirror_tag(&photo.raw_path, photo.classification, now_ms);
        match &result {
            Err(Error::VariantNotFound) => self.health.set("variant_not_found", &photo.raw_path),
            Ok(()) => self.health.clear("variant_not_found"),
            _ => {}
        }
        result
    }
}
