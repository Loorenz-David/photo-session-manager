//! The HTTP seam of the real adapters (R8·U1 task 1). One request → `{status, headers, body}` or
//! a transport failure. `ReqwestResearchHttp` is the live seam; tests use
//! [`crate::fakes::FakeResearchHttp`], so no adapter test reaches a network.
//!
//! RESEARCH-VERIFY-11 (`master_plan_research.md` §7.4): `is_connect()` is checked first (no byte
//! left, which includes DNS failures and a connect timeout); a timeout after that is `Timeout`;
//! anything else after send is `Reset`. Every error text is formed after `without_url()`, because
//! a vendor key can sit in a query parameter. Redirects are never followed by the client.

use std::{
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    sync::{Arc, Mutex},
    task::{Poll, Waker},
    time::Duration,
};

use serde_json::{Map, Value as Json, json};

use crate::domain::{CallFailure, FailureKind};

/// What a declared secret-bearing part reads in an `Exchange` (MC-A9 layer 1).
pub const REDACTED: &str = "<redacted>";
/// Headers whose values are never put in an `Exchange` (MC-A9 layer 1).
pub const SECRET_HEADERS: [&str; 5] = [
    "authorization",
    "x-api-key",
    "x-goog-api-key",
    "cookie",
    "set-cookie",
];
/// Query parameters whose values are never put in an `Exchange` (MC-A9 layer 1).
pub const SECRET_QUERY: [&str; 2] = ["key", "api_key"];

/// One part of a `multipart/form-data` body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub name: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

/// A request body: none (a plain GET), JSON, or a hand-built multipart body.
#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    Empty,
    Json(Json),
    Multipart(Vec<Part>),
}

/// The boundary of every multipart body (built by hand, as `research-tool/src/recording.rs`).
pub const BOUNDARY: &str = "----psm-research-boundary-7f3a9c";

impl Body {
    pub fn content_type(&self) -> Option<String> {
        match self {
            Body::Empty => None,
            Body::Json(_) => Some("application/json".to_owned()),
            Body::Multipart(_) => Some(format!("multipart/form-data; boundary={BOUNDARY}")),
        }
    }

    /// The bytes that go on the wire.
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            Body::Empty => Vec::new(),
            Body::Json(value) => serde_json::to_vec(value).unwrap_or_default(),
            Body::Multipart(parts) => {
                let mut out = Vec::new();
                for part in parts {
                    out.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
                    let mut disposition =
                        format!("Content-Disposition: form-data; name=\"{}\"", part.name);
                    if let Some(filename) = &part.filename {
                        disposition.push_str(&format!("; filename=\"{filename}\""));
                    }
                    out.extend_from_slice(disposition.as_bytes());
                    out.extend_from_slice(b"\r\n");
                    if let Some(content_type) = &part.content_type {
                        out.extend_from_slice(
                            format!("Content-Type: {content_type}\r\n").as_bytes(),
                        );
                    }
                    out.extend_from_slice(b"\r\n");
                    out.extend_from_slice(&part.data);
                    out.extend_from_slice(b"\r\n");
                }
                out.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
                out
            }
        }
    }

    /// The part named `name` of a multipart body.
    pub fn part(&self, name: &str) -> Option<&Part> {
        match self {
            Body::Multipart(parts) => parts.iter().find(|p| p.name == name),
            _ => None,
        }
    }
}

/// One request as it leaves an adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Body,
    /// At most this many body bytes are read; the reply then holds `limit + 1` bytes, which the
    /// caller reads as "over the limit". `None`: no limit.
    pub limit: Option<u64>,
}

impl HttpRequest {
    pub fn get(url: &str) -> Self {
        Self {
            method: "GET".to_owned(),
            url: url.to_owned(),
            headers: Vec::new(),
            body: Body::Empty,
            limit: None,
        }
    }

    pub fn post(url: &str, body: Body) -> Self {
        Self {
            method: "POST".to_owned(),
            url: url.to_owned(),
            headers: Vec::new(),
            body,
            limit: None,
        }
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }
}

/// One reply.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HttpReply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpReply {
    /// The first header named `name` (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// The body parsed as JSON; a body that is not JSON is its text; an empty body is `null`.
    pub fn body_json(&self) -> Json {
        if self.body.is_empty() {
            return Json::Null;
        }
        serde_json::from_slice(&self.body)
            .unwrap_or_else(|_| Json::String(String::from_utf8_lossy(&self.body).into_owned()))
    }
}

/// A request that produced no reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpFailure {
    /// `false` only when no request byte left.
    pub sent: bool,
    /// `Connect`, `Timeout` or `Reset`.
    pub kind: FailureKind,
    pub text: String,
    /// The address guard refused the host before anything was sent.
    pub blocked: bool,
}

impl HttpFailure {
    /// The `CallFailure` a provider adapter reports for it (MC-A9; the guard is never on for a
    /// provider, but a refusal would be "nothing sent").
    pub fn into_call_failure(self) -> CallFailure {
        CallFailure {
            sent: self.sent && !self.blocked,
            status: None,
            kind: if self.blocked {
                FailureKind::Connect
            } else {
                self.kind
            },
            quota: false,
            text: self.text,
            usage: None,
        }
    }
}

/// The seam every real adapter sends through. Sync: one request, one answer.
pub trait ResearchHttp: Send + Sync {
    fn send(&self, request: &HttpRequest) -> Result<HttpReply, HttpFailure>;
}

/// Which addresses a client may reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressGuard {
    /// Any address (the provider adapters, which call fixed vendor hosts).
    Any,
    /// Never a loopback, private, link-local or unspecified address (the page reader, whose URLs
    /// come from search results).
    PublicOnly,
}

/// True for an address the `PublicOnly` guard lets through.
pub fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            !(v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.octets()[0] == 0)
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_public(IpAddr::V4(v4));
            }
            let first = v6.segments()[0];
            !(v6.is_loopback()
                || v6.is_unspecified()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80)
        }
    }
}

/// The error the guard's resolver gives for a host with no public address.
#[derive(Debug)]
struct Blocked(String);

impl std::fmt::Display for Blocked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} has no public address", self.0)
    }
}

impl std::error::Error for Blocked {}

type LookupSlot = Arc<Mutex<(Option<Result<Vec<SocketAddr>, String>>, Option<Waker>)>>;

/// A system lookup on its own thread (the client's runtime is never blocked by it).
struct Lookup {
    host: String,
    slot: LookupSlot,
}

impl Lookup {
    fn spawn(host: String) -> Self {
        let slot: LookupSlot = Arc::new(Mutex::new((None, None)));
        let (name, inner) = (host.clone(), slot.clone());
        std::thread::spawn(move || {
            let found = (name.as_str(), 0)
                .to_socket_addrs()
                .map(|addrs| addrs.collect::<Vec<_>>())
                .map_err(|e| e.to_string());
            let mut guard = inner.lock().unwrap_or_else(|p| p.into_inner());
            guard.0 = Some(found);
            if let Some(waker) = guard.1.take() {
                waker.wake();
            }
        });
        Self { host, slot }
    }
}

impl std::future::Future for Lookup {
    type Output = Result<reqwest::dns::Addrs, Box<dyn std::error::Error + Send + Sync>>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut guard = self.slot.lock().unwrap_or_else(|p| p.into_inner());
        match guard.0.take() {
            None => {
                guard.1 = Some(cx.waker().clone());
                Poll::Pending
            }
            Some(Err(text)) => Poll::Ready(Err(text.into())),
            Some(Ok(addrs)) => {
                let public: Vec<SocketAddr> =
                    addrs.into_iter().filter(|a| is_public(a.ip())).collect();
                if public.is_empty() {
                    Poll::Ready(Err(Box::new(Blocked(self.host.clone()))))
                } else {
                    Poll::Ready(Ok(Box::new(public.into_iter())))
                }
            }
        }
    }
}

/// The `PublicOnly` resolver: refuses every loopback, private, link-local and unspecified
/// address (IPv4 and IPv6).
struct PublicResolver;

impl reqwest::dns::Resolve for PublicResolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        Box::pin(Lookup::spawn(name.as_str().to_owned()))
    }
}

/// The live seam. The client is built at the first request, never in `new`: reqwest refuses to
/// build a blocking client inside an async context, and the app is rebuilt from async command
/// handlers (a settings save); requests themselves run on the research runner's own thread.
pub struct ReqwestResearchHttp {
    timeout: Duration,
    guard: AddressGuard,
    client: std::sync::OnceLock<Result<reqwest::blocking::Client, String>>,
}

/// The generic client name of every request (V8 politeness rule: no product name).
pub const USER_AGENT: &str = "Mozilla/5.0 (compatible; research/0.1)";

impl ReqwestResearchHttp {
    pub fn new(timeout: Duration, guard: AddressGuard) -> Self {
        Self {
            timeout,
            guard,
            client: std::sync::OnceLock::new(),
        }
    }

    fn client(&self) -> Result<&reqwest::blocking::Client, HttpFailure> {
        self.client
            .get_or_init(|| {
                let mut builder = reqwest::blocking::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .user_agent(USER_AGENT)
                    .timeout(self.timeout);
                if self.guard == AddressGuard::PublicOnly {
                    builder = builder.dns_resolver(Arc::new(PublicResolver));
                }
                builder
                    .build()
                    .map_err(|e| format!("http client: {}", e.without_url()))
            })
            .as_ref()
            .map_err(|text| HttpFailure {
                // Not "offline": a client that cannot be built will not come back by waiting.
                sent: true,
                kind: FailureKind::Reset,
                text: text.clone(),
                blocked: false,
            })
    }

    fn blocked(text: String) -> HttpFailure {
        HttpFailure {
            sent: false,
            kind: FailureKind::Connect,
            text,
            blocked: true,
        }
    }
}

fn map_error(error: reqwest::Error) -> HttpFailure {
    let mut source: Option<&(dyn std::error::Error + 'static)> = std::error::Error::source(&error);
    let mut blocked = false;
    let mut detail = Vec::new();
    while let Some(cause) = source {
        if cause.downcast_ref::<Blocked>().is_some() {
            blocked = true;
        }
        detail.push(cause.to_string());
        source = cause.source();
    }
    let (sent, kind) = if blocked || error.is_connect() {
        (false, FailureKind::Connect)
    } else if error.is_timeout() {
        (true, FailureKind::Timeout)
    } else {
        (true, FailureKind::Reset)
    };
    let mut text = error.without_url().to_string();
    for cause in detail {
        if !text.contains(&cause) {
            text.push_str(": ");
            text.push_str(&cause);
        }
    }
    HttpFailure {
        sent: sent && !blocked,
        kind,
        text,
        blocked,
    }
}

fn read_error(error: std::io::Error) -> HttpFailure {
    HttpFailure {
        sent: true,
        kind: if error.kind() == std::io::ErrorKind::TimedOut {
            FailureKind::Timeout
        } else {
            FailureKind::Reset
        },
        text: format!("reading the reply: {error}"),
        blocked: false,
    }
}

impl ResearchHttp for ReqwestResearchHttp {
    fn send(&self, request: &HttpRequest) -> Result<HttpReply, HttpFailure> {
        let url = url::Url::parse(&request.url).map_err(|e| HttpFailure {
            sent: false,
            kind: FailureKind::Connect,
            text: format!("not a valid address: {e}"),
            blocked: false,
        })?;
        if self.guard == AddressGuard::PublicOnly {
            let literal = match url.host() {
                Some(url::Host::Ipv4(ip)) => Some(IpAddr::V4(ip)),
                Some(url::Host::Ipv6(ip)) => Some(IpAddr::V6(ip)),
                _ => None,
            };
            if literal.is_some_and(|ip| !is_public(ip)) {
                return Err(Self::blocked("the address is not public".to_owned()));
            }
        }
        let method =
            reqwest::Method::from_bytes(request.method.as_bytes()).map_err(|e| HttpFailure {
                sent: false,
                kind: FailureKind::Connect,
                text: format!("method: {e}"),
                blocked: false,
            })?;
        let mut builder = self.client()?.request(method, url);
        let has_type = request
            .headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
        for (name, value) in &request.headers {
            builder = builder.header(name.as_str(), value.as_str());
        }
        if let (false, Some(content_type)) = (has_type, request.body.content_type()) {
            builder = builder.header("content-type", content_type);
        }
        if request.body != Body::Empty {
            builder = builder.body(request.body.bytes());
        }
        let response = builder.send().map_err(map_error)?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_owned(),
                    String::from_utf8_lossy(v.as_bytes()).into_owned(),
                )
            })
            .collect();
        let mut body = Vec::new();
        {
            use std::io::Read;
            match request.limit {
                Some(limit) => response
                    .take(limit.saturating_add(1))
                    .read_to_end(&mut body)
                    .map_err(read_error)?,
                None => {
                    let mut response = response;
                    response.read_to_end(&mut body).map_err(read_error)?
                }
            };
        }
        Ok(HttpReply {
            status,
            headers,
            body,
        })
    }
}

/// Headers as an `Exchange` shows them: names lowercased, repeated names joined, every secret
/// header (and each of `extra`) `<redacted>`.
pub fn headers_json(headers: &[(String, String)], extra: &[&str]) -> Json {
    let mut map = Map::new();
    for (name, value) in headers {
        let name = name.to_ascii_lowercase();
        let secret = SECRET_HEADERS.contains(&name.as_str())
            || extra.iter().any(|e| e.eq_ignore_ascii_case(&name));
        let value = if secret {
            REDACTED.to_owned()
        } else {
            value.clone()
        };
        match map.get_mut(&name) {
            Some(Json::String(existing)) => {
                existing.push_str(", ");
                existing.push_str(&value);
            }
            _ => {
                map.insert(name, Json::String(value));
            }
        }
    }
    Json::Object(map)
}

/// The URL with the value of every secret query parameter `<redacted>`; the rest byte for byte.
pub fn redact_url(url: &str) -> String {
    let Some((base, rest)) = url.split_once('?') else {
        return url.to_owned();
    };
    let (query, fragment) = match rest.split_once('#') {
        Some((q, f)) => (q, Some(f)),
        None => (rest, None),
    };
    let query = query
        .split('&')
        .map(|pair| {
            let name = pair.split_once('=').map_or(pair, |(n, _)| n);
            if SECRET_QUERY.iter().any(|s| name.eq_ignore_ascii_case(s)) {
                format!("{name}={REDACTED}")
            } else {
                pair.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("&");
    match fragment {
        Some(f) => format!("{base}?{query}#{f}"),
        None => format!("{base}?{query}"),
    }
}

/// A request as an `Exchange` shows it, with `body` the adapter's own record of the body.
pub fn request_record(request: &HttpRequest, body: Json) -> Json {
    let mut headers = request.headers.clone();
    if let Some(content_type) = request.body.content_type()
        && !headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"))
    {
        headers.push(("content-type".to_owned(), content_type));
    }
    json!({
        "method": request.method,
        "url": redact_url(&request.url),
        "headers": headers_json(&headers, &[]),
        "body": body,
    })
}

/// A reply as an `Exchange` shows it.
pub fn reply_record(reply: &HttpReply, body: Json) -> Json {
    json!({
        "status": reply.status,
        "headers": headers_json(&reply.headers, &[]),
        "body": body,
    })
}

/// Percent-encoding of everything but unreserved characters (RFC 3986).
pub fn url_encode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
