use crate::Result;

pub trait Store: Send + Sync {
    fn mechanism_version(&self) -> Result<String>;
    fn record_hook(&self, hook: &str, payload: &str, received_at: i64) -> Result<()>;
}

