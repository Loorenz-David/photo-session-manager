//! R10·U1 (A24·C1, C2): the shipped research defaults reach a run. "A fresh app" is the
//! research harness with the `research` settings slot absent (so `ResearchSettings::default()`
//! is what the service reads), the keys in `FakeSecrets`, the fake shop, and the crate's fakes
//! registered under the default provider ids.

use std::{collections::BTreeSet, sync::Arc};

use psm_core::{
    domain::photo::{Classification, NewPhoto, PhotoSource},
    events::{AppEvent, ExportState},
    mechanisms::hash::sha256_file,
    ports::{clock::Clock, secrets::SecretStore},
};
use psm_infra::settings_file::Settings;
use psm_research::{
    adapters::openai::usage::usage_of,
    config::{
        DEFAULT_MODEL_PROVIDER, DEFAULT_PAGE_READER, DEFAULT_VISUAL_SEARCH_PROVIDER,
        ResearchConfig, ResearchSettings,
    },
    domain::{RunState, RunTrigger, Stage},
    fakes::{FakeModel, FakePageReader, FakeVisualSearch},
    ports::{
        CallFailure, ModelProvider, ModelReply, ModelRequest, PageReader, PageReply, Providers,
        SearchReply, VisualSearchProvider,
    },
    rules::cost::parse_micro_usd,
    service::{MODEL_ACCOUNT, ResearchService, SEARCH_ACCOUNT},
};
use serde_json::json;

use crate::harness::Harness;

/// A fake answering under a default adapter id.
struct Named<T>(Arc<T>, &'static str);

impl ModelProvider for Named<FakeModel> {
    fn id(&self) -> String {
        self.1.to_owned()
    }
    fn call(&self, request: &ModelRequest) -> Result<ModelReply, CallFailure> {
        self.0.call(request)
    }
}

impl VisualSearchProvider for Named<FakeVisualSearch> {
    fn id(&self) -> String {
        self.1.to_owned()
    }
    fn search(
        &self,
        image: &psm_research::images::ResearchImage,
    ) -> Result<SearchReply, CallFailure> {
        self.0.search(image)
    }
}

impl PageReader for Named<FakePageReader> {
    fn id(&self) -> String {
        self.1.to_owned()
    }
    fn read(&self, url: &str) -> Result<PageReply, CallFailure> {
        self.0.read(url)
    }
}

/// A fresh app: no `research` settings slot at all, and the fakes under the default ids.
fn fresh_app() -> (Harness, ResearchService) {
    let h = Harness::chair();
    let mut settings = Settings::open(&h.app.app.paths).unwrap();
    settings.research = None;
    Settings::set(&h.app.app.paths, &settings).unwrap();
    let providers = Providers {
        model: Some(Arc::new(Named(h.model.clone(), DEFAULT_MODEL_PROVIDER))),
        search: Some(Arc::new(Named(
            h.search.clone(),
            DEFAULT_VISUAL_SEARCH_PROVIDER,
        ))),
        reader: Some(Arc::new(Named(h.reader.clone(), DEFAULT_PAGE_READER))),
    };
    let svc = h.service_with(|deps| deps.providers = providers);
    (h, svc)
}

/// A Main photo with a preview, known to the fake Capture One document (so it can be exported).
fn first_main_photo(h: &Harness, svc: &ResearchService) -> i64 {
    let dir = h.app.dir.path().join("Capture/raws");
    std::fs::create_dir_all(&dir).unwrap();
    let raw = dir.join("first.ARW");
    std::fs::write(&raw, "raw bytes of the first photo").unwrap();
    let photo = h
        .app
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id: h.session_id,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(h.app.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: h.app.clock.now_ms(),
        })
        .unwrap();
    h.app
        .store()
        .upsert_raw_state(&photo.raw_path, None)
        .unwrap();
    h.app
        .fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    h.app
        .review()
        .classify(photo.id, Classification::Main)
        .unwrap();
    h.give_preview(photo.id);
    svc.session_changed(h.session_id);
    photo.id
}

fn reason(svc: &ResearchService, session_id: i64) -> Option<(String, String)> {
    svc.status(session_id)
        .unwrap()
        .reason
        .map(|r| (r.code, r.sentence))
}

#[test]
fn a24_c1a_shipped_defaults_start_a_requested_run() {
    let (h, svc) = fresh_app();
    first_main_photo(&h, &svc);
    // The quiet period passes: automatic research is off by default, so no run.
    h.app.clock.advance(2_000);
    assert_eq!(svc.tick().unwrap(), None);
    assert!(h.runs(h.session_id).is_empty());
    // The photographer asks for a run; nothing in the settings was edited.
    svc.run_again(h.session_id).unwrap();
    let started = svc
        .tick()
        .unwrap()
        .expect("the requested run did not start");
    let runs = h.runs(h.session_id);
    assert_eq!(runs.len(), 1);
    let run = &runs[0];
    assert_eq!(
        (run.id, run.trigger, run.state),
        (started, RunTrigger::Manual, RunState::Running)
    );
    assert!(run.started_at.is_some());
    // The run executes from the shipped defaults.
    let snapshot = ResearchConfig::from_snapshot(run.settings_snapshot.as_deref()).unwrap();
    assert_eq!(snapshot.stages, ResearchSettings::default().stages);
    assert_eq!(
        (
            snapshot.providers.model.as_str(),
            snapshot.providers.visual_search.as_str(),
            snapshot.providers.page_reader.as_str()
        ),
        (
            DEFAULT_MODEL_PROVIDER,
            DEFAULT_VISUAL_SEARCH_PROVIDER,
            DEFAULT_PAGE_READER
        )
    );
    // No prerequisite is named: only "automatic research off", which has no panel sentence.
    assert_eq!(
        reason(&svc, h.session_id),
        Some(("AutoOff".to_owned(), String::new()))
    );
}

#[test]
fn a24_c1b_without_the_keys_no_run_and_export_works() {
    let (h, svc) = fresh_app();
    h.app.secrets.delete(MODEL_ACCOUNT).unwrap();
    h.app.secrets.delete(SEARCH_ACCOUNT).unwrap();
    first_main_photo(&h, &svc);
    h.app.clock.advance(2_000);
    let refused = svc.run_again(h.session_id).unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("Add the model key in Settings → Research"),
        "{refused}"
    );
    assert_eq!(svc.tick().unwrap(), None);
    assert!(h.runs(h.session_id).is_empty());
    assert_eq!(
        reason(&svc, h.session_id),
        Some((
            "NoModelKey".to_owned(),
            "Add the model key in Settings → Research".to_owned()
        ))
    );
    // The session's export still works.
    h.app.app.export.run(h.session_id).unwrap();
    h.app.complete_all_renders().unwrap();
    h.app.tick(0).unwrap();
    h.app.tick(130_000).unwrap();
    let last_export = h
        .app
        .events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::ExportProgress {
                done, total, state, ..
            } => Some((*done, *total, *state)),
            _ => None,
        })
        .next_back();
    assert_eq!(last_export, Some((1, 1, ExportState::Done)));
    assert!(
        h.app
            .export_root()
            .join("R4-1")
            .join("R4-1_01_main.jpg")
            .is_file()
    );
}

/// Every unit the OpenAI adapter can report: its usage mapping applied to a reply that carries
/// every usage field.
fn openai_units() -> BTreeSet<String> {
    let reply = json!({
        "usage": {
            "input_tokens": 100,
            "input_tokens_details": {"cached_tokens": 10, "cache_write_tokens": 10},
            "output_tokens": 5,
        },
        "tool_usage": {"web_search": {"num_requests": 1}},
    });
    usage_of(&reply)
        .unwrap()
        .into_iter()
        .map(|(unit, _)| unit)
        .collect()
}

#[test]
fn a24_c2a_defaults_are_complete_and_priced() {
    let settings = ResearchSettings::default();
    // Every stage that calls the model has a model and an effort (Find is the visual search).
    let model_stages = [
        Stage::Observe,
        Stage::Read,
        Stage::Compare,
        Stage::Labels,
        Stage::Condition,
        Stage::Decide,
    ];
    for stage in model_stages {
        let config = settings.stage(stage);
        assert!(!config.model.trim().is_empty(), "{stage:?} has no model");
        assert!(!config.effort.trim().is_empty(), "{stage:?} has no effort");
        assert!(config.image_long_edge > 0, "{stage:?} has no long edge");
    }
    // The (provider, model) pairs the defaults use and the units each adapter reports.
    let providers = settings.providers.resolved();
    let mut pairs: Vec<(String, String, BTreeSet<String>)> = model_stages
        .iter()
        .map(|s| {
            (
                providers.model.clone(),
                settings.stage(*s).model,
                openai_units(),
            )
        })
        .collect();
    pairs.push((
        providers.visual_search.clone(),
        String::new(),
        BTreeSet::from(["image_search_call".to_owned()]),
    ));
    pairs.push((
        providers.page_reader.clone(),
        String::new(),
        BTreeSet::from(["page_fetch".to_owned()]),
    ));
    assert!(openai_units().contains("cache_write_token"));
    let unit_prices = settings.unit_prices();
    let fallbacks = settings.fallbacks();
    for (provider, model, units) in &pairs {
        for unit in units {
            assert!(
                unit_prices
                    .iter()
                    .any(|p| &p.provider == provider && &p.model == model && &p.unit == unit),
                "no {unit} price for {provider} {model}"
            );
        }
        let fallback = fallbacks
            .iter()
            .find(|f| &f.provider == provider && &f.model == model)
            .unwrap_or_else(|| panic!("no fallback for {provider} {model}"));
        assert!(fallback.micro_usd > 0, "fallback of {provider} {model}");
    }
    // Every price text parses (none is dropped by the parser).
    for row in &settings.prices {
        for text in [&row.usd_per_million, &row.fallback_call_usd]
            .into_iter()
            .flatten()
        {
            assert!(
                parse_micro_usd(text).is_some(),
                "price {text} does not parse"
            );
        }
    }
    assert!(parse_micro_usd(&settings.max_cost_per_run).is_some_and(|c| c > 0));
    assert!(parse_micro_usd(&settings.max_cost_per_day).is_some_and(|c| c > 0));
    assert!(settings.record_days >= 2);
    assert!(psm_research::config::validate(&serde_json::to_value(&settings).unwrap()).is_ok());
}

#[test]
fn a24_c2b_attribution_keys_are_empty_by_default() {
    assert!(ResearchSettings::default().attribution_keys.is_empty());
}
