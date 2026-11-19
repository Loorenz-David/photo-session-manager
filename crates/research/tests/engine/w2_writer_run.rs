//! W2 (book 08C §5–§7; contracts WC2, WC3, WC7 of `master_plan_writer.md` §5): the writer's
//! input (unit 1, rows W2·C4) and the run's life (unit 3, rows W2·C12–C15). Runs on the research
//! harness with the crate's fakes; the outcome is the built input, the answers of `request`, the
//! run rows and their snapshots.

use psm_core::domain::{
    attribute::{AiConfidence, AiStatus, AttributeGroup, CoreType, Provenance, Value},
    research::FeedbackAction,
};
use psm_research::{
    domain::{RunState, RunTrigger, Stage},
    store::{self as rstore, NewRun, OutputRow},
    writer::{config::WriterSettings, input},
};
use serde_json::{Value as Json, json};

use crate::{
    harness::{DESIGNER, Harness, WOOD},
    w1_note::{add_definitions, definition, layout},
};

pub(crate) const ERA: &str = "custom.era";
pub(crate) const DESCRIPTION: &str = "product.description";

/// An item of type Writer whose Details step is designer, era, wood and whose Condition step is
/// empty; the Product step is the default one (title, description, tags, the subtitle).
pub(crate) fn writer_harness() -> Harness {
    let mut h = Harness::new();
    add_definitions(
        &h,
        vec![definition(
            ERA,
            "Era",
            CoreType::Text,
            AttributeGroup::Details,
        )],
    );
    layout(
        &h,
        "Writer",
        AttributeGroup::Details,
        &[DESIGNER, ERA, WOOD],
    );
    layout(&h, "Writer", AttributeGroup::Condition, &[]);
    let (item, session) = h
        .app
        .items()
        .create_local("W2-1", "Teak chair", "Writer", None)
        .unwrap();
    h.item_id = item.id;
    h.session_id = session.session.id;
    h
}

/// A stored row of the item.
pub(crate) fn put(
    h: &Harness,
    key: &str,
    value: Json,
    provenance: Provenance,
    status: Option<AiStatus>,
    confidence: Option<AiConfidence>,
) {
    h.put(&Value {
        item_id: h.item_id,
        key: key.into(),
        value: value.to_string(),
        valid: true,
        provenance,
        ai_confidence: confidence,
        ai_status: status,
        ai_reason: None,
        ai_sources: None,
        ai_prior: None,
        updated_at: h.now(),
    });
}

fn build(h: &Harness, settings: &WriterSettings) -> Json {
    input::build(&h.service, h.session_id, settings).unwrap()
}

#[test]
fn w2_c4a_fields_carry_their_decision_and_whether_they_are_facts() {
    let h = writer_harness();
    put(
        &h,
        DESIGNER,
        json!("Børge Mogensen"),
        Provenance::Ai,
        Some(AiStatus::Accepted),
        Some(AiConfidence::Medium),
    );
    put(
        &h,
        ERA,
        json!("1950s"),
        Provenance::Ai,
        Some(AiStatus::Suggested),
        Some(AiConfidence::Low),
    );
    put(&h, WOOD, json!("Oak"), Provenance::Default, None, None);
    assert_eq!(
        build(&h, &WriterSettings::default())["fields"],
        json!([
            {"key": DESIGNER, "label": DESIGNER, "value": "Børge Mogensen",
             "decision": "accepted", "fact": true},
            {"key": ERA, "label": "Era", "value": "1950s", "decision": "unchecked", "fact": false},
            {"key": WOOD, "label": WOOD, "value": "Oak", "decision": "shop_default", "fact": true},
        ])
    );
}

/// A finished research run of the session whose stage 3 read `pages` (title, verdict) in order.
fn research_run_with_pages(h: &Harness, pages: &[(&str, Option<&str>)]) -> i64 {
    let now = h.now();
    h.query(|c| {
        let run = rstore::insert_run(
            c,
            &NewRun {
                item_id: h.item_id,
                session_id: h.session_id,
                trigger: RunTrigger::Manual,
                state: RunState::Finished,
                queued_at: now,
                started_at: Some(now),
                settings_snapshot: None,
                form_snapshot: None,
                input_snapshot: None,
            },
        )
        .unwrap();
        let output = |stage: Stage, unit: &str, output: Json| OutputRow {
            run_id: run,
            stage: stage.as_str().into(),
            unit_key: unit.into(),
            fingerprint: format!("{unit}-{run}"),
            output: output.to_string(),
            reused_from_run_id: None,
            committed_at: now,
        };
        for (title, _) in pages {
            let unit = format!("u-{title}");
            rstore::insert_output(
                c,
                &output(
                    Stage::Read,
                    &unit,
                    json!({"unit": unit, "title": title, "obtained": true, "statements": {}}),
                ),
            )
            .unwrap();
        }
        for (title, verdict) in pages {
            if let Some(verdict) = verdict {
                let unit = format!("u-{title}");
                rstore::insert_output(
                    c,
                    &output(
                        Stage::Compare,
                        &unit,
                        json!({"unit": unit, "verdict": verdict}),
                    ),
                )
                .unwrap();
            }
        }
        run
    })
}

#[test]
fn w2_c4b_listings_are_the_match_and_similar_titles_match_first_at_most_eight() {
    let h = writer_harness();
    let mut pages: Vec<(String, Option<&str>)> = vec![
        ("p1".into(), Some("similar")),
        ("p2".into(), Some("different")),
        ("p3".into(), Some("match")),
    ];
    for i in 4..=12 {
        pages.push((format!("p{i}"), Some("similar")));
    }
    let pages: Vec<(&str, Option<&str>)> = pages.iter().map(|(t, v)| (t.as_str(), *v)).collect();
    research_run_with_pages(&h, &pages);
    let listings = build(&h, &WriterSettings::default())["listings"].clone();
    let titles: Vec<&str> = listings
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, ["p3", "p1", "p4", "p5", "p6", "p7", "p8", "p9"]);
    assert_eq!(listings[0]["verdict"], json!("match"));
    assert_eq!(listings[1]["verdict"], json!("similar"));
}

/// Inserts a `writer_findings` row of the harness's item.
#[allow(clippy::too_many_arguments)]
pub(crate) fn finding_row(
    h: &Harness,
    finding_type: &str,
    keys: &[&str],
    identity: &str,
    text: &str,
    state: &str,
    answer: Option<&str>,
    resolved_at: Option<i64>,
) -> i64 {
    let kind = if finding_type == "question" {
        "question"
    } else {
        "finding"
    };
    let (item, session, now) = (h.item_id, h.session_id, h.now());
    h.query(|c| {
        c.execute(
            "INSERT INTO writer_findings(item_id,session_id,run_id,kind,type,keys,text_field,\
             identity,text,mode,state,answer,created_at,resolved_at) \
             VALUES (?1,?2,0,?3,?4,?5,'',?6,?7,'ask',?8,?9,?10,?11)",
            rusqlite::params![
                item,
                session,
                kind,
                finding_type,
                serde_json::to_string(keys).unwrap(),
                identity,
                text,
                state,
                answer,
                now,
                resolved_at,
            ],
        )
        .unwrap();
        c.last_insert_rowid()
    })
}

#[test]
fn w2_c4c_answers_are_the_answered_findings_oldest_first() {
    let h = writer_harness();
    let t = h.now();
    finding_row(
        &h,
        "field_vs_photos",
        &[WOOD],
        "field_vs_photos|custom.wood|",
        "The photos show oak; is the wood teak?",
        "answered",
        Some("It is teak"),
        Some(t + 10),
    );
    finding_row(
        &h,
        "field_vs_field",
        &[ERA],
        "field_vs_field|custom.era|",
        "The era does not fit the designer",
        "dismissed",
        None,
        Some(t + 20),
    );
    finding_row(
        &h,
        "question",
        &[DESIGNER],
        "question|custom.designer||who made it",
        "Who made it?",
        "answered",
        Some("FDB Møbler"),
        Some(t + 30),
    );
    assert_eq!(
        build(&h, &WriterSettings::default())["answers"],
        json!([
            {"type": "field_vs_photos", "keys": [WOOD],
             "question": "The photos show oak; is the wood teak?", "answer": "It is teak"},
            {"type": "question", "keys": [DESIGNER], "question": "Who made it?",
             "answer": "FDB Møbler"},
        ])
    );
}

#[test]
fn w2_c4d_rejected_holds_the_dismissed_and_kept_previous_texts() {
    let h = writer_harness();
    put(
        &h,
        DESCRIPTION,
        json!("Old text"),
        Provenance::Ai,
        Some(AiStatus::Cleared),
        Some(AiConfidence::Medium),
    );
    h.feedback(
        DESCRIPTION,
        FeedbackAction::KeptPrevious,
        json!("Other text"),
    );
    assert_eq!(
        build(&h, &WriterSettings::default())["rejected"],
        json!([
            {"text": "description", "value": "Old text"},
            {"text": "description", "value": "Other text"},
        ])
    );
}

#[test]
fn w2_c4e_without_the_report_field_only_title_and_description_are_requested() {
    let h = writer_harness();
    assert_eq!(
        build(&h, &WriterSettings::default())["texts"],
        json!(["title", "description"])
    );
}

#[test]
fn w2_c4f_the_shop_name_is_sent_only_when_set() {
    let h = writer_harness();
    let blank = build(&h, &WriterSettings::default());
    assert!(blank.get("shop_name").is_none(), "{blank}");
    let named = build(
        &h,
        &WriterSettings {
            shop_name: "Beyo Vintage".into(),
            ..WriterSettings::default()
        },
    );
    assert_eq!(named["shop_name"], json!("Beyo Vintage"));
}
