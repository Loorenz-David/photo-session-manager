//! W1 (book 08C §4; contracts WC1, WC2 of `master_plan_writer.md` §5): the product note. Unit 1:
//! stages 1 and 6 ask for the note and store it (rows W1·C2–C5). Unit 2: which note counts, the
//! owner's decisions over it, staleness, Look again and the research views (rows W1·C7(e),
//! C8–C12). Runs on the research harness with the crate's fakes; the outcome is the requests the
//! fakes received, the committed unit outputs, the returned views and the run rows.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use psm_core::{
    domain::{
        attribute::{
            AiConfidence, AiStatus, AttributeGroup, CoreType, Definition, Preset, PresetField,
            Provenance, Value,
        },
        photo::Classification,
    },
    ports::secrets::SecretStore,
};
use psm_research::{
    api::ResearchApi,
    config::ResearchConfig,
    domain::{RunState, RunTrigger, Stage},
    fakes,
    rules::fingerprint::fingerprint,
    stages::OBSERVE_UNIT,
    store::{self as rstore, NewRun, OutputRow},
    writer::{
        api::WriterApi,
        note::{NoteView, PHOTO_GONE, reviewed},
    },
};
use serde_json::{Value as Json, json};

use crate::harness::{Harness, INTEGRATION, MEDIA, SCRATCHES, WOOD};

/// The sentence §9.3 adds to the Observe instructions.
const OBSERVE_NOTE_SENTENCE: &str = "Also write `note`: short factual observations of what the \
    piece is, its form, construction, materials and visible details, each tagged with the keys \
    from `note_fields` it bears on (none when it bears on no field).";

pub(crate) const SHAPE: &str = "custom.shape";
pub(crate) const CONDITION_SCRATCH: &str = "custom.condition_scratch";

/// Adds definitions of the harness's integration (the import replaces the whole set).
pub(crate) fn add_definitions(h: &Harness, extra: Vec<Definition>) {
    let attributes = h.app.attributes(Some(INTEGRATION));
    let mut all: Vec<Definition> = attributes
        .definitions(Some(INTEGRATION))
        .unwrap()
        .into_iter()
        .filter(|d| d.integration == INTEGRATION)
        .collect();
    all.extend(extra);
    attributes.import_definitions(INTEGRATION, all).unwrap();
}

pub(crate) fn definition(
    key: &str,
    label: &str,
    core_type: CoreType,
    group: AttributeGroup,
) -> Definition {
    Definition {
        integration: INTEGRATION.into(),
        key: key.into(),
        label: label.into(),
        core_type,
        choices: Vec::new(),
        validations: json!({}),
        group,
        description: None,
        writable: true,
    }
}

/// Sets the layout of one step of an item type.
pub(crate) fn layout(h: &Harness, item_type: &str, group: AttributeGroup, keys: &[&str]) {
    h.app
        .attributes(Some(INTEGRATION))
        .presets_set(Preset {
            item_type: item_type.into(),
            group,
            layout: keys
                .iter()
                .map(|k| PresetField {
                    key: (*k).into(),
                    default: None,
                })
                .collect(),
        })
        .unwrap();
}

/// C2(a)'s harness: an item of type Table whose layout is Details `custom.wood` (chosen),
/// `custom.shape` (not chosen), Condition `custom.condition_scratch`; the Product step is the
/// default one (it holds `product.title`).
fn table() -> Harness {
    let mut h = Harness::new();
    add_definitions(
        &h,
        vec![
            definition(SHAPE, "Shape", CoreType::Text, AttributeGroup::Details),
            definition(
                CONDITION_SCRATCH,
                "Scratches",
                CoreType::Text,
                AttributeGroup::Condition,
            ),
        ],
    );
    layout(&h, "Table", AttributeGroup::Details, &[WOOD, SHAPE]);
    layout(&h, "Table", AttributeGroup::Condition, &[CONDITION_SCRATCH]);
    let (item, session) = h
        .app
        .items()
        .create_local("W1-1", "Teak table", "Table", None)
        .unwrap();
    h.item_id = item.id;
    h.session_id = session.session.id;
    h
}

fn requests(h: &Harness, stage: Stage) -> Vec<fakes::FakeCall> {
    h.model
        .requests()
        .into_iter()
        .filter(|c| c.stage == Some(stage))
        .collect()
}

/// The committed output of a run's unit, with the run it was copied from.
fn output(h: &Harness, run: i64, stage: Stage) -> (Json, Option<i64>) {
    let row = h
        .outputs(run)
        .into_iter()
        .find(|o| o.stage == stage.as_str())
        .unwrap_or_else(|| panic!("no {} output in run {run}", stage.as_str()));
    (
        serde_json::from_str(&row.output).unwrap(),
        row.reused_from_run_id,
    )
}

/// One run of stage 1 on the Sideboard harness (one Main photo), the Observe reply scripted.
fn observe_run(reply: Json) -> (Harness, i64) {
    let h = Harness::new();
    h.add_main_photo_with_preview(h.session_id);
    h.model.reply(Stage::Observe, OBSERVE_UNIT, reply);
    let run = h.start_run(&h.config());
    h.execute(run);
    (h, run)
}

#[test]
fn w1_c2a_stage_1_asks_for_a_note_over_the_details_and_condition_fields() {
    let h = table();
    h.add_main_photo_with_preview(h.session_id);
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    let observe = requests(&h, Stage::Observe);
    assert_eq!(observe.len(), 1);
    assert_eq!(
        observe[0].input["note_fields"],
        json!([
            {"key": WOOD, "label": WOOD},
            {"key": SHAPE, "label": "Shape"},
            {"key": CONDITION_SCRATCH, "label": "Scratches"},
        ])
    );
    let fields: Vec<&str> = observe[0].input["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["key"].as_str().unwrap())
        .collect();
    assert_eq!(fields, vec![WOOD]);
    let call = h
        .calls(run)
        .into_iter()
        .find(|c| c.stage == "observe")
        .unwrap();
    let request: Json = serde_json::from_str(&call.request).unwrap();
    let instructions = request["instructions"].as_str().unwrap();
    assert!(
        instructions.ends_with(&format!(" {OBSERVE_NOTE_SENTENCE}")),
        "{instructions}"
    );
}

#[test]
fn w1_c2b_a_stage_1_unit_of_instructions_version_2_is_not_reused() {
    let h = Harness::chair();
    let photo = h.add_main_photo_with_preview(h.session_id);
    let mut old = h.config();
    old.instructions_version = 2;
    let first = h.start_run(&old);
    h.execute(first);
    assert_eq!(h.run_row(first).state, RunState::Finished);
    h.model.clear_requests();
    let second = h.queue_follow_up();
    h.tick(1_000);
    assert_eq!(h.run_row(second).state, RunState::Finished);
    assert_eq!(requests(&h, Stage::Observe).len(), 1);
    let config =
        ResearchConfig::from_snapshot(h.run_row(second).settings_snapshot.as_deref()).unwrap();
    assert_eq!(config.instructions_version, 3);
    let settings = config.stage(Stage::Observe);
    let row = h
        .outputs(second)
        .into_iter()
        .find(|o| o.stage == "observe")
        .unwrap();
    assert_eq!(row.reused_from_run_id, None);
    let revision = h.app.composition(photo).unwrap().revision;
    assert_eq!(
        row.fingerprint,
        fingerprint(
            1,
            &json!({
                "photos": [[photo, revision, "wide"]],
                "long_edge": settings.image_long_edge,
                "model": settings.model,
                "effort": settings.effort,
                "instructions_version": 3,
                "language": config.language,
            })
        )
        .unwrap()
    );
}

#[test]
fn w1_c3a_note_keys_are_kept_only_when_asked_for_once_each() {
    let (h, run) = observe_run(json!({"reads": {}, "marking_photos": [], "note": [
        {"text": "Frame in teak", "keys": [WOOD, "custom.unknown", WOOD]},
        {"text": "Four tapered legs", "keys": []},
    ]}));
    assert_eq!(
        output(&h, run, Stage::Observe).0["note"],
        json!([
            {"text": "Frame in teak", "keys": [WOOD]},
            {"text": "Four tapered legs", "keys": []},
        ])
    );
}

#[test]
fn w1_c3b_blank_observations_are_dropped_and_long_ones_cut_at_300_characters() {
    // Two-byte letters before the cut: a cut by bytes would keep only 150 of them.
    let long = format!("{}ø{}", "å".repeat(300), "b".repeat(49));
    assert_eq!(long.chars().count(), 350);
    let (h, run) = observe_run(json!({"reads": {}, "marking_photos": [], "note": [
        {"text": "   ", "keys": []},
        {"text": long, "keys": []},
    ]}));
    assert_eq!(
        output(&h, run, Stage::Observe).0["note"],
        json!([{"text": "å".repeat(300), "keys": []}])
    );
}

#[test]
fn w1_c3c_at_most_20_observations_in_reply_order() {
    let note: Vec<Json> = (1..=25)
        .map(|i| json!({"text": format!("Observation {i}"), "keys": []}))
        .collect();
    let (h, run) = observe_run(json!({"reads": {}, "marking_photos": [], "note": note}));
    let expected: Vec<Json> = (1..=20)
        .map(|i| json!({"text": format!("Observation {i}"), "keys": []}))
        .collect();
    assert_eq!(output(&h, run, Stage::Observe).0["note"], json!(expected));
}

#[test]
fn w1_c3d_a_reply_without_note_stores_an_empty_note() {
    let (h, run) = observe_run(json!({"reads": {}, "marking_photos": []}));
    assert_eq!(output(&h, run, Stage::Observe).0["note"], json!([]));
    assert_eq!(h.run_row(run).state, RunState::Finished);
}

/// Three Condition photos with sources on the Chair harness (the scratches field is chosen).
fn condition_photos(h: &Harness) -> Vec<i64> {
    (0..3)
        .map(|_| h.add_photo_with_preview(h.session_id, Classification::Condition))
        .collect()
}

#[test]
fn w1_c4a_stage_6_numbers_its_photos_and_asks_over_the_condition_fields() {
    let h = Harness::chair();
    let photos = condition_photos(&h);
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    let condition = requests(&h, Stage::Condition);
    assert_eq!(condition.len(), 1);
    assert_eq!(
        condition[0].input["photos"],
        json!([{"index": 1}, {"index": 2}, {"index": 3}])
    );
    let attached: Vec<String> = photos
        .iter()
        .map(|p| h.images.render(*p, run, 2048).unwrap().sha256().to_owned())
        .collect();
    let received: Vec<String> = condition[0]
        .images
        .iter()
        .map(|(sha, _)| sha.clone())
        .collect();
    assert_eq!(received, attached);
    assert_eq!(
        condition[0].input["note_fields"],
        json!([{"key": SCRATCHES, "label": SCRATCHES}])
    );
}

#[test]
fn w1_c4b_a_note_photo_index_counts_only_the_photos_sent() {
    let h = Harness::chair();
    let photos = condition_photos(&h);
    let gone = photos[1];
    // The first condition call fails after the photo was deleted; the retry leaves it out (the
    // gateway's eligibility re-read), so the model receives two images.
    h.model
        .fault_for(Stage::Condition, "condition", fakes::transient("timeout"));
    let store = h.store().clone();
    let fired = Arc::new(AtomicBool::new(false));
    h.model.before_call(move |call| {
        if call.stage == Some(Stage::Condition) && !fired.swap(true, Ordering::SeqCst) {
            store
                .research_transaction(|tx| {
                    tx.conn()
                        .execute(
                            "UPDATE photos SET classification='deleted' WHERE id=?1",
                            [gone],
                        )
                        .map(|_| ())
                        .map_err(|e| psm_core::Error::InvalidData(e.to_string()))
                })
                .unwrap();
        }
    });
    h.model.reply(
        Stage::Condition,
        "condition",
        json!({"reads": {}, "note": [
            {"photo": 1, "text": "Scratch on the left arm", "keys": [SCRATCHES]},
            {"photo": 2, "text": "Worn seat edge", "keys": [SCRATCHES]},
        ]}),
    );
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    let condition = requests(&h, Stage::Condition);
    assert_eq!(condition.last().unwrap().images.len(), 2);
    assert_eq!(
        output(&h, run, Stage::Condition).0["note"],
        json!([
            {"text": "Scratch on the left arm", "keys": [SCRATCHES], "photo_id": photos[0]},
            {"text": "Worn seat edge", "keys": [SCRATCHES], "photo_id": photos[2]},
        ])
    );
}

#[test]
fn w1_c4c_a_note_photo_out_of_range_has_no_photo() {
    let h = Harness::chair();
    condition_photos(&h);
    h.model.reply(
        Stage::Condition,
        "condition",
        json!({"reads": {}, "note": [{"photo": 7, "text": "Dent", "keys": []}]}),
    );
    let run = h.start_run(&h.stage_config());
    h.execute(run);
    assert_eq!(
        output(&h, run, Stage::Condition).0["note"],
        json!([{"text": "Dent", "keys": [], "photo_id": null}])
    );
}

#[test]
fn w1_c5a_an_unchanged_stage_1_unit_is_copied_with_its_note() {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    h.model.reply(
        Stage::Observe,
        OBSERVE_UNIT,
        json!({"reads": {}, "marking_photos": [],
               "note": [{"text": "A lounge chair in teak", "keys": []}]}),
    );
    h.changed();
    h.tick(1_000);
    let first = h.runs(h.session_id)[0].id;
    assert_eq!(h.run_row(first).state, RunState::Finished);
    let (first_output, _) = output(&h, first, Stage::Observe);
    h.model.clear_requests();
    let second = h.queue_follow_up();
    h.tick(1_000);
    assert_eq!(h.run_row(second).state, RunState::Finished);
    assert!(requests(&h, Stage::Observe).is_empty());
    let (copied, from) = output(&h, second, Stage::Observe);
    assert_eq!(from, Some(first));
    assert_eq!(
        copied["note"],
        json!([{"text": "A lounge chair in teak", "keys": []}])
    );
    assert_eq!(copied["note"], first_output["note"]);
}

// ---- Unit 2 ----

fn note(h: &Harness) -> NoteView {
    WriterApi::note(h.service.as_ref(), h.session_id).unwrap()
}

/// Gives a definition of the harness's integration another label.
fn relabel(h: &Harness, key: &str, label: &str) {
    let attributes = h.app.attributes(Some(INTEGRATION));
    let all: Vec<Definition> = attributes
        .definitions(Some(INTEGRATION))
        .unwrap()
        .into_iter()
        .filter(|d| d.integration == INTEGRATION)
        .map(|mut d| {
            if d.key == key {
                d.label = label.into();
            }
            d
        })
        .collect();
    attributes.import_definitions(INTEGRATION, all).unwrap();
}

/// A stored row of the item.
fn put(h: &Harness, key: &str, value: Json, provenance: Provenance, status: Option<AiStatus>) {
    h.put(&Value {
        item_id: h.item_id,
        key: key.into(),
        value: value.to_string(),
        valid: true,
        provenance,
        ai_confidence: status.map(|_| AiConfidence::Medium),
        ai_status: status,
        ai_reason: None,
        ai_sources: None,
        ai_prior: None,
        updated_at: h.now(),
    });
}

/// The Table harness (wood labelled "Wood") after one stage 1 run whose note is `observations`.
fn table_with_note(observations: Json) -> Harness {
    let h = table();
    relabel(&h, WOOD, "Wood");
    h.add_main_photo_with_preview(h.session_id);
    h.model.reply(
        Stage::Observe,
        OBSERVE_UNIT,
        json!({"reads": {}, "marking_photos": [], "note": observations}),
    );
    let run = h.start_run(&h.config());
    h.execute(run);
    assert_eq!(h.run_row(run).state, RunState::Finished);
    h
}

fn oak_frame() -> Harness {
    table_with_note(json!([{"text": "Frame in oak", "keys": [WOOD]}]))
}

fn saw(view: &NoteView) -> Vec<(String, String)> {
    view.research_saw
        .iter()
        .map(|s| (s.key.clone(), s.text.clone()))
        .collect()
}

#[test]
fn w1_c7e_keep_previous_is_reviewed_until_it_is_undone() {
    let h = Harness::new();
    put(&h, WOOD, json!("Oak"), Provenance::Default, None);
    h.suggest(WOOD, json!("Teak"));
    let state = |h: &Harness| {
        let row = h.row(WOOD);
        let mut feedback = Vec::new();
        h.app
            .store()
            .transaction(&mut |tx| {
                feedback = tx.feedback_by_key(h.item_id, WOOD)?;
                Ok(())
            })
            .unwrap();
        reviewed(row.as_ref(), &feedback)
    };
    assert!(!state(&h));
    h.app
        .attributes(Some(INTEGRATION))
        .keep_previous(h.item_id, WOOD)
        .unwrap();
    assert!(state(&h));
    h.app.history().undo(h.session_id).unwrap();
    assert!(!state(&h));
}

#[test]
fn w1_c8a_an_accepted_value_replaces_the_observation() {
    let h = oak_frame();
    put(
        &h,
        WOOD,
        json!("Teak"),
        Provenance::Ai,
        Some(AiStatus::Accepted),
    );
    let view = note(&h);
    assert_eq!(view.observations.len(), 1);
    assert_eq!(view.observations[0].text, "Wood: Teak (owner's decision)");
    assert!(view.observations[0].owner);
    assert_eq!(saw(&view), vec![]);
}

#[test]
fn w1_c8b_a_typed_value_replaces_the_observation_and_keeps_what_research_saw() {
    let h = oak_frame();
    put(&h, WOOD, json!("Teak"), Provenance::Human, None);
    let view = note(&h);
    assert_eq!(view.observations[0].text, "Wood: Teak (owner's decision)");
    assert!(view.observations[0].owner);
    assert_eq!(
        saw(&view),
        vec![(WOOD.to_owned(), "Frame in oak".to_owned())]
    );
}

#[test]
fn w1_c8c_an_imported_value_keeps_the_observation() {
    let h = oak_frame();
    put(&h, WOOD, json!("Oak"), Provenance::Integration, None);
    let view = note(&h);
    assert_eq!(view.observations[0].text, "Frame in oak");
    assert!(!view.observations[0].owner);
    assert_eq!(
        saw(&view),
        vec![(WOOD.to_owned(), "Frame in oak".to_owned())]
    );
}

#[test]
fn w1_c8d_a_dismissed_value_is_stated_as_dismissed() {
    let h = oak_frame();
    put(
        &h,
        WOOD,
        json!("Oak"),
        Provenance::Ai,
        Some(AiStatus::Cleared),
    );
    let view = note(&h);
    assert_eq!(
        view.observations[0].text,
        "Wood: not stated (the owner dismissed Oak)"
    );
    assert!(view.observations[0].owner);
}

#[test]
fn w1_c8e_one_decided_key_replaces_and_the_undecided_key_keeps_the_words() {
    let h = table_with_note(json!([
        {"text": "Round top in oak", "keys": [WOOD, SHAPE]},
    ]));
    put(
        &h,
        WOOD,
        json!("Teak"),
        Provenance::Ai,
        Some(AiStatus::Accepted),
    );
    let view = note(&h);
    assert_eq!(view.observations[0].text, "Wood: Teak (owner's decision)");
    assert_eq!(
        saw(&view),
        vec![(SHAPE.to_owned(), "Round top in oak".to_owned())]
    );
}

#[test]
fn w1_c8f_condition_photos_are_numbered_among_the_selected_ones() {
    let h = table();
    layout(&h, "Table", AttributeGroup::Details, &[WOOD, SHAPE, MEDIA]);
    let photos = condition_photos(&h);
    h.model.reply(
        Stage::Condition,
        "condition",
        json!({"reads": {}, "note": [
            {"photo": 1, "text": "Scratch on the top", "keys": [CONDITION_SCRATCH]},
            {"photo": 2, "text": "Ring mark", "keys": [CONDITION_SCRATCH]},
            {"photo": 3, "text": "Chipped corner", "keys": [CONDITION_SCRATCH]},
        ]}),
    );
    let mut config = h.config();
    config.fill_keys.push(CONDITION_SCRATCH.into());
    let run = h.start_run(&config);
    h.execute(run);
    put(
        &h,
        MEDIA,
        json!({"excluded": [photos[1]]}),
        Provenance::Human,
        None,
    );
    let view = note(&h);
    let shown: Vec<(&str, Option<&str>, bool)> = view
        .observations
        .iter()
        .map(|o| (o.text.as_str(), o.photo.as_deref(), o.in_writer_input))
        .collect();
    assert_eq!(
        shown,
        vec![
            ("Scratch on the top", Some("condition photo 1"), true),
            ("Ring mark", Some(PHOTO_GONE), false),
            ("Chipped corner", Some("condition photo 2"), true),
        ]
    );
}

/// Inserts a run of the session with an observe unit whose output holds `note` (no `note` key
/// when `None`), as a run of that trigger commits it.
fn run_with_note(h: &Harness, trigger: RunTrigger, state: RunState, note: Option<Json>) -> i64 {
    let now = h.now();
    let mut output = json!({"photos": [], "reads": {}, "marking_photo_ids": [], "files": []});
    if let Some(note) = note {
        output["note"] = note;
    }
    h.query(|c| {
        let run = rstore::insert_run(
            c,
            &NewRun {
                item_id: h.item_id,
                session_id: h.session_id,
                trigger,
                state,
                queued_at: now,
                started_at: Some(now),
                settings_snapshot: None,
                form_snapshot: None,
                input_snapshot: None,
            },
        )
        .unwrap();
        rstore::insert_output(
            c,
            &OutputRow {
                run_id: run,
                stage: "observe".into(),
                unit_key: OBSERVE_UNIT.into(),
                fingerprint: format!("f{run}"),
                output: output.to_string(),
                reused_from_run_id: None,
                committed_at: now,
            },
        )
        .unwrap();
        run
    })
}

fn texts(view: &NoteView) -> Vec<String> {
    view.observations.iter().map(|o| o.text.clone()).collect()
}

#[test]
fn w1_c9a_the_latest_look_counts_and_a_browse_run_never_does() {
    let h = Harness::new();
    let one = |text: &str| Some(json!([{"text": text, "keys": []}]));
    run_with_note(
        &h,
        RunTrigger::Arranged,
        RunState::Finished,
        one("From research"),
    );
    run_with_note(
        &h,
        RunTrigger::Look,
        RunState::Finished,
        one("From Look again"),
    );
    run_with_note(
        &h,
        RunTrigger::Browse,
        RunState::Finished,
        one("From browsing"),
    );
    assert_eq!(texts(&note(&h)), vec!["From Look again"]);
}

#[test]
fn w1_c9b_an_output_without_a_note_does_not_count() {
    let h = Harness::new();
    run_with_note(
        &h,
        RunTrigger::Arranged,
        RunState::Finished,
        Some(json!([{"text": "From research", "keys": []}])),
    );
    run_with_note(&h, RunTrigger::Manual, RunState::Finished, None);
    assert_eq!(texts(&note(&h)), vec!["From research"]);
}

/// The Chair harness after a first arranged run over `mains` Main photos and `conditions`
/// Condition photos; the fakes' logs cleared. Returns (mains, conditions, run).
fn researched(h: &Harness, mains: usize, conditions: usize) -> (Vec<i64>, Vec<i64>, i64) {
    let main: Vec<i64> = (0..mains)
        .map(|_| h.add_main_photo_with_preview(h.session_id))
        .collect();
    let condition: Vec<i64> = (0..conditions)
        .map(|_| h.add_photo_with_preview(h.session_id, Classification::Condition))
        .collect();
    h.changed();
    h.tick(1_000);
    let run = h.runs(h.session_id).last().unwrap().id;
    assert_eq!(h.run_row(run).state, RunState::Finished);
    h.model.clear_requests();
    h.search.clear_requests();
    h.reader.clear_requests();
    (main, condition, run)
}

fn staleness(view: &NoteView) -> (bool, bool, bool) {
    (view.stale_main, view.stale_condition, view.missing_main)
}

#[test]
fn w1_c10a_nothing_changed_nothing_is_stale() {
    let h = Harness::chair();
    researched(&h, 2, 0);
    assert_eq!(staleness(&note(&h)), (false, false, false));
}

#[test]
fn w1_c10b_a_recomposed_main_photo_makes_the_note_stale() {
    let h = Harness::chair();
    let (mains, _, _) = researched(&h, 2, 0);
    h.recompose(mains[1]);
    assert!(note(&h).stale_main);
}

#[test]
fn w1_c10c_a_photo_beyond_the_limit_changes_nothing() {
    let h = Harness::chair();
    h.patch_settings(json!({"observe_max_photos": 2}));
    researched(&h, 2, 0);
    let third = h.add_main_photo_with_preview(h.session_id);
    assert_eq!(h.photo(third).position, Some(3));
    assert!(!note(&h).stale_main);
}

#[test]
fn w1_c10d_another_condition_photo_makes_the_condition_note_stale() {
    let h = Harness::chair();
    researched(&h, 1, 1);
    h.add_photo_with_preview(h.session_id, Classification::Condition);
    assert!(note(&h).stale_condition);
}

#[test]
fn w1_c10e_condition_photos_never_looked_at_are_stale() {
    let h = Harness::chair();
    h.add_photo_with_preview(h.session_id, Classification::Condition);
    assert!(note(&h).stale_condition);
}

fn look_again(h: &Harness) -> (bool, Option<String>) {
    let answer = WriterApi::look_again(h.service.as_ref(), h.session_id).unwrap();
    (answer.queued, answer.reason.map(|r| r.code))
}

fn all_runs(h: &Harness) -> Vec<rstore::RunRow> {
    h.query(|c| rstore::all_runs(c, Some(h.session_id)).unwrap())
}

#[test]
fn w1_c11a_look_again_looks_at_the_changed_photos_only() {
    let h = Harness::chair();
    let (mains, _, first) = researched(&h, 2, 1);
    let rows_before = h.end_state(first).values;
    h.recompose(mains[1]);
    assert_eq!(look_again(&h), (true, None));
    let looks: Vec<rstore::RunRow> = all_runs(&h)
        .into_iter()
        .filter(|r| r.trigger == RunTrigger::Look)
        .collect();
    assert_eq!(looks.len(), 1);
    let look = looks[0].id;
    h.tick(1_000);
    let stages: Vec<Option<Stage>> = h.model.requests().iter().map(|c| c.stage).collect();
    assert_eq!(stages, vec![Some(Stage::Observe)]);
    assert!(h.search.requests().is_empty());
    assert!(h.reader.requests().is_empty());
    let (_, copied_from) = output(&h, look, Stage::Condition);
    assert_eq!(copied_from, Some(first));
    assert_eq!(h.run_row(look).state, RunState::Finished);
    assert_eq!(h.end_state(look).values, rows_before);
    assert!(!note(&h).stale_main);
}

#[test]
fn w1_c11b_look_again_waits_for_a_queued_research_run() {
    let h = Harness::chair();
    let (mains, _, _) = researched(&h, 2, 0);
    h.recompose(mains[1]);
    h.service.run_again(h.session_id).unwrap();
    let before = all_runs(&h).len();
    assert_eq!(look_again(&h), (false, Some("Busy".into())));
    assert_eq!(all_runs(&h).len(), before);
}

#[test]
fn w1_c11c_look_again_with_unchanged_photos_is_refused() {
    let h = Harness::chair();
    researched(&h, 2, 0);
    assert_eq!(look_again(&h), (false, Some("NotStale".into())));
}

#[test]
fn w1_c11d_look_again_names_the_first_missing_prerequisite() {
    let h = Harness::chair();
    let (mains, _, _) = researched(&h, 2, 0);
    h.recompose(mains[1]);
    h.app.secrets.set("research-model-key", "").unwrap();
    let no_shop = h.service_with(|deps| deps.shop = None);
    let answer = WriterApi::look_again(&no_shop, h.session_id).unwrap();
    assert_eq!(
        (answer.queued, answer.reason.map(|r| r.code)),
        (false, Some("NoIntegration".into()))
    );
    assert_eq!(look_again(&h), (false, Some("NoModelKey".into())));
}

/// Inserts a run of the session with no output.
fn bare_run(h: &Harness, trigger: RunTrigger, state: RunState) -> i64 {
    let now = h.now();
    h.query(|c| {
        rstore::insert_run(
            c,
            &NewRun {
                item_id: h.item_id,
                session_id: h.session_id,
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

#[test]
fn w1_c12a_a_finished_look_run_does_not_stop_arranging() {
    let h = Harness::new();
    h.add_main_photo_with_preview(h.session_id);
    bare_run(&h, RunTrigger::Arranged, RunState::Failed);
    bare_run(&h, RunTrigger::Look, RunState::Finished);
    assert!(ResearchApi::arranged(h.service.as_ref(), h.session_id).unwrap());
    let queued: Vec<RunTrigger> = all_runs(&h)
        .into_iter()
        .filter(|r| r.state == RunState::Queued)
        .map(|r| r.trigger)
        .collect();
    assert_eq!(queued, vec![RunTrigger::Arranged]);
}

#[test]
fn w1_c12b_a_running_look_run_is_not_the_sessions_research() {
    let h = Harness::new();
    let first = bare_run(&h, RunTrigger::Arranged, RunState::Finished);
    bare_run(&h, RunTrigger::Look, RunState::Running);
    let status = ResearchApi::status(h.service.as_ref(), h.session_id).unwrap();
    let run = status.run.unwrap();
    assert_eq!((run.id, run.state.as_str()), (first, "finished"));
    let overview = ResearchApi::overview(h.service.as_ref(), Some(h.session_id)).unwrap();
    assert_eq!(overview.len(), 1);
    assert_eq!(overview[0].state, "done");
}

#[test]
fn w1_c12c_run_again_cancels_a_queued_look_run() {
    let h = Harness::new();
    h.add_main_photo_with_preview(h.session_id);
    let look = bare_run(&h, RunTrigger::Look, RunState::Queued);
    ResearchApi::run(h.service.as_ref(), h.session_id).unwrap();
    let row = h.run_row(look);
    assert_eq!(row.state, RunState::Cancelled);
    assert_eq!(row.error.as_deref(), Some("research started again"));
    let queued: Vec<RunTrigger> = all_runs(&h)
        .into_iter()
        .filter(|r| r.state == RunState::Queued)
        .map(|r| r.trigger)
        .collect();
    assert_eq!(queued, vec![RunTrigger::Manual]);
}
