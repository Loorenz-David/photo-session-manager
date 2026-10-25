use crate::events::AppEvent;

pub trait EventSink: Send + Sync {
    fn emit(&self, event: AppEvent);
}

