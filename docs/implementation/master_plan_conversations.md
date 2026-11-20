# Conversations with Pricing — master plan (v1.1, 2026-11-20)

Short on purpose. It adds to `docs/implementation/master_plan.md` (roles, standing rules §3, registry §6, environment
§7) and wins over it on a name listed here. Process: `docs/process/flow.md` §10 (milestones of units), with the deltas
of §3 below. Planner: Claude (`pipeline-planner`), from book `09B_pricing_conversations.md` (RATIFIED 2026-11-20, A27).

## 1. Goal
Build book `09B_pricing_conversations.md`: the questions a pricer asks in Beyo Pricing reach the photographer in
Session Manager (a header icon with a count, a right-side panel, a Pricing-style conversation view), the photographer
answers there (text, reply-to, published photos, Mark answered), jumps to a conversation's session as a **visit**
without moving the capture folder and comes back, and hears a chime and sees a toast and a Mac banner when a question
or a pricer reply arrives. Semantics live in 09B. The API is owned by the Pricing integration package
(`~/Desktop/Developer/pricing-app/docs/integrations/session-manager/`, read-only, another repo): `API.md`,
`EVENTS_AND_WEBHOOKS.md`, `CHAT_UX.md`, `openapi-integration.json`. §5 below is the part 09B leaves to planning: the
exact rules of the three mechanisms that fail silently (MC-C1 sync, MC-C2 send, MC-C3 visit). Where 09B and this file
differ, 09B wins and this file is amended; where the package and this file differ on an API shape, the package wins.

## 2. Milestones (one runner phase each; order in `docs/implementation/phases_conversations.txt`)
| Id | Builds | Units | Criteria | Rows | Changes app files | Projection | depends_on | Useful on its own |
|---|---|---|---|---|---|---|---|---|
| `C1_connection_and_sync` | the conversations port and vocabulary in core, the Beyo Pricing adapter and the fake Pricing server, Settings → Pricing (key in the Keychain, store match, Test connection), the local copy, the first connection, the 10-second pull, read marks, error states | 4 | 21 | 111 | yes | **yes** | — | the Mac keeps an exact local copy of the photographer's conversations; Settings says whether it is connected; nothing shows them yet |
| `C2_header_panel_notifications` | the list, counts and badges behind commands; the header icon with its count and dot, the panel list with Open / Resolved, the sessions-list badges; toast, chime, Mac banner, mute | 3 | 19 | 66 | yes | no | C1 | the photographer sees which products have questions and is told when one arrives |
| `C3_conversation_and_answering` | the outbox and send with `clientMutationId`, retry, Mark answered, attachable photos; the conversation view (bubbles, quote, photos, reply line, edited); the composer and photo picker | 3 | 21 | 73 | yes | **yes** | C2 | the photographer reads a question and answers it in Session Manager |
| `C4_visit_and_quote_jump` | the visit (Open session without moving the capture folder), the bar, Back, Shoot here instead; the quote chip that opens the form at its field | 2 | 12 | 44 | yes | **yes** | C3 | the photographer checks the product a question is about and returns to the shoot |
Totals (computed from the plan files): 4 milestones, 12 units, 73 criteria, 294 rows (C1·C11(h) added by the C1 projection, 2026-11-20); every unit has at most 8 criteria.
Each milestone needs the one before approved (its `depends_on`), gate = the **Tracker** line of that plan file.
Start each with `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase <id>` (the runner resolves any id as
`docs/implementation/plans/<id>.md`); `run_phase.sh all` reads `phases.txt`, not this list.

## 3. Process deltas (flow §10 applies; these add to it)
- **No mechanism-inventory gate.** §5 stands in for it (orchestrator's instruction, as for the writer): one contract
  per silent-failure mechanism (MC-C1, MC-C2, MC-C3), each with its invariant. 09B has no measurement ledger of its
  own; trace cells cite 09B sections, its rules `CR1`–`CR6` (= 09B §10 rules 1–6, in that order), its scenarios
  `CV-1`–`CV-8`, the contracts of §5 with their item numbers, and `H1`–`H8`. `[rule]` only for UI rendering rows. The
  orchestrator may backfill a lettered ledger section in book `14` from MC-C1–MC-C3 later; no row changes id when it
  does.
- **Milestones of units** (flow §10): one implementer session works the units in order with a
  `CHECKPOINT (not approved):` commit after each, one review, one fix budget, one narrative per milestone. Reason for
  more than 8 criteria per milestone: as for research and the writer, a phase cycle per unit costs more than the work.
- **Implementer Claude Opus** (`PSM_IMPLEMENTER=claude`), reviewer Opus. **No plan sets `implementer:`**: flow.md log
  v1.3 (2026-11-08, "no new plan may carry `implementer: fable`") is later than the runner's 2026-11-04 note and is the
  process authority; C2–C4 are frontend-heavy and still run on Opus.
- **Projection** for `C1` (the sync contract and every Pricing shape, rule 17), `C3` (send idempotency) and `C4` (the
  capture folder must not move); not for `C2`.
- **Tests** (flow §4, §4A, §10): one behaviour test per row at the boundary it names; unit tests only for the plan's
  **Core mechanisms**. While working: the milestone's own stems and UI files. The full suite runs **once, at the end
  of each milestone** (all four change app files).
- **Rule 9, here:** no test, no session and no tool run ever calls Pricing or Shopify. Every test runs the real
  adapter over the fake Pricing server (`psm_testkit::fake_pricing::FakePricing`, §4). **Sessions never launch the
  built app** in C1–C4 (the Keychain is shared by every build of the app, so a launched build could read the owner's
  real key); the manual try-out of each plan is the owner's or the orchestrator's. The live check against the real
  server is the orchestrator's, after C4, with a test question the owner creates.
- **Rule 17, here:** every Pricing reply a test feeds the adapter is built from the package's schema
  (`openapi-integration.json` components) and the examples of `API.md` / `EVENTS_AND_WEBHOOKS.md`; the fixture files
  of §4 are those examples verbatim, and the fake serialises through the adapter's own wire types. The transport
  errors are the two shapes `ReqwestTransport` really produces (`crates/integrations/src/commerce/shopify/transport.rs`
  `map_error`: `Error::Timeout`, `Error::Provider("http: …")`) and a non-JSON body as `Json::String`.
- **Row ids** `C<n>·C<k>(<row>)`, numbered once per milestone across its units; tests named after their row
  (`c1_c7a_<slug>`). Reviewer notes: `docs/implementation/backlog_conversations/<milestone id>.md`.
- **Secrets.** The Pricing key is typed only by the owner, in the installed app. No key, real store id, shop domain or
  real conversation text in a test, fixture, log, handoff or commit; fixtures use the package's example values.
- **Guide.** A milestone that changes a flow, table, command, setting, event or rule updates `docs/guide/` in the same
  change (flow §7A); each plan carries it as a task. `docs/implementation/STATUS_conversations.md` is the
  orchestrator's; `STATUS.md` and `master_plan.md` §4 are not used or edited by these milestones.
- **Join:** after C4 and the live check the orchestrator opens the PR into `main`.

## 4. Names (registry; exact)
**Code layout.** Core: `crates/core/src/domain/conversation.rs` (vocabulary), `crates/core/src/ports/conversations.rs`
(`ConversationsProvider`, `ConversationStore`), `crates/core/src/services/conversations.rs` (`ConversationsService`),
`crates/core/src/mechanisms/conversation_rules.rs` (pure rules, inline unit tests). Adapter:
`crates/integrations/src/conversations/{mod.rs, beyo_pricing/{mod.rs, adapter.rs, transport.rs, types.rs,
fixtures/*.json}}`. Storage: `crates/infra/src/sqlite/conversations.rs` + migration
`crates/infra/src/sqlite/migrations/019_conversations.sql`. Chime: `crates/infra/src/chime.rs`. No name of Pricing,
"pricer", "photographer", "case", "SKU of Shopify" or a Pricing URL appears in `crates/core` (H7, 09B §3): core says
*conversation*, *message*, *provider*, *store*, *asker*, *me*.

**Vocabulary** (`psm_core::domain::conversation`, serde snake_case): `Conversation {id, store_id, product_id, sku:
Option<String>, product_title, topic, blocking, status, resolved_by: Option<ResolvedBy>, created_at, updated_at,
read_mark: Option<i64>, messages: Vec<Message>}` · `ConversationStatus {Open, Answered, Resolved}` · `ResolvedBy
{Asker, Activation}` · `Message {id, conversation_id, author: Author {kind: AuthorKind, name}, text, quote:
Option<Quote {field, field_label, text}>, reply_to: Option<ReplyRef {message_id, author_name, excerpt}>, attachments:
Vec<Attachment {media_id, thumb_url: Option<String>, url: Option<String>}>, created_at, edited_at: Option<i64>}` ·
`AuthorKind {Asker, Me, Other}` · `ProviderStore {id, name, shop_domain}` · `ProviderEvent {id: i64, kind: EventKind,
conversation_id: Option<String>, for_us: bool}` · `EventKind {ConversationCreated, ConversationUpdated {deleted: bool,
resolved_by: Option<ResolvedBy>}, MessageCreated {by_asker: bool}, MessageUpdated, MessageDeleted, Other}` ·
`EventPage {events, next_cursor: i64}` · `NewMessage {client_mutation_id, text, reply_to_message_id: Option<String>,
attachment_media_ids: Vec<String>}` · `OutboxMessage {client_mutation_id, conversation_id, text, reply_to:
Option<ReplyRef>, attachment_media_ids, state: OutboxState, failure: Option<String>, server_message_id:
Option<String>, created_at}` · `OutboxState {Sending, Sent, Failed}` · `ConnectionState {NoKey, NoShop,
StoreUnmatched, Ready, Refused, StoreGone}` · `Connection {state, store: Option<ProviderStore>, visible_stores:
Vec<String>, shop: Option<String>, last_ok_at: Option<i64>, last_error: Option<String>, last_error_at: Option<i64>}` ·
`ConversationNotice {conversation_id, sku: Option<String>, product_title, topic, kind: NoticeKind {Question, Reply}}`
· `ConversationSettings {server: Option<String>, sound: bool, banner: bool}` · `ProviderError {Unreachable(String),
BadReply(String), Refused, StoreGone, NotFound, Resolved, Rejected {code: String, message: String}}`. All times are
UTC milliseconds (`i64`).

**Ports** (`psm_core::ports::conversations`). `ConversationsProvider: Send + Sync` with `stores()`,
`unresolved(store_id) -> Vec<Conversation>`, `conversation(store_id, id) -> Option<Conversation>` (`None` = the
provider no longer has it, or it is not addressed to this app), `events_after(store_id, cursor, limit) -> EventPage`,
`post_message(store_id, conversation_id, &NewMessage) -> Message`, `mark_answered(store_id, conversation_id) ->
Conversation`, each `-> std::result::Result<_, ProviderError>`; plus, from C1, `describe(&self, error:
&ProviderError, shop: Option<&str>) -> String` (pure, no request: the error's sentence of **Words**, `shop` filling
the `StoreGone` sentence; the only way core obtains a sentence, projection C1 L1(a)); C4 adds `attribute_key(&self,
quote_field: &str) -> Option<String>` (pure). `ConversationStore` is implemented for `SqliteStore`; its methods are
the transactions §5 names (each "one transaction" of §5 is one method). `ConversationSettingsSlot: Send + Sync`
(projection C1 L1(b)): `read(&self) -> Option<Json>` (the `conversations` slot of `settings.json`) and `write(&self,
ConversationSettings) -> psm_core::Result<()>`; implemented in `psm-integrations` over `psm_infra::Settings` (see
**Settings**).

**Storage** (migration `019_conversations.sql`; no table names a session, an item or a photo, so "Delete session" and
`relocate_raw` are untouched, CR3):
`conversation_sync(id INTEGER PRIMARY KEY CHECK (id = 1), state TEXT NOT NULL DEFAULT 'no_key', store_id TEXT,
store_name TEXT, shop_domain TEXT, visible_stores TEXT NOT NULL DEFAULT '[]', cursor INTEGER, first_sync_done INTEGER
NOT NULL DEFAULT 0, last_ok_at INTEGER, last_error TEXT, last_error_at INTEGER)` ·
`conversations(id TEXT PRIMARY KEY, store_id TEXT NOT NULL, product_id TEXT NOT NULL, sku TEXT, product_title TEXT NOT
NULL, topic TEXT NOT NULL, blocking INTEGER NOT NULL, status TEXT NOT NULL, resolved_by TEXT, created_at INTEGER NOT
NULL, updated_at INTEGER NOT NULL, last_activity_at INTEGER NOT NULL, read_mark INTEGER)` with index `(sku)` ·
`conversation_messages(id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL, position INTEGER NOT NULL, author_kind TEXT
NOT NULL, author_name TEXT NOT NULL, text TEXT NOT NULL, quote TEXT, reply_to TEXT, attachments TEXT NOT NULL DEFAULT
'[]', created_at INTEGER NOT NULL, edited_at INTEGER)` with index `(conversation_id, position)` (`quote`, `reply_to`,
`attachments` are JSON) ·
`conversation_outbox(client_mutation_id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL, text TEXT NOT NULL, reply_to
TEXT, attachment_media_ids TEXT NOT NULL DEFAULT '[]', state TEXT NOT NULL, failure TEXT, server_message_id TEXT,
created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL)` with index `(conversation_id, created_at)`.

**Secret.** Keychain account `psm_core::domain::conversation::KEY_ACCOUNT = "conversations-api-key"` through the
existing `SecretStore` port (`KeychainSecrets`, the app's one Keychain service). Registered with
`psm_infra::logging::add_secret` when it is set and when the service first reads it: core cannot call `psm_infra`, so
`App::build` hands the service a redaction-registering `SecretStore` decorator (`psm-integrations`) around the app's
secret store, which calls `add_secret` with the value on every `set` and on the first `get` that returns a value
(projection C1 L1(c)).

**Settings.** A new opaque slot `conversations` in `settings.json` (`psm_infra::Settings.conversations: Option<Json>`),
shape `ConversationSettings`, read tolerantly by `ConversationSettings::from_slot` and live at every use; accepted by
`SettingsService::set` (the key list in `crates/core/src/services/settings.rs`); nulled in the rebuild comparison of
`crates/integrations/src/lib.rs` beside `writer` (a save never rebuilds the App). Defaults: `server` none (= the
adapter's `beyo_pricing::DEFAULT_SERVER = "https://pricing.beyoworkaroundtheclock.com"`), `sound` true, `banner` true.
`server` when set: trimmed, one trailing `/` removed, must start with `https://`, have a host, and no path, query,
fragment or user info; anything else is refused naming `server`. The connected shop is
`settings.integration.shop_domain` (09A), handed to the service at build as `shop: Option<String>` (an integration
change already rebuilds the App).
**Where the rules live (projection C1 L1(b), L6, L25).** `beyo_pricing::checked_server` and `DEFAULT_SERVER` are in the
adapter crate; core never sees the URL. The `ConversationSettingsSlot` implementation (`psm-integrations`) does the
normalisation: `write` refuses a `server` that fails `checked_server` (`invalid_value` naming `server`, nothing
written, `settings.json` bytes unchanged), stores a `server` equal to `DEFAULT_SERVER` as none, and writes the full
normalised `ConversationSettings` object into the slot (never a partial merge on disk); `read` returns the slot with a
stored `server` that fails `checked_server` replaced by none, so `ConversationSettings::from_slot` (core, tolerant:
a missing or ill-typed field takes its default) never yields an invalid address. The service's `settings_save(patch)`
reads the slot, overlays the fields given on `from_slot` of it and calls `write`. The general path is guarded the same
way: `SettingsSetContext::prepare` validates a `conversations` patch with `checked_server` before `SettingsService::set`
(as it validates `research`/`writer`), and `SettingsService::set` merges `conversations` fields like `research`/
`writer` (the fields given replace those stored, the others are kept). The adapter's live `server` closure =
`checked_server(slot.server)` or `DEFAULT_SERVER`.

**Service.** `ConversationsService` (`App.conversations: Arc<ConversationsService>`, also in `AppHandles`; always
present, H6: it is `NoShop`/`NoKey` and silent without its prerequisites). Methods: C1 `tick(now_ms) ->
Result<Tick {more: bool}>`, `connection()`, `test_connection()`, `key_set(Option<&str>)`, `settings()`,
`settings_save(patch)`, `mark_read(conversation_id)` · C2 `list(scope, tab, now_ms)`, `counts(session_id)`,
`session_badges()` · C3 `view(conversation_id)`, `reply(NewMessage for a conversation)`, `retry(client_mutation_id)`,
`discard(client_mutation_id)`, `mark_answered(conversation_id)`, `attachable(conversation_id)`. A background thread
`conversations_background` in `App::build` (only when `run_startup`, like `storage_background`) calls `tick` every
250 ms and again at once while `more`; tests call `tick` themselves with `ManualClock`. Test seam:
`psm_integrations::with_test_conversations(paths, transport)` (keyed by the data folder, like `with_test_research`);
`TestApp::with_pricing()` installs `FakePricing` and a fake shop.
**Seams and lifetime (projection C1 L1, L4, L5, L12).**
- Constructor: `ConversationsService::new(store: Arc<dyn ConversationStore>, provider: Arc<dyn
  ConversationsProvider>, secrets: Arc<dyn SecretStore>` (the redaction-registering decorator of **Secret**)`, slot:
  Arc<dyn ConversationSettingsSlot>, shop: Option<String>, clock: Arc<dyn Clock>, events, lane)`. Sentences come only
  from `provider.describe(&error, shop)`; settings only through `slot`.
- Return types: `test_connection() -> Result<Connection, ProviderError>` and `key_set(Option<&str>) ->
  Result<Connection, ProviderError>` (`Ok` = the connection after MC-C1.3, also for `Refused`, which is a state;
  `Err` = `Unreachable`/`BadReply` of `stores()`, state unchanged). Other storage errors stay `psm_core::Error` as
  today (the implementer chooses how the two are combined, e.g. a small enum or `ProviderError` from the core error).
- The key: read from the Keychain **once per App build** (at the first `tick` or `connection()`, on a worker thread,
  never inside `App::build`) and on `key_set`, then held in memory; MC-C1.1 and the adapter's `key` closure read the
  in-memory value. No 250 ms Keychain read, no repeated access prompt (09B §10 rule 5).
- The lane: one process-wide mutex, created once per app process beside `Lanes` (`open_with_adapter_bind`) and
  carried through every App rebuild like `Lanes` (P48 L14), so an old and a new service never interleave requests
  and applies; the cursor a request uses is read from `S` inside the lane. `shutdown` sets the stop flag and joins
  `conversations_background` with `BACKGROUND_JOIN_BOUND` (like the research thread): it never waits for a request in
  flight, and a thread still inside one leaves after its apply.

**Adapter.** `BeyoPricingAdapter` over `PricingHttp` (`transport.rs`: `fn send(&self, PricingRequest {method, url,
bearer, body: Option<Json>}) -> psm_core::Result<HttpReply>`; live impl `ReqwestPricing`, connect timeout 10 s, total
20 s, the key only in the `Authorization: Bearer` header, errors without URL). Paths under
`{server}/integration/v1`. `types.rs` holds the wire types (camelCase, `Serialize + Deserialize`, unknown fields
ignored). Fixtures (`fixtures/`, the package's examples): `stores.json`, `cases_list.json`, `case.json`,
`message_created.json`, `events_page.json`, `events_other_types.json`, `error_envelope.json`.

**Commands** (`apps/desktop/src-tauri/src/commands/conversations.rs`, registered in `specta_builder()`; wrapper
`apps/desktop/src/services/conversations.ts`): C1 `conversations_settings_get`, `conversations_settings_save(patch)`,
`conversations_key_set(key)` (empty = remove), `conversations_test` · C2 `conversations_list(scope, tab)`,
`conversations_counts(session_id)`, `conversations_session_badges` · C3 `conversation_get(id)`,
`conversation_mark_read(id)`, `conversation_reply(input)`, `conversation_retry(client_mutation_id)`,
`conversation_discard(client_mutation_id)`, `conversation_mark_answered(id)`, `conversation_attachable(id)`.
Error codes (`CommandError.code`): `conversation_resolved`, `conversations_off` (`data.reason` = the
`ConnectionState`), `invalid_value` (existing), `not_found` (existing).

**Event.** `AppEvent::ConversationsChanged {notices: Vec<ConversationNotice>, chime: bool, banner: bool}` → window
event `conversations_changed {toasts: [{conversation_id, text}], more}` (listener `onConversationsChanged` in
`apps/desktop/src/services/events.ts`), built by `conversation_rules::notice_lines(notices) -> (lines, more)`: at most
three lines, in order, `New question · <SKU> — <topic>` for `Question` and `New reply · <SKU>` for `Reply` (the
product title in place of a missing SKU), `more` = the number left out. The window's toasts and the Mac banners use
these same lines.
Emitted when a check completes or stops on an error, when the outbox or a read mark changes, and when the connection
state changes.

**Words** (exact; shown by Settings and the panel): `Connected to <store name> (<shop>)` · `This Pricing key isn't
tied to <shop>. It can see: <names joined by ", ">.` · `This Pricing key isn't tied to <shop>. Press Test connection.`
(a saved store whose domain is no longer the shop's) · `Connect Shopify first; conversations are matched to its
products.` · `Connect Pricing in Settings → Pricing` · `Pricing refused the key: check Settings → Pricing` · `This key
is no longer tied to <shop>` · `Could not reach Pricing` · `Pricing answered in a way this version does not
understand` · `Couldn't send` · `Pricing did not accept this reply: <message>` · `This conversation no longer exists`
· `Pricing has already resolved this` · `Not on this Mac` · `Pricing is waiting` · `Publish first to attach photos` ·
`Replying re-opens it as answered` · `Resolved by the pricer` · `Resolved when the product was activated` ·
`Resolved`. The words that name Pricing are produced outside core: the adapter gives each `ProviderError` its
sentence (exposed to core only through the pure port method `describe`), the frontend holds the rest.

**Log lines.** `conversations.connection state= store=<matched: yes|no>` · `conversations.first_sync conversations=
cursor= truncated=` · `conversations.sync cursor= events= applied= notices=` · `conversations.sync_failed kind=
failures= next_s=` · `conversations.send outcome= conversation=`. Never a key, a message text or a URL with a query.

**Frontend.** `apps/desktop/src/features/conversations/`: C2 `ConversationsButton.tsx`, `ConversationsPanel.tsx`,
`ConversationRow.tsx`, `ConversationToasts.tsx`, `useConversations.ts` (panel open, selected conversation, scope; one
instance in `App.tsx`), `SessionBadge.tsx` · C3 `ConversationView.tsx`, `MessageBubble.tsx`, `QuoteBlock.tsx`,
`Composer.tsx`, `PhotoPicker.tsx` · C4 `visit.ts` (pure), `VisitBar.tsx`. C1 `features/settings/PricingSection.tsx` and
the nav entry `pricing` ("Pricing", after "Shopify") in `SettingsNav.tsx`.

**Tests.** Suite stems `crates/testkit/tests/suite/{c1_conversations_connection.rs, c1_conversations_sync.rs,
c2_conversations_lists.rs, c3_conversations_answer.rs, c4_conversations_visit.rs}` (each with its `mod` line in
`main.rs`); adapter tests inline in `beyo_pricing/adapter.rs`; desktop command and bridge tests in
`apps/desktop/src-tauri/src/lib.rs` (`c1_tests`, `c2_tests`, `c3_tests`); UI
`features/<f>/__tests__/c<n>_<slug>.test.tsx`.

## 5. Contracts (stand in for the mechanism inventory; plans implement them verbatim)

Notation: `S` = the `conversation_sync` row; `copy` = `conversations` + `conversation_messages`; `shop` = the
connected shop's domain; `≈` = equal after trimming, ignoring ASCII case; "lane" = one process-wide mutex (carried
through App rebuilds, §4 Service) held for one provider request **and** the transaction that applies its answer
(requests never interleave with another request's apply, not even across an old and a new App during a rebuild).
"State" below = the connection state reported by `connection()`; a state marked *(memory)* is never written to `S`.

### MC-C1 — Connection, the local copy and the pull (09B §4, §5, §6)
1. **Calls allowed** ⇔ `S.state = ready` ∧ a key is held in memory (read from the Keychain once per build, §4
   Service) ∧ `S.shop_domain ≈ shop` ∧ the live effective server = the connected server (rule 3). Otherwise the
   service makes no provider request of any kind (`tick`, `reply`, `mark_answered`); only `test_connection` and
   `key_set` may call `stores()`.
2. **State at build** (no request): no `shop` → `no_shop` *(memory)*; else no key → `no_key` *(memory)*; else no saved
   store and `S.state = store_unmatched` (written by rule 3) → as stored, with its `visible_stores`; else no saved
   store or `S.shop_domain ≉ shop` → `store_unmatched` *(memory)*, `visible_stores` reported empty (`S.visible_stores`
   is not written); else `S.state` as stored (`ready`, `refused` or `store_gone`). A build never connects by itself: a data folder that was never tested makes no call even when the
   Keychain holds a key. **Only connect and check outcomes are written to `S.state`** (rule 3: `ready`, `refused`,
   `store_unmatched` with `visible_stores`; rule 8: `refused`, `store_gone`); the derived states above are
   re-evaluated from the in-memory key, `shop` and `S` whenever the state is read and are never persisted, so one
   launch with an unreadable Keychain or a briefly invalid integration block leaves nothing behind (projection C1 L3).
3. **Connect** (`test_connection`; `key_set(Some(key))` trims the key (empty after trimming = `key_set(None)`), stores
   it, registers it for redaction, holds it in memory, then connects; `key_set(None)` deletes it and drops it from
   memory, so the state is `no_key` by rule 2): no `shop` → `no_shop`; no key → `no_key`; else `stores()`:
   `Refused` → `refused`; `Unreachable`/`BadReply` → the state is unchanged and the error is returned (`Err`, §4
   Service; its sentence through `describe`);
   else the first store in reply order with `shop_domain ≈ shop`: none → `store_unmatched` with `visible_stores` = the
   names in reply order, saved store cleared; found → if its `id ≠ S.store_id`: delete the copy and the outbox, `cursor`
   null, `first_sync_done` 0; then save the store, `state = ready`, clear `last_error`, and the next `tick` checks at
   once. One transaction per outcome. The copy is kept in every other state (it is only hidden by the UI when
   `no_key`, `no_shop` or `store_unmatched`).
   **Server change** (projection C1 L7): the service holds in memory the effective server of the last successful
   connect (initialised at build from the slot). When the live effective server differs from it (a `settings_save`,
   or a `conversations` patch through `settings_set`; compared in `settings_save` and before every provider request),
   the state is `store_unmatched` *(memory)* with `visible_stores` empty (the line "Press Test connection"), no call is
   made, and the old store id and cursor are never sent to the new server; the next `test_connection` or
   `key_set(Some)` connects to the new server by this rule (a different store id empties the copy as usual).
   `settings_save` emits `ConversationsChanged` without notices when it changes the state.
4. **Insert rule (read mark of a conversation entering the copy):** status `open` → `read_mark` null (unread);
   otherwise `read_mark` = the newest message's `created_at` (read). **Replace rule** (already in the copy): every
   field and the whole message list are replaced by the provider's answer (messages in the answer's order, `position`
   0…n−1); `read_mark` is kept; `last_activity_at` = the newest message's `created_at`, or `updated_at` when it has no
   message. A wire message with `deleted: true` or `text: null` is not stored.
5. **First connection** (`first_sync_done = 0`), silent. The same calls as 09B §5.2, ordered so that no change can
   fall between the list and the saved cursor:
   (a) page `events_after(store, after, 500)` from `after = 0` until a page has no events, setting `after :=
   next_cursor` after each page (`next_cursor ≤ after` with events present → `BadReply`); remember, in order, every
   `conversation_id` of an event with `for_us`, except that an id whose **last** `for_us` event in the paged log is
   `ConversationUpdated {deleted: true}` is forgotten (its earlier events do not bring it back; projection C1 L2);
   `E` := the last `after`;
   (b) `unresolved(store)`; apply each by rule 4;
   (c) for each remembered id not in the copy, once: `conversation(store, id)`; `Some` → rule 4; `None` → skipped;
   (d) one transaction: `cursor := E`, `first_sync_done := 1`, `last_ok_at := now`, `last_error` cleared. The first
   connection is a **complete check** for rules 7 and 9: `failures := 0`, `next_check_at = now + 10 000` (projection
   C1 L8).
   Then `ConversationsChanged {notices: [], chime: false, banner: false}`. An error at any step leaves `cursor` and
   `first_sync_done` untouched; the next attempt starts again at (a) and ends with the same copy (rule 4 is
   idempotent). `truncated=true` is logged when (b) returned 200 conversations (the package's maximum).
6. **Pull** (`first_sync_done = 1`), one page per `tick`: `page = events_after(store, S.cursor, 500)`.
   No events → the **check is complete** (rule 9). Otherwise for each event `e` in order:
   - `e.id ≤ S.cursor` → skipped (at-least-once);
   - `¬e.for_us` or no `conversation_id` (this covers event types outside the five) → transaction: `cursor := e.id`;
   - `ConversationUpdated {deleted: true}` → transaction: delete the conversation, its messages and its outbox rows not
     `sent`; `cursor := e.id`;
   - a later event of this page names the same conversation → transaction: `cursor := e.id` (the later one reads it);
   - otherwise `conversation(store, id)` in the lane: `Some(c)` → transaction: rule 4 for `c`; `resolved_by` :=
     when `c.status = resolved`: the `resolved_by` of the newest `ConversationUpdated` event of this conversation in
     this page that carries one, else the stored value, else `Asker` when this page holds a `ConversationUpdated`
     **of this conversation** (projection C1 L11), else null; when `c.status ≠ resolved`: null; `cursor := e.id`. `None` → transaction: delete
     as above; `cursor := e.id`. Error → the page stops here; `cursor` is the last applied event's id (rule 8).
   After the last event: `cursor := max(cursor, page.next_cursor)`; `Tick {more: true}` (the next page is asked at the
   next `tick`, at once).
   **Notices** (kept in memory for the running check): `ConversationCreated` with `for_us` marks its conversation
   `Question`; `MessageCreated {by_asker: true}` with `for_us` marks it `Reply` unless it is `Question`. Nothing else
   marks. A conversation deleted later in the check is unmarked.
7. **Schedule.** `next_check_at` (memory, wall clock): at build, after a successful connect and after `key_set` = now;
   after a complete check = now + 10 000 ms; after an error = now + `check_backoff(failures)`. A `tick` checks iff calls
   are allowed ∧ (`more` ∨ now ≥ `next_check_at`). A wall clock that jumped (wake from sleep) is simply past
   `next_check_at`. `check_backoff(n)` (`mechanisms/conversation_rules.rs`, n = consecutive failures ≥ 1) =
   `min(10 s × 2^(n−1), 300 s)`: 10, 20, 40, 80, 160, 300, 300 …; a success sets `failures := 0`.
8. **Errors of a check or of the first connection:** `Unreachable` or `BadReply` → `failures += 1`, `last_error` = its
   sentence, `last_error_at = now`, state stays `ready`, the copy is kept; `Refused` → `state = refused`; `StoreGone`
   (the store answers 404 on `unresolved` or `events_after`) → `state = store_gone`; `Refused` and `StoreGone` also set
   `last_error` = their sentence and `last_error_at = now` (projection C1 L9). Every sentence is
   `provider.describe(&error, shop)` (the `StoreGone` one names `shop`). Each is one transaction, followed by rule 9's
   emission. `refused` and `store_gone` end only by rule 3.
9. **Completion and emission.** When a check completes: `last_ok_at := now`, `last_error` cleared, `failures := 0`.
   On completion and on an error the service emits one `ConversationsChanged` with the marked conversations still in
   the copy as notices (ordered by the id of the event that first marked them; `sku`, `product_title`, `topic` from
   the copy), `chime = sound ∧ notices ≠ ∅`, `banner = banner ∧ notices ≠ ∅` (settings read at that moment), and
   clears the marks. A check that applied nothing and had no state change emits nothing; "applied" = changed the
   copy (a transaction that only moves `cursor`, e.g. an Operations event or `inbox.updated`, applies nothing;
   projection C1 L10).
10. **Unread.** A message is unread ⇔ `author.kind = Asker` ∧ (`read_mark` is null ∨ `created_at > read_mark`).
    `has_news(c)` ⇔ some message of `c` is unread. `mark_read(c)` sets `read_mark` to the newest message's
    `created_at` (no change → no event).
**Invariant MC-C1.** After every transaction, `S.cursor` is the id of the last event whose effect is in the copy, and
every event with a smaller or equal id needs no further work; for any interruption point and any redelivery, the copy
after the next complete check equals the copy of an uninterrupted run with the same provider answers; no conversation
with `for_us = false` is ever stored; no request is made unless rule 1 holds (or rule 3 for `stores()`); no notice is
produced by the first connection or by a message of kind `Me`.

### MC-C2 — Sending without duplicates (09B §8, CR1, CR6; CHAT_UX §5)
1. **Accept** (`reply`, no request, one transaction): `client_mutation_id` 1–80 characters (the frontend makes a UUID
   v4 when send is pressed); `text` trimmed, 1–4000 Unicode scalar values (the package's `minLength`/`maxLength`);
   at most 10 `attachment_media_ids`, none repeated; the conversation is in the copy; `reply_to_message_id`, when set,
   is a message of that conversation (its `ReplyRef` is stored for display). A row with that `client_mutation_id`
   already exists → nothing is written and that row is returned (idempotent). Otherwise insert `state = sending` — or
   `state = failed` with the state's sentence when calls are not allowed (MC-C1.1). Refusals are `invalid_value`
   naming the field, or `not_found`.
2. **Flush** (start of every `tick` when calls are allowed): the `sending` rows oldest first; for each, in the lane,
   `post_message` with exactly the stored `client_mutation_id`, text, reply target and media ids, then one
   transaction:
   - `Ok(m)` → row `sent`, `server_message_id = m.id`; `m` appended to the conversation's messages unless a message
     with `m.id` is already there; conversation `status = answered`, `resolved_by` null, `last_activity_at =
     m.created_at`;
   - `Unreachable`/`BadReply` → row `failed`, failure `Couldn't send`; every other `sending` row of this flush becomes
     `failed` with the same failure **without a request**;
   - `Rejected {message}` → `failed`, `Pricing did not accept this reply: <message>`;
   - `NotFound` → `failed`, `This conversation no longer exists`;
   - `Refused` → `failed` with the refused sentence, and MC-C1.8 (`state = refused`); `StoreGone` likewise.
   A `failed` row is never sent by a `tick`. A `sending` row found at launch is sent by the first `tick` (same id).
3. **Retry** (`retry(id)`): a `failed` row → `sending`, failure cleared (or `failed` again at once with the state's
   sentence when calls are not allowed); any other state → unchanged. **Discard** (`discard(id)`): a `failed` row is
   deleted (local only, no request); any other state → refused `invalid_value`.
4. **Echo.** The provider's event for the app's own message is a `MessageCreated {by_asker: false}`: MC-C1.6 re-reads
   the conversation and rule 4 replaces its messages; the message is identified by its id, so it exists once, and it
   marks nothing (no notice).
5. **Lost answer.** When the provider stored the message but the answer never arrived, the row is `failed` and the
   re-read conversation already shows the message; the row keeps showing `Couldn't send · Retry` until the
   photographer retries (the provider answers with the same message, its id is already in the copy, the row becomes
   `sent`) or discards it. Never an automatic resend of a `failed` row.
6. **View.** `view(c)` = the copy's messages in order, then the outbox rows of `c` in states `sending` and `failed`,
   oldest first; `sent` rows are not shown (their message is in the copy).
7. **Mark answered** (`mark_answered(c)`, in the lane): calls not allowed → `conversations_off`; `mark_answered` →
   `Ok(conv)` → rule MC-C1.4 replace; `Resolved` → `conversation(store, c)` is read and applied, and the error
   `conversation_resolved` is returned; other errors → the copy is unchanged and the error's sentence is returned
   (`Refused`/`StoreGone` also set the state).
8. **Nothing else is ever written to the provider** (CR1): the port has two writing methods and the adapter issues
   only `POST …/messages` and `PATCH … {"status":"answered"}`.
**Invariant MC-C2.** Every `post_message` the app issues for one press of send carries the same `client_mutation_id`,
whether it is the first attempt, a retry, or the attempt after a restart; no `failed` row is sent without a retry; a
message id appears at most once in a conversation's messages; no outbox row is in state `sent` without a
`server_message_id`.

### MC-C3 — The visit and the way back (09B §7, CR4)
The frontend holds route and visit in **one** state, changed only by the pure function
`visitStep(state, action)` (`features/conversations/visit.ts`); `App.tsx` gives `useCaptureTarget` the derived
`captureTarget(state)` instead of the route's session id.
1. **State.** `{route: Route, visit: Visit | null}`; `Route` = `{kind: 'sessions' | 'session' | 'settings' | 'health',
   sessionId?}`; `Visit = {origin: Route, capture: number | undefined, visited: number}` (`capture` = the session the
   capture folder points at when the first jump was made: the origin's session, or `undefined` = Unassigned when the
   origin is not a session).
2. **captureTarget** = `visit.capture` while `visit ≠ null`, else the route's session id (`undefined` off a session).
   The existing effect of `useCaptureTarget` (activate on a session other than Unassigned, deactivate when leaving one)
   is unchanged and keyed on this value, so it issues a call only when the value changes.
3. **Actions.**
   - `open-session(B)` (the panel's **Open session**): the route is already session `B` → unchanged; no visit →
     `visit = {origin: route, capture: route's session id, visited: B}`, `route = session B`; a visit exists and `B =
     visit.capture` (the session being shot, which is then the origin) → as `back`; a visit exists otherwise →
     `visit.visited = B`, origin and capture kept, `route = session B`.
   - `back` → `route = visit.origin`, `visit = null` (no visit → unchanged).
   - `shoot-here` → `visit = null` (route unchanged, so captureTarget becomes the visited session: one activation; no
     visit → unchanged).
   - `route-changed(r)` (a `hashchange` not caused by the actions above: the Sessions list, the breadcrumb, review
     navigation, the notifications drawer, "Arranged — next", Back keys): `r` is session `visit.visited` → unchanged;
     anything else → `route = r`, `visit = null`.
4. **One step, one render.** Each action produces its new route and visit together; the hash is written from the new
   state after the step (a `hashchange` that only confirms the state's route is ignored). There is no render in which
   the route is a visited session while `visit` is null, or the reverse.
5. **The bar** is shown iff `visit ≠ null`: `Visiting <SKU of visited>: new photos still go to <SKU of the session
   the backend reports active, or "Unassigned">.` with `← Back to <origin label>` (the origin session's SKU, or
   `Sessions`, `Settings`, `Health`) and `Shoot here instead`. The capture session's SKU is read from `activeSession()`
   (the backend's truth), not from `visit.capture`.
6. **The panel** (open state, selected conversation, scope) lives in `App.tsx` state outside the route, so it stays
   open on the same conversation through `open-session` and `back`.
**Invariant MC-C3.** From `open-session` until `shoot-here` or a `route-changed` that ends the visit, the frontend
calls neither `session_activate` nor `session_deactivate`; `back` calls neither; `shoot-here` calls
`session_activate(visited)` exactly once; with no visit, the calls are those of today for the same navigation.

## 6. Delegated choices (decided here; each is one sentence in a narrative's "Choices made for you")
- **Pull only.** No webhook is registered; the key needs `cases:read`, `cases:write`, `events:read` only.
- **Saving a key tests the connection at once;** Test connection repeats it. A Mac that changed its Shopify shop must
  press Test connection again before any call is made.
- **A changed server address** also asks for Test connection before any call is made (the old store and position are
  never sent to the new address; projection C1 L7).
- **A different Pricing store empties the local copy** (and unsent replies); the same store keeps it.
- **One request at a time** to Pricing (the lane), 20 s timeout; a reply waits at most for the request in flight.
- **Failed replies are never resent by themselves,** and a failed reply can be **removed** as well as retried (09B
  names Retry only; a reply Pricing refuses for good would otherwise stay forever).
- **When a reply was delivered but its confirmation was lost,** the conversation shows the delivered message and the
  "Couldn't send · Retry" line until Retry confirms it; nothing is sent twice.
- **The chime and the banner are made by the app itself, not by the page:** the chime is a short tone the app
  generates (no sound file in the repo) and plays with macOS's own player, so it also sounds right after launch; the
  banner goes through Tauri's notification plugin.
- **A banner click brings Session Manager forward** (macOS does that); the plugin does not tell the app which banner
  was clicked, so the toast with **Open** stays on screen until the window is focused again and for 10 s after.
- **Exact SKU match** (case-sensitive, as 09B says "exact"); a conversation whose product has no SKU is "Not on this
  Mac".
- **"Resolved by the pricer / when the product was activated"** is known only when this Mac saw the resolving event;
  a conversation that was already resolved at the first connection says "Resolved".
- **The Resolved tab** uses the conversation's last update as its resolution time (the API has no other).
- **Both outputs of a split photo are offered** as two attachable photos; photos are offered in gallery order, then
  condition order.
- **The icon is on every page** (Sessions, session, Settings, Health); the panel is mounted once in `App.tsx`.
- **No session Trash exists today** ("Delete session" removes the session), so 09B §3's "not in the Trash" holds
  structurally; trigger to make it a real rule: a session Trash landing.
- **First connection limit:** at most the 200 oldest unresolved conversations plus every conversation named in the
  last 30 days of events (the API's limits); logged when reached.

## 7. Environment pointers
Commands and caveats: `master_plan.md` §7, `docs/guide/05_working.md`. While working: `cargo test -p psm-core --lib
mechanisms::conversation_rules` · `cargo test -p psm-integrations --lib conversations::` · `cargo test -p psm-testkit
--test suite <stem>::` · `cargo test -p psm-desktop` (regenerates `apps/desktop/src/services/bindings.ts` through
`export_bindings`; never hand-edit) · `cd apps/desktop && npx vitest run <files>` · `npm run typecheck` · `cargo clippy
-p <crate> --all-targets -- -D warnings` · `cargo check --workspace`. One filter per `cargo test` invocation. Closing
gate of every milestone (once): `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` ·
`cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time, never
in the background, never piped. Known intermittent tests: `docs/guide/04_change_impact.md` §Fragile spots.
New dependencies: C1 adds `chrono` to `psm-integrations` if the adapter needs it for ISO 8601 parsing (already in the
lock file through `psm-core`) and may add `url` for `checked_server` (already in the lock through `psm-research`;
projection C1 L23); C2 adds `tauri-plugin-notification = "2"` to `apps/desktop/src-tauri/Cargo.toml`
(registered with `.plugin(tauri_plugin_notification::init())` beside dialog and opener; used from Rust only, so no
capability entry and no npm package). macOS asks the user once to allow notifications for the app the first time a
banner is sent (under `tauri dev` the request is attributed to the terminal); a refusal is not reported to the app.
Migration number: `019` is the next free file in `crates/infra/src/sqlite/migrations/` at `78869e5`; if another
change takes it first, use the next number (never renumber an applied file).

## 8. Log
- **v1, 2026-11-20 (planner).** Written from 09B and the Pricing integration package. No owner card. Points for the
  orchestrator in `handoffs/reviewer/handoff_conversations_plan_r1.md`.
- **v1.1, 2026-11-20 (orchestrator).** C1 projection r0 (`handoffs/reviewer/handoff_C1_connection_and_sync_projection_r0.md`)
  amendments applied: §4 Ports (`describe`, `ConversationSettingsSlot`), Secret (redacting decorator), Settings (where
  the rules live; `prepare` validation, merge), Service (seams and lifetime: constructor, return types, key in memory,
  process-wide lane, bounded join); §5 lane notation, MC-C1.1 (in-memory key, server), .2 (derived states in memory),
  .3 (trim, server change, `Err` return), .5(a) (last event deleted → forgotten), .5(d) (first connection = complete
  check), .6 (`resolved_by` scope), .8 (`last_error` for `Refused`/`StoreGone`), .9 ("applied" = changed the copy);
  §6 server-change choice; §7 `url`. C1 now 111 rows (C1·C11(h)); totals 294.
