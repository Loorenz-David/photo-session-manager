//! `ShopifyAdapter`: `CommerceAdapter` over the Admin GraphQL API (ch09, 07A MC-10…MC-14,
//! 09A MC-23/MC-24). Every Shopify-owned shape used here is grounded in
//! `docs/intention/evidence/shopify_admin.md` (SV-0…SV-7) and replayed from `fixtures/`.
//!
//! Primitives plus mapping only: no step machine, reconciliation or write-set logic (P13, P14).
//! Every mutation logs the run id; no log line or error ever carries a token or the secret.

use super::{
    auth::ClientCredentialsAuth,
    errors, graphql,
    transport::{HttpReply, HttpTransport},
    type_map,
    types::{DefinitionNode, DefinitionsPage, MediaNode, MetafieldsPage, ProductNode, VariantNode},
};
use psm_core::{
    Error, Result,
    domain::attribute::{AttributeGroup, CoreType, Definition},
    domain::publish::{
        ConfirmedRun, DefinitionLink, MediaImage, MediaReadback, RemoteIdentity, RemoteItem,
        RemoteMedia, RemoteMediaStatus, StagedTarget, StockChoice, StockLocation, StockLocations,
        StockOutcome, WriteValue,
    },
    mechanisms::{upload_name, values::canonical_decimal},
    ports::{
        clock::Clock,
        commerce::{ALL_CAPABILITIES, CommerceAdapter},
    },
};
use serde_json::{Value as Json, json};
use std::{sync::Arc, time::Duration};
use unicode_normalization::UnicodeNormalization;

/// The integration name used in core tables (`attribute_definitions.integration`, links).
pub const INTEGRATION: &str = "shopify";
/// SV-5: `metafieldsSet` accepts at most 25 entries per call.
pub const METAFIELDS_SET_LIMIT: usize = 25;
/// P64 (delegated choice): staged-upload inputs per `stagedUploadsCreate` request.
pub const STAGED_UPLOADS_LIMIT: usize = 50;

/// Settings of the adapter (09A MC-23/MC-24). `Debug` never shows the client secret.
#[derive(Clone, PartialEq, Eq)]
pub struct ShopifyConfig {
    pub shop_domain: String,
    pub api_version: String,
    pub client_id: String,
    pub client_secret: String,
    pub condition_photos_key: String,
}

impl std::fmt::Debug for ShopifyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShopifyConfig")
            .field("shop_domain", &self.shop_domain)
            .field("api_version", &self.api_version)
            .field("client_id", &self.client_id)
            .field("client_secret", &"<redacted>")
            .field("condition_photos_key", &self.condition_photos_key)
            .finish()
    }
}

impl From<&psm_infra::settings_file::IntegrationSettings> for ShopifyConfig {
    fn from(s: &psm_infra::settings_file::IntegrationSettings) -> Self {
        Self {
            shop_domain: s.shop_domain.clone(),
            api_version: s.api_version.clone(),
            client_id: s.client_id.clone(),
            client_secret: s.client_secret.clone(),
            condition_photos_key: s.condition_photos_key.clone(),
        }
    }
}

/// P65 (changelog 45): shown in the item form and Settings when the shop's app cannot read
/// locations or write inventory.
pub const STOCK_ACCESS_MISSING: &str =
    "Your Shopify app needs the read_locations and write_inventory permissions";

/// Whether a Shopify error says the app lacks an access scope (GraphQL `ACCESS_DENIED`).
fn access_denied(error: &Error) -> bool {
    let text = error.to_string().to_ascii_lowercase();
    text.contains("access denied")
        || text.contains("access_denied")
        || text.contains("read_locations")
        || text.contains("write_inventory")
        || text.contains("read_inventory")
}

/// A `ProductSetInput` for a new draft product with one default variant.
fn product_set_input(
    sku: &str,
    barcode: Option<&str>,
    title: &str,
    item_type: &str,
    stock: Option<&StockChoice>,
) -> Json {
    let mut variant = json!({
        "optionValues": [{ "optionName": "Title", "name": "Default Title" }],
        "sku": sku,
    });
    if let Some(barcode) = barcode {
        variant["barcodes"] = json!([{ "value": barcode }]);
    }
    if let Some(stock) = stock {
        // P65: tracked, with the chosen quantity available at the chosen location, in the same
        // `productSet` (ProductVariantSetInput.inventoryItem / .inventoryQuantities).
        variant["inventoryItem"] = json!({ "tracked": true });
        variant["inventoryQuantities"] = json!([{
            "locationId": stock.location_id,
            "name": "available",
            "quantity": stock.quantity,
        }]);
    }
    let mut input = json!({
        "title": title,
        "status": "DRAFT",
        "productOptions": [{ "name": "Title", "values": [{ "name": "Default Title" }] }],
        "variants": [variant],
    });
    if !item_type.is_empty() {
        input["productType"] = json!(item_type);
    }
    input
}

/// Polling and retry pacing (L19). Tests use 0 ms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PollConfig {
    pub interval_ms: u64,
    pub attempts: u32,
    /// Bounded retries of throttled (HTTP 429 / `THROTTLED`) calls and of 5xx replies to queries.
    pub retries: u32,
    /// Wait before a retry when the reply carries no `Retry-After`, and the cap on it.
    pub retry_wait_ms: u64,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1_000,
            attempts: 120,
            retries: 3,
            retry_wait_ms: 2_000,
        }
    }
}

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            other => out.push(other),
        }
    }
    out
}

/// MC-13 description: each blank-line-separated paragraph becomes `<p>…</p>` with `& < > "`
/// escaped. A single newline inside a paragraph is kept as is.
pub fn description_html(paragraphs: &str) -> String {
    let text = paragraphs.replace("\r\n", "\n");
    let mut out = String::new();
    let mut current: Vec<&str> = Vec::new();
    let flush = |current: &mut Vec<&str>, out: &mut String| {
        if !current.is_empty() {
            out.push_str("<p>");
            out.push_str(&escape_html(&current.join("\n")));
            out.push_str("</p>");
            current.clear();
        }
    };
    for line in text.split('\n') {
        if line.trim().is_empty() {
            flush(&mut current, &mut out);
        } else {
            current.push(line);
        }
    }
    flush(&mut current, &mut out);
    out
}

fn nfc(text: &str) -> String {
    text.nfc().collect()
}

fn media_status(status: Option<&str>) -> RemoteMediaStatus {
    match status {
        Some("READY") => RemoteMediaStatus::Ready,
        Some("FAILED") => RemoteMediaStatus::Failed,
        Some("UPLOADED") => RemoteMediaStatus::Uploaded,
        _ => RemoteMediaStatus::Processing,
    }
}

fn remote_media(nodes: &[MediaNode]) -> Vec<RemoteMedia> {
    nodes
        .iter()
        .enumerate()
        .map(|(i, m)| RemoteMedia {
            id: m.id.clone(),
            filename: m.filename(),
            status: media_status(m.status.as_deref()),
            position: i as i64 + 1,
        })
        .collect()
}

fn parse<T: serde::de::DeserializeOwned>(value: &Json, context: &str) -> Result<T> {
    serde_json::from_value(value.clone()).map_err(|e| errors::provider(context, e))
}

/// `"<namespace>.<key>"` → (namespace, key).
fn split_key(key: &str) -> Result<(&str, &str)> {
    key.split_once('.')
        .ok_or_else(|| Error::InvalidData(format!("not a namespace.key: {key}")))
}

pub struct ShopifyAdapter {
    config: ShopifyConfig,
    transport: Arc<dyn HttpTransport>,
    auth: ClientCredentialsAuth,
    poll: PollConfig,
}

impl ShopifyAdapter {
    pub fn new(
        config: ShopifyConfig,
        transport: Arc<dyn HttpTransport>,
        clock: Arc<dyn Clock>,
        poll: PollConfig,
    ) -> Self {
        let auth = ClientCredentialsAuth::new(
            transport.clone(),
            clock,
            &config.shop_domain,
            &config.client_id,
            &config.client_secret,
        );
        Self {
            config,
            transport,
            auth,
            poll,
        }
    }

    pub fn config(&self) -> &ShopifyConfig {
        &self.config
    }

    fn graphql_url(&self) -> String {
        format!(
            "https://{}/admin/api/{}/graphql.json",
            self.config.shop_domain, self.config.api_version
        )
    }

    fn sleep(ms: u64) {
        if ms > 0 {
            std::thread::sleep(Duration::from_millis(ms));
        }
    }

    /// One GraphQL document; returns `data`. Throttled replies are retried (bounded, honouring
    /// `Retry-After`); a 5xx is retried only for queries, never for mutations (MC-11: the outcome
    /// of a failed mutation is unknown and the caller reads before it writes again).
    fn call(&self, document: &str, variables: Json) -> Result<Json> {
        let is_mutation = document.trim_start().starts_with("mutation");
        let url = self.graphql_url();
        let started = std::time::Instant::now();
        let op = operation_name(document);
        let result = self.call_inner(&url, is_mutation, document, &variables);
        // ch12 §Logs: every provider call with its duration (never the token or the variables).
        tracing::info!(
            provider = "shopify",
            op = %op,
            duration_ms = started.elapsed().as_millis(),
            ok = result.is_ok(),
            "provider call"
        );
        result
    }

    fn call_inner(
        &self,
        url: &str,
        is_mutation: bool,
        document: &str,
        variables: &Json,
    ) -> Result<Json> {
        let mut attempt = 0;
        loop {
            let reply: HttpReply = self.auth.with_token(&|token| {
                self.transport.post_graphql(url, token, document, variables)
            })?;
            let retryable = reply.status == 429
                || errors::is_throttled(&reply.body)
                || (!is_mutation && reply.status >= 500);
            if retryable && attempt < self.poll.retries {
                attempt += 1;
                let wait = reply
                    .retry_after_s
                    .map(|s| s.saturating_mul(1000))
                    .unwrap_or(self.poll.retry_wait_ms)
                    .min(self.poll.retry_wait_ms.max(10_000));
                Self::sleep(wait);
                continue;
            }
            if reply.status != 200 {
                return Err(errors::provider(
                    "request",
                    format!("HTTP {}", reply.status),
                ));
            }
            if let Some(message) = errors::graphql_errors(&reply.body) {
                return Err(errors::provider("graphql", message));
            }
            return Ok(reply.body.get("data").cloned().unwrap_or(Json::Null));
        }
    }

    fn mutate(
        &self,
        run: &ConfirmedRun,
        op: &str,
        document: &str,
        variables: Json,
    ) -> Result<Json> {
        tracing::info!(run_id = run.run_id(), op, "shopify mutation");
        self.call(document, variables)
    }

    fn product(&self, external_id: &str) -> Result<ProductNode> {
        let data = self.call(graphql::PRODUCT_READ, json!({ "id": external_id }))?;
        match data.get("product") {
            Some(Json::Null) | None => Err(Error::NotFound(format!("product {external_id}"))),
            Some(product) => parse(product, "product"),
        }
    }

    fn definitions(&self) -> Result<Vec<DefinitionNode>> {
        let mut out = Vec::new();
        let mut after = Json::Null;
        loop {
            let data = self.call(graphql::DEFINITIONS, json!({ "after": after }))?;
            let page: DefinitionsPage = parse(&data["metafieldDefinitions"], "definitions")?;
            out.extend(page.nodes);
            match (page.page_info.has_next_page, page.page_info.end_cursor) {
                (true, Some(cursor)) => after = Json::from(cursor),
                _ => return Ok(out),
            }
        }
    }

    fn core_definition(&self, node: &DefinitionNode) -> (Definition, DefinitionLink) {
        let key = format!("{}.{}", node.namespace, node.key);
        let platform_type = node.type_.name.as_str();
        let (mut core_type, mut writable) =
            type_map::to_core(platform_type, &node.validations, &node.access);
        if key == self.config.condition_photos_key && platform_type == "list.file_reference" {
            core_type = CoreType::MediaList;
            writable = type_map::access_permits_write(&node.access);
        }
        let group = if (node.namespace == "custom" && node.key.starts_with("condition_"))
            || key == self.config.condition_photos_key
            || key == "custom.condition_report"
        {
            AttributeGroup::Condition
        } else {
            AttributeGroup::Details
        };
        let validations = if let Some(units) = type_map::units_for(platform_type) {
            json!({ "units": units })
        } else if core_type == CoreType::Rating {
            let bound = |name: &str| {
                type_map::validation(&node.validations, name)
                    .and_then(canonical_decimal)
                    .map(Json::from)
                    .unwrap_or(Json::Null)
            };
            json!({ "min": bound("scale_min"), "max": bound("scale_max") })
        } else {
            json!({})
        };
        let choices = match core_type {
            CoreType::Choice | CoreType::MultiChoice => type_map::choices(&node.validations),
            _ => Vec::new(),
        };
        let definition = Definition {
            integration: INTEGRATION.into(),
            key,
            label: node.name.clone(),
            core_type,
            choices,
            validations,
            group,
            description: node.description.clone().filter(|d| !d.is_empty()),
            writable,
        };
        let link = DefinitionLink {
            platform_type: platform_type.to_owned(),
            platform_meta: json!({
                "id": node.id,
                "namespace": node.namespace,
                "key": node.key,
                "validations": node.validations,
                "access": node.access,
            }),
        };
        (definition, link)
    }

    fn poll_until(
        &self,
        document: &str,
        id: &str,
        done: &dyn Fn(&Json) -> Result<bool>,
    ) -> Result<()> {
        for attempt in 0..self.poll.attempts {
            if attempt > 0 {
                Self::sleep(self.poll.interval_ms);
            }
            let data = self.call(document, json!({ "id": id }))?;
            if done(&data)? {
                return Ok(());
            }
        }
        Err(Error::Timeout)
    }

    fn update_product(&self, run: &ConfirmedRun, op: &str, product: Json) -> Result<()> {
        let data = self.mutate(
            run,
            op,
            graphql::PRODUCT_UPDATE,
            json!({ "product": product }),
        )?;
        match errors::user_errors(&data["productUpdate"]) {
            Some(message) => Err(errors::provider(op, message)),
            None => Ok(()),
        }
    }
}

/// The GraphQL operation's name for the log (`mutation productCreate(…)` → `productCreate`),
/// or its kind when unnamed.
fn operation_name(document: &str) -> String {
    let mut words = document
        .trim_start()
        .split(|c: char| c.is_whitespace() || c == '(' || c == '{')
        .filter(|w| !w.is_empty());
    let kind = words.next().unwrap_or("query");
    match words.next() {
        Some(name)
            if document.trim_start()[kind.len()..]
                .trim_start()
                .starts_with(name) =>
        {
            name.to_owned()
        }
        _ => kind.to_owned(),
    }
}

/// P64: the files search for an upload stem (SV-2 form `filename:<stem>`); a stem with characters
/// outside the upload-name alphabet is quoted.
fn files_query(stem: &str) -> String {
    if stem
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        format!("filename:{stem}")
    } else {
        let escaped = stem.replace('\\', "\\\\").replace('"', "\\\"");
        format!("filename:\"{escaped}\"")
    }
}

/// The upload stem of a staged resource URL (`…/<stem>.jpg?…`).
fn resource_stem(resource_url: &str) -> Result<&str> {
    resource_url
        .split('?')
        .next()
        .and_then(|url| url.rsplit('/').next())
        .and_then(|name| name.strip_suffix(".jpg"))
        .ok_or_else(|| errors::provider("product_create_media", "resource filename is not a jpg"))
}

fn search_query(sku: &str) -> String {
    let escaped = sku.replace('\\', "\\\\").replace('"', "\\\"");
    format!("sku:\"{escaped}\"")
}

impl CommerceAdapter for ShopifyAdapter {
    fn render_description(&self, paragraphs: &str) -> String {
        description_html(paragraphs)
    }
    fn check_token(&self) -> Result<()> {
        self.call("query { shop { name } }", serde_json::json!({}))?;
        Ok(())
    }
    fn condition_photos_key(&self) -> &str {
        &self.config.condition_photos_key
    }
    fn capabilities(&self) -> Vec<&'static str> {
        ALL_CAPABILITIES.to_vec()
    }

    fn item_search(&self, sku: &str) -> Result<Vec<RemoteItem>> {
        let wanted = nfc(sku);
        let data = self.call(
            graphql::VARIANT_SEARCH,
            json!({ "query": search_query(sku) }),
        )?;
        let variants: Vec<VariantNode> = parse(&data["productVariants"]["nodes"], "search")?;
        let mut items: Vec<RemoteItem> = Vec::new();
        for variant in variants {
            let Some(variant_sku) = variant.sku() else {
                continue;
            };
            if nfc(&variant_sku) != wanted {
                continue;
            }
            let Some(product) = variant.product.clone() else {
                continue;
            };
            if items.iter().any(|i| i.external_id == product.id) {
                continue;
            }
            items.push(RemoteItem {
                external_id: product.id,
                title: product.title,
                item_type: product.product_type,
                sku: Some(variant_sku),
                barcode: variant.barcode(),
                thumbnail_url: product
                    .featured_media
                    .and_then(|m| m.preview)
                    .and_then(|p| p.image)
                    .and_then(|i| i.url),
                media: Vec::new(),
            });
        }
        Ok(items)
    }

    fn item_find(&self, query: &str) -> Result<Vec<RemoteItem>> {
        let mut found = self.item_search(query)?;
        let safe = query.replace(['"', '\\', ':', '*'], " ");
        let terms = safe
            .split_whitespace()
            .map(|term| format!("{term}*"))
            .collect::<Vec<_>>()
            .join(" ");
        let Ok(data) = self.call(graphql::TITLE_SEARCH, json!({"query":terms})) else {
            return Ok(found);
        };
        let products: Vec<Json> = parse(&data["products"]["nodes"], "title search")?;
        for product in products {
            let id = product["id"].as_str().unwrap_or_default();
            if id.is_empty()
                || found.iter().any(|item| item.external_id == id)
                || !product["title"]
                    .as_str()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&query.to_lowercase())
            {
                continue;
            }
            let variant = &product["variants"]["nodes"][0];
            let Some(sku) = variant["inventoryItem"]["sku"]
                .as_str()
                .or_else(|| variant["sku"].as_str())
            else {
                continue;
            };
            found.push(RemoteItem {
                external_id: id.into(),
                title: product["title"].as_str().unwrap_or_default().into(),
                item_type: product["productType"].as_str().map(str::to_owned),
                sku: Some(sku.into()),
                barcode: variant["barcodes"]["nodes"][0]["value"]
                    .as_str()
                    .map(Into::into),
                thumbnail_url: product["featuredMedia"]["preview"]["image"]["url"]
                    .as_str()
                    .map(Into::into),
                media: Vec::new(),
            });
        }
        Ok(found)
    }

    fn item_get(&self, external_id: &str) -> Result<RemoteItem> {
        let product = self.product(external_id)?;
        let variant = product.variants.nodes.first();
        Ok(RemoteItem {
            external_id: product.id.clone(),
            title: product.title.clone(),
            item_type: product.product_type.clone(),
            sku: variant.and_then(VariantNode::sku),
            barcode: variant.and_then(VariantNode::barcode),
            thumbnail_url: product
                .media
                .nodes
                .first()
                .and_then(|m| m.image.as_ref())
                .and_then(|i| i.url.clone()),
            media: remote_media(&product.media.nodes),
        })
    }

    fn product_fields(&self, external_id: &str) -> Result<(Vec<String>, Option<String>)> {
        let product = self.product(external_id)?;
        Ok((product.tags, product.description_html))
    }

    fn item_types(&self) -> Result<Vec<String>> {
        let mut after: Option<String> = None;
        let mut types = std::collections::BTreeSet::new();
        loop {
            let data = self.call(graphql::PRODUCT_TYPES, json!({"after": after}))?;
            let page = &data["productTypes"];
            let nodes: Vec<String> = parse(&page["nodes"], "product types")?;
            types.extend(nodes.into_iter().filter(|name| !name.trim().is_empty()));
            if page["pageInfo"]["hasNextPage"] != true {
                break;
            }
            after = page["pageInfo"]["endCursor"].as_str().map(str::to_owned);
            if after.is_none() {
                return Err(errors::provider("product types", "missing page cursor"));
            }
        }
        Ok(types.into_iter().collect())
    }

    fn item_create(
        &self,
        run: &ConfirmedRun,
        sku: &str,
        barcode: Option<&str>,
        title: &str,
        item_type: &str,
    ) -> Result<String> {
        let input = product_set_input(sku, barcode, title, item_type, None);
        let data = self.mutate(
            run,
            "item_create",
            graphql::PRODUCT_SET,
            json!({ "input": input }),
        )?;
        let payload = &data["productSet"];
        if let Some(message) = errors::user_errors(payload) {
            return Err(errors::provider("item_create", message));
        }
        payload["product"]["id"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| errors::provider("item_create", "no product id in the reply"))
    }

    fn stock_locations(&self) -> Result<StockLocations> {
        let data = match self.call(graphql::LOCATIONS, json!({})) {
            Ok(data) => data,
            Err(error) if access_denied(&error) => {
                tracing::warn!(%error, "shop locations not readable");
                return Ok(StockLocations {
                    locations: Vec::new(),
                    missing_access: Some(STOCK_ACCESS_MISSING.into()),
                });
            }
            Err(error) => return Err(error),
        };
        let nodes = data["locations"]["nodes"]
            .as_array()
            .ok_or_else(|| errors::provider("locations", "no locations in the reply"))?;
        Ok(StockLocations {
            locations: nodes
                .iter()
                .filter(|node| node["isActive"].as_bool() != Some(false))
                .filter_map(|node| {
                    Some(StockLocation {
                        id: node["id"].as_str()?.to_owned(),
                        name: node["name"].as_str()?.to_owned(),
                    })
                })
                .collect(),
            missing_access: None,
        })
    }

    fn item_create_stocked(
        &self,
        run: &ConfirmedRun,
        sku: &str,
        barcode: Option<&str>,
        title: &str,
        item_type: &str,
        stock: &StockChoice,
    ) -> Result<(String, StockOutcome)> {
        let input = product_set_input(sku, barcode, title, item_type, Some(stock));
        let refused = match self.mutate(
            run,
            "item_create",
            graphql::PRODUCT_SET,
            json!({ "input": input }),
        ) {
            Ok(data) => {
                let payload = &data["productSet"];
                if let Some(id) = payload["product"]["id"].as_str() {
                    let id = id.to_owned();
                    if let Some(message) = errors::user_errors(payload) {
                        return Ok((id, StockOutcome::Untracked { reason: message }));
                    }
                    let verified = match self.call(
                        graphql::INVENTORY_READBACK,
                        json!({ "id": id, "locationId": stock.location_id }),
                    ) {
                        Ok(read) => {
                            let item = &read["product"]["variants"]["nodes"][0]["inventoryItem"];
                            let available = item["inventoryLevel"]["quantities"]
                                .as_array()
                                .and_then(|q| {
                                    q.iter()
                                        .find(|q| q["name"].as_str() == Some("available"))
                                        .and_then(|q| q["quantity"].as_i64())
                                });
                            item["tracked"].as_bool() == Some(true)
                                && available == Some(stock.quantity)
                        }
                        Err(error) => {
                            tracing::warn!(%error, "stock read-back failed");
                            false
                        }
                    };
                    if !verified {
                        tracing::warn!(product = id, "stock read-back does not match the create");
                    }
                    return Ok((id, StockOutcome::Tracked { verified }));
                }
                // Not created (user errors): the stock fields are dropped and it is sent again.
                errors::user_errors(payload).unwrap_or_else(|| "no product in the reply".into())
            }
            // A GraphQL error (an access scope missing): nothing was executed.
            Err(error) if access_denied(&error) || error.to_string().contains("shopify graphql:") => {
                error.to_string()
            }
            Err(error) => return Err(error),
        };
        tracing::warn!(
            reason = refused,
            "product create with stock refused; creating it without stock tracking"
        );
        let id = self.item_create(run, sku, barcode, title, item_type)?;
        let reason = if access_denied(&Error::Provider(refused.clone())) {
            STOCK_ACCESS_MISSING.to_string()
        } else {
            refused
        };
        Ok((id, StockOutcome::Untracked { reason }))
    }

    fn item_set_identity(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        sku: &str,
        barcode: Option<&str>,
    ) -> Result<RemoteIdentity> {
        let product = self.product(external_id)?;
        let variant = product
            .variants
            .nodes
            .first()
            .ok_or_else(|| errors::provider("item_set_identity", "product has no variant"))?;
        let current_sku = variant.sku();
        let current_barcode = variant.barcode();
        let mut input = json!({ "id": variant.id });
        let mut changed = false;
        if current_sku.as_deref() != Some(sku) {
            input["inventoryItem"] = json!({ "sku": sku });
            changed = true;
        }
        if let Some(barcode) = barcode
            && current_barcode.as_deref() != Some(barcode)
        {
            input["barcodes"] = json!([{ "value": barcode }]);
            changed = true;
        }
        if !changed {
            return Ok(RemoteIdentity {
                sku: current_sku.unwrap_or_default(),
                barcode: current_barcode,
            });
        }
        let data = self.mutate(
            run,
            "item_set_identity",
            graphql::ITEM_SET_IDENTITY,
            json!({ "productId": external_id, "variants": [input] }),
        )?;
        let payload = &data["productVariantsBulkUpdate"];
        if let Some(message) = errors::user_errors(payload) {
            return Err(errors::provider("item_set_identity", message));
        }
        let after: VariantNode = parse(&payload["productVariants"][0], "item_set_identity")?;
        Ok(RemoteIdentity {
            sku: after.sku().unwrap_or_default(),
            barcode: after.barcode(),
        })
    }

    fn attributes_schema(&self) -> Result<Vec<(Definition, DefinitionLink)>> {
        Ok(self
            .definitions()?
            .iter()
            .map(|node| self.core_definition(node))
            .collect())
    }

    fn attributes_read(&self, external_id: &str) -> Result<Vec<(String, Json)>> {
        let mut out = Vec::new();
        let mut after = Json::Null;
        loop {
            let data = self.call(
                graphql::METAFIELDS_READ,
                json!({ "id": external_id, "after": after }),
            )?;
            if data["product"].is_null() {
                return Err(Error::NotFound(format!("product {external_id}")));
            }
            let page: MetafieldsPage = parse(&data["product"]["metafields"], "metafields")?;
            for m in &page.nodes {
                let key = format!("{}.{}", m.namespace, m.key);
                // The condition-photos value is local (exclusions, 07A MC-14); the remote file list
                // is derived from it and is not read back as a core value.
                if key == self.config.condition_photos_key {
                    continue;
                }
                let core_type = type_map::value_core_type(&m.type_);
                out.push((
                    key,
                    type_map::from_platform_string(core_type, &m.type_, &m.value),
                ));
            }
            match (page.page_info.has_next_page, page.page_info.end_cursor) {
                (true, Some(cursor)) => after = Json::from(cursor),
                _ => return Ok(out),
            }
        }
    }

    fn condition_photos_readback(&self, external_id: &str) -> Result<Option<Json>> {
        let (namespace, key) = split_key(&self.config.condition_photos_key)?;
        let data = self.call(
            graphql::CONDITION_METAFIELD_READ,
            json!({ "id": external_id, "namespace": namespace, "key": key }),
        )?;
        let Some(value) = data["product"]["metafield"]["value"].as_str() else {
            return Ok(None);
        };
        condition_metafield_value(value).map(Some)
    }

    fn attributes_write(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        values: &[WriteValue],
    ) -> Result<()> {
        let mut entries = Vec::with_capacity(values.len());
        for v in values {
            let (namespace, key) = split_key(&v.key)?;
            let core_type = type_map::value_core_type(&v.link.platform_type);
            entries.push((
                v.key.clone(),
                json!({
                    "ownerId": external_id,
                    "namespace": namespace,
                    "key": key,
                    "type": v.link.platform_type,
                    "value": type_map::to_platform_string(core_type, &v.value),
                }),
            ));
        }
        for batch in entries.chunks(METAFIELDS_SET_LIMIT) {
            let metafields: Vec<&Json> = batch.iter().map(|(_, e)| e).collect();
            let data = self.mutate(
                run,
                "attributes_write",
                graphql::METAFIELDS_SET,
                json!({ "metafields": metafields }),
            )?;
            let user_errors = data["metafieldsSet"]["userErrors"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            if user_errors.is_empty() {
                continue;
            }
            // SV-5: a call is atomic; `field` is ["metafields", "<index>", …], `elementIndex` null.
            let mut keys: Vec<String> = Vec::new();
            for e in &user_errors {
                let index = e["elementIndex"].as_u64().map(|i| i as usize).or_else(|| {
                    e["field"]
                        .as_array()
                        .and_then(|f| f.get(1))
                        .and_then(Json::as_str)
                        .and_then(|i| i.parse().ok())
                });
                match index.and_then(|i| batch.get(i)) {
                    Some((key, _)) => keys.push(key.clone()),
                    None => keys.extend(batch.iter().map(|(k, _)| k.clone())),
                }
            }
            keys.dedup();
            let mut unique: Vec<String> = Vec::new();
            for k in keys {
                if !unique.contains(&k) {
                    unique.push(k);
                }
            }
            return Err(Error::AttributeVerification { keys: unique });
        }
        Ok(())
    }

    fn staged_uploads_create(
        &self,
        run: &ConfirmedRun,
        stem: &str,
        byte_size: u64,
    ) -> Result<StagedTarget> {
        // Core builds the stem (10D MC-29 §Upload filename); the adapter only adds `.jpg`.
        let filename = format!("{stem}.jpg");
        let data = self.mutate(
            run,
            "staged_uploads_create",
            graphql::STAGED_UPLOADS_CREATE,
            json!({ "input": [{
                "resource": "IMAGE",
                "filename": filename,
                "mimeType": "image/jpeg",
                "httpMethod": "PUT",
                "fileSize": byte_size.to_string(),
            }] }),
        )?;
        let payload = &data["stagedUploadsCreate"];
        if let Some(message) = errors::user_errors(payload) {
            return Err(errors::provider("staged_uploads_create", message));
        }
        let target: super::types::StagedTargetNode =
            parse(&payload["stagedTargets"][0], "staged_uploads_create")?;
        Ok(StagedTarget {
            filename,
            url: target.url,
            resource_url: target.resource_url,
            parameters: target
                .parameters
                .into_iter()
                .map(|p| (p.name, p.value))
                .collect(),
        })
    }

    fn staged_uploads_create_many(
        &self,
        run: &ConfirmedRun,
        files: &[(String, u64)],
    ) -> Result<Vec<StagedTarget>> {
        let mut out = Vec::with_capacity(files.len());
        for chunk in files.chunks(STAGED_UPLOADS_LIMIT) {
            let filenames: Vec<String> = chunk
                .iter()
                .map(|(stem, _)| format!("{stem}.jpg"))
                .collect();
            let input: Vec<Json> = chunk
                .iter()
                .zip(&filenames)
                .map(|((_, size), filename)| {
                    json!({
                        "resource": "IMAGE",
                        "filename": filename,
                        "mimeType": "image/jpeg",
                        "httpMethod": "PUT",
                        "fileSize": size.to_string(),
                    })
                })
                .collect();
            let data = self.mutate(
                run,
                "staged_uploads_create",
                graphql::STAGED_UPLOADS_CREATE,
                json!({ "input": input }),
            )?;
            let payload = &data["stagedUploadsCreate"];
            if let Some(message) = errors::user_errors(payload) {
                return Err(errors::provider("staged_uploads_create", message));
            }
            let targets: Vec<super::types::StagedTargetNode> =
                parse(&payload["stagedTargets"], "staged_uploads_create")?;
            if targets.len() != chunk.len() {
                return Err(errors::provider(
                    "staged_uploads_create",
                    format!("{} targets for {} files", targets.len(), chunk.len()),
                ));
            }
            for (target, filename) in targets.into_iter().zip(filenames) {
                out.push(StagedTarget {
                    filename,
                    url: target.url,
                    resource_url: target.resource_url,
                    parameters: target
                        .parameters
                        .into_iter()
                        .map(|p| (p.name, p.value))
                        .collect(),
                });
            }
        }
        Ok(out)
    }

    fn upload(&self, run: &ConfirmedRun, target: &StagedTarget, bytes: &[u8]) -> Result<()> {
        tracing::info!(
            run_id = run.run_id(),
            op = "upload",
            "shopify staged upload"
        );
        let reply = self
            .transport
            .put_bytes(&target.url, &target.parameters, bytes)?;
        if reply.status >= 300 {
            return Err(errors::provider("upload", format!("HTTP {}", reply.status)));
        }
        Ok(())
    }

    fn product_create_media(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        resource_url: &str,
        alt: &str,
    ) -> Result<String> {
        let data = self.mutate(
            run,
            "product_create_media",
            graphql::PRODUCT_CREATE_MEDIA,
            json!({
                "product": { "id": external_id },
                "media": [{ "originalSource": resource_url, "alt": alt, "mediaContentType": "IMAGE" }],
            }),
        )?;
        let payload = &data["productUpdate"];
        if let Some(message) = errors::user_errors(payload) {
            return Err(errors::provider("product_create_media", message));
        }
        let candidate = payload["product"]["newest"]["nodes"][0]["id"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| errors::provider("product_create_media", "no media id in the reply"))?;
        let stem = resource_stem(resource_url)?;
        for attempt in 0..self.poll.attempts {
            if attempt > 0 {
                Self::sleep(self.poll.interval_ms);
            }
            let data = self.call(graphql::MEDIA_POLL, json!({ "id": candidate }))?;
            let node = &data["node"];
            match node["status"].as_str() {
                Some("READY") => {
                    let media: MediaNode = parse(node, "product_create_media")?;
                    if media
                        .filename()
                        .as_deref()
                        .is_some_and(|name| upload_name::matches(stem, name))
                    {
                        return Ok(candidate);
                    }
                    return self
                        .list_media(external_id)?
                        .into_iter()
                        .find(|media| {
                            media
                                .filename
                                .as_deref()
                                .is_some_and(|name| upload_name::matches(stem, name))
                        })
                        .map(|media| media.id)
                        .ok_or_else(|| {
                            errors::provider(
                                "product_create_media",
                                format!("created media {stem}.jpg not found in the gallery"),
                            )
                        });
                }
                Some("FAILED") => {
                    let message = node["mediaErrors"]
                        .as_array()
                        .and_then(|errors| errors.first())
                        .and_then(|error| error["message"].as_str())
                        .unwrap_or("media processing failed");
                    return Err(Error::Provider(message.to_owned()));
                }
                _ => {}
            }
        }
        Err(Error::Timeout)
    }

    fn product_create_media_many(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        resource_urls: &[String],
        alt: &str,
    ) -> Result<Vec<String>> {
        if resource_urls.len() <= 1 {
            return resource_urls
                .iter()
                .map(|url| self.product_create_media(run, external_id, url, alt))
                .collect();
        }
        let stems = resource_urls
            .iter()
            .map(|url| resource_stem(url))
            .collect::<Result<Vec<_>>>()?;
        let media: Vec<Json> = resource_urls
            .iter()
            .map(|url| json!({ "originalSource": url, "alt": alt, "mediaContentType": "IMAGE" }))
            .collect();
        let data = self.mutate(
            run,
            "product_create_media",
            graphql::PRODUCT_CREATE_MEDIA_MANY,
            json!({
                "product": { "id": external_id },
                "media": media,
                "count": resource_urls.len(),
            }),
        )?;
        let payload = &data["productUpdate"];
        if let Some(message) = errors::user_errors(payload) {
            return Err(errors::provider("product_create_media", message));
        }
        let mut open: Vec<String> = payload["product"]["newest"]["nodes"]
            .as_array()
            .map(|nodes| {
                nodes
                    .iter()
                    .filter_map(|n| n["id"].as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        // The created media show their file name once READY (SV-2): poll them together, then
        // map each name to its upload stem; a stem not among them is looked up in the gallery.
        let mut ids: Vec<Option<String>> = vec![None; stems.len()];
        let mut attempt = 0;
        while !open.is_empty() {
            if attempt >= self.poll.attempts {
                return Err(Error::Timeout);
            }
            if attempt > 0 {
                Self::sleep(self.poll.interval_ms);
            }
            attempt += 1;
            let mut still = Vec::new();
            for candidate in open {
                let data = self.call(graphql::MEDIA_POLL, json!({ "id": candidate }))?;
                let node = &data["node"];
                match node["status"].as_str() {
                    Some("READY") => {
                        let media: MediaNode = parse(node, "product_create_media")?;
                        let name = media.filename();
                        if let Some(i) = (0..stems.len()).find(|&i| {
                            ids[i].is_none()
                                && name
                                    .as_deref()
                                    .is_some_and(|name| upload_name::matches(stems[i], name))
                        }) {
                            ids[i] = Some(candidate);
                        }
                    }
                    Some("FAILED") => {
                        let message = node["mediaErrors"]
                            .as_array()
                            .and_then(|errors| errors.first())
                            .and_then(|error| error["message"].as_str())
                            .unwrap_or("media processing failed");
                        return Err(Error::Provider(message.to_owned()));
                    }
                    _ => still.push(candidate),
                }
            }
            open = still;
        }
        if ids.iter().any(Option::is_none) {
            let gallery = self.list_media(external_id)?;
            for (i, stem) in stems.iter().enumerate() {
                if ids[i].is_none() {
                    ids[i] = gallery
                        .iter()
                        .find(|m| {
                            m.filename
                                .as_deref()
                                .is_some_and(|name| upload_name::matches(stem, name))
                        })
                        .map(|m| m.id.clone());
                }
            }
        }
        ids.into_iter()
            .zip(&stems)
            .map(|(id, stem)| {
                id.ok_or_else(|| {
                    errors::provider(
                        "product_create_media",
                        format!("created media {stem}.jpg not found in the gallery"),
                    )
                })
            })
            .collect()
    }

    fn poll_media_ready(&self, media_id: &str) -> Result<()> {
        self.poll_until(graphql::MEDIA_POLL, media_id, &|data| {
            let node = &data["node"];
            match node["status"].as_str() {
                Some("READY") => Ok(true),
                Some("FAILED") => {
                    let message = node["mediaErrors"]
                        .as_array()
                        .and_then(|e| e.first())
                        .and_then(|e| e["message"].as_str())
                        .unwrap_or("media processing failed");
                    Err(Error::Provider(message.to_owned()))
                }
                _ => Ok(false),
            }
        })
    }

    fn product_reorder_media(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        media_ids: &[String],
    ) -> Result<String> {
        // SV-4: `newPosition` is 0-based.
        let moves: Vec<Json> = media_ids
            .iter()
            .enumerate()
            .map(|(i, id)| json!({ "id": id, "newPosition": i.to_string() }))
            .collect();
        let data = self.mutate(
            run,
            "product_reorder_media",
            graphql::PRODUCT_REORDER_MEDIA,
            json!({ "id": external_id, "moves": moves }),
        )?;
        let payload = &data["productReorderMedia"];
        if let Some(message) = errors::user_errors(payload) {
            return Err(errors::provider("product_reorder_media", message));
        }
        Ok(payload["job"]["id"].as_str().unwrap_or_default().to_owned())
    }

    fn poll_job(&self, job_id: &str) -> Result<()> {
        if job_id.is_empty() {
            return Ok(());
        }
        self.poll_until(graphql::JOB_POLL, job_id, &|data| {
            Ok(data["job"]["done"].as_bool() == Some(true))
        })
    }

    fn list_media(&self, external_id: &str) -> Result<Vec<RemoteMedia>> {
        Ok(remote_media(&self.product(external_id)?.media.nodes))
    }

    fn media_readback(&self, external_id: &str) -> Result<MediaReadback> {
        let (namespace, key) = split_key(&self.config.condition_photos_key)?;
        let data = self.call(
            graphql::MEDIA_READBACK,
            json!({ "id": external_id, "namespace": namespace, "key": key }),
        )?;
        let product = match data.get("product") {
            Some(Json::Null) | None => {
                return Err(Error::NotFound(format!("product {external_id}")));
            }
            Some(product) => product,
        };
        media_readback_of(product)
    }

    fn fetch_image(&self, url: &str, max_edge: u32) -> Result<Vec<u8>> {
        if !url.starts_with("https://") {
            return Err(Error::InvalidData("image URL is not https".into()));
        }
        let separator = if url.contains('?') { '&' } else { '?' };
        let started = std::time::Instant::now();
        let result = self
            .transport
            .get_bytes(&format!("{url}{separator}width={max_edge}"));
        tracing::info!(
            provider = "shopify",
            op = "fetch_image",
            duration_ms = started.elapsed().as_millis(),
            ok = result.is_ok(),
            "provider call"
        );
        result
    }

    fn list_files_by_name(&self, stem: &str) -> Result<Vec<RemoteMedia>> {
        // P64: one search request by name (SV-2 `filename:<stem>` also finds uniquified copies);
        // the match rules are unchanged: the CDN name starts with the stem, or alt = `<stem>.jpg`.
        let data = self.call(
            graphql::FILES_BY_NAME,
            json!({ "query": files_query(stem) }),
        )?;
        let mut found = Vec::new();
        if let Some(nodes) = data["files"]["nodes"].as_array() {
            for (position, node) in nodes.iter().enumerate() {
                let filename = node["image"]["url"]
                    .as_str()
                    .and_then(|url| url.split('?').next())
                    .and_then(|path| path.rsplit('/').next())
                    .map(str::to_owned);
                let alt = node["alt"].as_str();
                if !filename
                    .as_deref()
                    .is_some_and(|name| name.starts_with(stem))
                    && alt != Some(format!("{stem}.jpg").as_str())
                {
                    continue;
                }
                let Some(id) = node["id"].as_str() else {
                    continue;
                };
                let status = if node["fileStatus"] == "READY" {
                    RemoteMediaStatus::Ready
                } else {
                    RemoteMediaStatus::Processing
                };
                found.push(RemoteMedia {
                    id: id.into(),
                    filename,
                    status,
                    position: position as i64 + 1,
                });
            }
        }
        Ok(found)
    }

    fn product_delete_media(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        media_ids: &[String],
    ) -> Result<()> {
        for id in media_ids {
            let data = self.mutate(
                run,
                "product_delete_media",
                graphql::PRODUCT_DELETE_MEDIA,
                json!({ "files": [{ "id": id, "referencesToRemove": [external_id] }] }),
            )?;
            let payload = &data["fileUpdate"];
            if errors::only_not_found(payload) {
                continue;
            }
            if let Some(message) = errors::user_errors(payload) {
                return Err(errors::provider("product_delete_media", message));
            }
        }
        Ok(())
    }

    fn file_create(
        &self,
        run: &ConfirmedRun,
        resource_url: &str,
        filename: &str,
    ) -> Result<String> {
        self.file_create_many(run, &[(resource_url.to_owned(), filename.to_owned())])?
            .pop()
            .ok_or_else(|| errors::provider("file_create", "no file id in the reply"))
    }

    fn file_create_many(
        &self,
        run: &ConfirmedRun,
        files: &[(String, String)],
    ) -> Result<Vec<String>> {
        let input: Vec<Json> = files
            .iter()
            .map(|(resource_url, filename)| {
                json!({
                    "originalSource": resource_url,
                    "filename": filename,
                    "alt": filename,
                    "contentType": "IMAGE",
                    "duplicateResolutionMode": "APPEND_UUID",
                })
            })
            .collect();
        let data = self.mutate(
            run,
            "file_create",
            graphql::FILE_CREATE,
            json!({ "files": input }),
        )?;
        let payload = &data["fileCreate"];
        if let Some(message) = errors::user_errors(payload) {
            return Err(errors::provider("file_create", message));
        }
        let ids: Vec<String> = payload["files"]
            .as_array()
            .map(|nodes| {
                nodes
                    .iter()
                    .filter_map(|n| n["id"].as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        if ids.len() != files.len() {
            return Err(errors::provider(
                "file_create",
                format!("{} file ids for {} files", ids.len(), files.len()),
            ));
        }
        Ok(ids)
    }

    fn poll_file(&self, file_id: &str) -> Result<()> {
        self.poll_until(graphql::FILE_POLL, file_id, &|data| {
            let node = &data["node"];
            match node["fileStatus"].as_str() {
                Some("READY") => Ok(true),
                Some("FAILED") => {
                    let message = node["fileErrors"]
                        .as_array()
                        .and_then(|e| e.first())
                        .and_then(|e| e["message"].as_str())
                        .unwrap_or("file processing failed");
                    Err(Error::Provider(message.to_owned()))
                }
                _ => Ok(false),
            }
        })
    }

    fn update_title(&self, run: &ConfirmedRun, external_id: &str, title: &str) -> Result<()> {
        self.update_product(
            run,
            "update_title",
            json!({ "id": external_id, "title": title }),
        )
    }

    fn update_item_type(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        item_type: &str,
    ) -> Result<()> {
        self.update_product(
            run,
            "update_item_type",
            json!({ "id": external_id, "productType": item_type }),
        )
    }

    fn update_description(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        paragraphs: &str,
    ) -> Result<()> {
        self.update_product(
            run,
            "update_description",
            json!({ "id": external_id, "descriptionHtml": description_html(paragraphs) }),
        )
    }

    fn update_tags(&self, run: &ConfirmedRun, external_id: &str, tags: &[String]) -> Result<()> {
        self.update_product(
            run,
            "update_tags",
            json!({ "id": external_id, "tags": tags }),
        )
    }
}

/// The read-back of `MEDIA_READBACK`'s `product`: gallery media in order (image URL, else the
/// original source), and the condition field's file ids in the field's order with the URLs of
/// their references.
fn media_readback_of(product: &Json) -> Result<MediaReadback> {
    let nodes: Vec<MediaNode> = parse(&product["media"]["nodes"], "media")?;
    let gallery = nodes
        .iter()
        .map(|node| MediaImage {
            id: node.id.clone(),
            url: [node.image.as_ref(), node.original_source.as_ref()]
                .into_iter()
                .flatten()
                .find_map(|u| u.url.clone()),
        })
        .collect();
    let mut condition = Vec::new();
    let metafield = &product["metafield"];
    if let Some(value) = metafield["value"].as_str() {
        let ids: Vec<String> = match condition_metafield_value(value)? {
            Json::Array(ids) => ids
                .into_iter()
                .filter_map(|id| id.as_str().map(str::to_owned))
                .collect(),
            _ => Vec::new(),
        };
        let references = metafield["references"]["nodes"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        for id in ids {
            let url = references
                .iter()
                .find(|node| node["id"].as_str() == Some(id.as_str()))
                .and_then(|node| {
                    node["image"]["url"]
                        .as_str()
                        .or_else(|| node["url"].as_str())
                        .map(str::to_owned)
                });
            condition.push(MediaImage { id, url });
        }
    }
    Ok(MediaReadback { gallery, condition })
}

fn condition_metafield_value(value: &str) -> Result<Json> {
    serde_json::from_str(value)
        .map_err(|error| Error::InvalidData(format!("condition metafield: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mc13_description_html() {
        assert_eq!(
            description_html("a & b\n\nc < d \"q\""),
            "<p>a &amp; b</p><p>c &lt; d &quot;q&quot;</p>"
        );
        assert_eq!(
            description_html("x > y\nline\n\n\n\nz"),
            "<p>x &gt; y\nline</p><p>z</p>"
        );
        assert_eq!(description_html(""), "");
    }

    #[test]
    fn mc14_condition_file_ids_are_json_values() {
        let value = r#"["gid://shopify/MediaImage/1","gid://shopify/MediaImage/2"]"#;
        assert_eq!(
            condition_metafield_value(value).unwrap(),
            json!(["gid://shopify/MediaImage/1", "gid://shopify/MediaImage/2"])
        );
    }
}
