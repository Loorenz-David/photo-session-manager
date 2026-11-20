//! The HTTP seam of the Beyo Pricing adapter (`master_plan_conversations.md` §4 Adapter):
//! `PricingHttp` (live impl `ReqwestPricing` here, testkit `FakePricing`).
//!
//! `ReqwestPricing` runs an async `reqwest` client on a private worker thread that owns its own
//! Tokio runtime, as `ReqwestTransport` does, so it is safe to create, use and drop from any
//! thread. The key goes only in the `Authorization: Bearer` header; error messages never carry a
//! URL; nothing here logs.

pub use crate::commerce::shopify::transport::HttpReply;
use psm_core::{Error, Result};
use serde_json::Value as Json;
use std::{
    sync::{Mutex, mpsc},
    time::Duration,
};

/// Connect time-out of a Pricing request.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Total time-out of a Pricing request.
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Patch,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Patch => "PATCH",
        }
    }
}

/// One request to Pricing. `Debug` never shows the bearer.
#[derive(Clone, PartialEq)]
pub struct PricingRequest {
    pub method: Method,
    /// The full address, query included.
    pub url: String,
    pub bearer: Option<String>,
    pub body: Option<Json>,
}

impl std::fmt::Debug for PricingRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PricingRequest")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("bearer", &self.bearer.as_ref().map(|_| "<redacted>"))
            .field("body", &self.body)
            .finish()
    }
}

pub trait PricingHttp: Send + Sync {
    /// Sends one request; `Err` only for a transport failure (`Error::Timeout` or
    /// `Error::Provider("http: …")`), never for an HTTP status.
    fn send(&self, request: PricingRequest) -> Result<HttpReply>;
}

type Task = Box<dyn FnOnce(&tokio::runtime::Runtime, &reqwest::Client) + Send>;

/// The live transport.
pub struct ReqwestPricing {
    tasks: Mutex<mpsc::Sender<Task>>,
}

impl ReqwestPricing {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel::<Task>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<()>>();
        std::thread::Builder::new()
            .name("psm-pricing-http".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(1)
                    .thread_name("psm-pricing-io")
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
                    .timeout(TOTAL_TIMEOUT)
                    .connect_timeout(CONNECT_TIMEOUT)
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
}

fn map_error(error: reqwest::Error) -> Error {
    if error.is_timeout() {
        Error::Timeout
    } else {
        Error::Provider(format!("http: {}", error.without_url()))
    }
}

impl PricingHttp for ReqwestPricing {
    fn send(&self, request: PricingRequest) -> Result<HttpReply> {
        let (reply_tx, reply_rx) = mpsc::channel::<Result<HttpReply>>();
        let task: Task = Box::new(move |runtime, client| {
            let mut builder = match request.method {
                Method::Get => client.get(&request.url),
                Method::Post => client.post(&request.url),
                Method::Patch => client.patch(&request.url),
            }
            .header("Accept", "application/json");
            if let Some(bearer) = &request.bearer {
                builder = builder.header("Authorization", format!("Bearer {bearer}"));
            }
            if let Some(body) = &request.body {
                builder = builder
                    .header("Content-Type", "application/json")
                    .body(body.to_string());
            }
            runtime.spawn(async move {
                let result = async move {
                    let response = builder.send().await.map_err(map_error)?;
                    let status = response.status().as_u16();
                    let bytes = response.bytes().await.map_err(map_error)?;
                    let body = if bytes.is_empty() {
                        Json::Null
                    } else {
                        serde_json::from_slice(&bytes).unwrap_or_else(|_| {
                            Json::String(String::from_utf8_lossy(&bytes).into_owned())
                        })
                    };
                    Ok(HttpReply::new(status, body))
                }
                .await;
                let _ = reply_tx.send(result);
            });
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
