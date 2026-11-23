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

/// `IntegrationRunItemIn` (plan and run items): this app always sends `productId` only (09C L8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationRunItemIn {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
}

/// `IntegrationPlanRequest`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationPlanRequest {
    pub items: Vec<IntegrationRunItemIn>,
}

/// `IntegrationPreviewRequest` (the same fields as one item).
pub type IntegrationPreviewRequest = IntegrationRunItemIn;

/// `IntegrationRunCreate`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationRunCreate {
    pub items: Vec<IntegrationRunItemIn>,
    pub idempotency_key: String,
}

/// `IntegrationPlanItem.outcome` (strict).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WirePlanOutcome {
    MarkPrinted,
    Reprint,
    NoPrice,
    Fail,
}

/// `IntegrationPlanTemplate` (`source` never read).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationPlanTemplate {
    pub id: String,
    pub name: String,
    pub width_mm: f64,
    pub height_mm: f64,
    #[serde(default)]
    pub source: Option<String>,
}

/// `IntegrationPlanIssue` (issues and notes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationPlanIssue {
    pub code: String,
    pub message: String,
}

/// `IntegrationPlanItem`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationPlanItem {
    pub position: usize,
    #[serde(default)]
    pub sku: Option<String>,
    pub product_id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub product_type: Option<String>,
    #[serde(default)]
    pub template: Option<IntegrationPlanTemplate>,
    pub has_price: bool,
    #[serde(default)]
    pub price_minor: Option<i64>,
    pub label_status: WireLabelStatus,
    #[serde(default)]
    pub label_printed_at: Option<String>,
    pub outcome: WirePlanOutcome,
    #[serde(default)]
    pub issues: Vec<IntegrationPlanIssue>,
    #[serde(default)]
    pub notes: Vec<IntegrationPlanIssue>,
}

/// `UnresolvedItem`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnresolvedItem {
    #[serde(default)]
    pub sku: Option<String>,
    #[serde(default)]
    pub product_id: Option<String>,
    pub error: ErrorBody,
}

/// `IntegrationPlanSummary`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationPlanSummary {
    pub items: i64,
    pub ready: i64,
    pub failing: i64,
    pub mark_printed: i64,
    pub reprint: i64,
    pub no_price: i64,
}

/// `IntegrationPlanSize`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationPlanSize {
    pub width_mm: f64,
    pub height_mm: f64,
    pub count: i64,
}

/// `IntegrationPlan`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationPlan {
    pub items: Vec<IntegrationPlanItem>,
    #[serde(default)]
    pub unresolved: Vec<UnresolvedItem>,
    pub summary: IntegrationPlanSummary,
    #[serde(default)]
    pub sizes: Vec<IntegrationPlanSize>,
}

/// `IntegrationLabelRun.status` (strict).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WireRunStatus {
    Rendered,
    Partial,
    Failed,
}

/// `RunCounts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WireRunCounts {
    pub items: i64,
    pub rendered: i64,
    pub failed: i64,
    pub marked_printed: i64,
    pub reprinted: i64,
    pub no_price: i64,
}

/// `RunArtifactOut`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunArtifactOut {
    pub id: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub page_count: i64,
    pub filename: String,
    pub size_bytes: i64,
    /// A path on the server (`/integration/v1/stores/{storeId}/labels/runs/{runId}/artifacts/…`).
    pub download_url: String,
    pub expired: bool,
}

/// `IssueOut`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueOut {
    pub code: String,
    pub message: String,
}

/// `RunItemOut`. `outcome` (`rendered | failed`) and `lifecycleResult` are read tolerantly: the
/// package's schema says `lifecycleResult` is one of three words or null, its `API.md` example
/// sends `"none"` (projection r0, D4); an unknown word must not turn a run that already marked
/// products Printed into a reply this version does not understand.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunItemOut {
    pub position: i64,
    pub product_id: String,
    #[serde(default)]
    pub sku: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub variant_id: Option<String>,
    #[serde(default)]
    pub template_revision_id: Option<String>,
    #[serde(default)]
    pub template_source: Option<String>,
    #[serde(default)]
    pub price_minor: Option<i64>,
    pub outcome: String,
    #[serde(default)]
    pub lifecycle_result: Option<String>,
    #[serde(default)]
    pub error: Option<IssueOut>,
    #[serde(default)]
    pub artifact_id: Option<String>,
    #[serde(default)]
    pub page_index: Option<i64>,
}

/// `IntegrationLabelRun` (the 201 of `POST …/labels/runs`, with `unresolved`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationLabelRun {
    pub id: String,
    pub status: WireRunStatus,
    pub created_at: String,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub requested_by_kind: Option<String>,
    #[serde(default)]
    pub requested_by_name: Option<String>,
    pub counts: WireRunCounts,
    pub artifacts: Vec<RunArtifactOut>,
    pub items: Vec<RunItemOut>,
    #[serde(default)]
    pub unresolved: Vec<UnresolvedItem>,
}
