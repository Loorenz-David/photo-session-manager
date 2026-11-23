//! Beyo Pricing's price-label wire types (`openapi-integration.json` components, the package's
//! authority; `master_plan_labels.md` §3 Rule 17): camelCase, unknown fields ignored (the API is
//! additive), nullable and optional fields as `Option` with `#[serde(default)]`. An enum field the
//! domain reads strictly (`labelStatus`) is a strict enum; one it reads tolerantly
//! (`labelResetReason`) or never reads (`source`) is kept as text. The fake Pricing server
//! serialises through these same types.

use super::types::ErrorBody;
use serde::{Deserialize, Serialize};

/// `labelStatus` (strict: anything else is a reply this version does not understand).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WireLabelStatus {
    Printed,
    NotPrinted,
}

/// `IntegrationLabelProductPage`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationLabelProductPage {
    pub items: Vec<IntegrationLabelProduct>,
    #[serde(default)]
    pub next_cursor: Option<String>,
    pub counts: IntegrationLabelCounts,
    #[serde(default)]
    pub work_from: Option<String>,
}

/// `IntegrationLabelProduct`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationLabelProduct {
    pub product_id: String,
    #[serde(default)]
    pub sku: Option<String>,
    pub title: String,
    #[serde(default)]
    pub product_type: Option<String>,
    pub has_price: bool,
    #[serde(default)]
    pub price_minor: Option<i64>,
    pub label_status: WireLabelStatus,
    #[serde(default)]
    pub label_printed_at: Option<String>,
    /// `price_changed | sku_changed | barcode_changed`, null or absent (read tolerantly).
    #[serde(default)]
    pub label_reset_reason: Option<String>,
    #[serde(default)]
    pub barcode: Option<String>,
    #[serde(default)]
    pub thumb_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// `IntegrationLabelCounts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationLabelCounts {
    pub total: i64,
    pub has_price: i64,
    pub missing_price: i64,
    pub missing_price_tag: i64,
    pub printed: i64,
    pub not_printed: i64,
}

/// `IntegrationLabelStatus`: one entry per requested product.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationLabelStatus {
    #[serde(default)]
    pub sku: Option<String>,
    #[serde(default)]
    pub product_id: Option<String>,
    /// Null for a product the store does not have (with `error`).
    #[serde(default)]
    pub label: Option<LabelStatusOut>,
    #[serde(default)]
    pub error: Option<ErrorBody>,
}

/// `LabelStatusOut`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelStatusOut {
    pub product_id: String,
    pub label_status: WireLabelStatus,
    #[serde(default)]
    pub label_printed_at: Option<String>,
    /// Nullable: null when the product is not in the label universe.
    #[serde(default)]
    pub has_price: Option<bool>,
    /// `export | baseline`; never read.
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub reset_reason: Option<String>,
    #[serde(default)]
    pub reset_at: Option<String>,
    #[serde(default)]
    pub printed_price_minor: Option<i64>,
    #[serde(default)]
    pub printed_sku: Option<String>,
}

/// `IntegrationLabelTemplate`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationLabelTemplate {
    pub id: String,
    pub name: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub is_default: bool,
    pub product_types: Vec<String>,
}
