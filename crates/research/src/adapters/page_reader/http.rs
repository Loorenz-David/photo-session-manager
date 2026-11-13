//! `PageReader` as a plain HTTP download (R8·U3 tasks 1–3; 08B MC-A3 with changelog 11).
//!
//! - **Never `Err`.** Every outcome is a `PageReply`; a page that cannot be read is
//!   `obtained: false` ("could not be read"), with `page_fetch` 1 when a request left and an
//!   empty usage when nothing was sent. A dead listing domain is therefore never an offline hold.
//! - **Redirects** are followed here (the client follows none), at most [`MAX_HOPS`], each hop
//!   checked by the address guard.
//! - **Address guard**, half 1 (here, no lookup): only `http`/`https`, never an IP-literal host or
//!   `localhost`. Half 2 is the `PublicOnly` client's resolver (`adapters::http`); its refusal
//!   (`blocked`) is "could not be read" too. Both apply to the page, every hop and every image.
//! - **Slow hosts.** A host that answered with `powered-by: Shopify`, an `x-shopid` header or a
//!   429 is paced: [`PACE_MS`] before every later request to it. A 429 with `Retry-After: n`,
//!   `n` ≤ the call timeout, waits `n` s and sends that request once more (no pace added).

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use serde_json::{Value as Json, json};

use super::read_html;
use crate::{
    adapters::http::{HttpFailure, HttpReply, HttpRequest, ResearchHttp, headers_json, redact_url},
    config::DEFAULT_PAGE_READER,
    domain::CallFailure,
    gateway::Waiter,
    ports::{Exchange, PageReader, PageReply},
};

/// Redirect hops followed per page; a sixth redirect is "could not be read".
pub const MAX_HOPS: usize = 5;
/// The wait before every request to a paced host.
pub const PACE_MS: u64 = 2_000;
/// Body limit of the page document.
pub const PAGE_LIMIT_BYTES: u64 = 4 * 1024 * 1024;
/// Image requests per page, at most.
pub const MAX_IMAGES: usize = 4;
/// Body limit of one image.
pub const IMAGE_LIMIT_BYTES: u64 = 3 * 1024 * 1024;
/// The request headers of R1's page captures (`fixtures/pages/*.json`).
pub const ACCEPT: &str = "text/html,application/xhtml+xml;q=0.9,*/*;q=0.8";
pub const ACCEPT_LANGUAGE: &str = "en";

pub struct HttpPageReader {
    http: Arc<dyn ResearchHttp>,
    wait: Waiter,
    /// `call_timeout_seconds.other`: the longest `Retry-After` that is waited for.
    retry_after_max_s: u64,
    paced: Mutex<HashSet<String>>,
}

/// The host of an address the reader may request (guard half 1), lowercased.
fn allowed_host(url: &url::Url) -> Option<String> {
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    match url.host()? {
        url::Host::Domain(domain) => {
            let domain = domain.trim_end_matches('.').to_ascii_lowercase();
            (domain != "localhost" && !domain.ends_with(".localhost") && !domain.is_empty())
                .then_some(domain)
        }
        url::Host::Ipv4(_) | url::Host::Ipv6(_) => None,
    }
}

/// The outcome of one guarded, paced request.
enum Fetched {
    Reply(HttpReply),
    Failed(HttpFailure),
    /// The guard refused the address before anything was sent.
    Refused,
}

/// What `read` keeps for the call record (no page body: third-party bytes are not recorded).
#[derive(Default)]
struct Trace {
    sent: bool,
    hops: Vec<Json>,
    last: Json,
}

impl HttpPageReader {
    pub fn new(http: Arc<dyn ResearchHttp>, wait: Waiter, retry_after_max_s: u32) -> Self {
        Self {
            http,
            wait,
            retry_after_max_s: u64::from(retry_after_max_s),
            paced: Mutex::new(HashSet::new()),
        }
    }

    fn is_paced(&self, host: &str) -> bool {
        self.paced.lock().map(|p| p.contains(host)).unwrap_or(false)
    }

    fn note(&self, host: &str, reply: &HttpReply) {
        let shopify = reply
            .header("powered-by")
            .is_some_and(|v| v.trim().eq_ignore_ascii_case("shopify"))
            || reply.header("x-shopid").is_some();
        if (shopify || reply.status == 429)
            && let Ok(mut paced) = self.paced.lock()
        {
            paced.insert(host.to_owned());
        }
    }

    fn send_once(&self, request: &HttpRequest, host: &str, trace: &mut Trace) -> Fetched {
        match self.http.send(request) {
            Ok(reply) => {
                trace.sent = true;
                self.note(host, &reply);
                Fetched::Reply(reply)
            }
            Err(failure) if failure.blocked => Fetched::Refused,
            Err(failure) => {
                trace.sent |= failure.sent;
                Fetched::Failed(failure)
            }
        }
    }

    /// One request through the guard and the pace, with the one `Retry-After` retry.
    fn fetch(&self, request: HttpRequest, trace: &mut Trace) -> Fetched {
        let Some(host) = url::Url::parse(&request.url)
            .ok()
            .as_ref()
            .and_then(allowed_host)
        else {
            return Fetched::Refused;
        };
        if self.is_paced(&host) {
            (self.wait)(PACE_MS);
        }
        let first = self.send_once(&request, &host, trace);
        let Fetched::Reply(reply) = &first else {
            return first;
        };
        if reply.status != 429 {
            return first;
        }
        let wait_s = reply
            .header("retry-after")
            .and_then(|v| v.trim().parse::<u64>().ok())
            .filter(|s| *s <= self.retry_after_max_s);
        match wait_s {
            Some(seconds) => {
                (self.wait)(seconds.saturating_mul(1_000));
                self.send_once(&request, &host, trace)
            }
            None => first,
        }
    }

    fn reply(
        &self,
        trace: Trace,
        page: Option<(String, String, String, Vec<Vec<u8>>)>,
    ) -> PageReply {
        let usage = Some(if trace.sent {
            vec![("page_fetch".to_owned(), 1)]
        } else {
            Vec::new()
        });
        let exchange = Exchange {
            request: json!({
                "method": "GET",
                "headers": {"accept": ACCEPT, "accept-language": ACCEPT_LANGUAGE},
                "hops": trace.hops,
            }),
            response: trace.last,
        };
        match page {
            Some((final_url, title, text, images)) => PageReply {
                obtained: true,
                final_url: Some(final_url),
                title,
                text,
                images,
                usage,
                exchange,
            },
            None => PageReply {
                obtained: false,
                final_url: None,
                title: String::new(),
                text: String::new(),
                images: Vec::new(),
                usage,
                exchange,
            },
        }
    }

    /// The page's images: at most [`MAX_IMAGES`] requests, each JPEG kept (the model adapter
    /// sends every image as `image/jpeg`); a failed image request is left out.
    fn images(&self, base: &url::Url, candidates: &[String], trace: &mut Trace) -> Vec<Vec<u8>> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut out = Vec::new();
        let mut requests = 0;
        for candidate in candidates {
            if requests >= MAX_IMAGES {
                break;
            }
            let Ok(url) = base.join(candidate.trim()) else {
                continue;
            };
            // One picture under several sizes or schemes is fetched once.
            let identity = format!("{}{}", url.host_str().unwrap_or(""), url.path());
            if !seen.insert(identity) || allowed_host(&url).is_none() {
                continue;
            }
            requests += 1;
            let mut request = HttpRequest::get(url.as_str())
                .header("accept", "image/jpeg,image/*;q=0.8")
                .header("accept-language", ACCEPT_LANGUAGE);
            request.limit = Some(IMAGE_LIMIT_BYTES);
            if let Fetched::Reply(reply) = self.fetch(request, trace)
                && reply.ok()
                && !reply.body.is_empty()
                && reply.body.len() as u64 <= IMAGE_LIMIT_BYTES
                && reply.body.starts_with(&[0xFF, 0xD8, 0xFF])
            {
                out.push(reply.body);
            }
        }
        out
    }
}

impl PageReader for HttpPageReader {
    fn id(&self) -> String {
        DEFAULT_PAGE_READER.to_owned()
    }

    fn read(&self, url: &str) -> Result<PageReply, CallFailure> {
        let mut trace = Trace::default();
        let mut current = url.to_owned();
        let mut redirects = 0;
        loop {
            let mut request = HttpRequest::get(&current)
                .header("accept", ACCEPT)
                .header("accept-language", ACCEPT_LANGUAGE);
            request.limit = Some(PAGE_LIMIT_BYTES);
            let reply = match self.fetch(request, &mut trace) {
                Fetched::Reply(reply) => reply,
                Fetched::Failed(failure) => {
                    trace.last = json!({ "url": redact_url(&current), "error": failure.text });
                    return Ok(self.reply(trace, None));
                }
                Fetched::Refused => {
                    trace.last = json!({
                        "url": redact_url(&current),
                        "error": "the address guard refused this address",
                    });
                    return Ok(self.reply(trace, None));
                }
            };
            trace.last = json!({
                "url": redact_url(&current),
                "status": reply.status,
                "headers": headers_json(&reply.headers, &[]),
            });
            let location = reply.header("location").map(str::to_owned);
            if (300..400).contains(&reply.status)
                && let Some(location) = location
            {
                redirects += 1;
                trace
                    .hops
                    .push(json!({ "status": reply.status, "url": redact_url(&current) }));
                let next = url::Url::parse(&current)
                    .and_then(|base| base.join(&location))
                    .map(|u| u.to_string());
                match next {
                    Ok(next) if redirects <= MAX_HOPS => {
                        current = next;
                        continue;
                    }
                    _ => return Ok(self.reply(trace, None)),
                }
            }
            if !reply.ok() || reply.body.len() as u64 > PAGE_LIMIT_BYTES {
                return Ok(self.reply(trace, None));
            }
            let html = String::from_utf8_lossy(&reply.body);
            let page = read_html(&html);
            if page.text.trim().is_empty() {
                return Ok(self.reply(trace, None));
            }
            let Ok(base) = url::Url::parse(&current) else {
                return Ok(self.reply(trace, None));
            };
            let images = self.images(&base, &page.images, &mut trace);
            return Ok(self.reply(trace, Some((current, page.title, page.text, images))));
        }
    }
}
