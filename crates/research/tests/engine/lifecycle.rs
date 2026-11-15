//! R4·U4: the life of a run. Photo changes go through the core services and are followed by
//! `ResearchService::session_changed` (the call the app makes from R7·U1 on); settings changes by
//! `settings_changed`; time by `Harness::tick` on the manual clock.

use std::sync::{Arc, Mutex};

use chrono::{Local, TimeZone};
use psm_core::{
    domain::attribute::{AttributeGroup, Preset, PresetField},
    ports::secrets::SecretStore,
};
use psm_research::{
    config::ResearchConfig,
    domain::{RunState, RunTrigger},
    fakes,
    service::{FormSnapshot, MODEL_ACCOUNT, SEARCH_ACCOUNT},
    store as rstore,
};
use serde_json::json;

use crate::harness::{DESIGNER, E1, E2, Harness, INTEGRATION, LOCKED, WOODS};

fn main_photo(h: &Harness) -> i64 {
    let id = h.add_main_photo_with_preview(h.session_id);
    h.changed();
    id
}

fn reason(h: &Harness) -> Option<(String, String)> {
    h.service
        .status(h.session_id)
        .unwrap()
        .reason
        .map(|r| (r.code, r.sentence))
}

fn reason_of(
    svc: &psm_research::service::ResearchService,
    session: i64,
) -> Option<(String, String)> {
    svc.status(session)
        .unwrap()
        .reason
        .map(|r| (r.code, r.sentence))
}

fn pair(code: &str, sentence: &str) -> Option<(String, String)> {
    Some((code.to_owned(), sentence.to_owned()))
}

/// Makes the gateway see a call that could not leave: a run starts and its first call is offline.
fn go_offline(h: &Harness) {
    h.model.fault(fakes::offline());
    main_photo(h);
    h.tick(1_000);
}

/// A session of an item type whose layout has no field research may write.
fn bare_session(h: &Harness) -> i64 {
    let attributes = h.app.attributes(Some(INTEGRATION));
    for (group, layout) in [
        (
            AttributeGroup::Details,
            vec![PresetField {
                key: LOCKED.into(),
                default: None,
            }],
        ),
        (AttributeGroup::Condition, vec![]),
    ] {
        attributes
            .presets_set(Preset {
                item_type: "Bare".into(),
                group,
                layout,
            })
            .unwrap();
    }
    let (_, session) = h
        .app
        .items()
        .create_local("R4-BARE", "Bare piece", "Bare", None)
        .unwrap();
    session.session.id
}

#[test]
fn a13_c1a_each_prerequisite_alone() {
    // 1. No integration.
    let h = Harness::chair();
    let svc = h.service_with(|deps| deps.shop = None);
    h.add_main_photo_with_preview(h.session_id);
    svc.session_changed(h.session_id);
    assert_eq!(
        reason_of(&svc, h.session_id),
        pair("NoIntegration", "Connect a shop to use research")
    );
    assert!(h.runs(h.session_id).is_empty());
    // 2. No model key.
    let h = Harness::chair();
    h.app.secrets.delete(MODEL_ACCOUNT).unwrap();
    main_photo(&h);
    assert_eq!(
        reason(&h),
        pair("NoModelKey", "Add the model key in Settings → Research")
    );
    assert!(h.runs(h.session_id).is_empty());
    // 3. No visual-search key.
    let h = Harness::chair();
    h.app.secrets.delete(SEARCH_ACCOUNT).unwrap();
    main_photo(&h);
    assert_eq!(
        reason(&h),
        pair(
            "NoSearchKey",
            "Add the visual search key in Settings → Research"
        )
    );
    assert!(h.runs(h.session_id).is_empty());
    // 4. A provider and model missing from the price table.
    let h = Harness::chair();
    h.patch_settings(json!({"prices": [
        {"provider": "fake-model", "model": "m1", "unit": "input_token", "usd_per_million": "1"},
        {"provider": "fake-search", "model": "", "fallback_call_usd": "0.001"},
        {"provider": "fake-reader", "model": "", "fallback_call_usd": "0.001"},
    ]}));
    main_photo(&h);
    assert_eq!(
        reason(&h),
        pair("PriceMissing", "No price is set for fake-model m1")
    );
    assert!(h.runs(h.session_id).is_empty());
    // 5. Nothing in the layout research may write.
    let h = Harness::chair();
    let bare = bare_session(&h);
    h.add_main_photo_with_preview(bare);
    h.service.session_changed(bare);
    assert_eq!(
        reason_of(&h.service, bare),
        pair(
            "NothingToResearch",
            "This item type has no fields research can fill"
        )
    );
    assert!(h.runs(bare).is_empty());
    // 6. Day limit reached.
    let h = Harness::chair();
    h.patch_settings(json!({"max_cost_per_day": "0"}));
    main_photo(&h);
    assert_eq!(reason(&h), pair("DayLimit", "Day limit reached"));
    // 7. Offline.
    let h = Harness::chair();
    go_offline(&h);
    assert_eq!(reason(&h), pair("Offline", "Offline"));
}

#[test]
fn a13_c1b_the_first_failing_is_named() {
    let h = Harness::chair();
    h.app.secrets.delete(MODEL_ACCOUNT).unwrap();
    h.patch_settings(json!({"max_cost_per_day": "0"}));
    main_photo(&h);
    assert_eq!(reason(&h).unwrap().0, "NoModelKey");
    let h = Harness::chair();
    go_offline(&h);
    assert_eq!(reason(&h).unwrap().0, "Offline");
}

#[test]
fn a13_c2c_unassigned_is_never_researched() {
    let h = Harness::chair();
    let unassigned: i64 = h.query(|c| {
        c.query_row("SELECT id FROM sessions WHERE item_id IS NULL", [], |r| {
            r.get(0)
        })
        .unwrap()
    });
    h.add_main_photo_with_preview(unassigned);
    h.add_main_photo_with_preview(unassigned);
    h.service.session_changed(unassigned);
    h.tick(10_000);
    assert!(h.all_runs().is_empty());
}

#[test]
fn a13_c3a_run_again_queues_a_manual_run() {
    let h = Harness::chair();
    h.service.run_again(h.session_id).unwrap();
    let runs = h.runs(h.session_id);
    assert_eq!(runs.len(), 1);
    assert_eq!(
        (runs[0].state, runs[0].trigger),
        (RunState::Queued, RunTrigger::Manual)
    );
}

#[test]
fn a13_c3b_run_again_makes_the_queued_run_manual() {
    let h = Harness::chair();
    main_photo(&h);
    let queued = h.runs(h.session_id)[0].id;
    h.service.run_again(h.session_id).unwrap();
    let runs = h.runs(h.session_id);
    assert_eq!(runs.len(), 1);
    assert_eq!((runs[0].id, runs[0].trigger), (queued, RunTrigger::Manual));
}

#[test]
fn a13_c3c_run_again_beside_a_running_run() {
    let h = Harness::chair();
    main_photo(&h);
    h.app.clock.advance(1_000);
    h.service.tick().unwrap();
    assert_eq!(h.runs(h.session_id)[0].state, RunState::Running);
    h.service.run_again(h.session_id).unwrap();
    h.service.run_again(h.session_id).unwrap();
    let queued: Vec<_> = h
        .runs(h.session_id)
        .into_iter()
        .filter(|r| r.state == RunState::Queued)
        .collect();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].trigger, RunTrigger::Manual);
    assert_eq!(h.runs(h.session_id).len(), 2);
}

#[test]
fn a13_c7a_one_run_at_a_time() {
    let h = Harness::chair();
    let (_, other) = h.new_session("R4-B");
    main_photo(&h);
    h.add_main_photo_with_preview(other);
    h.service.session_changed(other);
    h.service.arranged(other).unwrap();
    let seen: Arc<Mutex<Vec<usize>>> = Arc::default();
    let (store, out) = (h.store().clone(), seen.clone());
    h.model.before_call(move |_| {
        let running = store
            .research_transaction(|tx| rstore::runs_in_state(tx.conn(), RunState::Running))
            .unwrap()
            .len();
        out.lock().unwrap().push(running);
    });
    h.tick(1_000);
    let first = h.runs(h.session_id)[0].clone();
    let second = h.runs(other)[0].clone();
    assert_eq!(h.run_row(first.id).state, RunState::Finished);
    assert_eq!(h.run_row(second.id).state, RunState::Queued);
    h.tick(0);
    let second = h.run_row(second.id);
    assert_eq!(second.state, RunState::Finished);
    assert!(second.started_at.unwrap() >= h.run_row(first.id).finished_at.unwrap());
    let seen = seen.lock().unwrap();
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|n| *n == 1), "{seen:?}");
}

#[test]
fn a13_c7b_manual_starts_before_an_earlier_arranged() {
    let h = Harness::chair();
    let (_, other) = h.new_session("R4-B");
    main_photo(&h);
    let auto = h.runs(h.session_id)[0].id;
    h.app.clock.advance(1_000);
    h.add_main_photo_with_preview(other);
    h.service.session_changed(other);
    h.service.arranged(other).unwrap();
    h.service.run_again(other).unwrap();
    let manual = h.runs(other)[0].id;
    assert_eq!(h.run_row(manual).trigger, RunTrigger::Manual);
    assert!(h.run_row(manual).queued_at > h.run_row(auto).queued_at);
    h.app.clock.advance(1_000);
    h.service.tick().unwrap();
    assert_eq!(h.run_row(manual).state, RunState::Running);
    assert_eq!(h.run_row(auto).state, RunState::Queued);
}

#[test]
fn a13_c7c_the_start_transaction_snapshots_settings_and_form() {
    let h = Harness::chair();
    h.suggest(WOODS, json!(["Oak"]));
    main_photo(&h);
    h.app.clock.advance(1_000);
    h.service.tick().unwrap();
    h.patch_settings(json!({"trusted_sites": [E1]}));
    h.suggest(WOODS, json!(["Teak"]));
    let run = h.run_row(h.runs(h.session_id)[0].id);
    let config: ResearchConfig =
        serde_json::from_str(run.settings_snapshot.as_deref().unwrap()).unwrap();
    assert_eq!(config.trusted_sites, vec![E1.to_owned(), E2.to_owned()]);
    let form: FormSnapshot = serde_json::from_str(run.form_snapshot.as_deref().unwrap()).unwrap();
    let keys: Vec<&str> = form
        .fields
        .iter()
        .map(|f| f.definition.key.as_str())
        .collect();
    assert_eq!(
        &keys[..4],
        &[DESIGNER, WOODS, "custom.width", "custom.scratches"]
    );
    let woods = form
        .fields
        .iter()
        .find(|f| f.definition.key == WOODS)
        .unwrap();
    assert_eq!(woods.value.as_deref(), Some("[\"Oak\"]"));
    assert_eq!(woods.provenance.as_deref(), Some("ai"));
}

/// Milliseconds from `now_ms` to the next local midnight.
fn to_local_midnight(now_ms: i64) -> i64 {
    let now = Local.timestamp_millis_opt(now_ms).unwrap();
    let next = (now.date_naive() + chrono::Days::new(1))
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let next = Local.from_local_datetime(&next).earliest().unwrap();
    next.timestamp_millis() - now_ms
}

#[test]
fn a13_c8a_day_limit_waits_for_the_next_local_date() {
    let h = Harness::chair();
    h.patch_settings(json!({"max_cost_per_day": "0.01"}));
    let (other_item, other_session) = h.new_session("R4-B");
    let earlier = h.insert_run_for(other_item, other_session, RunState::Finished, None, None);
    let now = h.now();
    h.query(|c| {
        let id = rstore::insert_call(
            c,
            &rstore::NewCall {
                run_id: earlier,
                stage: "observe",
                unit_key: "photo:0",
                attempt: 1,
                provider: "fake-model",
                model: "m1",
                effort: "low",
                request: "{}",
                started_at: now,
                redaction_hit: false,
            },
        )
        .unwrap();
        rstore::end_call(
            c,
            id,
            &rstore::CallEnd {
                state: "answered",
                response: Some("{}"),
                usage: None,
                cost_micro_usd: 10_000,
                finished_at: now,
                duration_ms: 0,
                error_class: None,
                error_text: None,
                redaction_hit: false,
            },
        )
        .unwrap();
    });
    main_photo(&h);
    let run = h
        .runs(h.session_id)
        .into_iter()
        .find(|r| r.state == RunState::Queued)
        .unwrap()
        .id;
    h.tick(1_000);
    assert_eq!(h.run_row(run).state, RunState::Queued);
    assert_eq!(reason(&h), pair("DayLimit", "Day limit reached"));
    let wait = to_local_midnight(h.now());
    h.tick(wait - 1);
    assert_eq!(h.run_row(run).state, RunState::Queued);
    h.tick(1);
    assert_ne!(h.run_row(run).state, RunState::Queued);
}

fn job_next_attempt(h: &Harness, run: i64) -> (String, i64) {
    h.query(|c| {
        c.query_row(
            "SELECT status,next_attempt_at FROM jobs WHERE kind='research' AND target=?1",
            [run.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
    })
}

#[test]
fn a13_c8b_offline_holds_the_run_and_retries_within_a_minute() {
    let h = Harness::chair();
    h.model.fault(fakes::offline());
    h.model.fault(fakes::offline());
    main_photo(&h);
    h.tick(1_000);
    let run = h.runs(h.session_id)[0].id;
    for failures in 1..=2 {
        assert_eq!(h.run_row(run).state, RunState::Running);
        let not_sent = h
            .calls(run)
            .iter()
            .filter(|c| c.state == "not_sent")
            .count();
        assert_eq!(not_sent, failures);
        assert_eq!(reason(&h), pair("Offline", "Offline"));
        let (status, next) = job_next_attempt(&h, run);
        assert_eq!(status, "queued");
        assert!(
            next > h.now() && next - h.now() <= 60_000,
            "{}",
            next - h.now()
        );
        h.tick(next - h.now());
    }
    assert_eq!(h.run_row(run).state, RunState::Finished);
    let observe: Vec<(String, i64)> = h
        .calls(run)
        .into_iter()
        .filter(|c| c.stage == "observe")
        .map(|c| (c.state, c.attempt))
        .collect();
    assert_eq!(
        observe,
        vec![
            ("not_sent".to_owned(), 1),
            ("not_sent".to_owned(), 1),
            ("answered".to_owned(), 1)
        ]
    );
    assert_eq!(reason(&h), None);
}
