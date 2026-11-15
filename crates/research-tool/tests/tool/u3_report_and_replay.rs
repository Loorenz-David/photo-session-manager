//! R9·U1 (rows A22·C1–C5): `report` and `replay` over an export produced in the test by the
//! research crate's `export` (R7·U2) from scripted runs on the research harness, so every fixture
//! is the real export shape written by production code. Requests go to the tool's HTTP stub.

#[path = "../../../research/tests/engine/harness.rs"]
mod harness;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use harness::{DESIGNER, Harness, INTEGRATION, MAKER, OUTSIDE, SCRATCHES, WOODS};
use psm_core::{
    domain::{
        attribute::{AttributeGroup, Preset, PresetField, Provenance},
        photo::Classification,
    },
    ports::secrets::SecretStore,
};
use psm_research::{
    adapters::{
        http::HttpReply,
        openai::{OpenAiModel, adapter::RESPONSES_URL},
    },
    api::ExportSelection,
    domain::{RunState, Stage},
    fakes::{FakeResearchHttp, PageScript, reply_from_capture},
    gateway::{CallGateway, CallRequest, GatewayOutcome},
    ports::{ModelRequest, Providers, SearchCandidate},
    retention::DAY_MS,
    rules::sites::page_identity,
};
use psm_research_tool::{
    budget::{Ledger, Limits},
    export_reader,
    recording::{Call, Caller, Secrets},
    report::{self, Verdict},
    run_with,
};
use psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg;
use serde_json::{Value as Json, json};

use crate::Stub;

const P_E1: &str = "https://www.e1.example/p/1";
const P_E2: &str = "https://e2.example/item/2";
const P_UNTRUSTED: &str = "https://other.example/x";
/// The replay key of these rows: distinctive, in no expected text.
const REPLAY_KEY: &str = "sk-replay-KEY-31f9c0e2d7";

fn u(url: &str) -> String {
    page_identity(url).unwrap()
}

fn stated(name: &str) -> Json {
    json!({"stated": name, "value": name})
}

/// The command line with `env` holding only the given key, and the printed output.
fn command(args: &[&str], seam: &Stub, key: Option<&str>) -> (i32, String) {
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let key = key.map(str::to_owned);
    let env = move |name: &str| (name == "OPENAI_API_KEY").then(|| key.clone()).flatten();
    let mut out: Vec<u8> = Vec::new();
    let code = run_with(&args, seam, &env, &mut out);
    (code, String::from_utf8(out).unwrap())
}

fn export(h: &Harness) -> PathBuf {
    let dest = h.app.dir.path().join("exports");
    std::fs::create_dir_all(&dest).unwrap();
    h.service
        .export(ExportSelection::Items(vec![h.item_id]), &dest)
        .unwrap()
}

/// A Chair whose layout adds the maker and a visible text field, with designer and maker as the
/// attribution keys; e1 and e2 state designer A and maker B and match; observe reads Teak and
/// "Lacquered" (sure); condition reads Minor. The engine's run writes those five fields.
fn five_field_run() -> (Harness, i64) {
    let h = Harness::chair();
    h.app
        .attributes(Some(INTEGRATION))
        .presets_set(Preset {
            item_type: "Chair".into(),
            group: AttributeGroup::Details,
            layout: [DESIGNER, MAKER, WOODS, OUTSIDE]
                .into_iter()
                .map(|key| PresetField {
                    key: key.into(),
                    default: None,
                })
                .collect(),
        })
        .unwrap();
    h.patch_settings(json!({"attribution_keys": [DESIGNER, MAKER]}));
    h.add_main_photo_with_preview(h.session_id);
    h.add_main_photo_with_preview(h.session_id);
    h.add_photo_with_preview(h.session_id, Classification::Condition);
    h.search.set_results(
        [P_E1, P_E2, P_UNTRUSTED]
            .iter()
            .enumerate()
            .map(|(i, url)| SearchCandidate {
                url: (*url).to_owned(),
                rank: u32::try_from(i + 1).unwrap(),
                title: String::new(),
            })
            .collect(),
    );
    for (n, url) in [P_E1, P_E2, P_UNTRUSTED].iter().enumerate() {
        let n = u8::try_from(n + 1).unwrap() * 40;
        h.reader.set_page(
            url,
            PageScript::page(
                &format!("Page {n}"),
                vec![SyntheticJpeg::rgb(32, 24, [n, 255 - n, n / 2])],
            ),
        );
    }
    for url in [P_E1, P_E2] {
        h.model.reply(
            Stage::Read,
            &u(url),
            json!({"keys": {DESIGNER: stated("Designer A"), MAKER: stated("Maker B")}}),
        );
        h.model
            .reply(Stage::Compare, &u(url), json!({"verdict": "match"}));
    }
    h.model.reply(
        Stage::Compare,
        &u(P_UNTRUSTED),
        json!({"verdict": "different"}),
    );
    // R12·U1: one observe call over both main photos.
    h.model.reply(
        Stage::Observe,
        psm_research::stages::OBSERVE_UNIT,
        json!({"reads": {
            WOODS: {"value": ["Teak"], "sure": true},
            OUTSIDE: {"value": "Lacquered", "sure": true},
        }, "marking_photos": []}),
    );
    h.model.reply(
        Stage::Condition,
        "condition",
        json!({"reads": {SCRATCHES: {"value": "Minor", "sure": true}}}),
    );
    h.changed();
    h.tick(1_000);
    let run = h.runs(h.session_id)[0].id;
    assert_eq!(h.run_row(run).state, RunState::Finished);
    (h, run)
}

/// The five-field run, then the photographer: designer and woods accepted, maker edited to
/// "Maker C", scratches cleared, the visible text left; then the export, read back.
fn acted_on_export() -> (Harness, i64, export_reader::Opened) {
    let (h, run) = five_field_run();
    let attributes = h.app.attributes(Some(INTEGRATION));
    for key in [DESIGNER, MAKER, WOODS, OUTSIDE, SCRATCHES] {
        assert!(
            h.row(key).is_some_and(|r| r.provenance == Provenance::Ai),
            "{key} written"
        );
    }
    attributes.accept(h.item_id, DESIGNER).unwrap();
    attributes.accept(h.item_id, WOODS).unwrap();
    attributes
        .set_value(h.item_id, MAKER, json!("Maker C"), Provenance::Human)
        .unwrap();
    attributes.clear(h.item_id, SCRATCHES).unwrap();
    let opened = export_reader::open(&export(&h)).unwrap();
    (h, run, opened)
}

#[test]
fn a22_c1a_report_counts_actions_and_stage_cost() {
    let (h, run, opened) = acted_on_export();
    let report = report::export_report(opened.runs());
    let o = &report.overall;
    assert_eq!(o.suggestions, 5);
    assert_eq!(
        (o.accepted_unchanged, o.edited, o.cleared, o.not_acted_on),
        (2, 1, 1, 1)
    );
    assert_eq!(o.kept_previous, 0);
    let calls = h.calls(run);
    assert!(!calls.is_empty());
    let mut expected: Vec<(String, i64)> = Vec::new();
    for call in &calls {
        match expected.iter_mut().find(|(s, _)| *s == call.stage) {
            Some((_, sum)) => *sum += call.cost_micro_usd,
            None => expected.push((call.stage.clone(), call.cost_micro_usd)),
        }
    }
    let mut reported: Vec<(String, i64)> = report
        .stages
        .iter()
        .filter(|(_, c)| c.calls > 0)
        .map(|(s, c)| (s.clone(), c.cost_micro_usd))
        .collect();
    expected.sort();
    reported.sort();
    assert_eq!(reported, expected);
    assert_eq!(
        o.cost_micro_usd,
        calls.iter().map(|c| c.cost_micro_usd).sum::<i64>()
    );
}

#[test]
fn a22_c1b_report_attribution_right_and_wrong() {
    let (_h, run, opened) = acted_on_export();
    let report = report::export_report(opened.runs());
    let verdicts: Vec<(i64, String, Verdict)> = report
        .attributions
        .iter()
        .map(|a| (a.run_id, a.key.clone(), a.verdict))
        .collect();
    assert_eq!(
        verdicts,
        vec![
            (run, DESIGNER.to_owned(), Verdict::Right),
            (run, MAKER.to_owned(), Verdict::Wrong),
        ]
    );
    assert_eq!(
        (
            report.overall.attribution_right,
            report.overall.attribution_wrong,
            report.overall.attribution_open
        ),
        (1, 1, 0)
    );
}

/// One observe call with two images, sent by the real OpenAI adapter over `FakeResearchHttp`
/// through the gateway (MC-A5/MC-A9 record), on a run of the harness's Chair.
struct Recorded {
    h: Harness,
    run: i64,
    photos: [i64; 2],
    /// The body the adapter put on the wire.
    sent: Vec<u8>,
}

fn observe_reply() -> HttpReply {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../research/fixtures/openai/stage_observe_gpt-6-luna.json");
    let mut capture: Json = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let text = json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}, "shows_marking": false})
        .to_string();
    for item in capture["response"]["body"]["output"]
        .as_array_mut()
        .unwrap()
    {
        for content in item["content"].as_array_mut().into_iter().flatten() {
            if content["type"] == "output_text" {
                content["text"] = json!(text);
            }
        }
    }
    reply_from_capture(&capture)
}

fn recorded_observe_call() -> Recorded {
    let h = Harness::chair();
    let photos = [
        h.add_main_photo_with_preview(h.session_id),
        h.add_main_photo_with_preview(h.session_id),
    ];
    let mut config = h.stage_config();
    config.providers.model = "openai".into();
    let run = h.start_run(&config);
    let images = photos
        .iter()
        .map(|p| h.images.render(*p, run, 1024).ok().unwrap())
        .collect();
    let http = Arc::new(FakeResearchHttp::default());
    http.once("POST", RESPONSES_URL, Ok(observe_reply()));
    let model = OpenAiModel::new(
        http.clone(),
        h.app.secrets.clone() as Arc<dyn SecretStore>,
        h.files(),
    );
    let gateway = CallGateway::new(
        h.store().clone(),
        h.app.clock.clone(),
        Providers {
            model: Some(Arc::new(model)),
            search: None,
            reader: None,
        },
        h.secret_values(),
        h.files(),
        h.waiter(),
    );
    let request = ModelRequest {
        stage: Stage::Observe,
        unit_key: "photos".into(),
        model: "m1".into(),
        effort: "low".into(),
        language: "English".into(),
        instructions: "Read what the photos show.".into(),
        input: json!({"fields": [{"key": WOODS, "label": "Woods", "core_type": "multi_choice",
                                  "choices": ["Oak", "Teak"]}]}),
        images,
        listing_images: Vec::new(),
    };
    let called = gateway
        .call(run, Stage::Observe, "photos", CallRequest::Model(request))
        .unwrap();
    assert!(matches!(called.outcome, GatewayOutcome::Answered(_)));
    let requests = http.requests();
    assert_eq!(requests.len(), 1);
    Recorded {
        h,
        run,
        photos,
        sent: requests[0].body.bytes(),
    }
}

fn replay_args<'a>(export: &'a str, run: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["replay", export, "--run", run, "--stage", "observe"];
    args.extend_from_slice(extra);
    args
}

fn received_json(stub: &Stub) -> Json {
    let received = stub.received.borrow();
    assert_eq!(received.len(), 1, "one request");
    serde_json::from_slice(&received[0].body).unwrap()
}

#[test]
fn a22_c2a_replay_sends_the_recorded_body() {
    let r = recorded_observe_call();
    let zip = export(&r.h);
    let stub = Stub::default();
    let run = r.run.to_string();
    let (code, out) = command(
        &replay_args(zip.to_str().unwrap(), &run, &[]),
        &stub,
        Some(REPLAY_KEY),
    );
    assert_eq!(code, 0, "{out}");
    let received = stub.received.borrow();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].url, RESPONSES_URL);
    assert_eq!(received[0].body, r.sent, "byte-equal to the adapter's body");
    let body: Json = serde_json::from_slice(&received[0].body).unwrap();
    let images: Vec<&str> = body["input"][0]["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["image_url"].as_str())
        .collect();
    assert_eq!(images.len(), 2, "both images restored");
}

#[test]
fn a22_c2b_replay_model_substitutes_only_the_model() {
    let r = recorded_observe_call();
    let zip = export(&r.h);
    let stub = Stub::default();
    let run = r.run.to_string();
    let (code, out) = command(
        &replay_args(zip.to_str().unwrap(), &run, &["--model", "m2"]),
        &stub,
        Some(REPLAY_KEY),
    );
    assert_eq!(code, 0, "{out}");
    let original: Json = serde_json::from_slice(&r.sent).unwrap();
    let received = received_json(&stub);
    assert_ne!(received, original);
    let mut expected = original.clone();
    expected["model"] = json!("m2");
    assert_eq!(received, expected);
}

#[test]
fn a22_c2c_replay_instructions_substitutes_only_the_instructions() {
    let r = recorded_observe_call();
    let zip = export(&r.h);
    let file = r.h.app.dir.path().join("instructions.md");
    std::fs::write(&file, "Name the wood of every visible part.\n").unwrap();
    let stub = Stub::default();
    let run = r.run.to_string();
    let (code, out) = command(
        &replay_args(
            zip.to_str().unwrap(),
            &run,
            &["--instructions", file.to_str().unwrap()],
        ),
        &stub,
        Some(REPLAY_KEY),
    );
    assert_eq!(code, 0, "{out}");
    let original: Json = serde_json::from_slice(&r.sent).unwrap();
    let mut expected = original.clone();
    expected["instructions"] = json!("Name the wood of every visible part.\n");
    assert_ne!(expected, original);
    assert_eq!(received_json(&stub), expected);
}

/// The not-replayable line of `out` for call or run `who`.
fn not_replayable(out: &str, who: &str) -> String {
    out.lines()
        .find(|l| l.starts_with(&format!("{who}: not replayable: ")))
        .unwrap_or_else(|| panic!("no not-replayable line for {who} in:\n{out}"))
        .to_owned()
}

#[test]
fn a22_c3a_withheld_missing_and_expired_are_not_replayable() {
    // A photo deleted before the export: its image is withheld.
    let withheld = recorded_observe_call();
    withheld.h.app.review().delete(withheld.photos[0]).unwrap();
    let zip = export(&withheld.h);
    let call = withheld.h.calls(withheld.run)[0].id;
    let stub = Stub::default();
    let run = withheld.run.to_string();
    let (code, out) = command(
        &replay_args(zip.to_str().unwrap(), &run, &[]),
        &stub,
        Some(REPLAY_KEY),
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        not_replayable(&out, &format!("call {call}")),
        format!("call {call}: not replayable: an image was withheld (photo deleted)")
    );
    assert!(stub.received.borrow().is_empty());

    // An image file missing from the export's `images/`.
    let missing = recorded_observe_call();
    let zip = export(&missing.h);
    let folder = zip.with_extension("");
    let opened = export_reader::open(&folder).unwrap();
    let run_dir = opened.run(missing.run).unwrap().dir.clone();
    drop(opened);
    let gone = std::fs::read_dir(run_dir.join("images"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .min()
        .unwrap();
    std::fs::remove_file(&gone).unwrap();
    let sha = gone.file_stem().unwrap().to_string_lossy().into_owned();
    let call = missing.h.calls(missing.run)[0].id;
    let stub = Stub::default();
    let run = missing.run.to_string();
    let (code, out) = command(
        &replay_args(folder.to_str().unwrap(), &run, &[]),
        &stub,
        Some(REPLAY_KEY),
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        not_replayable(&out, &format!("call {call}")),
        format!("call {call}: not replayable: image {sha} is missing from images/")
    );
    assert!(stub.received.borrow().is_empty());

    // A run whose call records expired before the export.
    let expired = recorded_observe_call();
    expired
        .h
        .service
        .purge_expired(expired.h.now() + 31 * DAY_MS)
        .unwrap();
    let zip = export(&expired.h);
    let stub = Stub::default();
    let run = expired.run.to_string();
    let (code, out) = command(
        &replay_args(zip.to_str().unwrap(), &run, &[]),
        &stub,
        Some(REPLAY_KEY),
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        not_replayable(&out, &format!("run {}", expired.run)),
        format!("run {}: not replayable: call record expired", expired.run)
    );
    assert!(stub.received.borrow().is_empty());
}

#[test]
fn a22_c4a_replay_without_a_key_sends_nothing() {
    let r = recorded_observe_call();
    let zip = export(&r.h);
    let stub = Stub::default();
    let run = r.run.to_string();
    let (code, out) = command(&replay_args(zip.to_str().unwrap(), &run, &[]), &stub, None);
    assert_ne!(code, 0);
    assert_eq!(out, "");
    assert!(stub.received.borrow().is_empty());
}

#[test]
fn a22_c4b_the_key_is_in_no_output_and_no_file() {
    let r = recorded_observe_call();
    let zip = export(&r.h);
    let out_dir = r.h.app.dir.path().join("replay-out");
    let stub = Stub::default();
    let run = r.run.to_string();
    let (code, out) = command(
        &replay_args(
            zip.to_str().unwrap(),
            &run,
            &["--out", out_dir.to_str().unwrap()],
        ),
        &stub,
        Some(REPLAY_KEY),
    );
    assert_eq!(code, 0, "{out}");
    let received = stub.received.borrow();
    assert_eq!(received.len(), 1, "the replay was sent");
    assert!(
        received[0]
            .headers
            .iter()
            .any(|(k, v)| k == "authorization" && v.contains(REPLAY_KEY)),
        "the key went in the request header"
    );
    assert!(out.contains("call "), "{out}");
    assert!(!out.contains(REPLAY_KEY));
    let written: Vec<_> = std::fs::read_dir(&out_dir).unwrap().flatten().collect();
    assert_eq!(written.len(), 1, "one replay file");
    assert_eq!(
        crate::files_containing(&out_dir, REPLAY_KEY),
        Vec::<String>::new()
    );
    assert_eq!(
        crate::files_containing(&zip.with_extension(""), REPLAY_KEY),
        Vec::<String>::new()
    );
}

#[test]
fn a22_c5a_report_reads_bench_recordings() {
    let dir = tempfile::tempdir().unwrap();
    let stub = Stub::default();
    let caller = Caller::new(
        &stub,
        Secrets::default(),
        Ledger::default(),
        Limits::u1(10_000_000),
    );
    let calls = [
        ("lens-1", "serpapi_lens", "serpapi_search", "", 1_500),
        ("lens-2", "serpapi_lens", "serpapi_search", "", 1_500),
        ("observe-1", "openai", "openai", "m1", 2_250),
    ];
    for (label, provider, cap, model, cost) in calls {
        let mut call = Call::new(
            label,
            dir.path(),
            provider,
            cap,
            "GET",
            &format!("https://api.example/{label}"),
        );
        call.model = model.to_owned();
        call.costing = Box::new(move |_, _| (json!({}), cost));
        caller.call(call).unwrap();
    }
    let (code, out) = command(&["report", dir.path().to_str().unwrap()], &stub, None);
    assert_eq!(code, 0, "{out}");
    let line = |group: &str| {
        out.lines()
            .find(|l| {
                l.split_whitespace()
                    .take(group.split(' ').count())
                    .collect::<Vec<_>>()
                    == group.split(' ').collect::<Vec<_>>()
            })
            .unwrap_or_else(|| panic!("no line for {group} in:\n{out}"))
            .split_whitespace()
            .skip(group.split(' ').count())
            .take(2)
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_eq!(line("serpapi_lens"), vec!["2", "0.003000"]);
    assert_eq!(line("openai m1"), vec!["1", "0.002250"]);
    assert_eq!(line("total"), vec!["3", "0.005250"]);
}
