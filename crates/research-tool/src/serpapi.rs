//! SerpApi: image upload (`POST /image`, multipart, 500 KB) and the Google Lens engine
//! (`serpapi.com/google-lens-api`, `serpapi.com/google-lens-upload-an-image`).

use crate::recording::{Call, OutBody, Part, free};
use serde_json::{Value as Json, json};
use std::{path::Path, time::Duration};

pub const PROVIDER: &str = "serpapi_lens";
pub const UPLOAD_URL: &str = "https://serpapi.com/image";
pub const MAX_UPLOAD_BYTES: usize = 500 * 1024;

pub fn upload(label: &str, dir: &Path, key: &str, jpeg: Vec<u8>) -> Call {
    let mut call = Call::new(label, dir, PROVIDER, "serpapi_upload", "POST", UPLOAD_URL);
    call.body = OutBody::Multipart(vec![
        (
            "image".into(),
            Part::Image {
                index: 0,
                filename: "image.jpg".into(),
                content_type: "image/jpeg".into(),
            },
        ),
        ("api_key".into(), Part::Text(key.to_owned())),
    ]);
    call.images = vec![jpeg];
    call.costing = free(json!({"upload": 1}));
    call.timeout = Duration::from_secs(60);
    call
}

/// Google Lens search by an uploaded image id. `hl=en`, `country=se` (the owner's market).
pub fn search(label: &str, dir: &Path, key: &str, image_id: &str, kind: &str) -> Call {
    let url = format!(
        "https://serpapi.com/search.json?engine=google_lens&image_id={}&type={kind}&hl=en&country=se&api_key={key}",
        crate::http::url_encode(image_id)
    );
    let mut call = Call::new(label, dir, PROVIDER, "serpapi_search", "GET", &url);
    call.costing = free(json!({"search": 1}));
    call.timeout = Duration::from_secs(60);
    call
}

/// Account figures (free; no search credit).
pub fn account(label: &str, dir: &Path, key: &str) -> Call {
    let url = format!("https://serpapi.com/account.json?api_key={key}");
    let mut call = Call::new(label, dir, PROVIDER, "serpapi_account", "GET", &url);
    call.timeout = Duration::from_secs(30);
    call
}

/// The `image_id` of an upload reply.
pub fn image_id(body: &Json) -> Option<String> {
    body["image_id"]
        .as_str()
        .or_else(|| body["id"].as_str())
        .map(str::to_owned)
}

/// One ranked result.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub rank: u32,
    pub title: String,
    pub url: String,
    pub exact: bool,
}

/// `visual_matches` in `position` order.
pub fn hits(body: &Json) -> Vec<Hit> {
    let mut hits: Vec<Hit> = body["visual_matches"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, m)| Hit {
            rank: m["position"].as_u64().unwrap_or(i as u64 + 1) as u32,
            title: m["title"].as_str().unwrap_or("").to_owned(),
            url: m["link"].as_str().unwrap_or("").to_owned(),
            exact: m["exact_matches"].as_bool().unwrap_or(false),
        })
        .collect();
    hits.sort_by_key(|h| h.rank);
    hits
}
