//! The research harness: a `TestApp` with a fake shop and a real SQLite store, the engine's
//! tables (`schema::ensure`), the crate's fakes, `FakeSecrets` with distinctive key values, one
//! item of type Sideboard and the definitions the rows need. Later units extend it.

#![allow(dead_code)]

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use psm_core::ports::commerce::CommerceAdapter;
use psm_core::{
    domain::{
        attribute::{
            AiConfidence, AttributeGroup, CoreType, Definition, Preset, PresetField, Value,
        },
        composition::{CompositionValues, Origin, Rect},
        photo::Classification,
        research::{Feedback, FeedbackAction},
    },
    ports::{secrets::SecretStore, store::Store},
};
use psm_infra::{
    imaging::ImageOpsImpl, jobs::JobRunner, settings_file::Settings, sqlite::store::SqliteStore,
};
use psm_research::{
    config::{FallbackSnapshot, PriceSnapshot, ResearchConfig},
    domain::{RunState, RunTrigger},
    fakes::{FakeModel, FakePageReader, FakeVisualSearch},
    files::ResearchFiles,
    gateway::{CallGateway, SecretValues, Waiter},
    images::ResearchImages,
    job::ResearchJob,
    ports::Providers,
    service::{ResearchDeps, ResearchService, form_snapshot},
    shop::CommerceShopFacts,
    stages::ExecuteOutcome,
    store::{self as rstore, NewRun},
    writes::{self, WriteOutcome},
};
use psm_testkit::{TestApp, fixtures::synthetic_jpeg::SyntheticJpeg};
use serde_json::{Value as Json, json};

pub const INTEGRATION: &str = "shop";
pub const T0: i64 = 1_761_350_400_000;

/// Distinctive key values that appear in no expected text.
pub const MODEL_KEY: &str = "sk-psm-MODEL+key/7f3a9c";
pub const SEARCH_KEY: &str = "srch-psm-KEY-55e1d0";
pub const SHOP_SECRET: &str = "shpss-psm-SECRET-9b2c";

/// Text field in the Details step.
pub const MAKER: &str = "custom.maker";
/// Choice field `[Oak, Teak]` in the Details step.
pub const WOOD: &str = "custom.wood";
/// A Details definition left out of the item type's layout.
pub const OUTSIDE: &str = "test.outside";
/// A text definition of the Product step.
pub const SUBTITLE: &str = "custom.subtitle";
pub const MEDIA: &str = "test.media";
pub const REFERENCE: &str = "test.reference";
pub const RICH: &str = "test.rich";
pub const MONEY: &str = "test.money";
pub const RATING: &str = "test.rating";
/// A text definition with `writable = false`.
pub const LOCKED: &str = "test.locked";

/// Writes the `research` slot of the settings file.
pub fn save_research(paths: &psm_infra::paths::AppPaths, value: Json) {
    let mut settings = Settings::open(paths).unwrap();
    settings.research = Some(value);
    Settings::set(paths, &settings).unwrap();
}

fn build_service(
    app: &TestApp,
    model: &Arc<FakeModel>,
    search: &Arc<FakeVisualSearch>,
    reader: &Arc<FakePageReader>,
    edit: impl FnOnce(&mut ResearchDeps),
) -> ResearchService {
    let secrets = app.secrets.clone();
    let paths = app.app.paths.clone();
    let clock = app.clock.clone();
    let mut deps = ResearchDeps {
        store: app.app.store.clone(),
        clock: app.clock.clone(),
        imaging: Arc::new(ImageOpsImpl),
        data_dir: app.app.paths.root.clone(),
        providers: Providers {
            model: Some(model.clone()),
            search: Some(search.clone()),
            reader: Some(reader.clone()),
        },
        secrets: app.secrets.clone(),
        secret_values: Arc::new(move || {
            let mut out: Vec<String> = secrets.values.lock().unwrap().values().cloned().collect();
            out.push(SHOP_SECRET.to_owned());
            out
        }),
        shop: app
            .commerce
            .clone()
            .map(|c| Arc::new(CommerceShopFacts::new(c as Arc<dyn CommerceAdapter>))),
        integration: Some(INTEGRATION.to_owned()),
        settings: Arc::new(move || Settings::open(&paths).ok().and_then(|s| s.research)),
        events: app.events.clone(),
        wait: Arc::new(move |ms| clock.advance(i64::try_from(ms).unwrap())),
    };
    edit(&mut deps);
    ResearchService::new(deps)
}

/// The stage rows' layout (item type Chair): an attribution text field, a multi-choice visible
/// field, a dimension, a severity choice in the Condition step; the Product step holds the title.
pub const DESIGNER: &str = "custom.designer";
pub const WOODS: &str = "custom.woods";
pub const WIDTH: &str = "custom.width";
pub const SCRATCHES: &str = "custom.scratches";
pub const TITLE: &str = "product.title";
pub const SHOP_HOST: &str = "beyo.example";
pub const E1: &str = "e1.example";
pub const E2: &str = "e2.example";

fn stage_definitions() -> Vec<Definition> {
    use AttributeGroup::{Condition, Details};
    use CoreType as T;
    let mut width = def(WIDTH, T::Dimension, Details, &[]);
    width.validations = json!({"units": ["CENTIMETERS"], "unit_names": {"cm": "CENTIMETERS"}});
    vec![
        def(DESIGNER, T::Text, Details, &[]),
        def(WOODS, T::MultiChoice, Details, &["Oak", "Teak"]),
        width,
        def(
            SCRATCHES,
            T::Choice,
            Condition,
            &psm_core::mechanisms::values::SEVERITY_SET,
        ),
    ]
}

fn def(key: &str, core_type: CoreType, group: AttributeGroup, choices: &[&str]) -> Definition {
    Definition {
        integration: INTEGRATION.into(),
        key: key.into(),
        label: key.into(),
        core_type,
        choices: choices.iter().map(|c| c.to_string()).collect(),
        validations: json!({}),
        group,
        description: None,
        writable: key != LOCKED,
    }
}

fn definitions() -> Vec<Definition> {
    use AttributeGroup::{Details, Product};
    use CoreType as T;
    vec![
        def(MAKER, T::Text, Details, &[]),
        def(WOOD, T::Choice, Details, &["Oak", "Teak"]),
        def(OUTSIDE, T::Text, Details, &[]),
        def(SUBTITLE, T::Text, Product, &[]),
        def(MEDIA, T::MediaList, Details, &[]),
        def(REFERENCE, T::Reference, Details, &[]),
        def(RICH, T::RichText, Details, &[]),
        def(MONEY, T::Money, Details, &[]),
        def(RATING, T::Rating, Details, &[]),
        def(LOCKED, T::Text, Details, &[]),
    ]
}

pub struct Harness {
    pub app: TestApp,
    pub item_id: i64,
    pub session_id: i64,
    pub model: Arc<FakeModel>,
    pub search: Arc<FakeVisualSearch>,
    pub reader: Arc<FakePageReader>,
    pub images: ResearchImages,
    pub service: Arc<ResearchService>,
    pub runner: JobRunner,
    next_colour: AtomicU8,
}

/// Research settings under which every prerequisite passes: the fakes' ids, model `m1` for every
/// stage, the test prices (with a 0.001 USD fallback per provider), trusted `[e1, e2]`, the
/// designer as the attribution key.
pub fn test_settings() -> Json {
    let stages: serde_json::Map<String, Json> = psm_research::domain::Stage::ALL
        .into_iter()
        .map(|s| {
            let edge = match s {
                psm_research::domain::Stage::Labels | psm_research::domain::Stage::Condition => {
                    2048
                }
                _ => 1024,
            };
            (
                s.as_str().to_owned(),
                json!({"model": "m1", "effort": "low", "image_long_edge": edge}),
            )
        })
        .collect();
    json!({
        "auto": true,
        "quiet_seconds": 1,
        "source_wait_seconds": 120,
        "attribution_keys": [DESIGNER],
        "trusted_sites": [E1, E2],
        "max_pages": 10,
        "stages": stages,
        "max_cost_per_run": "0.50",
        "max_cost_per_day": "15",
        "prices": [
            {"provider": FakeModel::ID, "model": "m1", "unit": "input_token",
             "usd_per_million": "1", "fallback_call_usd": "0.001"},
            {"provider": FakeModel::ID, "model": "m1", "unit": "output_token",
             "usd_per_million": "2"},
            {"provider": FakeVisualSearch::ID, "model": "", "fallback_call_usd": "0.001"},
            {"provider": FakePageReader::ID, "model": "", "fallback_call_usd": "0.001"},
        ],
        "language": "English",
        "providers": {"model": FakeModel::ID, "visual_search": FakeVisualSearch::ID,
                      "page_reader": FakePageReader::ID},
    })
}

fn runner_for(app: &TestApp, service: &Arc<ResearchService>) -> JobRunner {
    let mut runner =
        JobRunner::new(app.app.store.clone() as Arc<dyn Store>).only_kind(psm_research::job::KIND);
    runner.register(
        psm_research::job::KIND,
        Arc::new(ResearchJob::new(service.clone())),
    );
    runner
}

impl Harness {
    pub fn new() -> Self {
        let app = TestApp::with_fake_shop().unwrap();
        app.clock.advance(T0);
        app.secrets.set("research-model-key", MODEL_KEY).unwrap();
        app.secrets.set("research-search-key", SEARCH_KEY).unwrap();
        psm_research::schema::ensure(&app.app.store).unwrap();
        let attributes = app.attributes(Some(INTEGRATION));
        attributes
            .import_definitions(
                INTEGRATION,
                definitions()
                    .into_iter()
                    .chain(stage_definitions())
                    .collect(),
            )
            .unwrap();
        for (group, keys) in [
            (AttributeGroup::Details, vec![DESIGNER, WOODS, WIDTH]),
            (AttributeGroup::Condition, vec![SCRATCHES]),
        ] {
            attributes
                .presets_set(Preset {
                    item_type: "Chair".into(),
                    group,
                    layout: keys
                        .into_iter()
                        .map(|key| PresetField {
                            key: key.into(),
                            default: None,
                        })
                        .collect(),
                })
                .unwrap();
        }
        let details: Vec<PresetField> = definitions()
            .into_iter()
            .filter(|d| d.group == AttributeGroup::Details && d.key != OUTSIDE)
            .map(|d| PresetField {
                key: d.key,
                default: None,
            })
            .collect();
        attributes
            .presets_set(Preset {
                item_type: "Sideboard".into(),
                group: AttributeGroup::Details,
                layout: details,
            })
            .unwrap();
        let (item, session) = app
            .items()
            .create_local("R3-1", "Oak sideboard", "Sideboard", None)
            .unwrap();
        let images = ResearchImages::new(
            app.app.store.clone(),
            Arc::new(ImageOpsImpl),
            ResearchFiles::new(&app.app.paths.root),
        );
        let model = Arc::new(FakeModel::default());
        let search = Arc::new(FakeVisualSearch::default());
        let reader = Arc::new(FakePageReader::default());
        save_research(&app.app.paths, test_settings());
        let service = Arc::new(build_service(&app, &model, &search, &reader, |_| {}));
        let runner = runner_for(&app, &service);
        Harness {
            item_id: item.id,
            session_id: session.session.id,
            app,
            model,
            search,
            reader,
            images,
            service,
            runner,
            next_colour: AtomicU8::new(0),
        }
    }

    /// Moves the manual clock, starts what may start, then runs the research jobs that are due.
    pub fn tick(&self, ms: i64) {
        self.app.clock.advance(ms);
        self.service.tick().unwrap();
        self.run_jobs();
    }

    pub fn run_jobs(&self) {
        self.runner.tick(self.now()).unwrap();
    }

    /// The app's launch over the same store: running jobs re-queued, a new service and runner
    /// (the launch instant is now).
    pub fn relaunch(&mut self) {
        self.app.store().requeue_running_jobs().unwrap();
        self.service = Arc::new(build_service(
            &self.app,
            &self.model,
            &self.search,
            &self.reader,
            |_| {},
        ));
        self.runner = runner_for(&self.app, &self.service);
    }

    /// Another service over the same app with some deps changed.
    pub fn service_with(&self, edit: impl FnOnce(&mut ResearchDeps)) -> ResearchService {
        build_service(&self.app, &self.model, &self.search, &self.reader, edit)
    }

    /// The call the app makes after a photo change in the session.
    pub fn changed(&self) {
        self.service.session_changed(self.session_id);
    }

    /// Updates the live research settings with `patch` (top-level keys), as a settings save does.
    pub fn patch_settings(&self, patch: Json) {
        let mut settings = self.research_settings().unwrap_or_else(test_settings);
        for (k, v) in patch.as_object().unwrap() {
            settings[k] = v.clone();
        }
        self.set_research_settings(settings);
    }

    pub fn runs(&self, session_id: i64) -> Vec<rstore::RunRow> {
        self.query(|c| rstore::runs(c, Some(session_id)).unwrap())
    }

    pub fn all_runs(&self) -> Vec<rstore::RunRow> {
        self.query(|c| rstore::runs(c, None).unwrap())
    }

    /// A harness whose item is a Chair (the stage rows' layout) with its own session.
    pub fn chair() -> Self {
        let mut h = Self::new();
        let (item, session) = h
            .app
            .items()
            .create_local("R4-1", "Oak chair", "Chair", None)
            .unwrap();
        h.item_id = item.id;
        h.session_id = session.session.id;
        h
    }

    /// The live `research` settings slot of the settings file.
    pub fn research_settings(&self) -> Option<Json> {
        Settings::open(&self.app.app.paths).unwrap().research
    }

    /// Saves the live `research` settings slot to the settings file.
    pub fn set_research_settings(&self, value: Json) {
        save_research(&self.app.app.paths, value);
    }

    /// The stage rows' config: the test prices, the shop host, trusted `[e1, e2]`, the designer
    /// as the attribution key, English.
    pub fn stage_config(&self) -> ResearchConfig {
        let mut config = self.config();
        let facts = CommerceShopFacts::new(self.commerce())
            .facts(self.external_id().as_deref())
            .unwrap();
        config.shop_hosts = if facts.hosts.is_empty() {
            vec![SHOP_HOST.into()]
        } else {
            facts.hosts
        };
        config.own_listing_url = facts.own_listing.url;
        config.own_listing_path_tail = facts.own_listing.path_tail;
        config.trusted_sites = vec![E1.into(), E2.into()];
        config.attribution_keys = vec![DESIGNER.into()];
        config
    }

    pub fn commerce(&self) -> Arc<dyn CommerceAdapter> {
        self.app.commerce.clone().unwrap()
    }

    /// The item's shop link id, if it is linked.
    pub fn external_id(&self) -> Option<String> {
        self.query(|c| {
            use rusqlite::OptionalExtension;
            c.query_row(
                "SELECT external_id FROM item_links WHERE item_id=?1",
                [self.item_id],
                |r| r.get(0),
            )
            .optional()
            .unwrap()
        })
    }

    /// Links the item to a shop product.
    pub fn link_item(&self, external_id: &str) {
        let item = self.item_id;
        self.query(|c| {
            c.execute(
                "INSERT INTO item_links(item_id,integration,external_id) VALUES (?1,'shopify',?2)",
                rusqlite::params![item, external_id],
            )
            .unwrap()
        });
    }

    /// A `running` run with `config` and the item's form as its snapshots (what the start
    /// transaction writes from R4·U4 on).
    pub fn start_run(&self, config: &ResearchConfig) -> i64 {
        let form = form_snapshot(
            self.store(),
            self.app.clock.as_ref(),
            Some(INTEGRATION),
            self.item_id,
        )
        .unwrap();
        self.insert_run(
            RunState::Running,
            Some(serde_json::to_value(config).unwrap()),
            Some(serde_json::to_value(&form).unwrap()),
        )
    }

    pub fn execute(&self, run_id: i64) -> ExecuteOutcome {
        self.service.execute(run_id).unwrap()
    }

    pub fn steps(&self, run_id: i64) -> Vec<rstore::StepRow> {
        self.query(|c| rstore::steps(c, run_id).unwrap())
    }

    pub fn outputs(&self, run_id: i64) -> Vec<rstore::OutputRow> {
        self.query(|c| rstore::outputs(c, run_id).unwrap())
    }

    pub fn store(&self) -> &Arc<SqliteStore> {
        &self.app.app.store
    }

    pub fn files(&self) -> ResearchFiles {
        ResearchFiles::new(&self.app.app.paths.root)
    }

    /// Every research copy file on disk.
    pub fn research_files(&self) -> Vec<std::path::PathBuf> {
        let root = self.app.app.paths.root.join("research").join("images");
        let mut out = Vec::new();
        let Ok(dirs) = std::fs::read_dir(&root) else {
            return out;
        };
        for dir in dirs.flatten() {
            if dir.path().is_dir() {
                for file in std::fs::read_dir(dir.path()).unwrap().flatten() {
                    out.push(file.path());
                }
            }
        }
        out.sort();
        out
    }

    /// A new item (type Sideboard) and its session.
    pub fn new_session(&self, sku: &str) -> (i64, i64) {
        let (item, session) = self
            .app
            .items()
            .create_local(sku, "Another piece", "Sideboard", None)
            .unwrap();
        (item.id, session.session.id)
    }

    /// A photo in `session`, classified as given, with no adjusted preview yet.
    pub fn add_photo_without_preview(&self, session: i64, class: Classification) -> i64 {
        let n = self.next_colour.fetch_add(1, Ordering::SeqCst);
        let photo = self
            .app
            .add_photo(session, &format!("IMG_{n:04}.ARW"))
            .unwrap();
        if class != Classification::Unreviewed {
            self.app.review().classify(photo.id, class).unwrap();
        }
        photo.id
    }

    /// Gives the photo an adjusted preview for a new `adjusted_gen` (a distinct solid colour, so
    /// every photo's research copy has its own bytes); `original` also gives it a camera preview.
    pub fn give_preview(&self, photo_id: i64) {
        let photo = self.photo(photo_id);
        self.app
            .store()
            .adjusted_event(&photo.raw_path, false)
            .unwrap();
        let generation = self.app.store().adjusted_gen(&photo.raw_path).unwrap();
        let n = self.next_colour.fetch_add(1, Ordering::SeqCst);
        let colour = [
            (n.wrapping_mul(37)) % 250,
            (n.wrapping_mul(91)) % 250,
            40 + n % 200,
        ];
        let path = self
            .app
            .dir
            .path()
            .join(format!("adjusted-{photo_id}-g{generation}.jpg"));
        std::fs::write(&path, SyntheticJpeg::rgb(1600, 1200, colour)).unwrap();
        self.app
            .store()
            .complete_preview(
                &photo.raw_path,
                generation,
                path.to_str().unwrap(),
                None,
                None,
            )
            .unwrap();
    }

    pub fn add_photo_with_preview(&self, session: i64, class: Classification) -> i64 {
        let id = self.add_photo_without_preview(session, class);
        self.give_preview(id);
        id
    }

    pub fn add_main_photo_with_preview(&self, session: i64) -> i64 {
        self.add_photo_with_preview(session, Classification::Main)
    }

    /// Writes an original camera preview path on the photo row (never a research source).
    pub fn give_original_preview(&self, photo_id: i64) {
        let path = self.app.dir.path().join(format!("original-{photo_id}.jpg"));
        std::fs::write(&path, SyntheticJpeg::rgb(800, 600, [10, 200, 10])).unwrap();
        self.store()
            .research_transaction(|tx| {
                tx.conn()
                    .execute(
                        "UPDATE photos SET original_preview_path=?1 WHERE id=?2",
                        rusqlite::params![path.to_str().unwrap(), photo_id],
                    )
                    .map(|_| ())
                    .map_err(|e| psm_core::Error::InvalidData(e.to_string()))
            })
            .unwrap();
    }

    pub fn photo(&self, photo_id: i64) -> psm_core::domain::photo::Photo {
        let mut out = None;
        self.app
            .store()
            .transaction(&mut |tx| {
                out = Some(tx.photo(photo_id)?);
                Ok(())
            })
            .unwrap();
        out.unwrap()
    }

    pub fn classify(&self, photo_id: i64, class: Classification) {
        self.app.review().classify(photo_id, class).unwrap();
    }

    /// Re-frames the photo (composition revision + 1): the left half, or the whole frame again.
    pub fn recompose(&self, photo_id: i64) {
        let current = self.app.composition(photo_id).unwrap();
        let half = current.rect.is_none_or(|r| r.w > 0.9);
        let values = CompositionValues {
            kind: current.kind,
            shape: current.shape,
            orientation: current.orientation,
            quarter_turns: current.quarter_turns,
            flip: current.flip,
            straighten: current.straighten,
            rect: Rect {
                x: 0.0,
                y: 0.0,
                w: if half { 0.5 } else { 1.0 },
                h: 1.0,
            },
        };
        self.app
            .store()
            .save_composition(photo_id, &values, Origin::User, self.now())
            .unwrap();
    }

    /// Inserts a run row for the harness's session in `state`.
    pub fn insert_run(&self, state: RunState, settings: Option<Json>, form: Option<Json>) -> i64 {
        self.insert_run_for(self.item_id, self.session_id, state, settings, form)
    }

    pub fn insert_run_for(
        &self,
        item_id: i64,
        session_id: i64,
        state: RunState,
        settings: Option<Json>,
        form: Option<Json>,
    ) -> i64 {
        let now = self.now();
        self.store()
            .research_transaction(|tx| {
                rstore::insert_run(
                    tx.conn(),
                    &NewRun {
                        item_id,
                        session_id,
                        trigger: RunTrigger::Auto,
                        state,
                        queued_at: now,
                        started_at: (state == RunState::Running).then_some(now),
                        settings_snapshot: settings.as_ref().map(Json::to_string),
                        form_snapshot: form.as_ref().map(Json::to_string),
                        input_snapshot: None,
                    },
                )
            })
            .unwrap()
    }

    /// The providers: the crate's three fakes.
    pub fn providers(&self) -> Providers {
        Providers {
            model: Some(self.model.clone()),
            search: Some(self.search.clone()),
            reader: Some(self.reader.clone()),
        }
    }

    /// Every value the secrets port holds, plus the shop's secret.
    pub fn secret_values(&self) -> SecretValues {
        let secrets = self.app.secrets.clone();
        Arc::new(move || {
            let mut out: Vec<String> = secrets.values.lock().unwrap().values().cloned().collect();
            out.push(SHOP_SECRET.to_owned());
            out
        })
    }

    /// Waits between attempts by moving the manual clock.
    pub fn waiter(&self) -> Waiter {
        let clock = self.app.clock.clone();
        Arc::new(move |ms| clock.advance(i64::try_from(ms).unwrap()))
    }

    pub fn gateway(&self) -> CallGateway {
        CallGateway::new(
            self.store().clone(),
            self.app.clock.clone(),
            self.providers(),
            self.secret_values(),
            self.files(),
            self.waiter(),
        )
    }

    /// The test price table: `m1` at 1 micro-USD per input token and 2 per output token, every
    /// provider with a fallback of 1 000 micro-USD per call; cap 0.50 USD.
    pub fn config(&self) -> ResearchConfig {
        let mut config = ResearchConfig::default();
        for stage in psm_research::domain::Stage::ALL {
            let entry = config.stages.get_mut(stage.as_str()).unwrap();
            entry.model = "m1".into();
            entry.effort = "low".into();
        }
        config.prices = vec![
            PriceSnapshot {
                provider: FakeModel::ID.into(),
                model: "m1".into(),
                unit: "input_token".into(),
                micro_usd_per_million: 1_000_000,
            },
            PriceSnapshot {
                provider: FakeModel::ID.into(),
                model: "m1".into(),
                unit: "output_token".into(),
                micro_usd_per_million: 2_000_000,
            },
        ];
        config.fallbacks = [
            (FakeModel::ID, "m1"),
            (FakeVisualSearch::ID, ""),
            (FakePageReader::ID, ""),
        ]
        .into_iter()
        .map(|(provider, model)| FallbackSnapshot {
            provider: provider.into(),
            model: model.into(),
            micro_usd: 1_000,
        })
        .collect();
        config.providers.model = FakeModel::ID.into();
        config.providers.visual_search = FakeVisualSearch::ID.into();
        config.providers.page_reader = FakePageReader::ID.into();
        config
    }

    /// A `running` run of the harness's session with `config` as its settings snapshot.
    pub fn running_run(&self, config: &ResearchConfig) -> i64 {
        self.insert_run(
            RunState::Running,
            Some(serde_json::to_value(config).unwrap()),
            None,
        )
    }

    pub fn run_row(&self, run_id: i64) -> rstore::RunRow {
        self.query(|c| rstore::run(c, run_id).unwrap().unwrap())
    }

    pub fn calls(&self, run_id: i64) -> Vec<rstore::CallRow> {
        self.query(|c| rstore::calls(c, Some(run_id)).unwrap())
    }

    pub fn outbound(&self, run_id: i64) -> Vec<rstore::OutboundRow> {
        self.query(|c| rstore::outbound(c, Some(run_id)).unwrap())
    }

    /// Sets a run's recorded cost directly.
    pub fn set_run_cost(&self, run_id: i64, micro_usd: i64) {
        self.query(|c| {
            c.execute(
                "UPDATE research_runs SET cost_micro_usd=?1 WHERE id=?2",
                rusqlite::params![micro_usd, run_id],
            )
            .unwrap()
        });
    }

    /// Runs a read-only query on the research connection.
    pub fn query<T>(&self, op: impl FnOnce(&rusqlite::Connection) -> T) -> T {
        let mut op = Some(op);
        let mut out = None;
        self.store()
            .research_transaction(|tx| {
                out = Some((op.take().unwrap())(tx.conn()));
                Ok(())
            })
            .unwrap();
        out.unwrap()
    }

    pub fn now(&self) -> i64 {
        use psm_core::ports::clock::Clock;
        self.app.clock.now_ms()
    }

    pub fn advance(&self, ms: i64) {
        self.app.clock.advance(ms);
    }

    pub fn definition(&self, key: &str) -> Definition {
        definitions()
            .into_iter()
            .chain(stage_definitions())
            .find(|d| d.key == key)
            .unwrap()
    }

    pub fn suggest_with(
        &self,
        key: &str,
        value: Json,
        confidence: AiConfidence,
        reason: Option<&str>,
        sources: &[String],
    ) -> WriteOutcome {
        writes::suggest(
            self.app.store(),
            self.app.clock.as_ref(),
            Some(INTEGRATION),
            self.item_id,
            key,
            &value,
            confidence,
            reason,
            sources,
        )
        .unwrap()
    }

    pub fn suggest(&self, key: &str, value: Json) -> WriteOutcome {
        self.suggest_with(
            key,
            value,
            AiConfidence::Medium,
            Some("seen on two sites"),
            &[],
        )
    }

    pub fn lower(&self, key: &str, reason: &str) -> WriteOutcome {
        writes::lower(
            self.app.store(),
            self.app.clock.as_ref(),
            self.item_id,
            key,
            reason,
        )
        .unwrap()
    }

    /// Stores a row straight through the store.
    pub fn put(&self, row: &Value) {
        self.app
            .store()
            .transaction(&mut |tx| tx.put_attribute_value(row))
            .unwrap();
    }

    /// The stored row of `key`, cleared rows included.
    pub fn row(&self, key: &str) -> Option<Value> {
        let mut out = None;
        self.app
            .store()
            .transaction(&mut |tx| {
                out = tx
                    .attribute_values(self.item_id)?
                    .into_iter()
                    .find(|v| v.key == key);
                Ok(())
            })
            .unwrap();
        out
    }

    /// Inserts a feedback row for `key` with the given action and suggested value.
    pub fn feedback(&self, key: &str, action: FeedbackAction, suggested: Json) {
        let feedback = Feedback {
            id: 0,
            item_id: self.item_id,
            key: key.into(),
            suggested_value: suggested.to_string(),
            confidence: AiConfidence::Medium,
            final_value: None,
            action,
            at: self.now(),
            step_token: None,
        };
        self.app
            .store()
            .transaction(&mut |tx| tx.insert_feedback(&feedback).map(|_| ()))
            .unwrap();
    }

    /// The session's activity time.
    pub fn activity(&self) -> i64 {
        self.app
            .store()
            .session(self.session_id)
            .unwrap()
            .session
            .last_activity_at
    }

    pub fn title(&self) -> String {
        self.app.store().item(self.item_id).unwrap().title
    }
}
