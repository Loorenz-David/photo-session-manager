//! `VisualSearchProvider` for Google Lens through SerpApi, in the two requests R1 captured
//! (`fixtures/serpapi_lens/upload.json`, `search_cut_to_3.json`; RESEARCH-VERIFY-6, -7): the
//! research copy's bytes uploaded unchanged (multipart, with `api_key`), then the Lens search by
//! the returned `image_id`. No size step: the 1024 copies are well under SerpApi's 500 KB limit
//! (V7); a larger one gets the captured HTTP 400 (`request`).

use std::sync::Arc;

use psm_core::ports::secrets::SecretStore;
use serde_json::{Value as Json, json};

use crate::{
    adapters::http::{
        Body, HttpReply, HttpRequest, Part, REDACTED, ResearchHttp, reply_record, request_record,
        url_encode,
    },
    config::DEFAULT_VISUAL_SEARCH_PROVIDER,
    domain::{CallFailure, FailureKind},
    images::ResearchImage,
    ports::{Exchange, SearchCandidate, SearchReply, VisualSearchProvider},
    service::SEARCH_ACCOUNT,
};

pub const UPLOAD_URL: &str = "https://serpapi.com/image";
pub const SEARCH_URL: &str = "https://serpapi.com/search.json";
/// The interface language of the results (as captured).
pub const HL: &str = "en";
/// The market whose results come back (as captured: the owner's market).
pub const COUNTRY: &str = "se";

pub struct SerpApiLens {
    http: Arc<dyn ResearchHttp>,
    secrets: Arc<dyn SecretStore>,
}

/// An HTTP error status: SerpApi's `error` text, else the body text, else the status.
fn status_failure(reply: &HttpReply) -> CallFailure {
    let body = reply.body_json();
    let text = body["error"]
        .as_str()
        .or_else(|| body.as_str())
        .map(str::to_owned)
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| format!("HTTP {}", reply.status));
    CallFailure {
        sent: true,
        status: Some(reply.status),
        kind: FailureKind::Status,
        quota: false,
        text,
        usage: None,
    }
}

/// `visual_matches` in the reply's own order; a result without a page URL is dropped; ranks
/// 1, 2, … in that order. An absent or empty list gives no candidates.
pub fn candidates(body: &Json) -> Vec<SearchCandidate> {
    body["visual_matches"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| {
            let url = m["link"].as_str().filter(|l| !l.trim().is_empty())?;
            Some((url.to_owned(), m["title"].as_str().unwrap_or("").to_owned()))
        })
        .enumerate()
        .map(|(i, (url, title))| SearchCandidate {
            url,
            rank: u32::try_from(i + 1).unwrap_or(u32::MAX),
            title,
        })
        .collect()
}

/// A12: `visual_matches` for the search window's list, in the reply's order: link, title, the
/// site's name and the match's thumbnail (remote addresses). A result without a link is dropped.
pub fn display_matches(body: &Json) -> Vec<crate::browse::VisualMatch> {
    body["visual_matches"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| {
            let url = m["link"].as_str().filter(|l| !l.trim().is_empty())?;
            let thumbnail = m["thumbnail"]
                .as_str()
                .filter(|t| t.starts_with("https://"))
                .map(str::to_owned);
            Some(crate::browse::VisualMatch {
                url: url.to_owned(),
                title: m["title"].as_str().unwrap_or("").to_owned(),
                source: m["source"]
                    .as_str()
                    .map(str::to_owned)
                    .or_else(|| crate::rules::sites::host(url))
                    .unwrap_or_default(),
                thumbnail,
            })
        })
        .collect()
}

impl SerpApiLens {
    pub fn new(http: Arc<dyn ResearchHttp>, secrets: Arc<dyn SecretStore>) -> Self {
        Self { http, secrets }
    }
}

impl VisualSearchProvider for SerpApiLens {
    fn id(&self) -> String {
        DEFAULT_VISUAL_SEARCH_PROVIDER.to_owned()
    }

    fn search(&self, image: &ResearchImage) -> Result<SearchReply, CallFailure> {
        let key = self
            .secrets
            .get(SEARCH_ACCOUNT)
            .ok()
            .flatten()
            .unwrap_or_default()
            .trim()
            .to_owned();
        let bytes = image.bytes().map_err(|e| CallFailure {
            sent: true,
            status: None,
            kind: FailureKind::Status,
            quota: false,
            text: format!(
                "the research copy of photo {} could not be read: {e}",
                image.photo_id()
            ),
            usage: None,
        })?;
        let upload = HttpRequest::post(
            UPLOAD_URL,
            Body::Multipart(vec![
                Part {
                    name: "image".into(),
                    filename: Some("image.jpg".into()),
                    content_type: Some("image/jpeg".into()),
                    data: bytes,
                },
                Part {
                    name: "api_key".into(),
                    filename: None,
                    content_type: None,
                    data: key.clone().into_bytes(),
                },
            ]),
        );
        let upload_seen = request_record(
            &upload,
            json!({ "multipart": {
                "image": { "image_sha256": image.sha256() },
                "api_key": REDACTED,
            } }),
        );
        let uploaded = self
            .http
            .send(&upload)
            .map_err(|failure| failure.into_call_failure())?;
        if !uploaded.ok() {
            return Err(status_failure(&uploaded));
        }
        let uploaded_body = uploaded.body_json();
        let Some(image_id) = uploaded_body["image_id"].as_str().map(str::to_owned) else {
            return Err(CallFailure {
                sent: true,
                status: Some(uploaded.status),
                kind: FailureKind::Schema,
                quota: false,
                text: "the upload reply has no image_id".to_owned(),
                usage: None,
            });
        };
        let search = HttpRequest::get(&format!(
            "{SEARCH_URL}?engine=google_lens&image_id={}&type=all&hl={HL}&country={COUNTRY}&api_key={}",
            url_encode(&image_id),
            url_encode(&key)
        ));
        let search_seen = request_record(&search, Json::Null);
        let found = self
            .http
            .send(&search)
            .map_err(|failure| failure.into_call_failure())?;
        if !found.ok() {
            return Err(status_failure(&found));
        }
        let body = found.body_json();
        Ok(SearchReply {
            candidates: candidates(&body),
            // Uploads cost no credit (V7): one search credit for the pair.
            usage: Some(vec![("image_search_call".to_owned(), 1)]),
            exchange: Exchange {
                request: json!({ "upload": upload_seen, "search": search_seen }),
                response: json!({
                    "upload": reply_record(&uploaded, uploaded_body),
                    "search": reply_record(&found, body),
                }),
            },
        })
    }
}
