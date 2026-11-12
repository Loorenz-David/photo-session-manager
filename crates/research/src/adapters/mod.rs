//! The one place adapters are chosen, by the ids in `research.providers.*`. No adapter is known
//! yet (R8 adds them): an unknown or empty id gives none, and research says the provider is not
//! set (prerequisites `NoModelKey` / `NoSearchKey`).

use psm_core::ports::secrets::SecretStore;

use crate::{config::ResearchSettings, ports::Providers};

pub fn build(settings: &ResearchSettings, secrets: &dyn SecretStore) -> Providers {
    let _ = (&settings.providers, secrets);
    Providers::default()
}
