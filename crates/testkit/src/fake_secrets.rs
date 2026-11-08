use psm_core::{Result, ports::secrets::SecretStore};
use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
pub struct FakeSecrets {
    pub values: Mutex<HashMap<String, String>>,
    cache_enabled: AtomicBool,
}
impl Default for FakeSecrets {
    fn default() -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
            cache_enabled: AtomicBool::new(true),
        }
    }
}
impl FakeSecrets {
    pub fn disable_cache(&self) {
        self.cache_enabled.store(false, Ordering::SeqCst);
    }
}
impl SecretStore for FakeSecrets {
    fn cached(&self, account: &str) -> Option<String> {
        if self.cache_enabled.load(Ordering::SeqCst) {
            self.values.lock().unwrap().get(account).cloned()
        } else {
            None
        }
    }
    fn get(&self, account: &str) -> Result<Option<String>> {
        Ok(self.values.lock().unwrap().get(account).cloned())
    }
    fn set(&self, account: &str, value: &str) -> Result<()> {
        self.values
            .lock()
            .unwrap()
            .insert(account.into(), value.into());
        Ok(())
    }
    fn delete(&self, account: &str) -> Result<()> {
        self.values.lock().unwrap().remove(account);
        Ok(())
    }
}
