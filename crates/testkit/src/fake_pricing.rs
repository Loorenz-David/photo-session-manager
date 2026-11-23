//! `FakePricing` (`master_plan_conversations.md` §4, C1·U1 task 7): an in-memory Beyo Pricing
//! behind the adapter's own HTTP seam (`PricingHttp`), so every test runs the real
//! `BeyoPricingAdapter`. It serialises through the adapter's wire types (`types.rs`) and follows
//! the package's semantics: the unresolved filter, at most 200 cases, `after`/`limit` paging with
//! `nextCursor` unchanged on an empty page, `clientMutationId` returning the same message, a reply
//! moving its case to `answered`, 409 `case_resolved`, 422 for `resolved`, 404 for an unknown
//! store, 401 `invalid_api_key` for a wrong or missing bearer. Every request is recorded without
//! its bearer value. Nothing here touches the network.

use psm_core::{Error, Result};
use psm_integrations::conversations::beyo_pricing::{
    label_types::{
        IntegrationLabelCounts, IntegrationLabelProduct, IntegrationLabelProductPage,
        IntegrationLabelRun, IntegrationLabelStatus, IntegrationLabelTemplate, IntegrationPlan,
        IntegrationPlanIssue, IntegrationPlanItem, IntegrationPlanSize, IntegrationPlanSummary,
        IntegrationPlanTemplate, IntegrationRunCreate, IntegrationRunItemIn, IssueOut,
        LabelStatusOut, RunArtifactOut, RunItemOut, UnresolvedItem, WireLabelStatus,
        WirePlanOutcome, WireRunCounts, WireRunStatus,
    },
    transport::{HttpReply, Method, PricingHttp, PricingRequest, RawReply},
    types::{
        Actor, ErrorBody, ErrorEnvelope, EventPage, IntegrationCase, IntegrationCaseUpdate,
        IntegrationMessageCreate, IntegrationStore, Message, Quote, StoreEvent,
    },
};
use serde_json::{Value as Json, json};
use std::{
    collections::HashMap,
    sync::{Condvar, Mutex},
    time::Duration,
};

/// 2026-11-20T08:00:00Z: the fake's own clock starts here and moves one second per change.
const FAKE_EPOCH_MS: i64 = 1_795_161_600_000;

/// One recorded request (never the bearer value).
#[derive(Debug, Clone, PartialEq)]
pub struct FakeRequest {
    pub method: String,
    /// `https://host` of the address.
    pub server: String,
    /// The path after `/integration/v1`.
    pub path: String,
    pub query: String,
    pub body: Option<Json>,
    pub authorized: bool,
}

impl FakeRequest {
    /// `METHOD /path?query` (the path after `/integration/v1`).
    pub fn line(&self) -> String {
        if self.query.is_empty() {
            format!("{} {}", self.method, self.path)
        } else {
            format!("{} {}?{}", self.method, self.path, self.query)
        }
    }
}

/// L1 (`master_plan_labels.md` §4 Tests): one product of the fake's label state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LabelSpec {
    pub product_id: String,
    pub sku: Option<String>,
    pub title: String,
    pub product_type: Option<String>,
    pub price_minor: Option<i64>,
    /// Printed at this time (UTC ms); none = not printed.
    pub printed_at: Option<i64>,
    /// `price_changed | sku_changed | barcode_changed` while not printed.
    pub reset: Option<String>,
    pub thumb_url: Option<String>,
    pub created_at: Option<i64>,
    pub template: Option<String>,
}

fn iso(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

struct FakeCase {
    store_id: String,
    case: IntegrationCase,
}

struct Scripted {
    path_part: String,
    remaining: usize,
    /// L2: this many matching requests pass before the rule applies.
    skip: usize,
    status: u16,
    /// The envelope's `code` and `message` (none = the status's default code, "Scripted failure.").
    envelope: Option<(String, String)>,
}

#[derive(Default)]
struct State {
    accepted_keys: Vec<String>,
    stores: Vec<IntegrationStore>,
    cases: Vec<FakeCase>,
    events: Vec<StoreEvent>,
    next_event: i64,
    seconds: i64,
    next_id: u64,
    scripted: Vec<Scripted>,
    timeouts: usize,
    refusals: usize,
    drops: usize,
    stale_cursor: bool,
    redeliver: Vec<i64>,
    mutations: HashMap<(String, String), Message>,
    requests: Vec<FakeRequest>,
    labels: Vec<LabelSpec>,
    templates: Vec<IntegrationLabelTemplate>,
    deny_labels: bool,
    /// L2: `workFrom` of every list answer (none = the default `2026-11-19`).
    work_from: Option<Option<String>>,
    /// L2: the next list answer carrying a cursor answers that same cursor as `nextCursor`.
    repeat_cursor: bool,
    /// L2: products added by `add_label_products`.
    bulk: usize,
    /// L3: products that fail in the check and in the run (code, message).
    fail_label: HashMap<String, (String, String)>,
    /// L3: products that pass the check and fail in the run (code, message).
    fail_in_run: HashMap<String, (String, String)>,
    /// L3: the stored runs, one per idempotency key.
    runs: Vec<StoredRun>,
}

/// L3: one run the fake made, under its idempotency key.
struct StoredRun {
    key: String,
    product_ids: Vec<String>,
    created_at: i64,
    run: IntegrationLabelRun,
}

/// L3: what `FakePricing::runs` answers for one stored run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeRun {
    pub id: String,
    pub key: String,
    /// The products the run was asked for, in order.
    pub product_ids: Vec<String>,
    /// UTC ms.
    pub created_at: i64,
}

/// L3: one answer of the fake: JSON, or the bytes of a file.
enum Answer {
    Json(HttpReply),
    File(Vec<u8>),
}

/// L2: the `workFrom` the fake answers unless `set_work_from` changed it.
pub const FAKE_WORK_FROM: &str = "2026-11-19";

#[derive(Default)]
struct Hold {
    held: bool,
    waiting: usize,
}

#[derive(Default)]
pub struct FakePricing {
    state: Mutex<State>,
    hold: Mutex<Hold>,
    hold_changed: Condvar,
}

fn envelope(status: u16, code: &str, message: &str) -> HttpReply {
    HttpReply::new(
        status,
        serde_json::to_value(ErrorEnvelope {
            error: ErrorBody {
                code: code.into(),
                message: message.into(),
                details: None,
            },
            correlation_id: Some("fake".into()),
        })
        .unwrap(),
    )
}

/// L3: 422 `no_products` with `details.unresolved`.
fn no_products(unresolved: &[UnresolvedItem]) -> HttpReply {
    HttpReply::new(
        422,
        serde_json::to_value(ErrorEnvelope {
            error: ErrorBody {
                code: "no_products".into(),
                message: "None of the items resolved to a product.".into(),
                details: Some(json!({ "unresolved": unresolved })),
            },
            correlation_id: Some("fake".into()),
        })
        .unwrap(),
    )
}

fn code_for(status: u16) -> &'static str {
    match status {
        401 => "invalid_api_key",
        403 => "insufficient_scope",
        404 => "not_found",
        409 => "conflict",
        422 => "validation_error",
        500..=599 => "internal_error",
        _ => "bad_request",
    }
}

/// Serialises a wire value (through the adapter's types).
macro_rules! to_json {
    ($value:expr) => {
        serde_json::to_value($value).unwrap()
    };
}

fn decode(segment: &str) -> String {
    let bytes = segment.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 3 <= bytes.len()
            && let Ok(b) = u8::from_str_radix(&segment[i + 1..i + 3], 16)
        {
            out.push(b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

impl State {
    fn now(&mut self) -> String {
        self.seconds += 1;
        let ms = FAKE_EPOCH_MS + self.seconds * 1000;
        chrono::DateTime::from_timestamp_millis(ms)
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
    }

    fn id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}-{}", self.next_id)
    }

    fn case_mut(&mut self, case_id: &str) -> &mut FakeCase {
        self.cases
            .iter_mut()
            .find(|c| c.case.id == case_id)
            .unwrap_or_else(|| panic!("FakePricing: no case {case_id}"))
    }

    /// Appends an event for `case_id` with the case's standard `data` plus `extra`.
    fn log_case_event(&mut self, case_id: &str, kind: &str, extra: Json) -> i64 {
        let fake = self.case_mut(case_id);
        let store_id = fake.store_id.clone();
        let case = &fake.case;
        let mut data = json!({
            "caseId": case.id, "sku": case.sku, "recipient": case.recipient,
            "status": case.status, "blocking": case.blocking,
        });
        for (k, v) in extra.as_object().cloned().unwrap_or_default() {
            data[k] = v;
        }
        let product_id = case.product_id.clone();
        self.push_event(
            &store_id,
            kind,
            Some(case_id.to_owned()),
            Some(product_id),
            data,
        )
    }

    fn push_event(
        &mut self,
        store_id: &str,
        kind: &str,
        case_id: Option<String>,
        product_id: Option<String>,
        data: Json,
    ) -> i64 {
        self.next_event += 1;
        let at = self.now();
        let id = self.next_event;
        self.events.push(StoreEvent {
            id,
            kind: kind.into(),
            store_id: store_id.into(),
            product_id,
            case_id,
            actor_id: None,
            data: data.as_object().cloned().unwrap_or_default(),
            at,
        });
        id
    }

    fn new_message(&mut self, case_id: &str, author: Actor, text: &str) -> Message {
        let id = self.id("msg");
        let created_at = self.now();
        Message {
            id,
            case_id: case_id.into(),
            author,
            text: Some(text.into()),
            quote: None,
            reply_to: None,
            attachments: vec![],
            created_at,
            edited_at: None,
            deleted: false,
        }
    }

    fn create_case(
        &mut self,
        store_id: &str,
        sku: Option<&str>,
        topic: &str,
        recipient: &str,
    ) -> String {
        let case_id = self.id("case");
        // A30: as in Pricing, a product's questions share its product id (same store, same SKU).
        let existing = sku.and_then(|sku| {
            self.cases
                .iter()
                .find(|c| c.store_id == store_id && c.case.sku.as_deref() == Some(sku))
                .map(|c| c.case.product_id.clone())
        });
        let product_id =
            existing.unwrap_or_else(|| self.id("product").replace("product-", "15531808"));
        let created = self.now();
        let mut message = self.new_message(&case_id, pricer(), &format!("Question on {case_id}"));
        message.quote = Some(Quote {
            field: "condition".into(),
            field_label: "Condition".into(),
            text: "Possible desilvering at the lower edge.".into(),
        });
        let message_id = message.id.clone();
        self.cases.push(FakeCase {
            store_id: store_id.into(),
            case: IntegrationCase {
                id: case_id.clone(),
                product_id,
                sku: sku.map(str::to_owned),
                product_title: format!("Product {}", sku.unwrap_or("without SKU")),
                topic: topic.into(),
                recipient: recipient.into(),
                blocking: true,
                status: "open".into(),
                created_at: created.clone(),
                updated_at: created,
                messages: vec![message],
            },
        });
        self.log_case_event(&case_id, "case.created", json!({"topic": topic}));
        self.log_case_event(
            &case_id,
            "message.created",
            json!({"messageId": message_id, "authorKind": "user"}),
        );
        case_id
    }

    fn label_mut(&mut self, product_id: &str) -> &mut LabelSpec {
        if !self.labels.iter().any(|l| l.product_id == product_id) {
            self.labels.push(LabelSpec {
                product_id: product_id.into(),
                title: format!("Product {product_id}"),
                ..LabelSpec::default()
            });
        }
        self.labels
            .iter_mut()
            .find(|l| l.product_id == product_id)
            .unwrap()
    }

    /// Logs a label event for the product (`productId` in `data` and in the envelope) in the
    /// first store's log; answers the event id.
    fn log_label_event(&mut self, product_id: &str, kind: &str, extra: Json) -> i64 {
        let store_id = self
            .stores
            .first()
            .map(|s| s.id.clone())
            .expect("FakePricing: a store for the label event");
        let sku = self.label_mut(product_id).sku.clone();
        let mut data = json!({"productId": product_id, "sku": sku});
        for (k, v) in extra.as_object().cloned().unwrap_or_default() {
            data[k] = v;
        }
        self.push_event(&store_id, kind, None, Some(product_id.to_owned()), data)
    }

    fn label_status(&self, product_id: &str) -> IntegrationLabelStatus {
        match self.labels.iter().find(|l| l.product_id == product_id) {
            Some(l) => IntegrationLabelStatus {
                sku: l.sku.clone(),
                product_id: Some(l.product_id.clone()),
                label: Some(LabelStatusOut {
                    product_id: l.product_id.clone(),
                    label_status: if l.printed_at.is_some() {
                        WireLabelStatus::Printed
                    } else {
                        WireLabelStatus::NotPrinted
                    },
                    label_printed_at: l.printed_at.map(iso),
                    has_price: Some(l.price_minor.is_some()),
                    source: l.printed_at.map(|_| "export".into()),
                    reset_reason: l.reset.clone().filter(|_| l.printed_at.is_none()),
                    reset_at: None,
                    printed_price_minor: l.printed_at.and(l.price_minor),
                    printed_sku: l.printed_at.and(l.sku.clone()),
                }),
                error: None,
            },
            None => IntegrationLabelStatus {
                sku: None,
                product_id: Some(product_id.to_owned()),
                label: None,
                error: Some(ErrorBody {
                    code: "product_not_found".into(),
                    message: format!("No product with id '{product_id}' in this store."),
                    details: None,
                }),
            },
        }
    }

    /// L2: `GET …/labels/products` by the package's meaning: newest created first, `q` over SKU,
    /// title and product type (case-insensitive), `counts` over the searched list before the
    /// filter, `limit` 1–500, an opaque cursor (`o:<offset>`, percent-encoded on the wire).
    fn label_products(&mut self, params: &HashMap<&str, &str>) -> HttpReply {
        let limit: usize = match params.get("limit").and_then(|l| l.parse().ok()) {
            Some(l) if (1..=500).contains(&l) => l,
            _ => return envelope(422, "validation_error", "limit must be 1 to 500."),
        };
        let offset: usize = match params.get("cursor").map(|c| decode(c)) {
            None => 0,
            Some(c) => match c.strip_prefix("o:").and_then(|n| n.parse().ok()) {
                Some(n) => n,
                None => return envelope(422, "invalid_cursor", "The cursor is not valid."),
            },
        };
        let q = params
            .get("q")
            .map(|q| decode(&q.replace('+', " ")).to_lowercase());
        let mut list: Vec<&LabelSpec> = self.labels.iter().rev().collect();
        list.sort_by_key(|l| std::cmp::Reverse(l.created_at.unwrap_or(i64::MIN)));
        let list: Vec<&LabelSpec> = list
            .into_iter()
            .filter(|l| {
                q.as_deref().is_none_or(|q| {
                    [Some(&l.title), l.sku.as_ref(), l.product_type.as_ref()]
                        .into_iter()
                        .flatten()
                        .any(|t| t.to_lowercase().contains(q))
                })
            })
            .collect();
        let printed = |l: &LabelSpec| l.printed_at.is_some();
        let priced = |l: &LabelSpec| l.price_minor.is_some();
        let count = |f: &dyn Fn(&LabelSpec) -> bool| list.iter().filter(|l| f(l)).count() as i64;
        let counts = IntegrationLabelCounts {
            total: list.len() as i64,
            has_price: count(&priced),
            missing_price: count(&|l| !priced(l)),
            missing_price_tag: count(&|l| priced(l) && !printed(l)),
            printed: count(&printed),
            not_printed: count(&|l| !printed(l)),
        };
        let filtered: Vec<&LabelSpec> = list
            .into_iter()
            .filter(|l| match params.get("filter").copied() {
                Some("missingPriceTag") => priced(l) && !printed(l),
                Some("notPrinted") => !printed(l),
                Some("printed") => printed(l),
                _ => true,
            })
            .collect();
        let end = (offset + limit).min(filtered.len());
        let items: Vec<IntegrationLabelProduct> = filtered
            .get(offset..end)
            .unwrap_or_default()
            .iter()
            .map(|l| IntegrationLabelProduct {
                product_id: l.product_id.clone(),
                sku: l.sku.clone(),
                title: l.title.clone(),
                product_type: l.product_type.clone(),
                has_price: priced(l),
                price_minor: l.price_minor,
                label_status: if printed(l) {
                    WireLabelStatus::Printed
                } else {
                    WireLabelStatus::NotPrinted
                },
                label_printed_at: l.printed_at.map(iso),
                label_reset_reason: l.reset.clone().filter(|_| !printed(l)),
                barcode: None,
                thumb_url: l.thumb_url.clone(),
                created_at: l.created_at.map(iso),
            })
            .collect();
        let mut next_cursor = (end < filtered.len()).then(|| format!("o:{end}"));
        if self.repeat_cursor && params.contains_key("cursor") {
            self.repeat_cursor = false;
            next_cursor = params.get("cursor").map(|c| decode(c));
        }
        HttpReply::new(
            200,
            to_json!(&IntegrationLabelProductPage {
                items,
                next_cursor,
                counts,
                work_from: self
                    .work_from
                    .clone()
                    .unwrap_or_else(|| Some(FAKE_WORK_FROM.into())),
            }),
        )
    }

    /// L3 (projection r0, D8): the template of a product: its assigned template, else the one
    /// whose product types hold its type, else the default, else none.
    fn template_of(&self, spec: &LabelSpec) -> Option<IntegrationLabelTemplate> {
        let by_id = spec
            .template
            .as_ref()
            .and_then(|id| self.templates.iter().find(|t| &t.id == id));
        let by_type = || {
            spec.product_type.as_ref().and_then(|ty| {
                self.templates
                    .iter()
                    .find(|t| t.product_types.iter().any(|p| p == ty))
            })
        };
        let default = || self.templates.iter().find(|t| t.is_default);
        by_id.or_else(by_type).or_else(default).cloned()
    }

    fn unresolved_item(product_id: &str) -> UnresolvedItem {
        UnresolvedItem {
            sku: None,
            product_id: Some(product_id.to_owned()),
            error: ErrorBody {
                code: "product_not_found".into(),
                message: format!("No product with id '{product_id}' in this store."),
                details: None,
            },
        }
    }

    /// L3: one plan item by the package's rules (`fail` when `fail_label` names it or no
    /// template applies, `reprint` when printed, `noPrice` when unpriced, else `markPrinted`).
    fn plan_item(&self, position: usize, spec: &LabelSpec) -> IntegrationPlanItem {
        let template = self.template_of(spec);
        let mut issues = Vec::new();
        if let Some((code, message)) = self.fail_label.get(&spec.product_id) {
            issues.push(IntegrationPlanIssue {
                code: code.clone(),
                message: message.clone(),
            });
        }
        if template.is_none() {
            issues.push(IntegrationPlanIssue {
                code: "no_template".into(),
                message: "No template applies.".into(),
            });
        }
        let outcome = if !issues.is_empty() {
            WirePlanOutcome::Fail
        } else if spec.printed_at.is_some() {
            WirePlanOutcome::Reprint
        } else if spec.price_minor.is_none() {
            WirePlanOutcome::NoPrice
        } else {
            WirePlanOutcome::MarkPrinted
        };
        IntegrationPlanItem {
            position,
            sku: spec.sku.clone(),
            product_id: spec.product_id.clone(),
            title: Some(spec.title.clone()),
            product_type: spec.product_type.clone(),
            template: template.map(|t| IntegrationPlanTemplate {
                id: t.id,
                name: t.name,
                width_mm: t.width_mm,
                height_mm: t.height_mm,
                source: Some("default".into()),
            }),
            has_price: spec.price_minor.is_some(),
            price_minor: spec.price_minor,
            label_status: if spec.printed_at.is_some() {
                WireLabelStatus::Printed
            } else {
                WireLabelStatus::NotPrinted
            },
            label_printed_at: spec.printed_at.map(iso),
            outcome,
            issues,
            notes: vec![],
        }
    }

    /// The product ids of a plan or run body (`items[].productId`).
    fn body_ids(body: Option<&Json>) -> Option<Vec<String>> {
        let items: Vec<IntegrationRunItemIn> =
            serde_json::from_value(body?.get("items")?.clone()).ok()?;
        items.into_iter().map(|i| i.product_id).collect()
    }

    /// L3: `POST …/labels/plan`: per product its plan item, unknown ids `unresolved`; none
    /// resolved → 422 `no_products` with `details.unresolved`; `sizes` over the items that do
    /// not fail (projection r0, D8).
    fn label_plan(&self, body: Option<&Json>) -> HttpReply {
        let Some(ids) = Self::body_ids(body).filter(|ids| (1..=300).contains(&ids.len())) else {
            return envelope(
                422,
                "validation_error",
                "1 to 300 items, each with productId.",
            );
        };
        let mut items = Vec::new();
        let mut unresolved = Vec::new();
        for (position, id) in ids.iter().enumerate() {
            match self.labels.iter().find(|l| &l.product_id == id) {
                Some(spec) => items.push(self.plan_item(position, spec)),
                None => unresolved.push(Self::unresolved_item(id)),
            }
        }
        if items.is_empty() {
            return no_products(&unresolved);
        }
        let count = |o: WirePlanOutcome| items.iter().filter(|i| i.outcome == o).count() as i64;
        let failing = count(WirePlanOutcome::Fail);
        let mut sizes: Vec<IntegrationPlanSize> = Vec::new();
        for item in items.iter().filter(|i| i.outcome != WirePlanOutcome::Fail) {
            let Some(t) = &item.template else { continue };
            match sizes
                .iter_mut()
                .find(|z| z.width_mm == t.width_mm && z.height_mm == t.height_mm)
            {
                Some(z) => z.count += 1,
                None => sizes.push(IntegrationPlanSize {
                    width_mm: t.width_mm,
                    height_mm: t.height_mm,
                    count: 1,
                }),
            }
        }
        let summary = IntegrationPlanSummary {
            items: items.len() as i64,
            ready: items.len() as i64 - failing,
            failing,
            mark_printed: count(WirePlanOutcome::MarkPrinted),
            reprint: count(WirePlanOutcome::Reprint),
            no_price: count(WirePlanOutcome::NoPrice),
        };
        HttpReply::new(
            200,
            to_json!(&IntegrationPlan {
                items,
                unresolved,
                summary,
                sizes,
            }),
        )
    }

    /// L3: `POST …/labels/preview`: the label's PDF; 404 `product_not_found` for an unknown id,
    /// 422 `no_template` when no template applies.
    fn label_preview(&self, body: Option<&Json>) -> Answer {
        let Some(id) = body
            .and_then(|b| serde_json::from_value::<IntegrationRunItemIn>(b.clone()).ok())
            .and_then(|i| i.product_id)
        else {
            return Answer::Json(envelope(422, "validation_error", "productId is required."));
        };
        let Some(spec) = self.labels.iter().find(|l| l.product_id == id) else {
            return Answer::Json(envelope(
                404,
                "product_not_found",
                &format!("No product with id '{id}' in this store."),
            ));
        };
        if self.template_of(spec).is_none() {
            return Answer::Json(envelope(422, "no_template", "No template applies."));
        }
        Answer::File(format!("%PDF-1.4 fake preview {id}").into_bytes())
    }

    /// L3: `POST …/labels/runs` (201): a key seen before answers its stored run without marking
    /// again; otherwise each resolved product is rendered unless `fail_label` / `fail_in_run`
    /// names it or no template applies (`failed`, `lifecycleResult` null); priced unprinted
    /// products become printed (one `label.printed` each), printed ones are `reprinted`, unpriced
    /// ones `no_price`; unknown ids are `unresolved`; none resolved → 422 `no_products`. One
    /// artifact per label size among the rendered items, its `pageCount` = those items.
    fn label_run(&mut self, store_id: &str, body: Option<&Json>) -> HttpReply {
        let Some(create) =
            body.and_then(|b| serde_json::from_value::<IntegrationRunCreate>(b.clone()).ok())
        else {
            return envelope(
                422,
                "validation_error",
                "items and idempotencyKey are required.",
            );
        };
        if !(8..=80).contains(&create.idempotency_key.len()) {
            return envelope(
                422,
                "validation_error",
                "idempotencyKey must be 8 to 80 characters.",
            );
        }
        if let Some(stored) = self.runs.iter().find(|r| r.key == create.idempotency_key) {
            return HttpReply::new(201, to_json!(&stored.run));
        }
        let Some(ids) = Self::body_ids(body).filter(|ids| (1..=300).contains(&ids.len())) else {
            return envelope(
                422,
                "validation_error",
                "1 to 300 items, each with productId.",
            );
        };
        let mut resolved: Vec<LabelSpec> = Vec::new();
        let mut unresolved = Vec::new();
        for id in &ids {
            match self.labels.iter().find(|l| &l.product_id == id) {
                Some(spec) => resolved.push(spec.clone()),
                None => unresolved.push(Self::unresolved_item(id)),
            }
        }
        if resolved.is_empty() {
            return no_products(&unresolved);
        }
        let n = self.runs.len() + 1;
        let run_id = format!("run-{n}");
        let created = self.now();
        let created_ms = chrono::DateTime::parse_from_rfc3339(&created)
            .unwrap()
            .timestamp_millis();
        let mut items = Vec::new();
        let mut sizes: Vec<(f64, f64, Vec<usize>)> = Vec::new();
        for (position, spec) in resolved.iter().enumerate() {
            let template = self.template_of(spec);
            let failure = self
                .fail_label
                .get(&spec.product_id)
                .or_else(|| self.fail_in_run.get(&spec.product_id))
                .cloned()
                .or_else(|| {
                    template
                        .is_none()
                        .then(|| ("no_template".into(), "No template applies.".into()))
                });
            let mut item = RunItemOut {
                position: position as i64,
                product_id: spec.product_id.clone(),
                sku: spec.sku.clone(),
                title: Some(spec.title.clone()),
                variant_id: None,
                template_revision_id: None,
                template_source: Some("default".into()),
                price_minor: spec.price_minor,
                outcome: "failed".into(),
                lifecycle_result: None,
                error: None,
                artifact_id: None,
                page_index: None,
            };
            match (failure, template) {
                (Some((code, message)), _) => item.error = Some(IssueOut { code, message }),
                (None, Some(t)) => {
                    item.outcome = "rendered".into();
                    let result = if spec.printed_at.is_some() {
                        "reprinted"
                    } else if spec.price_minor.is_none() {
                        "no_price"
                    } else {
                        let label = self.label_mut(&spec.product_id);
                        label.printed_at = Some(created_ms);
                        label.reset = None;
                        let price = spec.price_minor;
                        self.log_label_event(
                            &spec.product_id,
                            "label.printed",
                            json!({"runId": run_id, "priceMinor": price}),
                        );
                        "marked_printed"
                    };
                    item.lifecycle_result = Some(result.into());
                    let k = match sizes
                        .iter()
                        .position(|(w, h, _)| *w == t.width_mm && *h == t.height_mm)
                    {
                        Some(k) => k,
                        None => {
                            sizes.push((t.width_mm, t.height_mm, Vec::new()));
                            sizes.len() - 1
                        }
                    };
                    sizes[k].2.push(items.len());
                }
                (None, None) => unreachable!("a missing template is a failure"),
            }
            items.push(item);
        }
        let mut artifacts = Vec::new();
        for (k, (w, h, members)) in sizes.iter().enumerate() {
            let artifact_id = format!("art-{n}-{}", k + 1);
            for (page, i) in members.iter().enumerate() {
                items[*i].artifact_id = Some(artifact_id.clone());
                items[*i].page_index = Some(page as i64);
            }
            artifacts.push(RunArtifactOut {
                download_url: format!(
                    "/integration/v1/stores/{store_id}/labels/runs/{run_id}/artifacts/{artifact_id}"
                ),
                id: artifact_id,
                width_mm: *w,
                height_mm: *h,
                page_count: members.len() as i64,
                filename: format!("labels-{n}-{w}x{h}.pdf"),
                size_bytes: 1024 * members.len() as i64,
                expired: false,
            });
        }
        let with = |word: &str| {
            items
                .iter()
                .filter(|i| i.lifecycle_result.as_deref() == Some(word))
                .count() as i64
        };
        let rendered = items.iter().filter(|i| i.outcome == "rendered").count() as i64;
        let counts = WireRunCounts {
            items: items.len() as i64,
            rendered,
            failed: items.len() as i64 - rendered,
            marked_printed: with("marked_printed"),
            reprinted: with("reprinted"),
            no_price: with("no_price"),
        };
        let run = IntegrationLabelRun {
            id: run_id,
            status: if rendered == items.len() as i64 {
                WireRunStatus::Rendered
            } else if rendered > 0 {
                WireRunStatus::Partial
            } else {
                WireRunStatus::Failed
            },
            created_at: created,
            expires_at: None,
            requested_by_kind: Some("service".into()),
            requested_by_name: Some("Photographer".into()),
            counts,
            artifacts,
            items,
            unresolved,
        };
        self.runs.push(StoredRun {
            key: create.idempotency_key,
            product_ids: ids,
            created_at: created_ms,
            run: run.clone(),
        });
        HttpReply::new(201, to_json!(&run))
    }

    fn touch(&mut self, case_id: &str) {
        let now = self.now();
        self.case_mut(case_id).case.updated_at = now;
    }
}

fn pricer() -> Actor {
    Actor {
        kind: "user".into(),
        id: "6a7b8c9d-0e1f-4a2b-9c3d-4e5f6a7b8c9d".into(),
        name: "Elin Lindqvist".into(),
        is_me: false,
    }
}

fn service_account() -> Actor {
    Actor {
        kind: "service".into(),
        id: "1f2e3d4c-5b6a-4978-8a9b-0c1d2e3f4a5b".into(),
        name: "Photographer".into(),
        is_me: true,
    }
}

impl FakePricing {
    pub fn new() -> Self {
        Self::default()
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    // ---------- the account ----------

    /// A bearer the fake accepts from now on.
    pub fn accept_key(&self, key: &str) {
        self.state().accepted_keys.push(key.to_owned());
    }

    /// Adds a store the key may see; returns its id.
    pub fn add_store(&self, name: &str, shop_domain: &str) -> String {
        let mut state = self.state();
        let id = state.id("store");
        state.stores.push(IntegrationStore {
            id: id.clone(),
            name: name.into(),
            shop_domain: shop_domain.into(),
        });
        id
    }

    /// Empties the key's store list.
    pub fn clear_stores(&self) {
        self.state().stores.clear();
    }

    /// Removes a store from the key's list (its cases and events stay addressable by id).
    pub fn remove_store(&self, store_id: &str) {
        self.state().stores.retain(|s| s.id != store_id);
    }

    // ---------- the pricer's side ----------

    /// The pricer asks a photographer question (`case.created`, then `message.created` by a user).
    pub fn ask(&self, store_id: &str, sku: Option<&str>, topic: &str) -> String {
        self.state()
            .create_case(store_id, sku, topic, "photographer")
    }

    /// The pricer asks Operations a question (never for the photographer).
    pub fn ask_operations(&self, store_id: &str, sku: Option<&str>) -> String {
        self.state()
            .create_case(store_id, sku, "Other", "operations")
    }

    /// A new pricer message on the case (`message.created`, `authorKind: user`).
    pub fn pricer_reply(&self, case_id: &str, text: &str) -> String {
        let mut state = self.state();
        let message = state.new_message(case_id, pricer(), text);
        let id = message.id.clone();
        state.case_mut(case_id).case.messages.push(message);
        state.touch(case_id);
        state.log_case_event(
            case_id,
            "message.created",
            json!({"messageId": id, "authorKind": "user"}),
        );
        id
    }

    /// A message of the app's own account appears on the case (as a reply written elsewhere would:
    /// `message.created`, `authorKind: service`; the case becomes `answered`).
    pub fn own_reply(&self, case_id: &str, text: &str) -> String {
        let mut state = self.state();
        let message = state.new_message(case_id, service_account(), text);
        let id = message.id.clone();
        let fake = state.case_mut(case_id);
        fake.case.messages.push(message);
        fake.case.status = "answered".into();
        state.touch(case_id);
        state.log_case_event(
            case_id,
            "message.created",
            json!({"messageId": id, "authorKind": "service"}),
        );
        id
    }

    /// The pricer edits a message (`message.updated`).
    pub fn edit_message(&self, case_id: &str, message_id: &str, text: &str) {
        let mut state = self.state();
        let now = state.now();
        let fake = state.case_mut(case_id);
        let message = fake
            .case
            .messages
            .iter_mut()
            .find(|m| m.id == message_id)
            .expect("message");
        message.text = Some(text.into());
        message.edited_at = Some(now);
        state.touch(case_id);
        state.log_case_event(case_id, "message.updated", json!({"messageId": message_id}));
    }

    /// The pricer deletes a message (`message.deleted`); deleting the last one removes the case
    /// (`case.updated` with `deleted: true`, API.md).
    pub fn delete_message(&self, case_id: &str, message_id: &str) {
        let mut state = self.state();
        state
            .case_mut(case_id)
            .case
            .messages
            .retain(|m| m.id != message_id);
        if state.case_mut(case_id).case.messages.is_empty() {
            state.log_case_event(case_id, "case.updated", json!({"deleted": true}));
            state.cases.retain(|c| c.case.id != case_id);
        } else {
            state.touch(case_id);
            state.log_case_event(case_id, "message.deleted", json!({"messageId": message_id}));
        }
    }

    /// The pricer resolves the case (`case.updated`).
    pub fn resolve(&self, case_id: &str) {
        let mut state = self.state();
        state.case_mut(case_id).case.status = "resolved".into();
        state.touch(case_id);
        state.log_case_event(case_id, "case.updated", json!({}));
    }

    /// The product is activated, which resolves the case (`case.updated`, `resolvedBy: activation`).
    pub fn resolve_by_activation(&self, case_id: &str) {
        let mut state = self.state();
        state.case_mut(case_id).case.status = "resolved".into();
        state.touch(case_id);
        state.log_case_event(case_id, "case.updated", json!({"resolvedBy": "activation"}));
    }

    /// The case disappears without an event (its `GET` answers 404).
    pub fn delete_case(&self, case_id: &str) {
        self.state().cases.retain(|c| c.case.id != case_id);
    }

    /// Appends an event of any type to the store's log; returns its id.
    pub fn raw_event(&self, store_id: &str, kind: &str, case_id: Option<&str>, data: Json) -> i64 {
        self.state()
            .push_event(store_id, kind, case_id.map(str::to_owned), None, data)
    }

    /// The case changes with no event (what a redelivered event must not pick up).
    pub fn change_silently(&self, case_id: &str, change: impl FnOnce(&mut IntegrationCase)) {
        change(&mut self.state().case_mut(case_id).case);
    }

    /// The ids of the case's messages, in order.
    pub fn message_ids(&self, case_id: &str) -> Vec<String> {
        self.state()
            .case_mut(case_id)
            .case
            .messages
            .iter()
            .map(|m| m.id.clone())
            .collect()
    }

    /// The time of a logged event (UTC ms).
    pub fn event_at_ms(&self, event_id: i64) -> i64 {
        let state = self.state();
        let event = state
            .events
            .iter()
            .find(|e| e.id == event_id)
            .expect("FakePricing: no such event");
        chrono::DateTime::parse_from_rfc3339(&event.at)
            .unwrap()
            .timestamp_millis()
    }

    // ---------- price labels (L1) ----------

    /// Adds (or replaces) a product of the label state.
    pub fn add_label_product(&self, spec: LabelSpec) {
        let mut state = self.state();
        state.labels.retain(|l| l.product_id != spec.product_id);
        state.labels.push(spec);
    }

    /// Changes a product of the label state without an event.
    pub fn set_label(&self, product_id: &str, change: impl FnOnce(&mut LabelSpec)) {
        change(self.state().label_mut(product_id));
    }

    /// Adds an active label template.
    pub fn add_template(&self, template: IntegrationLabelTemplate) {
        self.state().templates.push(template);
    }

    /// L2: adds `n` priced, not printed products at once, each newer than the last: product id
    /// `77000<k>` (k from 1, counted over every call), SKU `TEST-<k>`, title `Test product <k>`.
    /// Answers their product ids in the order added.
    pub fn add_label_products(&self, n: usize) -> Vec<String> {
        let mut state = self.state();
        let mut ids = Vec::with_capacity(n);
        for _ in 0..n {
            state.bulk += 1;
            let k = state.bulk;
            let id = format!("77000{k}");
            state.labels.push(LabelSpec {
                product_id: id.clone(),
                sku: Some(format!("TEST-{k}")),
                title: format!("Test product {k}"),
                price_minor: Some(100_000),
                created_at: Some(FAKE_EPOCH_MS + k as i64 * 1_000),
                ..LabelSpec::default()
            });
            ids.push(id);
        }
        ids
    }

    /// L2: the `workFrom` of the list answers (none = no `workFrom`).
    pub fn set_work_from(&self, work_from: Option<&str>) {
        self.state().work_from = Some(work_from.map(str::to_owned));
    }

    /// L2: the next list answer for a request that carries a cursor answers that same cursor as
    /// its `nextCursor` (a list that never ends).
    pub fn repeat_cursor_next(&self) {
        self.state().repeat_cursor = true;
    }

    /// From now on every `/labels/` path answers `403 insufficient_scope` (a key without
    /// `labels:read`).
    pub fn deny_labels(&self) {
        self.state().deny_labels = true;
    }

    /// L3: the product's label state as the fake holds it now (none when it holds no such
    /// product).
    pub fn label(&self, product_id: &str) -> Option<LabelSpec> {
        self.state()
            .labels
            .iter()
            .find(|l| l.product_id == product_id)
            .cloned()
    }

    /// L3: the product passes the check and fails in the run with this error.
    pub fn fail_in_run(&self, product_id: &str, code: &str, message: &str) {
        self.state()
            .fail_in_run
            .insert(product_id.into(), (code.into(), message.into()));
    }

    /// L3: the product leaves the fake's label state (a later check or run does not find it).
    pub fn remove_label_product(&self, product_id: &str) {
        self.state().labels.retain(|l| l.product_id != product_id);
    }

    /// L3: the stored runs, in the order made, with their keys, products and times.
    pub fn runs(&self) -> Vec<FakeRun> {
        self.state()
            .runs
            .iter()
            .map(|r| FakeRun {
                id: r.run.id.clone(),
                key: r.key.clone(),
                product_ids: r.product_ids.clone(),
                created_at: r.created_at,
            })
            .collect()
    }

    /// L3: how many `label.printed` events the log holds for the product.
    pub fn printed_events(&self, product_id: &str) -> usize {
        self.state()
            .events
            .iter()
            .filter(|e| e.kind == "label.printed" && e.product_id.as_deref() == Some(product_id))
            .count()
    }

    /// L3: the product fails in the check and in the run with this issue.
    pub fn fail_label(&self, product_id: &str, code: &str, message: &str) {
        self.state()
            .fail_label
            .insert(product_id.into(), (code.into(), message.into()));
    }

    /// A label is generated for the product: it becomes Printed at the fake clock's time and
    /// `label.printed` is logged. Answers the event id (its time: [`Self::event_at_ms`]).
    pub fn label_printed(&self, product_id: &str) -> i64 {
        let mut state = self.state();
        let price = state.label_mut(product_id).price_minor;
        let id = state.log_label_event(
            product_id,
            "label.printed",
            json!({"runId": "fake-run", "priceMinor": price}),
        );
        let at = state
            .events
            .last()
            .map(|e| e.at.clone())
            .unwrap_or_default();
        let at = chrono::DateTime::parse_from_rfc3339(&at)
            .map(|t| t.timestamp_millis())
            .ok();
        let label = state.label_mut(product_id);
        label.printed_at = at;
        label.reset = None;
        id
    }

    /// The product's price, SKU or barcode changed: it is Not printed again with `reason` and
    /// `label.reset` is logged. Answers the event id.
    pub fn label_reset(&self, product_id: &str, reason: &str) -> i64 {
        let mut state = self.state();
        let label = state.label_mut(product_id);
        label.printed_at = None;
        label.reset = Some(reason.into());
        state.log_label_event(product_id, "label.reset", json!({"reason": reason}))
    }

    /// The id of the last event in the log (0 when empty).
    pub fn last_event_id(&self) -> i64 {
        self.state().next_event
    }

    // ---------- scripted failures ----------

    /// The next `n` requests whose path contains `path_part` answer `status` with the envelope.
    pub fn fail_matching(&self, path_part: &str, n: usize, status: u16) {
        self.state().scripted.push(Scripted {
            path_part: path_part.into(),
            remaining: n,
            skip: 0,
            status,
            envelope: None,
        });
    }

    /// L2: as [`Self::fail_matching`], after the first `skip` matching requests have passed.
    pub fn fail_matching_after(&self, path_part: &str, skip: usize, n: usize, status: u16) {
        self.state().scripted.push(Scripted {
            path_part: path_part.into(),
            remaining: n,
            skip,
            status,
            envelope: None,
        });
    }

    /// The next `n` requests whose path contains `path_part` answer `status` with the package
    /// envelope carrying `code` and `message`.
    pub fn fail_matching_with(
        &self,
        path_part: &str,
        n: usize,
        status: u16,
        code: &str,
        message: &str,
    ) {
        self.state().scripted.push(Scripted {
            path_part: path_part.into(),
            remaining: n,
            skip: 0,
            status,
            envelope: Some((code.into(), message.into())),
        });
    }

    /// The next `n` requests time out (`Error::Timeout`), with nothing done.
    pub fn timeout_next(&self, n: usize) {
        self.state().timeouts += n;
    }

    /// The next `n` requests fail to connect (`Error::Provider("http: connection refused")`).
    pub fn refuse_connection_next(&self, n: usize) {
        self.state().refusals += n;
    }

    /// The next `n` requests are acted on, then their answer is lost (`Error::Timeout`).
    pub fn drop_answer_next(&self, n: usize) {
        self.state().drops += n;
    }

    /// The next non-empty events page answers `nextCursor` equal to the request's `after`.
    pub fn stale_cursor_next(&self) {
        self.state().stale_cursor = true;
    }

    /// The event is returned once more, at the front of the next events page.
    pub fn redeliver(&self, event_id: i64) {
        self.state().redeliver.push(event_id);
    }

    /// Requests block until [`Self::release`].
    pub fn hold(&self) {
        self.hold.lock().unwrap().held = true;
    }

    pub fn release(&self) {
        self.hold.lock().unwrap().held = false;
        self.hold_changed.notify_all();
    }

    /// Waits (at most 10 s) until a request is blocked by [`Self::hold`]; `false` if none came.
    pub fn wait_until_held(&self) -> bool {
        let hold = self.hold.lock().unwrap();
        let (hold, _) = self
            .hold_changed
            .wait_timeout_while(hold, Duration::from_secs(10), |h| h.waiting == 0)
            .unwrap();
        hold.waiting > 0
    }

    // ---------- the record ----------

    pub fn requests(&self) -> Vec<FakeRequest> {
        self.state().requests.clone()
    }

    /// The recorded requests as `METHOD /path?query` lines.
    pub fn request_lines(&self) -> Vec<String> {
        self.requests().iter().map(FakeRequest::line).collect()
    }

    pub fn clear_requests(&self) {
        self.state().requests.clear();
    }

    // ---------- the endpoints ----------

    fn route(
        state: &mut State,
        method: Method,
        path: &str,
        query: &str,
        body: Option<&Json>,
    ) -> HttpReply {
        let segments: Vec<String> = path
            .trim_start_matches('/')
            .split('/')
            .map(decode)
            .collect();
        let segments: Vec<&str> = segments.iter().map(String::as_str).collect();
        let params: HashMap<&str, &str> =
            query.split('&').filter_map(|p| p.split_once('=')).collect();
        if segments == ["stores"] && method == Method::Get {
            return HttpReply::new(200, to_json!(&state.stores));
        }
        let ["stores", store_id, rest @ ..] = segments.as_slice() else {
            return envelope(404, "not_found", "Not found.");
        };
        let store_id = (*store_id).to_owned();
        if !state.stores.iter().any(|s| s.id == store_id) {
            return envelope(404, "not_found", "Not found.");
        }
        if rest.first() == Some(&"labels") && state.deny_labels {
            return envelope(403, "insufficient_scope", "This key lacks labels:read.");
        }
        match (method, rest) {
            (Method::Get, ["labels", "status"]) => {
                let ids: Vec<String> = params
                    .get("productIds")
                    .map(|ids| ids.split(',').map(decode).collect())
                    .unwrap_or_default();
                if ids.is_empty() || ids.len() > 200 {
                    return envelope(422, "validation_error", "1 to 200 productIds.");
                }
                let entries: Vec<IntegrationLabelStatus> =
                    ids.iter().map(|id| state.label_status(id)).collect();
                HttpReply::new(200, to_json!(&entries))
            }
            (Method::Get, ["labels", "products"]) => state.label_products(&params),
            (Method::Get, ["labels", "templates"]) => {
                HttpReply::new(200, to_json!(&state.templates))
            }
            (Method::Post, ["labels", "plan"]) => state.label_plan(body),
            (Method::Post, ["labels", "runs"]) => state.label_run(&store_id, body),
            (Method::Get, ["cases"]) => {
                let recipient = params.get("recipient").copied();
                let status = params.get("status").copied();
                let list: Vec<&IntegrationCase> = state
                    .cases
                    .iter()
                    .filter(|c| c.store_id == store_id)
                    .map(|c| &c.case)
                    .filter(|c| recipient.is_none_or(|r| c.recipient == r))
                    .filter(|c| match status {
                        Some("unresolved") => c.status != "resolved",
                        Some(s) => c.status == s,
                        None => true,
                    })
                    .take(200)
                    .collect();
                HttpReply::new(200, to_json!(&list))
            }
            (Method::Get, ["cases", case_id]) => match state
                .cases
                .iter()
                .find(|c| c.store_id == store_id && c.case.id == *case_id)
            {
                Some(c) => HttpReply::new(200, to_json!(&c.case)),
                None => envelope(404, "not_found", "No such case."),
            },
            (Method::Patch, ["cases", case_id]) => {
                let case_id = (*case_id).to_owned();
                if !state
                    .cases
                    .iter()
                    .any(|c| c.store_id == store_id && c.case.id == case_id)
                {
                    return envelope(404, "not_found", "No such case.");
                }
                let Some(update) = body
                    .and_then(|b| serde_json::from_value::<IntegrationCaseUpdate>(b.clone()).ok())
                    .filter(|u| u.status == "answered" || u.status == "open")
                else {
                    return envelope(422, "validation_error", "status must be open or answered.");
                };
                if state.case_mut(&case_id).case.status == "resolved" {
                    return envelope(409, "case_resolved", "The case is resolved.");
                }
                state.case_mut(&case_id).case.status = update.status;
                state.touch(&case_id);
                state.log_case_event(&case_id, "case.updated", json!({}));
                HttpReply::new(200, to_json!(&state.case_mut(&case_id).case))
            }
            (Method::Post, ["cases", case_id, "messages"]) => {
                let case_id = (*case_id).to_owned();
                if !state
                    .cases
                    .iter()
                    .any(|c| c.store_id == store_id && c.case.id == case_id)
                {
                    return envelope(404, "not_found", "No such case.");
                }
                let Some(create) = body.and_then(|b| {
                    serde_json::from_value::<IntegrationMessageCreate>(b.clone()).ok()
                }) else {
                    return envelope(422, "validation_error", "Invalid body.");
                };
                let length = create.text.chars().count();
                if !(1..=4000).contains(&length) || create.attachment_media_ids.len() > 10 {
                    return envelope(422, "validation_error", "Invalid body.");
                }
                if let Some(cmid) = &create.client_mutation_id
                    && let Some(message) = state.mutations.get(&(case_id.clone(), cmid.clone()))
                {
                    return HttpReply::new(201, to_json!(message));
                }
                let reply_to = match &create.reply_to_message_id {
                    Some(target) => {
                        let found = state
                            .case_mut(&case_id)
                            .case
                            .messages
                            .iter()
                            .find(|m| &m.id == target)
                            .cloned();
                        match found {
                            Some(m) => Some(
                                psm_integrations::conversations::beyo_pricing::types::ReplyRef {
                                    message_id: m.id,
                                    author_name: m.author.name,
                                    excerpt: m.text.unwrap_or_default().chars().take(80).collect(),
                                },
                            ),
                            None => {
                                return envelope(
                                    400,
                                    "invalid_reply",
                                    "Reply target not in this case.",
                                );
                            }
                        }
                    }
                    None => None,
                };
                let mut message = state.new_message(&case_id, service_account(), &create.text);
                message.reply_to = reply_to;
                message.attachments = create
                    .attachment_media_ids
                    .iter()
                    .map(
                        |id| psm_integrations::conversations::beyo_pricing::types::Attachment {
                            media_id: id.clone(),
                            thumb_url: None,
                            url: None,
                        },
                    )
                    .collect();
                if let Some(cmid) = &create.client_mutation_id {
                    state
                        .mutations
                        .insert((case_id.clone(), cmid.clone()), message.clone());
                }
                let id = message.id.clone();
                let fake = state.case_mut(&case_id);
                fake.case.messages.push(message.clone());
                fake.case.status = "answered".into();
                state.touch(&case_id);
                state.log_case_event(
                    &case_id,
                    "message.created",
                    json!({"messageId": id, "authorKind": "service"}),
                );
                HttpReply::new(201, to_json!(&message))
            }
            (Method::Get, ["events"]) => {
                let after = params
                    .get("after")
                    .and_then(|a| a.parse::<i64>().ok())
                    .unwrap_or(0);
                let limit = params
                    .get("limit")
                    .and_then(|l| l.parse::<usize>().ok())
                    .unwrap_or(100)
                    .clamp(1, 500);
                let redelivered: Vec<StoreEvent> = std::mem::take(&mut state.redeliver)
                    .into_iter()
                    .filter_map(|id| state.events.iter().find(|e| e.id == id).cloned())
                    .filter(|e| e.store_id == store_id)
                    .collect();
                let mut events = redelivered;
                events.extend(
                    state
                        .events
                        .iter()
                        .filter(|e| e.store_id == store_id && e.id > after)
                        .take(limit)
                        .cloned(),
                );
                let mut next_cursor = events
                    .iter()
                    .map(|e| e.id)
                    .max()
                    .unwrap_or(after)
                    .max(after);
                if !events.is_empty() && state.stale_cursor {
                    state.stale_cursor = false;
                    next_cursor = after;
                }
                HttpReply::new(
                    200,
                    to_json!(&EventPage {
                        events,
                        next_cursor,
                    }),
                )
            }
            _ => envelope(404, "not_found", "Not found."),
        }
    }
}

impl FakePricing {
    /// Records the request (never its bearer value); answers its path, query and whether the
    /// bearer is accepted.
    fn record(&self, request: &PricingRequest) -> (String, String, bool) {
        let (address, query) = request
            .url
            .split_once('?')
            .map(|(a, q)| (a.to_owned(), q.to_owned()))
            .unwrap_or((request.url.clone(), String::new()));
        let rest = address.strip_prefix("https://").unwrap_or(&address);
        let (host, full_path) = rest.split_once('/').map_or((rest, ""), |(h, p)| (h, p));
        let full_path = format!("/{full_path}");
        let path = full_path
            .strip_prefix("/integration/v1")
            .unwrap_or(&full_path)
            .to_owned();
        let mut state = self.state();
        let authorized = request
            .bearer
            .as_ref()
            .is_some_and(|b| state.accepted_keys.contains(b));
        state.requests.push(FakeRequest {
            method: request.method.as_str().into(),
            server: format!("https://{host}"),
            path: path.clone(),
            query: query.clone(),
            body: request.body.clone(),
            authorized,
        });
        (path, query, authorized)
    }
}

impl FakePricing {
    /// L3: a file route (`POST …/labels/preview`), with the store and scope checks of every
    /// label path; none for any other path.
    fn route_file(
        state: &mut State,
        method: Method,
        path: &str,
        body: Option<&Json>,
    ) -> Option<Answer> {
        let segments: Vec<String> = path
            .trim_start_matches('/')
            .split('/')
            .map(decode)
            .collect();
        let segments: Vec<&str> = segments.iter().map(String::as_str).collect();
        let ["stores", store_id, rest @ ..] = segments.as_slice() else {
            return None;
        };
        if !matches!((method, rest), (Method::Post, ["labels", "preview"])) {
            return None;
        }
        if !state.stores.iter().any(|s| s.id == *store_id) {
            return Some(Answer::Json(envelope(404, "not_found", "Not found.")));
        }
        if state.deny_labels {
            return Some(Answer::Json(envelope(
                403,
                "insufficient_scope",
                "This key lacks labels:read.",
            )));
        }
        Some(state.label_preview(body))
    }

    /// One request as `send` and `fetch` both take it: recorded, held while [`Self::hold`] is
    /// on, then the scripted time-outs, refusals and failures, the route, and the dropped
    /// answers (L3: `fetch` honours them all, as `send` does).
    fn answer(&self, request: &PricingRequest) -> Result<Answer> {
        let (path, query, authorized) = self.record(request);
        {
            let mut hold = self.hold.lock().unwrap();
            if hold.held {
                hold.waiting += 1;
                self.hold_changed.notify_all();
                while hold.held {
                    hold = self.hold_changed.wait(hold).unwrap();
                }
                hold.waiting -= 1;
            }
        }
        let mut state = self.state();
        if state.timeouts > 0 {
            state.timeouts -= 1;
            return Err(Error::Timeout);
        }
        if state.refusals > 0 {
            state.refusals -= 1;
            return Err(Error::Provider("http: connection refused".into()));
        }
        let mut rule = state
            .scripted
            .iter_mut()
            .find(|r| r.remaining > 0 && path.contains(&r.path_part));
        if let Some(r) = rule.as_mut()
            && r.skip > 0
        {
            r.skip -= 1;
            rule = None;
        }
        if let Some(rule) = rule {
            rule.remaining -= 1;
            let status = rule.status;
            return Ok(Answer::Json(match &rule.envelope {
                Some((code, message)) => envelope(status, code, message),
                None => envelope(status, code_for(status), "Scripted failure."),
            }));
        }
        let reply = if authorized {
            match Self::route_file(&mut state, request.method, &path, request.body.as_ref()) {
                Some(answer) => answer,
                None => Answer::Json(Self::route(
                    &mut state,
                    request.method,
                    &path,
                    &query,
                    request.body.as_ref(),
                )),
            }
        } else {
            Answer::Json(envelope(401, "invalid_api_key", "No/invalid/revoked key."))
        };
        if state.drops > 0 {
            state.drops -= 1;
            return Err(Error::Timeout);
        }
        Ok(reply)
    }
}

impl PricingHttp for FakePricing {
    /// L3: as `send` (held, scripted, dropped alike); a file answers `application/pdf` bytes,
    /// JSON answers its text.
    fn fetch(&self, request: PricingRequest, _timeout: Duration) -> Result<RawReply> {
        Ok(match self.answer(&request)? {
            Answer::Json(reply) => RawReply {
                status: reply.status,
                content_type: Some("application/json".into()),
                bytes: reply.body.to_string().into_bytes(),
            },
            Answer::File(bytes) => RawReply {
                status: 200,
                content_type: Some("application/pdf".into()),
                bytes,
            },
        })
    }

    fn send(&self, request: PricingRequest) -> Result<HttpReply> {
        Ok(match self.answer(&request)? {
            Answer::Json(reply) => reply,
            Answer::File(bytes) => HttpReply::new(
                200,
                Json::String(String::from_utf8_lossy(&bytes).into_owned()),
            ),
        })
    }
}
