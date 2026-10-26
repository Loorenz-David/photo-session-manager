//! Loader for the sanitized Shopify fixtures captured by the P12 probe
//! (`crates/integrations/src/commerce/shopify/fixtures/*.json`). Each file is
//! `{"source", "request", "response": {"status", "body"}}`; `source` says whether it was captured
//! or is a declared synthetic edit of a capture.

use psm_integrations::commerce::shopify::transport::HttpReply;
use serde_json::Value as Json;
use std::path::PathBuf;

pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../integrations/src/commerce/shopify/fixtures")
}

#[derive(Debug, Clone)]
pub struct Fixture {
    pub name: String,
    pub source: String,
    pub request: Json,
    pub status: u16,
    pub body: Json,
}

impl Fixture {
    pub fn reply(&self) -> HttpReply {
        HttpReply::new(self.status, self.body.clone())
    }

    /// The GraphQL operation name of the request's document, if it is a GraphQL request.
    pub fn operation(&self) -> Option<String> {
        self.request
            .get("query")
            .and_then(Json::as_str)
            .and_then(operation_name)
    }

    pub fn variables(&self) -> Json {
        self.request.get("variables").cloned().unwrap_or(Json::Null)
    }

    /// `response.body.data`.
    pub fn data(&self) -> &Json {
        &self.body["data"]
    }
}

/// `query Name(…) {…}` / `mutation Name …` → `Name`.
pub fn operation_name(document: &str) -> Option<String> {
    let rest = document
        .trim_start()
        .strip_prefix("query")
        .or_else(|| document.trim_start().strip_prefix("mutation"))?;
    let name: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

pub fn fixture(name: &str) -> Fixture {
    let path = fixtures_dir().join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {}: {e}", path.display()));
    let doc: Json = serde_json::from_str(&text).unwrap_or_else(|e| panic!("fixture {name}: {e}"));
    Fixture {
        name: name.to_owned(),
        source: doc["source"].as_str().unwrap_or_default().to_owned(),
        request: doc["request"].clone(),
        status: doc["response"]["status"].as_u64().unwrap_or(200) as u16,
        body: doc["response"]["body"].clone(),
    }
}
