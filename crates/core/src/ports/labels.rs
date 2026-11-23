//! The labels ports (book 09C §3; `master_plan_labels.md` §4 Ports). A labels provider answers
//! label state, label lists and label files in the core vocabulary (H7); core obtains a sentence
//! for a failure only through `describe`, and the two id forms of a product only through
//! `product_id_of` / `commerce_id_of`.

use crate::Result;
use crate::domain::label::{
    LabelChange, LabelFilter, LabelPlan, LabelProductPage, LabelRun, LabelState, LabelTemplate,
    LabelsAccess, LabelsError, LinkedItem,
};

pub type LabelsResult<T> = std::result::Result<T, LabelsError>;

pub trait LabelsProvider: Send + Sync {
    /// One page of the provider's label list, newest created first.
    fn products(
        &self,
        store_id: &str,
        filter: LabelFilter,
        query: Option<&str>,
        cursor: Option<&str>,
        limit: u32,
    ) -> LabelsResult<LabelProductPage>;
    /// One entry per id, in order; `None` = the provider does not know the product. At most
    /// `STATUS_CHUNK` ids; no request for an empty list.
    fn status(
        &self,
        store_id: &str,
        product_ids: &[String],
    ) -> LabelsResult<Vec<Option<LabelState>>>;
    /// The active label templates.
    fn templates(&self, store_id: &str) -> LabelsResult<Vec<LabelTemplate>>;
    /// What generating would do (changes nothing); at most `RUN_CHUNK` ids.
    fn plan(&self, store_id: &str, product_ids: &[String]) -> LabelsResult<LabelPlan>;
    /// One label as the provider's file (changes nothing).
    fn preview(&self, store_id: &str, product_id: &str) -> LabelsResult<Vec<u8>>;
    /// Generates the labels of `product_ids` under the idempotency `key`. **The only writing
    /// method** of the port.
    fn run(&self, store_id: &str, product_ids: &[String], key: &str) -> LabelsResult<LabelRun>;
    /// A file of a run, by the path the run named.
    fn download(&self, store_id: &str, download_url: &str) -> LabelsResult<Vec<u8>>;
    /// The sentence shown for `error` (pure, no request); `shop` fills the store-gone sentence.
    fn describe(&self, error: &LabelsError, shop: Option<&str>) -> String;
    /// The provider's product id of a commerce product's external id (pure); none when it is not
    /// one.
    fn product_id_of(&self, commerce_external_id: &str) -> Option<String>;
    /// The commerce external id of a provider product id (pure); none when it is not one.
    fn commerce_id_of(&self, product_id: &str) -> Option<String>;
}

/// What labels store on this Mac (`022_labels_access.sql`): whether the saved key may read
/// labels, and the reads that join sessions to their products. Nothing about label state.
pub trait LabelStore: Send + Sync {
    /// The stored access (`Unknown`, `Allowed` or `Denied`; never `Off`).
    fn labels_access(&self) -> Result<LabelsAccess>;
    /// Stores the access (`Off` is stored as `Unknown`).
    fn set_labels_access(&self, access: LabelsAccess) -> Result<()>;
    /// Each session with its item's commerce link external id (none without a link), in the
    /// order asked; an unknown session is left out.
    fn session_products(&self, session_ids: &[i64]) -> Result<Vec<(i64, Option<String>)>>;
    /// L2 (MC-L1.3): every item with its commerce link's external id (none without a link) and
    /// its latest session (none without one), by item id; one read.
    fn linked_items(&self) -> Result<Vec<LinkedItem>>;
}

/// What the conversations service tells the labels service (ACC.2, ACC.3, MC-L4.3).
pub trait LabelsLink: Send + Sync {
    /// A connect ended `Ready` (Test connection, Save key), after the lane was left.
    fn connected(&self);
    /// The key is about to be saved or removed.
    fn key_changed(&self);
    /// The label changes of one pull page, in log order (never empty).
    fn changed(&self, changes: &[LabelChange]);
}
