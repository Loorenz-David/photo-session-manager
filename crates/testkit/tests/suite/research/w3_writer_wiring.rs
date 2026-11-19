//! W3·U2 (rows W3·C4–C6): the `writer` settings slot and the wiring of the writer's four W3
//! operations. Boundary: the wired `TestApp` (the real `App` start-up and settings paths, the
//! research fakes) through the writer handle the commands `writer_status`, `writer_request`,
//! `writer_finding_answer` and `writer_finding_dismiss` call (`AppHandles::writer`), and
//! `settings_set` with its prepared kind.

use std::{fs, sync::Arc};

use psm_core::{
    domain::{
        photo::{Classification as C, NewPhoto, PhotoSource},
        session::PublishStep,
    },
    events::AppEvent,
    mechanisms::hash::sha256_file,
    ports::{clock::Clock, secrets::SecretStore},
};
use psm_integrations::PreparedSettings;
use psm_research::{
    domain::{RunState, RunTrigger, Stage},
    stages::OBSERVE_UNIT,
    store::{self as rstore, NewRun, OutputRow, RunRow},
    writer::{
        api::{Requested, WriterApi},
        run::Origin,
    },
};
use psm_testkit::{TestApp, research_test_settings};
use serde_json::{Value as Json, json};

const NOW_MS: i64 = 1_763_570_000_000;
const DESIGNER: &str = "custom.designer";
const QUESTION: &str = "Is the frame teak or oak?";

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

fn photo(a: &TestApp, s: i64, name: &str) {
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
    a.review().classify(photo.id, C::Main).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
}

/// An item with one session and one Main photo: `(session, item)`.
fn session(a: &TestApp, sku: &str) -> (i64, i64) {
    let (item, session) = a.items().create_local(sku, "", "Chair", None).unwrap();
    photo(a, session.session.id, &format!("{sku}.ARW"));
    (session.session.id, item.id)
}

fn research_run(a: &TestApp, s: i64, item: i64, state: RunState) -> i64 {
    let now = a.clock.now_ms();
    a.app
        .store
        .research_transaction(|tx| {
            rstore::insert_run(
                tx.conn(),
                &NewRun {
                    item_id: item,
                    session_id: s,
                    trigger: RunTrigger::Manual,
                    state,
                    queued_at: now,
                    started_at: Some(now),
                    settings_snapshot: None,
                    form_snapshot: None,
                    input_snapshot: None,
                },
            )
        })
        .unwrap()
}

/// A researched session (as the research fakes leave one): a finished research run whose photo
/// look noted two observations about the designer, whose stage 3 read two pages and whose stage
/// 4 judged one a match. Returns `(session, item)`.
fn researched(a: &TestApp, sku: &str) -> (i64, i64) {
    let (s, item) = session(a, sku);
    let run = research_run(a, s, item, RunState::Finished);
    let now = a.clock.now_ms();
    let output = |stage: Stage, unit: &str, output: Json| OutputRow {
        run_id: run,
        stage: stage.as_str().into(),
        unit_key: unit.into(),
        fingerprint: format!("{unit}-{run}"),
        output: output.to_string(),
        reused_from_run_id: None,
        committed_at: now,
    };
    a.app
        .store
        .research_transaction(|tx| {
            let c = tx.conn();
            rstore::insert_output(
                c,
                &output(
                    Stage::Observe,
                    OBSERVE_UNIT,
                    json!({"reads": {}, "marking_photos": [], "note": [
                        {"text": "A dining chair with a curved back rail", "keys": []},
                        {"text": "Stamped by the maker under the seat", "keys": [DESIGNER]},
                    ]}),
                ),
            )?;
            for (unit, title, verdict) in [
                ("u-1", "Match listing chair", "match"),
                ("u-2", "Other chair", "different"),
            ] {
                rstore::insert_output(
                    c,
                    &output(
                        Stage::Read,
                        unit,
                        json!({"unit": unit, "title": title, "obtained": true, "statements": {}}),
                    ),
                )?;
                rstore::insert_output(
                    c,
                    &output(
                        Stage::Compare,
                        unit,
                        json!({"unit": unit, "verdict": verdict}),
                    ),
                )?;
            }
            Ok(())
        })
        .unwrap();
    (s, item)
}

const CLEAN_PARAGRAPHS: [&str; 4] = [
    "A dining chair made in Denmark with a frame of solid beech and a seat of woven papercord.",
    "The back rail is curved and the legs taper toward the floor. The seat is square with rounded \
     corners and sits on four rails.",
    "We cleaned the frame, oiled the wood and rewove the seat with new natural papercord. It works \
     at a dining table or a desk.",
    "Dimensions: W 49 × D 46 × H 78 cm, seat height 45 cm.",
];
const CLEAN_TITLE: &str = "Danish Beech Dining Chair with Curved Back";

fn answer(description: &[&str], questions: Json) -> Json {
    json!({
        "texts": {
            "title": {"text": CLEAN_TITLE, "facts": [], "reason": "From the fields."},
            "description": {"paragraphs": description, "facts": [DESIGNER], "reason": "From the note."},
        },
        "findings": [],
        "questions": questions,
    })
}

/// The writer's fake model answers the three texts' call (and its rewrite) with `reply`.
fn script(a: &TestApp, reply: Json) {
    let model = &a.research_fakes().model;
    model.reply(Stage::Write, "texts", reply.clone());
    model.reply(Stage::Write, "retry", reply);
}

fn write_runs(a: &TestApp, s: i64) -> Vec<RunRow> {
    a.app
        .store
        .research_transaction(|tx| rstore::all_runs(tx.conn(), Some(s)))
        .unwrap()
        .into_iter()
        .filter(|r| r.trigger == RunTrigger::Write)
        .collect()
}

fn ticks(a: &TestApp) {
    for _ in 0..3 {
        a.tick(1_100).unwrap();
    }
}

fn form_value(a: &TestApp, item: i64, key: &str) -> Option<psm_core::domain::attribute::Value> {
    a.attributes(Some("shopify"))
        .form_get(item)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.fields)
        .find(|f| f.definition.key == key)
        .and_then(|f| f.value)
}

fn finding_state(a: &TestApp, id: i64) -> String {
    a.app
        .store
        .research_transaction(|tx| {
            Ok(tx
                .conn()
                .query_row("SELECT state FROM writer_findings WHERE id=?1", [id], |r| {
                    r.get::<_, String>(0)
                })
                .unwrap())
        })
        .unwrap()
}

#[test]
fn w3_c4a_saving_the_writer_slot_never_rebuilds_and_auto_off_queues_nothing() {
    let mut a = app();
    let (s, _) = researched(&a, "W3-C4A");
    let prepared = a
        .app
        .settings_set_context()
        .prepare(&json!({"writer": {"auto": false}}))
        .unwrap();
    assert!(
        matches!(prepared, PreparedSettings::Simple(_)),
        "a writer save must not rebuild the App"
    );
    a.app.apply_settings(prepared).unwrap();
    let stored = psm_infra::settings_file::Settings::open(&a.app.paths)
        .unwrap()
        .writer;
    assert_eq!(stored, Some(json!({"auto": false})));
    let answer = writer(&a).request(s, Origin::NextToReview).unwrap();
    assert_eq!(
        answer,
        Requested {
            queued: false,
            reason: None
        }
    );
    assert!(write_runs(&a, s).is_empty());
    // A theme-like save through the full path also leaves the App in place.
    a.settings_set(&json!({"writer": {"language": "Swedish"}}))
        .unwrap();
    assert_eq!(
        psm_infra::settings_file::Settings::open(&a.app.paths)
            .unwrap()
            .writer,
        Some(json!({"auto": false, "language": "Swedish"}))
    );
}

#[test]
fn w3_c4b_an_invalid_writer_slot_is_refused_and_nothing_is_saved() {
    let mut a = app();
    let before = fs::read(a.app.paths.settings()).unwrap();
    let error = a
        .settings_set(&json!({"writer": {"max_questions": 99}}))
        .unwrap_err()
        .to_string();
    assert!(error.contains("max_questions"), "{error}");
    assert_eq!(fs::read(a.app.paths.settings()).unwrap(), before);
}

#[test]
fn w3_c5a_write_texts_queues_a_run_whose_texts_reach_the_form() {
    let a = app();
    let (s, item) = researched(&a, "W3-C5A");
    script(&a, answer(&CLEAN_PARAGRAPHS, json!([])));
    a.events.0.lock().unwrap().clear();
    let requested = writer(&a).request(s, Origin::WriteTexts).unwrap();
    assert_eq!(
        requested,
        Requested {
            queued: true,
            reason: None
        }
    );
    ticks(&a);
    let runs = write_runs(&a, s);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].state, RunState::Finished);
    let title = form_value(&a, item, "product.title").unwrap();
    assert_eq!(title.value, json!(CLEAN_TITLE).to_string());
    assert_eq!(title.ai_status.map(|s| s.as_str()), Some("suggested"));
    let description = form_value(&a, item, "product.description").unwrap();
    assert_eq!(
        description.json().unwrap(),
        json!(CLEAN_PARAGRAPHS.join("\n\n"))
    );
    assert_eq!(description.ai_status.map(|s| s.as_str()), Some("suggested"));
    let events = a.events.0.lock().unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AppEvent::ResearchChanged { session_id } if *session_id == s)),
        "{events:?}"
    );
}

#[test]
fn w3_c5b_write_texts_while_research_runs_is_refused_with_its_sentence() {
    let a = app();
    let (s, item) = session(&a, "W3-C5B");
    research_run(&a, s, item, RunState::Running);
    let requested = writer(&a).request(s, Origin::WriteTexts).unwrap();
    assert!(!requested.queued);
    assert_eq!(
        requested.reason.unwrap().sentence,
        "Research is still working on this session; write texts when it finishes"
    );
    assert!(write_runs(&a, s).is_empty());
}

/// A finished writer run whose description failed the post-check twice and that asked one
/// question about the designer.
fn flagged_with_a_question(a: &TestApp, sku: &str) -> (i64, i64) {
    let (s, item) = researched(a, sku);
    let mut stunning = CLEAN_PARAGRAPHS.to_vec();
    stunning[0] = "A stunning dining chair made in Denmark with a frame of solid beech and a seat \
                   of woven papercord.";
    script(
        a,
        answer(&stunning, json!([{"keys": [DESIGNER], "text": QUESTION}])),
    );
    assert!(writer(a).request(s, Origin::WriteTexts).unwrap().queued);
    ticks(a);
    (s, item)
}

#[test]
fn w3_c5c_status_shows_the_run_its_flagged_text_the_ask_the_cost_and_the_inputs() {
    let a = app();
    let (s, _) = flagged_with_a_question(&a, "W3-C5C");
    let run = write_runs(&a, s).pop().unwrap();
    let status = writer(&a).status(s).unwrap();
    assert_eq!(status.run.as_ref().unwrap().id, run.id);
    assert_eq!(status.run.as_ref().unwrap().state, "finished");
    let flagged: Vec<_> = status.texts.iter().filter(|t| t.flagged).collect();
    assert_eq!(flagged.len(), 1, "{:?}", status.texts);
    assert_eq!(flagged[0].key, "product.description");
    assert!(
        flagged[0]
            .hits
            .iter()
            .any(|h| h.code == "banned" && h.detail.contains("stunning")),
        "{:?}",
        flagged[0].hits
    );
    assert_eq!(status.findings.len(), 1, "{:?}", status.findings);
    let ask = &status.findings[0];
    assert_eq!(ask.finding_type, "question");
    assert_eq!(ask.keys, [DESIGNER]);
    assert_eq!(ask.mode, "ask");
    assert_eq!(ask.text, QUESTION);
    assert!(run.cost_micro_usd > 0);
    assert_eq!(status.cost_micro_usd, run.cost_micro_usd);
    let input: Json = serde_json::from_str(run.input_snapshot.as_deref().unwrap()).unwrap();
    let fields = input["fields"].as_array().unwrap().len();
    assert!(fields > 0);
    assert_eq!(status.inputs.fields, fields);
    assert_eq!(status.inputs.note, 2);
    assert_eq!(status.inputs.listings, ["Match listing chair"]);
}

#[test]
fn w3_c5d_an_answer_writes_the_texts_again_with_the_answer_in_the_input() {
    let a = app();
    let (s, _) = flagged_with_a_question(&a, "W3-C5D");
    let id = writer(&a).status(s).unwrap().findings[0].id;
    let requested = writer(&a).answer(id, "Teak").unwrap();
    assert!(requested.queued, "{requested:?}");
    ticks(&a);
    assert_eq!(finding_state(&a, id), "answered");
    let runs = write_runs(&a, s);
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[1].state, RunState::Finished);
    let input: Json = serde_json::from_str(runs[1].input_snapshot.as_deref().unwrap()).unwrap();
    assert_eq!(
        input["answers"],
        json!([{"type": "question", "keys": [DESIGNER], "question": QUESTION, "answer": "Teak"}])
    );
}

#[test]
fn w3_c5e_dismissing_a_note_queues_nothing() {
    let a = app();
    let (s, item) = researched(&a, "W3-C5E");
    let id = a
        .app
        .store
        .research_transaction(|tx| {
            tx.conn()
                .execute(
                    "INSERT INTO writer_findings (item_id, session_id, run_id, kind, type, keys, \
                     text_field, identity, text, mode, state, created_at) VALUES (?1, ?2, 0, \
                     'finding', 'field_vs_field', '[\"custom.designer\"]', '', \
                     'field_vs_field|custom.designer|', 'The era does not fit the designer', \
                     'note', 'open', ?3)",
                    rusqlite::params![item, s, NOW_MS],
                )
                .unwrap();
            Ok(tx.conn().last_insert_rowid())
        })
        .unwrap();
    assert_eq!(writer(&a).status(s).unwrap().findings[0].mode, "note");
    writer(&a).dismiss(id).unwrap();
    assert_eq!(finding_state(&a, id), "dismissed");
    assert!(writer(&a).status(s).unwrap().findings.is_empty());
    ticks(&a);
    assert!(write_runs(&a, s).is_empty());
}

#[test]
fn w3_c6b_without_the_model_key_the_writer_says_why_and_publish_still_works() {
    let a = app();
    a.secrets
        .delete(psm_research::service::MODEL_ACCOUNT)
        .unwrap();
    let (s, _) = session(&a, "W3-C6B");
    let requested = writer(&a).request(s, Origin::WriteTexts).unwrap();
    assert!(!requested.queued);
    assert_eq!(
        requested.reason.unwrap().sentence,
        "Add the model key in Settings → Research"
    );
    assert!(write_runs(&a, s).is_empty());
    a.confirm_as_shown(&[s], "Publish").unwrap();
    for _ in 0..15 {
        a.tick(1_100).unwrap();
        a.complete_all_renders().unwrap();
    }
    let latest = a.store().latest_publish(s).unwrap().unwrap().item_id;
    assert_eq!(
        a.store().publish_item(latest).unwrap().step,
        PublishStep::Done
    );
}
