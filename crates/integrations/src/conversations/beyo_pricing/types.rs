//! Beyo Pricing's wire types (`openapi-integration.json` `components.schemas`, the package's
//! authority): camelCase, unknown fields ignored (API.md: changes are additive), optional and
//! nullable fields as `Option` with `#[serde(default)]`. The fake Pricing server serialises
//! through these same types (master plan §3 rule 17).

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationStore {
    pub id: String,
    pub name: String,
    pub shop_domain: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationCase {
    pub id: String,
    pub product_id: String,
    /// Required but nullable.
    pub sku: Option<String>,
    pub product_title: String,
    pub topic: String,
    pub recipient: String,
    pub blocking: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub case_id: String,
    pub author: Actor,
    /// Required; null when deleted.
    pub text: Option<String>,
    #[serde(default)]
    pub quote: Option<Quote>,
    #[serde(default)]
    pub reply_to: Option<ReplyRef>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    pub created_at: String,
    #[serde(default)]
    pub edited_at: Option<String>,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    /// `user` (a pricer) or `service` (a service account).
    pub kind: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub is_me: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub field: String,
    pub field_label: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplyRef {
    pub message_id: String,
    pub author_name: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub media_id: String,
    #[serde(default)]
    pub thumb_url: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventPage {
    pub events: Vec<StoreEvent>,
    pub next_cursor: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreEvent {
    pub id: i64,
    #[serde(rename = "type")]
    pub kind: String,
    pub store_id: String,
    #[serde(default)]
    pub product_id: Option<String>,
    #[serde(default)]
    pub case_id: Option<String>,
    #[serde(default)]
    pub actor_id: Option<String>,
    pub data: serde_json::Map<String, Json>,
    pub at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
    #[serde(default)]
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: Option<Json>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationMessageCreate {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachment_media_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_mutation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationCaseUpdate {
    pub status: String,
}
