//! MC-A6: which prerequisite fails first, whether a queued run may start, and the start order.
//! Plain values in, plain values out.

use serde::{Deserialize, Serialize};

use crate::domain::{Prerequisite, RunState, RunTrigger};

/// The photo-derived input identities a run started from (MC-A7 trigger set): the photos the one
/// stage 1 unit looks at `(photo id, composition revision)` (R12·U1), the cover's, and the stage 6
/// Condition photos'.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputSnapshot {
    pub observe: Vec<(i64, i64)>,
    pub cover: Option<(i64, i64)>,
    pub condition: Vec<(i64, i64)>,
}

/// The facts the prerequisites are read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrerequisiteInput {
    pub integration: bool,
    pub model_key: bool,
    pub search_key: bool,
    /// The first `(provider, model)` a run can call that has no `fallback_call_usd`.
    pub missing_price: Option<(String, String)>,
    /// The layout has a field research may write (MC-A1).
    pub writable_field: bool,
    pub chosen_field: bool,
    pub day_limit_reached: bool,
    pub offline: bool,
}

/// MC-A6 order: no integration; no model key; no visual-search key; a price missing; nothing to
/// research; day limit; offline.
pub fn first_failing(input: &PrerequisiteInput) -> Option<Prerequisite> {
    if !input.integration {
        Some(Prerequisite::NoIntegration)
    } else if !input.model_key {
        Some(Prerequisite::NoModelKey)
    } else if !input.search_key {
        Some(Prerequisite::NoSearchKey)
    } else if let Some((provider, model)) = &input.missing_price {
        Some(Prerequisite::PriceMissing {
            provider: provider.clone(),
            model: model.clone(),
        })
    } else if !input.writable_field {
        Some(Prerequisite::NothingToResearch)
    } else if !input.chosen_field {
        Some(Prerequisite::NoFieldChosen)
    } else if input.day_limit_reached {
        Some(Prerequisite::DayLimit)
    } else if input.offline {
        Some(Prerequisite::Offline)
    } else {
        None
    }
}

/// The facts one queued run's start depends on (times in ms on the monotonic clock).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartInput {
    pub trigger: RunTrigger,
    /// One of the first five prerequisites fails.
    pub blocked: bool,
    pub any_running: bool,
    pub day_limit_reached: bool,
    /// Since the session's last change (at launch: since the launch).
    pub since_change_ms: i64,
    pub source_wait_ms: i64,
    /// Eligible photos with and without a research source.
    pub with_source: usize,
    pub without_source: usize,
}

/// Whether a queued run may start, or why not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MayStart {
    Yes,
    Blocked,
    OtherRunning,
    DayLimit,
    WaitingForSource,
    /// No eligible photo has a source (Capture One closed): "Waiting for Capture One".
    WaitingForEditor,
}

/// MC-A6 start: one run app-wide; the day cap; readiness
/// (an automatic run waits up to the source wait for missing sources; no source at all waits).
pub fn may_start(input: &StartInput) -> MayStart {
    // W1 (WC1.8): a look run starts like a manual one.
    let automatic = !matches!(input.trigger, RunTrigger::Manual | RunTrigger::Look);
    if input.blocked {
        MayStart::Blocked
    } else if input.any_running {
        MayStart::OtherRunning
    } else if input.day_limit_reached {
        MayStart::DayLimit
    } else if input.with_source == 0 {
        MayStart::WaitingForEditor
    } else if automatic && input.without_source > 0 && input.since_change_ms < input.source_wait_ms
    {
        MayStart::WaitingForSource
    } else {
        MayStart::Yes
    }
}

/// MC-A6 start order over `(trigger, queued_at, id)`: tier 0 `manual` and (W1, WC3 "Start")
/// `look` first, then `queued_at`, then id.
pub fn start_order(queued: &[(RunTrigger, i64, i64)]) -> Vec<i64> {
    let mut ordered = queued.to_vec();
    ordered.sort_by_key(|(trigger, queued_at, id)| {
        (
            !matches!(trigger, RunTrigger::Manual | RunTrigger::Look),
            *queued_at,
            *id,
        )
    });
    ordered.into_iter().map(|(_, _, id)| id).collect()
}

/// A19 (owner 2026-11-18): research starts by itself at most once per session. "Arranged — next"
/// queues a run only when none of the session's runs (`states`, browse runs left out) is queued,
/// running, finished or stopped at the cost limit. A failed or cancelled run produced nothing, so
/// arranging again may retry it. After that the only way to research again is **Run again**
/// (`run_again`); photo changes never queue a follow-up (replaces MC-A7's trigger set).
pub fn arranged_queues(states: &[RunState]) -> bool {
    !states.iter().any(|state| {
        matches!(
            state,
            RunState::Queued | RunState::Running | RunState::Finished | RunState::StoppedAtCap
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passing() -> PrerequisiteInput {
        PrerequisiteInput {
            integration: true,
            model_key: true,
            search_key: true,
            missing_price: None,
            writable_field: true,
            chosen_field: true,
            day_limit_reached: false,
            offline: false,
        }
    }

    #[test]
    fn first_failing_none_when_all_pass() {
        assert_eq!(first_failing(&passing()), None);
    }

    /// Makes one prerequisite fail.
    type Fail = fn(&mut PrerequisiteInput);

    #[test]
    fn first_failing_each_alone() {
        let cases: Vec<(Fail, Prerequisite)> = vec![
            (|i| i.integration = false, Prerequisite::NoIntegration),
            (|i| i.model_key = false, Prerequisite::NoModelKey),
            (|i| i.search_key = false, Prerequisite::NoSearchKey),
            (
                |i| i.missing_price = Some(("p".into(), "m".into())),
                Prerequisite::PriceMissing {
                    provider: "p".into(),
                    model: "m".into(),
                },
            ),
            (
                |i| i.writable_field = false,
                Prerequisite::NothingToResearch,
            ),
            (|i| i.chosen_field = false, Prerequisite::NoFieldChosen),
            (|i| i.day_limit_reached = true, Prerequisite::DayLimit),
            (|i| i.offline = true, Prerequisite::Offline),
        ];
        for (fail, expected) in cases {
            let mut input = passing();
            fail(&mut input);
            assert_eq!(first_failing(&input), Some(expected));
        }
    }

    #[test]
    fn first_failing_takes_the_earlier_in_order() {
        let mut input = passing();
        input.model_key = false;
        input.day_limit_reached = true;
        assert_eq!(first_failing(&input), Some(Prerequisite::NoModelKey));
        let mut input = passing();
        input.offline = true;
        input.integration = false;
        assert_eq!(first_failing(&input), Some(Prerequisite::NoIntegration));
    }

    #[test]
    fn a27_c3a_no_chosen_field_fails() {
        let mut input = passing();
        input.chosen_field = false;
        assert_eq!(first_failing(&input), Some(Prerequisite::NoFieldChosen));
    }

    #[test]
    fn a27_c3b_nothing_to_research_precedes_no_chosen_field() {
        let mut input = passing();
        input.writable_field = false;
        input.chosen_field = false;
        assert_eq!(first_failing(&input), Some(Prerequisite::NothingToResearch));
    }

    #[test]
    fn a27_c3c_no_chosen_field_precedes_day_limit() {
        let mut input = passing();
        input.chosen_field = false;
        input.day_limit_reached = true;
        assert_eq!(first_failing(&input), Some(Prerequisite::NoFieldChosen));
    }

    fn ready(trigger: RunTrigger) -> StartInput {
        StartInput {
            trigger,
            blocked: false,
            any_running: false,
            day_limit_reached: false,
            since_change_ms: 10_000,
            source_wait_ms: 120_000,
            with_source: 1,
            without_source: 0,
        }
    }

    #[test]
    fn may_start_ready() {
        assert_eq!(may_start(&ready(RunTrigger::Arranged)), MayStart::Yes);
    }

    #[test]
    fn may_start_arranged_without_wait() {
        let mut input = ready(RunTrigger::Arranged);
        input.since_change_ms = 0;
        assert_eq!(may_start(&input), MayStart::Yes);
    }

    #[test]
    fn may_start_blockers_in_order() {
        let mut input = ready(RunTrigger::Manual);
        input.blocked = true;
        input.any_running = true;
        assert_eq!(may_start(&input), MayStart::Blocked);
        input.blocked = false;
        assert_eq!(may_start(&input), MayStart::OtherRunning);
        input.any_running = false;
        input.day_limit_reached = true;
        assert_eq!(may_start(&input), MayStart::DayLimit);
    }

    #[test]
    fn may_start_readiness() {
        let mut input = ready(RunTrigger::Arranged);
        input.without_source = 1;
        input.since_change_ms = 119_999;
        assert_eq!(may_start(&input), MayStart::WaitingForSource);
        input.since_change_ms = 120_000;
        assert_eq!(may_start(&input), MayStart::Yes);
        input.with_source = 0;
        assert_eq!(may_start(&input), MayStart::WaitingForEditor);
        let mut manual = ready(RunTrigger::Manual);
        manual.without_source = 1;
        manual.since_change_ms = 0;
        assert_eq!(may_start(&manual), MayStart::Yes);
    }

    #[test]
    fn start_order_manual_then_fifo_then_id() {
        let queued = vec![
            (RunTrigger::Arranged, 100, 1),
            (RunTrigger::Arranged, 50, 4),
            (RunTrigger::Manual, 200, 3),
            (RunTrigger::Arranged, 50, 2),
            (RunTrigger::Manual, 200, 5),
        ];
        assert_eq!(start_order(&queued), vec![3, 5, 2, 4, 1]);
    }

    #[test]
    fn a19_arranging_queues_only_while_no_run_produced_or_will_produce_anything() {
        use RunState::*;
        assert!(arranged_queues(&[]));
        assert!(arranged_queues(&[Failed]));
        assert!(arranged_queues(&[Cancelled, Failed]));
        for state in [Queued, Running, Finished, StoppedAtCap] {
            assert!(!arranged_queues(&[state]), "{state:?}");
            assert!(!arranged_queues(&[Failed, state, Cancelled]), "{state:?}");
        }
    }
}
