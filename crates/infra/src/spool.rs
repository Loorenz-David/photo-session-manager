use crate::paths::AppPaths;
use psm_core::{
    Error, Result,
    mechanisms::hook_payload::{HookEvent, parse},
    ports::{clock::Clock, store::Store},
    services::editing_events::EditingEventsDispatcher,
};
use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

pub type HookPayload = HookEvent;
pub fn parse_spool_file(bytes: &[u8]) -> Result<HookPayload> {
    parse(bytes)
}

pub struct Spool {
    paths: AppPaths,
    drain_lock: Mutex<()>,
    dispatcher: Mutex<Option<Arc<EditingEventsDispatcher>>>,
}

impl Spool {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            drain_lock: Mutex::new(()),
            dispatcher: Mutex::new(None),
        }
    }
    pub fn set_dispatcher(&self, dispatcher: Arc<EditingEventsDispatcher>) -> Result<()> {
        *self
            .dispatcher
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))? = Some(dispatcher);
        Ok(())
    }

    pub fn drain(&self, store: &dyn Store, clock: &dyn Clock) -> Result<()> {
        let _guard = self
            .drain_lock
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let mut files = fs::read_dir(self.paths.spool())?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|s| s == "json"))
            .collect::<Vec<_>>();
        files.sort();
        for path in files {
            let bytes = fs::read(&path)?;
            match parse_spool_file(&bytes) {
                Ok(payload) => {
                    let content =
                        String::from_utf8(bytes).map_err(|e| Error::InvalidData(e.to_string()))?;
                    let hook = payload.hook();
                    let name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .ok_or_else(|| Error::InvalidData("invalid spool filename".into()))?;
                    store.record_hook(name, hook, &content, clock.now_ms())?;
                    if let Some(dispatcher) = self
                        .dispatcher
                        .lock()
                        .map_err(|e| Error::InvalidData(e.to_string()))?
                        .as_ref()
                    {
                        dispatcher.dispatch(&payload)?;
                    }
                    fs::remove_file(path)?;
                }
                Err(_) => {
                    let dest = self
                        .paths
                        .spool()
                        .join("bad")
                        .join(path.file_name().unwrap_or_default());
                    fs::rename(path, dest)?;
                }
            }
        }
        Ok(())
    }

    pub fn write_for_test(&self, name: &str, content: &str) -> Result<()> {
        fs::write(self.paths.spool().join(Path::new(name)), content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mc15_parse_processing_payload() {
        let input =
            br#"{"hook":"processing_done","job":"J1","raw":"/tmp/a.ARW","outputs":["/tmp/a.jpg"]}"#;
        assert_eq!(
            parse_spool_file(input).unwrap(),
            HookPayload::ProcessingDone {
                job: "J1".into(),
                raw: "/tmp/a.ARW".into(),
                outputs: vec!["/tmp/a.jpg".into()],
            }
        );
    }
}
