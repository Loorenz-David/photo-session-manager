//! R12·U3 (rows A26·C10–C12): the read model also says whether a page lists the same kind of
//! piece; a page that clearly lists another kind is not compared. Full runs on the research
//! harness with scripted fakes; the OpenAI adapter over `FakeResearchHttp`.

use std::sync::Arc;

use psm_core::ports::secrets::SecretStore;
use psm_research::{
    adapters::{http::Body, openai::OpenAiModel},
    domain::Stage,
    fakes::{self, FakeResearchHttp, PageScript, reply_from_capture},
    ports::{ModelProvider, ModelRequest, SearchCandidate},
    rules::sites::page_identity,
    stages::ExecuteOutcome,
};
use psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg;
use serde_json::{Value as Json, json};

use crate::{
    adapter_openai::capture,
    harness::{DESIGNER, Harness},
    stages::{P_E1, P_E2},
};

const READ_LUNA: &str = "openai/stage_read_gpt-6-luna.json";

fn u(url: &str) -> String {
    page_identity(url).unwrap()
}

fn results(h: &Harness, urls: &[&str]) {
    h.search.set_results(
        urls.iter()
            .enumerate()
            .map(|(i, url)| SearchCandidate {
                url: (*url).to_owned(),
                rank: u32::try_from(i + 1).unwrap(),
                title: String::new(),
            })
            .collect(),
    );
}

/// Readable pages with one image each.
fn pages(h: &Harness, urls: &[&str]) {
    for (n, url) in urls.iter().enumerate() {
        let n = u8::try_from(n + 1).unwrap() * 50;
        h.reader.set_page(
            url,
            PageScript::page(
                &format!("Page {n}"),
                vec![SyntheticJpeg::rgb(32, 24, [n, 255 - n, 90])],
            ),
        );
    }
}

/// The page's read answers the kind question (`same_kind`, `page_kind`) and states `keys`.
fn read_answer(h: &Harness, url: &str, keys: Json, same_kind: Option<bool>, kind: &str) {
    h.model.reply(
        Stage::Read,
        &u(url),
        json!({"keys": keys, "same_kind": same_kind, "page_kind": kind}),
    );
}

fn compares(h: &Harness) -> Vec<String> {
    h.model
        .requests()
        .into_iter()
        .filter(|c| c.stage == Some(Stage::Compare))
        .map(|c| c.unit_key)
        .collect()
}

fn read_step(h: &Harness, run: i64, url: &str) -> psm_research::store::StepRow {
    h.steps(run)
        .into_iter()
        .find(|s| s.kind == "read" && s.source_url.as_deref() == Some(url))
        .unwrap()
}

#[test]
fn a26_c10a_a_page_of_another_kind_is_not_compared() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    results(&h, &[P_E1, P_E2]);
    pages(&h, &[P_E1, P_E2]);
    read_answer(&h, P_E1, json!({}), Some(false), "table");
    read_answer(&h, P_E2, json!({}), Some(true), "chair");
    let run = h.start_run(&h.stage_config());
    assert_eq!(h.execute(run), ExecuteOutcome::Finished);
    assert_eq!(compares(&h), vec![u(P_E2)]);
    assert_eq!(
        read_step(&h, run, P_E1).detail,
        "lists another kind of piece (table): not compared"
    );
}

#[test]
fn a26_c10b_the_read_request_names_the_piece() {
    let mut h = Harness::chair();
    let (item, session) = h
        .app
        .items()
        .create_local("R12-1", "Six dining chairs", "Dining Chairs", None)
        .unwrap();
    h.item_id = item.id;
    h.session_id = session.session.id;
    h.add_main_photo_with_preview(h.session_id);
    results(&h, &[P_E1]);
    pages(&h, &[P_E1]);
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    let reads: Vec<_> = h
        .model
        .requests()
        .into_iter()
        .filter(|c| c.stage == Some(Stage::Read))
        .collect();
    assert_eq!(reads.len(), 1);
    assert_eq!(reads[0].input["piece"], json!("Dining Chairs"));
}

#[test]
fn a26_c10c_a_refused_read_still_compares() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    results(&h, &[P_E1]);
    pages(&h, &[P_E1]);
    h.model.fault_for(
        Stage::Read,
        &u(P_E1),
        fakes::refusal("I can't read this page"),
    );
    let run = h.start_run(&h.stage_config());
    assert_eq!(h.execute(run), ExecuteOutcome::Finished);
    assert_eq!(compares(&h), vec![u(P_E1)]);
}

#[test]
fn a26_c10d_a_page_of_another_kind_counts_toward_the_page_limit() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    results(&h, &[P_E1, P_E2]);
    pages(&h, &[P_E1, P_E2]);
    read_answer(&h, P_E1, json!({}), Some(false), "table");
    let mut config = h.stage_config();
    config.max_pages = 1;
    let run = h.start_run(&config);
    h.execute(run);
    let read: Vec<String> = h
        .reader
        .requests()
        .into_iter()
        .filter_map(|c| c.url)
        .collect();
    assert_eq!(read, vec![P_E1.to_owned()]);
    assert!(compares(&h).is_empty());
}

#[test]
fn a26_c11a_another_kind_names_no_designer() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    results(&h, &[P_E1]);
    pages(&h, &[P_E1]);
    read_answer(
        &h,
        P_E1,
        json!({DESIGNER: {"stated": "Designer X", "value": "Designer X"}}),
        Some(false),
        "table",
    );
    let mut config = h.stage_config();
    config.attribution_keys = vec![DESIGNER.into()];
    let run = h.start_run(&config);
    assert_eq!(h.execute(run), ExecuteOutcome::Finished);
    assert!(h.row(DESIGNER).is_none());
    let decide: Vec<_> = h
        .steps(run)
        .into_iter()
        .filter(|s| s.stage == "decide")
        .collect();
    let not_determined = decide.iter().find(|s| s.kind == "not_determined").unwrap();
    assert!(not_determined.field_keys.contains(&DESIGNER.to_owned()));
    for step in &decide {
        assert!(
            !step.detail.contains("Designer X") && !step.summary.contains("Designer X"),
            "{}: {}",
            step.summary,
            step.detail
        );
    }
}

fn read_request(input: Json) -> ModelRequest {
    ModelRequest {
        stage: Stage::Read,
        unit_key: "u".into(),
        model: "m1".into(),
        effort: "low".into(),
        language: "English".into(),
        instructions: "read".into(),
        input,
        images: Vec::new(),
        listing_images: Vec::new(),
    }
}

fn field(key: &str) -> Json {
    json!({"key": key, "label": key, "core_type": "text", "choices": [], "description": null})
}

fn sent_schema_keys(http: &FakeResearchHttp) -> Vec<String> {
    let Body::Json(body) = &http.requests()[0].body else {
        panic!("not a JSON body");
    };
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
fn a26_c12a_the_read_schema_gains_the_kind_answer_only_with_the_piece() {
    let h = Harness::new();
    let model = |http: &Arc<FakeResearchHttp>| {
        OpenAiModel::new(
            http.clone(),
            h.app.secrets.clone() as Arc<dyn SecretStore>,
            h.files(),
        )
    };
    let fields = json!([field("designer"), field("manufacturer")]);
    let page = json!({"url": "https://e1.example/p/1", "title": "Table", "text": "A table."});
    // With the piece: a reply derived from the capture (envelope kept, the answer replaced).
    let answer = json!({
        "keys": {"designer": {"stated": null, "value": null},
                 "manufacturer": {"stated": null, "value": null}},
        "same_kind": false,
        "page_kind": "table",
    });
    let mut derived = capture(READ_LUNA);
    for item in derived["response"]["body"]["output"]
        .as_array_mut()
        .unwrap()
    {
        for content in item["content"].as_array_mut().into_iter().flatten() {
            if content["type"] == "output_text" {
                content["text"] = json!(answer.to_string());
            }
        }
    }
    let http = Arc::new(FakeResearchHttp::default());
    http.once(
        "POST",
        "https://api.openai.com/v1/responses",
        Ok(reply_from_capture(&derived)),
    );
    let with_piece = read_request(json!({"page": page, "fields": fields, "piece": "Chair"}));
    let reply = model(&http).call(&with_piece).unwrap();
    assert_eq!(
        sent_schema_keys(&http),
        vec!["keys", "page_kind", "same_kind"]
    );
    assert_eq!(reply.output["same_kind"], json!(false));
    assert_eq!(reply.output["page_kind"], json!("table"));
    // Without the piece: the capture as it is, accepted as before.
    let http = Arc::new(FakeResearchHttp::default());
    http.once(
        "POST",
        "https://api.openai.com/v1/responses",
        Ok(reply_from_capture(&capture(READ_LUNA))),
    );
    let without = read_request(json!({"page": page, "fields": fields}));
    let reply = model(&http).call(&without).unwrap();
    assert_eq!(sent_schema_keys(&http), vec!["keys"]);
    assert_eq!(
        reply.output,
        json!({"keys": {"designer": {"stated": null, "value": null},
                        "manufacturer": {"stated": null, "value": null}}})
    );
}
