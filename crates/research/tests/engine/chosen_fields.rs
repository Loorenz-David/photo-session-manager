//! R13 U1: chosen fields at the research service boundary.

use crate::harness::{DESIGNER, Harness, SCRATCHES, WIDTH, WOODS};
use psm_research::{
    api::ResearchApi,
    config::{ResearchConfig, ResearchSettings},
    domain::{RunState, Stage},
};
use serde_json::json;

#[test]
fn a27_c2a_only_chosen_fields_reach_requests_and_writes() {
    let h = Harness::chair();
    crate::stages::full_script(&h);
    let mut config = h.stage_config();
    config.fill_keys = vec![WOODS.into(), DESIGNER.into()];
    let run = h.start_run(&config);
    h.execute(run);
    for call in h.model.requests() {
        let fields = call.input["fields"].to_string();
        assert!(
            !fields.contains(WIDTH) && !fields.contains(SCRATCHES),
            "{fields}"
        );
    }
    assert!(h.row(WIDTH).is_none() && h.row(SCRATCHES).is_none());
    assert!(h.row(WOODS).is_some() && h.row(DESIGNER).is_some());
    for key in [WIDTH, SCRATCHES] {
        assert!(
            h.steps(run)
                .iter()
                .any(|s| s.field_keys.contains(&key.to_owned())
                    && s.detail.contains("not researched")),
            "{key}"
        );
    }
}

#[test]
fn a27_c2b_an_unchosen_prior_suggestion_is_untouched() {
    let h = Harness::chair();
    h.suggest(WIDTH, json!({"value": "120", "unit": "CENTIMETERS"}));
    let before = h.row(WIDTH).unwrap();
    h.add_main_photo_with_preview(h.session_id);
    let mut config = h.stage_config();
    config.fill_keys = vec![WOODS.into()];
    let run = h.start_run(&config);
    h.execute(run);
    assert_eq!(h.row(WIDTH).unwrap(), before);
    assert!(
        h.steps(run)
            .iter()
            .any(|s| s.field_keys.contains(&WIDTH.into()) && s.detail.contains("not researched"))
    );
}

#[test]
fn a27_c2c_no_chosen_condition_field_skips_stage_six() {
    let h = Harness::chair();
    crate::stages::full_script(&h);
    let mut config = h.stage_config();
    config.fill_keys = vec![WOODS.into()];
    let run = h.start_run(&config);
    h.execute(run);
    assert_eq!(h.run_row(run).state, RunState::Finished);
    assert!(h.row(WOODS).is_some());
    assert!(
        !h.model
            .requests()
            .iter()
            .any(|r| r.stage == Some(Stage::Condition))
    );
    assert!(!h.steps(run).iter().any(|s| s.stage == "condition"));
}

#[test]
fn a27_c2d_a_started_run_keeps_its_chosen_list() {
    let h = Harness::chair();
    crate::stages::full_script(&h);
    let mut config = h.stage_config();
    config.fill_keys = vec![WOODS.into()];
    let run = h.start_run(&config);
    h.patch_settings(json!({"fill_keys": [WOODS, DESIGNER]}));
    h.execute(run);
    let snapshot =
        ResearchConfig::from_snapshot(h.run_row(run).settings_snapshot.as_deref()).unwrap();
    assert_eq!(snapshot.fill_keys, vec![WOODS]);
    assert!(h.row(DESIGNER).is_none());
    assert!(h.steps(run).iter().any(|s| s.field_keys.contains(&DESIGNER.into()) && s.detail.contains("not researched")));
}

#[test]
fn a27_c3d_empty_chosen_list_blocks_run_creation() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    h.patch_settings(json!({"fill_keys": []}));
    assert!(!h.service.arranged(h.session_id).unwrap());
    let error = h.service.run_again(h.session_id).unwrap_err().to_string();
    assert!(
        error.contains("Choose the fields research fills in Settings → Research"),
        "{error}"
    );
    assert!(h.runs(h.session_id).is_empty());
    let status = h.service.status(h.session_id).unwrap();
    assert!(!status.can_run);
    let reason = status.reason.unwrap();
    assert_eq!(reason.code, "NoFieldChosen");
    assert_eq!(
        reason.sentence,
        "Choose the fields research fills in Settings → Research"
    );
}

#[test]
fn a27_c3e_a_key_outside_the_layout_is_not_chosen() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    h.patch_settings(json!({"fill_keys": ["test.outside"]}));
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert!(h.runs(h.session_id).is_empty());
    assert_eq!(
        h.service.status(h.session_id).unwrap().reason.unwrap().code,
        "NoFieldChosen"
    );
}

#[test]
fn a27_c4a_defaults_choose_no_field() {
    let settings = ResearchSettings::default();
    let json = serde_json::to_value(&settings).unwrap();
    assert!(settings.fill_keys.is_empty());
    assert_eq!(json["fill_keys"], json!([]));
    assert!(json.get("excluded_keys").is_none());
}

#[test]
fn a27_c5a_legacy_exclusions_are_ignored() {
    let legacy = json!({"excluded_keys": [WOODS], "max_pages": 7});
    let loaded = ResearchSettings::from_slot(Some(&legacy));
    assert_eq!(loaded.max_pages, 7);
    assert!(loaded.fill_keys.is_empty());
    let serialized = serde_json::to_value(loaded).unwrap();
    assert!(serialized.get("excluded_keys").is_none());
    let (h, service) = crate::defaults::fresh_app();
    h.set_research_settings(legacy);
    assert_eq!(
        service.status(h.session_id).unwrap().reason.unwrap().code,
        "NoFieldChosen"
    );
}

#[test]
fn a27_c5b_reset_preserves_chosen_keys_and_restores_prices() {
    let h = Harness::chair();
    h.patch_settings(json!({"fill_keys": [WOODS, DESIGNER], "prices": [], "stages": {}}));
    let reset = h.service.settings_reset().unwrap();
    let defaults = serde_json::to_value(ResearchSettings::default()).unwrap();
    assert_eq!(reset["fill_keys"], json!([WOODS, DESIGNER]));
    assert_eq!(reset["prices"], defaults["prices"]);
    assert_eq!(reset["stages"], defaults["stages"]);
    assert!(reset.get("excluded_keys").is_none());
}
