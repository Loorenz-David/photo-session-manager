//! `ResearchService`: the research application service. It owns the gateway, the research-copy
//! renderer and the snapshots, and runs a `running` run's stages (`execute`).

use std::{path::PathBuf, sync::Arc, time::Instant};

use psm_core::{
    Result,
    domain::attribute::{AttributeGroup, Definition},
    events::AppEvent,
    ports::{clock::Clock, events::EventSink, images::Imaging, secrets::SecretStore},
    services::attributes::AttributesService,
};
use psm_infra::sqlite::store::SqliteStore;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{
    files::ResearchFiles,
    gateway::{CallGateway, SecretValues, Waiter},
    images::ResearchImages,
    ports::Providers,
    shop::CommerceShopFacts,
};

/// Keychain accounts of the research keys (`master_plan_research.md` §6.6).
pub const MODEL_ACCOUNT: &str = "research-model-key";
pub const SEARCH_ACCOUNT: &str = "research-search-key";
pub const READER_ACCOUNT: &str = "research-reader-key";

/// Reads the live `research` settings slot (the app's settings file; the harness's in tests).
pub type SettingsSource = Arc<dyn Fn() -> Option<Json> + Send + Sync>;

/// Everything the service is built from (R7·U1 builds it in the app; the harness in tests).
pub struct ResearchDeps {
    pub store: Arc<SqliteStore>,
    pub clock: Arc<dyn Clock>,
    pub imaging: Arc<dyn Imaging>,
    /// The app's data folder (`AppPaths::root`).
    pub data_dir: PathBuf,
    pub providers: Providers,
    pub secrets: Arc<dyn SecretStore>,
    pub secret_values: SecretValues,
    /// `None`: no commerce integration is configured.
    pub shop: Option<Arc<CommerceShopFacts>>,
    /// The integration whose definitions the form uses.
    pub integration: Option<String>,
    pub settings: SettingsSource,
    pub events: Arc<dyn EventSink>,
    pub wait: Waiter,
}

/// One layout field as the run saw it at start (MC-A6 form snapshot).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldSnapshot {
    pub definition: Definition,
    /// The layout step holding the field.
    pub step: AttributeGroup,
    pub value: Option<String>,
    pub provenance: Option<String>,
}

/// The form snapshot: the item type and the ordered layout fields with their values at start.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormSnapshot {
    pub item_type: String,
    pub fields: Vec<FieldSnapshot>,
}

impl FormSnapshot {
    pub fn from_text(text: Option<&str>) -> Result<Self> {
        match text {
            Some(text) => serde_json::from_str(text)
                .map_err(|e| psm_core::Error::InvalidData(format!("form snapshot: {e}"))),
            None => Ok(Self::default()),
        }
    }
}

/// The item's form as research sees it: every layout field of the three steps, in order.
pub fn form_snapshot(
    store: &SqliteStore,
    clock: &dyn Clock,
    integration: Option<&str>,
    item_id: i64,
) -> Result<FormSnapshot> {
    let form = AttributesService::new(store, clock, integration).form_get(item_id)?;
    let item_type = psm_core::ports::store::Store::item(store, item_id)?.item_type;
    let fields = form
        .groups
        .into_iter()
        .flat_map(|group| {
            let step = group.group;
            group.fields.into_iter().map(move |field| FieldSnapshot {
                definition: field.definition,
                step,
                value: field.value.as_ref().map(|v| v.value.clone()),
                provenance: field
                    .value
                    .as_ref()
                    .map(|v| v.provenance.as_str().to_owned()),
            })
        })
        .collect();
    Ok(FormSnapshot { item_type, fields })
}

pub struct ResearchService {
    pub(crate) store: Arc<SqliteStore>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) images: ResearchImages,
    pub(crate) gateway: CallGateway,
    pub(crate) files: ResearchFiles,
    pub(crate) providers: Providers,
    pub(crate) secrets: Arc<dyn SecretStore>,
    pub(crate) shop: Option<Arc<CommerceShopFacts>>,
    pub(crate) integration: Option<String>,
    pub(crate) settings: SettingsSource,
    pub(crate) events: Arc<dyn EventSink>,
    /// The launch instant: the last change of every session until one is seen (MC-A6).
    pub(crate) launched: Instant,
    pub(crate) state: std::sync::Mutex<crate::runs::LiveState>,
}

impl ResearchService {
    pub fn new(deps: ResearchDeps) -> Self {
        let files = ResearchFiles::new(&deps.data_dir);
        let images = ResearchImages::new(deps.store.clone(), deps.imaging.clone(), files.clone());
        let gateway = CallGateway::new(
            deps.store.clone(),
            deps.clock.clone(),
            deps.providers.clone(),
            deps.secret_values.clone(),
            files.clone(),
            deps.wait.clone(),
        );
        Self {
            launched: deps.clock.mono(),
            store: deps.store,
            clock: deps.clock,
            images,
            gateway,
            files,
            providers: deps.providers,
            secrets: deps.secrets,
            shop: deps.shop,
            integration: deps.integration,
            settings: deps.settings,
            events: deps.events,
            state: std::sync::Mutex::new(crate::runs::LiveState::default()),
        }
    }

    pub fn gateway(&self) -> &CallGateway {
        &self.gateway
    }

    /// The research file store (research copies and listing images).
    pub fn files(&self) -> &ResearchFiles {
        &self.files
    }

    pub(crate) fn changed(&self, session_id: i64) {
        self.events.emit(AppEvent::ResearchChanged { session_id });
    }

    /// The external id of the item's shop listing, if it is linked.
    pub(crate) fn external_id(&self, item_id: i64) -> Result<Option<String>> {
        self.store.research_transaction(|tx| {
            use rusqlite::OptionalExtension;
            tx.conn()
                .query_row(
                    "SELECT external_id FROM item_links WHERE item_id=?1 ORDER BY integration LIMIT 1",
                    [item_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(crate::store::db)
        })
    }

    /// `fields_filled` and `to_review` of the status (R4·U6).
    pub(crate) fn derived_counts(
        &self,
        _item_id: i64,
        _latest: Option<&crate::store::RunRow>,
    ) -> Result<(usize, usize)> {
        Ok((0, 0))
    }

    /// Runs the stages of a `running` run (08A §5) and ends it (MC-A7 commit rule; MC-A8 endings).
    pub fn execute(&self, run_id: i64) -> Result<crate::stages::ExecuteOutcome> {
        crate::stages::execute(self, run_id)
    }
}
