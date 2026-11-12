//! V9: the shop's public hosts and a product's storefront URL and handle, by one read-only Admin
//! query (client credentials grant, 09A MC-23). The access token lives in memory only.

use crate::recording::{Call, OutBody};
use serde_json::{Value as Json, json};
use std::path::Path;

pub const PROVIDER: &str = "shop";
pub const API_VERSION: &str = "2026-10";

pub fn token(label: &str, dir: &Path, shop: &str, client_id: &str, secret: &str) -> Call {
    let mut call = Call::new(
        label,
        dir,
        PROVIDER,
        "shop",
        "POST",
        &format!("https://{shop}/admin/oauth/access_token"),
    );
    call.body = OutBody::Form(vec![
        ("grant_type".into(), "client_credentials".into()),
        ("client_id".into(), client_id.to_owned()),
        ("client_secret".into(), secret.to_owned()),
    ]);
    call
}

pub fn graphql(label: &str, dir: &Path, shop: &str, token: &str, query: &str, vars: Json) -> Call {
    let mut call = Call::new(
        label,
        dir,
        PROVIDER,
        "shop",
        "POST",
        &format!("https://{shop}/admin/api/{API_VERSION}/graphql.json"),
    );
    call.headers = vec![("x-shopify-access-token".into(), token.to_owned())];
    call.body = OutBody::Json(json!({"query": query, "variables": vars}));
    call
}

/// The read-only query of V9.
pub const HOSTS_QUERY: &str = "query($id: ID!) { shop { name myshopifyDomain primaryDomain { host url sslEnabled } } \
webPresences(first: 10) { nodes { domain { host url } rootUrls { locale url } } } \
product(id: $id) { handle status onlineStoreUrl onlineStorePreviewUrl } }";
