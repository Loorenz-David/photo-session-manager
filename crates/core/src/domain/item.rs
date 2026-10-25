use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: i64,
    pub sku: String,
    pub folder_key: String,
    pub barcode: Option<String>,
    pub item_type: String,
    pub title: String,
    pub origin: ItemOrigin,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemOrigin {
    Linked,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemLink {
    pub item_id: i64,
    pub integration: String,
    pub external_id: String,
    pub external_url: Option<String>,
    pub last_synced_at: Option<i64>,
    pub create_pending: bool,
    pub tags_base: Option<String>,
}

pub struct NewItem<'a> {
    pub sku: &'a str,
    pub folder_key: &'a str,
    pub barcode: Option<&'a str>,
    pub item_type: &'a str,
    pub title: &'a str,
    pub origin: &'a str,
    pub at: i64,
}
