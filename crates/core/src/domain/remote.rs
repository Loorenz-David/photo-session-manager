//! Remote items (book 10H §3): items whose pictures are only on the commerce platform, and the
//! platform's call budget (10H §8). Core vocabulary only: no platform name here (H7).

use serde::{Deserialize, Serialize};

/// The platform's call budget as last read and projected to now (MC-H5.2): the points available
/// and the most the budget holds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CallBudget {
    pub available: f64,
    pub maximum: f64,
}
