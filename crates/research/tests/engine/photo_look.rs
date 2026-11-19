//! R12·U1 (rows A26·C2–C7): stage 1 looks at the chosen photos in one model call, in position
//! order, the first Main photo named wide and the others closer. Full runs on the research harness
//! with scripted fakes; the OpenAI adapter over `FakeResearchHttp` replaying R1's captures.

use std::sync::Arc;

use base64::Engine;
use psm_core::{
    domain::{attribute::AiConfidence, photo::Classification},
    ports::secrets::SecretStore,
};
use psm_research::{
    adapters::{http::Body, openai::OpenAiModel},
    api::ResearchApi,
    config::ResearchSettings,
    domain::{FailureKind, RunState, Stage},
    fakes::{FakeResearchHttp, reply_from_capture},
    ports::{ModelProvider, ModelRequest},
    rules::fingerprint::canonical,
    stages::{ExecuteOutcome, OBSERVE_UNIT},
    store::{self as rstore, OutputRow},
};
use serde_json::{Value as Json, json};
use sha2::{Digest, Sha256};

use crate::{
    adapter_openai::capture,
    harness::{DESIGNER, Harness, WOOD, WOODS, kill},
    stages::observe,
};

fn mains(h: &Harness, n: usize) -> Vec<i64> {
    (0..n)
        .map(|_| h.add_main_photo_with_preview(h.session_id))
        .collect()
}

fn observe_requests(h: &Harness) -> Vec<psm_research::fakes::FakeCall> {
    h.model
        .requests()
        .into_iter()
        .filter(|c| c.stage == Some(Stage::Observe))
        .collect()
}

fn looked_steps(h: &Harness, run: i64) -> Vec<rstore::StepRow> {
    h.steps(run)
        .into_iter()
        .filter(|s| s.stage == "observe" && s.kind == "looked")
        .collect()
}

/// The research copy the run sends of a photo at the Observe size.
fn sha(h: &Harness, photo: i64, run: i64) -> String {
    h.images
        .render(photo, run, 1024)
        .unwrap()
        .sha256()
        .to_owned()
}

/// A first run started by "Arranged — next", run to its end; the fakes' logs cleared.
fn first_arranged_run(h: &Harness) -> i64 {
    h.changed();
    h.tick(1_000);
    let run = h.runs(h.session_id)[0].id;
    assert_eq!(h.run_row(run).state, RunState::Finished);
    h.model.clear_requests();
    h.search.clear_requests();
    h.reader.clear_requests();
    run
}

#[test]
fn a26_c2a_one_observe_request_for_three_photos_in_position_order() {
    let h = Harness::chair();
    let photos = mains(&h, 3);
    // The third photo is moved to position 1: position, not id, decides the order.
    h.app.review().reorder(photos[2], 1).unwrap();
    let order = vec![photos[2], photos[0], photos[1]];
    let run = h.start_run(&h.stage_config());
    assert_eq!(h.execute(run), ExecuteOutcome::Finished);
    let requests = observe_requests(&h);
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].images,
        order
            .iter()
            .map(|p| (sha(&h, *p, run), 1024))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        requests[0].input["photos"],
        json!([
            {"index": 1, "role": "wide"},
            {"index": 2, "role": "closer"},
            {"index": 3, "role": "closer"},
        ])
    );
    let looked = looked_steps(&h, run);
    assert_eq!(looked.len(), 1);
    assert_eq!(looked[0].photo_ids, order);
    let call = h
        .calls(run)
        .into_iter()
        .find(|c| c.stage == "observe")
        .unwrap();
    let mut sent: Vec<i64> = h
        .outbound(run)
        .into_iter()
        .filter(|o| o.call_id == call.id)
        .map(|o| o.photo_id)
        .collect();
    sent.sort();
    let mut expected = photos.clone();
    expected.sort();
    assert_eq!(sent, expected);
}

#[test]
fn a26_c2b_ten_photos_default_limit_sends_eight() {
    let h = Harness::chair();
    let photos = mains(&h, 10);
    let config = h.stage_config();
    assert_eq!(config.observe_max_photos, 8);
    let run = h.start_run(&config);
    h.execute(run);
    let requests = observe_requests(&h);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].images.len(), 8);
    let looked = looked_steps(&h, run);
    assert_eq!(looked.len(), 1);
    assert_eq!(looked[0].photo_ids, photos[..8].to_vec());
    assert!(
        looked[0]
            .detail
            .contains("2 photos were not looked at (the limit is 8)"),
        "{}",
        looked[0].detail
    );
}

#[test]
fn a26_c3a_the_observe_read_writes_a_visible_field() {
    let h = Harness::chair();
    mains(&h, 2);
    observe(
        &h,
        json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}, "marking_photos": []}),
    );
    let run = h.start_run(&h.stage_config());
    assert_eq!(h.execute(run), ExecuteOutcome::Finished);
    let row = h.row(WOODS).expect("woods written");
    assert_eq!(row.value, "[\"Teak\"]");
    assert_eq!(row.ai_confidence, Some(AiConfidence::Medium));
    let concluded: Vec<_> = h
        .steps(run)
        .into_iter()
        .filter(|s| s.kind == "concluded" && s.field_keys == vec![WOODS.to_owned()])
        .collect();
    assert_eq!(concluded.len(), 1);
}

#[test]
fn a26_c3b_a_listed_marking_photo_gets_one_labels_request() {
    let h = Harness::chair();
    let photos = mains(&h, 3);
    observe(&h, json!({"reads": {}, "marking_photos": [3]}));
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    let labels: Vec<_> = h
        .model
        .requests()
        .into_iter()
        .filter(|c| c.stage == Some(Stage::Labels))
        .collect();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].unit_key, format!("photo:{}", photos[2]));
    let third = h.images.render(photos[2], run, 2048).unwrap();
    assert_eq!(labels[0].images, vec![(third.sha256().to_owned(), 2048)]);
}

#[test]
fn a26_c3c_an_index_outside_the_photos_sent_is_ignored() {
    let h = Harness::chair();
    mains(&h, 3);
    observe(&h, json!({"reads": {}, "marking_photos": [5]}));
    let run = h.start_run(&h.stage_config());
    assert_eq!(h.execute(run), ExecuteOutcome::Finished);
    assert!(
        h.model
            .requests()
            .iter()
            .all(|c| c.stage != Some(Stage::Labels))
    );
    assert_eq!(h.run_row(run).state, RunState::Finished);
    let looked = looked_steps(&h, run);
    assert!(
        looked[0]
            .detail
            .contains("marking photo 5 ignored: only 3 photos were sent"),
        "{}",
        looked[0].detail
    );
}

#[test]
fn a26_c4a_a_new_condition_photo_reuses_the_observe_unit() {
    let h = Harness::chair();
    mains(&h, 2);
    let first = first_arranged_run(&h);
    h.add_photo_with_preview(h.session_id, Classification::Condition);
    // A19: arranging queues nothing after a finished run; a queued automatic run (an older
    // build's follow-up) still reuses the unchanged observe unit.
    assert!(!h.service.arranged(h.session_id).unwrap());
    h.queue_follow_up();
    h.tick(1_000);
    let second = h.runs(h.session_id).last().unwrap().id;
    assert_ne!(second, first);
    assert_eq!(h.run_row(second).state, RunState::Finished);
    assert!(observe_requests(&h).is_empty());
    let stage1: Vec<OutputRow> = h
        .outputs(second)
        .into_iter()
        .filter(|o| o.stage == "observe")
        .collect();
    assert_eq!(stage1.len(), 1);
    assert_eq!(stage1[0].reused_from_run_id, Some(first));
}

#[test]
fn a26_c4b_a_recomposed_photo_costs_one_observe_call_over_all() {
    let h = Harness::chair();
    let photos = mains(&h, 3);
    first_arranged_run(&h);
    h.recompose(photos[1]);
    assert!(!h.service.arranged(h.session_id).unwrap());
    h.queue_follow_up();
    h.tick(1_000);
    let second = h.runs(h.session_id).last().unwrap().id;
    assert_eq!(h.run_row(second).state, RunState::Finished);
    let requests = observe_requests(&h);
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].images,
        photos
            .iter()
            .map(|p| (sha(&h, *p, second), 1024))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a26_c4c_a_photo_beyond_the_limit_changes_nothing() {
    let h = Harness::chair();
    let photos = mains(&h, 10);
    first_arranged_run(&h);
    assert_eq!(h.photo(photos[9]).position, Some(10));
    h.recompose(photos[9]);
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert_eq!(h.runs(h.session_id).len(), 1);
}

#[test]
fn a26_c5a_kill_while_the_observe_call_is_in_flight() {
    let mut h = Harness::chair();
    mains(&h, 2);
    observe(
        &h,
        json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}, "marking_photos": []}),
    );
    h.model.before_call(|call| {
        if call.stage == Some(Stage::Observe) {
            kill();
        }
    });
    h.changed();
    h.tick_killed(1_000);
    h.tick(0);
    let run = h.runs(h.session_id)[0].id;
    assert_eq!(h.run_row(run).state, RunState::Finished);
    let calls: Vec<(String, i64)> = h
        .calls(run)
        .into_iter()
        .filter(|c| c.stage == "observe")
        .map(|c| (c.state, c.attempt))
        .collect();
    assert_eq!(
        calls,
        vec![("unknown".to_owned(), 1), ("answered".to_owned(), 2)]
    );
    let stage1 = h
        .outputs(run)
        .into_iter()
        .filter(|o| o.stage == "observe")
        .count();
    assert_eq!(stage1, 1);
    assert_eq!(looked_steps(&h, run).len(), 1);
}

#[test]
fn a26_c6a_observe_max_photos_default_and_validation() {
    assert_eq!(ResearchSettings::default().observe_max_photos, 8);
    let h = Harness::new();
    let refused = h
        .service
        .validate_settings(&json!({"observe_max_photos": 0}))
        .unwrap_err();
    assert!(
        refused.to_string().contains("observe_max_photos"),
        "{refused}"
    );
    h.service
        .validate_settings(&json!({"observe_max_photos": 3}))
        .unwrap();
}

/// A fingerprint as R11 wrote it: version 1 of MC-A7's `F`.
fn fingerprint_v1(stage: u8, unit: &Json) -> String {
    let text = canonical(&json!({"stage": stage, "unit": unit, "v": 1})).unwrap();
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn a26_c6b_units_of_version_1_are_not_reused() {
    let h = Harness::chair();
    let photos = mains(&h, 2);
    let revision = |p: i64| h.app.composition(p).unwrap().revision;
    // The latest earlier run, as R11 wrote it: one Observe unit per photo and the cover's Find
    // unit, each with a version 1 fingerprint over R11's identity; then it failed.
    let earlier = h.insert_run(RunState::Failed, None, None);
    let now = h.now();
    let mut rows: Vec<(String, String, Json, Json)> = photos
        .iter()
        .map(|p| {
            (
                "observe".to_owned(),
                format!("photo:{p}"),
                json!({
                    "photo_id": p, "composition_rev": revision(*p), "long_edge": 1024,
                    "model": "m1", "effort": "low", "instructions_version": 1,
                    "language": "English",
                }),
                json!({"photo_id": p, "composition_rev": revision(*p), "reads": {},
                       "shows_marking": false, "files": []}),
            )
        })
        .collect();
    rows.push((
        "find".to_owned(),
        "cover".to_owned(),
        json!({"cover_photo_id": photos[0], "composition_rev": revision(photos[0]),
               "provider": "fake-search"}),
        json!({"cover": photos[0], "candidates": [], "files": []}),
    ));
    for (stage, unit, identity, output) in &rows {
        let number = if stage == "observe" { 1 } else { 2 };
        h.query(|c| {
            rstore::insert_output(
                c,
                &OutputRow {
                    run_id: earlier,
                    stage: stage.clone(),
                    unit_key: unit.clone(),
                    fingerprint: fingerprint_v1(number, identity),
                    output: output.to_string(),
                    reused_from_run_id: None,
                    committed_at: now,
                },
            )
            .unwrap()
        });
    }
    h.changed();
    h.tick(1_000);
    let run = h.runs(h.session_id).last().unwrap().clone();
    assert_ne!(run.id, earlier);
    assert_eq!(h.run_row(run.id).state, RunState::Finished);
    let snapshot: Json = serde_json::from_str(run.settings_snapshot.as_deref().unwrap()).unwrap();
    assert_eq!(snapshot["fingerprint_version"], json!(2));
    assert_eq!(snapshot["instructions_version"], json!(3));
    assert!(
        h.outputs(run.id)
            .iter()
            .all(|o| o.reused_from_run_id.is_none())
    );
    assert_eq!(observe_requests(&h).len(), 1);
    assert_eq!(h.search.requests().len(), 1);
}

const OBSERVE_LUNA: &str = "openai/stage_observe_gpt-6-luna.json";

fn field(key: &str, core_type: &str, choices: &[&str]) -> Json {
    json!({"key": key, "label": key, "core_type": core_type, "choices": choices, "description": null})
}

fn adapter_request(input: Json, images: Vec<psm_research::images::ResearchImage>) -> ModelRequest {
    ModelRequest {
        stage: Stage::Observe,
        unit_key: OBSERVE_UNIT.into(),
        model: "m1".into(),
        effort: "low".into(),
        language: "English".into(),
        instructions: "look".into(),
        input,
        images,
        listing_images: Vec::new(),
    }
}

/// Derived: the observe capture's reply with the text of its `output_text` replaced.
fn observe_reply_with(text: &str) -> psm_research::adapters::http::HttpReply {
    let mut c = capture(OBSERVE_LUNA);
    for item in c["response"]["body"]["output"].as_array_mut().unwrap() {
        for content in item["content"].as_array_mut().into_iter().flatten() {
            if content["type"] == "output_text" {
                content["text"] = json!(text);
            }
        }
    }
    reply_from_capture(&c)
}

fn openai(h: &Harness, http: &Arc<FakeResearchHttp>) -> OpenAiModel {
    OpenAiModel::new(
        http.clone(),
        h.app.secrets.clone() as Arc<dyn SecretStore>,
        h.files(),
    )
}

fn sent_body(http: &FakeResearchHttp) -> Json {
    match &http.requests()[0].body {
        Body::Json(body) => body.clone(),
        other => panic!("not a JSON body: {other:?}"),
    }
}

fn schema_keys(body: &Json) -> Vec<String> {
    let mut keys: Vec<String> = body["text"]["format"]["schema"]["properties"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    keys.sort();
    keys
}

#[test]
fn a26_c7a_three_photos_in_one_request_with_marking_photos() {
    let h = Harness::new();
    let photos = mains(&h, 3);
    let run = h.running_run(&h.config());
    let images: Vec<_> = photos
        .iter()
        .map(|p| h.images.render(*p, run, 1024).unwrap())
        .collect();
    let request = adapter_request(
        json!({
            "photos": [{"index": 1, "role": "wide"}, {"index": 2, "role": "closer"},
                       {"index": 3, "role": "closer"}],
            "fields": [field(WOOD, "choice", &["Oak", "Teak"])],
        }),
        images.clone(),
    );
    let answer = json!({"reads": {WOOD: {"value": "Teak", "sure": true}}, "marking_photos": [2],
                        "note": []});
    let http = Arc::new(FakeResearchHttp::default());
    http.once(
        "POST",
        "https://api.openai.com/v1/responses",
        Ok(observe_reply_with(&answer.to_string())),
    );
    let reply = openai(&h, &http).call(&request).unwrap();
    let body = sent_body(&http);
    // The body shape of the six-image condition capture: one text part, then the images in order.
    let content = body["input"][0]["content"].as_array().unwrap();
    let kinds: Vec<&str> = content
        .iter()
        .map(|c| c["type"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        vec!["input_text", "input_image", "input_image", "input_image"]
    );
    for (part, image) in content[1..].iter().zip(&images) {
        let expected = format!(
            "data:image/jpeg;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(image.bytes().unwrap())
        );
        assert_eq!(part["image_url"], json!(expected));
    }
    assert_eq!(schema_keys(&body), vec!["marking_photos", "note", "reads"]);
    let schema = &body["text"]["format"]["schema"];
    assert_eq!(
        schema["properties"]["marking_photos"],
        json!({"type": "array", "items": {"type": "integer"}})
    );
    assert_eq!(schema["required"].as_array().unwrap().len(), 3);
    assert_eq!(body["text"]["format"]["strict"], json!(true));
    assert_eq!(reply.output["marking_photos"], json!([2]));
}

#[test]
fn a26_c7b_a_request_without_photos_keeps_the_one_photo_schema() {
    let h = Harness::new();
    let request = adapter_request(
        json!({"fields": [field(WOOD, "choice", &["Oak", "Teak"]), field(DESIGNER, "text", &[])]}),
        Vec::new(),
    );
    // As captured: R1's version-0 answer (`fields`, `piece_type`) is refused by the schema check
    // exactly as before R12 (no engine Observe schema has a `fields` member).
    let http = Arc::new(FakeResearchHttp::default());
    http.once(
        "POST",
        "https://api.openai.com/v1/responses",
        Ok(reply_from_capture(&capture(OBSERVE_LUNA))),
    );
    let as_captured = openai(&h, &http).call(&request).unwrap_err();
    assert_eq!(as_captured.kind, FailureKind::Schema);
    assert_eq!(
        schema_keys(&sent_body(&http)),
        vec!["reads", "shows_marking"]
    );
    // An answer in the schema the request carries is accepted as before.
    let answer = json!({
        "reads": {WOOD: {"value": "Oak", "sure": true}, DESIGNER: {"value": null, "sure": false}},
        "shows_marking": false,
    });
    let http = Arc::new(FakeResearchHttp::default());
    http.once(
        "POST",
        "https://api.openai.com/v1/responses",
        Ok(observe_reply_with(&answer.to_string())),
    );
    let reply = openai(&h, &http).call(&request).unwrap();
    assert_eq!(reply.output, answer);
}
