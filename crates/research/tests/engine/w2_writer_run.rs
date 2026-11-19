//! W2 (book 08C §5–§7; contracts WC2, WC3, WC7 of `master_plan_writer.md` §5): the writer's
//! input (unit 1, rows W2·C4) and the run's life (unit 3, rows W2·C12–C15). Runs on the research
//! harness with the crate's fakes; the outcome is the built input, the answers of `request`, the
//! run rows and their snapshots.

use psm_core::domain::{
    attribute::{AiConfidence, AiStatus, AttributeGroup, CoreType, Provenance, Value},
    research::FeedbackAction,
};
use psm_core::ports::secrets::SecretStore;
use psm_research::{
    config::{ResearchConfig, ResearchSettings},
    domain::{RunState, RunTrigger, Stage},
    service::ResearchService,
    store::{self as rstore, NewRun, OutputRow},
    writer::{
        api::{Requested, WriterApi},
        config::WriterSettings,
        input,
        run::Origin,
    },
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
            {"key": "local.restoration", "label": "Restoration done", "value": null,
             "decision": "empty", "fact": false},
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
    // W5 (WC8.1): the two search texts are built-in and always present, so they are requested
    // too; the condition report is still not.
    assert_eq!(
        build(&h, &WriterSettings::default())["texts"],
        json!(["title", "description", "seo_title", "seo_description"])
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

// ---- Unit 3: the run's life ----

/// Writer settings under which every prerequisite passes on the harness (model `m1`, whose price
/// the test settings carry).
pub(crate) fn writer_ok() -> Json {
    json!({"model": "m1", "effort": "low"})
}

fn request(h: &Harness, session: i64, origin: Origin) -> Requested {
    WriterApi::request(h.service.as_ref(), session, origin).unwrap()
}

fn reason_code(answer: &Requested) -> Option<String> {
    answer.reason.as_ref().map(|r| r.code.clone())
}

/// Inserts a run of `session` with the given trigger and state.
pub(crate) fn run_of(
    h: &Harness,
    item: i64,
    session: i64,
    trigger: RunTrigger,
    state: RunState,
) -> i64 {
    let now = h.now();
    h.query(|c| {
        rstore::insert_run(
            c,
            &NewRun {
                item_id: item,
                session_id: session,
                trigger,
                state,
                queued_at: now,
                started_at: (state != RunState::Queued).then_some(now),
                settings_snapshot: None,
                form_snapshot: None,
                input_snapshot: None,
            },
        )
        .unwrap()
    })
}

/// The `write` runs of a session, oldest first.
pub(crate) fn write_runs(h: &Harness, session: i64) -> Vec<rstore::RunRow> {
    h.query(|c| rstore::all_runs(c, Some(session)).unwrap())
        .into_iter()
        .filter(|r| r.trigger == RunTrigger::Write)
        .collect()
}

fn jobs_of(h: &Harness, run: i64) -> i64 {
    h.query(|c| {
        c.query_row(
            "SELECT COUNT(*) FROM jobs WHERE kind='research' AND target=?1",
            [run.to_string()],
            |r| r.get(0),
        )
        .unwrap()
    })
}

fn ready() -> Harness {
    let h = writer_harness();
    h.set_writer_settings(writer_ok());
    h
}

#[test]
fn w2_c12a_a_queued_research_run_refuses_writing() {
    let h = ready();
    run_of(
        &h,
        h.item_id,
        h.session_id,
        RunTrigger::Manual,
        RunState::Queued,
    );
    let answer = request(&h, h.session_id, Origin::WriteTexts);
    assert!(!answer.queued);
    assert_eq!(reason_code(&answer).as_deref(), Some("ResearchBusy"));
    assert!(write_runs(&h, h.session_id).is_empty());
}

#[test]
fn w2_c12b_a_running_look_run_refuses_writing() {
    let h = ready();
    run_of(
        &h,
        h.item_id,
        h.session_id,
        RunTrigger::Look,
        RunState::Running,
    );
    let answer = request(&h, h.session_id, Origin::WriteTexts);
    assert!(!answer.queued);
    assert_eq!(reason_code(&answer).as_deref(), Some("ResearchBusy"));
    assert!(write_runs(&h, h.session_id).is_empty());
}

#[test]
fn w2_c12c_the_prerequisites_in_order() {
    let h = writer_harness();
    h.app.secrets.delete("research-model-key").unwrap();
    let ask =
        |svc: &ResearchService| WriterApi::request(svc, h.session_id, Origin::WriteTexts).unwrap();
    let neither = h.service_with(|d| d.shop = None);
    let answer = ask(&neither);
    assert!(!answer.queued);
    assert_eq!(reason_code(&answer).as_deref(), Some("NoIntegration"));
    let answer = ask(&h.service);
    assert_eq!(reason_code(&answer).as_deref(), Some("NoModelKey"));
    h.app
        .secrets
        .set("research-model-key", "sk-test-key")
        .unwrap();
    let answer = ask(&h.service);
    assert!(!answer.queued);
    let reason = answer.reason.unwrap();
    assert_eq!(reason.code, "PriceMissing");
    assert_eq!(
        reason.sentence,
        "No price is set for fake-model gpt-6.1-sol"
    );
    assert!(write_runs(&h, h.session_id).is_empty());
}

#[test]
fn w2_c12h_the_unassigned_session_has_nothing_to_write() {
    let h = ready();
    run_of(
        &h,
        h.item_id,
        h.session_id,
        RunTrigger::Manual,
        RunState::Queued,
    );
    let unassigned: i64 = h.query(|c| {
        c.query_row("SELECT id FROM sessions WHERE item_id IS NULL", [], |r| {
            r.get(0)
        })
        .unwrap()
    });
    let answer = request(&h, unassigned, Origin::WriteTexts);
    assert!(!answer.queued);
    assert_eq!(reason_code(&answer).as_deref(), Some("NothingToWrite"));
    assert!(write_runs(&h, unassigned).is_empty());
}

#[test]
fn w2_c12d_automatic_writing_off_writes_only_on_request() {
    let h = writer_harness();
    h.set_writer_settings(json!({"model": "m1", "effort": "low", "auto": false}));
    let answer = request(&h, h.session_id, Origin::NextToReview);
    assert_eq!(
        answer,
        Requested {
            queued: false,
            reason: None
        }
    );
    assert!(write_runs(&h, h.session_id).is_empty());
    let answer = request(&h, h.session_id, Origin::WriteTexts);
    assert!(answer.queued);
    let runs = write_runs(&h, h.session_id);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].state, RunState::Queued);
}

#[test]
fn w2_c12e_automatic_writing_happens_once_unless_it_failed() {
    let h = ready();
    run_of(
        &h,
        h.item_id,
        h.session_id,
        RunTrigger::Write,
        RunState::Finished,
    );
    let answer = request(&h, h.session_id, Origin::AllChecked);
    assert_eq!(answer.reason, None);
    assert_eq!(write_runs(&h, h.session_id).len(), 1);
    let (item, session) = h.new_session("W2-2");
    run_of(&h, item, session, RunTrigger::Write, RunState::Failed);
    let answer = request(&h, session, Origin::AllChecked);
    assert!(answer.queued);
    let states: Vec<RunState> = write_runs(&h, session).iter().map(|r| r.state).collect();
    assert_eq!(states, [RunState::Failed, RunState::Queued]);
}

#[test]
fn w2_c12f_repeat_presses_keep_one_queued_writer_run() {
    let h = ready();
    run_of(
        &h,
        h.item_id,
        h.session_id,
        RunTrigger::Write,
        RunState::Queued,
    );
    for _ in 0..2 {
        let answer = request(&h, h.session_id, Origin::WriteTexts);
        assert_eq!(
            answer,
            Requested {
                queued: true,
                reason: None
            }
        );
    }
    assert_eq!(write_runs(&h, h.session_id).len(), 1);
}

#[test]
fn w2_c12g_while_texts_are_written_one_more_run_waits() {
    let h = ready();
    run_of(
        &h,
        h.item_id,
        h.session_id,
        RunTrigger::Write,
        RunState::Running,
    );
    let states = |h: &Harness| -> Vec<RunState> {
        write_runs(h, h.session_id)
            .iter()
            .map(|r| r.state)
            .collect()
    };
    assert!(request(&h, h.session_id, Origin::WriteTexts).queued);
    assert_eq!(states(&h), [RunState::Running, RunState::Queued]);
    assert!(request(&h, h.session_id, Origin::WriteTexts).queued);
    assert_eq!(states(&h), [RunState::Running, RunState::Queued]);
}

#[test]
fn w2_c13b_a_writer_run_starts_without_waiting_for_photos() {
    let h = ready();
    assert!(request(&h, h.session_id, Origin::WriteTexts).queued);
    h.service.session_changed(h.session_id);
    let run = write_runs(&h, h.session_id)[0].id;
    assert_eq!(h.service.tick().unwrap(), Some(run));
    assert_eq!(h.run_row(run).state, RunState::Running);
    assert_eq!(jobs_of(&h, run), 1);
}

#[test]
fn w2_c13c_the_day_cap_holds_a_writer_run() {
    let h = ready();
    h.patch_settings(json!({"max_cost_per_day": "0.001"}));
    let earlier = run_of(
        &h,
        h.item_id,
        h.session_id,
        RunTrigger::Manual,
        RunState::Finished,
    );
    let now = h.now();
    h.query(|c| {
        c.execute(
            "INSERT INTO research_calls(run_id,stage,unit_key,attempt,provider,model,effort,state,\
             request,started_at,cost_micro_usd) \
             VALUES (?1,'decide','decide',1,'fake-model','m1','low','answered','{}',?2,1000)",
            rusqlite::params![earlier, now],
        )
        .unwrap()
    });
    assert!(request(&h, h.session_id, Origin::WriteTexts).queued);
    let run = write_runs(&h, h.session_id)[0].id;
    assert_eq!(h.service.tick().unwrap(), None);
    assert_eq!(h.run_row(run).state, RunState::Queued);
    assert_eq!(jobs_of(&h, run), 0);
}

#[test]
fn w2_c14a_the_writer_stage_is_number_8_and_not_a_research_stage() {
    assert_eq!(Stage::parse("write"), Some(Stage::Write));
    assert_eq!(Stage::Write.number(), 8);
    assert_eq!(Stage::ALL.len(), 7);
    assert!(!Stage::ALL.contains(&Stage::Write));
    let stages: Vec<String> = ResearchSettings::default().stages.into_keys().collect();
    let mut research: Vec<String> = Stage::ALL.iter().map(|s| s.as_str().to_owned()).collect();
    research.sort();
    assert_eq!(stages, research);
}

/// C15's harness: an English instruction "My guide", model `gpt-6.1-terra` at effort `low`, whose
/// price the research settings carry.
fn my_guide() -> Harness {
    let h = writer_harness();
    let mut prices = h.research_settings().unwrap()["prices"].clone();
    prices.as_array_mut().unwrap().push(json!({
        "provider": "fake-model", "model": "gpt-6.1-terra", "fallback_call_usd": "0.001"
    }));
    h.patch_settings(json!({ "prices": prices }));
    h.set_writer_settings(json!({
        "model": "gpt-6.1-terra",
        "effort": "low",
        "instructions": {"English": {"text": "My guide", "versions": []}},
    }));
    h
}

fn snapshot(h: &Harness, run: i64) -> ResearchConfig {
    ResearchConfig::from_snapshot(h.run_row(run).settings_snapshot.as_deref()).unwrap()
}

#[test]
fn w2_c15a_the_start_snapshots_the_writer_settings_and_the_input() {
    let h = my_guide();
    assert!(request(&h, h.session_id, Origin::WriteTexts).queued);
    let run = write_runs(&h, h.session_id)[0].id;
    assert_eq!(h.service.tick().unwrap(), Some(run));
    let config = snapshot(&h, run);
    let writer = config.writer.clone().unwrap();
    assert_eq!(writer.instruction, "My guide");
    assert_eq!(writer.origin, "write_texts");
    let names: Vec<&str> = writer.examples.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["A", "B", "C"]);
    let stage = config.stage(Stage::Write);
    assert_eq!(
        (stage.model.as_str(), stage.effort.as_str()),
        ("gpt-6.1-terra", "low")
    );
    let stored: Json =
        serde_json::from_str(h.run_row(run).input_snapshot.as_deref().unwrap()).unwrap();
    assert_eq!(stored, build(&h, &h.service.writer_settings()));
}

#[test]
fn w2_c15b_a_later_instruction_change_does_not_reach_a_started_run() {
    let h = my_guide();
    assert!(request(&h, h.session_id, Origin::WriteTexts).queued);
    let run = write_runs(&h, h.session_id)[0].id;
    h.service.tick().unwrap();
    h.set_writer_settings(json!({
        "model": "gpt-6.1-terra",
        "effort": "low",
        "instructions": {"English": {"text": "Another guide", "versions": []}},
    }));
    h.run_jobs();
    assert_eq!(snapshot(&h, run).writer.unwrap().instruction, "My guide");
}
