//! R9·U2 (rows A23·C1(a), C1(b), C2(a)): the acceptance scoring on the owner's real test file and
//! corrections, and `accept` over a small pieces file with the real app, the real research
//! adapters over `FakeResearchHttp` replaying R1's captures, and image downloads from a stub.

use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    sync::Arc,
};

use psm_core::ports::{secrets::SecretStore, store::Store};
use psm_integrations::{ResearchTestSeams, with_test_research};
use psm_research::{
    adapters::{
        http::HttpReply,
        openai::{OpenAiModel, adapter::RESPONSES_URL},
        page_reader::HttpPageReader,
        visual_search::{SerpApiLens, serpapi_lens},
    },
    fakes::{FakeResearchHttp, reply_from_capture},
    files::ResearchFiles,
    ports::Providers,
};
use psm_research_tool::{
    accept::{self, AcceptOptions},
    answer_key::{self, KeyEntry},
    http::{HttpSeam, Wire, WireError, WireReply},
    score::{self, Outcome, Suggestion},
};
use psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg;
use serde_json::{Value as Json, json};

use crate::repo_root;

fn entries() -> Vec<KeyEntry> {
    let root = repo_root();
    let pieces = answer_key::load_pieces(&root.join(answer_key::TEST_FILE)).unwrap();
    let corrections =
        answer_key::load_corrections(&root.join(answer_key::CORRECTIONS_FILE)).unwrap();
    answer_key::key_entries(&pieces, &corrections)
}

fn suggested(value: &str, confidence: &str) -> Suggestion {
    Suggestion {
        value: json!(value),
        confidence: confidence.to_owned(),
        sources: vec!["https://www.pamono.com/x".to_owned()],
    }
}

#[test]
fn a23_c1a_hedged_attribution_rule() {
    let entries = entries();
    // Piece 2 is hedged ("most likely by"); its key names Magnus Olesen as the designer.
    let designer = entries
        .iter()
        .find(|e| e.n == 2 && e.field == "designer")
        .unwrap();
    assert!(designer.hedged);
    let at = |confidence: &str| {
        score::score_attribution(
            Some(designer),
            Some(&suggested("Magnus Olesen", confidence)),
        )
    };
    assert_eq!(
        (at("medium"), at("low"), at("high")),
        (Outcome::Correct, Outcome::Correct, Outcome::Wrong)
    );
}

fn writes_with_designer(value: Option<&str>) -> Json {
    let writes = match value {
        Some(v) => json!({"custom.designer": {
            "value": v, "confidence": "medium", "reason": "Two trusted dealers name him.",
            "sources": ["https://www.pamono.com/a", "https://auctionet.com/b"], "outcome": "written"}}),
        None => json!({}),
    };
    json!({"writes": writes, "not_determined": {}, "ending": "finished"})
}

#[test]
fn a23_c1b_attribution_the_key_leaves_empty() {
    let entries = entries();
    // Piece 27: no attribution in the key.
    assert!(!entries.iter().any(|e| e.n == 27 && e.field == "designer"));
    let definitions = accept::shop_definitions();
    let keys: Vec<String> = accept::ATTRIBUTION_KEYS
        .iter()
        .map(|k| (*k).to_owned())
        .collect();
    let steps = vec![json!({"seq": 4, "stage": "read", "kind": "read",
        "summary": "Designer: Hans Wegner", "source_url": "https://www.pamono.com/a",
        "field_keys": ["custom.designer"]})];
    let designer = |writes: &Json| {
        score::score_run(27, writes, &steps, &entries, &definitions, &keys)
            .into_iter()
            .find(|s| s.key == "custom.designer")
            .unwrap()
    };
    let wrong = designer(&writes_with_designer(Some("Hans Wegner")));
    assert_eq!(wrong.outcome, Outcome::Wrong);
    assert!(wrong.review_candidate);
    assert_eq!(
        wrong.sources,
        vec!["https://www.pamono.com/a", "https://auctionet.com/b"]
    );
    assert_eq!(
        wrong.evidence,
        vec!["read: Designer: Hans Wegner (https://www.pamono.com/a)"]
    );
    let left = designer(&writes_with_designer(None));
    assert_eq!(left.outcome, Outcome::Correct);
    assert!(!left.review_candidate);
}

/// Image downloads: every request gets a small JPEG; every URL is recorded.
#[derive(Default)]
struct Images {
    urls: RefCell<Vec<String>>,
}

impl HttpSeam for Images {
    fn send(&self, wire: &Wire) -> Result<WireReply, WireError> {
        self.urls.borrow_mut().push(wire.url.clone());
        let n = u8::try_from(self.urls.borrow().len() % 200).unwrap();
        Ok(WireReply {
            status: 200,
            headers: vec![("content-type".into(), "image/jpeg".into())],
            body: SyntheticJpeg::rgb(640, 480, [n, 120, 255 - n]),
            final_url: wire.url.clone(),
            redirects: Vec::new(),
        })
    }
}

fn capture(path: &str) -> HttpReply {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../research/fixtures")
        .join(path);
    reply_from_capture(&serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap())
}

/// The real adapters over `FakeResearchHttp` replaying R1's captures: every model call gets the
/// observe capture, the Lens upload and search their captures; listing pages are unscripted.
fn providers(data: &Path, secrets: Arc<dyn SecretStore>) -> (Providers, Arc<FakeResearchHttp>) {
    let http = Arc::new(FakeResearchHttp::default());
    http.always(
        "POST",
        RESPONSES_URL,
        Ok(capture("openai/stage_observe_gpt-6-luna.json")),
    );
    http.always(
        "POST",
        serpapi_lens::UPLOAD_URL,
        Ok(capture("serpapi_lens/upload.json")),
    );
    http.always(
        "GET",
        serpapi_lens::SEARCH_URL,
        Ok(capture("serpapi_lens/search_cut_to_3.json")),
    );
    let providers = Providers {
        model: Some(Arc::new(OpenAiModel::new(
            http.clone(),
            secrets.clone(),
            ResearchFiles::new(data),
        ))),
        search: Some(Arc::new(SerpApiLens::new(http.clone(), secrets))),
        reader: Some(Arc::new(HttpPageReader::new(
            http.clone(),
            Arc::new(|_| {}),
            60,
        ))),
    };
    (providers, http)
}

/// Every path under `dir`, relative, sorted.
fn tree(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).unwrap().flatten() {
            let path = entry.path();
            out.push(path.strip_prefix(dir).unwrap().display().to_string());
            if path.is_dir() {
                stack.push(path);
            }
        }
    }
    out.sort();
    out
}

/// `accept` over two pieces of the owner's file (one published, one not; two images and one
/// condition image each) in a fresh scratch folder, with `settings_patch` as `--settings`.
struct Accepted {
    _root: tempfile::TempDir,
    root_path: PathBuf,
    data: PathBuf,
    paths: psm_infra::paths::AppPaths,
    outcome: accept::AcceptOutcome,
    images: Images,
    log: String,
    /// The HTTP seam of the research adapters (every model request the stub received).
    http: Arc<FakeResearchHttp>,
}

fn accept_two(settings_patch: Option<Json>) -> Accepted {
    let root = tempfile::tempdir().unwrap();
    let root_path = root.path().canonicalize().unwrap();
    // Two pieces of the owner's file (one published, one not), with two images each.
    let all: Vec<Json> = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join(answer_key::TEST_FILE)).unwrap(),
    )
    .unwrap();
    let mut pieces: Vec<Json> = all
        .into_iter()
        .filter(|p| p["n"] == 1 || p["n"] == 31)
        .collect();
    for p in &mut pieces {
        let n = p["n"].as_u64().unwrap();
        p["images"] = json!([
            format!("https://cdn.example/{n}/cover.jpg"),
            format!("https://cdn.example/{n}/side.jpg")
        ]);
        p["condition_report_images"] = json!([format!("https://cdn.example/{n}/scratch.jpg")]);
    }
    let pieces_file = root_path.join("pieces.json");
    std::fs::write(&pieces_file, serde_json::to_vec(&pieces).unwrap()).unwrap();
    let data: PathBuf = root_path.join("data");
    std::fs::create_dir_all(&data).unwrap();
    let paths = psm_infra::paths::AppPaths::for_dir(&data);
    let secrets_for_providers: Arc<dyn SecretStore> = {
        let s = Arc::new(accept::MemorySecrets::default());
        s.set(
            psm_research::service::MODEL_ACCOUNT,
            "sk-accept-test-model-0001",
        )
        .unwrap();
        s.set(
            psm_research::service::SEARCH_ACCOUNT,
            "serp-accept-test-0002",
        )
        .unwrap();
        s
    };
    let (seams, http) = providers(&data, secrets_for_providers);
    with_test_research(
        &paths,
        ResearchTestSeams {
            providers: seams,
            wait: Arc::new(|_| {}),
        },
    );
    let env = |name: &str| match name {
        "OPENAI_API_KEY" => Some("sk-accept-test-model-0001".to_owned()),
        "SERPAPI_API_KEY" => Some("serp-accept-test-0002".to_owned()),
        _ => None,
    };
    let images = Images::default();
    let mut log: Vec<u8> = Vec::new();
    let outcome = accept::run(
        &AcceptOptions {
            pieces: pieces_file.clone(),
            data: data.clone(),
            ceiling_micro: 40_000_000,
            only: None,
            settings_patch,
            run_timeout: std::time::Duration::from_secs(120),
        },
        &images,
        &env,
        &mut log,
    )
    .unwrap();
    let log = String::from_utf8(log).unwrap();
    Accepted {
        _root: root,
        root_path,
        data,
        paths,
        outcome,
        images,
        log,
        http,
    }
}

#[test]
fn a23_c2a_accept_runs_two_pieces_in_the_scratch_folder() {
    let Accepted {
        _root,
        root_path,
        data,
        paths,
        outcome,
        images,
        log,
        ..
    } = accept_two(None);
    assert_eq!(outcome.pieces.len(), 2, "{log}");
    assert_eq!(
        images.urls.borrow().len(),
        6,
        "three images per piece downloaded"
    );

    // Two items, two sessions, each with its three photos (two main, one condition).
    let store = psm_infra::sqlite::store::SqliteStore::open(&paths).unwrap();
    let mut skus = Vec::new();
    store
        .transaction(&mut |tx| {
            skus = tx.items()?.into_iter().map(|i| i.sku).collect();
            Ok(())
        })
        .unwrap();
    skus.sort();
    assert_eq!(skus, vec![accept::sku(1), accept::sku(31)]);
    let mut session_ids = Vec::new();
    for piece in &outcome.pieces {
        let session = store.session(piece.session_id).unwrap();
        assert_eq!(session.session.item_id, Some(piece.item_id));
        assert_eq!(
            (
                session.photo_count,
                session.main_count,
                session.condition_count
            ),
            (3, 2, 1),
            "piece {}",
            piece.n
        );
        session_ids.push(piece.session_id);
    }
    session_ids.dedup();
    assert_eq!(session_ids.len(), 2);

    // One run per session, each in a final state, each with its own export.
    let runs = store
        .research_transaction(|tx| psm_research::store::runs(tx.conn(), None))
        .unwrap();
    assert_eq!(runs.len(), 2);
    for piece in &outcome.pieces {
        let of_session: Vec<_> = runs
            .iter()
            .filter(|r| r.session_id == piece.session_id)
            .collect();
        assert_eq!(of_session.len(), 1, "piece {}", piece.n);
        assert!(
            accept::FINAL_STATES.contains(&of_session[0].state.as_str()),
            "piece {}: {}",
            piece.n,
            of_session[0].state.as_str()
        );
        let zip = piece.export.as_ref().expect("an export");
        assert!(zip.starts_with(&data), "{}", zip.display());
        let folder = zip.with_extension("");
        let opened = psm_research_tool::export_reader::open(&folder).unwrap();
        let ids: Vec<i64> = opened.runs().iter().map(|r| r.id()).collect();
        assert_eq!(ids, vec![of_session[0].id]);
    }

    // Nothing outside the scratch folder: beside it only the pieces file this test wrote.
    let beside: Vec<String> = tree(&root_path)
        .into_iter()
        .filter(|p| !p.starts_with("data"))
        .collect();
    assert_eq!(beside, vec!["pieces.json".to_owned()]);
}

/// The six fields the owner's shop leaves alone (`docs/test-items/accept_settings_owner.json`).
const LEFT_ALONE: [&str; 6] = [
    "custom.totalwidth",
    "custom.totaldepth",
    "custom.totalheight",
    "custom.seatheightchairs",
    "custom.weight_definition",
    "custom.damage_details",
];

/// Each run's settings snapshot, parsed.
fn snapshots(a: &Accepted) -> Vec<Json> {
    let store = psm_infra::sqlite::store::SqliteStore::open(&a.paths).unwrap();
    let runs = store
        .research_transaction(|tx| psm_research::store::runs(tx.conn(), None))
        .unwrap();
    assert_eq!(runs.len(), 2, "{}", a.log);
    runs.iter()
        .map(|r| serde_json::from_str(r.settings_snapshot.as_deref().unwrap()).unwrap())
        .collect()
}

/// The field keys of every model request the stub received.
fn model_request_fields(a: &Accepted) -> Vec<String> {
    let mut keys = Vec::new();
    for request in a.http.requests().iter().filter(|r| r.url == RESPONSES_URL) {
        let psm_research::adapters::http::Body::Json(body) = &request.body else {
            panic!("a model request without a JSON body");
        };
        let text = body["input"][0]["content"][0]["text"].as_str().unwrap();
        let input: Json = serde_json::from_str(text).unwrap();
        for field in input["fields"].as_array().into_iter().flatten() {
            keys.push(field["key"].as_str().unwrap().to_owned());
        }
    }
    keys
}

#[test]
fn a26_c13a_accept_measures_the_shipped_models() {
    let a = accept_two(None);
    let shipped = psm_research::config::ResearchSettings::default();
    for snapshot in snapshots(&a) {
        for stage in psm_research::domain::Stage::ALL {
            let expected = shipped.stage(stage);
            let name = stage.as_str();
            assert_eq!(
                (
                    snapshot["stages"][name]["model"].as_str().unwrap(),
                    snapshot["stages"][name]["effort"].as_str().unwrap()
                ),
                (expected.model.as_str(), expected.effort.as_str()),
                "{name}"
            );
        }
        assert_eq!(
            snapshot["attribution_keys"],
            json!(accept::ATTRIBUTION_KEYS)
        );
    }
}

#[test]
fn a26_c13b_the_owner_settings_leave_six_fields_alone() {
    let file = repo_root().join("docs/test-items/accept_settings_owner.json");
    let owner: Json = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let a = accept_two(Some(owner));
    for snapshot in snapshots(&a) {
        assert_eq!(snapshot["excluded_keys"], json!(LEFT_ALONE));
        assert_eq!(
            snapshot["extra_shop_hosts"],
            json!([
                "beyovintage.com",
                "beyovintage.myshopify.com",
                "68c5b4-6a.myshopify.com"
            ])
        );
    }
    let fields = model_request_fields(&a);
    assert!(!fields.is_empty(), "{}", a.log);
    for key in LEFT_ALONE {
        assert!(!fields.iter().any(|f| f == key), "{key} reached a model");
    }
    // Without the file, some of the six do reach a model request (the rows are not vacuous).
    let plain = model_request_fields(&accept_two(None));
    assert!(
        LEFT_ALONE.iter().any(|key| plain.iter().any(|f| f == key)),
        "{plain:?}"
    );
}
