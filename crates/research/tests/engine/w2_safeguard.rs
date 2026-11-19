//! W2 unit 5 (contracts WC5 → WC6, WC7 of `master_plan_writer.md` §5): the findings a writer run
//! stores, answering and dismissing them, the writer's calls in research's cost, record and export
//! (rows W2·C22–C24). Runs on the research harness with `FakeModel` replying for stage `write`.

use std::path::{Path, PathBuf};

use chrono::{Datelike, TimeZone};
use psm_core::domain::{
    attribute::{AiConfidence, AttributeGroup, CoreType, Provenance},
    photo::Classification,
};
use psm_research::{
    api::{ExportSelection, ResearchApi},
    domain::{Prerequisite, RunState, RunTrigger, Stage},
    writer::api::WriterApi,
};
use serde_json::{Value as Json, json};

use crate::{
    harness::{DESIGNER, Harness, WOOD},
    w1_note::{add_definitions, definition, layout},
    w2_writer_run::{ERA, finding_row, put, run_of, write_runs},
    w2_writer_writes::{
        CLEAN_PARAGRAPHS, CLEAN_REPORT, CLEAN_TITLE, REPORT, answer, clean, run_with,
        texts_harness, write_requests,
    },
};

/// One `writer_findings` row as the tests read it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    id: i64,
    run_id: i64,
    kind: String,
    finding_type: String,
    keys: Json,
    text_field: String,
    identity: String,
    mode: String,
    state: String,
    answer: Option<String>,
    resolved_at: Option<i64>,
}

fn rows(h: &Harness) -> Vec<Row> {
    h.query(|c| {
        let mut stmt = c
            .prepare(
                "SELECT id,run_id,kind,type,keys,text_field,identity,mode,state,answer,resolved_at \
                 FROM writer_findings WHERE item_id=?1 ORDER BY id",
            )
            .unwrap();
        stmt.query_map([h.item_id], |r| {
            Ok(Row {
                id: r.get(0)?,
                run_id: r.get(1)?,
                kind: r.get(2)?,
                finding_type: r.get(3)?,
                keys: serde_json::from_str(&r.get::<_, String>(4)?).unwrap(),
                text_field: r.get(5)?,
                identity: r.get(6)?,
                mode: r.get(7)?,
                state: r.get(8)?,
                answer: r.get(9)?,
                resolved_at: r.get(10)?,
            })
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
    })
}

#[test]
fn w2_c22a_a_runs_findings_are_stored_open_and_supersede_the_earlier_ones() {
    let h = texts_harness(true);
    let earlier = finding_row(
        &h,
        "field_vs_field",
        &[WOOD],
        "field_vs_field|custom.wood|",
        "An earlier note",
        "open",
        None,
        None,
    );
    let mut reply = clean();
    reply["findings"] = json!([
        {"type": "field_vs_field", "keys": [ERA, DESIGNER],
         "text": "The era does not fit the designer"},
        {"type": "field_vs_listings", "keys": [DESIGNER],
         "text": "A matching listing credits another maker"},
    ]);
    let run = run_with(&h, reply, None);
    let all = rows(&h);
    let old = all.iter().find(|r| r.id == earlier).unwrap();
    assert_eq!(old.state, "superseded");
    let new: Vec<(String, String, Json, String, String, String)> = all
        .iter()
        .filter(|r| r.run_id == run)
        .map(|r| {
            (
                r.kind.clone(),
                r.finding_type.clone(),
                r.keys.clone(),
                r.identity.clone(),
                r.mode.clone(),
                r.state.clone(),
            )
        })
        .collect();
    let finding = |t: &str, keys: Json, identity: &str| {
        (
            "finding".to_owned(),
            t.to_owned(),
            keys,
            identity.to_owned(),
            "note".to_owned(),
            "open".to_owned(),
        )
    };
    assert_eq!(
        new,
        [
            finding(
                "field_vs_field",
                json!([DESIGNER, ERA]),
                "field_vs_field|custom.designer,custom.era|"
            ),
            finding(
                "field_vs_listings",
                json!([DESIGNER]),
                "field_vs_listings|custom.designer|"
            ),
        ]
    );
}

#[test]
fn w2_c22b_an_answer_is_kept_and_the_texts_are_written_again_with_it() {
    let h = texts_harness(true);
    h.model.reply(Stage::Write, "texts", clean());
    let id = finding_row(
        &h,
        "field_vs_photos",
        &[WOOD],
        "field_vs_photos|custom.wood|",
        "The photos show oak; is the wood teak?",
        "open",
        None,
        None,
    );
    let answered = WriterApi::answer(h.service.as_ref(), id, "It is teak").unwrap();
    assert!(answered.queued);
    let row = rows(&h).into_iter().find(|r| r.id == id).unwrap();
    assert_eq!(row.state, "answered");
    assert_eq!(row.answer.as_deref(), Some("It is teak"));
    assert_eq!(row.resolved_at, Some(h.now()));
    let runs = write_runs(&h, h.session_id);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].state, RunState::Queued);
    h.tick(0);
    let input: Json =
        serde_json::from_str(h.run_row(runs[0].id).input_snapshot.as_deref().unwrap()).unwrap();
    assert_eq!(
        input["answers"],
        json!([{"type": "field_vs_photos", "keys": [WOOD],
                "question": "The photos show oak; is the wood teak?", "answer": "It is teak"}])
    );
}

#[test]
fn w2_c22c_a_dismissed_note_writes_nothing_and_is_counted() {
    let h = texts_harness(true);
    let id = finding_row(
        &h,
        "field_vs_field",
        &[ERA],
        "field_vs_field|custom.era|",
        "The era does not fit the designer",
        "open",
        None,
        None,
    );
    WriterApi::dismiss(h.service.as_ref(), id).unwrap();
    let row = rows(&h).into_iter().find(|r| r.id == id).unwrap();
    assert_eq!(row.state, "dismissed");
    assert!(write_runs(&h, h.session_id).is_empty());
    let counts = h.service.writer_counts().unwrap();
    let field_vs_field = counts
        .iter()
        .find(|c| c.finding_type == "field_vs_field")
        .unwrap();
    assert_eq!((field_vs_field.answered, field_vs_field.dismissed), (0, 1));
    assert_eq!(counts.len(), 5);
}

const STAINS: &str = "custom.condition_stains_discoloration";

#[test]
fn w2_c23a_a_damage_the_report_omits_twice_becomes_a_question() {
    let h = texts_harness(true);
    let mut stains = definition(
        STAINS,
        "Condition.Stains/Discoloration",
        CoreType::Choice,
        AttributeGroup::Condition,
    );
    stains.choices = psm_core::mechanisms::values::SEVERITY_SET
        .iter()
        .map(|c| (*c).to_owned())
        .collect();
    add_definitions(&h, vec![stains]);
    layout(&h, "Texts", AttributeGroup::Condition, &[REPORT, STAINS]);
    put(&h, STAINS, json!("Minor"), Provenance::Human, None, None);
    let retry = json!({
        "texts": {"condition_report": {"text": CLEAN_REPORT, "facts": [], "reason": "Again."}},
        "findings": [], "questions": [],
    });
    let run = run_with(&h, clean(), Some(retry));
    assert_eq!(write_requests(&h).len(), 2);
    let stored: Vec<(String, Json, String, String)> = rows(&h)
        .into_iter()
        .filter(|r| r.run_id == run)
        .map(|r| (r.finding_type, r.keys, r.text_field, r.state))
        .collect();
    assert_eq!(
        stored,
        [(
            "missing_in_text".to_owned(),
            json!([STAINS]),
            REPORT.to_owned(),
            "open".to_owned()
        )]
    );
    assert_eq!(rows(&h)[0].mode, "ask");
}

#[test]
fn w2_c23b_with_text_vs_facts_off_a_year_flags_the_text_and_asks_nothing() {
    let h = texts_harness(true);
    h.set_writer_settings(json!({"safeguard": {"text_vs_facts": "off"}}));
    let title = "Danish Teak Chair, 1958";
    let retry = json!({
        "texts": {"title": {"text": title, "facts": [], "reason": "Again."}},
        "findings": [], "questions": [],
    });
    run_with(
        &h,
        answer(title, &CLEAN_PARAGRAPHS, CLEAN_REPORT),
        Some(retry),
    );
    assert!(rows(&h).is_empty());
    let row = h.row("product.title").unwrap();
    assert_eq!(row.ai_confidence, Some(AiConfidence::Low));
    assert!(row.ai_reason.unwrap().contains("1958"));
}

/// A first answer whose description has a hit, and a clean rewrite: two calls.
fn rewritten(h: &Harness) -> i64 {
    let mut paragraphs = CLEAN_PARAGRAPHS.to_vec();
    paragraphs[0] = "A stunning dining chair made in Denmark with a frame of solid teak.";
    let retry = json!({
        "texts": {"description": {"paragraphs": CLEAN_PARAGRAPHS, "facts": [], "reason": "R."}},
        "findings": [], "questions": [],
    });
    run_with(
        h,
        answer(CLEAN_TITLE, &paragraphs, CLEAN_REPORT),
        Some(retry),
    )
}

#[test]
fn w2_c24a_writer_calls_are_costed_and_count_toward_the_day() {
    let h = texts_harness(true);
    h.model.set_default_usage(Some(vec![
        ("input_token".into(), 1000),
        ("output_token".into(), 500),
    ]));
    h.patch_settings(json!({"max_cost_per_day": "0.014"}));
    let run = rewritten(&h);
    let calls = h.calls(run);
    let recorded: Vec<(String, String, i64)> = calls
        .iter()
        .map(|c| (c.stage.clone(), c.model.clone(), c.cost_micro_usd))
        .collect();
    // 1000 × 2 + 500 × 10 µUSD per call (USD 2 and 10 per million tokens).
    assert_eq!(
        recorded,
        [
            ("write".to_owned(), "gpt-6.1-sol".to_owned(), 7000),
            ("write".to_owned(), "gpt-6.1-sol".to_owned(), 7000),
        ]
    );
    assert_eq!(h.run_row(run).cost_micro_usd, 14_000);
    assert_eq!(
        h.service.first_failing(h.session_id).unwrap(),
        Some(Prerequisite::DayLimit)
    );
    h.patch_settings(json!({"max_cost_per_day": "0.014001"}));
    assert_eq!(h.service.first_failing(h.session_id).unwrap(), None);
}

#[test]
fn w2_c24b_no_photo_leaves_for_a_writer_run() {
    let h = texts_harness(true);
    h.add_main_photo_with_preview(h.session_id);
    h.add_photo_with_preview(h.session_id, Classification::Condition);
    let run = rewritten(&h);
    assert_eq!(h.calls(run).len(), 2);
    assert!(h.outbound(run).is_empty());
    assert!(write_requests(&h).iter().all(|r| r.images.is_empty()));
}

fn today(h: &Harness) -> i64 {
    let local = chrono::Local.timestamp_millis_opt(h.now()).unwrap();
    i64::from(local.year()) * 10_000 + i64::from(local.month()) * 100 + i64::from(local.day())
}

fn run_dir(folder: &Path, run: i64) -> PathBuf {
    for item in std::fs::read_dir(folder).unwrap().flatten() {
        let dir = item.path().join(format!("run-{run}"));
        if dir.is_dir() {
            return dir;
        }
    }
    panic!("no folder for run {run}");
}

fn read(path: PathBuf) -> Json {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn w2_c24c_writer_and_look_runs_are_in_the_export() {
    let h = texts_harness(true);
    let research = run_of(
        &h,
        h.item_id,
        h.session_id,
        RunTrigger::Manual,
        RunState::Finished,
    );
    let look = run_of(
        &h,
        h.item_id,
        h.session_id,
        RunTrigger::Look,
        RunState::Finished,
    );
    let write = run_with(&h, clean(), None);
    let dest = h.app.dir.path().join("w2-export");
    std::fs::create_dir_all(&dest).unwrap();
    let day = today(&h);
    let zip = ResearchApi::export(
        h.service.as_ref(),
        ExportSelection::Range { from: day, to: day },
        &dest,
    )
    .unwrap();
    let folder = zip.with_extension("");
    for run in [research, look, write] {
        assert!(
            run_dir(&folder, run).join("run.json").is_file(),
            "run {run}"
        );
    }
    let dir = run_dir(&folder, write);
    assert_eq!(read(dir.join("run.json"))["run"]["trigger"], json!("write"));
    let writes = read(dir.join("writes.json"));
    for key in ["texts", "hits", "writes", "findings", "questions"] {
        assert!(
            writes.get(key).is_some(),
            "writes.json has no {key}: {writes}"
        );
    }
    assert_eq!(writes["texts"]["title"]["value"], json!(CLEAN_TITLE));
    assert_eq!(read(run_dir(&folder, look).join("writes.json")), Json::Null);
}
