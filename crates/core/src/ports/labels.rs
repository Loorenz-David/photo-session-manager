//! The labels ports (book 09C §3; `master_plan_labels.md` §4 Ports). A labels provider answers
//! label state, label lists and label files in the core vocabulary (H7); core obtains a sentence
//! for a failure only through `describe`, and the two id forms of a product only through
//! `product_id_of` / `commerce_id_of`.

use crate::domain::label::{
    LabelFilter, LabelPlan, LabelProductPage, LabelRun, LabelState, LabelTemplate, LabelsError,
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
