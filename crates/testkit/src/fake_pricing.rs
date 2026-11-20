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
    transport::{HttpReply, Method, PricingHttp, PricingRequest},
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

struct FakeCase {
    store_id: String,
    case: IntegrationCase,
}

struct Scripted {
    path_part: String,
    remaining: usize,
    status: u16,
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
}

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
        let product_id = self.id("product").replace("product-", "15531808");
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
            status,
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
        match (method, rest) {
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

impl PricingHttp for FakePricing {
    fn send(&self, request: PricingRequest) -> Result<HttpReply> {
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
        let authorized = {
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
            authorized
        };
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
        if let Some(rule) = state
            .scripted
            .iter_mut()
            .find(|r| r.remaining > 0 && path.contains(&r.path_part))
        {
            rule.remaining -= 1;
            let status = rule.status;
            return Ok(envelope(status, code_for(status), "Scripted failure."));
        }
        let reply = if authorized {
            Self::route(
                &mut state,
                request.method,
                &path,
                &query,
                request.body.as_ref(),
            )
        } else {
            envelope(401, "invalid_api_key", "No/invalid/revoked key.")
        };
        if state.drops > 0 {
            state.drops -= 1;
            return Err(Error::Timeout);
        }
        Ok(reply)
    }
}
