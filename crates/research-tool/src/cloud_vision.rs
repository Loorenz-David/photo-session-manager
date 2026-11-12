//! Google Cloud Vision Web Detection (`docs.cloud.google.com/vision/docs/detecting-web`): the
//! image bytes inline (base64), the key in the `x-goog-api-key` header.

use crate::{
    recording::{Call, OutBody, free, image_marker},
    serpapi::Hit,
};
use serde_json::{Value as Json, json};
use std::{path::Path, time::Duration};

pub const PROVIDER: &str = "cloud_vision";
pub const URL: &str = "https://vision.googleapis.com/v1/images:annotate";

pub fn web_detection(label: &str, dir: &Path, key: &str, jpeg: Vec<u8>, max_results: u32) -> Call {
    let mut call = Call::new(label, dir, PROVIDER, "cloud_vision", "POST", URL);
    call.headers = vec![("x-goog-api-key".into(), key.to_owned())];
    call.body = OutBody::Json(json!({
        "requests": [{
            "image": {"content": image_marker(0)},
            "features": [{"type": "WEB_DETECTION", "maxResults": max_results}],
        }]
    }));
    call.images = vec![jpeg];
    call.costing = free(json!({"unit": 1}));
    call.timeout = Duration::from_secs(60);
    call
}

/// `pagesWithMatchingImages` in reply order; a page with `fullMatchingImages` is an exact copy.
pub fn hits(body: &Json) -> Vec<Hit> {
    body["responses"][0]["webDetection"]["pagesWithMatchingImages"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, p)| Hit {
            rank: i as u32 + 1,
            title: strip_tags(p["pageTitle"].as_str().unwrap_or("")),
            url: p["url"].as_str().unwrap_or("").to_owned(),
            exact: p["fullMatchingImages"]
                .as_array()
                .is_some_and(|a| !a.is_empty()),
        })
        .collect()
}

/// Cloud Vision marks matched words in `pageTitle` with `<b>`.
fn strip_tags(text: &str) -> String {
    crate::pages::strip_tags(text)
}
