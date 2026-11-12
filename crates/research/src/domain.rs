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
