//! The research harness: a `TestApp` with a fake shop and a real SQLite store, the engine's
//! tables (`schema::ensure`), the crate's fakes, `FakeSecrets` with distinctive key values, one
//! item of type Sideboard and the definitions the rows need. Later units extend it.

#![allow(dead_code)]

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use psm_core::{
    domain::{
        attribute::{
            AiConfidence, AttributeGroup, CoreType, Definition, Preset, PresetField, Value,
        },
        composition::{CompositionValues, Origin, Rect},
        photo::Classification,
        research::{Feedback, FeedbackAction},
    },
    ports::secrets::SecretStore,
};
use psm_infra::{imaging::ImageOpsImpl, sqlite::store::SqliteStore};
use psm_research::{
    domain::{RunState, RunTrigger},
    fakes::{FakeModel, FakePageReader, FakeVisualSearch},
    files::ResearchFiles,
    images::ResearchImages,
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
    next_colour: AtomicU8,
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
            .import_definitions(INTEGRATION, definitions())
            .unwrap();
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
        Harness {
            item_id: item.id,
            session_id: session.session.id,
            app,
            model: Arc::new(FakeModel::default()),
            search: Arc::new(FakeVisualSearch::default()),
            reader: Arc::new(FakePageReader::default()),
            images,
            next_colour: AtomicU8::new(0),
        }
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
        definitions().into_iter().find(|d| d.key == key).unwrap()
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
