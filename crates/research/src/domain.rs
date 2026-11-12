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
