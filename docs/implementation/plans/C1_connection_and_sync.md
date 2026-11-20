---
phase: C1_connection_and_sync
title: Connection and sync — Settings → Pricing takes the server address and the API key (kept in the Keychain), matches the key's store to the connected Shopify shop and says in words whether it is connected; Session Manager then keeps an exact local copy of the photographer's conversations, checking Pricing every 10 seconds without ever holding up a screen
difficulty: high
projection: yes
depends_on: []
blocked_on: none
---
# C1 — Connection and sync

**Tracker:** NOT_STARTED (2026-11-20, planner)

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. `master_plan.md` §4 and `STATUS.md` are not used or edited by conversations milestones.)

## Goal
Book `09B` §3–§6 by contract **MC-C1** (`docs/implementation/master_plan_conversations.md` §5). (U1) The core vocabulary and port, the Beyo Pricing adapter over its HTTP seam, the fake Pricing server. (U2) The settings slot, the key in the Keychain, the connection states and the store match. (U3) The local copy, the first connection, the pull, read marks, errors, notices. (U4) The commands and Settings → Pricing. **Not here:** any list, count, icon, toast, chime or banner (C2); sending, Mark answered, the conversation view (C3; the adapter's two writing methods are built and tested here, nothing calls them yet); the visit (C4); webhooks (out of scope, 09B §11).

## How to work this plan (flow.md §10, master_plan_conversations.md §3)
One runner phase of 4 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `C1·C<n>(<row>)`; tests named after their row (`c1_c7a_<slug>`). Open details: the smallest option consistent with 09B and MC-C1, recorded under **Assumptions** in the handoff. Reviewer notes: `docs/implementation/backlog_conversations/C1_connection_and_sync.md`. **Never call Pricing or Shopify, never launch the built app** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Vocabulary, port, the Beyo Pricing adapter, the fake Pricing server | 6 | 29 | **yes** |
| U2 | Settings slot, the key, connection states, store match | 4 | 16 | **yes** |
| U3 | Local copy, first connection, the pull, read marks, errors, notices | 8 | 50 | **yes** |
| U4 | Commands and Settings → Pricing | 3 | 15 | **yes** |
| **Total** | | **21** | **110** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6, §10) · `docs/implementation/master_plan_conversations.md` (all; **§5 MC-C1 is the contract**, §4 the names) · book `09B` §2–§6, §10, §11 · book `11` (layers) · book `09A` MC-23 (where the shop domain lives) · the package (read-only): `~/Desktop/Developer/pricing-app/docs/integrations/session-manager/{API.md, EVENTS_AND_WEBHOOKS.md, openapi-integration.json}` · `docs/guide/03_rules_and_principles.md` (H7, H8, secrets, persistence, tests) · `docs/guide/04_change_impact.md` rows "Tauri command/DTO", "AppEvent", "settings field", "The schema".

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Saving the key tests the connection at once;** Test connection repeats it.
- **Session Manager only reads from Pricing in this step;** nothing can be sent yet.
- **A Mac whose Shopify shop was changed makes no call to Pricing** until Test connection is pressed again.
- **A different Pricing store empties the local copy;** the same store keeps it.
- **The first connection is silent** and takes the 200 oldest unresolved conversations plus everything named in the last 30 days of events (Pricing's own limits).
- **When Pricing cannot be reached the wait doubles from 10 seconds up to 5 minutes;** the first success returns to 10 seconds.
- **The address must be https;** the key is never shown again after it is saved.

## Unit 1 — Vocabulary, port, the Beyo Pricing adapter, the fake Pricing server

**Unit id:** `C1·U1` · **Rows:** C1·C1–C6 · **Changes app files:** yes

### Goal
`psm_core::domain::conversation`, `psm_core::ports::conversations::ConversationsProvider`, `BeyoPricingAdapter` over `PricingHttp`, the fixtures, and `psm_testkit::fake_pricing::FakePricing` (an in-memory Pricing behind the same HTTP seam).

### Read first
Master plan §3 ("Rule 17, here"), §4 (Vocabulary, Ports, Adapter) · the package's `API.md` (every endpoint and the error table), `EVENTS_AND_WEBHOOKS.md` (envelope, types, `data`), `openapi-integration.json` (`components.schemas`: nullability and required lists are the truth where `API.md` is silent) · `crates/integrations/src/commerce/shopify/transport.rs` (`HttpReply`, `ReqwestTransport`, `map_error`: the pattern and the two error shapes) · `crates/testkit/src/fake_transport.rs` (how a fake sits at the HTTP seam) · `crates/core/src/ports/commerce.rs` (how a provider port is written) · `crates/core/src/error.rs`.

### Files
New: `crates/core/src/domain/conversation.rs`, `crates/core/src/ports/conversations.rs` (the provider trait only), `crates/integrations/src/conversations/{mod.rs, beyo_pricing/{mod.rs, adapter.rs, transport.rs, types.rs}}`, `crates/integrations/src/conversations/beyo_pricing/fixtures/{stores.json, cases_list.json, case.json, message_created.json, events_page.json, events_other_types.json, error_envelope.json}`, `crates/testkit/src/fake_pricing.rs`. Edited: `crates/core/src/domain/mod.rs`, `crates/core/src/ports/mod.rs`, `crates/integrations/src/lib.rs` (the `mod` line), `crates/integrations/Cargo.toml` (only if `chrono` is needed), `crates/testkit/src/lib.rs`.

### Core mechanisms (the only functions with unit tests)
The adapter's reply parsing and error mapping (`beyo_pricing/adapter.rs`, inline tests over the fixture files and a recording `PricingHttp`) · `beyo_pricing::checked_server` (the https rule of master plan §4 Settings).

### Ordered tasks
1. The vocabulary and `ProviderError` of master plan §4, exactly. `ProviderError` has a `sentence()` only in the adapter crate (the words name Pricing; core carries no such text).
2. `ConversationsProvider` (six methods; `attribute_key` is C4's).
3. `PricingHttp` + `ReqwestPricing` (own worker thread and runtime as `ReqwestTransport` does; connect 10 s, total 20 s; `Authorization: Bearer <key>`, `Content-Type: application/json` on bodies, `Accept: application/json`; errors through `without_url`).
4. `types.rs`: wire types from `openapi-integration.json` (`IntegrationStore`, `IntegrationCase`, `Message`, `Actor`, `Quote`, `ReplyRef`, `Attachment`, `EventPage`, `StoreEvent`, `ErrorEnvelope`, `IntegrationMessageCreate`, `IntegrationCaseUpdate`), camelCase, `Serialize + Deserialize`, unknown fields ignored, optional and nullable fields as `Option` with `#[serde(default)]` (`attachments`, `quote`, `replyTo`, `editedAt`, `deleted`, `isMe`, `sku`, `productId`/`caseId`/`actorId` of an event).
5. `BeyoPricingAdapter::new(http, server: impl Fn() -> String, key: impl Fn() -> Option<String>)`: requests and mappings as the rows below. Requests: `GET /stores` · `GET /stores/{id}/cases?recipient=photographer&status=unresolved` · `GET /stores/{id}/cases/{caseId}` · `GET /stores/{id}/events?after={cursor}&limit={limit}` · `POST /stores/{id}/cases/{caseId}/messages` · `PATCH /stores/{id}/cases/{caseId}` with `{"status":"answered"}`. Path segments are percent-encoded. `conversation()` answers `None` for a 404 and for a case whose `recipient` is not `photographer`; `unresolved()` drops cases whose `recipient` is not `photographer`. Times: ISO 8601 → UTC ms; an unparseable time, a missing required field or an unknown `status` → `BadReply`.
6. Fixture files: the JSON examples of `API.md` and `EVENTS_AND_WEBHOOKS.md` verbatim (elided ids `…` filled with made-up UUIDs; the example shop domain and names of the package kept), plus `events_other_types.json` built from the `StoreEvent` schema (a `workflow.updated` and an `inbox.updated` event with `caseId: null`, `productId: null` and a `data` object without `recipient`).
7. `FakePricing: PricingHttp`: stores, cases with messages, an event log with monotonic ids; pricer-side helpers (`ask`, `pricer_reply`, `edit_message`, `delete_message`, `resolve`, `resolve_by_activation`, `delete_case`, `ask_operations`, `raw_event`, `change_silently` = a case changes with no event), the six endpoints with the package's semantics (unresolved filter, max 200, `after`/`limit`, `nextCursor` unchanged on an empty page, `clientMutationId` returning the same message, a reply moving the case to `answered` and logging `message.created` with `authorKind: service`, PATCH on a resolved case → 409 `case_resolved`, PATCH `resolved` → 422, unknown store → 404, a wrong or missing bearer → 401 `invalid_api_key`), scripted failures (`fail_matching(path_part, n, status)` = the next `n` requests whose path contains `path_part` answer `status` with the error envelope, `timeout_next(n)`, `drop_answer_next(n)` = the server acts and the transport returns `Error::Timeout`, `hold()` / `release()`), `redeliver(event_id)` (the event is returned once more in the next page), and a record of every request (method, path, query, body, `authorized: bool`; the bearer value is never kept). It serialises through `types.rs`.
8. Tests for the rows.

### Criteria (6 criteria, 29 rows)
Boundary: `BeyoPricingAdapter` methods over a recording `PricingHttp` that replays a fixture file or a typed reply (outcome = the returned value or `ProviderError`, and the recorded request); rows C1·C6 over `FakePricing`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1·C1(a) | `stores()` | reply 200 `stores.json` | one `ProviderStore` with the fixture's id, name and `shop_domain` | 09B §4; MC-C1.3 |
| C1·C1(b) | same | server `https://pricing.example`, key `k1` | exactly one request: `GET https://pricing.example/integration/v1/stores`, bearer `k1`, no body | 09B §4 |
| C1·C2(a) | `unresolved(store)` | reply 200 `cases_list.json` | one `Conversation`: topic `Condition`, blocking, status `Open`, sku and product title of the fixture, `created_at`/`updated_at` = 2026-11-20T08:12:03.512Z in ms; one message: author `Asker` "Elin Lindqvist", the text, the quote (`field` `condition`, label, text), one attachment with its media id and both URLs, `edited_at` none | 09B §3; MC-C1.4 |
| C1·C2(b) | same | — | the request is `GET …/stores/{id}/cases?recipient=photographer&status=unresolved` | 09B §5.2 |
| C1·C2(c) | same | a case with three messages: one `deleted: true`, one `text: null`, one normal | the conversation has only the normal message | MC-C1.4; `Message.text` nullable (openapi) |
| C1·C2(d) | same | messages by `{kind: user}`, `{kind: service, isMe: true}`, `{kind: service}` (no `isMe`) | author kinds `Asker`, `Me`, `Other` in that order | 09B §3 |
| C1·C2(e) | same | a case with `sku: null`; a message without `attachments`, `quote`, `replyTo`, `editedAt` | `sku` none; the message has no attachment, quote, reply or edit time | `IntegrationCase.sku` nullable (openapi) |
| C1·C2(f) | same | a list holding one `recipient: operations` case and one `photographer` case | only the photographer one is returned | 09B §3 (only photographer conversations) |
| C1·C2(g) | `conversation(store, id)` | reply 200 `case.json` with a `replyTo` and an `editedAt` | the message's `reply_to` (id, author name, excerpt) and `edited_at` in ms | 09B §3 |
| C1·C2(h) | same | reply 200, a case with `recipient: operations` | `Ok(None)` | 09B §3 |
| C1·C3(a) | `events_after(store, 1800, 500)` | reply 200 `events_page.json` | one event: id 1842, `MessageCreated {by_asker: true}`, its conversation id, `for_us` true; `next_cursor` 1842; the request query is `after=1800&limit=500` | 09B §5.3; MC-C1.6 |
| C1·C3(b) | same | reply 200 `events_other_types.json` | two events of kind `Other`, `conversation_id` none, `for_us` false, ids kept | `StoreEvent.type` enum (openapi); MC-C1.6 |
| C1·C3(c) | same | a `case.updated` with `data.deleted: true`; one with `data.resolvedBy: "activation"`; one with neither | `ConversationUpdated {deleted: true, resolved_by: None}`; `{deleted: false, resolved_by: Some(Activation)}`; `{deleted: false, resolved_by: None}` | 09B §5.3; MC-C1.6 |
| C1·C3(d) | same | a `message.created` with `authorKind: service`; a `case.created`; a `message.updated`; a `message.deleted` | `MessageCreated {by_asker: false}`; `ConversationCreated`; `MessageUpdated`; `MessageDeleted` | 09B §5.3, §5.5 |
| C1·C3(e) | same | an event with `data.recipient: operations` | `for_us` false | 09B §5.3 |
| C1·C4(a) | `events_after` | reply 401 `error_envelope.json` with code `invalid_api_key` | `Err(Refused)` | 09B §5.4 |
| C1·C4(b) | same | reply 403 `insufficient_scope` | `Err(Refused)` | 09B §5.4 |
| C1·C4(c) | same; `unresolved` | reply 404 `not_found` | `Err(StoreGone)` for both | 09B §5.4 |
| C1·C4(d) | `conversation(store, id)` | reply 404 `not_found` | `Ok(None)` | 09B §5.3 |
| C1·C4(e) | `events_after` | reply 500 with the envelope; reply 502 whose body is the string `"<html>Bad Gateway</html>"` | `Err(Unreachable)` for both | 09B §5.4 |
| C1·C4(f) | same | the transport returns `Error::Timeout`; the transport returns `Error::Provider("http: connection refused")` | `Err(Unreachable)` for both | 09B §5.4; transport `map_error` |
| C1·C4(g) | same | reply 200 `{"events": "x"}`; reply 200 with an event whose `at` is `"yesterday"` | `Err(BadReply)` for both | MC-C1.8 |
| C1·C4(h) | `stores()` | reply 200 with an extra unknown field on the store and on the top level of an event page (second call) | parsed as if the fields were absent | API.md "changes are additive" |
| C1·C5(a) | `post_message` | text "Original glass.", client id `m-1`, no reply target, no media; reply 201 `message_created.json` | request `POST …/cases/{caseId}/messages` with body exactly `{"text":"Original glass.","clientMutationId":"m-1"}`; returns the message with author `Me` and the fixture's id | 09B §8; MC-C2.2 |
| C1·C5(b) | same | with a reply target `9c2d` and two media ids | the body also has `"replyToMessageId":"9c2d"` and `"attachmentMediaIds"` with the two ids in order | 09B §8 |
| C1·C5(c) | same | reply 400 `invalid_attachment`; reply 422 `validation_error`; reply 404 `not_found` | `Err(Rejected {code: "invalid_attachment", message})` with the envelope's message; `Err(Rejected {code: "validation_error", ..})`; `Err(NotFound)` | 09B §8; MC-C2.2 |
| C1·C5(d) | `mark_answered` | reply 200 `case.json` (status answered); then reply 409 `case_resolved` | request `PATCH …/cases/{caseId}` with body exactly `{"status":"answered"}`, returns the conversation with status `Answered`; then `Err(Resolved)` | 09B §8; MC-C2.7 |
| C1·C6(a) | `checked_server` | `https://pricing.example/` ; ` https://pricing.example ` ; `http://pricing.example` ; `https://pricing.example/x` ; `https://u:p@pricing.example` ; `pricing.example` ; `https://` | `https://pricing.example` for the first two; refused for each of the other five | 09B §4 (https only) |
| C1·C6(b) | the adapter over `FakePricing` | key `secret-key-123`; one call answered 401, one `Error::Provider`, one 500 | no returned `ProviderError` (Display and `sentence()`), no log line captured during the calls and no entry of the fake's request record contains `secret-key-123` | CR2 |

### Delegated choices
Private helper names · how the adapter percent-encodes · how `FakePricing` names its helpers beyond those listed (every helper must have a test caller in C1–C3, rule 4: drop any that has none).

### Notes
- Hazard: `product_id` is the query name in `openapi-integration.json` while `API.md` writes `productId`; neither is used (09B lists by store). Do not add a per-SKU or per-product call.
- Hazard: the event log also carries types outside the five case/message types (`workflow.updated`, `finalization.updated`, `inbox.updated`), with `caseId: null` and no `data.recipient`. They are `Other`, never an error.
- Hazard (H7): no Pricing word in `crates/core`. `recipient=photographer`, `authorKind`, "case" and the URL live in the adapter.
- The log capture for C1·C6(b): `psm_testkit::logs`.

### Unit test commands
`cargo test -p psm-integrations --lib conversations::` · `cargo check -p psm-testkit --tests` · `cargo clippy -p psm-core -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — Settings slot, the key, connection states, store match

**Unit id:** `C1·U2` · **Rows:** C1·C7–C10 · **Changes app files:** yes

### Goal
The `conversations` settings slot, the key in the Keychain, `conversation_sync` (migration 019), `ConversationsService::{connection, test_connection, key_set, settings, settings_save}` by MC-C1.1–3, and the wiring.

### Read first
Master plan §4 (Storage, Secret, Settings, Service), §5 MC-C1.1–3 · `crates/core/src/ports/secrets.rs`, `crates/infra/src/keychain.rs`, `crates/testkit/src/fake_secrets.rs` · `crates/research/src/api_impl.rs` (`secrets_set`, the research key precedent) · `crates/infra/src/settings_file.rs` (`Settings`, the `writer` slot) · `crates/core/src/services/settings.rs` (the accepted key list) · `crates/integrations/src/lib.rs` (`App`, `AppHandles`, `SettingsSetContext::prepare` and its rebuild comparison, `with_test_research` as the seam pattern, `storage_background` as the loop pattern) · `crates/infra/src/sqlite/{store.rs (`migrations()`), saved_pages.rs}` · `crates/testkit/src/test_app.rs` (`with_fake_shop`).

### Files
New: `crates/infra/src/sqlite/migrations/019_conversations.sql`, `crates/infra/src/sqlite/conversations.rs`, `crates/core/src/services/conversations.rs`, `crates/testkit/tests/suite/c1_conversations_connection.rs`. Edited: `crates/core/src/ports/conversations.rs` (`ConversationStore`), `crates/core/src/services/{mod.rs, settings.rs}`, `crates/infra/src/sqlite/{mod.rs, store.rs}`, `crates/infra/src/settings_file.rs`, `crates/integrations/src/lib.rs`, `crates/testkit/src/test_app.rs`, `crates/testkit/tests/suite/main.rs`. Tests that enumerate the tables or the settings keys follow.

### Core mechanisms (the only functions with unit tests)
None (the rules are transactions; the rows are behaviour tests).

### Ordered tasks
1. Migration 019 with the four tables of master plan §4 (all four now, so C2–C4 add no migration) and its `M::up` line; `ConversationStore` methods for this unit; `SqliteStore` impl.
2. `Settings.conversations` slot, accepted by `SettingsService::set`, nulled in the rebuild comparison; `ConversationSettings::from_slot`; `settings_save(patch)` validates `server` with `checked_server` (a `server` equal to the default is stored as none).
3. `ConversationsService::new(store, provider, secrets, settings source, shop, clock, events)`; `App.conversations` and `AppHandles.conversations`; the adapter built in `App::build` with the live server (slot or default) and the key read through `SecretStore`; `with_test_conversations`; `TestApp::with_pricing()` (a fake shop whose domain ≈ the fake store's) and `TestApp::with_pricing_no_shop()` (the seam without a commerce integration), both exposing `TestApp.pricing`.
4. MC-C1.2 (state at build, evaluated at the first `tick` or `connection()` on a worker thread, never inside `App::build`: a Keychain read can prompt), MC-C1.3 (`test_connection`, `key_set`), `connection()`.
5. `conversations.connection` log line. Tests for the rows.

### Criteria (4 criteria, 16 rows)
Boundary: `ConversationsService` through `TestApp::with_pricing()` unless the row names another constructor (outcome = the returned `Connection`, the `conversation_sync` row, the fake secrets, `settings.json`, the fake's request record).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1·C7(a) | `settings()` | no `conversations` slot | `server` none, `sound` true, `banner` true; the effective server is `beyo_pricing::DEFAULT_SERVER` | 09B §4 |
| C1·C7(b) | `settings_save` | `{server: "https://pricing.example/"}` then `{sound: false}` | the slot holds `server` `https://pricing.example`, `sound` false, `banner` true | 09B §4 |
| C1·C7(c) | same | `{server: "http://pricing.example"}` | refused naming `server`; `settings.json` bytes unchanged | 09B §4 (https only) |
| C1·C7(d) | `App::settings_set_context().prepare` | a patch that changes only the `conversations` slot | `PreparedSettings::Simple` (no App rebuild) | master plan §4 Settings |
| C1·C8(a) | `key_set(Some("k-secret-9"))` | a matching store in the fake | the fake secrets hold `k-secret-9` under `conversations-api-key`; `settings.json` does not contain the string; `connection()` (serialised) does not contain it | CR2; 09B §4 |
| C1·C8(b) | `key_set(None)` | after (a) | the account is gone; state `no_key`; a later `tick` makes no request | 09B §4; MC-C1.1 |
| C1·C8(c) | `key_set(Some("k-secret-9"))` then a log line carrying the key | — | the captured log line shows the redaction, not the key | CR2 |
| C1·C9(a) | `test_connection` | key set; fake stores [{Dev store, `Shop-A.myshopify.com`}]; shop `shop-a.myshopify.com` | state `ready`; the saved store has the fake's id, name and domain; one request (`GET /stores`) | 09B §4; MC-C1.3 |
| C1·C9(b) | same | two stores, the second one matches | the second store is saved | MC-C1.3 |
| C1·C9(c) | same | no store matches; stores named "One", "Two" | state `store_unmatched`; `visible_stores` = ["One", "Two"]; no store saved; a later `tick` makes no request | 09B §4; MC-C1.1 |
| C1·C9(d) | same | `TestApp::with_pricing_no_shop()`, key set | state `no_shop`; zero requests | 09B §4 |
| C1·C9(e) | same | shop present, no key | state `no_key`; zero requests | 09B §4 |
| C1·C9(f) | same | the fake answers 401; in a second test it times out while the state was `ready` | `refused`; in the second test the state is still `ready` and the returned error is `Unreachable` | 09B §5.4; MC-C1.3 |
| C1·C10(a) | reopen the app | a saved store for `shop-a…`; the shop in settings is now `shop-b.myshopify.com` | state `store_unmatched`, `visible_stores` empty; `tick` makes no request | 09B §2.2; MC-C1.2 |
| C1·C10(b) | `key_set(Some(other key))` | state `refused`, the copy holds two conversations, cursor 40; the fake returns the same store id | state `ready`; the two conversations and cursor 40 are kept | MC-C1.3 |
| C1·C10(c) | `test_connection` | the copy holds two conversations and one `failed` outbox row; the matching store now has another id | the copy and the outbox are empty; `cursor` null; `first_sync_done` 0; the new store is saved | MC-C1.3 |

### Delegated choices
`ConversationStore` method names · whether `visible_stores` is JSON text or a joined string in the row (the port returns a list).

### Notes
- Hazard: `Settings` is `#[serde(default)]`; the new field needs `skip_serializing_if = "Option::is_none"` like `writer`, or every settings file gains a `null`.
- Hazard: `SettingsSetContext::prepare` copies the live `research` and `writer` slots from disk before comparing; do the same for `conversations`, or a save elsewhere writes back a stale slot.
- Rows C1·C10(b)/(c) seed the copy through the store port (U3 fills it for real).

### Unit test commands
`cargo test -p psm-testkit --test suite c1_conversations_connection::` · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 3 — Local copy, first connection, the pull, read marks, errors, notices

**Unit id:** `C1·U3` · **Rows:** C1·C11–C18 · **Changes app files:** yes

### Goal
`ConversationsService::tick` by MC-C1.4–10, `mark_read`, the `ConversationsChanged` event, the background thread.

### Read first
Master plan §5 **MC-C1** (every item) · book `09B` §5, §6, §10 · `EVENTS_AND_WEBHOOKS.md` ("Catch-up by pull", "Recommended handler") · `crates/core/src/events.rs` · `crates/core/src/mechanisms/backoff.rs` (the style of a backoff rule) · `crates/testkit/src/{clock.rs, test_app.rs}` (`ManualClock`, `TestApp.events`) · `crates/integrations/src/lib.rs` (the `storage_background` thread, `shutdown`) · `crates/core/src/services/storage/session_delete.rs` (`delete_session`, for C1·C18(d)).

### Files
New: `crates/core/src/mechanisms/conversation_rules.rs`, `crates/testkit/tests/suite/c1_conversations_sync.rs`. Edited: `crates/core/src/mechanisms/mod.rs`, `crates/core/src/{events.rs, ports/conversations.rs, services/conversations.rs}`, `crates/infra/src/sqlite/conversations.rs`, `crates/integrations/src/lib.rs` (the thread, its stop and join in `shutdown`), `apps/desktop/src-tauri/src/lib.rs` (the `TauriEvents` arm that forwards `conversations_changed {toasts, more}`), `crates/testkit/tests/suite/main.rs`.

### Core mechanisms (the only functions with unit tests)
`conversation_rules::check_backoff(failures) -> Duration` (MC-C1.7) · `conversation_rules::unread(message, read_mark)` and `has_news` (MC-C1.10) · `conversation_rules::entering_read_mark(conversation)` (MC-C1.4) · `conversation_rules::notice_lines(notices) -> (Vec<(conversation_id, text)>, more)` (master plan §4 Event).

### Ordered tasks
1. The four rules, with their unit tests.
2. Store transactions: apply (insert or replace by MC-C1.4) with the cursor, delete with the cursor, cursor only, first-connection finish, error record, completion.
3. `tick`: MC-C1.1 gate; first connection (MC-C1.5); one page of the pull (MC-C1.6); schedule and backoff (MC-C1.7); errors (MC-C1.8); completion and emission (MC-C1.9). Every provider call and its transaction run inside the lane.
4. `mark_read`; `AppEvent::ConversationsChanged` (also emitted, without notices, when `key_set` or `test_connection` changes the connection state); the `TauriEvents` arm, which forwards `conversations_changed {toasts, more}` built by `notice_lines` (the chime and the banner are C2's).
5. The `conversations_background` thread (250 ms; again at once while `more`; only when `run_startup`; stopped and joined in `shutdown`). Log lines `conversations.first_sync`, `conversations.sync`, `conversations.sync_failed`.
6. Tests for the rows.

### Criteria (8 criteria, 50 rows)
Boundary: `app.conversations.tick(now)` on `TestApp::with_pricing()` after a successful `key_set`, with `ManualClock` (outcome = the copy read through the store port, the `conversation_sync` row, the fake's request record, the collected `ConversationsChanged` events); the four rules (pure).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1·C11(a) | first `tick` | the fake holds: open case A, answered case B, a resolved case C (resolved 2 days ago, named by events), an operations case D; event log ends at id 37 | the copy holds A, B and C with their messages, not D; `cursor` 37; `first_sync_done` 1; `last_ok_at` = now | 09B §5.2; MC-C1.5 |
| C1·C11(b) | same | same | requests in order: `events?after=0&limit=500` pages until one is empty, then `cases?recipient=photographer&status=unresolved`, then one `GET` for C; no `GET` for A, B or D | MC-C1.5 |
| C1·C11(c) | same | same | exactly one `ConversationsChanged` with no notices, `chime` false, `banner` false | 09B §5.2 (silent), §9; MC-C1.5 |
| C1·C11(d) | same | same | `has_news` is true for A (open) and false for B (answered) and C (resolved) | 09B §5.2 (read marks) |
| C1·C11(e) | same | the log names a case deleted since (`case.updated deleted: true` is its last event) and a case whose `GET` answers 404 | neither is in the copy; no `GET` for the first; the first connection still finishes | MC-C1.5 |
| C1·C11(f) | two `tick`s | `fail_matching("/cases", 1, 500)` makes the `cases` list answer 500 at the first attempt | after the first: `cursor` null, `first_sync_done` 0, `last_error` "Could not reach Pricing"; after the second (clock +10 s): the same copy and cursor as an uninterrupted first connection, each conversation once | MC-C1.5; invariant MC-C1 |
| C1·C11(g) | first `tick` | 520 events in the log | the event requests are `after=0`, `after=<id of the 500th>`, `after=<last id>` (the empty page); `cursor` = the last id | 09B §5.2 (limit 500) |
| C1·C12(a) | `tick` after the first connection | the pricer asks a new question (events `case.created`, `message.created`) | the conversation is in the copy with its message; `cursor` = the second event's id; exactly one `GET` for that case | 09B §5.3; MC-C1.6 |
| C1·C12(b) | same | the pricer edits a message | the stored text is the new text and `edited_at` is set | 09B §5.3 |
| C1·C12(c) | same | the pricer deletes one of two messages | the copy holds the other message only | 09B §5.3 |
| C1·C12(d) | same | the pricer resolves case A; in a second test the product is activated | status `Resolved` with `resolved_by` `Asker`; in the second test `Activation` | 09B §8; MC-C1.6 |
| C1·C12(e) | same | the pricer deletes case A's last message (`case.updated deleted: true`) | A and its messages are gone; no `GET` was made for A; `cursor` = that event's id | 09B §5.3 |
| C1·C12(f) | same | an event names a case whose `GET` answers 404 | the case is removed from the copy; `cursor` advanced | 09B §5.3 |
| C1·C12(g) | same | a question to Operations (`recipient: operations`) | `cursor` advanced; no `GET`; the copy unchanged; no event emitted with a notice | 09B §3, §5.3 |
| C1·C12(h) | same | `raw_event` of type `inbox.updated` with no case | `cursor` advanced; no `GET`; the copy unchanged | MC-C1.6 |
| C1·C12(i) | same | the fake redelivers an already applied event (id ≤ cursor) in the next page, with the case changed meanwhile on the fake without a new event | no `GET`; the copy unchanged | 09B §5.3 (at-least-once) |
| C1·C12(j) | same | an answered case gets a new pricer message | the message list is the provider's; `read_mark` is unchanged, so `has_news` is true | MC-C1.4 (replace keeps the mark) |
| C1·C13(a) | two `tick`s | a page with events for case A then case B; the `GET` of B answers 500 once | after the first: A's change is in the copy, `cursor` = A's event id, B unchanged; after the second (clock +10 s): the copy and cursor equal an uninterrupted run | MC-C1.6; invariant MC-C1 |
| C1·C13(b) | `tick`, reopen the app, `tick` | events applied, then the app is dropped and opened on the same data folder | `cursor` and the copy are as before the reopen; the next request is `events?after=<that cursor>` | MC-C1.6 (cursor in the transaction) |
| C1·C13(c) | `tick`s | 3 events for one case in one page | one `GET` for the case; `cursor` = the third id | MC-C1.6 (later event reads it) |
| C1·C13(d) | `tick`s until `more` is false | 620 new events | the first `tick` answers `more`; the pages are asked without waiting 10 s; the check completes on the empty page | 09B §5.3 (repeat until empty); MC-C1.7 |
| C1·C14(a) | `tick` | just after `key_set` succeeded | a check runs at once | 09B §5.3 (at once) |
| C1·C14(b) | `tick` at +9 999 ms, then at +10 000 ms after a complete check | — | no request at +9 999; one `events` request at +10 000 | 09B §5.3 (every 10 seconds) |
| C1·C14(c) | reopen the app, `tick` | a connected data folder | a check runs at the first `tick` (launch) | 09B §5.3 |
| C1·C14(d) | `tick` | the clock is advanced by 1 hour in one step (wake) | a check runs at that `tick` | 09B §5.3 (on waking) |
| C1·C15(a) | `check_backoff` | 1 | 10 s | 09B §5.4; MC-C1.7 |
| C1·C15(b) | same | 2 | 20 s | same |
| C1·C15(c) | same | 3 | 40 s | same |
| C1·C15(d) | same | 4 | 80 s | same |
| C1·C15(e) | same | 5 | 160 s | same |
| C1·C15(f) | same | 6 | 300 s | same |
| C1·C15(g) | same | 7 and 40 | 300 s both | same |
| C1·C15(h) | `tick`s | the fake answers 500 twice, then succeeds | the second attempt is not made before +10 s, the third not before +20 s after the second; after the success the next check is at +10 s | 09B §5.4 |
| C1·C16(a) | `tick` | the fake answers 401 on `events` | state `refused`; `last_error` "Pricing refused the key: check Settings → Pricing"; the copy kept; one `ConversationsChanged` without notices; ticks over the next hour make no request | 09B §5.4; MC-C1.8 |
| C1·C16(b) | `key_set(Some(new key))`, `tick` | after (a), the fake accepts the new key | state `ready`; a check runs; `last_error` cleared | 09B §5.4 (resumes) |
| C1·C16(c) | `tick` | the store answers 404 on `events` | state `store_gone`; `last_error` "This key is no longer tied to shop-a.myshopify.com"; no further request | 09B §5.4 |
| C1·C16(d) | `tick` | a 500, then in a second test a time-out, then in a third a 200 that is not an event page | state `ready`; the copy and `last_ok_at` unchanged; `last_error` "Could not reach Pricing" (first two) and "Pricing answered in a way this version does not understand" (third) | 09B §5.4; MC-C1.8 |
| C1·C17(a) | `tick` | a new blocking question on SKU `BEYO-1880`, topic Condition; sound and banner on | one event with one notice `{kind: Question, sku: "BEYO-1880", topic: "Condition"}`, `chime` true, `banner` true | 09B §5.5, §9; CV-1 |
| C1·C17(b) | same | a pricer reply on an existing conversation | one notice of kind `Reply` | 09B §5.5 |
| C1·C17(c) | same | a `message.created` with `authorKind: service` on an existing conversation | the copy is updated; the emitted event has no notice, `chime` false | 09B §5.5 (own replies never notify) |
| C1·C17(d) | same | in one check: a new question and two more pricer messages on it, and two pricer replies on another conversation | two notices in the order of their first event: `Question` for the first, `Reply` for the second | 09B §5.5 (one per conversation) |
| C1·C17(e) | same | `sound` false, `banner` true, one new question | `chime` false, `banner` true | 09B §4, CV-8 |
| C1·C17(f) | same | the pricer resolves a conversation (no message) | one event with no notice, `chime` false, `banner` false | 09B §5.5 |
| C1·C17(g) | `notice_lines` | a `Question` for SKU `BEYO-1880`, topic Condition; a `Reply` for `BEYO-7`; a `Question` with no SKU, product title "Brass Mirror", topic Other | "New question · BEYO-1880 — Condition"; "New reply · BEYO-7"; "New question · Brass Mirror — Other"; `more` 0 | 09B §9 |
| C1·C17(h) | same | five notices; no notice | the first three lines in order and `more` 2; no line and `more` 0 | 09B §5.5 (at most three, then "and N more") |
| C1·C17(i) | reopen the app, `tick`s until the check completes | while the app was closed the pricer asked two new questions and resolved a third conversation | the copy holds the two new conversations and the third is `Resolved`; exactly one `ConversationsChanged` carries the two `Question` notices with `chime` true | CV-5 |
| C1·C18(a) | `unread` / `has_news` | pricer message at t=100: mark null → unread; mark 99 → unread; mark 100 → read; a message of kind `Me` at t=200 with mark null → read | as stated | 09B §6; MC-C1.10 |
| C1·C18(b) | `entering_read_mark` | open with messages at 100 and 200; answered with the same; resolved with none | null; 200; null (no message) | MC-C1.4 |
| C1·C18(c) | `mark_read` then `tick` | a conversation with news; then a new pricer message arrives | after `mark_read`: `has_news` false and one `ConversationsChanged` without notices; after the `tick`: `has_news` true | 09B §6 |
| C1·C18(d) | `delete_session` | a session whose item has SKU `BEYO-1880`, with a conversation for that SKU in the copy | the conversation and its messages are still in the copy; the fake recorded no request during the delete | CR3 |

### Delegated choices
How the service keeps the lane, `next_check_at`, `failures` and the notice marks in memory · the transaction method names.

### Notes
- Hazard: the first connection pages the events **before** it lists (MC-C1.5): a list taken first could miss a change made while paging.
- Hazard: `nextCursor` is unchanged on an empty page (openapi `EventPage.nextCursor`); never treat an unchanged cursor on an **empty** page as an error.
- Hazard: no table of this migration names a session; do not add a session id "for speed" (CR3 and the "Delete session" coupling of `docs/guide/04_change_impact.md`).
- Hazard: tests share one process: no real sleeping, no thread of their own; drive `tick` with `ManualClock`. `TestApp` runs without the background thread.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::conversation_rules` · `cargo test -p psm-testkit --test suite c1_conversations_sync::` · `cargo test -p psm-testkit --test suite c1_conversations_connection::` · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit --all-targets -- -D warnings` · `cargo check --workspace`

## Unit 4 — Commands and Settings → Pricing

**Unit id:** `C1·U4` · **Rows:** C1·C19–C21 · **Changes app files:** yes

### Goal
The four C1 commands, `services/conversations.ts`, `PricingSection.tsx` with its nav entry, the guide.

### Read first
Master plan §4 (Commands, Words, Frontend) · book `09B` §4 · `apps/desktop/src-tauri/src/commands/{writer.rs, research.rs (`research_secrets_set`, `research_secrets_state`)}`, `apps/desktop/src-tauri/src/lib.rs` (`specta_builder`, `export_bindings`, an inline command test module such as `r6_command_tests`) · `apps/desktop/src/features/settings/{SettingsNav.tsx, SettingsPage.tsx, ResearchSection.tsx (the key fields), WriterSection.tsx}` and `__tests__/{t1_settings_drawer.test.tsx, w4_writer_settings.test.tsx}` · `apps/desktop/src/services/writer.ts` · `docs/intention/evidence/design_handoff.md`.

### Files
New: `apps/desktop/src-tauri/src/commands/conversations.rs`, `apps/desktop/src/services/conversations.ts`, `apps/desktop/src/features/settings/PricingSection.tsx`, `apps/desktop/src/features/settings/__tests__/c1_pricing_settings.test.tsx`. Edited: `apps/desktop/src-tauri/src/{commands/mod.rs, lib.rs}`, `apps/desktop/src/services/{bindings.ts (generated), events.ts}`, `apps/desktop/src/features/settings/{SettingsNav.tsx, SettingsPage.tsx}`, `apps/desktop/src/features/settings/__tests__/t1_settings_drawer.test.tsx` (its section list), `docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md}`.

### Core mechanisms
None.

### Ordered tasks
1. `conversations_settings_get -> ConversationsSettingsDto {server, server_default, key_saved, sound, banner, connection: ConnectionDto {state, store_name, shop, visible_stores, last_ok_at, last_error, last_error_at}}`; `conversations_settings_save(patch)`; `conversations_key_set(key)` (empty = remove); `conversations_test`. Each runs in `spawn_blocking`; none returns the key. Bindings regenerated; wrappers; `onConversationsChanged` in `events.ts`.
2. `PricingSection`: nav entry "Pricing" after "Shopify" (always shown). **Server address** (prefilled with the effective server; Save). **API key**: a password field with Save when none is saved; "Saved" with **Replace** and **Remove** when one is. **Test connection**. The connection line, from `state`: `ready` → `Connected to <store name> (<shop>)`; `store_unmatched` with names → `This Pricing key isn't tied to <shop>. It can see: <names>.`; `store_unmatched` without names → `This Pricing key isn't tied to <shop>. Press Test connection.`; `no_shop` → `Connect Shopify first; conversations are matched to its products.`; `no_key` → nothing; `refused` → `Pricing refused the key: check Settings → Pricing`; `store_gone` → `This key is no longer tied to <shop>`. **Sound for new questions and replies** and **Mac banner when Session Manager is in the background** (switches, saved at once). **Last checked**: the time of `last_ok_at`, or `last_error` when it is newer. The section refreshes on `conversations_changed`. It is outside the Settings page's unsaved-changes guard (its controls save by themselves, like the research keys).
3. Guide: `01_product.md` (a short "Conversations with Pricing" paragraph and vocabulary), `02_architecture.md` (the service, the adapter, the thread, the four tables, the event, the settings slot), `03_rules_and_principles.md` (CR1–CR6 with where they bite), `04_change_impact.md` (rows: a Pricing reply shape → `types.rs`, the fixture, `FakePricing`; a `conversations` settings field; the `ConversationsChanged` payload).
4. Tests for the rows.

### Criteria (3 criteria, 15 rows)
Boundary: the command layer and the event bridge in `apps/desktop/src-tauri/src/lib.rs` (`c1_tests`, the wired app with `FakePricing`); `PricingSection` rendered in a UI test with `services/conversations.ts` mocked (outcome = what is shown and the service calls with their arguments); `TestApp` for C1·C21.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1·C19(a) | `conversations_settings_get` | a fresh data folder with a fake shop | `server` and `server_default` = the default address, `key_saved` false, `sound` true, `banner` true, `connection.state` `no_key` | 09B §4 |
| C1·C19(b) | `conversations_key_set("k-secret-9")` then `conversations_settings_get` | a matching store | `key_saved` true, `connection.state` `ready`, `store_name` and `shop` set; neither answer, serialised, contains `k-secret-9` | 09B §4; CR2 |
| C1·C19(c) | `conversations_key_set("")` | after (b) | `key_saved` false; `connection.state` `no_key` | 09B §4 (Remove) |
| C1·C19(d) | `conversations_test` | no store matches | `connection.state` `store_unmatched` with the visible names | 09B §4 |
| C1·C19(e) | `conversations_settings_save` | `{server: "http://x.example"}` | a `CommandError` with code `invalid_value` naming `server` | 09B §4 |
| C1·C19(f) | the `TauriEvents` bridge | `AppEvent::ConversationsChanged` with one `Question` notice for conversation `c1`, SKU `BEYO-1880`, topic Condition | the window receives `conversations_changed` with `toasts` = [{`conversation_id`: "c1", `text`: "New question · BEYO-1880 — Condition"}] and `more` 0 | master plan §4 Event |
| C1·C20(a) | `SettingsNav`, `PricingSection` | defaults, no key | a "Pricing" entry after "Shopify"; the server field holds the default address; the key field is empty with Save; both switches on | `[rule]` 09B §4 |
| C1·C20(b) | `PricingSection` | `key_saved` true | "Saved" with Replace and Remove; no input holds a key | `[rule]` 09B §4; CR2 |
| C1·C20(c) | same | type a key, Save | `conversationsKeySet` called once with the typed key; the field is cleared and shows "Saved" | `[rule]` 09B §4 |
| C1·C20(d) | same | state `ready`, store "Beyo", shop `shop-a.myshopify.com` | the line "Connected to Beyo (shop-a.myshopify.com)" | `[rule]` 09B §4 |
| C1·C20(e) | same | state `store_unmatched` with names One, Two; then state `no_shop` | "This Pricing key isn't tied to shop-a.myshopify.com. It can see: One, Two."; then "Connect Shopify first; conversations are matched to its products." | `[rule]` 09B §4 |
| C1·C20(f) | same | click Test connection; turn the sound switch off | `conversationsTest` called once and its answer shown; `conversationsSettingsSave` called with `{sound: false}` | `[rule]` 09B §4 |
| C1·C20(g) | same | `last_ok_at` set and no error; then `last_error` "Could not reach Pricing" newer than `last_ok_at` | "Last checked" with the time; then the error's words | `[rule]` 09B §4 |
| C1·C20(h) | same | state `refused` | "Pricing refused the key: check Settings → Pricing" | `[rule]` 09B §5.4; CV-6 |
| C1·C21(a) | `TestApp` | `FakePricing.hold()` blocks a `tick` running on a test thread; meanwhile `app` lists the sessions and reads the conversations `connection()` | both answer while the `tick` is still blocked; after `release()` the `tick` finishes | CR5 |

### Delegated choices
Layout inside the section (design handoff) · how "Last checked" words a time ("2 minutes ago" or a clock time).

### Notes
- Hazard: a `#[derive(Type)]` DTO must not carry the key; `ConversationsSettingsDto` has `key_saved: bool` only.
- Hazard (C1·C21): `connection()` must not take the lane; only provider calls and their transactions do.
- C1·C21(a) is the one row that uses a thread; it blocks on the fake's own latch, not on time.

### Unit test commands
`cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/settings` · `npm run typecheck` · `cargo test -p psm-testkit --test suite c1_conversations_sync::` · `cargo clippy -p psm-desktop --all-targets -- -D warnings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core --lib mechanisms::conversation_rules` · `cargo test -p psm-integrations --lib conversations::` · `cargo test -p psm-testkit --test suite c1_conversations_connection::` · `cargo test -p psm-testkit --test suite c1_conversations_sync::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/settings` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: no Pricing word, URL or `photographer` literal in `crates/core`; the cursor is written in the same transaction as the change it follows, at every site; no code path reaches the provider when MC-C1.1 is false; the key appears in no DTO, log call or error text; every fixture file matches the package's example or schema it names; nothing in the tree calls a real host. Notes go to `docs/implementation/backlog_conversations/C1_connection_and_sync.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
1. Open Settings → Pricing. The address is filled in. Paste the API key from Pricing (Settings → Service accounts) and press Save: the line reads "Connected to <your store> (<your shop>)".
2. Press Test connection: the same line. "Last checked" shows a time that moves every 10 seconds or so.
3. Turn Wi-Fi off for a minute: "Last checked" says Pricing could not be reached; every other screen works as before. Turn it on again: the time returns.
4. Nothing else changes yet: the icon and the conversations arrive with the next step.

## Review log
