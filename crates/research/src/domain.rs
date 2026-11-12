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

/// The seven stages of a run, numbered 1–7 (MC-A7).
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
}

impl Stage {
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
