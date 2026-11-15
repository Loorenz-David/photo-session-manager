//! R8·U1 (rows A15·C1–C6, C5(d) first half): `ModelProvider::call` on the OpenAI adapter over
//! `FakeResearchHttp` replaying R1's captures (`crates/research/fixtures/openai/`). A **derived**
//! reply is built here from the capture its row names: the envelope (status, headers, `usage`)
//! kept, one named part replaced. Rows C5(b), (c) use loopback sockets on port 0.

use std::{
    io::Read,
    net::TcpListener,
    sync::Arc,
    time::{Duration, Instant},
};

use base64::Engine;
use psm_core::ports::secrets::SecretStore;
use psm_research::{
    adapters::{
        http::{AddressGuard, HttpReply, HttpRequest, ReqwestResearchHttp, ResearchHttp},
        openai::OpenAiModel,
    },
    config::{FallbackSnapshot, PriceSnapshot},
    domain::{CallFailure, ErrorClass, FailureKind, Stage},
    fakes::{FakeResearchHttp, ModelScript, reply_from_capture},
    gateway::CallRequest,
    images::ResearchImage,
    ports::{ModelProvider, ModelRequest},
    rules::{
        calls::classify,
        cost::{Price, PriceTable, call_cost},
    },
    service::MODEL_ACCOUNT,
};
use serde_json::{Value as Json, json};

use crate::{
    harness::{DESIGNER, Harness, MODEL_KEY, SCRATCHES, WIDTH, WOOD, WOODS},
    stages::{P_E1, P_E2, P_UNTRUSTED, assert_form_filled, full_script},
};

/// A capture file under `crates/research/fixtures/`.
pub(crate) fn capture(path: &str) -> Json {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(path);
    serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap()
}

/// Derived: the capture's reply with the text of its `output_text` replaced.
fn with_output_text(path: &str, text: &str) -> HttpReply {
    let mut c = capture(path);
    for item in c["response"]["body"]["output"].as_array_mut().unwrap() {
        for content in item["content"].as_array_mut().into_iter().flatten() {
            if content["type"] == "output_text" {
                content["text"] = json!(text);
            }
        }
    }
    reply_from_capture(&c)
}

const OBSERVE_LUNA: &str = "openai/stage_observe_gpt-6-luna.json";

fn adapter(h: &Harness, http: &Arc<FakeResearchHttp>) -> OpenAiModel {
    OpenAiModel::new(
        http.clone(),
        h.app.secrets.clone() as Arc<dyn SecretStore>,
        h.files(),
    )
}

fn request(stage: Stage, input: Json, images: Vec<ResearchImage>) -> ModelRequest {
    ModelRequest {
        stage,
        unit_key: "u".into(),
        model: "m1".into(),
        effort: "low".into(),
        language: "English".into(),
        instructions: "look".into(),
        input,
        images,
        listing_images: Vec::new(),
    }
}

fn field(key: &str, core_type: &str, choices: &[&str]) -> Json {
    json!({"key": key, "label": key, "core_type": core_type, "choices": choices, "description": null})
}

/// The two-field observe form of the derived rows: a choice and a text field.
fn two_fields() -> Json {
    json!({"fields": [field(WOOD, "choice", &["Oak", "Teak"]), field(DESIGNER, "text", &[])]})
}

/// One answered call: the adapter over a fake that replies `reply` to the Responses URL.
fn call_with(
    h: &Harness,
    req: &ModelRequest,
    reply: HttpReply,
) -> Result<psm_research::ports::ModelReply, CallFailure> {
    let http = Arc::new(FakeResearchHttp::default());
    http.once("POST", "https://api.openai.com/v1/responses", Ok(reply));
    adapter(h, &http).call(req)
}

fn units(list: &[(&str, i64)]) -> Option<Vec<(String, i64)>> {
    Some(list.iter().map(|(u, n)| ((*u).to_owned(), *n)).collect())
}

/// The body the fake received for its only request.
fn sent_body(http: &FakeResearchHttp) -> Json {
    match &http.requests()[0].body {
        psm_research::adapters::http::Body::Json(body) => body.clone(),
        other => panic!("not a JSON body: {other:?}"),
    }
}

#[test]
fn a15_c1a_usage_of_the_sol_capture() {
    let h = Harness::new();
    let reply = reply_from_capture(&capture("openai/image_strict_schema_gpt-6.1-sol.json"));
    // The capture's output is R1's version-0 shape, which no stage schema reads: the adapter
    // reports it as a schema failure carrying the reply's usage.
    let failure = call_with(
        &h,
        &request(Stage::Observe, two_fields(), Vec::new()),
        reply,
    )
    .unwrap_err();
    assert_eq!(failure.kind, FailureKind::Schema);
    let expected = units(&[
        ("input_token", 3),
        ("cache_write_token", 1313),
        ("output_token", 26),
    ]);
    assert_eq!(failure.usage, expected);
    let price = |unit: &str, micro: i64| Price {
        provider: "openai".into(),
        model: "gpt-6.1-sol".into(),
        unit: unit.into(),
        micro_usd_per_million: micro,
    };
    let prices = PriceTable {
        prices: vec![
            price("input_token", 2_000_000),
            price("cache_write_token", 2_500_000),
            price("output_token", 10_000_000),
        ],
        fallbacks: Vec::new(),
    };
    assert_eq!(
        call_cost(expected.as_deref(), true, "openai", "gpt-6.1-sol", &prices),
        3549
    );
}

#[test]
fn a15_c1b_web_search_calls_are_the_billed_count() {
    let h = Harness::new();
    let reply = reply_from_capture(&capture("openai/image_and_web_search.json"));
    let failure = call_with(
        &h,
        &request(Stage::Observe, two_fields(), Vec::new()),
        reply,
    )
    .unwrap_err();
    let usage = failure.usage.unwrap();
    for unit in [
        ("web_search_call", 3),
        ("input_token", 19715),
        ("cache_write_token", 5746),
        ("output_token", 561),
    ] {
        assert!(
            usage.contains(&(unit.0.to_owned(), unit.1)),
            "{usage:?} lacks {unit:?}"
        );
    }
}

#[test]
fn a15_c1c_incomplete_reply_is_a_schema_failure_with_usage() {
    let h = Harness::new();
    let reply = reply_from_capture(&capture("openai/incomplete_max_output_tokens.json"));
    let failure = call_with(
        &h,
        &request(Stage::Observe, two_fields(), Vec::new()),
        reply,
    )
    .unwrap_err();
    assert!(failure.sent);
    assert_eq!(failure.kind, FailureKind::Schema);
    assert_eq!(
        failure.usage,
        units(&[
            ("input_token", 3),
            ("cached_input_token", 1313),
            ("output_token", 16)
        ])
    );
}

#[test]
fn a15_c2a_invalid_key() {
    let h = Harness::new();
    let reply = reply_from_capture(&capture("openai/error_invalid_key.json"));
    let failure = call_with(
        &h,
        &request(Stage::Observe, two_fields(), Vec::new()),
        reply,
    )
    .unwrap_err();
    assert_eq!(
        (failure.status, failure.kind, failure.quota),
        (Some(401), FailureKind::Status, false)
    );
    assert_eq!(classify(&failure), ErrorClass::Key);
}

#[test]
fn a15_c2b_rate_limit_is_transient() {
    let h = Harness::new();
    // Derived: classify reads the status alone (`rules/calls.rs`), so an empty JSON body suffices.
    let reply = HttpReply {
        status: 429,
        headers: vec![("content-type".into(), "application/json".into())],
        body: b"{}".to_vec(),
    };
    let failure = call_with(
        &h,
        &request(Stage::Observe, two_fields(), Vec::new()),
        reply,
    )
    .unwrap_err();
    assert_eq!(
        (failure.status, failure.kind, failure.quota),
        (Some(429), FailureKind::Status, false)
    );
    assert_eq!(classify(&failure), ErrorClass::Transient);
}

#[test]
fn a15_c2c_context_length_is_a_request_error() {
    let h = Harness::new();
    let reply = reply_from_capture(&capture("openai/error_context_length.json"));
    let failure = call_with(
        &h,
        &request(Stage::Observe, two_fields(), Vec::new()),
        reply,
    )
    .unwrap_err();
    assert_eq!(
        (failure.status, failure.kind),
        (Some(400), FailureKind::Status)
    );
    assert_eq!(classify(&failure), ErrorClass::Request);
}

#[test]
fn a15_c3a_refusal() {
    let h = Harness::new();
    // Derived (documented shape, evidence V3): the observe envelope with the message content
    // replaced by a refusal.
    let mut c = capture(OBSERVE_LUNA);
    for item in c["response"]["body"]["output"].as_array_mut().unwrap() {
        if item["type"] == "message" {
            item["content"] = json!([{"type": "refusal", "refusal": "I can't help with that."}]);
        }
    }
    let failure = call_with(
        &h,
        &request(Stage::Observe, two_fields(), Vec::new()),
        reply_from_capture(&c),
    )
    .unwrap_err();
    assert!(failure.sent);
    assert_eq!(failure.kind, FailureKind::Refusal);
    assert_eq!(failure.text, "I can't help with that.");
    assert_eq!(
        failure.usage,
        units(&[
            ("input_token", 3),
            ("cache_write_token", 1555),
            ("output_token", 77)
        ])
    );
}

#[test]
fn a15_c3b_output_missing_a_schema_key_is_a_schema_failure() {
    let h = Harness::new();
    let req = request(Stage::Observe, two_fields(), Vec::new());
    let reads = json!({
        WOOD: {"value": "Oak", "sure": true},
        DESIGNER: {"value": null, "sure": false},
    });
    let without = json!({ "reads": reads }).to_string();
    let failure = call_with(&h, &req, with_output_text(OBSERVE_LUNA, &without)).unwrap_err();
    assert_eq!(
        (failure.sent, failure.status, failure.kind),
        (true, Some(200), FailureKind::Schema)
    );
    let with = json!({ "reads": reads, "shows_marking": false }).to_string();
    let reply = call_with(&h, &req, with_output_text(OBSERVE_LUNA, &with)).unwrap();
    assert_eq!(
        reply.output,
        json!({ "reads": reads, "shows_marking": false })
    );
}

#[test]
fn a15_c4a_request_form() {
    let h = Harness::new();
    let photo = h.add_main_photo_with_preview(h.session_id);
    let run = h.running_run(&h.config());
    let image = h.images.render(photo, run, 1024).unwrap();
    let mut req = request(
        Stage::Observe,
        json!({"fields": [field(WOOD, "choice", &["Oak", "Teak"])]}),
        vec![image.clone()],
    );
    req.effort = String::new();
    req.language = "Swedish".into();
    let http = Arc::new(FakeResearchHttp::default());
    let _ = adapter(&h, &http).call(&req);
    let body = sent_body(&http);
    let mut keys: Vec<&str> = body
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "input",
            "instructions",
            "max_output_tokens",
            "model",
            "store",
            "text"
        ]
    );
    let content = body["input"][0]["content"].as_array().unwrap();
    let images: Vec<&Json> = content
        .iter()
        .filter(|c| c["type"] == "input_image")
        .collect();
    assert_eq!(images.len(), 1);
    let expected = format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(image.bytes().unwrap())
    );
    assert_eq!(images[0]["image_url"], json!(expected));
    assert!(images[0].get("detail").is_none());
    assert_eq!(
        body["text"]["format"]["schema"]["properties"]["reads"]["properties"][WOOD]["properties"]["value"],
        json!({"anyOf": [{"type": "string", "enum": ["Oak", "Teak"]}, {"type": "null"}]})
    );
    assert!(body.to_string().contains("Swedish"));
    assert_eq!(body["max_output_tokens"], json!(4000));
}

#[test]
fn a15_c4b_instructions_model_effort_and_schema_per_stage() {
    let h = Harness::new();
    let fields = json!([field(WOOD, "choice", &["Oak", "Teak"])]);
    let cases = [
        (Stage::Observe, vec!["reads", "shows_marking"]),
        (Stage::Read, vec!["keys"]),
        (Stage::Compare, vec!["verdict"]),
        (Stage::Labels, vec!["labels", "text"]),
        (Stage::Condition, vec!["reads"]),
        (Stage::Decide, vec!["reasons"]),
    ];
    for (stage, top) in cases {
        let mut req = request(stage, json!({ "fields": fields }), Vec::new());
        req.instructions = format!("I-{}", stage.as_str());
        req.model = format!("m-{}", stage.as_str());
        req.effort = "low".into();
        let http = Arc::new(FakeResearchHttp::default());
        let _ = adapter(&h, &http).call(&req);
        let body = sent_body(&http);
        assert_eq!(body["instructions"], json!(format!("I-{}", stage.as_str())));
        assert_eq!(body["model"], json!(format!("m-{}", stage.as_str())));
        assert_eq!(body["reasoning"], json!({"effort": "low"}), "{stage:?}");
        let schema = &body["text"]["format"]["schema"];
        let mut keys: Vec<&str> = schema["properties"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, top, "{stage:?}");
        assert_eq!(schema["required"].as_array().unwrap().len(), top.len());
    }
    let mut plain = request(Stage::Observe, json!({ "fields": fields }), Vec::new());
    plain.effort = String::new();
    let http = Arc::new(FakeResearchHttp::default());
    let _ = adapter(&h, &http).call(&plain);
    assert!(sent_body(&http).get("reasoning").is_none());
}

#[test]
fn a15_c5a_the_key_is_redacted_in_the_exchange() {
    let h = Harness::new();
    let req = request(Stage::Observe, two_fields(), Vec::new());
    let payload = json!({
        "reads": {WOOD: {"value": "Oak", "sure": true}, DESIGNER: {"value": null, "sure": false}},
        "shows_marking": false,
    });
    let http = Arc::new(FakeResearchHttp::default());
    http.once(
        "POST",
        "https://api.openai.com/",
        Ok(with_output_text(OBSERVE_LUNA, &payload.to_string())),
    );
    let reply = adapter(&h, &http).call(&req).unwrap();
    assert_eq!(
        reply.exchange.request["headers"]["authorization"],
        json!("<redacted>")
    );
    let shown = serde_json::to_string(&reply.exchange).unwrap();
    assert!(!shown.contains(MODEL_KEY), "{shown}");
    // The key itself went out, read from the secrets port.
    let sent = &http.requests()[0];
    assert!(
        sent.headers
            .contains(&("authorization".to_owned(), format!("Bearer {MODEL_KEY}")))
    );
}

#[test]
fn a15_c5b_closed_port_is_not_sent() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let http = ReqwestResearchHttp::new(Duration::from_secs(5), AddressGuard::Any);
    let url = format!("http://127.0.0.1:{port}/v1/responses?api_key=k-123");
    let failure = http
        .send(&HttpRequest::get(&url))
        .unwrap_err()
        .into_call_failure();
    assert_eq!((failure.sent, failure.kind), (false, FailureKind::Connect));
    assert!(!failure.text.contains("http://"), "{}", failure.text);
    assert!(!failure.text.contains("/v1/responses"), "{}", failure.text);
    assert!(!failure.text.contains("k-123"), "{}", failure.text);
}

#[test]
fn a15_c5c_silent_server_times_out_after_send() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            std::thread::sleep(Duration::from_secs(3));
        }
    });
    let http = ReqwestResearchHttp::new(Duration::from_millis(200), AddressGuard::Any);
    let started = Instant::now();
    let failure = http
        .send(&HttpRequest::get(&format!("http://127.0.0.1:{port}/x")))
        .unwrap_err()
        .into_call_failure();
    assert_eq!((failure.sent, failure.kind), (true, FailureKind::Timeout));
    assert!(started.elapsed() < Duration::from_secs(3));
}

/// The key as `serde_json` writes it inside a string.
pub(crate) const ESCAPED_KEY: &str = "sk-a\"b\\c";
pub(crate) const ESCAPED_KEY_IN_JSON: &str = r#"sk-a\"b\\c"#;

#[test]
fn a15_c5d_escaped_key_is_redacted_in_the_call_record() {
    let h = Harness::new();
    h.app.secrets.set(MODEL_ACCOUNT, ESCAPED_KEY).unwrap();
    let run = h.running_run(&h.config());
    h.model.script(
        Stage::Observe,
        "u",
        ModelScript {
            output: json!({}),
            usage: None,
            response: Some(json!({ "echo": ESCAPED_KEY })),
        },
    );
    h.gateway()
        .call(
            run,
            Stage::Observe,
            "u",
            CallRequest::Model(request(Stage::Observe, json!({}), Vec::new())),
        )
        .unwrap();
    let call = h.calls(run).remove(0);
    let response = call.response.unwrap();
    assert!(response.contains("<redacted>"), "{response}");
    assert!(!response.contains(ESCAPED_KEY), "{response}");
    assert!(!response.contains(ESCAPED_KEY_IN_JSON), "{response}");
    assert!(call.redaction_hit);
}

#[test]
fn a15_c6a_full_run_with_the_openai_adapter() {
    let h = Harness::chair();
    let (mains, _condition) = full_script(&h);
    let none = json!({"stated": null, "value": null});
    // R12·U3: the run's Read requests carry the piece, so the answer also says the kind.
    let states = |designer: Json| {
        json!({"keys": {DESIGNER: designer, WOODS: none, WIDTH: none},
               "same_kind": true, "page_kind": "chair"})
        .to_string()
    };
    let designer_a = json!({"stated": "Designer A", "value": "Designer A"});
    // Derived: each reply is a captured envelope with `output_text` replaced by our payload,
    // in the order the run calls the model.
    let replies = [
        // R12·U1: one observe call over both main photos.
        (
            OBSERVE_LUNA,
            json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}, "marking_photos": []})
                .to_string(),
        ),
        (
            "openai/stage_read_gpt-6-luna.json",
            states(designer_a.clone()),
        ),
        ("openai/stage_read_gpt-6-luna.json", states(designer_a)),
        ("openai/stage_read_gpt-6-luna.json", states(none.clone())),
        (
            "openai/image_strict_schema_gpt-6-luna.json",
            json!({"verdict": "match"}).to_string(),
        ),
        (
            "openai/image_strict_schema_gpt-6-luna.json",
            json!({"verdict": "match"}).to_string(),
        ),
        (
            "openai/image_strict_schema_gpt-6-luna.json",
            json!({"verdict": "different"}).to_string(),
        ),
        (
            "openai/stage_condition_gpt-6-luna.json",
            json!({"reads": {SCRATCHES: {"value": "Minor", "sure": true}}}).to_string(),
        ),
        (
            "openai/image_strict_schema_gpt-6-luna.json",
            json!({"reasons": {
                DESIGNER: "Two trusted dealers name Designer A.",
                WOODS: "The photo shows teak.",
                SCRATCHES: "Minor scratches are visible.",
            }})
            .to_string(),
        ),
    ];
    let http = Arc::new(FakeResearchHttp::default());
    for (path, text) in &replies {
        http.once(
            "POST",
            "https://api.openai.com/v1/responses",
            Ok(with_output_text(path, text)),
        );
    }
    let model = Arc::new(adapter(&h, &http));
    let service =
        h.service_with(|deps| deps.providers.model = Some(model as Arc<dyn ModelProvider>));
    let mut config = h.stage_config();
    let micro = |unit: &str, n: i64| PriceSnapshot {
        provider: "openai".into(),
        model: "m1".into(),
        unit: unit.into(),
        micro_usd_per_million: n,
    };
    config.prices = vec![
        micro("input_token", 100_000),
        micro("cached_input_token", 10_000),
        micro("cache_write_token", 125_000),
        micro("output_token", 500_000),
    ];
    config.fallbacks.push(FallbackSnapshot {
        provider: "openai".into(),
        model: "m1".into(),
        micro_usd: 1_000,
    });
    config.providers.model = "openai".into();
    let run = h.start_run(&config);
    service.execute(run).unwrap();
    assert_form_filled(&h, run);
    let model_calls: Vec<(String, i64)> = h
        .calls(run)
        .into_iter()
        .filter(|c| c.provider == "openai")
        .map(|c| (c.stage, c.cost_micro_usd))
        .collect();
    let expected: Vec<(String, i64)> = [
        ("observe", 234),
        ("read", 243),
        ("read", 243),
        ("read", 243),
        ("compare", 194),
        ("compare", 194),
        ("compare", 194),
        ("condition", 4076),
        ("decide", 194),
    ]
    .into_iter()
    .map(|(s, c)| (s.to_owned(), c))
    .collect();
    assert_eq!(model_calls, expected);
    // The pages were the script's, read in the script's order.
    let read: Vec<String> = h
        .reader
        .requests()
        .into_iter()
        .filter_map(|c| c.url)
        .collect();
    assert_eq!(read, vec![P_E1, P_E2, P_UNTRUSTED]);
    assert_eq!(mains.len(), 2);
}
