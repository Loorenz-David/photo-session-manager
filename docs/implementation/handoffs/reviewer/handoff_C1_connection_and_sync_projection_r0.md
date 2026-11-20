---
plan: C1_connection_and_sync
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-20
actor: Claude Opus 5.5 (pipeline-projectionist, plan-projection doctrine; orchestrated)
---
# Projection of C1_connection_and_sync (round 0)

The plan can be built, and most of its 110 test rows can be written today with one exact expected result. But some things
it needs cannot be put where it puts them. The core layer is told to write messages that mention Pricing, check the
https address, register the key for log hiding and read and write the settings file, and the core layer can do none of
these without a hook the plan never names. One contract step contradicts its own test row: the first connection
would fetch a conversation that was already deleted, and the row expects no fetch. A few rules can quietly go wrong at
runtime: the Keychain could be read four times a second, a saved state could stay stuck after one bad launch, an http
address could get in through the general settings save, and quitting or saving Shopify settings could hang for up to
20 seconds while Pricing is slow. None of this needs the owner. Book 09B already settles every behaviour, so the
coordinator can make each fix as a paragraph in the plan or the conversations master plan before the implementer
prompt is compiled.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. Every gap below is a planning gap or a free choice the coordinator can settle from book 09B
and the code.

## Gate check
- Intention map header `docs/intention/intention.md:3`: **RATIFIED**. Book `09B_pricing_conversations.md:3`:
  **RATIFIED** (owner, 2026-11-20, A27).
- Upstream: `handoffs/reviewer/handoff_conversations_plan_r1.md`: `state: DONE`, 0 owner cards. No upstream gate is
  in `OWNER_DECISIONS_PENDING`.
- Plan frontmatter `projection: yes`. Tree projected: `75d8fcd`, clean.
- Inputs read: `docs/process/flow.md`, `master_plan_conversations.md` (all of it; it names itself the authority over
  `master_plan.md` for these milestones), base `master_plan.md` section index plus guide H6–H8, the plan, 09B (all
  of it), and the Pricing package `API.md`, `EVENTS_AND_WEBHOOKS.md` and `openapi-integration.json`
  (`components.schemas` plus the parameters and responses of every path). I also read the code each task touches.

## Decision ledger
Classification: **plan gap** = the artifacts do not decide it, and a wrong guess is a B1/B2/B3 risk or a silent
failure (amendment proposed). **free choice** = harmless if decided either way, but grant it in writing. No row is an
intention gap. MP = `master_plan_conversations.md`, P = `plans/C1_connection_and_sync.md`.

| # | Decision point (artifact:line) | Class | Proposed routing (amendment) |
|---|---|---|---|
| L1 | **Core cannot reach four things the plan says the core service uses.** (a) `ProviderError::sentence()` is "only in the adapter crate" (P:59, MP:179), but the core `ConversationsService` must write `last_error` = "its sentence" (MP:260, rows C1·C11(f), C1·C16(a)/(c)/(d)) and `test_connection` "returns the error's sentence" (MP:213). The `StoreGone` sentence also embeds `<shop>` ("This key is no longer tied to shop-a.myshopify.com", C1·C16(c)), which the adapter does not know. (b) `settings_save` "validates `server` with `checked_server`" and stores the default as none (P:131), but `checked_server` and `DEFAULT_SERVER` live in `psm-integrations`. A core service cannot call them (core depends on nothing app-side, `crates/core/Cargo.toml`), and the URL is an H7 literal. (c) `key_set` must call `psm_infra::logging::add_secret` (MP:122), but core does not depend on infra. (d) "settings source" in `ConversationsService::new(store, provider, secrets, settings source, shop, clock, events)` (P:132) has no type, and `settings_save` must also *write* `settings.json` byte-safely (C1·C7(c)). | plan gap (H7 = B3 if guessed wrong) | Name the seams in MP §4 Service. Suggested: one pure provider-port method `describe(&ProviderError, shop: Option<&str>) -> String` (precedent: C4's pure `attribute_key` on the same port), or an injected `Fn`. A `ConversationSettingsSlot` port (`read() -> Option<Json>`, `write(ConversationSettings) -> Result<()>`) implemented in integrations over `psm_infra::Settings`, with `check_server`/default normalisation done there, or move `settings_save` validation into the integrations implementation. A redaction-registering `SecretStore` decorator built in `App::build`, which calls `add_secret` on `set` and on the first `get`. Update the constructor signature at P:132. |
| L2 | **MC-C1.5(a) contradicts C1·C11(e).** MP:229–231 says to remember every `conversation_id` of a `for_us` event "that is not `ConversationUpdated {deleted: true}`". A deleted case's *earlier* `case.created`/`message.created` events still qualify, so step (c) GETs it. Row C1·C11(e) (P:200) asserts "no `GET` for the first". | plan gap (contract vs row) | Amend MC-C1.5(a): "an id whose last `for_us` event in the paged log is `ConversationUpdated {deleted: true}` is forgotten". The row then follows the contract. |
| L3 | **Is MC-C1.2's derived state written to `S.state`?** P:133 evaluates it lazily at the first `tick`/`connection()`. MP:209–212 ends "else `S.state` as stored (`ready`, `refused` or `store_gone`)". If a derived `no_key`/`no_shop` is persisted (the column default is `'no_key'`, MP:108), one launch with an unreadable Keychain or a briefly invalid integration block leaves the Mac stuck in `no_key`/`no_shop` after the cause is gone, with no call ever made. | plan gap (silent: sync never resumes) | Amend MC-C1.2: the derived `no_shop`/`no_key`/shop-mismatch `store_unmatched` live in memory only. `S.state` stores only connect and check outcomes (`ready`, `refused`, `store_gone`, `store_unmatched` from rule 3). Say whether the shop-mismatch case persists `visible_stores = []` (C1·C10(a) reads it through `connection()`, so memory is enough). |
| L4 | **How often the Keychain is read.** MC-C1.1 (MP:206) needs "a key is in the Keychain" on every `tick`, and the background thread ticks every 250 ms. The adapter's `key: impl Fn() -> Option<String>` (P:62) is called per request. `KeychainSecrets` has no cache (`crates/infra/src/keychain.rs`, no `cached`). Read literally, that is about 4 Keychain reads a second even with no key, and each can raise an access prompt. | plan gap (performance/prompt risk; 09B §10 rule 5) | Amend MP §4 Service: the key is read from the Keychain once per App build (first `tick`/`connection()`, worker thread) and on `key_set`, then held in memory. Rule 1 and the adapter's key closure read the in-memory value. |
| L5 | **Join and lane lifetime across an App rebuild.** P:189 says "stopped and joined in `shutdown`". `AppState::settings_set` calls `old.shutdown()` inline (`apps/desktop/src-tauri/src/state.rs:33–35`), and App drop does too (`lib.rs:1585`). A thread inside a 20 s request makes a Shopify settings save or quit wait up to 20 s, which breaks 09B §10 rule 5 and CR5. A bounded join (the research thread's `BACKGROUND_JOIN_BOUND` pattern, `lib.rs` shutdown) instead lets the old and new services run at once with *separate* lanes (MP:201 "one mutex of the service"). The old thread can then write an older `cursor` after the new one, so events are re-applied and notices are duplicated. | plan gap (CR5 / invariant MC-C1) | Amend MP:201 and P:189: the lane is process-wide and carried through rebuilds, like `Lanes` (P48 L14 precedent, `lib.rs:175–186`), or keyed by data folder. Shutdown joins with `BACKGROUND_JOIN_BOUND` and does not wait for a request in flight. |
| L6 | **http can get in through the general settings path.** `conversations` is "accepted by `SettingsService::set`" (MP:127) and goes through `SettingsSetContext::prepare` (C1·C7(d)). Only `settings_save` validates `server` (P:131). A `settings_set({"conversations":{"server":"http://…"}})` stores it, and `from_slot` "reads tolerantly" (MP:126), so the bearer key could be sent over http. That breaks CR2 and 09B §4 ("https only"). | plan gap (security) | Amend MP §4 Settings: `prepare` validates a `conversations` patch with `checked_server` (as it does for `research`/`writer`, `lib.rs:221–232`), **and** `from_slot` treats an invalid stored `server` as none (default). Say whether `SettingsService::set` merges `conversations` fields like `research`/`writer` (`services/settings.rs:84`) or replaces the slot. |
| L7 | **What happens to the connection when the server address changes.** MC-C1.1–3 key on shop and key only. Saving a different `server` while `ready` sends the old store id and old `cursor` to another server, so the cursor invariant no longer holds. | plan gap | Amend MC-C1.3 or MP §4 Settings: saving a different effective `server` sets in-memory `store_unmatched` (with "Press Test connection"), like a shop change. Or delegate explicitly. Either way, one sentence for "Choices made for you". |
| L8 | **Does the first connection count as a "complete check" for MC-C1.7/9?** Rule 7 (MP:255–258) lists "after a complete check = now + 10 000" and "after a successful connect … = now". Rule 5(d) sets `last_ok_at` but does not say `failures := 0` or how `next_check_at` changes. C1·C14(b) "after a complete check" passes or fails depending on this. | plan gap (decidability) | Amend MC-C1.5(d): "the first connection is a complete check for rules 7 and 9 (`failures := 0`, `next_check_at = now + 10 000`)". |
| L9 | **`last_error` on `Refused`/`StoreGone`.** MC-C1.8 (MP:260–263) sets `last_error` only for `Unreachable`/`BadReply`. Rows C1·C16(a) and (c) assert `last_error` = the refused / store-gone sentence. | plan gap (row exceeds contract) | Amend MC-C1.8: `Refused`/`StoreGone` also set `last_error`/`last_error_at` to their sentence (with L1's shop for `StoreGone`). |
| L10 | **What "applied nothing" means in MC-C1.9** (MP:269, "A check that applied nothing and had no state change emits nothing"). Cursor-only transactions (Operations events, `inbox.updated`)? Row C1·C12(g) asserts "no event emitted *with a notice*", which accepts both readings (a rule-2 disjunction). | plan gap (small) | Amend: "applied" = changed the copy. C1·C12(g)/(h) then assert "no `ConversationsChanged`". |
| L11 | **Scope of `resolved_by` in MC-C1.6** (MP:245–249): "else `Asker` when an event of this page is a `ConversationUpdated`" does not say *of this conversation*. Read literally, a `case.updated` for conversation Y marks a resolved conversation X as `Asker`. | plan gap (wording) | Amend to "a `ConversationUpdated` of this conversation in this page". |
| L12 | **Return types of `test_connection`, `key_set` and the three mutating commands.** U2's boundary returns `Connection` (P:137), but C1·C9(f) asserts "the returned error is `Unreachable`" and MC-C1.3 says "the sentence is returned". P:276 gives a DTO only for `conversations_settings_get`. C1·C19(b) ("neither answer"), C1·C20(c) ("shows Saved") and C1·C20(f) ("its answer shown") depend on what `conversations_key_set`/`conversations_test` return. The UI for an `Unreachable` Test connection (state unchanged, no `last_error` written by rule 3) is not described. | plan gap (decidability of 4 rows) | Amend P U2 task 3/4 and U4 task 1. Suggested: `test_connection`/`key_set -> Result<Connection, ProviderError>`. The commands return `ConversationsSettingsDto`. An `Unreachable`/`BadReply` from the test maps to the existing `CommandError` code `provider` (`commands/items.rs` mapping) with the adapter's sentence, and `PricingSection` shows it in the connection line until the next refresh. |
| L13 | **Where C1·C6(b) can run.** It runs "over `FakePricing`" with a log capture (P:99, P:108), but `FakePricing` and the capture helpers live in `psm-testkit`, which depends on `psm-integrations`. A `psm-integrations --lib` inline test cannot use them (cycle; a dev-dep would build a second, type-incompatible copy of the trait). U1's commands only `cargo check` testkit (P:111), and U1 creates no suite file, so the row has no runnable home in U1. | plan gap | Amend P U1: C1·C6(b) lives in `crates/testkit/tests/suite/c1_conversations_connection.rs` (created in U1, `mod` line in `main.rs` moved to U1), and U1's commands gain `cargo test -p psm-testkit --test suite c1_conversations_connection::`. Also state that the capture is **raw** (not through `psm_infra::logging::redacting`), otherwise the log clause cannot fail. |
| L14 | **Fixture provenance: "verbatim" does not match the rows** (task 6, P:63; reviewer check P:316 "every fixture file matches the package's example or schema it names"). `error_envelope.json` verbatim has code `sku_not_found` (API.md §Errors), but C1·C4(a) wants `invalid_api_key`. `case.json` has no verbatim example (API.md gives only the list), and C1·C2(g) needs a `replyTo` + `editedAt` while C1·C5(d) needs `status: answered`, but the example has `replyTo: null`, `editedAt: null`, `status: open`. `message_created.json` has no example ("Response 201 — Message (shape above, `author.isMe: true`)"). | plan gap (rule 17 provenance) | Amend task 6 with a per-file provenance list. Verbatim: `stores.json`, `cases_list.json`, `events_page.json`, `error_envelope.json`. Schema-built: `case.json` (the list example's case with `status: answered` plus a second message `{kind: service, isMe: true}` whose `replyTo` points at the first and which has `editedAt` set), `message_created.json` (`Message` schema, `author {kind: service, isMe: true}`), `events_other_types.json`. Other error codes are built in the test by replacing `error.code`. |
| L15 | **C1·C7(a) "the effective server is `beyo_pricing::DEFAULT_SERVER`"** has no observable at its boundary. `settings()` returns `ConversationSettings {server: Option, sound, banner}`, and `FakePricing` records path and query only, not the host. | plan gap (undecidable clause) | Amend C1·C7(a): drop the clause (C1·C19(a) already asserts `server`/`server_default` at the command), or name an observable (for example the URL a recording `PricingHttp` receives through `with_test_conversations`). |
| L16 | **Emission and request counts need a stated window.** `key_set` itself emits `ConversationsChanged` (P:188) and records `GET /stores`. C1·C11(b)/(c) count "exactly one" event and list requests "in order" from `events?after=0`. Emission happens only when a check completes (MC-C1.9), and a non-empty pull page returns `more: true` (MP:251–252). So C1·C17(a)–(f), whose boundary says "`tick`", emit at the *second* tick. | plan gap (decidability, small) | Amend P U3 boundary: "events and requests are counted from after `key_set` returns. Rows whose outcome is an emission run `tick`s until `more` is false". |
| L17 | **C1·C15(h) states one-sided bounds** ("not before +10 s"), so a 15 s backoff passes (rule 2). | plan gap (weak outcome) | Amend: "no request at +9 999 ms, one at +10 000; none at +19 999 after it, one at +20 000; after the success none at +9 999, one at +10 000". |
| L18 | **Trimming the key.** MC-C1.3 does not trim. A pasted key with a trailing newline makes an invalid header value. reqwest then fails the build, the error becomes `Error::Provider("http: …")`, and that maps to `Unreachable`: "Could not reach Pricing" for what is really a bad paste. Research trims (`crates/research/src/api_impl.rs:114`). | plan gap (small, misleading state) | Amend MC-C1.3: the key is trimmed, and empty after trimming = remove. |
| L19 | **HTTP statuses the rows do not list.** P:62 says "mappings as the rows below", but the rows leave out 400/409/422/429 on reads, 401/403/5xx/transport on `post_message`/`mark_answered`, and 404 on `mark_answered`. | free choice | Delegate in writing: 401/403 → `Refused` on every method. 5xx and transport errors → `Unreachable` on every method. Any other non-2xx on a read → `BadReply`. 404 on a write → `NotFound` (C3 refines). |
| L20 | **Type of `topic`** (MP:77–79, no type). API.md lists five values, but the schema has no enum, and API.md says changes are additive. | free choice | Delegate: `topic: String`, kept exactly as received. An unknown topic is never `BadReply`. |
| L21 | **How strictly times are parsed** (P:62: "unparseable → `BadReply`"). The package's evidence is its examples (`…Z`, 3 fractional digits). A pydantic-style server may send 6 digits or `+00:00`. | free choice | Delegate: accept any RFC 3339 form (`Z` or offset, 0–9 fractional digits). `"yesterday"` is still `BadReply`. |
| L22 | **`FakePricing` endpoints with no C1 caller** (POST/PATCH semantics, `drop_answer_next`; P:64, P:102 "test caller in C1–C3"). The C1 reviewer cannot see C3 callers. | free choice (record it) | Record in P: these are built in C1 and first called in C3, and the C1 review does not flag them under rule 4. |
| L23 | **Dependencies the adapter may add.** Files allow `Cargo.toml` "only if `chrono` is needed" (P:52). `checked_server` could use `url` (already in the lock through `psm-research`). | free choice | Delegate: `url` may be added to `psm-integrations`, or the address is parsed by hand. |
| L24 | **Contradiction over `visible_stores` encoding.** P:158 delegates "JSON text or a joined string", but MP:108 fixes `DEFAULT '[]'` (JSON). | free choice (already decided) | Remove the delegation and keep JSON. |
| L25 | **C1·C7(b) "the slot holds … `banner` true"**: does `settings_save` write a full, normalised `ConversationSettings` or merge only the fields given? | free choice | Delegate: write the full normalised object (like `ResearchSettings::from_slot` in `prepare`, `lib.rs:234–239`), and assert through `ConversationSettings::from_slot`. |

## Reality checks
- **Paths.** Every existing path in Read-first and Files resolves at `75d8fcd` (checked by script, 46 paths). None of
  the files marked new exists yet. Migration `019` is free (latest is `018_saved_pages.sql`, `store.rs:86`).
- **Prior-phase dependencies** match the plan's claims: `HttpReply` and `map_error` (`Error::Timeout`,
  `Error::Provider("http: …")`, non-JSON body → `Json::String`), `transport.rs`. `SecretStore` port and `FakeSecrets`.
  `Settings.writer` with `skip_serializing_if` (`settings_file.rs:41`). `prepare` copies `research`/`writer` from
  disk (`lib.rs:212–218`) and nulls them in the comparison (`lib.rs:264`). `with_test_research` keyed by data folder
  (`lib.rs:1707`). The `storage_background` loop (`lib.rs:1132`). `ManualClock`. `TestApp::kill_and_restart` keeps
  secrets and the clock, so the "reopen" rows (C1·C10(a), C1·C13(b), C1·C14(c), C1·C17(i)) are buildable.
- **Note for `TestApp::with_pricing()`:** `with_fake_shop` writes **no** `integration` block (`test_app.rs:618–637`),
  so the "connected shop" (`settings.integration.shop_domain`, MP:132) would be `None` and every row would read
  `no_shop`. `IntegrationSettings` needs `kind`, `shop_domain`, `client_id` and `client_secret`
  (`settings_file.rs:206–214`), so `with_pricing()` must write a full block (fake id and secret). This follows from the
  code and is not a plan gap. Folding the sentence into U2 task 3 would save the implementer a debugging loop.
- **C1·C18(d)** names core `storage.delete_session`. The user's path is `psm_integrations::session_delete::delete`
  (`commands/sessions.rs:606`, which wraps it). Nit: use the command's path so a future hook in the wrapper is covered.
- **a17 schema check** (`a17_delete_session.rs:95–120`) scans `session_id`/`item_id`/`photo_id`/`publish_item_id`
  columns. The four new tables carry none of them (`product_id`, `conversation_id`, `store_id`), so there is no
  collision.
- **H7 is B3 here.** Guide H7 says "do not add more" H7 debts. That is why L1 matters: the obvious shortcut
  (sentences or `DEFAULT_SERVER` in core) is a blocking finding at review.

## Criteria decidability and producibility (both questions, every row)
"Decidable" = the test can be written now with one exact outcome at the named boundary. "Producible" = the fixture's
external shape can be built from the package for this repo (rule 17). These are external-shape rows, so neither
demotion nor waiver applies.

| Rows | Decidable | Producible | Note |
|---|---|---|---|
| C1·C1(a)(b) | yes | yes: `stores.json` verbatim (API.md `GET /stores`) | — |
| C1·C2(a)(b)(c)(d)(e)(f) | yes | yes: `cases_list.json` verbatim. `text: null` / `deleted` per `Message` schema. `sku` nullable. `{kind: service}` without `isMe` defaults to false (`Actor` schema) | — |
| C1·C2(g)(h) | yes | yes once L14 names `case.json` as schema-built | L14 |
| C1·C3(a)–(e) | yes | yes: `events_page.json` verbatim. `StoreEvent.caseId`/`productId` nullable. `data` is a free object. `resolvedBy: "activation"` and `deleted: true` per EVENTS_AND_WEBHOOKS | Free choice: `conversation_id` comes from the top-level `caseId` (equals `data.caseId`) |
| C1·C4(a)–(h) | yes | yes: envelope per `ErrorEnvelope`. 502 is in every path's responses. The transport shapes come from `map_error` | (a) needs L14. Other statuses: L19 |
| C1·C5(a)–(d) | yes (compare bodies as JSON values, not bytes) | yes once L14 names `message_created.json` / `case.json` as built | L14 |
| C1·C6(a) | yes | n/a (our rule) | — |
| C1·C6(b) | **no home** | yes | L13 |
| C1·C7(a) | **third clause undecidable** | n/a | L15 |
| C1·C7(b)(c)(d) | yes ((b) after L25) | n/a | L6 for the generic path |
| C1·C8(a)(b)(c) | yes ((c): the capture goes through `redacting`, and the seam needs L1(c)) | n/a | L1 |
| C1·C9(a)–(e) | yes | yes (`FakePricing`, serialised through `types.rs`) | `with_pricing` integration block (reality note) |
| C1·C9(f) | **no, return type unspecified** | yes (`timeout_next` → `Error::Timeout`) | L12 |
| C1·C10(a)(b)(c) | yes ((a) after L3) | yes | L3 |
| C1·C11(a)(b)(d)(f)(g) | yes ((b) after L16) | yes | — |
| C1·C11(c) | yes after L16 | yes | L16 |
| C1·C11(e) | **contract contradicts the row** | yes | L2 |
| C1·C12(a)–(f), (h), (j) | yes | yes. (e): whether real Pricing also emits `message.deleted` before `case.updated {deleted}` is undocumented, but both shapes give the same outcome under MC-C1.6 within one page | — |
| C1·C12(g) | disjunctive outcome | yes | L10 |
| C1·C12(i) | yes | **not producible by the real pull** (`after` is exclusive per API.md). This is a defensive check that 09B §5.3 explicitly mandates, so keep it as a guard, not as fidelity evidence | — |
| C1·C13(a)–(d) | yes | yes | — |
| C1·C14(a)(c)(d) | yes | yes | — |
| C1·C14(b) | depends on L8 | yes | L8 |
| C1·C15(a)–(g) | yes (pure. (g): saturate `2^(n−1)`) | n/a | — |
| C1·C15(h) | weak (one-sided) | yes | L17 |
| C1·C16(a)(c) | yes after L9 (and L1 for `<shop>`) | yes | L9, L1 |
| C1·C16(b)(d) | yes | yes | — |
| C1·C17(a)–(f), (i) | yes after L16 | yes | L16 |
| C1·C17(g)(h) | yes (pure; exact `·` and `—` characters) | n/a | — |
| C1·C18(a)–(d) | yes | yes | (d): nit, delete path |
| C1·C19(a)(c)(d)(e)(f) | yes | yes (`listen_any` bridge pattern exists, `lib.rs:270–295`) | — |
| C1·C19(b) | depends on L12 | yes | L12 |
| C1·C20(a)(b)(d)(e)(g)(h) | yes ((g) once the "Last checked" format is chosen; the delegation exists) | n/a (mocked service) | — |
| C1·C20(c)(f) | depend on L12 | n/a | L12 |
| C1·C21(a) | yes (blocks on the fake's latch) | yes | L5 (lane scope) leaves the row as written |

Counts (derived by script from P): U1 6 criteria / 29 rows, U2 4 / 16, U3 8 / 50, U4 3 / 15. Total 21 / 110, which
matches P:21–27 and MP:22. Every unit is at or below 8 criteria (flow §10).

## Trace verification
- **Row → trace:** every row has a trace cell that resolves to 09B §§, MC-C1.x, MC-C2.x, CR or CV. Spot checks of
  the cited content agree with the rows, except C1·C11(e) against MC-C1.5 (L2) and C1·C16(a)/(c) against MC-C1.8 (L9).
- **Contract → rows:** MC-C1.1 (C8(b), C9(c), C10(a)), .2 (C10(a)), .3 (C9, C10), .4 (C2, C12(j), C18(b)), .5 (C11),
  .6 (C3, C12, C13), .7 (C13(d), C14, C15), .8 (C4, C16), .9 (served in substance by C11(a)(c) and C17(a)–(f)/(d)
  ordering, though no row cites "MC-C1.9"; that is a nit), .10 (C18(a)(c)). Invariant MC-C1 (C11(f), C13(a)).
  CR2 (C6(b), C8), CR3 (C18(d)), CR5 (C21(a)).
- **No row covers two of MC-C1's clauses:** "`next_cursor ≤ after` with events present → `BadReply`" (MP:229),
  which guards an endless first-connection loop, and the background thread actually being spawned when
  `run_startup` is on (charter rule 10: the shipped default must reach the path, and `TestApp` runs without the
  thread, P:254). Candidates for the coordinator: add the first as a row (a scripted bad page in `FakePricing`). Add
  the second to the Review mode's reading checklist (P:316), since a test of it needs real time.

## Write perimeter
- Created: this file only (`docs/implementation/handoffs/reviewer/handoff_C1_connection_and_sync_projection_r0.md`).
- No code, plan, master plan, intention, tracker or prompt was edited. No archgraph (none in this repo). No commit.
- Scratch: none written. The skeleton was derived on paper and thrown away, as the doctrine requires. No appendix is
  attached, so the implementer gets nothing from this file but the routed amendments.

## Exit
The implementer prompt should not be compiled until L1–L18 are applied as amendments to P or MP (MP §5 is a planning
contract, so the coordinator owns it) and L19–L25 are recorded as written delegations in P's "Delegated choices".
