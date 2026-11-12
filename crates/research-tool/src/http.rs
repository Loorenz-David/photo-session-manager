//! The HTTP seam of the tool. `ReqwestSeam` is the live one; tests pass a stub. Errors never carry
//! a URL (`reqwest::Error::without_url`), because a vendor key can sit in a query parameter.

use std::time::Duration;

/// One request as it leaves the tool.
#[derive(Debug, Clone)]
pub struct Wire {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub timeout: Duration,
    /// Follow up to 10 redirects and report each hop (page reading); otherwise none are followed.
    pub follow_redirects: bool,
}

/// One reply.
#[derive(Debug, Clone, Default)]
pub struct WireReply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    /// The URL whose content `body` is (after redirects).
    pub final_url: String,
    /// Each redirect hop: status and the `Location` it pointed to.
    pub redirects: Vec<(u16, String)>,
}

/// A request that produced no reply.
#[derive(Debug, Clone)]
pub struct WireError {
    /// `false` only when no request byte left (connect or DNS failure).
    pub sent: bool,
    pub timed_out: bool,
    pub text: String,
}

pub trait HttpSeam {
    fn send(&self, wire: &Wire) -> Result<WireReply, WireError>;
}

/// The generic client name used for every request (V8 politeness rule: no product name).
pub const USER_AGENT: &str = "Mozilla/5.0 (compatible; research-bench/0.1)";

pub struct ReqwestSeam {
    client: reqwest::blocking::Client,
}

impl ReqwestSeam {
    pub fn new() -> Result<Self, String> {
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| format!("http client: {}", e.without_url()))?;
        Ok(Self { client })
    }

    fn once(&self, wire: &Wire, url: &str, method: &str) -> Result<WireReply, WireError> {
        let method = reqwest::Method::from_bytes(method.as_bytes()).map_err(|e| WireError {
            sent: false,
            timed_out: false,
            text: format!("method: {e}"),
        })?;
        let mut request = self
            .client
            .request(method.clone(), url)
            .timeout(wire.timeout);
        for (name, value) in &wire.headers {
            request = request.header(name.as_str(), value.as_str());
        }
        if !wire.body.is_empty() && method != reqwest::Method::GET {
            request = request.body(wire.body.clone());
        }
        let response = request.send().map_err(map_error)?;
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
        let body = response.bytes().map_err(map_error)?.to_vec();
        Ok(WireReply {
            status,
            headers,
            body,
            final_url: url.to_owned(),
            redirects: Vec::new(),
        })
    }
}

fn map_error(e: reqwest::Error) -> WireError {
    // RESEARCH-VERIFY-11 (master_plan_research §7.4): `is_connect` first, it means no byte left.
    let sent = !e.is_connect();
    let timed_out = e.is_timeout();
    WireError {
        sent,
        timed_out,
        text: e.without_url().to_string(),
    }
}

impl HttpSeam for ReqwestSeam {
    fn send(&self, wire: &Wire) -> Result<WireReply, WireError> {
        if !wire.follow_redirects {
            return self.once(wire, &wire.url, &wire.method);
        }
        let mut url = wire.url.clone();
        let mut redirects = Vec::new();
        for _ in 0..=10 {
            let mut reply = self.once(wire, &url, &wire.method)?;
            if (300..400).contains(&reply.status)
                && let Some(location) = header(&reply.headers, "location")
            {
                let next = url::Url::parse(&url)
                    .and_then(|base| base.join(location))
                    .map(|u| u.to_string())
                    .unwrap_or_else(|_| location.to_owned());
                redirects.push((reply.status, next.clone()));
                url = next;
                continue;
            }
            reply.redirects = redirects;
            reply.final_url = url;
            return Ok(reply);
        }
        Err(WireError {
            sent: true,
            timed_out: false,
            text: "more than 10 redirects".into(),
        })
    }
}

/// First header named `name` (case-insensitive).
pub fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

/// Standard base64 (RFC 4648, with padding).
pub fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Percent-encoding of everything but unreserved characters (RFC 3986).
pub fn url_encode(text: &str) -> String {
    let mut out = String::new();
    for b in text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
