//! The HTTP seam of the Shopify adapter (master plan §6.4): `HttpTransport` (impls
//! `ReqwestTransport` here, testkit `FakeTransport`).
//!
//! `ReqwestTransport` runs an async `reqwest` client on a private worker thread that owns its own
//! current-thread Tokio runtime. Callers block on a plain channel, so creating, using or dropping
//! the transport from inside a Tauri async command never panics (no runtime is entered, started or
//! dropped on the caller's thread). The access token goes in the `X-Shopify-Access-Token` header
//! and is never logged; error messages never carry a URL (signed upload URLs stay out of logs).

use psm_core::{Error, Result};
use serde_json::Value as Json;
use std::{
    sync::{Mutex, mpsc},
    time::Duration,
};

/// One HTTP reply: status and the body parsed as JSON (`Json::Null` for an empty body, a JSON
/// string holding the text for a non-JSON body). `retry_after_s` is the `Retry-After` header.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpReply {
    pub status: u16,
    pub body: Json,
    pub retry_after_s: Option<u64>,
}

impl HttpReply {
    pub fn new(status: u16, body: Json) -> Self {
        Self {
            status,
            body,
            retry_after_s: None,
        }
    }
}

pub trait HttpTransport: Send + Sync {
    /// POST `{"query", "variables"}` to the Admin GraphQL endpoint `url` with the access token in
    /// the `X-Shopify-Access-Token` header.
    fn post_graphql(
        &self,
        url: &str,
        access_token: &str,
        query: &str,
        variables: &Json,
    ) -> Result<HttpReply>;
    /// POST an `application/x-www-form-urlencoded` body (the token request, 09A MC-23).
    fn post_form(&self, url: &str, fields: &[(&str, &str)]) -> Result<HttpReply>;
    /// PUT raw bytes to a staged-upload target; `headers` are the target's `parameters`.
    fn put_bytes(&self, url: &str, headers: &[(String, String)], bytes: &[u8])
    -> Result<HttpReply>;
}

/// `application/x-www-form-urlencoded` encoding of `fields`.
pub fn form_encode(fields: &[(&str, &str)]) -> String {
    fn enc(text: &str) -> String {
        let mut out = String::new();
        for b in text.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(b as char)
                }
                b' ' => out.push('+'),
                _ => out.push_str(&format!("%{b:02X}")),
            }
        }
        out
    }
    fields
        .iter()
        .map(|(k, v)| format!("{}={}", enc(k), enc(v)))
        .collect::<Vec<_>>()
        .join("&")
}

type Task = Box<dyn FnOnce(&tokio::runtime::Runtime, &reqwest::Client) + Send>;

/// The live transport (timeouts 30 s).
pub struct ReqwestTransport {
    tasks: Mutex<mpsc::Sender<Task>>,
}

impl ReqwestTransport {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel::<Task>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<()>>();
        std::thread::Builder::new()
            .name("psm-shopify-http".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = ready_tx.send(Err(Error::Provider(format!("http runtime: {e}"))));
                        return;
                    }
                };
                let client = match reqwest::Client::builder()
                    .timeout(Duration::from_secs(30))
                    .connect_timeout(Duration::from_secs(30))
                    .build()
                {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = ready_tx.send(Err(Error::Provider(format!(
                            "http client: {}",
                            e.without_url()
                        ))));
                        return;
                    }
                };
                let _ = ready_tx.send(Ok(()));
                while let Ok(task) = rx.recv() {
                    task(&runtime, &client);
                }
            })
            .map_err(Error::Io)?;
        ready_rx
            .recv()
            .map_err(|_| Error::Provider("http worker did not start".into()))??;
        Ok(Self {
            tasks: Mutex::new(tx),
        })
    }

    fn run(
        &self,
        build: impl FnOnce(&reqwest::Client) -> reqwest::RequestBuilder + Send + 'static,
    ) -> Result<HttpReply> {
        let (reply_tx, reply_rx) = mpsc::channel::<Result<HttpReply>>();
        let task: Task = Box::new(move |runtime, client| {
            let request = build(client);
            let result = runtime.block_on(async move {
                let response = request.send().await.map_err(map_error)?;
                let status = response.status().as_u16();
                let retry_after_s = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.trim().parse::<f64>().ok())
                    .map(|v| v.ceil().max(0.0) as u64);
                let bytes = response.bytes().await.map_err(map_error)?;
                let body = if bytes.is_empty() {
                    Json::Null
                } else {
                    serde_json::from_slice(&bytes).unwrap_or_else(|_| {
                        Json::String(String::from_utf8_lossy(&bytes).into_owned())
                    })
                };
                Ok(HttpReply {
                    status,
                    body,
                    retry_after_s,
                })
            });
            let _ = reply_tx.send(result);
        });
        self.tasks
            .lock()
            .map_err(|_| Error::Provider("http worker poisoned".into()))?
            .send(task)
            .map_err(|_| Error::Provider("http worker stopped".into()))?;
        reply_rx
            .recv()
            .map_err(|_| Error::Provider("http worker stopped".into()))?
    }
}

fn map_error(error: reqwest::Error) -> Error {
    if error.is_timeout() {
        Error::Timeout
    } else {
        Error::Provider(format!("http: {}", error.without_url()))
    }
}

impl HttpTransport for ReqwestTransport {
    fn post_graphql(
        &self,
        url: &str,
        access_token: &str,
        query: &str,
        variables: &Json,
    ) -> Result<HttpReply> {
        let url = url.to_owned();
        let token = access_token.to_owned();
        let body = serde_json::json!({ "query": query, "variables": variables });
        self.run(move |client| {
            client
                .post(url)
                .header("X-Shopify-Access-Token", token)
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .body(body.to_string())
        })
    }

    fn post_form(&self, url: &str, fields: &[(&str, &str)]) -> Result<HttpReply> {
        let url = url.to_owned();
        let body = form_encode(fields);
        self.run(move |client| {
            client
                .post(url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("Accept", "application/json")
                .body(body)
        })
    }

    fn put_bytes(
        &self,
        url: &str,
        headers: &[(String, String)],
        bytes: &[u8],
    ) -> Result<HttpReply> {
        let url = url.to_owned();
        let headers = headers.to_vec();
        let bytes = bytes.to_vec();
        self.run(move |client| {
            let mut request = client.put(url);
            for (name, value) in headers {
                request = request.header(name, value);
            }
            request.body(bytes)
        })
    }
}
