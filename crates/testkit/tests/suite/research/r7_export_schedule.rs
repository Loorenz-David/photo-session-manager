//! R7·U2 (row A21·C1(d)): the wired app purges expired research call records at launch and
//! hourly, without an explicit call (08B MC-A11 retention), on the manual clock.

use psm_core::ports::clock::Clock;
use psm_research::{
    domain::{RunState, RunTrigger},
    retention::DAY_MS,
    store::{self as rstore, NewRun},
};
use psm_testkit::TestApp;

const NOW_MS: i64 = 1_762_900_000_000;

fn insert_call(a: &TestApp, run: i64, started_at: i64) -> i64 {
    a.app
        .store
        .research_transaction(|tx| {
            tx.conn()
                .execute(
                    "INSERT INTO research_calls(run_id,stage,unit_key,attempt,provider,model,\
                     effort,state,request,started_at) VALUES \
                     (?1,'observe','u',1,'fake-model','m1','low','answered','{}',?2)",
                    rusqlite::params![run, started_at],
                )
                .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
            Ok(tx.conn().last_insert_rowid())
        })
        .unwrap()
}

fn call_ids(a: &TestApp) -> Vec<i64> {
    a.app
        .store
        .research_transaction(|tx| rstore::calls(tx.conn(), None))
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect()
}

#[test]
fn a21_c1d_purged_at_launch_and_hourly_without_a_call() {
    let mut a = TestApp::with_research().unwrap();
    a.clock.advance(NOW_MS);
    let (item, session) = a
        .items()
        .create_local("R7-C1D", "Oak chair", "Chair", None)
        .unwrap();
    let now = a.clock.now_ms();
    let run = a
        .app
        .store
        .research_transaction(|tx| {
            rstore::insert_run(
                tx.conn(),
                &NewRun {
                    item_id: item.id,
                    session_id: session.session.id,
                    trigger: RunTrigger::Auto,
                    state: RunState::Finished,
                    queued_at: now - 31 * DAY_MS,
                    started_at: Some(now - 31 * DAY_MS),
                    settings_snapshot: None,
                    form_snapshot: None,
                    input_snapshot: None,
                },
            )
        })
        .unwrap();
    let expired = insert_call(&a, run, now - 31 * DAY_MS);
    // Expires 30 minutes from now (`record_days` 30).
    let later = insert_call(&a, run, now - 30 * DAY_MS + 30 * 60_000);
    a.kill_and_restart().unwrap();
    let left = call_ids(&a);
    assert!(!left.contains(&expired), "not purged at launch");
    assert_eq!(left, vec![later]);
    a.tick(30 * 60_000 + 1).unwrap();
    assert_eq!(call_ids(&a), vec![later], "purged before the hour");
    a.tick(30 * 60_000).unwrap();
    assert!(call_ids(&a).is_empty(), "not purged an hour after launch");
}
