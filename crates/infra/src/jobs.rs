use psm_core::{Error, Result, domain::editing::Job, ports::store::Store};
use std::{collections::HashMap, sync::Arc, time::Duration};

pub trait JobHandler: Send + Sync {
    fn run(&self, job: &Job, now_ms: i64) -> Result<()>;
    fn max_attempts(&self) -> Option<i64> {
        None
    }
    /// The longest wait between two attempts (changelog 45: a publish waiting for its export is
    /// looked at again at least every 15 s); `None` = the standard back-off.
    fn max_backoff_ms(&self) -> Option<i64> {
        None
    }
}
pub struct JobRunner {
    store: Arc<dyn Store>,
    handlers: HashMap<String, Arc<dyn JobHandler>>,
    only_kind: Option<String>,
    excluded_kinds: Vec<String>,
}
impl JobRunner {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self {
            store,
            handlers: HashMap::new(),
            only_kind: None,
            excluded_kinds: Vec::new(),
        }
    }
    pub fn only_kind(mut self, kind: &str) -> Self {
        self.only_kind = Some(kind.into());
        self
    }
    /// This runner leaves jobs of these kinds to their own runners.
    pub fn excluding_kinds(mut self, kinds: &[&str]) -> Self {
        self.excluded_kinds = kinds.iter().map(|k| (*k).to_string()).collect();
        self
    }
    pub fn register(&mut self, kind: &str, handler: Arc<dyn JobHandler>) {
        self.handlers.insert(kind.into(), handler);
    }
    pub fn tick(&self, now_ms: i64) -> Result<usize> {
        if self.handlers.is_empty() {
            return Ok(0);
        }
        let mut count = 0;
        let excluded: Vec<&str> = self.excluded_kinds.iter().map(String::as_str).collect();
        while let Some(job) = self
            .store
            .job_due(now_ms, self.only_kind.as_deref(), &excluded)?
        {
            let Some(handler) = self.handlers.get(&job.kind) else {
                self.store
                    .job_retry(job.id, "unknown_job_kind", now_ms + 60_000, now_ms)?;
                count += 1;
                continue;
            };
            let outcome = handler.run(&job, now_ms);
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
                    if handler
                        .max_attempts()
                        .is_some_and(|limit| attempts >= limit)
                    {
                        self.store.job_fail(job.id, code, now_ms)?;
                    } else {
                        let mut wait = backoff(attempts).as_millis() as i64;
                        if let Some(cap) = handler.max_backoff_ms() {
                            wait = wait.min(cap);
                        }
                        self.store
                            .job_retry(job.id, code, now_ms + wait, now_ms)?;
                    }
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
