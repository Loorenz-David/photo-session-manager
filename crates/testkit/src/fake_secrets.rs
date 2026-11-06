use psm_core::{Result, ports::secrets::SecretStore};
use std::{collections::HashMap, sync::Mutex};
#[derive(Default)]
pub struct FakeSecrets(pub Mutex<HashMap<String, String>>);
impl SecretStore for FakeSecrets {
    fn get(&self, account: &str) -> Result<Option<String>> {
        Ok(self.0.lock().unwrap().get(account).cloned())
    }
    fn set(&self, account: &str, value: &str) -> Result<()> {
        self.0.lock().unwrap().insert(account.into(), value.into());
        Ok(())
    }
    fn delete(&self, account: &str) -> Result<()> {
        self.0.lock().unwrap().remove(account);
        Ok(())
    }
}
