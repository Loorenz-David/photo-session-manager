//! A6 (owner 2026-11-17): the research state of every session in one call (sessions list and
//! session header), and where a queued run stands in the line (research runs one at a time).

use psm_core::domain::attribute::AiStatus;
use psm_research::{
    api::{QueueView, ResearchApi, SessionResearch},
    domain::RunState,
};
use serde_json::json;

use crate::harness::{DESIGNER, Harness};

fn api(h: &Harness) -> &dyn ResearchApi {
    h.service.as_ref()
}

fn row(session_id: i64, state: &str, position: Option<usize>, to_review: usize) -> SessionResearch {
    SessionResearch {
        session_id,
        state: state.into(),
        position,
        to_review,
        reason: None,
        writer: None,
    }
}

/// Session A is being researched; B (arranged, queued first) and C (asked for by hand, queued
/// later) wait. MC-A6 start order puts C before B.
fn a_line() -> (Harness, i64, i64) {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    assert!(h.service.arranged(h.session_id).unwrap());
    h.service.tick().unwrap();
    assert_eq!(h.runs(h.session_id)[0].state, RunState::Running);
    let (b_item, b) = h.new_session("R4-2");
    let (c_item, c) = h.new_session("R4-3");
    let b_run = h.insert_run_for(b_item, b, RunState::Queued, None, None);
    let c_run = h.insert_run_for(c_item, c, RunState::Queued, None, None);
    h.query(|conn| {
        conn.execute(
            "UPDATE research_runs SET trigger='arranged', queued_at=100 WHERE id=?1",
            [b_run],
        )
        .unwrap();
        conn.execute(
            "UPDATE research_runs SET trigger='manual', queued_at=200 WHERE id=?1",
            [c_run],
        )
        .unwrap();
    });
    (h, b, c)
}

#[test]
fn a6_3a_one_call_gives_researching_and_each_place_in_the_line() {
    let (h, b, c) = a_line();
    assert_eq!(
        api(&h).overview(None).unwrap(),
        vec![
            row(h.session_id, "researching", None, 0),
            row(b, "waiting", Some(2), 0),
            row(c, "waiting", Some(1), 0),
        ]
    );
    assert_eq!(
        api(&h).overview(Some(b)).unwrap(),
        vec![row(b, "waiting", Some(2), 0)]
    );
}

#[test]
fn a6_4a_a_queued_run_names_the_session_being_researched_and_its_place() {
    let (h, b, c) = a_line();
    assert_eq!(
        api(&h).status(c).unwrap().queue,
        Some(QueueView {
            position: 1,
            running_session_id: Some(h.session_id),
            running_name: Some("Chair · R4-1".into()),
            manual: true,
        })
    );
    let queue = api(&h).status(b).unwrap().queue.unwrap();
    assert_eq!((queue.position, queue.manual), (2, false));
    // Research / Run again on a waiting arranged run moves it to the front (MC-A6).
    api(&h).run(b).unwrap();
    let queue = api(&h).status(b).unwrap().queue.unwrap();
    assert_eq!(queue.position, 1);
    assert!(queue.manual);
    // A running or finished run has no place in the line.
    assert_eq!(api(&h).status(h.session_id).unwrap().queue, None);
}

#[test]
fn a6_3b_suggestions_left_show_to_review_and_none_left_show_done() {
    let h = Harness::chair();
    h.insert_run(RunState::Finished, None, None);
    h.suggest(DESIGNER, json!("Hans J. Wegner"));
    assert_eq!(
        api(&h).overview(None).unwrap(),
        vec![row(h.session_id, "to_review", None, 1)]
    );
    let mut designer = h.row(DESIGNER).unwrap();
    designer.ai_status = Some(AiStatus::Accepted);
    h.put(&designer);
    assert_eq!(
        api(&h).overview(None).unwrap(),
        vec![row(h.session_id, "done", None, 0)]
    );
    // A published session is no longer listed.
    h.query(|conn| {
        conn.execute(
            "UPDATE sessions SET published_at=1 WHERE id=?1",
            [h.session_id],
        )
        .unwrap()
    });
    assert_eq!(api(&h).overview(None).unwrap(), vec![]);
}

#[test]
fn a6_3c_an_arranged_session_that_cannot_be_researched_says_why() {
    let h = Harness::chair();
    h.query(|conn| {
        conn.execute(
            "UPDATE sessions SET arranged_at=1 WHERE id=?1",
            [h.session_id],
        )
        .unwrap()
    });
    // Ready to research but not queued (nothing to show).
    assert_eq!(api(&h).overview(None).unwrap(), vec![]);
    h.patch_settings(json!({"fill_keys": []}));
    let rows = api(&h).overview(None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].state, "blocked");
    let reason = rows[0].reason.clone().unwrap();
    assert_eq!(
        (reason.code.as_str(), reason.sentence.as_str()),
        (
            "NoFieldChosen",
            "Choose the fields research fills in Settings → Research"
        )
    );
}
