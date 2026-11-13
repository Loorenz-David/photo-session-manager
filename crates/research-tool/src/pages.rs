//! Plain HTTP page reading (V8 route a): one GET per page, redirects followed and reported, at
//! most one request every 2 s per host, the generic client name of `http::USER_AGENT`. Text is
//! extracted by plain tag stripping (a delegated choice of the plan).

use crate::recording::{Call, free};
use serde_json::{Value as Json, json};
use std::{path::Path, time::Duration};

pub const PROVIDER: &str = "page_http";

pub fn get(label: &str, dir: &Path, url: &str) -> Call {
    let mut call = Call::new(label, dir, PROVIDER, "page_fetch", "GET", url);
    call.headers = vec![
        (
            "accept".into(),
            "text/html,application/xhtml+xml;q=0.9,*/*;q=0.8".into(),
        ),
        ("accept-language".into(), "en".into()),
    ];
    call.follow_redirects = true;
    call.polite = true;
    call.timeout = Duration::from_secs(30);
    call.costing = free(json!({"page_fetch": 1}));
    call
}

/// Removes `<script>`, `<style>` and every tag; decodes the common entities; collapses spaces.
pub fn strip_tags(html: &str) -> String {
    let mut text = String::with_capacity(html.len() / 2);
    let lower = html.to_ascii_lowercase();
    let mut i = 0;
    let bytes = html.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'<' {
            for block in ["script", "style", "noscript"] {
                if lower[i..].starts_with(&format!("<{block}")) {
                    let close = format!("</{block}");
                    i = lower[i..].find(&close).map_or(bytes.len(), |p| i + p);
                    break;
                }
            }
            if i >= bytes.len() {
                break;
            }
            match html[i..].find('>') {
                Some(p) => i += p + 1,
                None => break,
            }
            text.push(' ');
        } else {
            let next = html[i..].find('<').map_or(bytes.len(), |p| i + p);
            text.push_str(&html[i..next]);
            i = next;
        }
    }
    let text = text
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#039;", "'")
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">");
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The text of the first `<title>` element.
pub fn title_of(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let open_end = lower[start..].find('>')? + start + 1;
    let close = lower[open_end..].find("</title>")? + open_end;
    Some(strip_tags(&html[open_end..close]))
}

/// What a fetched page offers without running scripts.
pub fn analyse(html: &str) -> Json {
    let title = title_of(html).unwrap_or_default();
    let lower = html.to_ascii_lowercase();
    let og_image = meta_content(html, "og:image");
    let og_title = meta_content(html, "og:title");
    let img_tags = lower.matches("<img").count();
    let json_ld = lower.matches("application/ld+json").count();
    let product_ld =
        lower.contains("\"@type\":\"product\"") || lower.contains("\"@type\": \"product\"");
    let text = strip_tags(html);
    json!({
        "title": title,
        "og_title": og_title,
        "og_image": og_image,
        "img_tags": img_tags,
        "json_ld_blocks": json_ld,
        "product_json_ld": product_ld,
        "text_chars": text.chars().count(),
        "html_bytes": html.len(),
    })
}

/// `<meta property="<name>" content="…">` (either attribute order).
pub fn meta_content(html: &str, name: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let needle = format!("\"{name}\"");
    let mut from = 0;
    while let Some(p) = lower[from..].find(&needle) {
        let at = from + p;
        let start = lower[..at].rfind("<meta")?;
        let end = lower[at..].find('>').map(|e| at + e)?;
        let tag = &html[start..end];
        if let Some(c) = tag.to_ascii_lowercase().find("content=") {
            let rest = &tag[c + 8..];
            let quote = rest.chars().next()?;
            if quote == '"' || quote == '\'' {
                let inner = &rest[1..];
                return inner.find(quote).map(|e| inner[..e].to_owned());
            }
        }
        from = end;
    }
    None
}

/// The body text of a recorded page reply.
pub fn body_text(record: &Json) -> String {
    match &record["response"]["body"] {
        Json::String(s) => s.clone(),
        Json::Null => String::new(),
        other => other.to_string(),
    }
}
