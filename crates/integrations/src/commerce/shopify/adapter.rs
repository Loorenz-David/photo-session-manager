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
        ConfirmedRun, DefinitionLink, RemoteIdentity, RemoteItem, RemoteMedia, RemoteMediaStatus,
        StagedTarget, WriteValue,
    },
    mechanisms::values::canonical_decimal,
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

/// MC-11 / MC-21 upload filename: `<folder_key>-<first 12 hex of uid>.jpg`, revision n ≥ 1 adds
/// `-r<n>` before the extension; lowercase.
pub fn upload_filename(folder_key: &str, uid: &str, revision: i64) -> String {
    let hex: String = uid
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(12)
        .collect();
    let suffix = if revision >= 1 {
        format!("-r{revision}")
    } else {
        String::new()
    };
    format!("{folder_key}-{hex}{suffix}.jpg").to_lowercase()
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
        let mut attempt = 0;
        loop {
            let reply: HttpReply = self.auth.with_token(&|token| {
                self.transport
                    .post_graphql(&url, token, document, &variables)
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

fn search_query(sku: &str) -> String {
    let escaped = sku.replace('\\', "\\\\").replace('"', "\\\"");
    format!("sku:\"{escaped}\"")
}

impl CommerceAdapter for ShopifyAdapter {
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

    fn item_create(
        &self,
        run: &ConfirmedRun,
        sku: &str,
        barcode: Option<&str>,
        title: &str,
    ) -> Result<String> {
        let mut variant = json!({
            "optionValues": [{ "optionName": "Title", "name": "Default Title" }],
            "sku": sku,
        });
        if let Some(barcode) = barcode {
            variant["barcodes"] = json!([{ "value": barcode }]);
        }
        let input = json!({
            "title": title,
            "status": "DRAFT",
            "productOptions": [{ "name": "Title", "values": [{ "name": "Default Title" }] }],
            "variants": [variant],
        });
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
        folder_key: &str,
        uid: &str,
        revision: i64,
        byte_size: u64,
    ) -> Result<StagedTarget> {
        let filename = upload_filename(folder_key, uid, revision);
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
        // SV-2: the new media is the last in gallery order, returned as `newest`.
        payload["product"]["newest"]["nodes"][0]["id"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| errors::provider("product_create_media", "no media id in the reply"))
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

    fn list_files_by_name(&self, stem: &str) -> Result<Vec<RemoteMedia>> {
        let mut found = Vec::new();
        let mut after: Option<String> = None;
        loop {
            let data = self.call(graphql::FILES_ALL, json!({"after":after}))?;
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
            if data["files"]["pageInfo"]["hasNextPage"] != true {
                break;
            }
            after = data["files"]["pageInfo"]["endCursor"]
                .as_str()
                .map(Into::into);
            if after.is_none() {
                return Err(Error::InvalidData("file pagination lacks cursor".into()));
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
        let data = self.mutate(
            run,
            "file_create",
            graphql::FILE_CREATE,
            json!({ "files": [{
                "originalSource": resource_url,
                "filename": filename,
                "alt": filename,
                "contentType": "IMAGE",
                "duplicateResolutionMode": "APPEND_UUID",
            }] }),
        )?;
        let payload = &data["fileCreate"];
        if let Some(message) = errors::user_errors(payload) {
            return Err(errors::provider("file_create", message));
        }
        payload["files"][0]["id"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| errors::provider("file_create", "no file id in the reply"))
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

fn condition_metafield_value(value: &str) -> Result<Json> {
    serde_json::from_str(value)
        .map_err(|error| Error::InvalidData(format!("condition metafield: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mc11_upload_filename() {
        assert_eq!(
            upload_filename("K-1", "AB12CD34-EF56-4a7b-8c9d-0123456789ab", 0),
            "k-1-ab12cd34ef56.jpg"
        );
        assert_eq!(
            upload_filename("k-1", "ab12cd34-ef56-4a7b-8c9d-0123456789ab", 2),
            "k-1-ab12cd34ef56-r2.jpg"
        );
    }

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
