//! R4·U1: the research image guard, the stored copies, the fakes' log and the engine's tables.

use psm_core::domain::photo::Classification;
use psm_research::{
    domain::{RunState, Stage},
    fakes::{self, ModelScript},
    images::NoSource,
    ports::{ModelProvider, ModelRequest},
};
use serde_json::json;

use crate::harness::Harness;

fn request(
    stage: Stage,
    unit_key: &str,
    images: Vec<psm_research::images::ResearchImage>,
) -> ModelRequest {
    ModelRequest {
        stage,
        unit_key: unit_key.to_owned(),
        model: "m".into(),
        effort: "low".into(),
        language: "English".into(),
        instructions: String::new(),
        input: json!({}),
        images,
        listing_images: Vec::new(),
    }
}

#[test]
fn a10_c3a_deleted_photo_is_refused() {
    let h = Harness::new();
    let photo = h.add_main_photo_with_preview(h.session_id);
    let run = h.insert_run(RunState::Running, None, None);
    h.app.review().delete(photo).unwrap();
    assert_eq!(
        h.images.render(photo, run, 1024),
        Err(NoSource::NotEligible)
    );
    assert!(h.research_files().is_empty());
}

#[test]
fn a10_c3b_photo_moved_to_another_session_is_refused() {
    let h = Harness::new();
    let photo = h.add_main_photo_with_preview(h.session_id);
    let run = h.insert_run(RunState::Running, None, None);
    let (_, other) = h.new_session("R4-OTHER");
    h.app.review().move_photos(&[photo], other).unwrap();
    assert_eq!(h.photo(photo).session_id, other);
    assert_eq!(
        h.images.render(photo, run, 1024),
        Err(NoSource::NotEligible)
    );
    assert!(h.research_files().is_empty());
}

#[test]
fn a10_c3c_main_condition_unreviewed_are_rendered() {
    let h = Harness::new();
    let main = h.add_photo_with_preview(h.session_id, Classification::Main);
    let condition = h.add_photo_with_preview(h.session_id, Classification::Condition);
    let unreviewed = h.add_photo_with_preview(h.session_id, Classification::Unreviewed);
    assert_eq!(
        h.photo(unreviewed).classification,
        Classification::Unreviewed
    );
    let run = h.insert_run(RunState::Running, None, None);
    for photo in [main, condition, unreviewed] {
        let image = h.images.render(photo, run, 1024).unwrap();
        assert_eq!(image.photo_id(), photo);
        assert!(image.path().is_file());
    }
    assert_eq!(h.research_files().len(), 3);
}

#[test]
fn a10_c3d_original_preview_without_adjusted_is_no_source() {
    let h = Harness::new();
    let photo = h.add_photo_without_preview(h.session_id, Classification::Main);
    h.give_original_preview(photo);
    assert!(h.photo(photo).original_preview_path.is_some());
    let run = h.insert_run(RunState::Running, None, None);
    assert_eq!(
        h.images.render(photo, run, 1024),
        Err(NoSource::NoAdjustedPreview)
    );
    assert!(h.research_files().is_empty());
}

#[test]
fn a10_c4a_same_photo_twice_is_one_copy() {
    let h = Harness::new();
    let photo = h.add_main_photo_with_preview(h.session_id);
    let run = h.insert_run(RunState::Running, None, None);
    let first = h.images.render(photo, run, 1024).unwrap();
    let second = h.images.render(photo, run, 1024).unwrap();
    assert_eq!(first.sha256(), second.sha256());
    assert_eq!(first.path(), second.path());
    assert_eq!(h.research_files(), vec![first.path().to_path_buf()]);
}

#[test]
fn a10_c4b_recomposed_photo_is_a_new_copy() {
    let h = Harness::new();
    let photo = h.add_main_photo_with_preview(h.session_id);
    let run = h.insert_run(RunState::Running, None, None);
    let before = h.images.render(photo, run, 1024).unwrap();
    let revision = h.app.composition(photo).unwrap().revision;
    h.recompose(photo);
    assert_eq!(h.app.composition(photo).unwrap().revision, revision + 1);
    let after = h.images.render(photo, run, 1024).unwrap();
    assert_ne!(before.sha256(), after.sha256());
    assert!(before.path().is_file() && after.path().is_file());
    assert_eq!(h.research_files().len(), 2);
}

#[test]
fn a10_c5a_fake_model_fault_then_reply_and_log() {
    let h = Harness::new();
    let photo = h.add_main_photo_with_preview(h.session_id);
    let run = h.insert_run(RunState::Running, None, None);
    let image = h.images.render(photo, run, 1024).unwrap();
    let unit = format!("photo:{photo}");
    let reply = json!({"reads": {"custom.wood": {"value": "Teak", "sure": true}}});
    h.model.script(
        Stage::Observe,
        &unit,
        ModelScript {
            output: reply.clone(),
            usage: None,
            response: None,
        },
    );
    h.model.fault(fakes::transient("timed out"));
    let first = h
        .model
        .call(&request(Stage::Observe, &unit, vec![image.clone()]));
    assert_eq!(first, Err(fakes::transient("timed out")));
    let second = h
        .model
        .call(&request(Stage::Observe, &unit, vec![image.clone()]))
        .unwrap();
    assert_eq!(second.output, reply);
    let log = h.model.requests();
    assert_eq!(log.len(), 2);
    for call in log {
        assert_eq!(call.images, vec![(image.sha256().to_owned(), 1024)]);
    }
}

#[test]
fn a11_c8a_one_running_and_one_queued_per_session() {
    let h = Harness::new();
    h.insert_run(RunState::Running, None, None);
    h.insert_run(RunState::Queued, None, None);
    let insert = |state: RunState| {
        let now = h.now();
        h.store().research_transaction(|tx| {
            psm_research::store::insert_run(
                tx.conn(),
                &psm_research::store::NewRun {
                    item_id: h.item_id,
                    session_id: h.session_id,
                    trigger: psm_research::domain::RunTrigger::Auto,
                    state,
                    queued_at: now,
                    started_at: None,
                    settings_snapshot: None,
                    form_snapshot: None,
                    input_snapshot: None,
                },
            )
        })
    };
    assert!(insert(RunState::Running).is_err());
    assert!(insert(RunState::Queued).is_err());
    let count: i64 = h.query(|c| {
        c.query_row("SELECT COUNT(*) FROM research_runs", [], |r| r.get(0))
            .unwrap()
    });
    assert_eq!(count, 2);
}
