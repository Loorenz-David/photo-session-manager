//! R4·U6: the `ResearchApi` methods on the harness (the Tauri commands are thin pass-throughs
//! frozen in R2·U2 and connected in R7·U1).

use psm_core::{
    Error,
    domain::{
        attribute::{AiConfidence, AiStatus, AttributeGroup, Preset, PresetField},
        photo::Classification,
    },
    ports::secrets::SecretStore,
};
use psm_research::{
    api::{ResearchApi, SecretsInput},
    domain::{RunState, RunTrigger, Stage},
    fakes::PageScript,
    ports::SearchCandidate,
    rules::sites::page_identity,
    service::{MODEL_ACCOUNT, SEARCH_ACCOUNT},
};
use psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg;
use serde_json::json;

use crate::harness::{DESIGNER, Harness, INTEGRATION, SCRATCHES, WIDTH, WOODS};

fn api(h: &Harness) -> &dyn ResearchApi {
    h.service.as_ref()
}

fn refusal(result: psm_core::Result<serde_json::Value>) -> String {
    match result {
        Err(Error::InvalidData(text)) => text,
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// Pages on `urls`, each stating designer A, compared `match`.
fn listing_pages(h: &Harness, urls: &[&str]) {
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
    for (n, url) in urls.iter().enumerate() {
        let n = u8::try_from(n).unwrap();
        h.reader.set_page(
            url,
            PageScript::page(url, vec![SyntheticJpeg::rgb(20, 20, [n * 40, 10, 99])]),
        );
        let unit = page_identity(url).unwrap();
        h.model.reply(
            Stage::Read,
            &unit,
            json!({"keys": {DESIGNER: {"stated": "Designer A", "value": "Designer A"}}}),
        );
        h.model
            .reply(Stage::Compare, &unit, json!({"verdict": "match"}));
    }
}

/// A finished run that wrote designer, woods and scratches.
fn three_written() -> Harness {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    h.add_photo_with_preview(h.session_id, Classification::Condition);
    listing_pages(&h, &["https://e1.example/a", "https://e2.example/b"]);
    h.model.reply(
        Stage::Observe,
        psm_research::stages::OBSERVE_UNIT,
        json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}}),
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
    h
}

#[test]
fn a18_c1a_status_counts_filled_and_to_review() {
    let h = three_written();
    let mut woods = h.row(WOODS).unwrap();
    woods.ai_status = Some(AiStatus::Accepted);
    h.put(&woods);
    let mut designer = h.row(DESIGNER).unwrap();
    designer.ai_confidence = Some(AiConfidence::Low);
    h.put(&designer);
    let status = api(&h).status(h.session_id).unwrap();
    assert_eq!((status.fields_filled, status.to_review), (3, 2));
    assert_eq!(status.run.unwrap().state, "finished");
}

#[test]
fn a18_c1b_a_field_removed_from_the_layout_leaves_to_review() {
    let h = three_written();
    let before = api(&h).status(h.session_id).unwrap().to_review;
    h.app
        .attributes(Some(INTEGRATION))
        .presets_set(Preset {
            item_type: "Chair".into(),
            group: AttributeGroup::Details,
            layout: [DESIGNER, WIDTH]
                .into_iter()
                .map(|key| PresetField {
                    key: key.into(),
                    default: None,
                })
                .collect(),
        })
        .unwrap();
    assert_eq!(api(&h).status(h.session_id).unwrap().to_review, before - 1);
}

#[test]
fn a18_c1c_no_model_key() {
    let h = Harness::chair();
    h.app.secrets.delete(MODEL_ACCOUNT).unwrap();
    let status = api(&h).status(h.session_id).unwrap();
    let reason = status.reason.unwrap();
    assert_eq!(
        (reason.code.as_str(), reason.sentence.as_str()),
        ("NoModelKey", "Add the model key in Settings → Research")
    );
    assert!(!status.can_run);
}

#[test]
fn a18_c6a_run_queues_a_manual_run() {
    let h = three_written();
    api(&h).run(h.session_id).unwrap();
    let runs = h.runs(h.session_id);
    assert_eq!(runs.len(), 2);
    assert_eq!(
        (runs[1].state, runs[1].trigger),
        (RunState::Queued, RunTrigger::Manual)
    );
}

#[test]
fn a19_c1a_keys_go_to_the_secrets_port_only() {
    let h = Harness::chair();
    h.app.secrets.delete(MODEL_ACCOUNT).unwrap();
    h.app.secrets.delete(SEARCH_ACCOUNT).unwrap();
    let key = "sk-psm-ENTERED-key-4d2e9a";
    api(&h)
        .secrets_set(SecretsInput {
            model_key: Some(key.into()),
            search_key: None,
            reader_key: None,
        })
        .unwrap();
    assert_eq!(
        h.app.secrets.get(MODEL_ACCOUNT).unwrap().as_deref(),
        Some(key)
    );
    let state = api(&h).secrets_state().unwrap();
    assert_eq!(
        (state.model, state.search, state.reader),
        (true, false, false)
    );
    let settings = std::fs::read_to_string(h.app.app.paths.settings()).unwrap();
    assert!(!settings.contains(key));
}

#[test]
fn a19_c1b_a_saved_key_lifts_the_prerequisite() {
    let h = Harness::chair();
    h.app.secrets.delete(MODEL_ACCOUNT).unwrap();
    h.add_main_photo_with_preview(h.session_id);
    h.changed();
    assert_eq!(
        api(&h).status(h.session_id).unwrap().reason.unwrap().code,
        "NoModelKey"
    );
    api(&h)
        .secrets_set(SecretsInput {
            model_key: Some("sk-psm-ENTERED-key-77".into()),
            search_key: None,
            reader_key: None,
        })
        .unwrap();
    let status = api(&h).status(h.session_id).unwrap();
    assert!(status.reason.is_none_or(|r| r.code != "NoModelKey"));
    assert!(status.can_run);
}

#[test]
fn a19_c2a_a_site_is_added_last() {
    let h = Harness::chair();
    let value = api(&h).site_add("https://www.pamono.com/x").unwrap();
    assert_eq!(
        value["trusted_sites"],
        json!(["e1.example", "e2.example", "pamono.com"])
    );
    h.set_research_settings(value);
    assert_eq!(
        h.research_settings().unwrap()["trusted_sites"]
            .as_array()
            .unwrap()
            .last()
            .unwrap(),
        "pamono.com"
    );
}

#[test]
fn a19_c2b_covered_sites_are_refused() {
    let h = Harness::chair();
    h.patch_settings(json!({"trusted_sites": ["etsy.com"]}));
    assert_eq!(
        refusal(api(&h).site_add("shop.etsy.com")),
        "already covered by etsy.com"
    );
    h.app
        .fake_shop()
        .set_public_listing(vec!["beyo.example".into()], None, String::new());
    assert_eq!(
        refusal(api(&h).site_add("beyo.example")),
        "already covered by beyo.example"
    );
}

#[test]
fn a19_c2c_reorder_and_remove_reach_runs_started_later() {
    let h = Harness::chair();
    h.patch_settings(json!({"trusted_sites": ["e1.example", "e2.example", "e3.example"]}));
    let (p1, p2, p3) = (
        "https://e1.example/one",
        "https://e2.example/two",
        "https://e3.example/three",
    );
    listing_pages(&h, &[p1, p2, p3]);
    h.add_main_photo_with_preview(h.session_id);
    h.changed();
    h.app.clock.advance(1_000);
    h.service.tick().unwrap();
    let before = h.runs(h.session_id)[0].id;
    let value = api(&h)
        .sites_reorder(&[
            "e3.example".into(),
            "e1.example".into(),
            "e2.example".into(),
        ])
        .unwrap();
    h.set_research_settings(value);
    let value = api(&h).site_remove("e2.example").unwrap();
    h.set_research_settings(value);
    api(&h).settings_changed();
    assert_eq!(
        h.research_settings().unwrap()["trusted_sites"],
        json!(["e3.example", "e1.example"])
    );
    h.run_jobs();
    assert_eq!(h.run_row(before).state, RunState::Finished);
    let read_order = |h: &Harness| -> Vec<String> {
        h.reader
            .requests()
            .into_iter()
            .map(|c| c.url.unwrap())
            .collect()
    };
    assert_eq!(read_order(&h), vec![p1, p2, p3]);
    h.reader.clear_requests();
    api(&h).run(h.session_id).unwrap();
    h.tick(0);
    assert_eq!(read_order(&h), vec![p3, p1, p2]);
}

#[test]
fn a19_c3a_settings_validation() {
    let h = Harness::chair();
    let ok = json!({"max_cost_per_run": "0.50"});
    api(&h).validate_settings(&ok).unwrap();
    h.patch_settings(ok);
    assert_eq!(
        h.research_settings().unwrap()["max_cost_per_run"],
        json!("0.50")
    );
    for refused in [
        json!({"max_cost_per_run": "0,5"}),
        json!({"max_cost_per_run": "0.0000001"}),
        json!({"record_days": 1}),
    ] {
        assert!(
            matches!(
                api(&h).validate_settings(&refused),
                Err(Error::InvalidData(_))
            ),
            "{refused}"
        );
    }
}
