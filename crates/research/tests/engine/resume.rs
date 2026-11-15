//! R4·U5: kill-and-resume, reuse and follow-up runs. A kill is `Harness::restart()` from a fake's
//! `before_call` hook or from the service's event sink between two commits; "equals the
//! uninterrupted run" compares against a reference run of the same script on a second harness.

use psm_core::domain::photo::Classification;
use psm_research::{
    domain::{RunState, RunTrigger, Stage},
    fakes::PageScript,
    ports::SearchCandidate,
    rules::sites::page_identity,
    stages::OBSERVE_UNIT,
};
use psm_testkit::fixtures::synthetic_jpeg::SyntheticJpeg;
use serde_json::json;

use crate::harness::{DESIGNER, Harness, SCRATCHES, WOODS, kill};

const P_E1: &str = "https://www.e1.example/p/1";
const P_E2: &str = "https://e2.example/item/2";
const P_E1B: &str = "https://www.e1.example/p/7";
const P_E2B: &str = "https://e2.example/item/8";

fn u(url: &str) -> String {
    page_identity(url).unwrap()
}

fn results(h: &Harness, urls: &[&str]) {
    h.search.set_results(
        urls.iter()
            .enumerate()
            .map(|(i, url)| SearchCandidate {
                url: (*url).to_owned(),
                rank: u32::try_from(i + 1).unwrap(),
                title: String::new(),
            })
            .collect(),
    );
}

/// Pages that state designer A, with an image each, compared `match`.
fn pages(h: &Harness, urls: &[&str]) {
    for (n, url) in urls.iter().enumerate() {
        let n = u8::try_from(n).unwrap();
        h.reader.set_page(
            url,
            PageScript::page(
                &format!("Listing {url}"),
                vec![SyntheticJpeg::rgb(24, 24, [n * 30, 90, 200 - n])],
            ),
        );
        h.model.reply(
            Stage::Read,
            &u(url),
            json!({"keys": {DESIGNER: {"stated": "Designer A", "value": "Designer A"}}}),
        );
        h.model
            .reply(Stage::Compare, &u(url), json!({"verdict": "match"}));
    }
}

/// The run's script: two main photos and one condition photo, two matching pages naming
/// designer A, Teak read by the one Observe call, Minor on the condition photo.
struct Script {
    mains: Vec<i64>,
    condition: i64,
}

fn script(h: &Harness) -> Script {
    let mains = vec![
        h.add_main_photo_with_preview(h.session_id),
        h.add_main_photo_with_preview(h.session_id),
    ];
    let condition = h.add_photo_with_preview(h.session_id, Classification::Condition);
    results(h, &[P_E1, P_E2]);
    pages(h, &[P_E1, P_E2]);
    h.model.reply(
        Stage::Observe,
        OBSERVE_UNIT,
        json!({"reads": {WOODS: {"value": ["Teak"], "sure": true}}, "marking_photos": []}),
    );
    h.model.reply(
        Stage::Condition,
        "condition",
        json!({"reads": {SCRATCHES: {"value": "Minor", "sure": true}}}),
    );
    h.model.reply(
        Stage::Decide,
        "decide",
        json!({"reasons": {DESIGNER: "Two trusted dealers name Designer A."}}),
    );
    h.changed();
    Script { mains, condition }
}

/// The uninterrupted run of the script.
fn reference() -> (Harness, i64) {
    let h = Harness::chair();
    script(&h);
    h.tick(1_000);
    let run = h.runs(h.session_id)[0].id;
    assert_eq!(h.run_row(run).state, RunState::Finished);
    (h, run)
}

fn resumed(h: &mut Harness) -> i64 {
    h.tick(0);
    let run = h.runs(h.session_id)[0].id;
    assert_eq!(h.run_row(run).state, RunState::Finished);
    run
}

#[test]
fn a14_c1b_kill_with_a_read_call_in_flight() {
    let (reference, reference_run) = reference();
    let mut h = Harness::chair();
    script(&h);
    let mut fired = false;
    h.reader.before_call(move |_| {
        if !fired {
            fired = true;
            kill();
        }
    });
    h.tick_killed(1_000);
    let run = h.runs(h.session_id)[0].id;
    let open = h.calls(run);
    assert_eq!(open.last().unwrap().state, "sent");
    let run = resumed(&mut h);
    let reads: Vec<(String, i64, i64)> = h
        .calls(run)
        .into_iter()
        .filter(|c| c.provider == "fake-reader" && c.unit_key == u(P_E1))
        .map(|c| (c.state, c.attempt, c.cost_micro_usd))
        .collect();
    assert_eq!(
        reads,
        vec![
            ("unknown".to_owned(), 1, 1_000),
            ("answered".to_owned(), 2, 1_000)
        ]
    );
    assert_eq!(h.end_state(run), reference.end_state(reference_run));
    assert_eq!(
        h.run_row(run).cost_micro_usd - reference.run_row(reference_run).cost_micro_usd,
        1_000
    );
}

fn has_output(store: &psm_infra::sqlite::store::SqliteStore, stage: &str) -> bool {
    store
        .research_transaction(|tx| {
            let mut stmt = tx
                .conn()
                .prepare("SELECT COUNT(*) FROM research_stage_outputs WHERE stage=?1")
                .unwrap();
            Ok(stmt.query_row([stage], |r| r.get::<_, i64>(0)).unwrap() > 0)
        })
        .unwrap()
}

#[test]
fn a14_c1c_kill_between_find_and_the_first_read() {
    let (reference, reference_run) = reference();
    let mut h = Harness::chair();
    script(&h);
    let store = h.store().clone();
    h.kill_when(move || has_output(&store, "find") && !has_output(&store, "read"));
    h.tick_killed(1_000);
    let run = resumed(&mut h);
    assert_eq!(h.end_state(run), reference.end_state(reference_run));
    assert_eq!(h.search.requests().len(), 1);
}

#[test]
fn a14_c1d_kill_before_decide() {
    let (reference, reference_run) = reference();
    let mut h = Harness::chair();
    script(&h);
    let store = h.store().clone();
    h.kill_when(move || has_output(&store, "condition") && !has_output(&store, "decide"));
    h.tick_killed(1_000);
    let run = h.runs(h.session_id)[0].id;
    assert!(h.outputs(run).iter().all(|o| o.stage != "decide"));
    let run = resumed(&mut h);
    assert_eq!(h.end_state(run), reference.end_state(reference_run));
}

#[test]
fn a14_c1e_kill_inside_decide() {
    let (reference, reference_run) = reference();
    let mut h = Harness::chair();
    script(&h);
    h.model.before_call(|call| {
        if call.stage == Some(Stage::Decide) {
            kill();
        }
    });
    h.tick_killed(1_000);
    let run = resumed(&mut h);
    assert_eq!(h.end_state(run), reference.end_state(reference_run));
    for key in [DESIGNER, WOODS, SCRATCHES] {
        let concluded = h
            .steps(run)
            .iter()
            .filter(|s| s.kind == "concluded" && s.field_keys == vec![key.to_owned()])
            .count();
        assert_eq!(concluded, 1, "{key}");
    }
}

#[test]
fn a14_c1f_kill_after_the_stage_7_commit() {
    let mut h = Harness::chair();
    script(&h);
    let store = h.store().clone();
    h.kill_when(move || has_output(&store, "decide"));
    h.tick_killed(1_000);
    let run = h.runs(h.session_id)[0].id;
    assert_eq!(h.run_row(run).state, RunState::Running);
    let before = h.end_state(run);
    let updated: Vec<i64> = h.query(|c| {
        let mut stmt = c
            .prepare("SELECT updated_at FROM attribute_values ORDER BY key")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    });
    h.tick(5_000);
    assert_eq!(h.run_row(run).state, RunState::Finished);
    assert_eq!(h.end_state(run), before);
    let after: Vec<i64> = h.query(|c| {
        let mut stmt = c
            .prepare("SELECT updated_at FROM attribute_values ORDER BY key")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    });
    assert_eq!(after, updated);
}

#[test]
fn a14_c2a_repeat_of_a_committed_stage_7_changes_nothing() {
    let (h, run) = reference();
    let before = h.end_state(run);
    let updated: Vec<(String, i64)> = h.query(|c| {
        let mut stmt = c
            .prepare("SELECT key,updated_at FROM attribute_values ORDER BY key")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    });
    // A resume that finds stage 7 committed: the run is `running` again and executes.
    h.app.clock.advance(10_000);
    h.query(|c| {
        c.execute(
            "UPDATE research_runs SET state='running' WHERE id=?1",
            [run],
        )
        .unwrap()
    });
    h.execute(run);
    assert_eq!(h.run_row(run).state, RunState::Finished);
    assert_eq!(h.end_state(run), before);
    let after: Vec<(String, i64)> = h.query(|c| {
        let mut stmt = c
            .prepare("SELECT key,updated_at FROM attribute_values ORDER BY key")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    });
    assert_eq!(after, updated);
}

/// A finished first run of the script; the fakes' logs cleared.
fn finished() -> (Harness, Script, i64) {
    let h = Harness::chair();
    let s = script(&h);
    h.tick(1_000);
    let run = h.runs(h.session_id)[0].id;
    assert_eq!(h.run_row(run).state, RunState::Finished);
    h.model.clear_requests();
    h.search.clear_requests();
    h.reader.clear_requests();
    (h, s, run)
}

/// The follow-up queued by the change, run to its end.
fn follow_up(h: &Harness, first: i64) -> i64 {
    let runs = h.runs(h.session_id);
    let follow = runs
        .iter()
        .find(|r| r.id > first && r.state == RunState::Queued)
        .expect("a follow-up");
    assert_eq!(follow.trigger, RunTrigger::Arranged);
    h.tick(1_000);
    assert_eq!(h.run_row(follow.id).state, RunState::Finished);
    follow.id
}

/// The model calls of the follow-up, as (stage, unit key).
fn model_calls(h: &Harness) -> Vec<(Stage, String)> {
    h.model
        .requests()
        .into_iter()
        .map(|c| (c.stage.unwrap(), c.unit_key))
        .collect()
}

#[test]
fn a14_c3a_follow_up_for_a_new_main_photo() {
    let (h, s, first) = finished();
    let added = h.add_main_photo_with_preview(h.session_id);
    h.changed();
    let run = follow_up(&h, first);
    // R12·U1: the new photo costs one Observe call over all chosen photos.
    assert_eq!(
        model_calls(&h),
        vec![
            (Stage::Observe, OBSERVE_UNIT.to_owned()),
            (Stage::Decide, "decide".to_owned())
        ]
    );
    let observe = &h.model.requests()[0];
    assert_eq!(observe.images.len(), 3);
    assert!(h.search.requests().is_empty() && h.reader.requests().is_empty());
    for output in h.outputs(run) {
        let fresh = output.stage == "observe" || output.stage == "decide";
        assert_eq!(
            output.reused_from_run_id.is_none(),
            fresh,
            "{}",
            output.unit_key
        );
    }
    let looked: Vec<Vec<i64>> = h
        .steps(run)
        .into_iter()
        .filter(|s| s.kind == "looked")
        .map(|s| s.photo_ids)
        .collect();
    assert_eq!(looked, vec![vec![s.mains[0], s.mains[1], added]]);
}

#[test]
fn a14_c3b_follow_up_for_a_new_condition_photo() {
    let (h, _, first) = finished();
    h.add_photo_with_preview(h.session_id, Classification::Condition);
    h.changed();
    follow_up(&h, first);
    let stages: Vec<Stage> = model_calls(&h).into_iter().map(|(s, _)| s).collect();
    assert_eq!(stages, vec![Stage::Condition, Stage::Decide]);
    assert!(h.search.requests().is_empty() && h.reader.requests().is_empty());
}

#[test]
fn a14_c3c_follow_up_for_a_new_cover() {
    let (h, s, first) = finished();
    // The new cover finds other pages.
    results(&h, &[P_E1B, P_E2B]);
    pages(&h, &[P_E1B, P_E2B]);
    h.app.review().reorder(s.mains[1], 1).unwrap();
    h.changed();
    let run = follow_up(&h, first);
    assert_eq!(h.search.requests().len(), 1);
    assert_eq!(h.reader.requests().len(), 2);
    // R12·U1: the new cover is the wide photo, so the Observe identity (photos in order with their
    // roles) changed too: one Observe call over both photos.
    let stages: Vec<Stage> = model_calls(&h).into_iter().map(|(s, _)| s).collect();
    assert_eq!(
        stages,
        vec![
            Stage::Observe,
            Stage::Read,
            Stage::Read,
            Stage::Compare,
            Stage::Compare,
            Stage::Decide
        ]
    );
    let observe_fresh = h
        .outputs(run)
        .into_iter()
        .filter(|o| o.stage == "observe")
        .all(|o| o.reused_from_run_id.is_none());
    assert!(observe_fresh);
}

#[test]
fn a14_c3d_follow_up_for_a_recomposed_photo() {
    let (h, s, first) = finished();
    h.recompose(s.mains[1]);
    h.changed();
    follow_up(&h, first);
    // R12·U1: one Observe call over all chosen photos.
    assert_eq!(
        model_calls(&h),
        vec![
            (Stage::Observe, OBSERVE_UNIT.to_owned()),
            (Stage::Decide, "decide".to_owned())
        ]
    );
    assert_eq!(h.model.requests()[0].images.len(), 2);
    assert!(h.search.requests().is_empty() && h.reader.requests().is_empty());
    // The cover re-framed (its search finding other pages): find, read and compare too.
    let second = h.runs(h.session_id).last().unwrap().id;
    h.model.clear_requests();
    results(&h, &[P_E1B, P_E2B]);
    pages(&h, &[P_E1B, P_E2B]);
    h.recompose(s.mains[0]);
    h.changed();
    follow_up(&h, second);
    assert_eq!(h.search.requests().len(), 1);
    let stages: Vec<Stage> = model_calls(&h).into_iter().map(|(s, _)| s).collect();
    assert_eq!(
        stages,
        vec![
            Stage::Observe,
            Stage::Read,
            Stage::Read,
            Stage::Compare,
            Stage::Compare,
            Stage::Decide
        ]
    );
}

#[test]
fn a14_c3e_a_unit_whose_file_is_gone_runs_again() {
    let (h, _, first) = finished();
    let read = h
        .outputs(first)
        .into_iter()
        .find(|o| o.stage == "read" && o.unit_key == u(P_E1))
        .unwrap();
    let output: serde_json::Value = serde_json::from_str(&read.output).unwrap();
    let sha = output["images"][0].as_str().unwrap().to_owned();
    std::fs::remove_file(h.files().path(&sha)).unwrap();
    h.add_main_photo_with_preview(h.session_id);
    h.changed();
    let run = follow_up(&h, first);
    assert_eq!(
        h.reader
            .requests()
            .into_iter()
            .map(|c| c.url.unwrap())
            .collect::<Vec<_>>(),
        vec![P_E1.to_owned()]
    );
    let again = h
        .outputs(run)
        .into_iter()
        .find(|o| o.stage == "read" && o.unit_key == u(P_E1))
        .unwrap();
    assert_eq!(again.reused_from_run_id, None);
    assert!(h.files().exists(&sha));
}

#[test]
fn a14_c4a_a_manual_run_reuses_nothing() {
    let (h, s, first) = finished();
    h.service.run_again(h.session_id).unwrap();
    h.tick(0);
    let run = h.runs(h.session_id).last().unwrap().clone();
    assert_ne!(run.id, first);
    assert_eq!(
        (run.trigger, h.run_row(run.id).state),
        (RunTrigger::Manual, RunState::Finished)
    );
    assert!(
        h.outputs(run.id)
            .iter()
            .all(|o| o.reused_from_run_id.is_none())
    );
    let calls = model_calls(&h);
    assert!(calls.contains(&(Stage::Observe, OBSERVE_UNIT.to_owned())));
    assert_eq!(
        h.model
            .requests()
            .iter()
            .find(|c| c.stage == Some(Stage::Observe))
            .unwrap()
            .images
            .len(),
        s.mains.len()
    );
    assert_eq!(h.search.requests().len(), 1);
    assert_eq!(h.reader.requests().len(), 2);
    assert!(calls.contains(&(Stage::Condition, "condition".to_owned())));
    assert!(calls.contains(&(Stage::Decide, "decide".to_owned())));
    let _ = s.condition;
}

/// After a finished run of: two main photos, one condition photo and one deleted photo.
fn trigger_base() -> (Harness, Script, i64, i64) {
    let h = Harness::chair();
    let s = script(&h);
    let deleted = h.add_photo_with_preview(h.session_id, Classification::Main);
    h.classify(deleted, Classification::Deleted);
    h.changed();
    h.tick(1_000);
    let first = h.runs(h.session_id)[0].id;
    assert_eq!(h.run_row(first).state, RunState::Finished);
    (h, s, first, deleted)
}

fn queued_follow_ups(h: &Harness) -> usize {
    h.runs(h.session_id)
        .iter()
        .filter(|r| r.state == RunState::Queued && r.trigger == RunTrigger::Arranged)
        .count()
}

/// One photo change after the first run (the deleted photo's id is the last argument).
type Change = fn(&Harness, &Script, i64);

#[test]
fn a14_c5a_each_photo_derived_change_queues_one_follow_up() {
    let changes: Vec<(&str, Change)> = vec![
        ("main photo added", |h, _, _| {
            h.add_main_photo_with_preview(h.session_id);
        }),
        ("main photo deleted", |h, s, _| {
            h.classify(s.mains[1], Classification::Deleted)
        }),
        ("deleted photo restored", |h, _, deleted| {
            h.classify(deleted, Classification::Main)
        }),
        ("photo re-framed", |h, s, _| h.recompose(s.mains[1])),
        ("main to condition", |h, s, _| {
            h.classify(s.mains[1], Classification::Condition)
        }),
        ("condition to main", |h, s, _| {
            h.classify(s.condition, Classification::Main)
        }),
        ("new cover by reorder", |h, s, _| {
            h.app.review().reorder(s.mains[1], 1).unwrap();
        }),
    ];
    for (name, change) in changes {
        let (h, s, _, deleted) = trigger_base();
        change(&h, &s, deleted);
        h.changed();
        assert_eq!(queued_follow_ups(&h), 1, "{name}");
        assert_eq!(h.runs(h.session_id).len(), 2, "{name}");
    }
}

#[test]
fn a14_c5b_retouch_settings_and_non_cover_swap_queue_nothing() {
    // A Capture One retouch: a new adjusted generation, same photo identity.
    let (h, s, _, _) = trigger_base();
    h.give_preview(s.mains[1]);
    h.changed();
    assert_eq!(h.runs(h.session_id).len(), 1, "retouch");
    // A stage's model changed in settings.
    let (h, _, _, _) = trigger_base();
    h.patch_settings(
        json!({"stages": {"observe": {"model": "m2", "effort": "low", "image_long_edge": 1024}}}),
    );
    h.service.settings_changed();
    h.changed();
    assert_eq!(h.runs(h.session_id).len(), 1, "settings");
    // Two non-cover main photos swapped.
    let (h, s, _, _) = trigger_base();
    let third = h.add_main_photo_with_preview(h.session_id);
    let runs_before = {
        h.changed();
        h.tick(1_000);
        h.runs(h.session_id).len()
    };
    h.app.review().reorder(third, 2).unwrap();
    assert_eq!(h.photo(s.mains[0]).position, Some(1));
    h.changed();
    assert_eq!(h.runs(h.session_id).len(), runs_before, "swap");
}

#[test]
fn a14_c6a_changes_during_a_run_queue_one_follow_up() {
    let h = Harness::chair();
    script(&h);
    h.app.clock.advance(1_000);
    h.service.tick().unwrap();
    let first = h.runs(h.session_id)[0].id;
    assert_eq!(h.run_row(first).state, RunState::Running);
    for _ in 0..3 {
        h.add_main_photo_with_preview(h.session_id);
        h.changed();
    }
    let last_change = h.now();
    assert_eq!(queued_follow_ups(&h), 1);
    h.run_jobs();
    assert_eq!(h.run_row(first).state, RunState::Finished);
    let follow = h
        .runs(h.session_id)
        .into_iter()
        .find(|r| r.trigger == RunTrigger::Arranged)
        .unwrap()
        .id;
    h.tick(0);
    let row = h.run_row(follow);
    assert_eq!(row.started_at, Some(last_change));
    assert!(row.started_at.unwrap() >= h.run_row(first).finished_at.unwrap());
    assert_eq!(queued_follow_ups(&h), 0);
}
