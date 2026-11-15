//! Every exchange is recorded in the export layout of MC-A11 (`<dir>/calls/<call id>.json`,
//! `<dir>/images/<sha256>.jpg`) with the MC-A9 record fields. Declared secret-bearing parts read
//! `"<redacted>"`; before anything is written the text is scanned for every key value, plain and
//! URL-encoded, and a hit stops the write (fail closed) naming the call id, never the value.
//!
//! `Caller` is the one path a request takes: resume check (same request hash → the recorded call
//! is returned and nothing leaves), budget check, send, record.

use crate::{
    budget::{Ledger, Limits, Refusal},
    http::{HttpSeam, Wire, WireReply, base64, url_encode},
};
use serde_json::{Map, Value as Json, json};
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub const REDACTED: &str = "<redacted>";
/// Headers whose values are never recorded (MC-A9 layer 1, plus the Shopify token header).
pub const SECRET_HEADERS: [&str; 6] = [
    "authorization",
    "x-api-key",
    "x-goog-api-key",
    "cookie",
    "set-cookie",
    "x-shopify-access-token",
];
/// Query parameters whose values are never recorded.
pub const SECRET_QUERY: [&str; 2] = ["key", "api_key"];
/// Body fields (JSON keys, form and multipart fields) whose values are never recorded.
pub const SECRET_FIELDS: [&str; 3] = ["api_key", "client_secret", "access_token"];
/// Marker a provider module puts where an image's base64 goes in a JSON body.
pub fn image_marker(index: usize) -> String {
    format!("@@IMAGE:{index}@@")
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// The body of a request, before images are inlined.
#[derive(Debug, Clone)]
pub enum OutBody {
    Empty,
    /// A JSON body; every string holding `image_marker(i)` gets image `i`'s base64 in place of
    /// the marker when sent, and is recorded as `{"image_sha256": …}`.
    Json(Json),
    /// `multipart/form-data`.
    Multipart(Vec<(String, Part)>),
    /// `application/x-www-form-urlencoded`.
    Form(Vec<(String, String)>),
}

#[derive(Debug, Clone)]
pub enum Part {
    Text(String),
    Image {
        index: usize,
        filename: String,
        content_type: String,
    },
}

/// How a reply is priced: `(usage, cost_micro_usd)` from the status and the parsed body.
pub type Costing = Box<dyn Fn(u16, &Json) -> (Json, i64)>;

pub fn free(usage: Json) -> Costing {
    Box::new(move |_, _| (usage.clone(), 0))
}

/// One call to make.
pub struct Call {
    /// Prefix of the call id (`<label>-<first 12 hex of the request hash>`).
    pub label: String,
    /// Folder holding `calls/` and `images/`.
    pub dir: PathBuf,
    pub provider: String,
    /// Which call cap this call counts against.
    pub cap: String,
    pub model: String,
    pub effort: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: OutBody,
    pub images: Vec<Vec<u8>>,
    pub worst_case_micro: i64,
    pub timeout: Duration,
    pub follow_redirects: bool,
    /// At most one request every 2 s to this host (page reading).
    pub polite: bool,
    pub costing: Costing,
    /// Recorded as `meta` (benchmark bookkeeping: stage, piece, unit); not part of the request hash.
    pub meta: Json,
}

impl Call {
    pub fn new(
        label: &str,
        dir: &Path,
        provider: &str,
        cap: &str,
        method: &str,
        url: &str,
    ) -> Self {
        Self {
            label: label.to_owned(),
            dir: dir.to_owned(),
            provider: provider.to_owned(),
            cap: cap.to_owned(),
            model: String::new(),
            effort: String::new(),
            method: method.to_owned(),
            url: url.to_owned(),
            headers: Vec::new(),
            body: OutBody::Empty,
            images: Vec::new(),
            worst_case_micro: 0,
            timeout: Duration::from_secs(60),
            follow_redirects: false,
            polite: false,
            costing: free(json!({})),
            meta: Json::Null,
        }
    }
}

/// A recorded call (fresh or found on disk).
#[derive(Debug, Clone)]
pub struct Recorded {
    pub call_id: String,
    pub file: PathBuf,
    pub from_disk: bool,
    pub record: Json,
    /// The reply body before field redaction (fresh calls only; never written anywhere).
    pub live_body: Option<Json>,
}

impl Recorded {
    pub fn status(&self) -> Option<u16> {
        self.record["response"]["status"].as_u64().map(|s| s as u16)
    }
    pub fn body(&self) -> &Json {
        &self.record["response"]["body"]
    }
    pub fn ok(&self) -> bool {
        matches!(self.status(), Some(200..=299))
    }
}

#[derive(Debug)]
pub enum ToolError {
    Refused(Refusal),
    /// A key value was found in what was about to be written; nothing was written.
    SecretInRecording {
        call_id: String,
    },
    Io(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::Refused(r) => write!(f, "{r}"),
            ToolError::SecretInRecording { call_id } => write!(
                f,
                "call {call_id}: a key value was found in the recording; nothing was written"
            ),
            ToolError::Io(t) => write!(f, "{t}"),
        }
    }
}

impl From<std::io::Error> for ToolError {
    fn from(e: std::io::Error) -> Self {
        ToolError::Io(e.to_string())
    }
}

/// Every `calls/*.json` under `out`.
pub fn call_files(out: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![out.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "json")
                && path
                    .parent()
                    .and_then(Path::file_name)
                    .is_some_and(|n| n == "calls")
            {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// Redacts the value of every declared query parameter, keeping the rest of the URL byte for byte.
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

fn headers_json(headers: &[(String, String)]) -> Json {
    let mut map = Map::new();
    for (name, value) in headers {
        let name = name.to_ascii_lowercase();
        let value = if SECRET_HEADERS.contains(&name.as_str()) {
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

/// Replaces every declared secret field's value, at any depth.
pub fn redact_fields(value: &mut Json) {
    match value {
        Json::Object(map) => {
            for (k, v) in map.iter_mut() {
                if SECRET_FIELDS.iter().any(|s| k.eq_ignore_ascii_case(s)) {
                    *v = Json::String(REDACTED.into());
                } else {
                    redact_fields(v);
                }
            }
        }
        Json::Array(items) => items.iter_mut().for_each(redact_fields),
        _ => {}
    }
}

fn replace_markers(value: &Json, images: &[Vec<u8>], record: bool) -> Json {
    match value {
        Json::String(s) if s.contains("@@IMAGE:") => {
            let index = s
                .split("@@IMAGE:")
                .nth(1)
                .and_then(|t| t.split("@@").next())
                .and_then(|n| n.parse::<usize>().ok());
            match index.and_then(|i| images.get(i).map(|b| (i, b))) {
                Some((i, bytes)) if record => {
                    let _ = i;
                    json!({"image_sha256": sha256_hex(bytes)})
                }
                Some((i, bytes)) => Json::String(s.replace(&image_marker(i), &base64(bytes))),
                None => value.clone(),
            }
        }
        Json::Object(map) => Json::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), replace_markers(v, images, record)))
                .collect(),
        ),
        Json::Array(items) => Json::Array(
            items
                .iter()
                .map(|v| replace_markers(v, images, record))
                .collect(),
        ),
        other => other.clone(),
    }
}

const BOUNDARY: &str = "----research-bench-boundary-7f3a9c";

/// `(bytes to send, content type, body as recorded)`.
fn materialize(body: &OutBody, images: &[Vec<u8>]) -> (Vec<u8>, Option<String>, Json) {
    match body {
        OutBody::Empty => (Vec::new(), None, Json::Null),
        OutBody::Json(value) => {
            let sent = replace_markers(value, images, false);
            (
                serde_json::to_vec(&sent).unwrap_or_default(),
                Some("application/json".into()),
                replace_markers(value, images, true),
            )
        }
        OutBody::Form(fields) => {
            let text = fields
                .iter()
                .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            let record: Map<String, Json> = fields
                .iter()
                .map(|(k, v)| (k.clone(), Json::String(v.clone())))
                .collect();
            (
                text.into_bytes(),
                Some("application/x-www-form-urlencoded".into()),
                json!({ "form": record }),
            )
        }
        OutBody::Multipart(parts) => {
            let mut bytes = Vec::new();
            let mut record = Map::new();
            for (name, part) in parts {
                bytes.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
                match part {
                    Part::Text(text) => {
                        bytes.extend_from_slice(
                            format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n")
                                .as_bytes(),
                        );
                        bytes.extend_from_slice(text.as_bytes());
                        record.insert(name.clone(), Json::String(text.clone()));
                    }
                    Part::Image {
                        index,
                        filename,
                        content_type,
                    } => {
                        let data = images.get(*index).cloned().unwrap_or_default();
                        bytes.extend_from_slice(
                            format!(
                                "Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
                            )
                            .as_bytes(),
                        );
                        record.insert(name.clone(), json!({"image_sha256": sha256_hex(&data)}));
                        bytes.extend_from_slice(&data);
                    }
                }
                bytes.extend_from_slice(b"\r\n");
            }
            bytes.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
            (
                bytes,
                Some(format!("multipart/form-data; boundary={BOUNDARY}")),
                json!({ "multipart": record }),
            )
        }
    }
}

fn parse_body(bytes: &[u8]) -> Json {
    if bytes.is_empty() {
        return Json::Null;
    }
    serde_json::from_slice(bytes)
        .unwrap_or_else(|_| Json::String(String::from_utf8_lossy(bytes).into_owned()))
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// The secret values a recording is scanned for.
#[derive(Debug, Clone, Default)]
pub struct Secrets {
    values: Vec<String>,
}

impl Secrets {
    pub fn new(values: impl IntoIterator<Item = String>) -> Self {
        let mut s = Self::default();
        for v in values {
            s.add(&v);
        }
        s
    }

    /// Values shorter than 8 characters are not scanned for (they would match ordinary text).
    pub fn add(&mut self, value: &str) {
        let value = value.trim();
        if value.len() >= 8 && !self.values.iter().any(|v| v == value) {
            self.values.push(value.to_owned());
        }
    }

    /// The well-known key variables of the environment.
    pub fn from_env() -> Self {
        Self::new(
            [
                "OPENAI_API_KEY",
                "SERPAPI_API_KEY",
                "GOOGLE_VISION_API_KEY",
                "SHOPIFY_CLIENT_SECRET",
            ]
            .iter()
            .filter_map(|n| std::env::var(n).ok()),
        )
    }

    /// True when `bytes` contains any value, plain or URL-encoded.
    pub fn found_in(&self, bytes: &[u8]) -> bool {
        self.values.iter().any(|v| {
            let encoded = url_encode(v);
            let form = encoded.replace("%20", "+");
            [v.as_str(), encoded.as_str(), form.as_str()]
                .iter()
                .any(|needle| contains(bytes, needle.as_bytes()))
        })
    }
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && hay.windows(needle.len()).any(|w| w == needle)
}

/// Writes one file after the secret scan (temp file, then rename).
pub fn write_scanned(secrets: &Secrets, path: &Path, bytes: &[u8]) -> Result<(), ToolError> {
    if secrets.found_in(bytes) {
        return Err(ToolError::SecretInRecording {
            call_id: path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default(),
        });
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, bytes)?;
    fs::rename(&temp, path)?;
    Ok(())
}

pub struct Caller<'a> {
    seam: &'a dyn HttpSeam,
    pub secrets: RefCell<Secrets>,
    pub ledger: RefCell<Ledger>,
    pub limits: Limits,
    last_by_host: RefCell<HashMap<String, Instant>>,
    /// Requests that actually left (resume skips do not count).
    pub sent: RefCell<u32>,
}

impl<'a> Caller<'a> {
    pub fn new(seam: &'a dyn HttpSeam, secrets: Secrets, ledger: Ledger, limits: Limits) -> Self {
        Self {
            seam,
            secrets: RefCell::new(secrets),
            ledger: RefCell::new(ledger),
            limits,
            last_by_host: RefCell::new(HashMap::new()),
            sent: RefCell::new(0),
        }
    }

    pub fn call(&self, call: Call) -> Result<Recorded, ToolError> {
        let (bytes, content_type, record_body) = materialize(&call.body, &call.images);
        let mut record_body = record_body;
        redact_fields(&mut record_body);
        let mut headers = call.headers.clone();
        if let Some(ct) = content_type
            && !headers
                .iter()
                .any(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        {
            headers.push(("content-type".into(), ct));
        }
        let request = json!({
            "method": call.method,
            "url": redact_url(&call.url),
            "headers": headers_json(&headers),
            "body": record_body,
        });
        let hash = sha256_hex(
            serde_json::to_string(&json!({
                "method": request["method"], "url": request["url"], "body": request["body"],
                "model": call.model, "effort": call.effort,
            }))
            .unwrap_or_default()
            .as_bytes(),
        );
        let call_id = format!("{}-{}", call.label, &hash[..12]);
        let file = call.dir.join("calls").join(format!("{call_id}.json"));
        if let Ok(text) = fs::read_to_string(&file)
            && let Ok(record) = serde_json::from_str::<Json>(&text)
        {
            if refused_by_provider(&record) {
                // A key, billing, rate-limit or server refusal is tried again on a re-run; the
                // old record moves to `superseded/` (outside `calls/`, so it is not reused).
                let aside = call.dir.join("superseded");
                fs::create_dir_all(&aside)?;
                let stamp = record["started"].as_str().unwrap_or("").replace(':', "-");
                fs::rename(&file, aside.join(format!("{call_id}-{stamp}.json")))?;
            } else {
                return Ok(Recorded {
                    call_id,
                    file,
                    from_disk: true,
                    record,
                    live_body: None,
                });
            }
        }
        self.ledger
            .borrow()
            .check(&self.limits, &call.cap, call.worst_case_micro)
            .map_err(ToolError::Refused)?;
        if call.polite {
            self.wait_for_host(&call.url);
        }
        let wire = Wire {
            method: call.method.clone(),
            url: call.url.clone(),
            headers,
            body: bytes,
            timeout: call.timeout,
            follow_redirects: call.follow_redirects,
        };
        let started = now_iso();
        let clock = Instant::now();
        *self.sent.borrow_mut() += 1;
        let mut result = self.seam.send(&wire);
        let mut waited_after_429 = Json::Null;
        // Page reading honours a 429's `Retry-After` (up to 120 s): wait, then one more try.
        if call.polite
            && let Ok(reply) = &result
            && reply.status == 429
            && let Some(wait) = crate::http::header(&reply.headers, "retry-after")
                .and_then(|v| v.trim().parse::<u64>().ok())
                .filter(|w| *w <= 120)
        {
            std::thread::sleep(Duration::from_secs(wait));
            waited_after_429 = json!(wait);
            *self.sent.borrow_mut() += 1;
            result = self.seam.send(&wire);
            self.wait_for_host(&call.url);
        }
        let duration_ms = clock.elapsed().as_millis() as u64;
        let finished = now_iso();
        let mut live_body = None;
        let (response, usage, cost, error, sent) = match &result {
            Ok(reply) => {
                let body = parse_body(&reply.body);
                live_body = Some(body.clone());
                let (usage, cost) = (call.costing)(reply.status, &body);
                let mut body = body;
                redact_fields(&mut body);
                (response_json(reply, body), usage, cost, Json::Null, true)
            }
            Err(e) => {
                // No reply: a request that left is charged its worst case (no usage reported).
                let cost = if e.sent { call.worst_case_micro } else { 0 };
                (
                    Json::Null,
                    Json::Null,
                    cost,
                    json!({"sent": e.sent, "timed_out": e.timed_out, "text": e.text}),
                    e.sent,
                )
            }
        };
        let record = json!({
            "id": call_id,
            "provider": call.provider,
            "cap": call.cap,
            "model": call.model,
            "effort": call.effort,
            "sent": sent,
            "request": request,
            "response": response,
            "usage": usage,
            "cost_micro_usd": cost,
            "worst_case_micro_usd": call.worst_case_micro,
            "started": started,
            "finished": finished,
            "duration_ms": duration_ms,
            "error": error,
            "request_hash": hash,
            "waited_after_429_s": waited_after_429,
            "meta": call.meta,
        });
        if sent {
            self.ledger.borrow_mut().add(&call.cap, cost);
        }
        self.write(&call.dir, &call_id, &record, &call.images)?;
        Ok(Recorded {
            call_id,
            file,
            from_disk: false,
            record,
            live_body,
        })
    }

    /// Scans the record and every image first; writes nothing on a hit.
    fn write(
        &self,
        dir: &Path,
        call_id: &str,
        record: &Json,
        images: &[Vec<u8>],
    ) -> Result<(), ToolError> {
        let text = serde_json::to_vec_pretty(record).unwrap_or_default();
        let secrets = self.secrets.borrow();
        if secrets.found_in(&text) || images.iter().any(|i| secrets.found_in(i)) {
            return Err(ToolError::SecretInRecording {
                call_id: call_id.to_owned(),
            });
        }
        for image in images {
            let path = dir
                .join("images")
                .join(format!("{}.jpg", sha256_hex(image)));
            if !path.exists() {
                write_scanned(&secrets, &path, image)?;
            }
        }
        write_scanned(
            &secrets,
            &dir.join("calls").join(format!("{call_id}.json")),
            &text,
        )
        .map_err(|e| match e {
            ToolError::SecretInRecording { .. } => ToolError::SecretInRecording {
                call_id: call_id.to_owned(),
            },
            other => other,
        })
    }

    fn wait_for_host(&self, url: &str) {
        let host = url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned))
            .unwrap_or_default();
        let gap = Duration::from_secs(2);
        if let Some(last) = self.last_by_host.borrow().get(&host) {
            let since = last.elapsed();
            if since < gap {
                std::thread::sleep(gap - since);
            }
        }
        self.last_by_host.borrow_mut().insert(host, Instant::now());
    }
}

/// HTTP 401, 403, 429 or 5xx: the provider refused or failed before doing the work.
pub fn refused_by_provider(record: &Json) -> bool {
    matches!(
        record["response"]["status"].as_u64(),
        Some(401 | 403 | 429 | 500..=599)
    )
}

fn response_json(reply: &WireReply, body: Json) -> Json {
    json!({
        "status": reply.status,
        "headers": headers_json(&reply.headers),
        "body": body,
        "final_url": redact_url(&reply.final_url),
        "redirects": reply.redirects.iter().map(|(s, l)| json!({"status": s, "location": redact_url(l)})).collect::<Vec<_>>(),
        "body_bytes": reply.body.len(),
    })
}
