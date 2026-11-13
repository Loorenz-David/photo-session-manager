//! The one place adapters are chosen, by the ids in `research.providers.*` (a saved `""` means
//! the default id, `ProviderIds::resolved`). An unknown id gives no adapter for that port. `build`
//! never looks at whether a key is set: MC-A6's prerequisites decide "Add the model key". Keys
//! are read from the secrets port on every call, so a key saved while the app runs is used by the
//! next call.

pub mod http;
pub mod openai;
pub mod page_reader;
pub mod visual_search;

use std::{sync::Arc, time::Duration};

use psm_core::ports::secrets::SecretStore;

use crate::{
    config::{
        DEFAULT_MODEL_PROVIDER, DEFAULT_PAGE_READER, DEFAULT_VISUAL_SEARCH_PROVIDER,
        ResearchSettings,
    },
    files::ResearchFiles,
    gateway::Waiter,
    ports::{ModelProvider, PageReader, Providers, VisualSearchProvider},
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

/// The adapters for the saved ids. `wait` is the wiring's sleep: the page reader paces slow hosts
/// with it (R8·U3 task 3). The page reader needs no key.
pub fn build(
    settings: &ResearchSettings,
    secrets: Arc<dyn SecretStore>,
    files: ResearchFiles,
    wait: Waiter,
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
    let search: Option<Arc<dyn VisualSearchProvider>> =
        if ids.visual_search == DEFAULT_VISUAL_SEARCH_PROVIDER {
            client(timeouts.other, AddressGuard::Any).map(|http| {
                Arc::new(visual_search::SerpApiLens::new(http, secrets.clone()))
                    as Arc<dyn VisualSearchProvider>
            })
        } else {
            None
        };
    let reader: Option<Arc<dyn PageReader>> = if ids.page_reader == DEFAULT_PAGE_READER {
        client(timeouts.other, AddressGuard::PublicOnly).map(|http| {
            Arc::new(page_reader::HttpPageReader::new(http, wait, timeouts.other))
                as Arc<dyn PageReader>
        })
    } else {
        None
    };
    Providers {
        model,
        search,
        reader,
    }
}
