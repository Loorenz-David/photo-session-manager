//! `BeyoPricingAdapter`: the conversations port over Pricing's integration API
//! (`/integration/v1`, the package's `API.md` and `openapi-integration.json`). Requests go through
//! `PricingHttp`; replies are parsed through `types.rs` and translated into the core vocabulary.
//! Statuses (C1 delegated choices): 401/403 → `Refused`; 5xx and transport errors →
//! `Unreachable`; 404 on the store's lists → `StoreGone`, on one conversation → `None`, on a
//! write → `NotFound`; any other non-2xx on a read → `BadReply`.

use super::transport::{HttpReply, Method, PricingHttp, PricingRequest};
use super::types;
use psm_core::{
    domain::conversation::{
        Attachment, Author, AuthorKind, Conversation, ConversationStatus, EventKind, EventPage,
        Message, NewMessage, ProviderError, ProviderEvent, ProviderStore, Quote, ReplyRef,
        ResolvedBy,
    },
    ports::conversations::{ConversationsProvider, ProviderResult},
};
use serde::de::DeserializeOwned;
use serde_json::Value as Json;
use std::sync::Arc;

/// The recipient whose conversations this app keeps (09B §3).
const RECIPIENT: &str = "photographer";

type ServerFn = Box<dyn Fn() -> String + Send + Sync>;
type KeyFn = Box<dyn Fn() -> Option<String> + Send + Sync>;

pub struct BeyoPricingAdapter {
    http: Arc<dyn PricingHttp>,
    server: ServerFn,
    key: KeyFn,
}

impl BeyoPricingAdapter {
    /// `server` = the live effective server (checked, no trailing `/`); `key` = the key held in
    /// memory (read at each request, never stored here).
    pub fn new(
        http: Arc<dyn PricingHttp>,
        server: impl Fn() -> String + Send + Sync + 'static,
        key: impl Fn() -> Option<String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            http,
            server: Box::new(server),
            key: Box::new(key),
        }
    }

    fn send(&self, method: Method, path: &str, body: Option<Json>) -> ProviderResult<HttpReply> {
        let url = format!("{}/integration/v1{path}", (self.server)());
        self.http
            .send(PricingRequest {
                method,
                url,
                bearer: (self.key)(),
                body,
            })
            .map_err(|error| ProviderError::Unreachable(error.to_string()))
    }

    fn store_path(store_id: &str) -> String {
        format!("/stores/{}", encode(store_id))
    }

    fn case_path(store_id: &str, case_id: &str) -> String {
        format!("{}/cases/{}", Self::store_path(store_id), encode(case_id))
    }
}

/// Percent-encodes one path segment (RFC 3986 unreserved characters kept).
fn encode(segment: &str) -> String {
    let mut out = String::new();
    for b in segment.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn success(reply: &HttpReply) -> bool {
    (200..300).contains(&reply.status)
}

fn envelope(reply: &HttpReply) -> Option<types::ErrorBody> {
    serde_json::from_value::<types::ErrorEnvelope>(reply.body.clone())
        .ok()
        .map(|e| e.error)
}

/// A non-2xx answer to a read the caller did not handle itself.
fn read_error(reply: &HttpReply) -> ProviderError {
    match reply.status {
        401 | 403 => ProviderError::Refused,
        500..=599 => ProviderError::Unreachable(format!("HTTP {}", reply.status)),
        status => ProviderError::BadReply(format!("HTTP {status}")),
    }
}

/// A non-2xx answer to a write.
fn write_error(reply: &HttpReply) -> ProviderError {
    match reply.status {
        401 | 403 => ProviderError::Refused,
        404 => ProviderError::NotFound,
        500..=599 => ProviderError::Unreachable(format!("HTTP {}", reply.status)),
        status => match envelope(reply) {
            Some(body) if status == 409 && body.code == "case_resolved" => ProviderError::Resolved,
            Some(body) => ProviderError::Rejected {
                code: body.code,
                message: body.message,
            },
            None => ProviderError::BadReply(format!("HTTP {status}")),
        },
    }
}

fn parse<T: DeserializeOwned>(reply: &HttpReply) -> ProviderResult<T> {
    serde_json::from_value(reply.body.clone())
        .map_err(|error| ProviderError::BadReply(format!("reply shape: {error}")))
}

/// ISO 8601 / RFC 3339 → UTC milliseconds.
fn time(text: &str) -> ProviderResult<i64> {
    chrono::DateTime::parse_from_rfc3339(text)
        .map(|t| t.timestamp_millis())
        .map_err(|_| ProviderError::BadReply("time".into()))
}

fn store(wire: types::IntegrationStore) -> ProviderStore {
    ProviderStore {
        id: wire.id,
        name: wire.name,
        shop_domain: wire.shop_domain,
    }
}

/// A wire message as stored, or `None` when it is deleted or has no text (MC-C1.4).
fn message(wire: types::Message) -> ProviderResult<Option<Message>> {
    let created_at = time(&wire.created_at)?;
    let edited_at = wire.edited_at.as_deref().map(time).transpose()?;
    let Some(text) = wire.text.filter(|_| !wire.deleted) else {
        return Ok(None);
    };
    let kind = if wire.author.is_me {
        AuthorKind::Me
    } else if wire.author.kind == "user" {
        AuthorKind::Asker
    } else {
        AuthorKind::Other
    };
    Ok(Some(Message {
        id: wire.id,
        conversation_id: wire.case_id,
        author: Author {
            kind,
            name: wire.author.name,
        },
        text,
        quote: wire.quote.map(|q| Quote {
            field: q.field,
            field_label: q.field_label,
            text: q.text,
        }),
        reply_to: wire.reply_to.map(|r| ReplyRef {
            message_id: r.message_id,
            author_name: r.author_name,
            excerpt: r.excerpt,
        }),
        attachments: wire
            .attachments
            .into_iter()
            .map(|a| Attachment {
                media_id: a.media_id,
                thumb_url: a.thumb_url,
                url: a.url,
            })
            .collect(),
        created_at,
        edited_at,
    }))
}

/// A wire case as a conversation of `store_id`, or `None` when it is not addressed to this app.
fn conversation(
    store_id: &str,
    wire: types::IntegrationCase,
) -> ProviderResult<Option<Conversation>> {
    let status = ConversationStatus::parse(&wire.status)
        .ok_or_else(|| ProviderError::BadReply("status".into()))?;
    let created_at = time(&wire.created_at)?;
    let updated_at = time(&wire.updated_at)?;
    let mut messages = Vec::new();
    for m in wire.messages {
        if let Some(m) = message(m)? {
            messages.push(m);
        }
    }
    if wire.recipient != RECIPIENT {
        return Ok(None);
    }
    Ok(Some(Conversation {
        id: wire.id,
        store_id: store_id.to_owned(),
        product_id: wire.product_id,
        sku: wire.sku,
        product_title: wire.product_title,
        topic: wire.topic,
        blocking: wire.blocking,
        status,
        resolved_by: None,
        created_at,
        updated_at,
        read_mark: None,
        messages,
    }))
}

fn event(wire: types::StoreEvent) -> ProviderResult<ProviderEvent> {
    time(&wire.at)?;
    let data = &wire.data;
    let text = |key: &str| data.get(key).and_then(Json::as_str);
    let conversation_id = wire
        .case_id
        .clone()
        .or_else(|| text("caseId").map(str::to_owned));
    let kind = match wire.kind.as_str() {
        "case.created" => EventKind::ConversationCreated,
        "case.updated" => EventKind::ConversationUpdated {
            deleted: data.get("deleted").and_then(Json::as_bool) == Some(true),
            resolved_by: (text("resolvedBy") == Some("activation"))
                .then_some(ResolvedBy::Activation),
        },
        "message.created" => EventKind::MessageCreated {
            by_asker: text("authorKind") == Some("user"),
        },
        "message.updated" => EventKind::MessageUpdated,
        "message.deleted" => EventKind::MessageDeleted,
        _ => EventKind::Other,
    };
    let for_us = kind != EventKind::Other
        && conversation_id.is_some()
        && text("recipient") == Some(RECIPIENT);
    Ok(ProviderEvent {
        id: wire.id,
        kind,
        conversation_id,
        for_us,
    })
}

impl ConversationsProvider for BeyoPricingAdapter {
    fn stores(&self) -> ProviderResult<Vec<ProviderStore>> {
        let reply = self.send(Method::Get, "/stores", None)?;
        if !success(&reply) {
            return Err(read_error(&reply));
        }
        Ok(parse::<Vec<types::IntegrationStore>>(&reply)?
            .into_iter()
            .map(store)
            .collect())
    }

    fn unresolved(&self, store_id: &str) -> ProviderResult<Vec<Conversation>> {
        let path = format!(
            "{}/cases?recipient={RECIPIENT}&status=unresolved",
            Self::store_path(store_id)
        );
        let reply = self.send(Method::Get, &path, None)?;
        if reply.status == 404 {
            return Err(ProviderError::StoreGone);
        }
        if !success(&reply) {
            return Err(read_error(&reply));
        }
        let mut out = Vec::new();
        for case in parse::<Vec<types::IntegrationCase>>(&reply)? {
            if let Some(c) = conversation(store_id, case)? {
                out.push(c);
            }
        }
        Ok(out)
    }

    fn conversation(&self, store_id: &str, id: &str) -> ProviderResult<Option<Conversation>> {
        let reply = self.send(Method::Get, &Self::case_path(store_id, id), None)?;
        if reply.status == 404 {
            return Ok(None);
        }
        if !success(&reply) {
            return Err(read_error(&reply));
        }
        conversation(store_id, parse(&reply)?)
    }

    fn events_after(&self, store_id: &str, cursor: i64, limit: u32) -> ProviderResult<EventPage> {
        let path = format!(
            "{}/events?after={cursor}&limit={limit}",
            Self::store_path(store_id)
        );
        let reply = self.send(Method::Get, &path, None)?;
        if reply.status == 404 {
            return Err(ProviderError::StoreGone);
        }
        if !success(&reply) {
            return Err(read_error(&reply));
        }
        let page: types::EventPage = parse(&reply)?;
        Ok(EventPage {
            events: page
                .events
                .into_iter()
                .map(event)
                .collect::<ProviderResult<_>>()?,
            next_cursor: page.next_cursor,
        })
    }

    fn post_message(
        &self,
        store_id: &str,
        conversation_id: &str,
        new: &NewMessage,
    ) -> ProviderResult<Message> {
        let body = serde_json::to_value(types::IntegrationMessageCreate {
            text: new.text.clone(),
            reply_to_message_id: new.reply_to_message_id.clone(),
            attachment_media_ids: new.attachment_media_ids.clone(),
            client_mutation_id: Some(new.client_mutation_id.clone()),
        })
        .map_err(|e| ProviderError::BadReply(e.to_string()))?;
        let path = format!("{}/messages", Self::case_path(store_id, conversation_id));
        let reply = self.send(Method::Post, &path, Some(body))?;
        if !success(&reply) {
            return Err(write_error(&reply));
        }
        message(parse(&reply)?)?.ok_or_else(|| ProviderError::BadReply("message".into()))
    }

    fn mark_answered(&self, store_id: &str, conversation_id: &str) -> ProviderResult<Conversation> {
        let body = serde_json::to_value(types::IntegrationCaseUpdate {
            status: "answered".into(),
        })
        .map_err(|e| ProviderError::BadReply(e.to_string()))?;
        let reply = self.send(
            Method::Patch,
            &Self::case_path(store_id, conversation_id),
            Some(body),
        )?;
        if !success(&reply) {
            return Err(write_error(&reply));
        }
        conversation(store_id, parse(&reply)?)?
            .ok_or_else(|| ProviderError::BadReply("recipient".into()))
    }

    fn describe(&self, error: &ProviderError, shop: Option<&str>) -> String {
        match error {
            ProviderError::Unreachable(_) => "Could not reach Pricing".into(),
            ProviderError::BadReply(_) => {
                "Pricing answered in a way this version does not understand".into()
            }
            ProviderError::Refused => "Pricing refused the key: check Settings → Pricing".into(),
            ProviderError::StoreGone => format!(
                "This key is no longer tied to {}",
                shop.unwrap_or("the connected shop")
            ),
            ProviderError::NotFound => "This conversation no longer exists".into(),
            ProviderError::Resolved => "Pricing has already resolved this".into(),
            ProviderError::Rejected { message, .. } => {
                format!("Pricing did not accept this reply: {message}")
            }
        }
    }

    /// 09B §7 (quote chip jump): Pricing's quote field names → the form's keys. `title`,
    /// `description` and `condition` name fixed fields; `mf:<namespace>.<key>` names a metafield
    /// definition (a non-empty namespace without a dot, a dot, a non-empty key); anything else
    /// names no field.
    fn attribute_key(&self, quote_field: &str) -> Option<String> {
        match quote_field {
            "title" => Some("product.title".into()),
            "description" => Some("product.description".into()),
            "condition" => Some("custom.condition_report_description".into()),
            _ => {
                let (namespace, key) = quote_field.strip_prefix("mf:")?.split_once('.')?;
                (!namespace.is_empty() && !key.is_empty()).then(|| format!("{namespace}.{key}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use psm_core::{Error, Result};
    use serde_json::json;
    use std::{collections::VecDeque, sync::Mutex};

    const STORES: &str = include_str!("fixtures/stores.json");
    const CASES_LIST: &str = include_str!("fixtures/cases_list.json");
    const CASE: &str = include_str!("fixtures/case.json");
    const MESSAGE_CREATED: &str = include_str!("fixtures/message_created.json");
    const EVENTS_PAGE: &str = include_str!("fixtures/events_page.json");
    const EVENTS_OTHER: &str = include_str!("fixtures/events_other_types.json");
    const ERROR_ENVELOPE: &str = include_str!("fixtures/error_envelope.json");

    const STORE: &str = "e8f6b61a-40a4-4504-8090-d9ea5b62a069";
    const CASE_ID: &str = "3b1f8c52-1c2e-4c35-9a0e-8a1f2b3c4d5e";
    const MESSAGE_ID: &str = "9c2d4e6f-1a3b-4c5d-8e7f-0a1b2c3d4e5f";
    /// 2026-11-20T08:12:03.512Z
    const CREATED_MS: i64 = 1_795_162_323_512;

    /// Replays the queued replies in order and records every request.
    #[derive(Default)]
    struct Recording {
        replies: Mutex<VecDeque<Result<HttpReply>>>,
        requests: Mutex<Vec<PricingRequest>>,
    }
    impl Recording {
        fn reply(&self, status: u16, body: Json) -> &Self {
            self.replies
                .lock()
                .unwrap()
                .push_back(Ok(HttpReply::new(status, body)));
            self
        }
        fn fail(&self, error: Error) -> &Self {
            self.replies.lock().unwrap().push_back(Err(error));
            self
        }
        fn requests(&self) -> Vec<PricingRequest> {
            self.requests.lock().unwrap().clone()
        }
    }
    impl PricingHttp for Recording {
        fn send(&self, request: PricingRequest) -> Result<HttpReply> {
            self.requests.lock().unwrap().push(request);
            self.replies
                .lock()
                .unwrap()
                .pop_front()
                .expect("a queued reply")
        }
    }

    fn fixture(text: &str) -> Json {
        serde_json::from_str(text).unwrap()
    }

    fn error_body(code: &str) -> Json {
        let mut body = fixture(ERROR_ENVELOPE);
        body["error"]["code"] = json!(code);
        body
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

    /// The cases-list fixture's case with its messages replaced.
    fn case_with_messages(messages: Json) -> Json {
        let mut case = fixture(CASES_LIST)[0].clone();
        case["messages"] = messages;
        case
    }

    fn wire_message(id: &str, author: Json, extra: Json) -> Json {
        let mut m = json!({
            "id": id, "caseId": CASE_ID, "author": author, "text": format!("text {id}"),
            "createdAt": "2026-11-20T08:12:03.512Z"
        });
        for (k, v) in extra.as_object().unwrap() {
            m[k] = v.clone();
        }
        m
    }

    fn user() -> Json {
        json!({"kind": "user", "id": "u1", "name": "Elin Lindqvist"})
    }

    fn event_with(kind: &str, data: Json) -> Json {
        json!({"id": 7, "type": kind, "storeId": STORE, "productId": "1", "caseId": CASE_ID,
               "actorId": null, "data": data, "at": "2026-11-20T08:12:03.530Z"})
    }

    fn page(events: Vec<Json>) -> Json {
        json!({"events": events, "nextCursor": 7})
    }

    #[test]
    fn c1_c1a_stores_parses_the_fixture() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(STORES));
        assert_eq!(
            adapter.stores().unwrap(),
            vec![ProviderStore {
                id: STORE.into(),
                name: "Dev store".into(),
                shop_domain: "test-item-position-scanner.myshopify.com".into(),
            }]
        );
    }

    #[test]
    fn c1_c1b_stores_sends_one_get_with_the_bearer() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(STORES));
        adapter.stores().unwrap();
        assert_eq!(
            http.requests(),
            vec![PricingRequest {
                method: Method::Get,
                url: "https://pricing.example/integration/v1/stores".into(),
                bearer: Some("k1".into()),
                body: None,
            }]
        );
    }

    #[test]
    fn c1_c2a_unresolved_maps_the_case_and_its_message() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(CASES_LIST));
        let list = adapter.unresolved(STORE).unwrap();
        assert_eq!(
            list,
            vec![Conversation {
                id: CASE_ID.into(),
                store_id: STORE.into(),
                product_id: "15531808096409".into(),
                sku: Some("BEYO-1880".into()),
                product_title: "Vintage Brass Wall Mirror, Round".into(),
                topic: "Condition".into(),
                blocking: true,
                status: ConversationStatus::Open,
                resolved_by: None,
                created_at: CREATED_MS,
                updated_at: CREATED_MS,
                read_mark: None,
                messages: vec![Message {
                    id: MESSAGE_ID.into(),
                    conversation_id: CASE_ID.into(),
                    author: Author {
                        kind: AuthorKind::Asker,
                        name: "Elin Lindqvist".into(),
                    },
                    text: "Is the mirror glass original? How far does the desilvering extend?"
                        .into(),
                    quote: Some(Quote {
                        field: "condition".into(),
                        field_label: "Condition".into(),
                        text: "Possible desilvering at the lower edge.".into(),
                    }),
                    reply_to: None,
                    attachments: vec![Attachment {
                        media_id: "gid://shopify/MediaImage/41234567890".into(),
                        thumb_url: Some("https://cdn.shopify.com/s/files/…_480x.jpg".into()),
                        url: Some("https://cdn.shopify.com/s/files/…_1600x.jpg".into()),
                    }],
                    created_at: CREATED_MS,
                    edited_at: None,
                }],
            }]
        );
    }

    #[test]
    fn c1_c2b_unresolved_asks_for_the_photographer_unresolved_cases() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(CASES_LIST));
        adapter.unresolved(STORE).unwrap();
        let requests = http.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, Method::Get);
        assert_eq!(
            requests[0].url,
            format!(
                "https://pricing.example/integration/v1/stores/{STORE}/cases?recipient=photographer&status=unresolved"
            )
        );
    }

    #[test]
    fn c1_c2c_deleted_and_textless_messages_are_not_kept() {
        let (http, adapter) = adapter();
        http.reply(
            200,
            json!([case_with_messages(json!([
                wire_message("m1", user(), json!({"deleted": true})),
                wire_message("m2", user(), json!({"text": null})),
                wire_message("m3", user(), json!({})),
            ]))]),
        );
        let list = adapter.unresolved(STORE).unwrap();
        let ids: Vec<_> = list[0].messages.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["m3"]);
    }

    #[test]
    fn c1_c2d_author_kinds() {
        let (http, adapter) = adapter();
        http.reply(
            200,
            json!([case_with_messages(json!([
                wire_message("m1", user(), json!({})),
                wire_message(
                    "m2",
                    json!({"kind": "service", "id": "s", "name": "Photographer", "isMe": true}),
                    json!({})
                ),
                wire_message(
                    "m3",
                    json!({"kind": "service", "id": "s2", "name": "Ops"}),
                    json!({})
                ),
            ]))]),
        );
        let list = adapter.unresolved(STORE).unwrap();
        let kinds: Vec<_> = list[0].messages.iter().map(|m| m.author.kind).collect();
        assert_eq!(
            kinds,
            [AuthorKind::Asker, AuthorKind::Me, AuthorKind::Other]
        );
    }

    #[test]
    fn c1_c2e_null_sku_and_absent_optional_fields() {
        let (http, adapter) = adapter();
        let mut case = case_with_messages(json!([{
            "id": "m1", "caseId": CASE_ID, "author": user(), "text": "plain",
            "createdAt": "2026-11-20T08:12:03.512Z"
        }]));
        case["sku"] = Json::Null;
        http.reply(200, json!([case]));
        let list = adapter.unresolved(STORE).unwrap();
        assert_eq!(list[0].sku, None);
        let m = &list[0].messages[0];
        assert!(m.attachments.is_empty());
        assert_eq!(m.quote, None);
        assert_eq!(m.reply_to, None);
        assert_eq!(m.edited_at, None);
    }

    #[test]
    fn c1_c2f_operations_cases_are_dropped() {
        let (http, adapter) = adapter();
        let mut operations = case_with_messages(json!([]));
        operations["id"] = json!("ops-case");
        operations["recipient"] = json!("operations");
        let photographer = case_with_messages(json!([]));
        http.reply(200, json!([operations, photographer]));
        let ids: Vec<_> = adapter
            .unresolved(STORE)
            .unwrap()
            .into_iter()
            .map(|c| c.id)
            .collect();
        assert_eq!(ids, [CASE_ID]);
    }

    #[test]
    fn c1_c2g_conversation_reads_reply_to_and_edit_time() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(CASE));
        let c = adapter.conversation(STORE, CASE_ID).unwrap().unwrap();
        let reply = &c.messages[1];
        assert_eq!(
            reply.reply_to,
            Some(ReplyRef {
                message_id: MESSAGE_ID.into(),
                author_name: "Elin Lindqvist".into(),
                excerpt: "Is the mirror glass original?".into(),
            })
        );
        // 2026-11-20T08:21:00.000Z
        assert_eq!(reply.edited_at, Some(1_795_162_860_000));
    }

    #[test]
    fn c1_c2h_conversation_for_operations_is_none() {
        let (http, adapter) = adapter();
        let mut case = fixture(CASE);
        case["recipient"] = json!("operations");
        http.reply(200, case);
        assert_eq!(adapter.conversation(STORE, CASE_ID).unwrap(), None);
    }

    #[test]
    fn c1_c3a_events_page_maps_the_message_created_event() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(EVENTS_PAGE));
        let page = adapter.events_after(STORE, 1800, 500).unwrap();
        assert_eq!(
            page,
            EventPage {
                events: vec![ProviderEvent {
                    id: 1842,
                    kind: EventKind::MessageCreated { by_asker: true },
                    conversation_id: Some(CASE_ID.into()),
                    for_us: true,
                }],
                next_cursor: 1842,
            }
        );
        let url = &http.requests()[0].url;
        assert_eq!(url.split_once('?').unwrap().1, "after=1800&limit=500");
    }

    #[test]
    fn c1_c3b_other_event_types_are_other_and_not_for_us() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(EVENTS_OTHER));
        let page = adapter.events_after(STORE, 1842, 500).unwrap();
        assert_eq!(
            page.events,
            vec![
                ProviderEvent {
                    id: 1843,
                    kind: EventKind::Other,
                    conversation_id: None,
                    for_us: false
                },
                ProviderEvent {
                    id: 1844,
                    kind: EventKind::Other,
                    conversation_id: None,
                    for_us: false
                },
            ]
        );
    }

    #[test]
    fn c1_c3c_case_updated_reads_deleted_and_resolved_by() {
        let (http, adapter) = adapter();
        let data = |extra: Json| {
            let mut d = json!({"caseId": CASE_ID, "sku": "BEYO-1880", "recipient": "photographer",
                               "status": "resolved", "blocking": true});
            for (k, v) in extra.as_object().unwrap() {
                d[k] = v.clone();
            }
            d
        };
        http.reply(
            200,
            page(vec![
                event_with("case.updated", data(json!({"deleted": true}))),
                event_with("case.updated", data(json!({"resolvedBy": "activation"}))),
                event_with("case.updated", data(json!({}))),
            ]),
        );
        let kinds: Vec<_> = adapter
            .events_after(STORE, 0, 500)
            .unwrap()
            .events
            .into_iter()
            .map(|e| e.kind)
            .collect();
        assert_eq!(
            kinds,
            [
                EventKind::ConversationUpdated {
                    deleted: true,
                    resolved_by: None
                },
                EventKind::ConversationUpdated {
                    deleted: false,
                    resolved_by: Some(ResolvedBy::Activation)
                },
                EventKind::ConversationUpdated {
                    deleted: false,
                    resolved_by: None
                },
            ]
        );
    }

    #[test]
    fn c1_c3d_the_other_case_and_message_types() {
        let (http, adapter) = adapter();
        let data = |extra: Json| {
            let mut d = json!({"caseId": CASE_ID, "recipient": "photographer"});
            for (k, v) in extra.as_object().unwrap() {
                d[k] = v.clone();
            }
            d
        };
        http.reply(
            200,
            page(vec![
                event_with(
                    "message.created",
                    data(json!({"messageId": "m", "authorKind": "service"})),
                ),
                event_with("case.created", data(json!({"topic": "Condition"}))),
                event_with("message.updated", data(json!({"messageId": "m"}))),
                event_with("message.deleted", data(json!({"messageId": "m"}))),
            ]),
        );
        let kinds: Vec<_> = adapter
            .events_after(STORE, 0, 500)
            .unwrap()
            .events
            .into_iter()
            .map(|e| e.kind)
            .collect();
        assert_eq!(
            kinds,
            [
                EventKind::MessageCreated { by_asker: false },
                EventKind::ConversationCreated,
                EventKind::MessageUpdated,
                EventKind::MessageDeleted,
            ]
        );
    }

    #[test]
    fn c1_c3e_operations_events_are_not_for_us() {
        let (http, adapter) = adapter();
        http.reply(
            200,
            page(vec![event_with(
                "case.created",
                json!({"caseId": CASE_ID, "recipient": "operations", "topic": "Other"}),
            )]),
        );
        let page = adapter.events_after(STORE, 0, 500).unwrap();
        assert!(!page.events[0].for_us);
    }

    #[test]
    fn c1_c4a_invalid_key_is_refused() {
        let (http, adapter) = adapter();
        http.reply(401, error_body("invalid_api_key"));
        assert_eq!(
            adapter.events_after(STORE, 0, 500),
            Err(ProviderError::Refused)
        );
    }

    #[test]
    fn c1_c4b_missing_scope_is_refused() {
        let (http, adapter) = adapter();
        http.reply(403, error_body("insufficient_scope"));
        assert_eq!(
            adapter.events_after(STORE, 0, 500),
            Err(ProviderError::Refused)
        );
    }

    #[test]
    fn c1_c4c_unknown_store_is_store_gone() {
        let (http, adapter) = adapter();
        http.reply(404, error_body("not_found"))
            .reply(404, error_body("not_found"));
        assert_eq!(
            adapter.events_after(STORE, 0, 500),
            Err(ProviderError::StoreGone)
        );
        assert_eq!(adapter.unresolved(STORE), Err(ProviderError::StoreGone));
    }

    #[test]
    fn c1_c4d_unknown_conversation_is_none() {
        let (http, adapter) = adapter();
        http.reply(404, error_body("not_found"));
        assert_eq!(adapter.conversation(STORE, CASE_ID), Ok(None));
    }

    #[test]
    fn c1_c4e_server_errors_are_unreachable() {
        let (http, adapter) = adapter();
        http.reply(500, error_body("internal_error"))
            .reply(502, json!("<html>Bad Gateway</html>"));
        assert!(matches!(
            adapter.events_after(STORE, 0, 500),
            Err(ProviderError::Unreachable(_))
        ));
        assert!(matches!(
            adapter.events_after(STORE, 0, 500),
            Err(ProviderError::Unreachable(_))
        ));
    }

    #[test]
    fn c1_c4f_transport_errors_are_unreachable() {
        let (http, adapter) = adapter();
        http.fail(Error::Timeout)
            .fail(Error::Provider("http: connection refused".into()));
        assert!(matches!(
            adapter.events_after(STORE, 0, 500),
            Err(ProviderError::Unreachable(_))
        ));
        assert!(matches!(
            adapter.events_after(STORE, 0, 500),
            Err(ProviderError::Unreachable(_))
        ));
    }

    #[test]
    fn c1_c4g_unexpected_shapes_are_bad_replies() {
        let (http, adapter) = adapter();
        let mut bad_time = fixture(EVENTS_PAGE);
        bad_time["events"][0]["at"] = json!("yesterday");
        http.reply(200, json!({"events": "x"})).reply(200, bad_time);
        assert!(matches!(
            adapter.events_after(STORE, 0, 500),
            Err(ProviderError::BadReply(_))
        ));
        assert!(matches!(
            adapter.events_after(STORE, 0, 500),
            Err(ProviderError::BadReply(_))
        ));
    }

    #[test]
    fn c1_c4h_unknown_fields_are_ignored() {
        let (http, adapter) = adapter();
        let mut stores = fixture(STORES);
        stores[0]["region"] = json!("eu");
        let mut events = fixture(EVENTS_PAGE);
        events["hasMore"] = json!(false);
        http.reply(200, stores).reply(200, events);
        let (plain_http, plain) = self::adapter();
        plain_http
            .reply(200, fixture(STORES))
            .reply(200, fixture(EVENTS_PAGE));
        assert_eq!(adapter.stores().unwrap(), plain.stores().unwrap());
        assert_eq!(
            adapter.events_after(STORE, 1800, 500).unwrap(),
            plain.events_after(STORE, 1800, 500).unwrap()
        );
    }

    fn new_message(reply_to: Option<&str>, media: &[&str]) -> NewMessage {
        NewMessage {
            client_mutation_id: "m-1".into(),
            text: "Original glass.".into(),
            reply_to_message_id: reply_to.map(str::to_owned),
            attachment_media_ids: media.iter().map(|m| (*m).to_owned()).collect(),
        }
    }

    #[test]
    fn c1_c5a_post_message_sends_text_and_client_id() {
        let (http, adapter) = adapter();
        http.reply(201, fixture(MESSAGE_CREATED));
        let m = adapter
            .post_message(STORE, CASE_ID, &new_message(None, &[]))
            .unwrap();
        let request = &http.requests()[0];
        assert_eq!(request.method, Method::Post);
        assert_eq!(
            request.url,
            format!(
                "https://pricing.example/integration/v1/stores/{STORE}/cases/{CASE_ID}/messages"
            )
        );
        // Exactly these two fields (JSON object equality; key order is not meaningful).
        assert_eq!(
            request.body,
            Some(json!({"text": "Original glass.", "clientMutationId": "m-1"}))
        );
        assert_eq!(m.author.kind, AuthorKind::Me);
        assert_eq!(m.id, "7a8b9c0d-1e2f-4a3b-8c4d-5e6f7a8b9c0d");
    }

    #[test]
    fn c1_c5b_post_message_with_reply_target_and_media() {
        let (http, adapter) = adapter();
        http.reply(201, fixture(MESSAGE_CREATED));
        adapter
            .post_message(STORE, CASE_ID, &new_message(Some("9c2d"), &["g1", "g2"]))
            .unwrap();
        let body = http.requests()[0].body.clone().unwrap();
        assert_eq!(body["replyToMessageId"], json!("9c2d"));
        assert_eq!(body["attachmentMediaIds"], json!(["g1", "g2"]));
    }

    #[test]
    fn c1_c5c_post_message_refusals() {
        let (http, adapter) = adapter();
        http.reply(400, error_body("invalid_attachment"))
            .reply(422, error_body("validation_error"))
            .reply(404, error_body("not_found"));
        let message = "No product with SKU 'X' in this store.".to_owned();
        assert_eq!(
            adapter.post_message(STORE, CASE_ID, &new_message(None, &[])),
            Err(ProviderError::Rejected {
                code: "invalid_attachment".into(),
                message: message.clone(),
            })
        );
        assert!(matches!(
            adapter.post_message(STORE, CASE_ID, &new_message(None, &[])),
            Err(ProviderError::Rejected { code, .. }) if code == "validation_error"
        ));
        assert_eq!(
            adapter.post_message(STORE, CASE_ID, &new_message(None, &[])),
            Err(ProviderError::NotFound)
        );
    }

    #[test]
    fn c1_c5d_mark_answered_patches_the_status() {
        let (http, adapter) = adapter();
        http.reply(200, fixture(CASE))
            .reply(409, error_body("case_resolved"));
        let c = adapter.mark_answered(STORE, CASE_ID).unwrap();
        let request = &http.requests()[0];
        assert_eq!(request.method, Method::Patch);
        assert_eq!(
            request.url,
            format!("https://pricing.example/integration/v1/stores/{STORE}/cases/{CASE_ID}")
        );
        assert_eq!(request.body, Some(json!({"status": "answered"})));
        assert_eq!(c.status, ConversationStatus::Answered);
        assert_eq!(
            adapter.mark_answered(STORE, CASE_ID),
            Err(ProviderError::Resolved)
        );
    }

    #[test]
    fn c4_c9a_title_names_the_product_title() {
        assert_eq!(
            adapter().1.attribute_key("title").as_deref(),
            Some("product.title")
        );
    }

    #[test]
    fn c4_c9b_description_names_the_product_description() {
        assert_eq!(
            adapter().1.attribute_key("description").as_deref(),
            Some("product.description")
        );
    }

    #[test]
    fn c4_c9c_condition_names_the_condition_report_text() {
        assert_eq!(
            adapter().1.attribute_key("condition").as_deref(),
            Some("custom.condition_report_description")
        );
    }

    #[test]
    fn c4_c9d_a_metafield_names_its_definition() {
        let (_, a) = adapter();
        assert_eq!(
            a.attribute_key("mf:beyo.designer").as_deref(),
            Some("beyo.designer")
        );
        assert_eq!(
            a.attribute_key("mf:beyo_pricing.material").as_deref(),
            Some("beyo_pricing.material")
        );
    }

    #[test]
    fn c4_c9e_anything_else_names_no_field() {
        let (_, a) = adapter();
        for field in ["mf:designer", "mf:.x", "mf:beyo.", "price", ""] {
            assert_eq!(a.attribute_key(field), None, "{field:?}");
        }
    }
}
