//! The conversations integration family (book 09B): the Beyo Pricing adapter and the app-side
//! seams of `ConversationsService` (`master_plan_conversations.md` §4 Settings, Secret, Service):
//! the settings slot over `settings.json`, the redaction-registering secret store, the test seam
//! and the wiring `App::build` uses.

pub mod beyo_pricing;

use beyo_pricing::{
    BeyoPricingAdapter, DEFAULT_SERVER, checked_server, effective_server,
    transport::{HttpReply, PricingHttp, PricingRequest, RawReply, ReqwestPricing},
};
use psm_core::{
    Error, Result,
    domain::conversation::ConversationSettings,
    ports::{
        clock::Clock, conversations::ConversationSettingsSlot, events::EventSink,
        secrets::SecretStore,
    },
    services::conversations::{ConversationsLane, ConversationsService},
};
use psm_infra::{paths::AppPaths, settings_file::Settings, sqlite::store::SqliteStore};
use serde_json::Value as Json;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock, Weak, atomic::AtomicBool, atomic::Ordering},
};

/// The stored form of a server setting: `None` for the default, the checked address otherwise;
/// `Err` when it fails `checked_server`.
fn stored_server(server: Option<&str>) -> std::result::Result<Option<String>, ()> {
    match server.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(raw) => match checked_server(raw) {
            Some(server) if server == DEFAULT_SERVER => Ok(None),
            Some(server) => Ok(Some(server)),
            None => Err(()),
        },
    }
}

/// Validates a `conversations` settings patch (the general settings path, as `research` and
/// `writer` are validated): a `server` that fails `checked_server` is refused naming `server`.
pub fn validate_patch(patch: &Json) -> Result<()> {
    if !patch.is_object() {
        return Err(Error::InvalidValue("conversations".into()));
    }
    match patch.get("server") {
        None | Some(Json::Null) => Ok(()),
        Some(Json::String(server)) => stored_server(Some(server))
            .map(|_| ())
            .map_err(|_| Error::InvalidValue("server".into())),
        Some(_) => Err(Error::InvalidValue("server".into())),
    }
}

/// The full normalised slot object for a merged slot value (default server stored as none).
pub fn normalised_slot(slot: Option<&Json>) -> Json {
    let mut settings = ConversationSettings::from_slot(slot);
    settings.server = stored_server(settings.server.as_deref()).unwrap_or(None);
    serde_json::to_value(settings).unwrap_or(Json::Null)
}

/// `ConversationSettingsSlot` over the `conversations` slot of `settings.json`.
pub struct SettingsFileSlot {
    paths: AppPaths,
    write: Mutex<()>,
}

impl SettingsFileSlot {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            write: Mutex::new(()),
        }
    }
}

impl ConversationSettingsSlot for SettingsFileSlot {
    fn read(&self) -> Option<Json> {
        let mut slot = Settings::open(&self.paths).ok()?.conversations?;
        if let Some(fields) = slot.as_object_mut() {
            let server = fields.get("server").and_then(Json::as_str);
            let stored = stored_server(server).unwrap_or(None);
            fields.insert("server".into(), stored.map_or(Json::Null, Json::String));
        }
        Some(slot)
    }

    fn write(&self, settings: ConversationSettings) -> Result<()> {
        let server = stored_server(settings.server.as_deref())
            .map_err(|_| Error::InvalidValue("server".into()))?;
        let normalised = ConversationSettings { server, ..settings };
        let _write = self.write.lock().unwrap_or_else(|e| e.into_inner());
        let mut file = Settings::open(&self.paths)?;
        file.conversations =
            Some(serde_json::to_value(normalised).map_err(|e| Error::InvalidData(e.to_string()))?);
        Settings::set(&self.paths, &file)
    }
}

/// The app's secret store with every conversations key value registered for log redaction: on
/// each `set`, and on the first `get` that returns a value (master plan §4 Secret).
pub struct RedactingSecrets {
    inner: Arc<dyn SecretStore>,
    registered: AtomicBool,
}

impl RedactingSecrets {
    pub fn new(inner: Arc<dyn SecretStore>) -> Self {
        Self {
            inner,
            registered: AtomicBool::new(false),
        }
    }
}

impl SecretStore for RedactingSecrets {
    fn get(&self, account: &str) -> Result<Option<String>> {
        let value = self.inner.get(account)?;
        if let Some(value) = &value
            && !self.registered.swap(true, Ordering::SeqCst)
        {
            psm_infra::logging::add_secret(value);
        }
        Ok(value)
    }
    fn cached(&self, account: &str) -> Option<String> {
        self.inner.cached(account)
    }
    fn set(&self, account: &str, value: &str) -> Result<()> {
        psm_infra::logging::add_secret(value);
        self.inner.set(account, value)
    }
    fn delete(&self, account: &str) -> Result<()> {
        self.inner.delete(account)
    }
}

/// A transport that never reaches anything (when the live HTTP worker cannot start).
struct NoTransport;
impl PricingHttp for NoTransport {
    fn send(&self, _: PricingRequest) -> Result<HttpReply> {
        Err(Error::Provider("http: unavailable".into()))
    }
    fn fetch(&self, _: PricingRequest, _: std::time::Duration) -> Result<RawReply> {
        Err(Error::Provider("http: unavailable".into()))
    }
}

type TestTransports = Mutex<HashMap<PathBuf, Arc<dyn PricingHttp>>>;
static TEST_TRANSPORTS: OnceLock<TestTransports> = OnceLock::new();

/// The suite's stand-in for Pricing for the app at `paths` (keyed by the data folder, survives a
/// reopen): no test ever reaches the network.
pub fn with_test_conversations(paths: &AppPaths, transport: Arc<dyn PricingHttp>) {
    if let Ok(mut all) = TEST_TRANSPORTS.get_or_init(Default::default).lock() {
        all.insert(paths.root.clone(), transport);
    }
}

fn transport_for(paths: &AppPaths) -> Arc<dyn PricingHttp> {
    if let Some(transport) = TEST_TRANSPORTS
        .get()
        .and_then(|all| all.lock().ok()?.get(&paths.root).cloned())
    {
        return transport;
    }
    match ReqwestPricing::new() {
        Ok(transport) => Arc::new(transport),
        Err(error) => {
            tracing::warn!(%error, "conversations transport not available");
            Arc::new(NoTransport)
        }
    }
}

/// The pieces `App::build` hands the conversations wiring.
pub struct ServiceParts {
    pub paths: AppPaths,
    pub store: Arc<SqliteStore>,
    pub secrets: Arc<dyn SecretStore>,
    pub shop: Option<String>,
    /// C4: the connected integration's kind (`settings.integration.kind`), for the form's keys.
    pub integration: Option<String>,
    pub clock: Arc<dyn Clock>,
    pub events: Arc<dyn EventSink>,
    pub lane: Arc<ConversationsLane>,
}

/// Builds the service over the Beyo Pricing adapter: the adapter reads the live effective server
/// from the slot and the key the service holds in memory. The same adapter instance is handed
/// back for the labels port (one instance, two ports: the label calls use the same live server
/// and key closures).
pub fn build_service(parts: ServiceParts) -> (Arc<ConversationsService>, Arc<BeyoPricingAdapter>) {
    let slot = Arc::new(SettingsFileSlot::new(parts.paths.clone()));
    let service_cell: Arc<OnceLock<Weak<ConversationsService>>> = Arc::new(OnceLock::new());
    let server_slot = slot.clone();
    let key_cell = service_cell.clone();
    let adapter = BeyoPricingAdapter::new(
        transport_for(&parts.paths),
        move || {
            effective_server(
                ConversationSettings::from_slot(server_slot.read().as_ref())
                    .server
                    .as_deref(),
            )
        },
        move || {
            key_cell
                .get()
                .and_then(Weak::upgrade)
                .and_then(|service| service.held_key())
        },
    );
    let adapter = Arc::new(adapter);
    let service = Arc::new(ConversationsService::new(
        parts.store,
        adapter.clone(),
        Arc::new(RedactingSecrets::new(parts.secrets)),
        slot,
        parts.shop,
        parts.integration,
        parts.clock,
        parts.events,
        parts.lane,
    ));
    let _ = service_cell.set(Arc::downgrade(&service));
    (service, adapter)
}
