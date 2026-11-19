//! W1 (book 08C §4; contracts WC1, WC2 of `master_plan_writer.md` §5): the product note. Unit 1:
//! stages 1 and 6 ask for the note and store it (rows W1·C2–C5). Runs on the research harness
//! with the crate's fakes; the outcome is the requests the fakes received and the committed unit
//! outputs.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use psm_core::domain::{
    attribute::{AttributeGroup, CoreType, Definition, Preset, PresetField},
    photo::Classification,
};
use psm_research::{
    config::ResearchConfig,
    domain::{RunState, Stage},
    fakes,
    rules::fingerprint::fingerprint,
    stages::OBSERVE_UNIT,
};
use serde_json::{Value as Json, json};

use crate::harness::{Harness, INTEGRATION, SCRATCHES, WOOD};

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
