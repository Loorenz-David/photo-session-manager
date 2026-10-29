use psm_core::{
    Error, Result,
    domain::editing::{Job, ShapeOutcome},
    ports::{editing::EditingProvider, store::Store},
    services::{health::HealthService, previews::PreviewService},
};
use psm_infra::jobs::JobHandler;
use std::sync::Arc;

pub const CROP_GIVE_UP_ATTEMPTS: i64 = 8;

pub struct ShapeOnArrival {
    pub editing: Arc<dyn EditingProvider>,
    pub store: Arc<dyn Store>,
    pub health: Arc<HealthService>,
    pub previews: Arc<PreviewService>,
    pub ratio: String,
}
impl JobHandler for ShapeOnArrival {
    fn max_attempts(&self) -> Option<i64> {
        Some(CROP_GIVE_UP_ATTEMPTS)
    }
    fn run(&self, job: &Job, now_ms: i64) -> Result<()> {
        if self.ratio == "off" {
            return Ok(());
        }
        match self
            .editing
            .shape_on_arrival(&job.target, &self.ratio, now_ms)
        {
            Ok(ShapeOutcome::Applied) => {
                self.previews.on_variant_adjusted("", &job.target)?;
                self.health.clear("variant_not_found");
                self.health.clear("crop_ratio_unknown");
                Ok(())
            }
            Ok(ShapeOutcome::Skipped) => Ok(()),
            Ok(ShapeOutcome::UnknownRatio) | Err(Error::CropRatioUnknown { .. }) => {
                self.health.set(
                    "crop_ratio_unknown",
                    format!(
                        "Shape on arrival: '{}' is not a Capture One crop ratio",
                        self.ratio
                    ),
                );
                Err(Error::CropRatioUnknown {
                    name: self.ratio.clone(),
                })
            }
            Err(Error::VariantNotFound) => {
                self.health.set("variant_not_found", &job.target);
                Err(Error::VariantNotFound)
            }
            Err(error) => Err(error),
        }
    }
}
