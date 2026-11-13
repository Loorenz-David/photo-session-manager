//! The one place adapters are chosen, by the ids in `research.providers.*` (a saved `""` means
//! the default id, `ProviderIds::resolved`). An unknown id gives no adapter for that port. `build`
//! never looks at whether a key is set: MC-A6's prerequisites decide "Add the model key". Keys
//! are read from the secrets port on every call, so a key saved while the app runs is used by the
//! next call.

pub mod http;
pub mod openai;

use std::{sync::Arc, time::Duration};

use psm_core::ports::secrets::SecretStore;

use crate::{
    config::{DEFAULT_MODEL_PROVIDER, ResearchSettings},
    files::ResearchFiles,
    ports::{ModelProvider, Providers},
};
use http::{AddressGuard, ReqwestResearchHttp, ResearchHttp};

fn client(seconds: u32, guard: AddressGuard) -> Option<Arc<dyn ResearchHttp>> {
    match ReqwestResearchHttp::new(Duration::from_secs(u64::from(seconds)), guard) {
        Ok(client) => Some(Arc::new(client)),
        Err(error) => {
            tracing::warn!(%error, "research http client not available");
            None
        }
    }
}

pub fn build(
    settings: &ResearchSettings,
    secrets: Arc<dyn SecretStore>,
    files: ResearchFiles,
) -> Providers {
    let ids = settings.providers.resolved();
    let timeouts = &settings.call_timeout_seconds;
    let model: Option<Arc<dyn ModelProvider>> = if ids.model == DEFAULT_MODEL_PROVIDER {
        client(timeouts.model, AddressGuard::Any).map(|http| {
            Arc::new(openai::OpenAiModel::new(
                http,
                secrets.clone(),
                files.clone(),
            )) as Arc<dyn ModelProvider>
        })
    } else {
        None
    };
    Providers {
        model,
        search: None,
        reader: None,
    }
}
