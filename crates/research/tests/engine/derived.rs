//! R11 U3: the derived condition grade and the shop look-alike boundary.

use crate::harness::{DESIGNER, Harness, INTEGRATION, SCRATCHES};
use psm_core::domain::attribute::{
    AiConfidence, AiStatus, AttributeGroup, CoreType, Definition, Preset, PresetField, Provenance,
};
use psm_research::{
    config::{ConditionLevel, DetailedCondition, ResearchConfig},
    domain::Stage,
    fakes::PageScript,
    ports::SearchCandidate,
    rules::sites::page_identity,
};
use psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg;
use serde_json::json;

const GRADE: &str = "custom.detailed_condition";
const VENEER: &str = "custom.veneer";

fn definition(key: &str, label: &str, group: AttributeGroup, choices: &[&str]) -> Definition {
    Definition {
        integration: INTEGRATION.into(),
        key: key.into(),
        label: label.into(),
        core_type: CoreType::Choice,
        choices: choices.iter().map(|v| (*v).into()).collect(),
        validations: json!({}),
        group,
        description: None,
        writable: true,
    }
}

fn setup(h: &Harness) {
    let attrs = h.app.attributes(Some(INTEGRATION));
    let mut definitions: Vec<Definition> = attrs
        .definitions(Some(INTEGRATION))
        .unwrap()
        .into_iter()
        .filter(|d| d.integration == INTEGRATION)
        .collect();
    definitions.push(definition(
        GRADE,
        "Detailed condition",
        AttributeGroup::Details,
        &["A", "B", "C", "D"],
    ));
    definitions.push(definition(
        VENEER,
        "Veneer",
        AttributeGroup::Condition,
        &["N/A", "None", "Minor", "Moderate", "Major"],
    ));
    attrs.import_definitions(INTEGRATION, definitions).unwrap();
    attrs
        .presets_set(Preset {
            item_type: "Chair".into(),
            group: AttributeGroup::Details,
            layout: [DESIGNER, "custom.woods", "custom.width", GRADE]
                .into_iter()
                .map(|key| PresetField {
                    key: key.into(),
                    default: None,
                })
                .collect(),
        })
        .unwrap();
    attrs
        .presets_set(Preset {
            item_type: "Chair".into(),
            group: AttributeGroup::Condition,
            layout: [SCRATCHES, VENEER]
                .into_iter()
                .map(|key| PresetField {
                    key: key.into(),
                    default: None,
                })
                .collect(),
        })
        .unwrap();
}

fn config(h: &Harness) -> ResearchConfig {
    let mut config = h.stage_config();
    config
        .fill_keys
        .extend([GRADE.to_owned(), VENEER.to_owned()]);
    config.detailed_condition = DetailedCondition {
        key: GRADE.into(),
        levels: [
            ("None", Some("A")),
            ("Minor", Some("B")),
            ("Moderate", Some("C")),
            ("Major", Some("D")),
            ("N/A", None),
        ]
        .into_iter()
        .map(|(level, choice)| ConditionLevel {
            level: level.into(),
            choice: choice.map(str::to_owned),
        })
        .collect(),
    };
    config
}

#[test]
fn a25_c14a_condition_grade_uses_the_levels_written_in_this_run() {
    let h = Harness::chair();
    setup(&h);
    crate::stages::full_script(&h);
    h.add_photo_with_preview(
        h.session_id,
        psm_core::domain::photo::Classification::Condition,
    );
    h.model.reply(
        Stage::Condition,
        "condition",
        json!({"reads": {
            SCRATCHES: {"value": "Minor", "sure": true}, VENEER: {"value": "Moderate", "sure": true}
        }}),
    );
    let run = h.start_run(&config(&h));
    h.execute(run);
    let row = h.row(GRADE).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&row.value).unwrap(),
        json!("C")
    );
    assert_eq!(
        (row.provenance, row.ai_status, row.ai_confidence),
        (
            Provenance::Ai,
            Some(AiStatus::Suggested),
            Some(AiConfidence::Medium)
        )
    );
    assert_eq!(
        row.ai_reason.as_deref(),
        Some("From the condition levels: the worst is Moderate (Veneer).")
    );
    assert_eq!(row.ai_sources, Some(vec![]));
    for call in h.model.requests().into_iter().filter(|c| {
        matches!(
            c.stage,
            Some(Stage::Observe | Stage::Read | Stage::Labels | Stage::Condition)
        )
    }) {
        assert!(!call.input["fields"].to_string().contains(GRADE));
    }
    assert_eq!(
        h.steps(run)
            .iter()
            .filter(|s| s.kind == "concluded" && s.field_keys.contains(&GRADE.into()))
            .count(),
        1
    );
}

#[test]
fn a25_c14b_missing_levels_do_not_write_grade() {
    let h = Harness::chair();
    setup(&h);
    h.add_main_photo_with_preview(h.session_id);
    let run = h.start_run(&config(&h));
    h.execute(run);
    assert!(h.row(GRADE).is_none());
    assert!(h.steps(run).iter().any(|s| s.kind == "not_determined"
        && s.field_keys.contains(&GRADE.into())
        && s.detail.contains("no condition level has a value")));
}

#[test]
fn a27_c9a_a_filled_level_derives_the_grade_with_empty_level_count() {
    let h = Harness::chair();
    setup(&h);
    crate::stages::full_script(&h);
    h.add_photo_with_preview(
        h.session_id,
        psm_core::domain::photo::Classification::Condition,
    );
    h.model.reply(
        Stage::Condition,
        "condition",
        json!({"reads": {SCRATCHES: {"value": "Minor", "sure": true}}}),
    );
    let run = h.start_run(&config(&h));
    h.execute(run);
    let row = h.row(GRADE).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&row.value).unwrap(),
        json!("B")
    );
    assert_eq!(
        (row.provenance, row.ai_status, row.ai_confidence),
        (
            Provenance::Ai,
            Some(AiStatus::Suggested),
            Some(AiConfidence::Medium)
        )
    );
    assert_eq!(
        row.ai_reason.as_deref(),
        Some(
            "From the condition levels: the worst is Minor (custom.scratches). 1 of 2 condition levels have no value."
        )
    );
    assert_eq!(row.ai_sources, Some(vec![]));
    assert_eq!(
        h.steps(run)
            .iter()
            .filter(|s| s.kind == "concluded" && s.field_keys.contains(&GRADE.into()))
            .count(),
        1
    );
}

#[test]
fn a27_c9b_no_condition_photo_or_level_leaves_grade_unwritten() {
    let h = Harness::chair();
    setup(&h);
    h.add_main_photo_with_preview(h.session_id);
    let run = h.start_run(&config(&h));
    h.execute(run);
    assert!(h.row(GRADE).is_none());
    assert!(h.steps(run).iter().any(|s| s.kind == "not_determined"
        && s.field_keys.contains(&GRADE.into())
        && s.detail.contains("no condition level has a value")));
}

#[test]
fn a27_c9c_human_level_counts_without_a_condition_model_call() {
    let h = Harness::chair();
    setup(&h);
    h.add_main_photo_with_preview(h.session_id);
    h.add_photo_with_preview(
        h.session_id,
        psm_core::domain::photo::Classification::Condition,
    );
    h.app
        .attributes(Some(INTEGRATION))
        .set_value(h.item_id, SCRATCHES, json!("Minor"), Provenance::Human)
        .unwrap();
    let before = h.row(SCRATCHES);
    let mut cfg = config(&h);
    cfg.fill_keys = vec![GRADE.into(), "custom.woods".into()];
    let run = h.start_run(&cfg);
    h.execute(run);
    let row = h.row(GRADE).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&row.value).unwrap(),
        json!("B")
    );
    assert_eq!(row.ai_confidence, Some(AiConfidence::Medium));
    assert_eq!(h.row(SCRATCHES), before);
    assert!(h.row(VENEER).is_none());
    assert!(
        !h.model
            .requests()
            .iter()
            .any(|c| c.stage == Some(Stage::Condition))
    );
}

#[test]
fn a27_c9d_unchosen_grade_is_not_derived() {
    let h = Harness::chair();
    setup(&h);
    crate::stages::full_script(&h);
    h.add_photo_with_preview(
        h.session_id,
        psm_core::domain::photo::Classification::Condition,
    );
    h.model.reply(Stage::Condition, "condition", json!({"reads": {SCRATCHES: {"value": "Minor", "sure": true}, VENEER: {"value": "Moderate", "sure": true}}}));
    let mut cfg = config(&h);
    cfg.fill_keys.retain(|key| key != GRADE);
    let run = h.start_run(&cfg);
    h.execute(run);
    assert!(h.row(SCRATCHES).is_some() && h.row(VENEER).is_some());
    assert!(h.row(GRADE).is_none());
    assert!(
        h.steps(run)
            .iter()
            .any(|s| s.field_keys.contains(&GRADE.into()) && s.detail.contains("not researched"))
    );
}

#[test]
fn a25_c14c_human_levels_derive_without_condition_photo() {
    let h = Harness::chair();
    setup(&h);
    h.add_main_photo_with_preview(h.session_id);
    let attrs = h.app.attributes(Some(INTEGRATION));
    attrs
        .set_value(h.item_id, SCRATCHES, json!("None"), Provenance::Human)
        .unwrap();
    attrs
        .set_value(h.item_id, VENEER, json!("Minor"), Provenance::Human)
        .unwrap();
    let before = (h.row(SCRATCHES), h.row(VENEER));
    let run = h.start_run(&config(&h));
    h.execute(run);
    let row = h.row(GRADE).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&row.value).unwrap(),
        json!("B")
    );
    assert_eq!(row.ai_confidence, Some(AiConfidence::Medium));
    assert_eq!((h.row(SCRATCHES), h.row(VENEER)), before);
}

#[test]
fn a25_c14d_empty_setting_keeps_grade_visible() {
    let h = Harness::chair();
    setup(&h);
    h.add_main_photo_with_preview(h.session_id);
    let mut config = config(&h);
    config.detailed_condition.key.clear();
    let run = h.start_run(&config);
    h.execute(run);
    assert!(
        h.model
            .requests()
            .iter()
            .any(|c| c.stage == Some(Stage::Observe)
                && c.input["fields"].to_string().contains(GRADE))
    );
}

#[test]
fn a25_c16a_other_shop_product_is_read_but_not_attribution_evidence() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    let shop = "https://beyo.example/products/lookalike";
    let other = "https://other.example/p";
    h.search.set_results(vec![
        SearchCandidate {
            url: shop.into(),
            rank: 1,
            title: "Look-alike".into(),
        },
        SearchCandidate {
            url: other.into(),
            rank: 2,
            title: "Other".into(),
        },
    ]);
    for url in [shop, other] {
        h.reader.set_page(
            url,
            PageScript::page("Listing", vec![SyntheticJpeg::rgb(32, 24, [10, 20, 30])]),
        );
    }
    h.model.reply(
        Stage::Read,
        &page_identity(shop).unwrap(),
        json!({"keys": {DESIGNER: {"stated": "Designer X", "value": "Designer X"}}}),
    );
    h.model.reply(
        Stage::Compare,
        &page_identity(shop).unwrap(),
        json!({"verdict": "match"}),
    );
    h.model.reply(
        Stage::Compare,
        &page_identity(other).unwrap(),
        json!({"verdict": "different"}),
    );
    let mut config = h.stage_config();
    config.attribution_keys = vec![DESIGNER.into()];
    config.own_listing_path_tail = vec!["products".into(), "own".into()];
    let run = h.start_run(&config);
    h.execute(run);
    assert!(h.row(DESIGNER).is_none());
    assert!(h.steps(run).iter().any(|s| s.kind == "read"
        && s.source_url.as_deref() == Some(shop)
        && s.detail == "another product of your shop: not used for designer and maker"));
}
