//! Response structs for the Admin GraphQL documents in `graphql.rs` (only the fields read).

use serde::Deserialize;
use serde_json::Value as Json;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    #[serde(default)]
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TypeName {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefinitionNode {
    pub id: String,
    pub namespace: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_: TypeName,
    #[serde(default)]
    pub validations: Json,
    #[serde(default)]
    pub access: Json,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefinitionsPage {
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
    pub nodes: Vec<DefinitionNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Nodes<T> {
    pub nodes: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BarcodeNode {
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InventoryItem {
    pub sku: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariantNode {
    pub id: String,
    pub sku: Option<String>,
    pub barcodes: Option<Nodes<BarcodeNode>>,
    pub inventory_item: Option<InventoryItem>,
    pub product: Option<ProductRef>,
}

impl VariantNode {
    pub fn barcode(&self) -> Option<String> {
        self.barcodes
            .as_ref()
            .and_then(|b| b.nodes.first())
            .map(|b| b.value.clone())
    }

    pub fn sku(&self) -> Option<String> {
        self.inventory_item
            .as_ref()
            .and_then(|i| i.sku.clone())
            .or_else(|| self.sku.clone())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UrlHolder {
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreviewImage {
    pub image: Option<UrlHolder>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductRef {
    pub id: String,
    pub title: String,
    pub featured_media: Option<FeaturedMedia>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeaturedMedia {
    pub preview: Option<PreviewImage>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaNode {
    pub id: String,
    pub status: Option<String>,
    pub image: Option<UrlHolder>,
    pub original_source: Option<UrlHolder>,
}

impl MediaNode {
    /// The stored filename: the last path segment of the CDN image URL, else of the original
    /// source URL, without the query string (SV-2). `None` while neither is exposed.
    pub fn filename(&self) -> Option<String> {
        [self.image.as_ref(), self.original_source.as_ref()]
            .into_iter()
            .flatten()
            .filter_map(|u| u.url.as_deref())
            .find_map(filename_of_url)
    }
}

pub fn filename_of_url(url: &str) -> Option<String> {
    let path = url.split(['?', '#']).next()?;
    let name = path.rsplit('/').next()?;
    (!name.is_empty()).then(|| name.to_owned())
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProductNode {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, rename = "descriptionHtml")]
    pub description_html: Option<String>,
    pub variants: Nodes<VariantNode>,
    pub media: Nodes<MediaNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetafieldNode {
    pub namespace: String,
    pub key: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetafieldsPage {
    pub page_info: PageInfo,
    pub nodes: Vec<MetafieldNode>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StagedTargetNode {
    pub url: String,
    pub resource_url: String,
    #[serde(default)]
    pub parameters: Vec<NameValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NameValue {
    pub name: String,
    pub value: String,
}
