//! `ResearchJob`: the `jobs` handler of kind `research` (target = the run id). It runs the run's
//! stages; a call that could not leave (offline) makes the job wait with the jobs back-off,
//! capped at 60 s, without counting an attempt of the call (MC-A6 Offline).

use std::sync::Arc;

use psm_core::{Error, Result, domain::editing::Job};
use psm_infra::jobs::JobHandler;

use crate::{service::ResearchService, stages::ExecuteOutcome};

/// The job kind.
pub const KIND: &str = "research";
/// The longest wait between two tries of an offline run.
pub const MAX_BACKOFF_MS: i64 = 60_000;

pub struct ResearchJob {
    service: Arc<ResearchService>,
}

impl ResearchJob {
    pub fn new(service: Arc<ResearchService>) -> Self {
        Self { service }
    }
}

impl JobHandler for ResearchJob {
    fn run(&self, job: &Job, _now_ms: i64) -> Result<()> {
        let run_id: i64 = job
            .target
            .parse()
            .map_err(|_| Error::InvalidData(format!("research job target {}", job.target)))?;
        match self.service.execute(run_id)? {
            ExecuteOutcome::HoldOffline => Err(Error::Timeout),
            _ => Ok(()),
        }
    }

    fn max_backoff_ms(&self) -> Option<i64> {
        Some(MAX_BACKOFF_MS)
    }
}
