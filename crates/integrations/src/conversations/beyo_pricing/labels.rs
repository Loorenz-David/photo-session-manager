//! `impl LabelsProvider for BeyoPricingAdapter` (book 09C §3; `master_plan_labels.md` §4 Adapter
//! paths, Words): Pricing's `/labels/…` calls over the same transport, server and key as the
//! conversations calls. **No label call maps to `ProviderError`** (ACC.5): a refused label call
//! must never mark the conversations connection refused, so every label answer goes through
//! [`label_error`] and never through the conversations `read_error` / `write_error`.

use super::adapter::{BeyoPricingAdapter, encode};
use super::label_types::{
    IntegrationLabelProduct, IntegrationLabelProductPage, IntegrationLabelRun,
    IntegrationLabelStatus, IntegrationLabelTemplate, IntegrationPlan, IntegrationPlanIssue,
    IntegrationPlanItem, IntegrationPlanRequest, IntegrationPreviewRequest, IntegrationRunCreate,
    IntegrationRunItemIn, LabelStatusOut, UnresolvedItem, WireLabelStatus, WirePlanOutcome,
    WireRunStatus,
};
use super::transport::{HttpReply, Method, PricingRequest, RawReply};
use super::types::{ErrorBody, ErrorEnvelope};
use psm_core::{
    domain::label::{
        CheckOutcome, LabelArtifact, LabelCounts, LabelFilter, LabelPlan, LabelProduct,
        LabelProductPage, LabelRun, LabelSize, LabelState, LabelStatus, LabelTemplate, LabelsError,
        NoteKind, PlanItem, PlanNote, PlanSummary, PlanTemplate, ResetReason, RunCounts, RunItem,
        RunResult, RunStatus, Unresolved,
    },
    ports::labels::{LabelsProvider, LabelsResult},
};
use serde::de::DeserializeOwned;
use serde_json::Value as Json;
use std::time::Duration;

/// The time-out of a run (rendering is synchronous: a few seconds for hundreds of labels).
pub const RUN_TIMEOUT: Duration = Duration::from_secs(60);
/// The time-out of a preview or a file download.
pub const FILE_TIMEOUT: Duration = Duration::from_secs(60);

/// The first bytes of every PDF (MC-L2.12): anything else is not a label file.
const PDF_MAGIC: &[u8] = b"%PDF-";

/// The commerce external id form of a product (Shopify's global id).
const COMMERCE_PRODUCT_PREFIX: &str = "gid://shopify/Product/";

/// Which call answered, for the statuses whose meaning depends on it (Adapter paths).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Call {
    /// List, status, templates, plan, runs.
    Read,
    /// `POST …/labels/preview`: a 404 names a product, with its message.
    Preview,
    /// The artifact download: a 404 is a file that is gone.
    Download,
}

/// A non-2xx label answer (Adapter paths "Status mapping"; the only error function of label
/// calls).
pub(super) fn label_error(status: u16, envelope: Option<ErrorBody>, call: Call) -> LabelsError {
    let code = envelope.as_ref().map(|e| e.code.as_str());
    let rejected = |envelope: Option<ErrorBody>| match envelope {
        Some(body) => LabelsError::Rejected {
            code: body.code,
            message: body.message,
        },
        None => LabelsError::BadReply(format!("HTTP {status}")),
    };
    match status {
        401 | 403 => LabelsError::NotAllowed,
        404 => match call {
            Call::Read => LabelsError::StoreGone,
            Call::Preview => rejected(envelope),
            Call::Download => LabelsError::Expired,
        },
        409 => rejected(envelope),
        410 => LabelsError::Expired,
        422 if code == Some("no_products") => LabelsError::NoProducts,
        422 => rejected(envelope),
        503 if code == Some("renderer_unavailable") => LabelsError::RendererUnavailable,
        503 if code == Some("product_list_unavailable") => LabelsError::ListUnavailable,
        500..=599 => LabelsError::Unreachable(format!("HTTP {status}")),
        _ => rejected(envelope),
    }
}

fn json_envelope(body: &Json) -> Option<ErrorBody> {
    serde_json::from_value::<ErrorEnvelope>(body.clone())
        .ok()
        .map(|e| e.error)
}

fn bytes_envelope(bytes: &[u8]) -> Option<ErrorBody> {
    serde_json::from_slice::<ErrorEnvelope>(bytes)
        .ok()
        .map(|e| e.error)
}

/// A file answer: the bytes of a 2xx PDF, else the mapped error.
fn pdf(reply: RawReply, call: Call) -> LabelsResult<Vec<u8>> {
    if !(200..300).contains(&reply.status) {
        return Err(label_error(
            reply.status,
            bytes_envelope(&reply.bytes),
            call,
        ));
    }
    if !reply.bytes.starts_with(PDF_MAGIC) {
        return Err(LabelsError::BadReply("not a PDF".into()));
    }
    Ok(reply.bytes)
}

/// MC-L2.12: a run's download path is sent only when it is a path under this store's runs
/// (`/integration/v1/stores/<store, encoded as the adapter encodes it>/labels/runs/`) with no
/// `..` segment, no `?`, no `#` and no `//`. A backslash and a percent-encoded dot segment are
/// refused too: an address parser reads them as `/` and `..`.
fn download_path_ok(store_id: &str, path: &str) -> bool {
    let prefix = format!("/integration/v1/stores/{}/labels/runs/", encode(store_id));
    let dot_segment = |segment: &str| {
        matches!(
            segment.to_ascii_lowercase().as_str(),
            "." | ".." | ".%2e" | "%2e." | "%2e%2e" | "%2e"
        )
    };
    path.starts_with(&prefix)
        && !path.contains(['?', '#', '\\'])
        && !path.contains("//")
        && !path.split('/').any(dot_segment)
}

fn items_in(product_ids: &[String]) -> Vec<IntegrationRunItemIn> {
    product_ids
        .iter()
        .map(|id| IntegrationRunItemIn {
            sku: None,
            product_id: Some(id.clone()),
            template_id: None,
        })
        .collect()
}

fn body<T: serde::Serialize>(value: &T) -> LabelsResult<Json> {
    serde_json::to_value(value).map_err(|e| LabelsError::BadReply(e.to_string()))
}

fn unresolved(wire: UnresolvedItem) -> Unresolved {
    Unresolved {
        product_id: wire.product_id,
        sku: wire.sku,
        message: wire.error.message,
    }
}

fn plan_item(wire: IntegrationPlanItem) -> LabelsResult<PlanItem> {
    Ok(PlanItem {
        position: wire.position,
        printed_at: optional_time(wire.label_printed_at.as_deref())?,
        product_id: wire.product_id,
        sku: wire.sku,
        title: wire.title,
        product_type: wire.product_type,
        template: wire.template.map(|t| PlanTemplate {
            id: t.id,
            name: t.name,
            width_mm: t.width_mm,
            height_mm: t.height_mm,
        }),
        price_minor: wire.price_minor,
        status: status(wire.label_status),
        outcome: match wire.outcome {
            WirePlanOutcome::MarkPrinted => CheckOutcome::MarkPrinted,
            WirePlanOutcome::Reprint => CheckOutcome::Reprint,
            WirePlanOutcome::NoPrice => CheckOutcome::NoPrice,
            WirePlanOutcome::Fail => CheckOutcome::Fail,
        },
        issues: wire.issues.into_iter().map(|i| i.message).collect(),
        notes: wire.notes.into_iter().map(plan_note).collect(),
    })
}

/// MC-L2.15: Pricing's `url_not_published` note is `NotPublished`, any other code `Other`. Its
/// address is read out of the sentence (`… the QR code opens {url}.`): the first word starting
/// with `https://` or `http://`, trailing `.`, `,`, `;`, `)` removed; none → no link (Rule 17:
/// the schema gives only `code` and `message`, so this fails soft).
fn plan_note(wire: IntegrationPlanIssue) -> PlanNote {
    if wire.code != "url_not_published" {
        return PlanNote {
            kind: NoteKind::Other,
            message: wire.message,
            link: None,
        };
    }
    let link = wire
        .message
        .split_whitespace()
        .find(|word| word.starts_with("https://") || word.starts_with("http://"))
        .map(|word| word.trim_end_matches(['.', ',', ';', ')']).to_owned());
    PlanNote {
        kind: NoteKind::NotPublished,
        message: wire.message,
        link,
    }
}

fn plan(wire: IntegrationPlan) -> LabelsResult<LabelPlan> {
    let s = wire.summary;
    Ok(LabelPlan {
        items: wire
            .items
            .into_iter()
            .map(plan_item)
            .collect::<LabelsResult<_>>()?,
        unresolved: wire.unresolved.into_iter().map(unresolved).collect(),
        summary: PlanSummary {
            items: s.items,
            ready: s.ready,
            failing: s.failing,
            mark_printed: s.mark_printed,
            reprint: s.reprint,
            no_price: s.no_price,
        },
        sizes: wire
            .sizes
            .into_iter()
            .map(|z| LabelSize {
                width_mm: z.width_mm,
                height_mm: z.height_mm,
                count: z.count,
            })
            .collect(),
    })
}

/// `lifecycleResult`, read tolerantly (anything but the three words → none; projection r0 D4).
fn run_result(word: Option<&str>) -> Option<RunResult> {
    match word? {
        "marked_printed" => Some(RunResult::MarkedPrinted),
        "reprinted" => Some(RunResult::Reprinted),
        "no_price" => Some(RunResult::NoPrice),
        _ => None,
    }
}

fn run(wire: IntegrationLabelRun) -> LabelsResult<LabelRun> {
    let c = wire.counts;
    Ok(LabelRun {
        created_at: time(&wire.created_at)?,
        id: wire.id,
        status: match wire.status {
            WireRunStatus::Rendered => RunStatus::Rendered,
            WireRunStatus::Partial => RunStatus::Partial,
            WireRunStatus::Failed => RunStatus::Failed,
        },
        counts: RunCounts {
            items: c.items,
            rendered: c.rendered,
            failed: c.failed,
            marked_printed: c.marked_printed,
            reprinted: c.reprinted,
            no_price: c.no_price,
        },
        artifacts: wire
            .artifacts
            .into_iter()
            .map(|a| LabelArtifact {
                id: a.id,
                width_mm: a.width_mm,
                height_mm: a.height_mm,
                page_count: a.page_count,
                filename: a.filename,
                size_bytes: a.size_bytes,
                download_url: a.download_url,
                expired: a.expired,
            })
            .collect(),
        items: wire
            .items
            .into_iter()
            .map(|i| RunItem {
                rendered: i.outcome == "rendered",
                result: run_result(i.lifecycle_result.as_deref()),
                error: i.error.map(|e| e.message),
                product_id: i.product_id,
                sku: i.sku,
                title: i.title,
            })
            .collect(),
        unresolved: wire.unresolved.into_iter().map(unresolved).collect(),
    })
}

fn parse<T: DeserializeOwned>(body: &Json) -> LabelsResult<T> {
    serde_json::from_value(body.clone())
        .map_err(|error| LabelsError::BadReply(format!("reply shape: {error}")))
}

/// ISO 8601 / RFC 3339 → UTC milliseconds.
fn time(text: &str) -> LabelsResult<i64> {
    chrono::DateTime::parse_from_rfc3339(text)
        .map(|t| t.timestamp_millis())
        .map_err(|_| LabelsError::BadReply("time".into()))
}

fn optional_time(text: Option<&str>) -> LabelsResult<Option<i64>> {
    text.map(time).transpose()
}

fn status(wire: WireLabelStatus) -> LabelStatus {
    match wire {
        WireLabelStatus::Printed => LabelStatus::Printed,
        WireLabelStatus::NotPrinted => LabelStatus::NotPrinted,
    }
}

/// A reset reason word; absent, null or an unknown word → none (MC-L3.1).
pub(super) fn reset_reason(word: Option<&str>) -> Option<ResetReason> {
    match word? {
        "price_changed" => Some(ResetReason::PriceChanged),
        "sku_changed" => Some(ResetReason::SkuChanged),
        "barcode_changed" => Some(ResetReason::BarcodeChanged),
        _ => None,
    }
}

fn product(wire: IntegrationLabelProduct) -> LabelsResult<LabelProduct> {
    Ok(LabelProduct {
        state: LabelState {
            status: status(wire.label_status),
            printed_at: optional_time(wire.label_printed_at.as_deref())?,
            has_price: Some(wire.has_price),
            reset: reset_reason(wire.label_reset_reason.as_deref()),
        },
        created_at: optional_time(wire.created_at.as_deref())?,
        product_id: wire.product_id,
        sku: wire.sku,
        title: wire.title,
        product_type: wire.product_type,
        price_minor: wire.price_minor,
        thumb_url: wire.thumb_url,
    })
}

fn status_state(wire: LabelStatusOut) -> LabelsResult<LabelState> {
    Ok(LabelState {
        status: status(wire.label_status),
        printed_at: optional_time(wire.label_printed_at.as_deref())?,
        has_price: wire.has_price,
        reset: reset_reason(wire.reset_reason.as_deref()),
    })
}

fn filter_word(filter: LabelFilter) -> Option<&'static str> {
    match filter {
        LabelFilter::MissingPriceTag => Some("missingPriceTag"),
        LabelFilter::NotPrinted => Some("notPrinted"),
        LabelFilter::Printed => Some("printed"),
        LabelFilter::All => None,
    }
}

impl BeyoPricingAdapter {
    /// One JSON label request through `send`; a transport failure is `Unreachable`.
    pub(super) fn label_send(
        &self,
        method: Method,
        path: &str,
        body: Option<Json>,
    ) -> LabelsResult<HttpReply> {
        let url = format!("{}/integration/v1{path}", (self.server)());
        self.http
            .send(PricingRequest {
                method,
                url,
                bearer: (self.key)(),
                body,
            })
            .map_err(|error| LabelsError::Unreachable(error.to_string()))
    }

    /// A label read answered with JSON: the 2xx body parsed as `T`, else [`label_error`].
    fn label_read<T: DeserializeOwned>(&self, path: &str) -> LabelsResult<T> {
        let reply = self.label_send(Method::Get, path, None)?;
        if !(200..300).contains(&reply.status) {
            return Err(label_error(
                reply.status,
                json_envelope(&reply.body),
                Call::Read,
            ));
        }
        parse(&reply.body)
    }

    /// One label request through `fetch` (the body untouched); a transport failure is
    /// `Unreachable`.
    fn label_fetch(
        &self,
        method: Method,
        url: String,
        body: Option<Json>,
        timeout: Duration,
    ) -> LabelsResult<RawReply> {
        self.http
            .fetch(
                PricingRequest {
                    method,
                    url,
                    bearer: (self.key)(),
                    body,
                },
                timeout,
            )
            .map_err(|error| LabelsError::Unreachable(error.to_string()))
    }

    fn label_url(&self, path: &str) -> String {
        format!("{}/integration/v1{path}", (self.server)())
    }

    pub(super) fn labels_path(store_id: &str) -> String {
        format!("{}/labels", Self::store_path(store_id))
    }
}

impl LabelsProvider for BeyoPricingAdapter {
    fn products(
        &self,
        store_id: &str,
        filter: LabelFilter,
        query: Option<&str>,
        cursor: Option<&str>,
        limit: u32,
    ) -> LabelsResult<LabelProductPage> {
        let mut params = Vec::new();
        if let Some(word) = filter_word(filter) {
            params.push(format!("filter={word}"));
        }
        if let Some(q) = query {
            params.push(format!("q={}", encode(q)));
        }
        if let Some(cursor) = cursor {
            params.push(format!("cursor={}", encode(cursor)));
        }
        params.push(format!("limit={limit}"));
        let path = format!(
            "{}/products?{}",
            Self::labels_path(store_id),
            params.join("&")
        );
        let page: IntegrationLabelProductPage = self.label_read(&path)?;
        let c = page.counts;
        Ok(LabelProductPage {
            products: page
                .items
                .into_iter()
                .map(product)
                .collect::<LabelsResult<_>>()?,
            next_cursor: page.next_cursor,
            counts: LabelCounts {
                total: c.total,
                has_price: c.has_price,
                missing_price: c.missing_price,
                missing_price_tag: c.missing_price_tag,
                printed: c.printed,
                not_printed: c.not_printed,
            },
            work_from: page.work_from,
        })
    }

    fn status(
        &self,
        store_id: &str,
        product_ids: &[String],
    ) -> LabelsResult<Vec<Option<LabelState>>> {
        if product_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<String> = product_ids.iter().map(|id| encode(id)).collect();
        let path = format!(
            "{}/status?productIds={}",
            Self::labels_path(store_id),
            ids.join(",")
        );
        let entries: Vec<IntegrationLabelStatus> = self.label_read(&path)?;
        if entries.len() != product_ids.len() {
            return Err(LabelsError::BadReply(format!(
                "{} status entries for {} products",
                entries.len(),
                product_ids.len()
            )));
        }
        entries
            .into_iter()
            .map(|entry| entry.label.map(status_state).transpose())
            .collect()
    }

    fn templates(&self, store_id: &str) -> LabelsResult<Vec<LabelTemplate>> {
        let path = format!("{}/templates", Self::labels_path(store_id));
        let list: Vec<IntegrationLabelTemplate> = self.label_read(&path)?;
        Ok(list
            .into_iter()
            .map(|t| LabelTemplate {
                id: t.id,
                name: t.name,
                width_mm: t.width_mm,
                height_mm: t.height_mm,
                is_default: t.is_default,
                product_types: t.product_types,
            })
            .collect())
    }

    fn plan(&self, store_id: &str, product_ids: &[String]) -> LabelsResult<LabelPlan> {
        let request = body(&IntegrationPlanRequest {
            items: items_in(product_ids),
        })?;
        let path = format!("{}/plan", Self::labels_path(store_id));
        let reply = self.label_send(Method::Post, &path, Some(request))?;
        if !(200..300).contains(&reply.status) {
            return Err(label_error(
                reply.status,
                json_envelope(&reply.body),
                Call::Read,
            ));
        }
        plan(parse(&reply.body)?)
    }

    fn preview(&self, store_id: &str, product_id: &str) -> LabelsResult<Vec<u8>> {
        let request = body(&IntegrationPreviewRequest {
            sku: None,
            product_id: Some(product_id.to_owned()),
            template_id: None,
        })?;
        let url = self.label_url(&format!("{}/preview", Self::labels_path(store_id)));
        let reply = self.label_fetch(Method::Post, url, Some(request), FILE_TIMEOUT)?;
        pdf(reply, Call::Preview)
    }

    fn run(&self, store_id: &str, product_ids: &[String], key: &str) -> LabelsResult<LabelRun> {
        let request = body(&IntegrationRunCreate {
            items: items_in(product_ids),
            idempotency_key: key.to_owned(),
        })?;
        let url = self.label_url(&format!("{}/runs", Self::labels_path(store_id)));
        let reply = self.label_fetch(Method::Post, url, Some(request), RUN_TIMEOUT)?;
        if !(200..300).contains(&reply.status) {
            return Err(label_error(
                reply.status,
                bytes_envelope(&reply.bytes),
                Call::Read,
            ));
        }
        let wire: IntegrationLabelRun = serde_json::from_slice(&reply.bytes)
            .map_err(|error| LabelsError::BadReply(format!("reply shape: {error}")))?;
        run(wire)
    }

    fn download(&self, store_id: &str, download_url: &str) -> LabelsResult<Vec<u8>> {
        if !download_path_ok(store_id, download_url) {
            return Err(LabelsError::BadReply("download address".into()));
        }
        let url = format!("{}{download_url}", (self.server)());
        let reply = self.label_fetch(Method::Get, url, None, FILE_TIMEOUT)?;
        pdf(reply, Call::Download)
    }

    fn describe(&self, error: &LabelsError, shop: Option<&str>) -> String {
        match error {
            LabelsError::NotAllowed => {
                "Session Manager isn't allowed to print labels. Ask a Pricing admin.".into()
            }
            LabelsError::StoreGone => format!(
                "This key is no longer tied to {}",
                shop.unwrap_or("the connected shop")
            ),
            LabelsError::NoProducts => "None of these products were found in Pricing.".into(),
            LabelsError::RendererUnavailable => {
                "Labels can't be generated right now. Try again in a moment.".into()
            }
            LabelsError::Expired => "This file has expired. Generate the labels again.".into(),
            LabelsError::ListUnavailable => "The product list isn't available right now.".into(),
            LabelsError::Unreachable(_) => "Pricing can't be reached right now.".into(),
            LabelsError::BadReply(_) => {
                "Pricing answered in a way this version does not understand".into()
            }
            LabelsError::Rejected { message, .. } => message.clone(),
        }
    }

    fn product_id_of(&self, commerce_external_id: &str) -> Option<String> {
        let digits = commerce_external_id.strip_prefix(COMMERCE_PRODUCT_PREFIX)?;
        is_digits(digits).then(|| digits.to_owned())
    }

    fn commerce_id_of(&self, product_id: &str) -> Option<String> {
        is_digits(product_id).then(|| format!("{COMMERCE_PRODUCT_PREFIX}{product_id}"))
    }
}

/// One or more ASCII digits.
fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::super::adapter::testing::Recording;
    use super::*;
    use psm_core::Error;
    use serde_json::json;
    use std::sync::Arc;

    const PRODUCTS: &str = include_str!("fixtures/labels_products.json");
    const STATUS: &str = include_str!("fixtures/labels_status.json");
    const TEMPLATES: &str = include_str!("fixtures/labels_templates.json");

    const S: &str = "e8f6b61a-40a4-4504-8090-d9ea5b62a069";
    const PRODUCT: &str = "15531808096409";

    fn fixture(text: &str) -> Json {
        serde_json::from_str(text).unwrap()
    }

    fn ms(text: &str) -> i64 {
        chrono::DateTime::parse_from_rfc3339(text)
            .unwrap()
            .timestamp_millis()
    }

    fn adapter() -> (Arc<Recording>, BeyoPricingAdapter) {
        let http = Arc::new(Recording::default());
        let adapter = BeyoPricingAdapter::new(
            http.clone(),
            || "https://pricing.example".into(),
            || Some("k1".into()),
        );
        (http, adapter)
    }

    fn envelope(code: &str, message: &str) -> Json {
        json!({"error": {"code": code, "message": message, "details": null},
               "correlationId": "c1"})
    }

    fn get(url: &str) -> PricingRequest {
        PricingRequest {
            method: Method::Get,
            url: url.into(),
            bearer: Some("k1".into()),
            body: None,
        }
    }

    /// The products fixture with the first product's fields changed.
    fn products_with(change: impl FnOnce(&mut Json)) -> Json {
        let mut page = fixture(PRODUCTS);
        change(&mut page["items"][0]);
        page
    }

    fn first_state(http: &Recording, adapter: &BeyoPricingAdapter, page: Json) -> LabelState {
        http.reply(200, page);
        adapter
            .products(S, LabelFilter::All, None, None, 25)
            .unwrap()
            .products[0]
            .state
    }

    fn fixture_page() -> LabelProductPage {
        LabelProductPage {
            products: vec![LabelProduct {
                product_id: PRODUCT.into(),
                sku: Some("TEST-1802".into()),
                title: "Test table".into(),
                product_type: Some("Test tables".into()),
                price_minor: Some(890000),
                state: LabelState {
                    status: LabelStatus::NotPrinted,
                    printed_at: None,
                    has_price: Some(true),
                    reset: Some(ResetReason::PriceChanged),
                },
                thumb_url: Some(
                    "https://cdn.shopify.com/s/files/1/0000/0000/0000/files/test-table_160x.jpg"
                        .into(),
                ),
                created_at: Some(ms("2026-11-20T08:41:00Z")),
            }],
            next_cursor: Some("bzoxMDA".into()),
            counts: LabelCounts {
                total: 214,
                has_price: 180,
                missing_price: 34,
                missing_price_tag: 12,
                printed: 168,
                not_printed: 46,
            },
            work_from: Some("2026-11-19".into()),
        }
    }

    #[test]
    fn l1_c1a_products_maps_the_fixture() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(PRODUCTS));
        let page = adapter
            .products(S, LabelFilter::MissingPriceTag, None, None, 25)
            .unwrap();
        assert_eq!(page, fixture_page());
    }

    #[test]
    fn l1_c1b_products_sends_one_get() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(PRODUCTS));
        adapter
            .products(S, LabelFilter::MissingPriceTag, None, None, 25)
            .unwrap();
        assert_eq!(
            http.requests(),
            vec![get(&format!(
                "https://pricing.example/integration/v1/stores/{S}/labels/products?filter=missingPriceTag&limit=25"
            ))]
        );
    }

    #[test]
    fn l1_c1c_products_filters_query_and_cursor() {
        let (http, adapter) = adapter();
        let filters = [
            LabelFilter::NotPrinted,
            LabelFilter::Printed,
            LabelFilter::All,
        ];
        for filter in filters {
            http.reply(200, fixture(PRODUCTS));
            adapter
                .products(S, filter, Some("teak bord & co"), Some("a+b/c="), 500)
                .unwrap();
        }
        let base = format!("https://pricing.example/integration/v1/stores/{S}/labels/products?");
        let tail = "q=teak%20bord%20%26%20co&cursor=a%2Bb%2Fc%3D&limit=500";
        let urls: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
        assert_eq!(
            urls,
            vec![
                format!("{base}filter=notPrinted&{tail}"),
                format!("{base}filter=printed&{tail}"),
                format!("{base}{tail}"),
            ]
        );
    }

    #[test]
    fn l1_c1d_nullable_and_absent_product_fields() {
        let (http, adapter) = adapter();
        let mut page = products_with(|p| {
            for key in [
                "sku",
                "productType",
                "priceMinor",
                "labelPrintedAt",
                "createdAt",
            ] {
                p[key] = Json::Null;
            }
            let fields = p.as_object_mut().unwrap();
            fields.remove("thumbUrl");
            fields.remove("labelResetReason");
            fields.insert("hasPrice".into(), json!(false));
        });
        page["nextCursor"] = Json::Null;
        http.reply(200, page);
        let page = adapter
            .products(S, LabelFilter::All, None, None, 25)
            .unwrap();
        let p = &page.products[0];
        assert_eq!(p.sku, None);
        assert_eq!(p.product_type, None);
        assert_eq!(p.price_minor, None);
        assert_eq!(p.created_at, None);
        assert_eq!(p.thumb_url, None);
        assert_eq!(
            p.state,
            LabelState {
                status: LabelStatus::NotPrinted,
                printed_at: None,
                has_price: Some(false),
                reset: None,
            }
        );
        assert_eq!(page.next_cursor, None);
    }

    #[test]
    fn l1_c1e_printed_product_with_its_time() {
        let (http, adapter) = adapter();
        let page = products_with(|p| {
            p["labelStatus"] = json!("printed");
            p["labelPrintedAt"] = json!("2026-11-20T09:12:00Z");
            p["labelResetReason"] = Json::Null;
        });
        assert_eq!(
            first_state(&http, &adapter, page),
            LabelState {
                status: LabelStatus::Printed,
                printed_at: Some(ms("2026-11-20T09:12:00Z")),
                has_price: Some(true),
                reset: None,
            }
        );
    }

    #[test]
    fn l1_c1f_reset_reasons() {
        let (http, adapter) = adapter();
        let reasons: Vec<Option<ResetReason>> = ["sku_changed", "barcode_changed", "something_new"]
            .into_iter()
            .map(|word| {
                let page = products_with(|p| p["labelResetReason"] = json!(word));
                first_state(&http, &adapter, page).reset
            })
            .collect();
        assert_eq!(
            reasons,
            vec![
                Some(ResetReason::SkuChanged),
                Some(ResetReason::BarcodeChanged),
                None
            ]
        );
    }

    #[test]
    fn l1_c1g_unknown_label_status_is_a_bad_reply() {
        let (http, adapter) = adapter();
        http.reply(200, products_with(|p| p["labelStatus"] = json!("queued")));
        assert!(matches!(
            adapter.products(S, LabelFilter::All, None, None, 25),
            Err(LabelsError::BadReply(_))
        ));
    }

    #[test]
    fn l1_c2a_status_maps_known_and_unknown_products() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(STATUS));
        let states = adapter.status(S, &[PRODUCT.into(), "999".into()]).unwrap();
        assert_eq!(
            states,
            vec![
                Some(LabelState {
                    status: LabelStatus::Printed,
                    printed_at: Some(ms("2026-11-20T09:12:00Z")),
                    has_price: Some(true),
                    reset: None,
                }),
                None
            ]
        );
    }

    #[test]
    fn l1_c2b_status_sends_one_get_with_the_ids() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(STATUS));
        adapter.status(S, &[PRODUCT.into(), "999".into()]).unwrap();
        assert_eq!(
            http.requests(),
            vec![get(&format!(
                "https://pricing.example/integration/v1/stores/{S}/labels/status?productIds={PRODUCT},999"
            ))]
        );
    }

    #[test]
    fn l1_c2c_status_with_unknown_price() {
        let (http, adapter) = adapter();
        let mut entry = fixture(STATUS)[0].clone();
        entry["label"]["hasPrice"] = Json::Null;
        entry["label"]["labelStatus"] = json!("notPrinted");
        entry["label"]["labelPrintedAt"] = Json::Null;
        entry["label"]["resetReason"] = json!("price_changed");
        http.reply(200, json!([entry]));
        assert_eq!(
            adapter.status(S, &[PRODUCT.into()]).unwrap(),
            vec![Some(LabelState {
                status: LabelStatus::NotPrinted,
                printed_at: None,
                has_price: None,
                reset: Some(ResetReason::PriceChanged),
            })]
        );
    }

    #[test]
    fn l1_c2d_status_of_no_products_asks_nothing() {
        let (http, adapter) = adapter();
        assert_eq!(adapter.status(S, &[]), Ok(vec![]));
        assert!(http.requests().is_empty());
    }

    #[test]
    fn l1_c2e_status_with_a_missing_entry_is_a_bad_reply() {
        let (http, adapter) = adapter();
        http.reply(200, json!([fixture(STATUS)[0].clone()]));
        assert!(matches!(
            adapter.status(S, &["1".into(), "2".into()]),
            Err(LabelsError::BadReply(_))
        ));
    }

    #[test]
    fn l1_c3a_templates_maps_the_fixture() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(TEMPLATES));
        assert_eq!(
            adapter.templates(S).unwrap(),
            vec![LabelTemplate {
                id: "tpl-1".into(),
                name: "Label 70×100".into(),
                width_mm: 70.0,
                height_mm: 100.0,
                is_default: true,
                product_types: vec!["Test tables".into(), "Test chairs".into()],
            }]
        );
        assert_eq!(
            http.requests(),
            vec![get(&format!(
                "https://pricing.example/integration/v1/stores/{S}/labels/templates"
            ))]
        );
    }

    #[test]
    fn l1_c3b_no_templates() {
        let (http, adapter) = adapter();
        http.reply(200, json!([]));
        assert_eq!(adapter.templates(S), Ok(vec![]));
    }

    #[test]
    fn l1_c4a_invalid_key_is_not_allowed() {
        let (http, adapter) = adapter();
        http.reply(401, envelope("invalid_api_key", "No key."));
        assert_eq!(adapter.templates(S), Err(LabelsError::NotAllowed));
    }

    #[test]
    fn l1_c4b_missing_scope_is_not_allowed() {
        let (http, adapter) = adapter();
        http.reply(403, envelope("insufficient_scope", "Missing scope."));
        assert_eq!(
            adapter.status(S, &["1".into()]),
            Err(LabelsError::NotAllowed)
        );
    }

    #[test]
    fn l1_c4c_unknown_store_is_store_gone() {
        let (http, adapter) = adapter();
        http.reply(404, envelope("not_found", "Not found."));
        assert_eq!(
            adapter.products(S, LabelFilter::All, None, None, 25),
            Err(LabelsError::StoreGone)
        );
    }

    #[test]
    fn l1_c4d_list_unavailable() {
        let (http, adapter) = adapter();
        http.reply(503, envelope("product_list_unavailable", "Not yet."));
        assert_eq!(
            adapter.products(S, LabelFilter::All, None, None, 25),
            Err(LabelsError::ListUnavailable)
        );
    }

    #[test]
    fn l1_c4e_server_errors_are_unreachable() {
        let (http, adapter) = adapter();
        http.reply(500, envelope("internal_error", "Oops."))
            .reply(502, json!("<html>Bad Gateway</html>"));
        for _ in 0..2 {
            assert!(matches!(
                adapter.products(S, LabelFilter::All, None, None, 25),
                Err(LabelsError::Unreachable(_))
            ));
        }
    }

    #[test]
    fn l1_c4f_transport_errors_are_unreachable() {
        let (http, adapter) = adapter();
        http.fail(Error::Timeout)
            .fail(Error::Provider("http: connection refused".into()));
        for _ in 0..2 {
            assert!(matches!(
                adapter.status(S, &["1".into()]),
                Err(LabelsError::Unreachable(_))
            ));
        }
    }

    #[test]
    fn l1_c4g_unexpected_shapes_are_bad_replies() {
        let (http, adapter) = adapter();
        http.reply(200, json!({"unexpected": true}))
            .reply(200, json!("<html>"));
        for _ in 0..2 {
            assert!(matches!(
                adapter.templates(S),
                Err(LabelsError::BadReply(_))
            ));
        }
    }

    #[test]
    fn l1_c4h_validation_error_is_rejected_with_its_message() {
        let (http, adapter) = adapter();
        http.reply(
            422,
            json!({"error": {"code": "validation_error", "message": "Bad cursor."}}),
        );
        assert_eq!(
            adapter.products(S, LabelFilter::All, None, None, 25),
            Err(LabelsError::Rejected {
                code: "validation_error".into(),
                message: "Bad cursor.".into(),
            })
        );
    }

    #[test]
    fn l1_c4i_unknown_fields_are_ignored() {
        let (http, adapter) = adapter();
        let mut page = fixture(PRODUCTS);
        page["hasMore"] = json!(true);
        page["items"][0]["vendor"] = json!("x");
        page["counts"]["archived"] = json!(3);
        http.reply(200, page);
        assert_eq!(
            adapter
                .products(S, LabelFilter::MissingPriceTag, None, None, 25)
                .unwrap(),
            fixture_page()
        );
    }

    fn describe(error: LabelsError, shop: Option<&str>) -> String {
        LabelsProvider::describe(&adapter().1, &error, shop)
    }

    #[test]
    fn l1_c5a_not_allowed_sentence() {
        assert_eq!(
            describe(LabelsError::NotAllowed, Some("shop-a.myshopify.com")),
            "Session Manager isn't allowed to print labels. Ask a Pricing admin."
        );
    }

    #[test]
    fn l1_c5b_store_gone_sentence() {
        assert_eq!(
            describe(LabelsError::StoreGone, Some("shop-a.myshopify.com")),
            "This key is no longer tied to shop-a.myshopify.com"
        );
        assert_eq!(
            describe(LabelsError::StoreGone, None),
            "This key is no longer tied to the connected shop"
        );
    }

    #[test]
    fn l1_c5c_no_products_sentence() {
        assert_eq!(
            describe(LabelsError::NoProducts, None),
            "None of these products were found in Pricing."
        );
    }

    #[test]
    fn l1_c5d_renderer_unavailable_sentence() {
        assert_eq!(
            describe(LabelsError::RendererUnavailable, None),
            "Labels can't be generated right now. Try again in a moment."
        );
    }

    #[test]
    fn l1_c5e_expired_sentence() {
        assert_eq!(
            describe(LabelsError::Expired, None),
            "This file has expired. Generate the labels again."
        );
    }

    #[test]
    fn l1_c5f_list_unavailable_sentence() {
        assert_eq!(
            describe(LabelsError::ListUnavailable, None),
            "The product list isn't available right now."
        );
    }

    #[test]
    fn l1_c5g_unreachable_sentence() {
        assert_eq!(
            describe(LabelsError::Unreachable("x".into()), None),
            "Pricing can't be reached right now."
        );
    }

    #[test]
    fn l1_c5h_bad_reply_sentence() {
        assert_eq!(
            describe(LabelsError::BadReply("x".into()), None),
            "Pricing answered in a way this version does not understand"
        );
    }

    #[test]
    fn l1_c5i_rejected_sentence_is_the_message_as_given() {
        assert_eq!(
            describe(
                LabelsError::Rejected {
                    code: "no_template".into(),
                    message: "No template applies.".into(),
                },
                None
            ),
            "No template applies."
        );
    }

    #[test]
    fn l1_c6a_product_id_of_a_product_gid() {
        assert_eq!(
            adapter()
                .1
                .product_id_of("gid://shopify/Product/15531808096409")
                .as_deref(),
            Some(PRODUCT)
        );
    }

    #[test]
    fn l1_c6b_product_id_of_anything_else_is_none() {
        let (_, a) = adapter();
        for id in [
            "gid://shopify/ProductVariant/15531808096409",
            "15531808096409",
            "gid://shopify/Product/",
            "gid://shopify/Product/12a",
            "product-3",
        ] {
            assert_eq!(a.product_id_of(id), None, "{id}");
        }
    }

    #[test]
    fn l1_c6c_commerce_id_of() {
        let (_, a) = adapter();
        assert_eq!(
            a.commerce_id_of(PRODUCT).as_deref(),
            Some("gid://shopify/Product/15531808096409")
        );
        for id in ["", "12a", "gid://shopify/Product/1"] {
            assert_eq!(a.commerce_id_of(id), None, "{id}");
        }
    }

    // ---------- L1·U2: plan, preview, run, download ----------

    const PLAN: &str = include_str!("fixtures/labels_plan.json");
    const RUN: &str = include_str!("fixtures/labels_run.json");
    const PDF_BYTES: &[u8] = b"%PDF-1.4 test label";
    const BARCODE: &str = "Barcode: The product's variant has no barcode in Shopify.";

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|id| (*id).to_owned()).collect()
    }

    fn envelope_bytes(code: &str, message: &str) -> Vec<u8> {
        envelope(code, message).to_string().into_bytes()
    }

    fn plan_with(items: Vec<Json>) -> Json {
        let mut plan = fixture(PLAN);
        plan["items"] = json!(items);
        plan
    }

    fn run_reply(http: &Recording, run: &Json) {
        http.raw(201, Some("application/json"), run.to_string().as_bytes());
    }

    fn artifact_path() -> String {
        format!("/integration/v1/stores/{S}/labels/runs/r1/artifacts/a1")
    }

    #[test]
    fn l1_c7a_plan_posts_product_ids_only() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(PLAN));
        adapter.plan(S, &ids(&["1", "2"])).unwrap();
        assert_eq!(
            http.requests(),
            vec![PricingRequest {
                method: Method::Post,
                url: format!("https://pricing.example/integration/v1/stores/{S}/labels/plan"),
                bearer: Some("k1".into()),
                body: Some(json!({"items": [{"productId": "1"}, {"productId": "2"}]})),
            }]
        );
    }

    #[test]
    fn l1_c7b_plan_maps_items_unresolved_summary_and_sizes() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(PLAN));
        let plan = adapter.plan(S, &ids(&["1", "2"])).unwrap();
        let first = &plan.items[0];
        assert_eq!(first.position, 0);
        assert_eq!(first.outcome, CheckOutcome::MarkPrinted);
        assert_eq!(
            first.template,
            Some(PlanTemplate {
                id: "tpl-1".into(),
                name: "Label 70×100".into(),
                width_mm: 70.0,
                height_mm: 100.0,
            })
        );
        assert_eq!(first.price_minor, Some(890000));
        assert_eq!(first.status, LabelStatus::NotPrinted);
        assert!(first.issues.is_empty());
        assert!(first.notes.is_empty());
        assert_eq!(plan.items[1].position, 1);
        assert_eq!(plan.items[1].outcome, CheckOutcome::NoPrice);
        assert_eq!(plan.items[1].price_minor, None);
        assert_eq!(plan.items[2].position, 2);
        assert_eq!(plan.items[2].outcome, CheckOutcome::Fail);
        assert_eq!(plan.items[2].issues, vec![BARCODE.to_owned()]);
        assert_eq!(
            plan.unresolved,
            vec![Unresolved {
                product_id: None,
                sku: Some("NOPE".into()),
                message: "No product with SKU 'NOPE' in this store.".into(),
            }]
        );
        assert_eq!(
            plan.summary,
            PlanSummary {
                items: 3,
                ready: 2,
                failing: 1,
                mark_printed: 1,
                reprint: 0,
                no_price: 1,
            }
        );
        assert_eq!(
            plan.sizes,
            vec![LabelSize {
                width_mm: 70.0,
                height_mm: 100.0,
                count: 2,
            }]
        );
    }

    #[test]
    fn l1_c7c_plan_reprint_and_nullable_template_and_title() {
        let (http, adapter) = adapter();
        let base = fixture(PLAN)["items"][0].clone();
        let mut reprint = base.clone();
        reprint["outcome"] = json!("reprint");
        reprint["labelStatus"] = json!("printed");
        reprint["labelPrintedAt"] = json!("2026-11-20T09:12:00Z");
        let mut bare = base;
        bare["template"] = Json::Null;
        bare["title"] = Json::Null;
        bare["notes"] = json!([{"code": "url_not_published", "message": "M"}]);
        http.reply(200, plan_with(vec![reprint, bare]));
        let plan = adapter.plan(S, &ids(&["1", "2"])).unwrap();
        assert_eq!(plan.items[0].outcome, CheckOutcome::Reprint);
        assert_eq!(plan.items[0].status, LabelStatus::Printed);
        assert_eq!(plan.items[0].printed_at, Some(ms("2026-11-20T09:12:00Z")));
        assert_eq!(plan.items[1].template, None);
        assert_eq!(plan.items[1].title, None);
        assert_eq!(
            plan.items[1].notes,
            vec![PlanNote {
                kind: NoteKind::NotPublished,
                message: "M".into(),
                link: None,
            }]
        );
    }

    // L4·C3 — the notes keep their kind and their address (MC-L2.15).

    const ADDR: &str = "https://shop.example/search?q=TEST-1802";
    const SENT: &str = "Not published to the Online Store yet; the QR code opens https://shop.example/search?q=TEST-1802.";

    /// The plan answer of one item carrying `notes`, mapped by the adapter: that item's notes.
    fn notes_of(notes: Json) -> Vec<PlanNote> {
        let (http, adapter) = adapter();
        let mut item = fixture(PLAN)["items"][0].clone();
        item["notes"] = notes;
        http.reply(200, plan_with(vec![item]));
        adapter.plan(S, &ids(&["1"])).unwrap().items[0]
            .notes
            .clone()
    }

    fn note(kind: NoteKind, message: &str, link: Option<&str>) -> PlanNote {
        PlanNote {
            kind,
            message: message.into(),
            link: link.map(Into::into),
        }
    }

    #[test]
    fn l4_c3a_a_not_published_note_keeps_its_address() {
        assert_eq!(
            notes_of(json!([{"code": "url_not_published", "message": SENT}])),
            vec![note(NoteKind::NotPublished, SENT, Some(ADDR))]
        );
    }

    #[test]
    fn l4_c3b_a_not_published_note_without_an_address_has_no_link() {
        assert_eq!(
            notes_of(json!([{"code": "url_not_published", "message": "M"}])),
            vec![note(NoteKind::NotPublished, "M", None)]
        );
    }

    #[test]
    fn l4_c3c_any_other_note_is_other_without_a_link() {
        let message = "See https://shop.example/a.";
        assert_eq!(
            notes_of(json!([{"code": "something_else", "message": message}])),
            vec![note(NoteKind::Other, message, None)]
        );
    }

    #[test]
    fn l4_c3d_the_address_is_an_http_word_with_its_trailing_marks_removed() {
        let http_message = "Opens http://shop.example/a, soon.";
        let ftp_message = "Opens ftp://shop.example/a.";
        assert_eq!(
            notes_of(json!([{"code": "url_not_published", "message": http_message}])),
            vec![note(
                NoteKind::NotPublished,
                http_message,
                Some("http://shop.example/a")
            )]
        );
        assert_eq!(
            notes_of(json!([{"code": "url_not_published", "message": ftp_message}])),
            vec![note(NoteKind::NotPublished, ftp_message, None)]
        );
    }

    #[test]
    fn l1_c7d_plan_with_no_products() {
        let (http, adapter) = adapter();
        http.reply(422, envelope("no_products", "Nothing resolved."));
        assert_eq!(adapter.plan(S, &ids(&["1"])), Err(LabelsError::NoProducts));
    }

    #[test]
    fn l1_c7e_plan_with_an_unknown_outcome_is_a_bad_reply() {
        let (http, adapter) = adapter();
        let mut item = fixture(PLAN)["items"][0].clone();
        item["outcome"] = json!("skip");
        http.reply(200, plan_with(vec![item]));
        assert!(matches!(
            adapter.plan(S, &ids(&["1"])),
            Err(LabelsError::BadReply(_))
        ));
    }

    #[test]
    fn l1_c7f_plan_errors() {
        let (http, adapter) = adapter();
        http.reply(403, envelope("insufficient_scope", "Missing scope."))
            .reply(404, envelope("not_found", "Not found."))
            .reply(500, envelope("internal_error", "Oops."));
        assert_eq!(adapter.plan(S, &ids(&["1"])), Err(LabelsError::NotAllowed));
        assert_eq!(adapter.plan(S, &ids(&["1"])), Err(LabelsError::StoreGone));
        assert!(matches!(
            adapter.plan(S, &ids(&["1"])),
            Err(LabelsError::Unreachable(_))
        ));
    }

    #[test]
    fn l1_c8a_preview_answers_the_pdf_bytes() {
        let (http, adapter) = adapter();
        http.raw(200, Some("application/pdf"), PDF_BYTES);
        assert_eq!(adapter.preview(S, "1"), Ok(PDF_BYTES.to_vec()));
        assert_eq!(
            http.fetches(),
            vec![(
                PricingRequest {
                    method: Method::Post,
                    url: format!(
                        "https://pricing.example/integration/v1/stores/{S}/labels/preview"
                    ),
                    bearer: Some("k1".into()),
                    body: Some(json!({"productId": "1"})),
                },
                FILE_TIMEOUT
            )]
        );
        assert!(http.requests().is_empty());
    }

    #[test]
    fn l1_c8b_preview_that_is_not_a_pdf_is_a_bad_reply() {
        let (http, adapter) = adapter();
        http.raw(200, Some("application/json"), &envelope_bytes("x", "y"))
            .raw(200, Some("application/pdf"), b"");
        for _ in 0..2 {
            assert!(matches!(
                adapter.preview(S, "1"),
                Err(LabelsError::BadReply(_))
            ));
        }
    }

    #[test]
    fn l1_c8c_preview_refusals_keep_their_message() {
        let (http, adapter) = adapter();
        let cases = [
            (404, "sku_not_found", "M1"),
            (409, "sku_ambiguous", "M2"),
            (422, "no_template", "M3"),
        ];
        for (status, code, message) in cases {
            http.raw(
                status,
                Some("application/json"),
                &envelope_bytes(code, message),
            );
            assert_eq!(
                adapter.preview(S, "1"),
                Err(LabelsError::Rejected {
                    code: code.into(),
                    message: message.into(),
                })
            );
        }
    }

    #[test]
    fn l1_c8d_preview_renderer_unavailable() {
        let (http, adapter) = adapter();
        http.raw(
            503,
            Some("application/json"),
            &envelope_bytes("renderer_unavailable", "Down."),
        );
        assert_eq!(
            adapter.preview(S, "1"),
            Err(LabelsError::RendererUnavailable)
        );
    }

    #[test]
    fn l1_c8e_preview_not_allowed_and_timeout() {
        let (http, adapter) = adapter();
        http.raw(
            403,
            Some("application/json"),
            &envelope_bytes("insufficient_scope", "Missing scope."),
        )
        .raw_fail(Error::Timeout);
        assert_eq!(adapter.preview(S, "1"), Err(LabelsError::NotAllowed));
        assert!(matches!(
            adapter.preview(S, "1"),
            Err(LabelsError::Unreachable(_))
        ));
    }

    #[test]
    fn l1_c9a_run_posts_ids_and_the_key_through_fetch() {
        let (http, adapter) = adapter();
        run_reply(&http, &fixture(RUN));
        adapter.run(S, &ids(&["1", "2"]), "sm-0123456789").unwrap();
        assert_eq!(
            http.fetches(),
            vec![(
                PricingRequest {
                    method: Method::Post,
                    url: format!("https://pricing.example/integration/v1/stores/{S}/labels/runs"),
                    bearer: Some("k1".into()),
                    body: Some(json!({
                        "items": [{"productId": "1"}, {"productId": "2"}],
                        "idempotencyKey": "sm-0123456789"
                    })),
                },
                RUN_TIMEOUT
            )]
        );
        assert!(http.requests().is_empty());
    }

    #[test]
    fn l1_c9b_run_maps_the_answer() {
        let (http, adapter) = adapter();
        run_reply(&http, &fixture(RUN));
        assert_eq!(
            adapter.run(S, &ids(&["1", "2"]), "sm-0123456789").unwrap(),
            LabelRun {
                id: "r1".into(),
                status: RunStatus::Partial,
                created_at: ms("2026-11-20T09:30:00Z"),
                counts: RunCounts {
                    items: 2,
                    rendered: 1,
                    failed: 1,
                    marked_printed: 1,
                    reprinted: 0,
                    no_price: 0,
                },
                artifacts: vec![LabelArtifact {
                    id: "a1".into(),
                    width_mm: 70.0,
                    height_mm: 100.0,
                    page_count: 1,
                    filename: "labels-2026-11-20.pdf".into(),
                    size_bytes: 18211,
                    download_url: artifact_path(),
                    expired: false,
                }],
                items: vec![
                    RunItem {
                        product_id: PRODUCT.into(),
                        sku: Some("TEST-1802".into()),
                        title: Some("Test table".into()),
                        rendered: true,
                        result: Some(RunResult::MarkedPrinted),
                        error: None,
                    },
                    RunItem {
                        product_id: "15531808096411".into(),
                        sku: Some("TEST-1808".into()),
                        title: Some("Test lamp".into()),
                        rendered: false,
                        result: None,
                        error: Some(BARCODE.into()),
                    },
                ],
                unresolved: vec![],
            }
        );
    }

    #[test]
    fn l1_c9c_run_variants_parse() {
        let (http, adapter) = adapter();
        let mut rendered = fixture(RUN);
        rendered["status"] = json!("rendered");
        rendered["items"][0]["lifecycleResult"] = json!("reprinted");
        let mut failed = fixture(RUN);
        failed["status"] = json!("failed");
        failed["artifacts"] = json!([]);
        let mut no_price = fixture(RUN);
        no_price["items"][0]["lifecycleResult"] = json!("no_price");
        let mut without_unresolved = fixture(RUN);
        without_unresolved
            .as_object_mut()
            .unwrap()
            .remove("unresolved");
        let mut none_word = fixture(RUN);
        none_word["items"][1]["outcome"] = json!("failed");
        none_word["items"][1]["lifecycleResult"] = json!("none");
        for run in [
            &rendered,
            &failed,
            &no_price,
            &without_unresolved,
            &none_word,
        ] {
            run_reply(&http, run);
        }
        let key = "sm-0123456789";
        let r = adapter.run(S, &ids(&["1"]), key).unwrap();
        assert_eq!(r.status, RunStatus::Rendered);
        assert_eq!(r.items[0].result, Some(RunResult::Reprinted));
        let r = adapter.run(S, &ids(&["1"]), key).unwrap();
        assert_eq!(r.status, RunStatus::Failed);
        assert!(r.artifacts.is_empty());
        let r = adapter.run(S, &ids(&["1"]), key).unwrap();
        assert_eq!(r.items[0].result, Some(RunResult::NoPrice));
        let r = adapter.run(S, &ids(&["1"]), key).unwrap();
        assert_eq!(r.unresolved, vec![]);
        let r = adapter.run(S, &ids(&["1"]), key).unwrap();
        assert_eq!(r.items[1].result, None);
        assert!(!r.items[1].rendered);
    }

    #[test]
    fn l1_c9d_run_with_no_products() {
        let (http, adapter) = adapter();
        http.raw(
            422,
            Some("application/json"),
            &envelope_bytes("no_products", "Nothing resolved."),
        );
        assert_eq!(
            adapter.run(S, &ids(&["1"]), "sm-0123456789"),
            Err(LabelsError::NoProducts)
        );
    }

    #[test]
    fn l1_c9e_run_retryable_errors() {
        let (http, adapter) = adapter();
        http.raw(
            503,
            Some("application/json"),
            &envelope_bytes("renderer_unavailable", "Down."),
        )
        .raw(
            500,
            Some("application/json"),
            &envelope_bytes("internal_error", "Oops."),
        )
        .raw_fail(Error::Timeout);
        let key = "sm-0123456789";
        assert_eq!(
            adapter.run(S, &ids(&["1"]), key),
            Err(LabelsError::RendererUnavailable)
        );
        for _ in 0..2 {
            assert!(matches!(
                adapter.run(S, &ids(&["1"]), key),
                Err(LabelsError::Unreachable(_))
            ));
        }
    }

    #[test]
    fn l1_c9f_run_errors_that_are_not_retried() {
        let (http, adapter) = adapter();
        http.raw(
            403,
            Some("application/json"),
            &envelope_bytes("insufficient_scope", "Missing scope."),
        )
        .raw(
            404,
            Some("application/json"),
            &envelope_bytes("not_found", "Gone."),
        )
        .raw(
            422,
            Some("application/json"),
            &envelope_bytes("validation_error", "M"),
        );
        let key = "sm-0123456789";
        assert_eq!(
            adapter.run(S, &ids(&["1"]), key),
            Err(LabelsError::NotAllowed)
        );
        assert_eq!(
            adapter.run(S, &ids(&["1"]), key),
            Err(LabelsError::StoreGone)
        );
        assert_eq!(
            adapter.run(S, &ids(&["1"]), key),
            Err(LabelsError::Rejected {
                code: "validation_error".into(),
                message: "M".into(),
            })
        );
    }

    #[test]
    fn l1_c9g_run_answer_that_is_not_a_run_is_a_bad_reply() {
        let (http, adapter) = adapter();
        http.raw(201, Some("application/json"), br#"{"id": "r1"}"#);
        assert!(matches!(
            adapter.run(S, &ids(&["1"]), "sm-0123456789"),
            Err(LabelsError::BadReply(_))
        ));
    }

    #[test]
    fn l1_c10a_download_answers_the_pdf_bytes() {
        let (http, adapter) = adapter();
        http.raw(200, Some("application/pdf"), PDF_BYTES);
        assert_eq!(
            adapter.download(S, &artifact_path()),
            Ok(PDF_BYTES.to_vec())
        );
        assert_eq!(
            http.fetches(),
            vec![(
                get(&format!("https://pricing.example{}", artifact_path())),
                FILE_TIMEOUT
            )]
        );
    }

    #[test]
    fn l1_c10b_download_refuses_any_other_address_without_a_request() {
        let (http, adapter) = adapter();
        let refused = [
            format!("https://evil.example/integration/v1/stores/{S}/labels/runs/r1/artifacts/a1"),
            "//evil.example/x".to_owned(),
            "/integration/v1/stores/OTHER/labels/runs/r1/artifacts/a1".to_owned(),
            format!("/integration/v1/stores/{S}/labels/runs/../../cases"),
            format!("/integration/v1/stores/{S}/labels/runs/r1/artifacts/a1?x=1"),
            format!("/integration/v1/stores/{S}/cases"),
        ];
        for path in &refused {
            assert!(
                matches!(adapter.download(S, path), Err(LabelsError::BadReply(_))),
                "{path}"
            );
        }
        assert!(http.fetches().is_empty());
        assert!(http.requests().is_empty());
    }

    #[test]
    fn l1_c10c_expired_and_missing_files() {
        let (http, adapter) = adapter();
        http.raw(
            410,
            Some("application/json"),
            &envelope_bytes("artifact_expired", "Expired."),
        )
        .raw(
            404,
            Some("application/json"),
            &envelope_bytes("not_found", "Gone."),
        );
        for _ in 0..2 {
            assert_eq!(
                adapter.download(S, &artifact_path()),
                Err(LabelsError::Expired)
            );
        }
    }

    #[test]
    fn l1_c10d_download_that_is_not_a_pdf_is_a_bad_reply() {
        let (http, adapter) = adapter();
        http.raw(200, Some("application/pdf"), b"<html>not a pdf</html>");
        assert!(matches!(
            adapter.download(S, &artifact_path()),
            Err(LabelsError::BadReply(_))
        ));
    }

    #[test]
    fn l1_c10e_download_not_allowed_and_server_error() {
        let (http, adapter) = adapter();
        http.raw(
            403,
            Some("application/json"),
            &envelope_bytes("insufficient_scope", "Missing scope."),
        )
        .raw(
            500,
            Some("application/json"),
            &envelope_bytes("internal_error", "Oops."),
        );
        assert_eq!(
            adapter.download(S, &artifact_path()),
            Err(LabelsError::NotAllowed)
        );
        assert!(matches!(
            adapter.download(S, &artifact_path()),
            Err(LabelsError::Unreachable(_))
        ));
    }

    // ---------- L1·U3: label events in the pull ----------

    use psm_core::{
        domain::conversation::{EventKind, ProviderEvent},
        domain::label::{LabelChange, LabelChangeKind},
        ports::conversations::ConversationsProvider,
    };

    const EVENTS_LABELS: &str = include_str!("fixtures/events_labels.json");

    fn label_event(kind: &str, envelope_product: Option<&str>, data: Json) -> Json {
        json!({"id": 1860, "type": kind, "storeId": S, "productId": envelope_product,
               "caseId": null, "actorId": null, "data": data, "at": "2026-11-20T09:30:01Z"})
    }

    fn kinds(http: &Recording, adapter: &BeyoPricingAdapter, events: Vec<Json>) -> Vec<EventKind> {
        http.reply(200, json!({"events": events, "nextCursor": 1860}));
        adapter
            .events_after(S, 0, 500)
            .unwrap()
            .events
            .into_iter()
            .map(|e| e.kind)
            .collect()
    }

    #[test]
    fn l1_c11a_label_events_map_to_label_changes() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(EVENTS_LABELS));
        let page = adapter.events_after(S, 1849, 500).unwrap();
        assert_eq!(
            page.events,
            vec![
                ProviderEvent {
                    id: 1850,
                    kind: EventKind::Label(LabelChange {
                        product_id: PRODUCT.into(),
                        at: ms("2026-11-20T09:30:01Z"),
                        kind: LabelChangeKind::Printed {
                            price_minor: Some(890000)
                        },
                    }),
                    conversation_id: None,
                    for_us: false,
                },
                ProviderEvent {
                    id: 1851,
                    kind: EventKind::Label(LabelChange {
                        product_id: PRODUCT.into(),
                        at: ms("2026-11-20T10:05:00Z"),
                        kind: LabelChangeKind::Reset {
                            reason: Some(ResetReason::PriceChanged)
                        },
                    }),
                    conversation_id: None,
                    for_us: false,
                },
            ]
        );
    }

    #[test]
    fn l1_c11b_reset_reasons_of_events() {
        let (http, adapter) = adapter();
        let events = ["sku_changed", "barcode_changed", "other"]
            .into_iter()
            .map(|reason| {
                label_event(
                    "label.reset",
                    Some("7"),
                    json!({"productId": "7", "sku": "TEST-1", "reason": reason}),
                )
            })
            .collect();
        let reasons: Vec<Option<ResetReason>> = kinds(&http, &adapter, events)
            .into_iter()
            .map(|kind| match kind {
                EventKind::Label(LabelChange {
                    kind: LabelChangeKind::Reset { reason },
                    ..
                }) => reason,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            reasons,
            vec![
                Some(ResetReason::SkuChanged),
                Some(ResetReason::BarcodeChanged),
                None
            ]
        );
    }

    #[test]
    fn l1_c11c_the_product_comes_from_data_else_the_envelope() {
        let (http, adapter) = adapter();
        let got = kinds(
            &http,
            &adapter,
            vec![
                label_event("label.printed", Some("7"), json!({"sku": "TEST-1"})),
                label_event("label.printed", None, json!({"sku": "TEST-1"})),
            ],
        );
        assert_eq!(
            got,
            vec![
                EventKind::Label(LabelChange {
                    product_id: "7".into(),
                    at: ms("2026-11-20T09:30:01Z"),
                    kind: LabelChangeKind::Printed { price_minor: None },
                }),
                EventKind::Other,
            ]
        );
    }

    #[test]
    fn l1_c11d_printed_without_a_price() {
        let (http, adapter) = adapter();
        let got = kinds(
            &http,
            &adapter,
            vec![label_event(
                "label.printed",
                Some("7"),
                json!({"productId": "7", "sku": "TEST-1", "runId": "r1"}),
            )],
        );
        assert!(matches!(
            &got[0],
            EventKind::Label(LabelChange {
                kind: LabelChangeKind::Printed { price_minor: None },
                ..
            })
        ));
    }
}
