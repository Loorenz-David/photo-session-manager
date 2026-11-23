//! `impl LabelsProvider for BeyoPricingAdapter` (book 09C §3; `master_plan_labels.md` §4 Adapter
//! paths, Words): Pricing's `/labels/…` calls over the same transport, server and key as the
//! conversations calls. **No label call maps to `ProviderError`** (ACC.5): a refused label call
//! must never mark the conversations connection refused, so every label answer goes through
//! [`label_error`] and never through the conversations `read_error` / `write_error`.

use super::adapter::{BeyoPricingAdapter, encode};
use super::label_types::{
    IntegrationLabelProduct, IntegrationLabelProductPage, IntegrationLabelStatus,
    IntegrationLabelTemplate, LabelStatusOut, WireLabelStatus,
};
use super::transport::{HttpReply, Method, PricingRequest};
use super::types::{ErrorBody, ErrorEnvelope};
use psm_core::{
    domain::label::{
        LabelCounts, LabelFilter, LabelProduct, LabelProductPage, LabelState, LabelStatus,
        LabelTemplate, LabelsError, ResetReason,
    },
    ports::labels::{LabelsProvider, LabelsResult},
};
use serde::de::DeserializeOwned;
use serde_json::Value as Json;

/// The commerce external id form of a product (Shopify's global id).
const COMMERCE_PRODUCT_PREFIX: &str = "gid://shopify/Product/";

/// Which call answered, for the statuses whose meaning depends on it (Adapter paths).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Call {
    /// List, status, templates, plan, runs.
    Read,
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
        adapter().1.describe(&error, shop)
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
}
