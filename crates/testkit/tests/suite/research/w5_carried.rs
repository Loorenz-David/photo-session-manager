//! W5·U4 (rows W5·C10, C11, C14): fixes carried from the W1–W4 reviews. C10 at the post-check
//! (`writer::postcheck::check`, WC5.10 (iii) amended); C11 a `write` run on the wired app with the
//! research fakes (WC6 rule 2 amended); C14 the writer handle's `status` (WC1.7 amended).

use std::{fs, sync::Arc};

use psm_core::{
    domain::{
        attribute::{AiConfidence, Provenance, Value},
        photo::{Classification as C, NewPhoto, PhotoSource},
    },
    mechanisms::hash::sha256_file,
    ports::clock::Clock,
};
use psm_research::{
    domain::{RunState, RunTrigger, Stage},
    stages::OBSERVE_UNIT,
    store::{self as rstore, NewRun, OutputRow, RunRow},
    writer::{
        api::WriterApi,
        postcheck::{CheckContext, HitCode, check},
        run::Origin,
    },
};
use psm_testkit::{TestApp, research_test_settings};
use serde_json::{Value as Json, json};

const NOW_MS: i64 = 1_763_590_000_000;
const DESCRIPTION: &str = "product.description";

// ---------------------------------------------------------------- C10: the Models rule

fn models(text_name: &str, text: &str, facts: &[&str]) -> Vec<String> {
    let ctx = CheckContext {
        language: "English".into(),
        facts: facts.iter().map(|f| (*f).to_owned()).collect(),
        ..CheckContext::default()
    };
    check(&[(text_name.to_owned(), text.to_owned())], &ctx)
        .into_iter()
        .filter(|h| h.code == HitCode::Models)
        .map(|h| h.detail)
        .collect()
}

#[test]
fn w5_c10a_a_condition_sentence_with_a_and_a_unit_is_no_model_number() {
    let report = "Very Good condition. A 1 cm veneer chip on the front edge (photo 3). Stable.";
    let hits = models(
        "condition_report",
        report,
        &["Minor", "Very Good - light wear"],
    );
    assert_eq!(hits, Vec::<String>::new());
}

#[test]
fn w5_c10b_a_model_number_not_in_the_fields_is_one_hit() {
    let hits = models("description", "Model PK 22 lounge chair.", &["Teak"]);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].contains("PK 22"), "{hits:?}");
}

// ---------------------------------------------------------------- C11 and C14: the wired writer

fn writer(a: &TestApp) -> Arc<dyn WriterApi> {
    a.app
        .handles()
        .writer
        .expect("the wired app hands the writer")
}

/// The wired research app with the writer's default model priced for the fake provider.
fn app() -> TestApp {
    let a = TestApp::with_research().unwrap();
    a.clock.advance(NOW_MS);
    let mut settings = research_test_settings();
    settings["prices"].as_array_mut().unwrap().push(json!({
        "provider": psm_research::fakes::FakeModel::ID, "model": "gpt-6.1-sol",
        "unit": "input_token", "usd_per_million": "2", "fallback_call_usd": "0.001"
    }));
    a.set_research_settings(settings).unwrap();
    a
}

fn photo(a: &TestApp, s: i64, name: &str, class: C) {
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(name);
    fs::write(&raw, format!("raw bytes of {name}")).unwrap();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id: s,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(NOW_MS),
            source: PhotoSource::Capture,
            created_at: NOW_MS,
        })
        .unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.review().classify(photo.id, class).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
}

/// An item with one session and one Main photo: `(session, item)`.
fn session(a: &TestApp, sku: &str) -> (i64, i64) {
    let (item, session) = a.items().create_local(sku, "", "Chair", None).unwrap();
    photo(a, session.session.id, &format!("{sku}.ARW"), C::Main);
    (session.session.id, item.id)
}

/// About 70 words, clean for every rule but the year 1958, which no field holds.
const PARAGRAPHS_1958: [&str; 4] = [
    "A dining chair made in Denmark in 1958 with a frame of solid beech and a seat of woven \
     papercord.",
    "The back rail is curved and the legs taper toward the floor. The seat is square with rounded \
     corners and sits on four rails.",
    "We cleaned the frame, oiled the wood and rewove the seat with new natural papercord. It works \
     at a dining table or a desk.",
    "Dimensions: W 49 × D 46 × H 78 cm, seat height 45 cm.",
];

fn answer() -> Json {
    json!({
        "texts": {
            "title": {"text": "Danish Beech Dining Chair with Curved Back", "facts": [], "reason": "From the fields."},
            "description": {"paragraphs": PARAGRAPHS_1958, "facts": [], "reason": "From the fields."},
        },
        "findings": [],
        "questions": [],
    })
}

/// Scripts both writer calls, presses Write texts and lets the run finish.
fn write(a: &TestApp, s: i64) {
    write_as(a, s, Origin::WriteTexts);
}

/// As [`write`], asked for by `origin` (A24: an automatic origin keeps MC-A1's refusals).
fn write_as(a: &TestApp, s: i64, origin: Origin) {
    let model = &a.research_fakes().model;
    model.reply(Stage::Write, "texts", answer());
    model.reply(Stage::Write, "retry", answer());
    assert!(writer(a).request(s, origin).unwrap().queued);
    for _ in 0..3 {
        a.tick(1_100).unwrap();
    }
    let runs: Vec<RunRow> = a
        .app
        .store
        .research_transaction(|tx| rstore::all_runs(tx.conn(), Some(s)))
        .unwrap()
        .into_iter()
        .filter(|r| r.trigger == RunTrigger::Write)
        .collect();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].state, RunState::Finished);
}

/// The item's `writer_findings` rows as `(type, text_field, state)`.
fn findings(a: &TestApp, item: i64) -> Vec<(String, String, String)> {
    a.app
        .store
        .research_transaction(|tx| {
            let mut stmt = tx
                .conn()
                .prepare(
                    "SELECT type, text_field, state FROM writer_findings WHERE item_id=?1 \
                     ORDER BY id",
                )
                .unwrap();
            Ok(stmt
                .query_map([item], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap())
        })
        .unwrap()
}

fn row(a: &TestApp, item: i64, key: &str) -> Option<Value> {
    let mut out = None;
    a.store()
        .transaction(&mut |tx| {
            out = tx
                .attribute_values(item)?
                .into_iter()
                .find(|v| v.key == key);
            Ok(())
        })
        .unwrap();
    out
}

#[test]
fn w5_c11a_no_text_finding_about_the_owners_own_description() {
    let a = app();
    let (s, item) = session(&a, "W5-C11A");
    // The owner's own description, stored without a feedback row (never reviewed by research).
    let own = Value {
        item_id: item,
        key: DESCRIPTION.into(),
        value: json!("My own words about the chair.").to_string(),
        valid: true,
        provenance: Provenance::Human,
        ai_confidence: None,
        ai_status: None,
        ai_reason: None,
        ai_sources: None,
        ai_prior: None,
        updated_at: NOW_MS,
    };
    a.store()
        .transaction(&mut |tx| tx.put_attribute_value(&own))
        .unwrap();
    // A24: an automatic run (a deliberate Write texts offers a new description over it).
    write_as(&a, s, Origin::NextToReview);
    let about_description: Vec<_> = findings(&a, item)
        .into_iter()
        .filter(|(t, tf, _)| t == "text_vs_facts" && tf == DESCRIPTION)
        .collect();
    assert_eq!(about_description, Vec::new());
    assert_eq!(row(&a, item, DESCRIPTION), Some(own));
}

#[test]
fn w5_c11b_a_written_description_with_an_unknown_year_is_flagged_and_asked() {
    let a = app();
    let (s, item) = session(&a, "W5-C11B");
    write(&a, s);
    let description = row(&a, item, DESCRIPTION).expect("the description is written");
    assert_eq!(description.provenance, Provenance::Ai);
    assert_eq!(description.ai_confidence, Some(AiConfidence::Low));
    let about_description: Vec<_> = findings(&a, item)
        .into_iter()
        .filter(|(t, tf, _)| t == "text_vs_facts" && tf == DESCRIPTION)
        .collect();
    assert_eq!(
        about_description,
        [(
            "text_vs_facts".to_owned(),
            DESCRIPTION.to_owned(),
            "open".to_owned()
        )]
    );
}

#[test]
fn w5_c14a_changed_condition_photos_show_no_banner_without_a_condition_field() {
    let a = app();
    let (s, item) = session(&a, "W5-C14A");
    // An earlier research run whose condition look (N6) kept a note about other photos.
    let now = a.clock.now_ms();
    a.app
        .store
        .research_transaction(|tx| {
            let run = rstore::insert_run(
                tx.conn(),
                &NewRun {
                    item_id: item,
                    session_id: s,
                    trigger: RunTrigger::Manual,
                    state: RunState::Finished,
                    queued_at: now,
                    started_at: Some(now),
                    settings_snapshot: None,
                    form_snapshot: None,
                    input_snapshot: None,
                },
            )?;
            for (stage, unit, output) in [
                (
                    Stage::Observe,
                    OBSERVE_UNIT,
                    json!({"reads": {}, "marking_photos": [], "note": []}),
                ),
                (
                    Stage::Condition,
                    "condition",
                    json!({"note": [{"text": "A chip on the front edge", "keys": [], "photo_id": null}]}),
                ),
            ] {
                rstore::insert_output(
                    tx.conn(),
                    &OutputRow {
                        run_id: run,
                        stage: stage.as_str().into(),
                        unit_key: unit.into(),
                        fingerprint: format!("{unit}-earlier-photos"),
                        output: output.to_string(),
                        reused_from_run_id: None,
                        committed_at: now,
                    },
                )?;
            }
            Ok(())
        })
        .unwrap();
    // A condition photo the look never saw; research fills no Condition field.
    photo(&a, s, "W5-C14A-condition.ARW", C::Condition);
    let status = writer(&a).status(s).unwrap();
    assert!(!status.stale_condition);
}
