//! MC-A6: which prerequisite fails first, whether a queued run may start, and the start order.
//! Plain values in, plain values out.

use serde::{Deserialize, Serialize};

use crate::domain::{Prerequisite, RunTrigger};

/// The photo-derived input identities a run started from (MC-A7 trigger set): the stage 1 unit
/// photos `(photo id, composition revision)`, the cover's, and the stage 6 Condition photos'.
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
    pub day_limit_reached: bool,
    pub offline: bool,
    pub auto: bool,
}

/// MC-A6 order: no integration; no model key; no visual-search key; a price missing; nothing to
/// research; day limit; offline; `auto` off.
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
    } else if input.day_limit_reached {
        Some(Prerequisite::DayLimit)
    } else if input.offline {
        Some(Prerequisite::Offline)
    } else if !input.auto {
        Some(Prerequisite::AutoOff)
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
    pub quiet_ms: i64,
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
    Quiet,
    WaitingForSource,
    /// No eligible photo has a source (Capture One closed): "Waiting for Capture One".
    WaitingForEditor,
}

/// MC-A6 start: one run app-wide; the day cap; the quiet period (`manual` skips it); readiness
/// (an automatic run waits up to the source wait for missing sources; no source at all waits).
pub fn may_start(input: &StartInput) -> MayStart {
    let automatic = input.trigger != RunTrigger::Manual;
    if input.blocked {
        MayStart::Blocked
    } else if input.any_running {
        MayStart::OtherRunning
    } else if input.day_limit_reached {
        MayStart::DayLimit
    } else if automatic && input.since_change_ms < input.quiet_ms {
        MayStart::Quiet
    } else if input.with_source == 0 {
        MayStart::WaitingForEditor
    } else if automatic && input.without_source > 0 && input.since_change_ms < input.source_wait_ms
    {
        MayStart::WaitingForSource
    } else {
        MayStart::Yes
    }
}

/// MC-A6 start order over `(trigger, queued_at, id)`: `manual` first, then `queued_at`, then id.
pub fn start_order(queued: &[(RunTrigger, i64, i64)]) -> Vec<i64> {
    let mut ordered = queued.to_vec();
    ordered
        .sort_by_key(|(trigger, queued_at, id)| (*trigger != RunTrigger::Manual, *queued_at, *id));
    ordered.into_iter().map(|(_, _, id)| id).collect()
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
            day_limit_reached: false,
            offline: false,
            auto: true,
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
            (|i| i.day_limit_reached = true, Prerequisite::DayLimit),
            (|i| i.offline = true, Prerequisite::Offline),
            (|i| i.auto = false, Prerequisite::AutoOff),
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
        input.auto = false;
        assert_eq!(first_failing(&input), Some(Prerequisite::Offline));
        let mut input = passing();
        input.integration = false;
        input.auto = false;
        assert_eq!(first_failing(&input), Some(Prerequisite::NoIntegration));
    }

    fn ready(trigger: RunTrigger) -> StartInput {
        StartInput {
            trigger,
            blocked: false,
            any_running: false,
            day_limit_reached: false,
            since_change_ms: 10_000,
            quiet_ms: 1_000,
            source_wait_ms: 120_000,
            with_source: 1,
            without_source: 0,
        }
    }

    #[test]
    fn may_start_ready() {
        assert_eq!(may_start(&ready(RunTrigger::Auto)), MayStart::Yes);
    }

    #[test]
    fn may_start_quiet_period_boundary() {
        let mut input = ready(RunTrigger::Auto);
        input.since_change_ms = 999;
        assert_eq!(may_start(&input), MayStart::Quiet);
        input.since_change_ms = 1_000;
        assert_eq!(may_start(&input), MayStart::Yes);
        let mut manual = ready(RunTrigger::Manual);
        manual.since_change_ms = 0;
        assert_eq!(may_start(&manual), MayStart::Yes);
        let mut follow = ready(RunTrigger::FollowUp);
        follow.since_change_ms = 0;
        assert_eq!(may_start(&follow), MayStart::Quiet);
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
        let mut input = ready(RunTrigger::Auto);
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
            (RunTrigger::Auto, 100, 1),
            (RunTrigger::FollowUp, 50, 4),
            (RunTrigger::Manual, 200, 3),
            (RunTrigger::Auto, 50, 2),
            (RunTrigger::Manual, 200, 5),
        ];
        assert_eq!(start_order(&queued), vec![3, 5, 2, 4, 1]);
    }
}
