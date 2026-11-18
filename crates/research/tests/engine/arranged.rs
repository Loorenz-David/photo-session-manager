//! R11 U1: arranging is the only automatic research request.

use crate::harness::Harness;
use psm_core::domain::{
    attribute::{AttributeGroup, Preset},
    photo::Classification,
};
use psm_core::ports::secrets::SecretStore;
use psm_research::{
    config::ResearchSettings,
    domain::{RunState, RunTrigger},
};
use serde_json::json;

fn ready() -> Harness {
    let h = Harness::chair();
    h.add_main_photo_with_preview(h.session_id);
    h
}

fn state(h: &Harness) -> Vec<(RunState, RunTrigger)> {
    h.runs(h.session_id)
        .into_iter()
        .map(|r| (r.state, r.trigger))
        .collect()
}

#[test]
fn a25_c1a_first_press_queues_arranged() {
    let h = ready();
    assert!(h.service.arranged(h.session_id).unwrap());
    assert_eq!(state(&h), vec![(RunState::Queued, RunTrigger::Arranged)]);
}

#[test]
fn a25_c1b_second_press_keeps_one_queued() {
    let h = ready();
    assert!(h.service.arranged(h.session_id).unwrap());
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert_eq!(state(&h), vec![(RunState::Queued, RunTrigger::Arranged)]);
}

#[test]
fn a25_c1c_manual_queue_is_not_replaced() {
    let h = ready();
    h.service.run_again(h.session_id).unwrap();
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert_eq!(state(&h), vec![(RunState::Queued, RunTrigger::Manual)]);
}

#[test]
fn a25_c1d_finished_unchanged_is_not_queued() {
    let h = ready();
    h.service.arranged(h.session_id).unwrap();
    h.tick(0);
    assert_eq!(state(&h), vec![(RunState::Finished, RunTrigger::Arranged)]);
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert_eq!(h.runs(h.session_id).len(), 1);
}

#[test]
fn a25_c1e_cap_stopped_unchanged_is_not_queued() {
    let h = ready();
    h.service.arranged(h.session_id).unwrap();
    h.service.tick().unwrap();
    let id = h.runs(h.session_id)[0].id;
    h.query(|c| {
        c.execute(
            "UPDATE research_runs SET state='stopped_at_cap' WHERE id=?1",
            [id],
        )
        .unwrap()
    });
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert_eq!(h.runs(h.session_id).len(), 1);
}

#[test]
fn a25_c1f_failed_run_can_be_retried() {
    let h = ready();
    h.insert_run(RunState::Failed, None, None);
    assert!(h.service.arranged(h.session_id).unwrap());
    assert_eq!(
        state(&h).last(),
        Some(&(RunState::Queued, RunTrigger::Arranged))
    );
}

#[test]
fn a25_c1g_cancelled_run_can_be_retried() {
    let h = ready();
    h.insert_run(RunState::Cancelled, None, None);
    assert!(h.service.arranged(h.session_id).unwrap());
    assert_eq!(
        state(&h).last(),
        Some(&(RunState::Queued, RunTrigger::Arranged))
    );
}

/// A19 (owner 2026-11-18): research starts by itself at most once per session; a new photo
/// after the run no longer queues one. Run again does.
#[test]
fn a19_c1h_new_main_after_finished_queues_nothing_run_again_does() {
    let h = ready();
    h.service.arranged(h.session_id).unwrap();
    h.tick(0);
    h.add_main_photo_with_preview(h.session_id);
    h.recompose(h.add_main_photo_with_preview(h.session_id));
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert_eq!(state(&h), vec![(RunState::Finished, RunTrigger::Arranged)]);
    h.service.run_again(h.session_id).unwrap();
    assert_eq!(
        state(&h).last(),
        Some(&(RunState::Queued, RunTrigger::Manual))
    );
}

#[test]
fn a19_c1i_condition_change_during_run_queues_nothing() {
    let h = ready();
    h.service.arranged(h.session_id).unwrap();
    h.service.tick().unwrap();
    h.add_photo_with_preview(h.session_id, Classification::Condition);
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert_eq!(state(&h), vec![(RunState::Running, RunTrigger::Arranged)]);
}

#[test]
fn a25_c1j_running_unchanged_queues_nothing() {
    let h = ready();
    h.service.arranged(h.session_id).unwrap();
    h.service.tick().unwrap();
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert_eq!(h.runs(h.session_id).len(), 1);
}

#[test]
fn a25_c2a_without_main_arranging_queues_nothing() {
    let h = Harness::chair();
    h.add_photo_with_preview(h.session_id, Classification::Condition);
    h.add_photo_with_preview(h.session_id, Classification::Unreviewed);
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert!(h.runs(h.session_id).is_empty());
}

#[test]
fn a25_c2c_unassigned_is_refused() {
    let h = Harness::chair();
    let unassigned: i64 = h.query(|c| {
        c.query_row("SELECT id FROM sessions WHERE item_id IS NULL", [], |r| {
            r.get(0)
        })
        .unwrap()
    });
    assert_eq!(
        h.service.arranged(unassigned).unwrap_err().to_string(),
        "invalid data: the Unassigned session is never researched"
    );
    assert!(h.all_runs().is_empty());
}

#[test]
fn a25_c3a_photo_arrival_never_queues_even_with_old_settings() {
    let h = ready();
    h.patch_settings(json!({"auto": true, "quiet_seconds": 1}));
    h.service.session_changed(h.session_id);
    h.tick(600_000);
    assert!(h.runs(h.session_id).is_empty());
}

#[test]
fn a25_c4a_arranged_starts_without_quiet_period() {
    let h = ready();
    h.service.session_changed(h.session_id);
    h.service.arranged(h.session_id).unwrap();
    h.service.tick().unwrap();
    assert_eq!(h.runs(h.session_id)[0].state, RunState::Running);
}

#[test]
fn a25_c5a_status_has_no_reason_when_ready() {
    let h = ready();
    let status = h.service.status(h.session_id).unwrap();
    assert!(status.reason.is_none());
    assert!(status.can_run);
}

#[test]
fn a25_c7a_old_settings_keys_are_ignored() {
    let settings = ResearchSettings::from_slot(Some(
        &json!({"auto": true, "quiet_seconds": 5, "max_pages": 7}),
    ));
    assert_eq!(settings.max_pages, 7);
    let value = serde_json::to_value(settings).unwrap();
    assert!(value.get("auto").is_none());
    assert!(value.get("quiet_seconds").is_none());
}

#[test]
fn a25_c2b_creation_prerequisites_name_the_reason() {
    let h = ready();
    let without_shop = h.service_with(|deps| deps.shop = None);
    assert!(!without_shop.arranged(h.session_id).unwrap());
    assert_eq!(
        without_shop
            .status(h.session_id)
            .unwrap()
            .reason
            .unwrap()
            .code,
        "NoIntegration"
    );
    assert_eq!(
        without_shop
            .status(h.session_id)
            .unwrap()
            .reason
            .unwrap()
            .sentence,
        "Connect a shop to use research"
    );
    assert!(h.runs(h.session_id).is_empty());
    let h = ready();
    h.app.secrets.delete("research-model-key").unwrap();
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert_eq!(
        h.service.status(h.session_id).unwrap().reason.unwrap().code,
        "NoModelKey"
    );
    assert_eq!(
        h.service
            .status(h.session_id)
            .unwrap()
            .reason
            .unwrap()
            .sentence,
        "Add the model key in Settings → Research"
    );
    assert!(h.runs(h.session_id).is_empty());
    let h = ready();
    h.app.secrets.delete("research-search-key").unwrap();
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert_eq!(
        h.service.status(h.session_id).unwrap().reason.unwrap().code,
        "NoSearchKey"
    );
    assert_eq!(
        h.service
            .status(h.session_id)
            .unwrap()
            .reason
            .unwrap()
            .sentence,
        "Add the visual search key in Settings → Research"
    );
    assert!(h.runs(h.session_id).is_empty());
    let h = ready();
    h.patch_settings(json!({"prices": []}));
    assert!(!h.service.arranged(h.session_id).unwrap());
    assert_eq!(
        h.service.status(h.session_id).unwrap().reason.unwrap().code,
        "PriceMissing"
    );
    assert_eq!(
        h.service
            .status(h.session_id)
            .unwrap()
            .reason
            .unwrap()
            .sentence,
        "No price is set for fake-model m1"
    );
    assert!(h.runs(h.session_id).is_empty());
    let h = ready();
    let (item, session) = h.new_session("R11-EMPTY");
    let _ = item;
    let attrs = h.app.attributes(Some(crate::harness::INTEGRATION));
    for group in [AttributeGroup::Details, AttributeGroup::Condition] {
        attrs
            .presets_set(Preset {
                item_type: "Sideboard".into(),
                group,
                layout: vec![],
            })
            .unwrap();
    }
    h.add_main_photo_with_preview(session);
    assert!(!h.service.arranged(session).unwrap());
    assert_eq!(
        h.service.status(session).unwrap().reason.unwrap().code,
        "NothingToResearch"
    );
    assert_eq!(
        h.service.status(session).unwrap().reason.unwrap().sentence,
        "This item type has no fields research can fill"
    );
    assert!(h.runs(session).is_empty());
}

#[test]
fn a25_c3b_changes_after_finished_do_not_queue() {
    let h = ready();
    h.service.arranged(h.session_id).unwrap();
    h.tick(0);
    let first = h.runs(h.session_id).len();
    let second = h.add_main_photo_with_preview(h.session_id);
    h.recompose(second);
    h.classify(second, Classification::Condition);
    h.service.session_changed(h.session_id);
    h.tick(600_000);
    assert_eq!(h.runs(h.session_id).len(), first);
}

#[test]
fn a25_c4b_missing_source_wait_uses_last_change() {
    let h = ready();
    h.add_photo_without_preview(h.session_id, Classification::Main);
    h.service.session_changed(h.session_id);
    h.service.arranged(h.session_id).unwrap();
    h.app.clock.advance(119_000);
    assert_eq!(h.service.tick().unwrap(), None);
    assert_eq!(h.runs(h.session_id)[0].state, RunState::Queued);
    h.app.clock.advance(1_000);
    assert!(h.service.tick().unwrap().is_some());
    let snapshot: serde_json::Value =
        serde_json::from_str(h.runs(h.session_id)[0].input_snapshot.as_deref().unwrap()).unwrap();
    assert_eq!(snapshot["observe"].as_array().unwrap().len(), 1);
}

#[test]
fn a25_c4c_no_source_waits_for_editor() {
    let h = Harness::chair();
    h.add_photo_without_preview(h.session_id, Classification::Main);
    h.service.session_changed(h.session_id);
    h.service.arranged(h.session_id).unwrap();
    let change = h.now();
    h.app.clock.advance(120_000);
    assert_eq!(h.service.tick().unwrap(), None);
    let status = h.service.status(h.session_id).unwrap();
    assert_eq!(status.reason.unwrap().sentence, "Waiting for Capture One");
    assert_eq!(status.wait_ends_at, Some(change + 120_000));
}

#[test]
fn a25_c4d_manual_starts_before_two_arranged_in_order() {
    let h = ready();
    let (_, b) = h.new_session("R11-B");
    let (_, c) = h.new_session("R11-C");
    h.service.arranged(h.session_id).unwrap();
    h.add_main_photo_with_preview(b);
    h.service.arranged(b).unwrap();
    h.add_main_photo_with_preview(c);
    h.service.run_again(c).unwrap();
    let ids = [h.runs(c)[0].id, h.runs(h.session_id)[0].id, h.runs(b)[0].id];
    for id in ids {
        assert_eq!(h.service.tick().unwrap(), Some(id));
        h.run_jobs();
    }
    assert_eq!(
        h.all_runs()
            .into_iter()
            .filter(|r| r.state == RunState::Running)
            .count(),
        0
    );
}

#[test]
fn a25_c6a_legacy_queued_runs_cancel_on_service_build() {
    let h = ready();
    let auto = h.insert_run(RunState::Queued, None, None);
    let (bi, bs) = h.new_session("R11-LEGACY-B");
    let (ci, cs) = h.new_session("R11-LEGACY-C");
    let (di, ds) = h.new_session("R11-LEGACY-D");
    let follow = h.insert_run_for(bi, bs, RunState::Queued, None, None);
    let manual = h.insert_run_for(ci, cs, RunState::Queued, None, None);
    let running = h.insert_run_for(di, ds, RunState::Running, None, None);
    h.query(|c| {
        c.execute(
            "UPDATE research_runs SET trigger='follow_up' WHERE id=?1",
            [follow],
        )
        .unwrap();
        c.execute(
            "UPDATE research_runs SET trigger='manual' WHERE id=?1",
            [manual],
        )
        .unwrap();
    });
    let _new = h.service_with(|_| {});
    assert_eq!(h.run_row(auto).state, RunState::Cancelled);
    assert_eq!(h.run_row(follow).state, RunState::Cancelled);
    assert_eq!(h.run_row(manual).state, RunState::Queued);
    assert_eq!(h.run_row(running).state, RunState::Running);
}
