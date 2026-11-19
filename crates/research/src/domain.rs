//! Research domain types (`master_plan_research.md` §6.3). Serialized snake_case.

use serde::{Deserialize, Serialize};

/// The class of a page's site (MC-A3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteClass {
    Shop,
    Trusted,
    Untrusted,
}

/// A stage 4 verdict on a listing page (MC-A4: only `Match` and `Similar` count).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Match,
    Similar,
    Different,
}

/// The class of a field for confidence derivation (MC-A4; the first match wins).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldClass {
    NeverWritten,
    Derived,
    Condition,
    Attribution,
    Measured,
    Visible,
}

/// A stage 5 reading on one photo that names a value for the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelItem {
    pub photo_id: i64,
    pub stated: crate::rules::value_map::Stated,
}

/// A page read in stage 3 that states a value for the key, with its stage 4 verdict (`None` when
/// it was not compared) and its site (MC-A3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListingItem {
    /// The page identity `U`.
    pub page: String,
    pub site: crate::rules::sites::Site,
    pub verdict: Option<Verdict>,
    pub stated: crate::rules::value_map::Stated,
}

/// The model's read of a value from our own photos, with its `sure` flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoRead {
    pub value: serde_json::Value,
    pub sure: bool,
}

/// The seven stages of a run, numbered 1–7 (MC-A7), and (W2) the writer's one stage, number 8,
/// which is not one of the research stages ([`Stage::ALL`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Observe,
    Find,
    Read,
    Compare,
    Labels,
    Condition,
    Decide,
    /// W2 (WC7): the writer's call, units `texts` and `retry`, and its `result` unit.
    Write,
}

impl Stage {
    /// The seven research stages (never [`Stage::Write`]).
    pub const ALL: [Stage; 7] = [
        Self::Observe,
        Self::Find,
        Self::Read,
        Self::Compare,
        Self::Labels,
        Self::Condition,
        Self::Decide,
    ];

    pub fn number(self) -> u8 {
        match self {
            Self::Observe => 1,
            Self::Find => 2,
            Self::Read => 3,
            Self::Compare => 4,
            Self::Labels => 5,
            Self::Condition => 6,
            Self::Decide => 7,
            Self::Write => 8,
        }
    }
}

/// The class of a failed provider call (MC-A9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorClass {
    Offline,
    Transient,
    Key,
    Quota,
    Request,
    Schema,
    Refusal,
}

/// What went wrong with a call, before classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    /// Connect or DNS failure.
    Connect,
    Timeout,
    /// The connection was reset or dropped.
    Reset,
    /// The provider answered with an HTTP error status.
    Status,
    /// The reply does not match the stage's output schema.
    Schema,
    /// The model refused.
    Refusal,
}

/// A failed provider call as an adapter reports it (`sent` false: no request byte left).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallFailure {
    pub sent: bool,
    pub status: Option<u16>,
    pub kind: FailureKind,
    /// The provider reports no credit or quota.
    pub quota: bool,
    pub text: String,
    pub usage: Option<Vec<(String, i64)>>,
}

impl Stage {
    /// The settings and storage name (`observe` … `decide`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::Find => "find",
            Self::Read => "read",
            Self::Compare => "compare",
            Self::Labels => "labels",
            Self::Condition => "condition",
            Self::Decide => "decide",
            Self::Write => "write",
        }
    }

    /// Every stage by its storage name, the writer's included (so a replay can rebuild a writer
    /// call, WC7).
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .chain([Self::Write])
            .find(|s| s.as_str() == text)
    }
}

/// Generates `as_str` / `parse` for a snake_case storage enum.
macro_rules! stored_names {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        impl $name {
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),+
                }
            }

            pub fn parse(text: &str) -> Option<Self> {
                match text {
                    $($text => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

/// The life of a run (MC-A6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Queued,
    Running,
    Finished,
    Failed,
    StoppedAtCap,
    Cancelled,
}

stored_names!(RunState {
    Queued => "queued",
    Running => "running",
    Finished => "finished",
    Failed => "failed",
    StoppedAtCap => "stopped_at_cap",
    Cancelled => "cancelled",
});

/// Why a run exists (MC-A6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTrigger {
    Auto,
    Manual,
    FollowUp,
    Arranged,
    /// Owner correction A12: one call from the "Search this photo" window (Ask AI on selected
    /// text, or a visual search of the pinned photo). Never queued or started; it holds that one
    /// call's record and cost, and is not one of the session's research runs (the panel, the
    /// "Arranged" check and reuse never see it). The export includes it.
    Browse,
    /// W1 (WC1.8): **Look again**: stages 1 and 6 only, to renew the product note after the
    /// photos changed. Not one of the session's research runs (WC1.9: the panel, the overview,
    /// the "Arranged" check and a research run's reuse never see it); started like `manual`.
    Look,
    /// W2 (WC3): a **writer** run: one text call (two with a rewrite), no photo. Not one of the
    /// session's research runs (WC1.9); started like `manual`, without the source wait.
    Write,
}

stored_names!(RunTrigger {
    Auto => "auto",
    Manual => "manual",
    FollowUp => "follow_up",
    Arranged => "arranged",
    Browse => "browse",
    Look => "look",
    Write => "write",
});

impl RunTrigger {
    /// W1 (WC1.5): the runs whose stage 1 and stage 6 outputs hold a product note.
    pub fn looks_at_photos(self) -> bool {
        matches!(
            self,
            Self::Arranged | Self::Manual | Self::Auto | Self::FollowUp | Self::Look
        )
    }
}

/// The kind of a trace step (08A §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Looked,
    Searched,
    Read,
    Compared,
    Concluded,
    NotDetermined,
}

stored_names!(StepKind {
    Looked => "looked",
    Searched => "searched",
    Read => "read",
    Compared => "compared",
    Concluded => "concluded",
    NotDetermined => "not_determined",
});

/// The state of one provider call record (MC-A9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallState {
    Sent,
    Answered,
    Failed,
    Unknown,
    NotSent,
}

stored_names!(CallState {
    Sent => "sent",
    Answered => "answered",
    Failed => "failed",
    Unknown => "unknown",
    NotSent => "not_sent",
});

stored_names!(ErrorClass {
    Offline => "offline",
    Transient => "transient",
    Key => "key",
    Quota => "quota",
    Request => "request",
    Schema => "schema",
    Refusal => "refusal",
});

/// Why research does not run (MC-A6 order), plus the waiting reason.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Prerequisite {
    NoIntegration,
    NoModelKey,
    NoSearchKey,
    /// The `(provider, model)` pair with no `fallback_call_usd`.
    PriceMissing {
        provider: String,
        model: String,
    },
    NothingToResearch,
    NoFieldChosen,
    DayLimit,
    Offline,
    WaitingForEditor,
}

impl Prerequisite {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoIntegration => "NoIntegration",
            Self::NoModelKey => "NoModelKey",
            Self::NoSearchKey => "NoSearchKey",
            Self::PriceMissing { .. } => "PriceMissing",
            Self::NothingToResearch => "NothingToResearch",
            Self::NoFieldChosen => "NoFieldChosen",
            Self::DayLimit => "DayLimit",
            Self::Offline => "Offline",
            Self::WaitingForEditor => "WaitingForEditor",
        }
    }

    /// The panel sentence (`master_plan_research.md` §6.6).
    pub fn sentence(&self) -> String {
        match self {
            Self::NoIntegration => "Connect a shop to use research".to_owned(),
            Self::NoModelKey => "Add the model key in Settings → Research".to_owned(),
            Self::NoSearchKey => "Add the visual search key in Settings → Research".to_owned(),
            Self::PriceMissing { provider, model } => {
                format!("No price is set for {provider} {model}")
                    .trim_end()
                    .to_owned()
            }
            Self::NothingToResearch => "This item type has no fields research can fill".to_owned(),
            Self::NoFieldChosen => {
                "Choose the fields research fills in Settings → Research".to_owned()
            }
            Self::DayLimit => "Day limit reached".to_owned(),
            Self::Offline => "Offline".to_owned(),
            Self::WaitingForEditor => "Waiting for Capture One".to_owned(),
        }
    }

    /// The first five stop a run row from being created at all.
    pub fn blocks_creation(&self) -> bool {
        matches!(
            self,
            Self::NoIntegration
                | Self::NoModelKey
                | Self::NoSearchKey
                | Self::PriceMissing { .. }
                | Self::NothingToResearch
                | Self::NoFieldChosen
        )
    }
}
