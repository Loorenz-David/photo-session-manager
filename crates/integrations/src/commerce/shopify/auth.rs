//! 09A MC-23: the Admin API access token from Shopify's client credentials grant.
//!
//! `POST https://<shop>/admin/oauth/access_token` with the form fields `grant_type`,
//! `client_id`, `client_secret` answers `{"access_token", "expires_in", "scope"}` (SV-0:
//! `expires_in` = 86399 s). The token is kept in memory only, fetched on first use, reused until
//! `now ≥ obtained_at + lifetime − 60 s`, then fetched again. A GraphQL call answered 401 fetches
//! a new token and is retried once; a second 401 is `credentials_rejected`. The client secret is
//! only ever sent in the token request.

use super::{errors, transport::HttpReply, transport::HttpTransport};
use psm_core::{Error, Result, ports::clock::Clock};
use serde_json::Value as Json;
use std::sync::{Arc, Mutex};

const REFRESH_MARGIN_MS: i64 = 60_000;

struct Token {
    value: String,
    obtained_at_ms: i64,
    lifetime_ms: Option<i64>,
}

pub struct ClientCredentialsAuth {
    transport: Arc<dyn HttpTransport>,
    clock: Arc<dyn Clock>,
    token_url: String,
    client_id: String,
    client_secret: String,
    token: Mutex<Option<Token>>,
}

impl std::fmt::Debug for ClientCredentialsAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientCredentialsAuth")
            .field("token_url", &self.token_url)
            .field("client_id", &self.client_id)
            .finish_non_exhaustive()
    }
}

impl ClientCredentialsAuth {
    pub fn new(
        transport: Arc<dyn HttpTransport>,
        clock: Arc<dyn Clock>,
        shop_domain: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Self {
        Self {
            transport,
            clock,
            token_url: format!("https://{shop_domain}/admin/oauth/access_token"),
            client_id: client_id.to_owned(),
            client_secret: client_secret.to_owned(),
            token: Mutex::new(None),
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Option<Token>>> {
        self.token
            .lock()
            .map_err(|_| Error::Provider("shopify token state poisoned".into()))
    }

    /// The cached token while it is fresh, else a newly fetched one.
    pub fn access_token(&self) -> Result<String> {
        let now = self.clock.now_ms();
        if let Some(token) = self.lock()?.as_ref() {
            let fresh = token
                .lifetime_ms
                .is_none_or(|lifetime| now < token.obtained_at_ms + lifetime - REFRESH_MARGIN_MS);
            if fresh {
                return Ok(token.value.clone());
            }
        }
        self.fetch()
    }

    fn fetch(&self) -> Result<String> {
        *self.lock()? = None;
        let fields = [
            ("grant_type", "client_credentials"),
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
        ];
        let reply = self.transport.post_form(&self.token_url, &fields)?;
        if reply.status != 200 {
            return Err(errors::grant_refusal(&reply.body));
        }
        let value = reply
            .body
            .get("access_token")
            .and_then(Json::as_str)
            .filter(|t| !t.is_empty())
            .ok_or_else(|| Error::Provider("shopify token reply without access_token".into()))?
            .to_owned();
        psm_infra::logging::add_secret(&value);
        let lifetime_ms = reply
            .body
            .get("expires_in")
            .and_then(Json::as_i64)
            .map(|s| s * 1000);
        *self.lock()? = Some(Token {
            value: value.clone(),
            obtained_at_ms: self.clock.now_ms(),
            lifetime_ms,
        });
        Ok(value)
    }

    /// Runs `call` with the access token; on a 401 fetches a new token and retries once.
    pub fn with_token(&self, call: &dyn Fn(&str) -> Result<HttpReply>) -> Result<HttpReply> {
        let token = self.access_token()?;
        let reply = call(&token)?;
        if reply.status != 401 {
            return Ok(reply);
        }
        let token = self.fetch()?;
        let reply = call(&token)?;
        if reply.status == 401 {
            *self.lock()? = None;
            return Err(Error::CredentialsRejected);
        }
        Ok(reply)
    }
}
