//! The labels ports (book 09C §3; `master_plan_labels.md` §4 Ports). A labels provider answers
//! label state, label lists and label files in the core vocabulary (H7); core obtains a sentence
//! for a failure only through `describe`, and the two id forms of a product only through
//! `product_id_of` / `commerce_id_of`.

use crate::domain::label::{LabelFilter, LabelProductPage, LabelState, LabelTemplate, LabelsError};

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
    /// The sentence shown for `error` (pure, no request); `shop` fills the store-gone sentence.
    fn describe(&self, error: &LabelsError, shop: Option<&str>) -> String;
    /// The provider's product id of a commerce product's external id (pure); none when it is not
    /// one.
    fn product_id_of(&self, commerce_external_id: &str) -> Option<String>;
    /// The commerce external id of a provider product id (pure); none when it is not one.
    fn commerce_id_of(&self, product_id: &str) -> Option<String>;
}
