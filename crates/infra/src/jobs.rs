use psm_core::{Error, Result, domain::editing::Job, ports::store::Store};
use std::{collections::HashMap, sync::Arc, time::Duration};

pub trait JobHandler: Send + Sync {
    fn run(&self, job: &Job, now_ms: i64) -> Result<()>;
}
pub struct JobRunner {
    store: Arc<dyn Store>,
    handlers: HashMap<String, Arc<dyn JobHandler>>,
    only_kind: Option<String>,
    excluded_kind: Option<String>,
}
impl JobRunner {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self {
            store,
            handlers: HashMap::new(),
            only_kind: None,
            excluded_kind: None,
        }
    }
    pub fn only_kind(mut self, kind: &str) -> Self {
        self.only_kind = Some(kind.into());
        self
    }
    pub fn excluding_kind(mut self, kind: &str) -> Self {
        self.excluded_kind = Some(kind.into());
        self
    }
    pub fn register(&mut self, kind: &str, handler: Arc<dyn JobHandler>) {
        self.handlers.insert(kind.into(), handler);
    }
    pub fn tick(&self, now_ms: i64) -> Result<usize> {
        let mut count = 0;
        while let Some(job) = self.store.job_due(
            now_ms,
            self.only_kind.as_deref(),
            self.excluded_kind.as_deref(),
        )? {
            let outcome = self
                .handlers
                .get(&job.kind)
                .ok_or_else(|| Error::InvalidData(format!("unknown job kind: {}", job.kind)))?
                .run(&job, now_ms);
            match outcome {
                Ok(()) => self.store.job_done(job.id, now_ms)?,
                Err(error) => {
                    let code = match error {
                        Error::VariantNotFound => "variant_not_found",
                        Error::TagVerification => "tag_verification",
                        Error::Timeout => "timeout",
                        _ => "provider",
                    };
                    let attempts = job.attempts + 1;
                    self.store.job_retry(
                        job.id,
                        code,
                        now_ms + backoff(attempts).as_millis() as i64,
                        now_ms,
                    )?;
                }
            }
            count += 1;
        }
        Ok(count)
    }
}
pub fn backoff(attempts: i64) -> Duration {
    Duration::from_secs((1_u64 << attempts.saturating_sub(1).min(6)).min(60))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mc19_backoff() {
        assert_eq!(
            (1..=7).map(|n| backoff(n).as_secs()).collect::<Vec<_>>(),
            [1, 2, 4, 8, 16, 32, 60]
        );
    }
}
