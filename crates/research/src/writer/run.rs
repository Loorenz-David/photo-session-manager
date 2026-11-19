//! W2: the writer run (`master_plan_writer.md` §5 WC3, WC4, WC5 "Regenerate once", WC7).
//!
//! Unit 3: when a `write` run is queued ([`request_decision`], `ResearchService::writer_request`)
//! and what stops it from being created ([`WriterBlock`]). Unit 4: what it does once started
//! ([`execute`]).

use psm_core::Result;
use serde_json::json;

use crate::{
    api::Reason,
    config::ResearchSettings,
    domain::{RunState, RunTrigger},
    service::{MODEL_ACCOUNT, ResearchService, form_snapshot},
    stages::{Ctx, ExecuteOutcome},
    store::{self, NewRun},
    writer::{api::Requested, config::WriterSettings, input::requestable_texts},
};

/// Why a writer run was asked for (`master_plan_writer.md` §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// "Next to review →" after the AI review.
    NextToReview,
    /// Leaving a session whose AI review shows "All checked ✓".
    AllChecked,
    /// The "Write texts" button.
    WriteTexts,
    /// An answer to a writer question or finding (internal).
    Answer,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NextToReview => "next_to_review",
            Self::AllChecked => "all_checked",
            Self::WriteTexts => "write_texts",
            Self::Answer => "answer",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [
            Self::NextToReview,
            Self::AllChecked,
            Self::WriteTexts,
            Self::Answer,
        ]
        .into_iter()
        .find(|o| o.as_str() == text)
    }

    /// The origins of automatic writing (WC3.4).
    pub fn automatic(self) -> bool {
        matches!(self, Self::NextToReview | Self::AllChecked)
    }
}

/// Why the writer does not run (`master_plan_writer.md` §4 `writer::WriterBlock`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriterBlock {
    NoIntegration,
    NoModelKey,
    PriceMissing { provider: String, model: String },
    NothingToWrite,
    ResearchBusy,
    DayLimit,
    Offline,
}

impl WriterBlock {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoIntegration => "NoIntegration",
            Self::NoModelKey => "NoModelKey",
            Self::PriceMissing { .. } => "PriceMissing",
            Self::NothingToWrite => "NothingToWrite",
            Self::ResearchBusy => "ResearchBusy",
            Self::DayLimit => "DayLimit",
            Self::Offline => "Offline",
        }
    }

    pub fn sentence(&self) -> String {
        match self {
            Self::NoIntegration => "Connect a shop to write texts".to_owned(),
            Self::NoModelKey => "Add the model key in Settings → Research".to_owned(),
            Self::PriceMissing { provider, model } => {
                format!("No price is set for {provider} {model}")
                    .trim_end()
                    .to_owned()
            }
            Self::NothingToWrite => "This item has no text the writer can write".to_owned(),
            Self::ResearchBusy => "Research is still working on this session".to_owned(),
            Self::DayLimit => "Day limit reached".to_owned(),
            Self::Offline => "Offline".to_owned(),
        }
    }

    pub fn reason(&self) -> Reason {
        Reason {
            code: self.code().to_owned(),
            sentence: self.sentence(),
        }
    }
}

/// The facts WC3 decides from, read in the request's transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestInput {
    /// The session has an item (not the Unassigned session).
    pub has_item: bool,
    /// A run of the session with trigger ∉ {`browse`, `write`} is queued or running.
    pub research_busy: bool,
    /// The first failing creation-blocking prerequisite (WC3.3).
    pub block: Option<WriterBlock>,
    pub origin: Origin,
    /// `writer.auto`.
    pub auto: bool,
    /// The states of the session's `write` runs.
    pub write_states: Vec<RunState>,
}

/// What a request does (WC3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestDecision {
    Refused(WriterBlock),
    /// Nothing, no reason shown (an automatic origin that is off or already used).
    Nothing,
    /// A `write` run is already queued: nothing new, `queued: true`.
    AlreadyQueued,
    /// Insert one `queued` `write` run.
    Insert,
}

/// WC3 items 1–6 in order (the pure part of `request`).
pub fn request_decision(input: &RequestInput) -> RequestDecision {
    if !input.has_item {
        return RequestDecision::Refused(WriterBlock::NothingToWrite);
    }
    if input.research_busy {
        return RequestDecision::Refused(WriterBlock::ResearchBusy);
    }
    if let Some(block) = &input.block {
        return RequestDecision::Refused(block.clone());
    }
    if input.origin.automatic()
        && (!input.auto
            || input.write_states.iter().any(|s| {
                matches!(
                    s,
                    RunState::Queued
                        | RunState::Running
                        | RunState::Finished
                        | RunState::StoppedAtCap
                )
            }))
    {
        return RequestDecision::Nothing;
    }
    if input.write_states.contains(&RunState::Queued) {
        return RequestDecision::AlreadyQueued;
    }
    RequestDecision::Insert
}

/// Where a queued writer run keeps its origin until the start transaction snapshots it.
pub(crate) const ORIGIN_KEY: &str = "writer_origin";

impl ResearchService {
    /// WC3.3: the first failing creation-blocking prerequisite of a writer run of the item.
    pub(crate) fn writer_block(
        &self,
        item_id: i64,
        settings: &ResearchSettings,
        writer: &WriterSettings,
    ) -> Result<Option<WriterBlock>> {
        if self.shop.is_none() {
            return Ok(Some(WriterBlock::NoIntegration));
        }
        if self.providers.model.is_none() || !self.key_set(MODEL_ACCOUNT) {
            return Ok(Some(WriterBlock::NoModelKey));
        }
        let provider = settings.providers.resolved().model;
        if !settings
            .fallbacks()
            .iter()
            .any(|f| f.provider == provider && f.model == writer.model)
        {
            return Ok(Some(WriterBlock::PriceMissing {
                provider,
                model: writer.model.clone(),
            }));
        }
        let form = form_snapshot(
            &self.store,
            self.clock.as_ref(),
            self.integration.as_deref(),
            item_id,
        )?;
        if requestable_texts(&form, writer).is_empty() {
            return Ok(Some(WriterBlock::NothingToWrite));
        }
        Ok(None)
    }

    /// `writer::run::request` (WC3): queues a `write` run of the session, or says why not.
    pub fn writer_request(&self, session_id: i64, origin: Origin) -> Result<Requested> {
        let item = self.session_item(session_id)?;
        let writer = self.writer_settings();
        let block = match item {
            Some(item_id) => self.writer_block(item_id, &self.settings(), &writer)?,
            None => None,
        };
        let now = self.clock.now_ms();
        let (decision, queued) = self.store.research_transaction(|tx| {
            let conn = tx.conn();
            let runs = match item {
                Some(_) => store::all_runs(conn, Some(session_id))?,
                None => Vec::new(),
            };
            let decision = request_decision(&RequestInput {
                has_item: item.is_some(),
                research_busy: runs.iter().any(|r| {
                    !matches!(r.trigger, RunTrigger::Browse | RunTrigger::Write)
                        && matches!(r.state, RunState::Queued | RunState::Running)
                }),
                block: block.clone(),
                origin,
                auto: writer.auto,
                write_states: runs
                    .iter()
                    .filter(|r| r.trigger == RunTrigger::Write)
                    .map(|r| r.state)
                    .collect(),
            });
            let queued_before = runs
                .iter()
                .any(|r| r.trigger == RunTrigger::Write && r.state == RunState::Queued);
            let queued = match &decision {
                RequestDecision::Insert => {
                    store::insert_run(
                        conn,
                        &NewRun {
                            item_id: item.unwrap_or_default(),
                            session_id,
                            trigger: RunTrigger::Write,
                            state: RunState::Queued,
                            queued_at: now,
                            started_at: None,
                            settings_snapshot: Some(
                                json!({ ORIGIN_KEY: origin.as_str() }).to_string(),
                            ),
                            form_snapshot: None,
                            input_snapshot: None,
                        },
                    )?;
                    true
                }
                RequestDecision::AlreadyQueued => true,
                RequestDecision::Nothing => queued_before,
                RequestDecision::Refused(_) => false,
            };
            Ok((decision, queued))
        })?;
        let reason = match &decision {
            RequestDecision::Refused(block) => Some(block.clone()),
            _ => None,
        };
        if decision == RequestDecision::Insert {
            self.changed(session_id);
        }
        tracing::info!(
            target: "psm::writer",
            "writer.requested session={session_id} origin={} queued={queued} reason={}",
            origin.as_str(),
            reason.as_ref().map_or("", WriterBlock::code)
        );
        Ok(Requested {
            queued,
            reason: reason.map(|r| r.reason()),
        })
    }
}

/// Runs a started `write` run (WC4–WC7).
pub(crate) fn execute(ctx: &Ctx<'_>) -> Result<ExecuteOutcome> {
    crate::decide::end(ctx, RunState::Finished)?;
    Ok(ExecuteOutcome::Finished)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(origin: Origin) -> RequestInput {
        RequestInput {
            has_item: true,
            research_busy: false,
            block: None,
            origin,
            auto: true,
            write_states: Vec::new(),
        }
    }

    #[test]
    fn wc3_the_order_of_the_request_rules() {
        use RequestDecision::*;
        assert_eq!(request_decision(&input(Origin::WriteTexts)), Insert);
        let unassigned = RequestInput {
            has_item: false,
            research_busy: true,
            ..input(Origin::WriteTexts)
        };
        assert_eq!(
            request_decision(&unassigned),
            Refused(WriterBlock::NothingToWrite)
        );
        let busy = RequestInput {
            research_busy: true,
            block: Some(WriterBlock::NoModelKey),
            ..input(Origin::WriteTexts)
        };
        assert_eq!(request_decision(&busy), Refused(WriterBlock::ResearchBusy));
        let blocked = RequestInput {
            block: Some(WriterBlock::NoModelKey),
            ..input(Origin::NextToReview)
        };
        assert_eq!(request_decision(&blocked), Refused(WriterBlock::NoModelKey));
        let off = RequestInput {
            auto: false,
            ..input(Origin::NextToReview)
        };
        assert_eq!(request_decision(&off), Nothing);
        assert_eq!(
            request_decision(&RequestInput {
                auto: false,
                ..input(Origin::WriteTexts)
            }),
            Insert
        );
        for state in [
            RunState::Queued,
            RunState::Running,
            RunState::Finished,
            RunState::StoppedAtCap,
        ] {
            let used = RequestInput {
                write_states: vec![state],
                ..input(Origin::AllChecked)
            };
            assert_eq!(request_decision(&used), Nothing, "{state:?}");
        }
        for state in [RunState::Failed, RunState::Cancelled] {
            let retried = RequestInput {
                write_states: vec![state],
                ..input(Origin::AllChecked)
            };
            assert_eq!(request_decision(&retried), Insert, "{state:?}");
        }
        let queued = RequestInput {
            write_states: vec![RunState::Running, RunState::Queued],
            ..input(Origin::WriteTexts)
        };
        assert_eq!(request_decision(&queued), AlreadyQueued);
        let beside_running = RequestInput {
            write_states: vec![RunState::Running],
            ..input(Origin::Answer)
        };
        assert_eq!(request_decision(&beside_running), Insert);
    }
}
