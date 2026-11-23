//! Remote items (book 10H §3): items whose pictures are only on the commerce platform, and the
//! platform's call budget (10H §8). Core vocabulary only: no platform name here (H7).

use serde::{Deserialize, Serialize};

/// The platform's call budget as last read and projected to now (MC-H5.2): the budget available
/// and the most the budget holds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CallBudget {
    pub available: f64,
    pub maximum: f64,
}

/// The two kinds of remote item (10H §3): files cleared on this Mac (`Archived`), or a shop
/// product never photographed on this Mac (`ShopOnly`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteKind {
    Archived,
    ShopOnly,
}

/// Where an item's pictures are (MC-H3.1). Wire form: `local`, `unknown`, `archived`,
/// `shop_only`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteState {
    /// On this Mac (or no integration).
    Local,
    /// An adopted shop product not yet read (its first open reads it).
    Unknown,
    Remote(RemoteKind),
}

impl RemoteState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Unknown => "unknown",
            Self::Remote(RemoteKind::Archived) => "archived",
            Self::Remote(RemoteKind::ShopOnly) => "shop_only",
        }
    }
}

impl Serialize for RemoteState {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RemoteState {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        match text.as_str() {
            "local" => Ok(Self::Local),
            "unknown" => Ok(Self::Unknown),
            "archived" => Ok(Self::Remote(RemoteKind::Archived)),
            "shop_only" => Ok(Self::Remote(RemoteKind::ShopOnly)),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["local", "unknown", "archived", "shop_only"],
            )),
        }
    }
}
