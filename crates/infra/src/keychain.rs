use psm_core::{Error, Result, domain::camera::secrets, ports::secrets::SecretStore};
use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};

pub struct KeychainSecrets {
    pub service: &'static str,
}
impl Default for KeychainSecrets {
    fn default() -> Self {
        Self {
            service: secrets::SERVICE,
        }
    }
}
impl SecretStore for KeychainSecrets {
    fn get(&self, account: &str) -> Result<Option<String>> {
        match get_generic_password(self.service, account) {
            Ok(bytes) => String::from_utf8(bytes)
                .map(Some)
                .map_err(|e| Error::InvalidData(e.to_string())),
            Err(e) if e.code() == -25300 => Ok(None),
            Err(e) => Err(Error::InvalidData(format!("Keychain read failed: {e}"))),
        }
    }
    fn set(&self, account: &str, value: &str) -> Result<()> {
        set_generic_password(self.service, account, value.as_bytes())
            .map_err(|e| Error::InvalidData(format!("Keychain write failed: {e}")))
    }
    fn delete(&self, account: &str) -> Result<()> {
        match delete_generic_password(self.service, account) {
            Ok(()) => Ok(()),
            Err(e) if e.code() == -25300 => Ok(()),
            Err(e) => Err(Error::InvalidData(format!("Keychain delete failed: {e}"))),
        }
    }
}
