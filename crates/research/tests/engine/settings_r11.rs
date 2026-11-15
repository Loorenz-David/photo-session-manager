//! R11 U2: settings that bound what the engine researches.

use crate::harness::{DESIGNER, E1, E2, Harness, SCRATCHES, WIDTH, WOODS};
use psm_research::{
    api::ResearchApi,
    config::{
        DEFAULT_MODEL_PROVIDER, DEFAULT_PAGE_READER, DEFAULT_VISUAL_SEARCH_PROVIDER,
        ResearchConfig, ResearchSettings,
    },
    domain::{RunState, SiteClass, Stage},
    ports::{
        CallFailure, ModelProvider, ModelReply, ModelRequest, PageReader, PageReply, Providers,
        ResearchImage, SearchCandidate, SearchReply, VisualSearchProvider,
    },
    rules::sites::{TrustedSites, site_of},
    shop::ShopFacts,
};
use serde_json::json;
use std::sync::Arc;

#[test]
fn a25_c8b_excluded_fields_leave_all_requests_and_writes() {
    let h = Harness::chair();
    crate::stages::full_script(&h);
    let mut config = h.stage_config();
    config.excluded_keys = vec![WIDTH.into(), SCRATCHES.into()];
    let run = h.start_run(&config);
    h.execute(run);
    for call in h.model.requests().into_iter().filter(|c| {
        matches!(
            c.stage,
            Some(Stage::Observe | Stage::Read | Stage::Labels | Stage::Condition)
        )
    }) {
        let fields = call.input["fields"].to_string();
        assert!(
            !fields.contains(WIDTH) && !fields.contains(SCRATCHES),
            "{fields}"
        );
    }
    assert!(h.row(WIDTH).is_none());
    assert!(h.row(SCRATCHES).is_none());
    assert!(h.row(WOODS).is_some());
    assert!(h.row(DESIGNER).is_some());
    let steps = h.steps(run);
    for key in [WIDTH, SCRATCHES] {
        assert!(
            steps
                .iter()
                .any(|s| s.field_keys.contains(&key.to_owned())
                    && s.detail.contains("not researched")),
            "{key}"
        );
    }
}

#[test]
fn a25_c8c_all_fields_left_alone_means_nothing_to_research() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    h.patch_settings(json!({"excluded_keys": [DESIGNER, WOODS, WIDTH, SCRATCHES]}));
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert!(h.runs(h.session_id).is_empty());
    assert_eq!(
        h.service.status(h.session_id).unwrap().reason.unwrap().code,
        "NothingToResearch"
    );
}

#[test]
fn a25_c8d_existing_suggestion_in_left_alone_field_is_untouched() {
    let h = Harness::chair();
    let before = {
        h.suggest(WIDTH, json!({"value": "120", "unit": "CENTIMETERS"}));
        h.row(WIDTH).unwrap()
    };
    h.add_main_photo_with_preview(h.session_id);
    let mut config = h.stage_config();
    config.excluded_keys = vec![WIDTH.into()];
    let run = h.start_run(&config);
    h.execute(run);
    assert_eq!(h.row(WIDTH).unwrap(), before);
}

#[test]
fn a25_c9a_extra_hosts_join_snapshot_in_order_and_classify() {
    let settings = ResearchSettings {
        extra_shop_hosts: vec!["www.x.com".into(), "y.myshopify.com".into(), "x.se".into()],
        ..ResearchSettings::default()
    };
    let facts = ShopFacts {
        hosts: vec!["x.se".into(), "x.myshopify.com".into()],
        ..Default::default()
    };
    let config = ResearchConfig::from_settings(&settings, &facts);
    assert_eq!(
        config.shop_hosts,
        vec!["x.se", "x.myshopify.com", "x.com", "y.myshopify.com"]
    );
    assert_eq!(
        site_of(
            "https://www.x.com/products/p",
            &config.shop_hosts,
            &TrustedSites::default()
        )
        .unwrap()
        .class,
        SiteClass::Shop
    );
}

#[test]
fn a25_c9b_www_primary_is_a_shop_host() {
    let site = site_of(
        "https://www.x.se/p",
        &["x.se".into()],
        &TrustedSites::default(),
    )
    .unwrap();
    assert_eq!(site.class, SiteClass::Shop);
}

#[test]
fn a25_c9c_own_listing_on_extra_address_is_never_read() {
    let h = Harness::chair();
    crate::stages::full_script(&h);
    let own = "https://x.com/products/p";
    h.search.set_results(vec![
        SearchCandidate {
            url: own.into(),
            rank: 1,
            title: "Own listing".into(),
        },
        SearchCandidate {
            url: crate::stages::P_E1.into(),
            rank: 2,
            title: "Trusted listing".into(),
        },
    ]);
    let mut config = h.stage_config();
    config.shop_hosts.push("x.com".into());
    config.own_listing_path_tail = vec!["products".into(), "p".into()];
    let run = h.start_run(&config);
    h.execute(run);
    let urls: Vec<_> = h
        .reader
        .requests()
        .into_iter()
        .filter_map(|r| r.url)
        .collect();
    assert!(!urls.iter().any(|url| url == own), "{urls:?}");
    assert!(
        urls.iter().any(|url| url == crate::stages::P_E1),
        "{urls:?}"
    );
}

#[test]
fn a25_c9d_bad_extra_addresses_are_refused() {
    let h = Harness::chair();
    h.patch_settings(json!({"trusted_sites": ["pamono.com"]}));
    for typed in ["localhost", "10.0.0.1", "", "shop.pamono.com"] {
        let patch = json!({"extra_shop_hosts": [typed]});
        let error = h.service.validate_settings(&patch).unwrap_err().to_string();
        assert!(error.contains(typed), "{error}");
        if typed == "shop.pamono.com" {
            assert!(error.contains("already a trusted site"), "{error}");
        }
    }
    h.service
        .validate_settings(&json!({"extra_shop_hosts": ["x.com"]}))
        .unwrap();
}

#[test]
fn a25_c9e_trusted_site_covered_by_extra_host_is_refused() {
    let h = Harness::chair();
    h.patch_settings(json!({"extra_shop_hosts": ["x.com"]}));
    let error = h.service.site_add("shop.x.com").unwrap_err().to_string();
    assert!(error.contains("already covered by x.com"), "{error}");
    assert_eq!(h.service.settings().trusted_sites, vec![E1, E2]);
}

#[test]
fn a25_c10a_reset_keeps_personal_lists_and_restores_defaults() {
    let h = Harness::chair();
    h.patch_settings(json!({"auto": true, "prices": [], "stages": {}, "max_pages": 3,
        "language": "Svenska", "trusted_sites": [E1, E2], "attribution_keys": [DESIGNER],
        "excluded_keys": [WIDTH], "extra_shop_hosts": ["x.com"],
        "detailed_condition": {"key": "custom.grade", "levels": [{"level":"Minor","choice":"Good"}]}}));
    let reset = h.service.settings_reset().unwrap();
    let defaults = serde_json::to_value(ResearchSettings::default()).unwrap();
    for key in [
        "stages",
        "prices",
        "max_pages",
        "max_cost_per_run",
        "max_cost_per_day",
        "source_wait_seconds",
        "record_days",
        "call_timeout_seconds",
        "providers",
    ] {
        assert_eq!(reset[key], defaults[key], "{key}");
    }
    for key in [
        "trusted_sites",
        "attribution_keys",
        "excluded_keys",
        "extra_shop_hosts",
        "detailed_condition",
        "language",
    ] {
        assert_eq!(reset[key], h.research_settings().unwrap()[key], "{key}");
    }
    assert!(reset.get("auto").is_none() && reset.get("quiet_seconds").is_none());
}

#[test]
fn a25_c10b_saved_reset_allows_arranged_run() {
    let h = Harness::chair();
    h.patch_settings(json!({"prices": [], "stages": {}}));
    let reset = h.service.settings_reset().unwrap();
    h.set_research_settings(reset);
    h.add_main_photo_with_preview(h.session_id);
    let service = h.service_with(|deps| {
        deps.providers = Providers {
            model: Some(Arc::new(DefaultModel)),
            search: Some(Arc::new(DefaultSearch)),
            reader: Some(Arc::new(DefaultReader)),
        }
    });
    assert!(service.status(h.session_id).unwrap().reason.is_none());
    assert!(service.arranged(h.session_id).unwrap());
    assert_eq!(h.runs(h.session_id)[0].state, RunState::Queued);
}

struct DefaultModel;
impl ModelProvider for DefaultModel {
    fn id(&self) -> String {
        DEFAULT_MODEL_PROVIDER.into()
    }
    fn call(&self, _: &ModelRequest) -> Result<ModelReply, CallFailure> {
        unreachable!()
    }
}
struct DefaultSearch;
impl VisualSearchProvider for DefaultSearch {
    fn id(&self) -> String {
        DEFAULT_VISUAL_SEARCH_PROVIDER.into()
    }
    fn search(&self, _: &ResearchImage) -> Result<SearchReply, CallFailure> {
        unreachable!()
    }
}
struct DefaultReader;
impl PageReader for DefaultReader {
    fn id(&self) -> String {
        DEFAULT_PAGE_READER.into()
    }
    fn read(&self, _: &str) -> Result<PageReply, CallFailure> {
        unreachable!()
    }
}

#[test]
fn a25_c11a_new_settings_default_empty() {
    let value = ResearchSettings::default();
    assert!(value.excluded_keys.is_empty());
    assert!(value.extra_shop_hosts.is_empty());
    // R12 (owner 2026-11-15, answer A to R11's card): `detailed_condition` ships filled; its row
    // is `a26_owner_detailed_condition_default`.
}

/// Carried into R12 from the owner (2026-11-15): `ResearchSettings::default()` carries the
/// detailed-condition table of answer A: None and Minor give Very Good, Moderate Good, Major
/// Worn, never Excellent; the choices are the shop's own strings for `custom.detailed_condition`.
#[test]
fn a26_owner_detailed_condition_default() {
    use psm_core::domain::attribute::AiConfidence;
    use psm_research::rules::derived_condition::{LevelValue, derive};
    let table = ResearchSettings::default().detailed_condition;
    assert_eq!(table.key, "custom.detailed_condition");
    let grade = psm_testkit::fixtures::definitions::shop_definitions()
        .into_iter()
        .find(|d| d.key == table.key)
        .unwrap();
    let shop_choices = grade.choices.clone();
    assert!(shop_choices[0].starts_with("Excellent"));
    let worst = |levels: &[&str]| {
        let values: Vec<LevelValue> = levels
            .iter()
            .enumerate()
            .map(|(i, v)| LevelValue {
                label: format!("level {i}"),
                value: Some((*v).to_owned()),
                ai_low: false,
            })
            .collect();
        derive(&table.levels, &values, &grade)
    };
    for (levels, expected) in [
        (vec!["None", "N/A"], 1),
        (vec!["None", "Minor", "N/A"], 1),
        (vec!["Minor", "Moderate", "None"], 2),
        (vec!["Moderate", "Major"], 3),
    ] {
        let derived = worst(&levels);
        assert_eq!(
            derived.value,
            Some(json!(shop_choices[expected])),
            "{levels:?}"
        );
        assert_eq!(derived.confidence, Some(AiConfidence::Medium));
    }
    // Excellent is never written.
    assert!(
        table
            .levels
            .iter()
            .all(|l| l.choice.as_deref() != Some(shop_choices[0].as_str()))
    );
}
