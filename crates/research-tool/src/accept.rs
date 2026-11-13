//! `accept --pieces <file> --data <scratch dir> --ceiling-usd <n> [--only <n,…>] [--settings <file>]`
//! (R9·U2 task 1; 08A §11 operational measurement).
//!
//! Opens the real app headless on a scratch data folder: the real `App`, the real research engine
//! and its real adapters (chosen by the saved settings), keys from the environment handed to an
//! in-memory secrets store (never the Keychain). The shop is [`StandInShop`], a read-only
//! `CommerceAdapter` inside this tool: the shop definitions fixture, the pieces file's public
//! listings, and every mutating method refusing (H1: this tool writes nothing to a shop). For
//! each piece an item and a session are created through the services, its images are added as
//! photos whose adjusted preview is the downloaded image (first image = cover, condition report
//! images as Condition photos), the engine runs the session (manual trigger) and the run is
//! exported (R7·U2). Everything is written under the scratch folder.

use psm_core::{
    Error,
    domain::{
        attribute::{AttributeGroup, Definition, Preset, PresetField},
        photo::{Classification, NewPhoto, PhotoSource},
        publish::{
            ConfirmedRun, DefinitionLink, MediaReadback, RemoteIdentity, RemoteItem, RemoteMedia,
            RemoteValues, StagedTarget, WriteValue,
        },
    },
    ports::{
        clock::{Clock, SystemClock},
        commerce::{CommerceAdapter, PublicListing},
        images::Imaging,
        secrets::SecretStore,
        store::Store,
    },
    services::{attributes::AttributesService, items::ItemsService, review::ReviewService},
};
use psm_infra::{paths::AppPaths, settings_file::Settings};
use psm_research::{
    api::ExportSelection,
    rules::write_table::never_written,
    service::{MODEL_ACCOUNT, SEARCH_ACCOUNT},
};
use serde_json::{Value as Json, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    answer_key::{Piece, load_pieces},
    budget::format_usd,
    http::{HttpSeam, Wire},
    recording::sha256_hex,
};

/// The integration name the app files the shop's definitions under.
pub const INTEGRATION: &str = "shopify";
/// The environment variables the keys come from.
pub const MODEL_KEY_VARIABLE: &str = "OPENAI_API_KEY";
pub const SEARCH_KEY_VARIABLE: &str = "SERPAPI_API_KEY";
/// The long edge of the adjusted preview made from a downloaded shop image.
pub const PREVIEW_LONG_EDGE: u32 = 2048;
/// How long one run may take before the tool stops waiting for it.
pub const RUN_TIMEOUT: Duration = Duration::from_secs(30 * 60);
/// The attribution keys of the scratch settings: the shop definitions fixture's designer and
/// manufacturer fields (08A §6 "the shop's designer and manufacturer keys").
pub const ATTRIBUTION_KEYS: [&str; 2] = ["custom.designer", "custom.manufacturer"];
/// Run states after which nothing more happens.
pub const FINAL_STATES: [&str; 4] = ["finished", "failed", "stopped_at_cap", "cancelled"];

/// The owner's trusted sites in priority order (evidence doc; pamono country sites 2026-11-13).
pub fn trusted_sites() -> Vec<String> {
    crate::answer_key::trusted_sites()
}

/// A secrets store in memory: the keys of this run only, never the Keychain.
#[derive(Default)]
pub struct MemorySecrets(Mutex<BTreeMap<String, String>>);

impl SecretStore for MemorySecrets {
    fn get(&self, account: &str) -> psm_core::Result<Option<String>> {
        Ok(self.0.lock().ok().and_then(|m| m.get(account).cloned()))
    }
    fn cached(&self, account: &str) -> Option<String> {
        self.0.lock().ok().and_then(|m| m.get(account).cloned())
    }
    fn set(&self, account: &str, value: &str) -> psm_core::Result<()> {
        if let Ok(mut m) = self.0.lock() {
            m.insert(account.to_owned(), value.to_owned());
        }
        Ok(())
    }
    fn delete(&self, account: &str) -> psm_core::Result<()> {
        if let Ok(mut m) = self.0.lock() {
            m.remove(account);
        }
        Ok(())
    }
}

/// The shop as the acceptance run sees it: definitions from the fixture, public listings from
/// the pieces file, and no way to change anything.
pub struct StandInShop {
    definitions: Vec<Definition>,
    hosts: Vec<String>,
    /// Published pieces: shop product id → (URL, handle).
    published: BTreeMap<String, (String, String)>,
    item_types: Vec<String>,
}

fn refused(what: &str) -> Error {
    Error::Provider(format!(
        "{what}: the acceptance tool writes nothing to a shop (H1)"
    ))
}

impl StandInShop {
    pub fn new(pieces: &[Piece], raw: &[Json], definitions: Vec<Definition>) -> Self {
        let mut hosts: Vec<String> = Vec::new();
        let mut published = BTreeMap::new();
        for (piece, json) in pieces.iter().zip(raw) {
            if !piece.published {
                continue;
            }
            if let Some(host) = psm_research::rules::sites::host(&piece.url)
                && !hosts.contains(&host)
            {
                hosts.push(host);
            }
            if let Some(id) = json["id"].as_str() {
                published.insert(id.to_owned(), (piece.url.clone(), piece.handle.clone()));
            }
        }
        let item_types: BTreeSet<String> = pieces.iter().map(|p| p.product_type.clone()).collect();
        Self {
            definitions,
            hosts,
            published,
            item_types: item_types.into_iter().collect(),
        }
    }
}

impl CommerceAdapter for StandInShop {
    fn public_listing(&self, external_id: &str) -> psm_core::Result<PublicListing> {
        let listing = self.published.get(external_id);
        Ok(PublicListing {
            hosts: self.hosts.clone(),
            item_url: listing.map(|(url, _)| url.clone()),
            item_path_tail: listing
                .map(|(_, handle)| format!("products/{handle}"))
                .unwrap_or_default(),
        })
    }
    fn check_token(&self) -> psm_core::Result<()> {
        Ok(())
    }
    fn capabilities(&self) -> Vec<&'static str> {
        vec!["attributes.schema"]
    }
    fn condition_photos_key(&self) -> &str {
        psm_testkit::fixtures::definitions::DAMAGE_REPORT
    }
    fn item_search(&self, _sku: &str) -> psm_core::Result<Vec<RemoteItem>> {
        Ok(Vec::new())
    }
    fn item_find(&self, _query: &str) -> psm_core::Result<Vec<RemoteItem>> {
        Ok(Vec::new())
    }
    fn item_get(&self, external_id: &str) -> psm_core::Result<RemoteItem> {
        Err(Error::NotFound(format!("product {external_id}")))
    }
    fn item_types(&self) -> psm_core::Result<Vec<String>> {
        Ok(self.item_types.clone())
    }
    fn product_fields(&self, external_id: &str) -> psm_core::Result<(Vec<String>, Option<String>)> {
        Err(Error::NotFound(format!("product {external_id}")))
    }
    fn product_values(&self, external_id: &str) -> psm_core::Result<RemoteValues> {
        Err(Error::NotFound(format!("product {external_id}")))
    }
    fn render_description(&self, paragraphs: &str) -> String {
        paragraphs.to_owned()
    }
    fn item_create(
        &self,
        _run: &ConfirmedRun,
        _sku: &str,
        _barcode: Option<&str>,
        _title: &str,
        _item_type: &str,
    ) -> psm_core::Result<String> {
        Err(refused("item_create"))
    }
    fn item_set_identity(
        &self,
        _run: &ConfirmedRun,
        _external_id: &str,
        _sku: &str,
        _barcode: Option<&str>,
    ) -> psm_core::Result<RemoteIdentity> {
        Err(refused("item_set_identity"))
    }
    fn attributes_schema(&self) -> psm_core::Result<Vec<(Definition, DefinitionLink)>> {
        Ok(self
            .definitions
            .iter()
            .map(|d| {
                (
                    d.clone(),
                    DefinitionLink {
                        platform_type: String::new(),
                        platform_meta: json!({}),
                    },
                )
            })
            .collect())
    }
    fn attributes_read(&self, _external_id: &str) -> psm_core::Result<Vec<(String, Json)>> {
        Ok(Vec::new())
    }
    fn condition_photos_readback(&self, _external_id: &str) -> psm_core::Result<Option<Json>> {
        Ok(None)
    }
    fn attributes_write(
        &self,
        _run: &ConfirmedRun,
        _external_id: &str,
        _values: &[WriteValue],
    ) -> psm_core::Result<()> {
        Err(refused("attributes_write"))
    }
    fn staged_uploads_create(
        &self,
        _run: &ConfirmedRun,
        _stem: &str,
        _byte_size: u64,
    ) -> psm_core::Result<StagedTarget> {
        Err(refused("staged_uploads_create"))
    }
    fn upload(
        &self,
        _run: &ConfirmedRun,
        _target: &StagedTarget,
        _bytes: &[u8],
    ) -> psm_core::Result<()> {
        Err(refused("upload"))
    }
    fn product_create_media(
        &self,
        _run: &ConfirmedRun,
        _external_id: &str,
        _resource_url: &str,
        _alt: &str,
    ) -> psm_core::Result<String> {
        Err(refused("product_create_media"))
    }
    fn poll_media_ready(&self, _media_id: &str) -> psm_core::Result<()> {
        Err(refused("poll_media_ready"))
    }
    fn product_reorder_media(
        &self,
        _run: &ConfirmedRun,
        _external_id: &str,
        _media_ids: &[String],
    ) -> psm_core::Result<String> {
        Err(refused("product_reorder_media"))
    }
    fn poll_job(&self, _job_id: &str) -> psm_core::Result<()> {
        Err(refused("poll_job"))
    }
    fn list_media(&self, _external_id: &str) -> psm_core::Result<Vec<RemoteMedia>> {
        Ok(Vec::new())
    }
    fn media_readback(&self, _external_id: &str) -> psm_core::Result<MediaReadback> {
        Ok(MediaReadback {
            gallery: Vec::new(),
            condition: Vec::new(),
        })
    }
    fn fetch_image(&self, url: &str, _max_edge: u32) -> psm_core::Result<Vec<u8>> {
        Err(Error::NotFound(url.to_owned()))
    }
    fn list_files_by_name(&self, _stem: &str) -> psm_core::Result<Vec<RemoteMedia>> {
        Ok(Vec::new())
    }
    fn product_delete_media(
        &self,
        _run: &ConfirmedRun,
        _external_id: &str,
        _media_ids: &[String],
    ) -> psm_core::Result<()> {
        Err(refused("product_delete_media"))
    }
    fn file_create(
        &self,
        _run: &ConfirmedRun,
        _resource_url: &str,
        _filename: &str,
    ) -> psm_core::Result<String> {
        Err(refused("file_create"))
    }
    fn poll_file(&self, _file_id: &str) -> psm_core::Result<()> {
        Err(refused("poll_file"))
    }
    fn update_title(
        &self,
        _run: &ConfirmedRun,
        _external_id: &str,
        _title: &str,
    ) -> psm_core::Result<()> {
        Err(refused("update_title"))
    }
    fn update_item_type(
        &self,
        _run: &ConfirmedRun,
        _external_id: &str,
        _item_type: &str,
    ) -> psm_core::Result<()> {
        Err(refused("update_item_type"))
    }
    fn update_description(
        &self,
        _run: &ConfirmedRun,
        _external_id: &str,
        _paragraphs: &str,
    ) -> psm_core::Result<()> {
        Err(refused("update_description"))
    }
    fn update_tags(
        &self,
        _run: &ConfirmedRun,
        _external_id: &str,
        _tags: &[String],
    ) -> psm_core::Result<()> {
        Err(refused("update_tags"))
    }
}

/// The shop definitions fixture without its six `test.*` definitions (they stand for core types
/// the shop has no definition of; they are not the shop's).
pub fn shop_definitions() -> Vec<Definition> {
    psm_testkit::fixtures::definitions::shop_definitions()
        .into_iter()
        .filter(|d| !d.key.starts_with("test."))
        .collect()
}

/// The layout of every item type: the fixture has no presets, so every researchable definition
/// (MC-A1: not never-written) in its own step, except the condition photos field.
pub fn layout(definitions: &[Definition], group: AttributeGroup) -> Vec<PresetField> {
    definitions
        .iter()
        .filter(|d| d.group == group)
        .filter(|d| d.key != psm_testkit::fixtures::definitions::DAMAGE_REPORT)
        .filter(|d| !never_written(d, Some(d.group)))
        .map(|d| PresetField {
            key: d.key.clone(),
            default: None,
        })
        .collect()
}

/// The scratch settings: models per stage from the evidence doc's Owner picks (observe
/// `gpt-6.1-sol` low, read `gpt-6.1-terra` low, condition `gpt-6.1-terra` high; the stages the
/// picks do not name take the vision pick for images and the read pick for text), prices from
/// R1's `prices.json`, caps 0.50 per run and 25 per day, manual runs only, the owner's trusted sites,
/// the fixture's designer and manufacturer as attribution keys. `patch` replaces top-level keys
/// (`stages` per stage).
pub fn scratch_settings(patch: Option<&Json>) -> Result<Json, String> {
    let stage = |model: &str, effort: &str, edge: u32| json!({"model": model, "effort": effort, "image_long_edge": edge});
    let mut prices = Vec::new();
    for (model, input, cached, write, output) in [
        ("gpt-6.1-sol", "2.00", "0.10", "2.50", "10.00"),
        ("gpt-6.1-terra", "1.00", "0.05", "1.25", "5.00"),
        ("gpt-6-luna", "0.10", "0.01", "0.125", "0.50"),
    ] {
        for (unit, usd) in [
            ("input_token", input),
            ("cached_input_token", cached),
            ("cache_write_token", write),
            ("output_token", output),
            ("web_search_call", "10000.00"),
        ] {
            prices.push(json!({"provider": "openai", "model": model, "unit": unit,
                               "usd_per_million": usd}));
        }
        prices.push(json!({"provider": "openai", "model": model, "fallback_call_usd": "0.05"}));
    }
    prices.push(
        json!({"provider": "serpapi_lens", "model": "", "unit": "image_search_call",
                       "usd_per_million": "0", "fallback_call_usd": "0"}),
    );
    prices.push(
        json!({"provider": "page_http", "model": "", "unit": "page_fetch",
                       "usd_per_million": "0", "fallback_call_usd": "0"}),
    );
    let mut settings = json!({
        "auto": false,
        "quiet_seconds": 1,
        "source_wait_seconds": 120,
        "attribution_keys": ATTRIBUTION_KEYS,
        "trusted_sites": trusted_sites(),
        "max_pages": 10,
        "stages": {
            "observe": stage("gpt-6.1-sol", "low", 1024),
            "find": stage("", "", 1024),
            "read": stage("gpt-6.1-terra", "low", 1024),
            "compare": stage("gpt-6.1-sol", "low", 1024),
            "labels": stage("gpt-6.1-sol", "low", 2048),
            "condition": stage("gpt-6.1-terra", "high", 2048),
            "decide": stage("gpt-6.1-terra", "low", 1024),
        },
        "max_cost_per_run": "0.50",
        "max_cost_per_day": "25",
        "prices": prices,
        "language": "English",
        "record_days": 30,
        "call_timeout_seconds": {"model": 180, "other": 60},
        "providers": {"model": "openai", "visual_search": "serpapi_lens",
                      "page_reader": "page_http"},
    });
    if let Some(Json::Object(patch)) = patch {
        for (key, value) in patch {
            match (key.as_str(), value) {
                ("stages", Json::Object(stages)) => {
                    for (name, stage) in stages {
                        settings["stages"][name] = stage.clone();
                    }
                }
                _ => settings[key] = value.clone(),
            }
        }
    }
    psm_research::config::validate(&settings).map_err(|e| format!("scratch settings: {e}"))?;
    Ok(settings)
}

/// What `accept` was asked to do.
#[derive(Debug, Clone)]
pub struct AcceptOptions {
    pub pieces: PathBuf,
    pub data: PathBuf,
    pub ceiling_micro: i64,
    pub only: Option<Vec<u32>>,
    pub settings_patch: Option<Json>,
    pub run_timeout: Duration,
}

/// One piece's result.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PieceRun {
    pub n: u32,
    pub item_id: i64,
    pub session_id: i64,
    pub run_id: Option<i64>,
    pub state: String,
    pub cost_micro_usd: i64,
    pub seconds: u64,
    /// The zip of the run's export (R7·U2), under `<data>/exports/<n>-<handle>/`.
    pub export: Option<PathBuf>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AcceptOutcome {
    pub pieces: Vec<PieceRun>,
    pub spent_micro_usd: i64,
    /// Why the tool stopped before the last piece, if it did.
    pub stopped: Option<String>,
}

/// The SKU of a piece's item in the scratch store.
pub fn sku(n: u32) -> String {
    format!("A23-{n:02}")
}

fn io(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Downloads `url` once into `<data>/source/<sha prefix>.<ext>` (through the tool's HTTP seam).
fn download(seam: &dyn HttpSeam, data: &Path, url: &str) -> Result<PathBuf, String> {
    let ext = url
        .rsplit('.')
        .next()
        .filter(|e| e.len() <= 4 && e.bytes().all(|b| b.is_ascii_alphanumeric()))
        .unwrap_or("img")
        .to_ascii_lowercase();
    let path = data
        .join("source")
        .join(format!("{}.{ext}", &sha256_hex(url.as_bytes())[..24]));
    if path.is_file() {
        return Ok(path);
    }
    let reply = seam
        .send(&Wire {
            method: "GET".into(),
            url: url.to_owned(),
            headers: Vec::new(),
            body: Vec::new(),
            timeout: Duration::from_secs(120),
            follow_redirects: true,
        })
        .map_err(|e| format!("download of {url} failed: {}", e.text))?;
    if reply.status != 200 {
        return Err(format!("download of {url}: HTTP {}", reply.status));
    }
    std::fs::create_dir_all(data.join("source")).map_err(io)?;
    std::fs::write(&path, &reply.body).map_err(io)?;
    Ok(path)
}

/// The sum of every run's recorded cost in the scratch store.
fn spent(store: &psm_infra::sqlite::store::SqliteStore) -> Result<i64, String> {
    store
        .research_transaction(|tx| psm_research::store::runs(tx.conn(), None))
        .map(|runs| runs.iter().map(|r| r.cost_micro_usd).sum())
        .map_err(io)
}

fn run_cost(store: &psm_infra::sqlite::store::SqliteStore, run_id: i64) -> i64 {
    store
        .research_transaction(|tx| psm_research::store::run(tx.conn(), run_id))
        .ok()
        .flatten()
        .map(|r| r.cost_micro_usd)
        .unwrap_or(0)
}

/// Runs the acceptance over the pieces. Keys come only from `env`; nothing is written outside
/// `opts.data`. Progress lines go to `log`.
pub fn run(
    opts: &AcceptOptions,
    seam: &dyn HttpSeam,
    env: &dyn Fn(&str) -> Option<String>,
    log: &mut dyn Write,
) -> Result<AcceptOutcome, String> {
    let key = |name: &str| {
        env(name)
            .map(|k| k.trim().to_owned())
            .filter(|k| !k.is_empty())
            .ok_or(format!("{name} is not set; nothing was started"))
    };
    let model_key = key(MODEL_KEY_VARIABLE)?;
    let search_key = key(SEARCH_KEY_VARIABLE)?;
    let text = std::fs::read_to_string(&opts.pieces)
        .map_err(|e| format!("cannot read {}: {e}", opts.pieces.display()))?;
    let raw: Vec<Json> = serde_json::from_str::<Json>(&text)
        .map_err(|e| format!("pieces file: {e}"))?
        .as_array()
        .cloned()
        .unwrap_or_default();
    let all = load_pieces(&opts.pieces)?;
    let chosen: Vec<(Piece, Json)> = all
        .iter()
        .cloned()
        .zip(raw.iter().cloned())
        .filter(|(p, _)| opts.only.as_ref().is_none_or(|only| only.contains(&p.n)))
        .collect();
    let settings = scratch_settings(opts.settings_patch.as_ref())?;
    let per_run = psm_research::rules::cost::parse_micro_usd(
        settings["max_cost_per_run"].as_str().unwrap_or("0.50"),
    )
    .unwrap_or(500_000);

    std::fs::create_dir_all(&opts.data).map_err(io)?;
    let data = opts.data.canonicalize().map_err(io)?;
    let paths = AppPaths::for_dir(&data);
    paths.create().map_err(io)?;
    let mut file_settings = Settings::open(&paths).map_err(io)?;
    file_settings.research = Some(settings);
    Settings::set(&paths, &file_settings).map_err(io)?;

    let secrets = Arc::new(MemorySecrets::default());
    secrets.set(MODEL_ACCOUNT, &model_key).map_err(io)?;
    secrets.set(SEARCH_ACCOUNT, &search_key).map_err(io)?;
    let definitions = shop_definitions();
    let pieces: Vec<Piece> = chosen.iter().map(|(p, _)| p.clone()).collect();
    let jsons: Vec<Json> = chosen.iter().map(|(_, j)| j.clone()).collect();
    let shop = Arc::new(StandInShop::new(&pieces, &jsons, definitions.clone()));
    let editor = data.join("editor");
    let scripting = Arc::new(psm_testkit::fake_capture_one::FakeCaptureOneScripting::new(
        editor.to_string_lossy().into_owned(),
        editor.join("Capture").to_string_lossy().into_owned(),
    ));
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let mut app = psm_integrations::App::open_with_adapter(
        paths.clone(),
        scripting,
        clock.clone(),
        false,
        Some(shop.clone() as Arc<dyn CommerceAdapter>),
        secrets as Arc<dyn SecretStore>,
    )
    .map_err(io)?;
    let outcome = run_pieces(
        opts,
        seam,
        log,
        &mut app,
        &data,
        &chosen,
        &definitions,
        per_run,
    );
    app.shutdown();
    let outcome = outcome?;
    let summary = serde_json::to_vec_pretty(&outcome).map_err(io)?;
    std::fs::write(data.join("accept.json"), summary).map_err(io)?;
    Ok(outcome)
}

#[allow(clippy::too_many_arguments)]
fn run_pieces(
    opts: &AcceptOptions,
    seam: &dyn HttpSeam,
    log: &mut dyn Write,
    app: &mut psm_integrations::App,
    data: &Path,
    chosen: &[(Piece, Json)],
    definitions: &[Definition],
    per_run: i64,
) -> Result<AcceptOutcome, String> {
    let store = app.store.clone();
    let clock = app.clock.clone();
    let research = app.research.clone().ok_or("research is not available")?;
    let shop = app
        .commerce
        .clone()
        .ok_or("the stand-in shop was not taken")?;
    let attributes = AttributesService::new(store.as_ref(), clock.as_ref(), Some(INTEGRATION));
    attributes.refresh_schema(shop.as_ref()).map_err(io)?;
    let types: BTreeSet<&str> = chosen
        .iter()
        .map(|(p, _)| p.product_type.as_str())
        .collect();
    for item_type in types {
        for group in [AttributeGroup::Details, AttributeGroup::Condition] {
            attributes
                .presets_set(Preset {
                    item_type: item_type.to_owned(),
                    group,
                    layout: layout(definitions, group),
                })
                .map_err(io)?;
        }
    }
    let items = ItemsService::new(store.as_ref(), clock.as_ref());
    let review = ReviewService::new(store.as_ref(), clock.as_ref(), app.history.as_ref());
    let mut existing: BTreeSet<String> = BTreeSet::new();
    store
        .transaction(&mut |tx| {
            existing = tx.items()?.into_iter().map(|i| i.sku).collect();
            Ok(())
        })
        .map_err(io)?;
    let mut outcome = AcceptOutcome::default();
    for (piece, json) in chosen {
        if existing.contains(&sku(piece.n)) {
            let _ = writeln!(
                log,
                "piece {}: already in the scratch store, skipped",
                piece.n
            );
            continue;
        }
        let spent_now = spent(&store)?;
        if spent_now + per_run > opts.ceiling_micro {
            outcome.stopped = Some(format!(
                "refused before piece {}: spent USD {} + one run's cap USD {} exceeds the ceiling USD {}",
                piece.n,
                format_usd(spent_now),
                format_usd(per_run),
                format_usd(opts.ceiling_micro)
            ));
            break;
        }
        let started = Instant::now();
        let (item, session) = items
            .create_local(&sku(piece.n), &piece.title, &piece.product_type, None)
            .map_err(io)?;
        let session_id = session.session.id;
        if piece.published
            && let Some(id) = json["id"].as_str()
        {
            items
                .link(item.id, INTEGRATION, id, Some(&piece.url))
                .map_err(io)?;
        }
        let mut row = PieceRun {
            n: piece.n,
            item_id: item.id,
            session_id,
            run_id: None,
            state: String::new(),
            cost_micro_usd: 0,
            seconds: 0,
            export: None,
            error: None,
        };
        let photos = piece
            .images
            .iter()
            .map(|u| (u, Classification::Main))
            .chain(
                piece
                    .condition_images
                    .iter()
                    .map(|u| (u, Classification::Condition)),
            );
        let mut added = 0;
        for (i, (url, class)) in photos.enumerate() {
            match add_photo(app, &review, data, seam, piece.n, session_id, i, url, class) {
                Ok(()) => added += 1,
                Err(e) => {
                    let _ = writeln!(log, "piece {}: photo {i} left out: {e}", piece.n);
                }
            }
        }
        if added == 0 {
            row.error = Some("no photo could be added".into());
            outcome.pieces.push(row);
            continue;
        }
        research.run(session_id).map_err(io)?;
        let deadline = Instant::now() + opts.run_timeout;
        loop {
            app.research_tick(clock.now_ms()).map_err(io)?;
            let status = research.status(session_id).map_err(io)?;
            if let Some(run) = &status.run {
                row.run_id = Some(run.id);
                row.state = run.state.clone();
                row.error = run.error.clone();
                if FINAL_STATES.contains(&run.state.as_str()) {
                    break;
                }
            }
            if Instant::now() >= deadline {
                row.error = Some(format!(
                    "the run did not end within {} s",
                    opts.run_timeout.as_secs()
                ));
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        if let Some(run_id) = row.run_id {
            row.cost_micro_usd = run_cost(&store, run_id);
            let dest = data
                .join("exports")
                .join(format!("{}-{}", piece.n, piece.handle));
            std::fs::create_dir_all(&dest).map_err(io)?;
            match research.export(ExportSelection::Items(vec![item.id]), &dest) {
                Ok(zip) => row.export = Some(zip),
                Err(e) => row.error = Some(format!("export: {e}")),
            }
        }
        row.seconds = started.elapsed().as_secs();
        let _ = writeln!(
            log,
            "piece {}: run {} {} USD {} in {} s{}",
            piece.n,
            row.run_id.map_or("-".to_owned(), |r| r.to_string()),
            row.state,
            format_usd(row.cost_micro_usd),
            row.seconds,
            row.error
                .as_deref()
                .map(|e| format!(" ({e})"))
                .unwrap_or_default()
        );
        outcome.pieces.push(row);
    }
    outcome.spent_micro_usd = spent(&store)?;
    Ok(outcome)
}

/// One photo of a piece: a placeholder RAW in the scratch folder, the classification, and the
/// downloaded image (reduced to [`PREVIEW_LONG_EDGE`] by the app's own thumbnail function, H3) as
/// its completed adjusted preview.
#[allow(clippy::too_many_arguments)]
fn add_photo(
    app: &psm_integrations::App,
    review: &ReviewService<'_>,
    data: &Path,
    seam: &dyn HttpSeam,
    n: u32,
    session_id: i64,
    index: usize,
    url: &str,
    class: Classification,
) -> Result<(), String> {
    let source = download(seam, data, url)?;
    let piece_dir = data.join("pieces").join(n.to_string());
    std::fs::create_dir_all(&piece_dir).map_err(io)?;
    let preview = piece_dir.join(format!("preview-{index:02}.jpg"));
    if !preview.is_file() {
        let made = psm_infra::imaging::ImageOpsImpl
            .thumbnail(&source, &preview, PREVIEW_LONG_EDGE)
            .map_err(io)?;
        if !made {
            return Err(format!("{url} could not be reduced"));
        }
    }
    let raw = piece_dir.join(format!("IMG_{index:04}.ARW"));
    std::fs::write(&raw, format!("acceptance placeholder for {url}")).map_err(io)?;
    let raw_path = raw.to_string_lossy().into_owned();
    let now = app.clock.now_ms();
    let photo = app
        .store
        .photo_insert_or_get(NewPhoto {
            session_id,
            raw_path: raw_path.clone(),
            raw_hash: Some(sha256_hex(url.as_bytes())),
            captured_at: Some(now + index as i64),
            source: PhotoSource::Capture,
            created_at: now + index as i64,
        })
        .map_err(io)?;
    review.classify(photo.id, class).map_err(io)?;
    app.store.adjusted_event(&raw_path, false).map_err(io)?;
    let generation = app.store.adjusted_gen(&raw_path).map_err(io)?;
    app.store
        .complete_preview(
            &raw_path,
            generation,
            &preview.to_string_lossy(),
            None,
            None,
        )
        .map_err(io)?;
    Ok(())
}

/// The piece number of an export's item folder (`<item id>-A23-<n>`).
fn piece_of(item_dir: &str) -> Option<u32> {
    item_dir.rsplit_once("A23-")?.1.parse().ok()
}

fn median(mut values: Vec<i64>) -> i64 {
    values.sort_unstable();
    match values.len() {
        0 => 0,
        n if n % 2 == 1 => values[n / 2],
        n => (values[n / 2 - 1] + values[n / 2]) / 2,
    }
}

/// `score-accept --data <dir> [--pieces <file>] [--corrections <file>]` (R9·U2 tasks 2 and 4):
/// every exported run of the scratch folder scored against the key, and the numbers of 08A §11
/// per group (published / unpublished): attribution at medium or high (correct, of which hedged,
/// wrong), at low, not determined, empty-key attributions; visible and condition fields
/// (owner-verified and corrected key apart); USD and seconds per piece (median, maximum); run
/// states; every disagreement with its evidence. Written to `<data>/scores.json`.
pub fn score_data(
    data: &Path,
    pieces_file: &Path,
    corrections_file: &Path,
) -> Result<Json, String> {
    use crate::score::{FieldGroup, Outcome, score_run};
    let pieces = load_pieces(pieces_file)?;
    let corrections = crate::answer_key::load_corrections(corrections_file)?;
    let entries = crate::answer_key::key_entries(&pieces, &corrections);
    let definitions = shop_definitions();
    let mut zips: Vec<PathBuf> = Vec::new();
    for dir in std::fs::read_dir(data.join("exports"))
        .map_err(io)?
        .flatten()
    {
        for file in std::fs::read_dir(dir.path()).map_err(io)?.flatten() {
            if file.path().extension().is_some_and(|e| e == "zip") {
                zips.push(file.path());
            }
        }
    }
    zips.sort();
    let mut scores = Vec::new();
    let mut runs_json = Vec::new();
    let mut groups: BTreeMap<&str, BTreeMap<String, i64>> = BTreeMap::new();
    let mut money: BTreeMap<&str, (Vec<i64>, Vec<i64>)> = BTreeMap::new();
    for zip in &zips {
        let opened = crate::export_reader::open(zip)?;
        for run in opened.runs() {
            let Some(n) = piece_of(&run.item_dir) else {
                continue;
            };
            let Some(piece) = pieces.iter().find(|p| p.n == n) else {
                continue;
            };
            let group = if piece.published {
                "published"
            } else {
                "unpublished"
            };
            let keys: Vec<String> = run.run["settings_snapshot"]["attribution_keys"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|k| k.as_str().map(str::to_owned))
                .collect();
            let fields = score_run(n, &run.writes, &run.steps, &entries, &definitions, &keys);
            let counts = groups.entry(group).or_default();
            let state = run.run["run"]["state"].as_str().unwrap_or("").to_owned();
            *counts.entry(format!("runs {state}")).or_default() += 1;
            for f in &fields {
                let outcome = match f.outcome {
                    Outcome::Correct => "correct",
                    Outcome::Wrong => "wrong",
                    Outcome::NotDetermined => "not determined",
                };
                let name = match (f.group, f.corrected) {
                    (FieldGroup::Attribution, None) => {
                        format!("attribution, key empty: {outcome}")
                    }
                    (FieldGroup::Attribution, Some(_)) if f.outcome == Outcome::NotDetermined => {
                        "attribution: not determined".to_owned()
                    }
                    (FieldGroup::Attribution, Some(corrected)) => {
                        let level = match f.confidence.as_deref() {
                            Some("low") => "low",
                            _ => "medium or high",
                        };
                        let bucket = if corrected { " (corrected key)" } else { "" };
                        let hedged = if f.hedged && f.outcome == Outcome::Correct {
                            ", hedged"
                        } else {
                            ""
                        };
                        format!("attribution at {level}{bucket}: {outcome}{hedged}")
                    }
                    (g, corrected) => format!(
                        "{} ({}): {outcome}",
                        if g == FieldGroup::Condition {
                            "condition"
                        } else {
                            "visible"
                        },
                        if corrected == Some(true) {
                            "corrected key"
                        } else {
                            "owner-verified"
                        }
                    ),
                };
                *counts.entry(name).or_default() += 1;
            }
            let cost = run.run["run"]["cost_micro_usd"].as_i64().unwrap_or(0);
            let ms = match (
                run.run["run"]["started_at"].as_i64(),
                run.run["run"]["finished_at"].as_i64(),
            ) {
                (Some(a), Some(b)) => (b - a).max(0),
                _ => 0,
            };
            let m = money.entry(group).or_default();
            m.0.push(cost);
            m.1.push(ms);
            runs_json.push(
                json!({"n": n, "run": run.id(), "group": group, "state": state,
                "error": run.run["run"]["error"], "cost_micro_usd": cost, "ms": ms}),
            );
            scores.extend(fields);
        }
    }
    let per_piece: BTreeMap<&str, Json> = money
        .into_iter()
        .map(|(g, (cost, ms))| {
            (
                g,
                json!({
                    "pieces": cost.len(),
                    "usd_median": format_usd(median(cost.clone())),
                    "usd_max": format_usd(cost.iter().copied().max().unwrap_or(0)),
                    "seconds_median": median(ms.clone()) / 1000,
                    "seconds_max": ms.iter().copied().max().unwrap_or(0) / 1000,
                }),
            )
        })
        .collect();
    let candidates: Vec<&crate::score::FieldScore> =
        scores.iter().filter(|s| s.review_candidate).collect();
    let out = json!({
        "groups": groups,
        "per_piece": per_piece,
        "runs": runs_json,
        "review_candidates": candidates,
        "fields": scores,
    });
    std::fs::write(
        data.join("scores.json"),
        serde_json::to_vec_pretty(&out).map_err(io)?,
    )
    .map_err(io)?;
    Ok(out)
}
