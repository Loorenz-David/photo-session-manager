//! The three provider ports and their neutral shapes (`master_plan_research.md` §6.6). Every
//! string naming a provider, a model or an effort is opaque here (HA5). Images reach a provider
//! only as a [`ResearchImage`] (MC-A5 type guard).

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

pub use crate::domain::CallFailure;
use crate::domain::Stage;
pub use crate::images::ResearchImage;

/// What an adapter hands back for the call record: the request as it went out and the response
/// as it came back, in the adapter's own words (secret-bearing parts already `<redacted>`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Exchange {
    pub request: Json,
    pub response: Json,
}

/// One structured request to the model provider.
#[derive(Debug, Clone)]
pub struct ModelRequest {
    pub stage: Stage,
    pub unit_key: String,
    pub model: String,
    pub effort: String,
    /// The language reasons, trace text and free-text values are written in (`research.language`).
    pub language: String,
    pub instructions: String,
    /// The stage's structured input (ours).
    pub input: Json,
    /// Our photos, as research copies only.
    pub images: Vec<ResearchImage>,
    /// Third-party listing images, by the SHA-256 of their stored file (never our photos).
    pub listing_images: Vec<String>,
}

/// A model answer: the stage's structured output (ours) and the usage the provider reported.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelReply {
    pub output: Json,
    pub usage: Option<Vec<(String, i64)>>,
    pub exchange: Exchange,
}

/// One visual match: the page URL, its rank (1 best) and title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchCandidate {
    pub url: String,
    pub rank: u32,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchReply {
    pub candidates: Vec<SearchCandidate>,
    pub usage: Option<Vec<(String, i64)>>,
    pub exchange: Exchange,
}

/// One page read. `obtained` false: the page could not be read (it does not count, MC-A3).
#[derive(Debug, Clone, PartialEq)]
pub struct PageReply {
    pub obtained: bool,
    /// The URL whose content was read, after redirects, when the reader reports it.
    pub final_url: Option<String>,
    pub title: String,
    pub text: String,
    /// The page's main images (third-party bytes; stored, never outbound rows).
    pub images: Vec<Vec<u8>>,
    pub usage: Option<Vec<(String, i64)>>,
    pub exchange: Exchange,
    /// The site refused automated reading (HTTP 403, or 429 with no short `Retry-After`): the run
    /// reads nothing more from that host (owner correction A9, 2026-11-18).
    pub blocked: bool,
    /// Why the page could not be read, in plain words, when the reader knows (shown under the
    /// step); `None` for a read page.
    pub note: Option<String>,
}

pub trait ModelProvider: Send + Sync {
    /// The provider id used in call records and the price table.
    fn id(&self) -> String;
    fn call(&self, request: &ModelRequest) -> Result<ModelReply, CallFailure>;
}

pub trait VisualSearchProvider: Send + Sync {
    fn id(&self) -> String;
    fn search(&self, image: &ResearchImage) -> Result<SearchReply, CallFailure>;
}

pub trait PageReader: Send + Sync {
    fn id(&self) -> String;
    fn read(&self, url: &str) -> Result<PageReply, CallFailure>;
}

/// The adapters a run calls; `None` when no adapter is configured for that port.
#[derive(Clone, Default)]
pub struct Providers {
    pub model: Option<Arc<dyn ModelProvider>>,
    pub search: Option<Arc<dyn VisualSearchProvider>>,
    pub reader: Option<Arc<dyn PageReader>>,
}
