use crate::Result;

pub trait SecretStore: Send + Sync {
    fn get(&self, account: &str) -> Result<Option<String>>;
    /// A prompt-free value if the store has one in memory. Keychain never does; startup reads it
    /// on a background thread so its access prompt cannot hold the USB start order.
    fn cached(&self, _account: &str) -> Option<String> {
        None
    }
    fn set(&self, account: &str, value: &str) -> Result<()>;
    fn delete(&self, account: &str) -> Result<()>;
}
