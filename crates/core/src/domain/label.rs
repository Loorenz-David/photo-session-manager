//! The labels vocabulary (book 09C §3; `master_plan_labels.md` §4 Vocabulary, Errors). A labels
//! provider keeps the label state of a product (printed or not, why it went back to not printed)
//! and makes label files; core sees only these words (H7): no provider name, no address, no
//! currency word. Times are UTC milliseconds; money is the provider's minor units, never a float.

use super::conversation::ConnectionState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelStatus {
    Printed,
    NotPrinted,
}

/// Why a printed label went back to not printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetReason {
    PriceChanged,
    SkuChanged,
    BarcodeChanged,
}

/// A product's label state as the provider answers it (MC-L3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelState {
    pub status: LabelStatus,
    pub printed_at: Option<i64>,
    /// `None` = unknown.
    pub has_price: Option<bool>,
    pub reset: Option<ResetReason>,
}

/// Whether label calls may be made (ACC). The stored part is `Unknown | Allowed | Denied`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelsAccess {
    /// The conversations connection is not ready; its state.
    Off(ConnectionState),
    Unknown,
    Allowed,
    Denied,
}

impl LabelsAccess {
    /// The stored word (`Off` is never stored and reads as `unknown`).
    pub fn stored(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
            Self::Unknown | Self::Off(_) => "unknown",
        }
    }

    /// The stored word back (anything else → `Unknown`).
    pub fn from_stored(text: &str) -> Self {
        match text {
            "allowed" => Self::Allowed,
            "denied" => Self::Denied,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelFilter {
    MissingPriceTag,
    NotPrinted,
    Printed,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelCounts {
    pub total: i64,
    pub has_price: i64,
    pub missing_price: i64,
    pub missing_price_tag: i64,
    pub printed: i64,
    pub not_printed: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelProduct {
    pub product_id: String,
    pub sku: Option<String>,
    pub title: String,
    pub product_type: Option<String>,
    pub price_minor: Option<i64>,
    pub state: LabelState,
    pub thumb_url: Option<String>,
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelProductPage {
    pub products: Vec<LabelProduct>,
    pub next_cursor: Option<String>,
    pub counts: LabelCounts,
    /// The provider's `YYYY-MM-DD`, passed through.
    pub work_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelTemplate {
    pub id: String,
    pub name: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub is_default: bool,
    pub product_types: Vec<String>,
}

/// What generating will do to one product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckOutcome {
    MarkPrinted,
    Reprint,
    NoPrice,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanTemplate {
    pub id: String,
    pub name: String,
    pub width_mm: f64,
    pub height_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanItem {
    pub position: usize,
    pub product_id: String,
    pub sku: Option<String>,
    pub title: Option<String>,
    pub product_type: Option<String>,
    pub template: Option<PlanTemplate>,
    pub price_minor: Option<i64>,
    pub status: LabelStatus,
    pub printed_at: Option<i64>,
    pub outcome: CheckOutcome,
    /// The messages as given.
    pub issues: Vec<String>,
    /// The messages as given.
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unresolved {
    pub product_id: Option<String>,
    pub sku: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanSummary {
    pub items: i64,
    pub ready: i64,
    pub failing: i64,
    pub mark_printed: i64,
    pub reprint: i64,
    pub no_price: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelSize {
    pub width_mm: f64,
    pub height_mm: f64,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelPlan {
    pub items: Vec<PlanItem>,
    pub unresolved: Vec<Unresolved>,
    pub summary: PlanSummary,
    pub sizes: Vec<LabelSize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Rendered,
    Partial,
    Failed,
}

/// What a run did to one product's label state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunResult {
    MarkedPrinted,
    Reprinted,
    NoPrice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunItem {
    pub product_id: String,
    pub sku: Option<String>,
    pub title: Option<String>,
    pub rendered: bool,
    pub result: Option<RunResult>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunCounts {
    pub items: i64,
    pub rendered: i64,
    pub failed: i64,
    pub marked_printed: i64,
    pub reprinted: i64,
    pub no_price: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelArtifact {
    pub id: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub page_count: i64,
    pub filename: String,
    pub size_bytes: i64,
    /// A path on the provider's server (never an address).
    pub download_url: String,
    pub expired: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelRun {
    pub id: String,
    pub status: RunStatus,
    pub created_at: i64,
    pub counts: RunCounts,
    pub artifacts: Vec<LabelArtifact>,
    pub items: Vec<RunItem>,
    pub unresolved: Vec<Unresolved>,
}

/// A label change the provider's event log reports (MC-L4.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelChange {
    pub product_id: String,
    pub at: i64,
    pub kind: LabelChangeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelChangeKind {
    Printed { price_minor: Option<i64> },
    Reset { reason: Option<ResetReason> },
}

/// One session with its product's label state (MC-L3.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionLabel {
    pub session_id: i64,
    pub product_id: Option<String>,
    pub state: Option<LabelState>,
}

/// What a labels provider answers when a call fails (master plan §4 Errors).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LabelsError {
    Unreachable(String),
    BadReply(String),
    NotAllowed,
    StoreGone,
    NoProducts,
    RendererUnavailable,
    ListUnavailable,
    Expired,
    Rejected { code: String, message: String },
}

impl std::fmt::Display for LabelsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreachable(detail) => write!(f, "unreachable: {detail}"),
            Self::BadReply(detail) => write!(f, "bad reply: {detail}"),
            Self::NotAllowed => f.write_str("not allowed"),
            Self::StoreGone => f.write_str("store gone"),
            Self::NoProducts => f.write_str("no products"),
            Self::RendererUnavailable => f.write_str("renderer unavailable"),
            Self::ListUnavailable => f.write_str("list unavailable"),
            Self::Expired => f.write_str("expired"),
            Self::Rejected { code, .. } => write!(f, "rejected: {code}"),
        }
    }
}

/// L2 (MC-L1.3): one local item as the matching rule sees it: its SKU, its commerce link's
/// external id and its latest session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkedItem {
    pub item_id: i64,
    pub sku: String,
    pub external_id: Option<String>,
    pub session_id: Option<i64>,
}

/// L2 (MC-L1.2): one row of the Labels box, in the provider's order. `session` is the matched
/// session's own row (10H §4); `picture_path` is an existing cached picture of an unmatched row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelRow {
    pub product_id: String,
    pub commerce_id: Option<String>,
    pub sku: Option<String>,
    pub title: String,
    pub product_type: Option<String>,
    pub price_minor: Option<i64>,
    pub state: LabelState,
    pub created_at: Option<i64>,
    pub session: Option<super::session::SessionRow>,
    pub picture_path: Option<std::path::PathBuf>,
}

/// L2 (MC-L1.1): one page of the Labels box.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelPage {
    pub rows: Vec<LabelRow>,
    pub next_cursor: Option<String>,
    pub counts: LabelCounts,
    pub work_from: Option<String>,
}

/// L2 (MC-L1.7): one product Select all chose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selected {
    pub product_id: String,
    pub sku: String,
}

/// L3 (MC-L2.4): one group of the check, by template in first-appearance order; the items
/// without a template form one last group (`template` none).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckGroup {
    pub template: Option<PlanTemplate>,
    /// The template's product types (empty when unknown).
    pub product_types: Vec<String>,
    pub items: Vec<PlanItem>,
}

/// L3 (MC-L2.1–.4): what generating would do, as shown in the check dialog. `id` names the one
/// kept check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelCheck {
    pub id: String,
    pub groups: Vec<CheckGroup>,
    pub unresolved: Vec<Unresolved>,
    pub summary: PlanSummary,
    pub sizes: Vec<LabelSize>,
}

/// L3 (MC-L2.9): one product that was not generated, with its message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelFailure {
    pub product_id: Option<String>,
    pub sku: Option<String>,
    pub title: Option<String>,
    pub message: String,
}

/// L3 (MC-L2.9): the result of a generation. Fully generated ⇔ `failures` and `not_run` are
/// both empty (M-L4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelRunResult {
    /// The check's items (those that failed the check included).
    pub total: i64,
    pub rendered: i64,
    pub marked_printed: i64,
    pub reprinted: i64,
    pub no_price: i64,
    pub failures: Vec<LabelFailure>,
    /// The products of chunks that have no outcome yet.
    pub not_run: Vec<String>,
    pub artifacts: Vec<LabelArtifact>,
    pub failed_product_ids: Vec<String>,
}

/// L3 (MC-L2.11): a label file saved in the labels folder, and whether it was opened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedLabel {
    pub path: std::path::PathBuf,
    pub opened: bool,
}
