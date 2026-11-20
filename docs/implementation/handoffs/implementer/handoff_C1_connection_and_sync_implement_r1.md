---
plan: C1_connection_and_sync
role: implement
round: 1
state: DONE
date: 2026-11-20
actor: claude (opus, pipeline-implementer)
---
# Handoff — C1_connection_and_sync, implement r1

Gate: intention RATIFIED (2026-10-24); `depends_on: []`; the plan's Tracker line read NOT_STARTED; projection r0 done
(amendments applied by the orchestrator, plan and master plan v1.2). Worked the four units in order, a
`CHECKPOINT (not approved):` commit after each unit's test commands passed.

## What was built
**For the photographer (once the owner pastes a key in the installed app):**
- **Settings → Pricing** (a new section after "Shopify", always shown). **Server address** prefilled with
  `https://pricing.beyoworkaroundtheclock.com` (https only; anything else is refused naming the server).
  **API key**: a password field with **Save key**; once saved it reads **Saved** with **Replace** and **Remove**; the key
  is kept only in the macOS Keychain and never shown again. **Test connection**. A line in words: "Connected to <store>
  (<shop>)", "This Pricing key isn't tied to <shop>. It can see: One, Two.", "… Press Test connection.", "Connect Shopify
  first; conversations are matched to its products.", "Pricing refused the key: check Settings → Pricing", "This key is
  no longer tied to <shop>"; when Pricing cannot be reached or answers oddly during Save key / Test connection, its
  words ("Could not reach Pricing", …) are shown there until the next refresh. Two switches saved at once: **Sound for
  new questions and replies** and **Mac banner when Session Manager is in the background** (both on). **Last checked**:
  the clock time of the last good check, or the last error in words when it is newer. The section refreshes on
  `conversations_changed` and sits outside the page's unsaved-changes guard.
- **The local copy.** Saving the key tests the connection at once. When the key's store matches the connected Shopify
  shop, the app makes a **silent first connection** (pages Pricing's event log from the start, lists the unresolved
  photographer conversations, reads any other conversation the log names, saves the cursor), then checks **every 10
  seconds** (at once at launch, on waking, after the key changes), one events page per turn, re-reading each named
  conversation and replacing it in the copy, the cursor in the same transaction. Questions to Operations, other event
  types and redelivered events only move the cursor. Unreachable Pricing → wait 10 s, 20 s, 40 s … 5 min, back to 10 s
  after a success; a refused key or a gone store stops checking until the key is replaced. A different Pricing store
  empties the copy; a changed shop or server asks for Test connection again (persisted, so a restart cannot send the
  old store and cursor to a new server). Read marks per Mac (open conversations enter unread), `mark_read`, notices
  per conversation (Question / Reply, never for the app's own replies or the first connection), and the window event
  `conversations_changed {toasts, more}` with the toast words. Nothing is shown yet (C2) and nothing is written to
  Pricing (C3).

**In code:** core vocabulary `crates/core/src/domain/conversation.rs`; ports `crates/core/src/ports/conversations.rs`
(`ConversationsProvider` incl. pure `describe`, `ConversationStore`, `ConversationSettingsSlot`); rules
`mechanisms/conversation_rules.rs`; `services/conversations.rs` (`ConversationsService`, `ConversationsLane`,
`ConnectError`, `ConversationSettingsPatch`); adapter `crates/integrations/src/conversations/beyo_pricing/`
(`BeyoPricingAdapter`, `PricingHttp`/`ReqwestPricing`, `types.rs`, `checked_server`, `DEFAULT_SERVER`, 7 fixtures);
`conversations/mod.rs` (`SettingsFileSlot`, `RedactingSecrets`, `with_test_conversations`, `build_service`); migration
`019_conversations.sql` + `crates/infra/src/sqlite/conversations.rs`; `Settings.conversations`; the
`conversations_background` thread (only with `run_startup`, joined within `BACKGROUND_JOIN_BOUND`) and the lane in
`Lanes`; `AppEvent::ConversationsChanged` and its `TauriEvents` arm; commands
`apps/desktop/src-tauri/src/commands/conversations.rs`; `services/conversations.ts`, `onConversationsChanged`,
`PricingSection.tsx`; `psm_testkit::fake_pricing::FakePricing` and `TestApp::{with_pricing, with_pricing_no_shop}`;
the guide (`01`–`04`).

## Tests → criterion rows
Every row has one test at the boundary it names, asserting the exact outcome. All 112 rows covered; no orphan tests.
Files: adapter inline `crates/integrations/src/conversations/beyo_pricing/{adapter.rs, mod.rs}` (28); rules inline
`crates/core/src/mechanisms/conversation_rules.rs` (11); `crates/testkit/tests/suite/c1_conversations_connection.rs`
(19); `crates/testkit/tests/suite/c1_conversations_sync.rs` (44); `apps/desktop/src-tauri/src/lib.rs` `c1_tests` (6);
`apps/desktop/src/features/settings/__tests__/c1_pricing_settings.test.tsx` (8).

| Row | Test | Shape |
|---|---|---|
| C1·C1(a) | `adapter::tests::c1_c1a_stores_parses_the_fixture` | exact |
| C1·C1(b) | `c1_c1b_stores_sends_one_get_with_the_bearer` | exact (whole request list) |
| C1·C2(a) | `c1_c2a_unresolved_maps_the_case_and_its_message` | exact (whole `Conversation`) |
| C1·C2(b) | `c1_c2b_unresolved_asks_for_the_photographer_unresolved_cases` | exact |
| C1·C2(c) | `c1_c2c_deleted_and_textless_messages_are_not_kept` | exact |
| C1·C2(d) | `c1_c2d_author_kinds` | exact |
| C1·C2(e) | `c1_c2e_null_sku_and_absent_optional_fields` | exact |
| C1·C2(f) | `c1_c2f_operations_cases_are_dropped` | exact |
| C1·C2(g) | `c1_c2g_conversation_reads_reply_to_and_edit_time` | exact |
| C1·C2(h) | `c1_c2h_conversation_for_operations_is_none` | exact |
| C1·C3(a) | `c1_c3a_events_page_maps_the_message_created_event` | exact (page + query) |
| C1·C3(b) | `c1_c3b_other_event_types_are_other_and_not_for_us` | exact |
| C1·C3(c) | `c1_c3c_case_updated_reads_deleted_and_resolved_by` | exact |
| C1·C3(d) | `c1_c3d_the_other_case_and_message_types` | exact |
| C1·C3(e) | `c1_c3e_operations_events_are_not_for_us` | exact |
| C1·C4(a)–(h) | `c1_c4a_…` … `c1_c4h_unknown_fields_are_ignored` (8 tests) | exact (variant; (h) equality with the plain parse) |
| C1·C5(a) | `c1_c5a_post_message_sends_text_and_client_id` | exact (body compared as a JSON object, see Assumptions) |
| C1·C5(b) | `c1_c5b_post_message_with_reply_target_and_media` | exact |
| C1·C5(c) | `c1_c5c_post_message_refusals` | exact |
| C1·C5(d) | `c1_c5d_mark_answered_patches_the_status` | exact |
| C1·C6(a) | `beyo_pricing::tests::c1_c6a_checked_server_keeps_only_bare_https_hosts` | exact |
| C1·C6(b) | `c1_conversations_connection::c1_c6b_the_key_never_reaches_an_error_a_log_or_the_request_record` | exact (raw capture) |
| C1·C7(a)–(d) | `c1_c7a_…`, `c1_c7b_…`, `c1_c7c_…` (bytes compared), `c1_c7d_…` | exact |
| C1·C8(a)–(c) | `c1_c8a_…`, `c1_c8b_…`, `c1_c8c_…` (redacting writer) | exact |
| C1·C9(a)–(e) | `c1_c9a_…` … `c1_c9e_…` | exact |
| C1·C9(f) | `c1_c9f_a_refused_key_is_refused`, `c1_c9f_a_time_out_keeps_the_ready_state_and_returns_unreachable` | exact |
| C1·C10(a)–(d) | `c1_c10a_…` … `c1_c10d_…` | exact |
| C1·C11(a)–(h) | `c1_conversations_sync::c1_c11a_…` … `c1_c11h_…` | exact ((a) uses the fake's actual last event id in place of "37") |
| C1·C12(a)–(j) | `c1_c12a_…` … `c1_c12j_…` ((d): two tests) | exact |
| C1·C13(a)–(d) | `c1_c13a_…` … `c1_c13d_…` | exact |
| C1·C14(a)–(d) | `c1_c14a_…` … `c1_c14d_…` | exact |
| C1·C15(a)–(g) | `conversation_rules::tests::c1_c15a_…` … `c1_c15g_…` | exact |
| C1·C15(h) | `c1_c15h_failures_back_off_and_a_success_returns_to_10_seconds` | exact (both bounds) |
| C1·C16(a)–(c) | `c1_c16a_…`, `c1_c16b_…`, `c1_c16c_…` | exact |
| C1·C16(d) | `c1_c16d_a_server_error_…`, `c1_c16d_a_time_out_…`, `c1_c16d_a_reply_that_is_not_an_event_page_…` | exact |
| C1·C17(a)–(f), (i) | `c1_c17a_…` … `c1_c17f_…`, `c1_c17i_…` | exact |
| C1·C17(g), (h) | `conversation_rules::tests::c1_c17g_…`, `c1_c17h_…` | exact |
| C1·C18(a), (b) | `conversation_rules::tests::c1_c18a_…`, `c1_c18b_…` | exact |
| C1·C18(c), (d) | `c1_c18c_…`, `c1_c18d_…` | exact |
| C1·C19(a)–(f) | `c1_tests::c1_c19a_…` … `c1_c19f_the_bridge_forwards_the_toast_words` | exact |
| C1·C20(a)–(h) | `c1_pricing_settings.test.tsx` `c1_c20a_…` … `c1_c20h_…` | exact |
| C1·C21(a) | `c1_conversations_sync::c1_c21a_screens_answer_while_a_request_is_held` | exact (the fake's latch) |

Followed tests: `p01_storage_identity::c1a_open_creates_all_domain_columns` (enumerates the tables) lists the four new
tables; `t1_settings_drawer.test.tsx` lists "Pricing" after "Shopify" (and mocks the section).

## Assumptions
1. **Return type of `test_connection` / `key_set`**: `Result<Connection, ConnectError>`, with
   `ConnectError::{Provider(ProviderError), Storage(psm_core::Error)}` (the master plan allows "a small enum"); only
   `Unreachable`/`BadReply` of `stores()` come back as `Provider`.
2. **A key refused before any store was ever saved reports `refused`.** MC-C1.2 read literally (no saved store →
   `store_unmatched`) would show "Press Test connection" after a wrong key on a fresh Mac; row C1·C9(f) asks for
   `refused`, so "no saved store and `S.state = refused`" reports `refused`.
3. A refused key at connect also writes `last_error` (the refused sentence) and its time, like MC-C1.8.
4. **First connection (c):** "for each remembered id not in the copy" is read as "not returned by this attempt's list
   (b)" (a copy left by an earlier, interrupted attempt can be stale), and an id whose last event removed it, or whose
   read answers 404, is also removed from such a left-over copy. This keeps MC-C1's invariant ("the next complete check
   ends with the copy of an uninterrupted run"); rows C1·C11(b)/(e)/(f) hold either way.
5. **Server change**: compared on the stored form of the setting (none = the default address); the slot stores a
   server equal to the default as none and reads it back as none, so `S.server` is null when connected through the
   default (C1·C10(d)).
6. **Settings patch**: `{server?, sound?, banner?}`; `server: ""` returns to the default address.
7. **Emission at a completed check**: when it changed the copy, found notices, **or cleared a stored error** (the
   "Last checked" line changes); every check error emits (also each backoff failure).
8. **Lane**: each lane section re-checks that calls are allowed for the same store and, during the pull, that the
   cursor is the one the check expects; otherwise the check stops quietly (another connect or an old App's thread
   moved it).
9. Adapter: an author with `isMe` is `Me` whatever its kind; an unknown kind is `Other`; an event's conversation id is
   its `caseId`, else `data.caseId`; `resolvedBy` other than `activation` reads as none; a non-2xx write with an envelope
   is `Rejected {code, message}`, 409 `case_resolved` is `Resolved`.
10. **C1·C5(a)/(d) "body exactly"** is asserted as JSON-object equality (`serde_json` writes keys sorted, so the bytes
    read `{"clientMutationId":…,"text":…}`); the body has exactly the two fields.
11. `checked_server` is hand-parsed (no `url` crate): host of letters, digits, `-`, `.`, optional numeric port.
12. If the live HTTP worker cannot start, the adapter gets a transport that always answers "could not reach" (the App
    still builds, H6).
13. "Last checked" words a local clock time `HH:MM:SS`; the section re-reads the settings every 10 s while it is open
    (a quiet check emits nothing, MC-C1.9, yet the try-out expects the time to move); the switches are
    `role="switch"` buttons (the repo has no shadcn Switch).
14. `FakePricing` helpers beyond the listed ones (each has a caller): `accept_key`, `add_store`, `clear_stores`,
    `remove_store`, `own_reply` (an echo of the app's own account), `message_ids`, `last_event_id`,
    `refuse_connection_next` (the `Error::Provider("http: connection refused")` of C1·C6(b)), `wait_until_held`,
    `request_lines`, `clear_requests`. "A 200 that is not an event page" (C1·C16(d)) is `fail_matching("/events", 1,
    200)` (the error envelope with status 200). Its questions carry a condition quote, `blocking: true` and the title
    "Product <SKU>".
15. Fixtures: the package's elided CDN addresses (`…_480x.jpg`) are kept verbatim; `events_other_types.json` has
    `data: {}`.

## Deviations
- U2 added a gate-only `tick` (rows C1·C8(b), C9(c), C10(a), C10(d) need "a later tick makes no request"); U3 filled it.
- U2 added `outbox_insert` / `outbox` to `ConversationStore` so C1·C10(c) can seed and read the outbox through the
  port (C3 builds on them).
- U4 edited `crates/core/src/services/conversations.rs` (outside U4's file list) to add `key_saved()` and
  `describe(&error)`, which the commands need.
- `chrono` added to `psm-integrations` (master plan §7 allows it).

## Files changed
Perimeter (all four checkpoints):
- Core: `crates/core/src/domain/{conversation.rs (new), mod.rs}`, `crates/core/src/ports/{conversations.rs (new),
  mod.rs}`, `crates/core/src/services/{conversations.rs (new), mod.rs, settings.rs}`,
  `crates/core/src/mechanisms/{conversation_rules.rs (new), mod.rs}`, `crates/core/src/events.rs`.
- Infra: `crates/infra/src/sqlite/migrations/019_conversations.sql` (new), `crates/infra/src/sqlite/conversations.rs`
  (new), `crates/infra/src/sqlite/{mod.rs, store.rs}`, `crates/infra/src/settings_file.rs`.
- Integrations: `crates/integrations/src/conversations/{mod.rs, beyo_pricing/{mod.rs, adapter.rs, transport.rs,
  types.rs, fixtures/*.json}}` (new), `crates/integrations/src/lib.rs`, `crates/integrations/Cargo.toml`, `Cargo.lock`.
- Testkit: `crates/testkit/src/fake_pricing.rs` (new), `crates/testkit/src/{lib.rs, test_app.rs}`,
  `crates/testkit/tests/suite/{c1_conversations_connection.rs, c1_conversations_sync.rs}` (new),
  `crates/testkit/tests/suite/{main.rs, p01_storage_identity.rs}`.
- Desktop: `apps/desktop/src-tauri/src/commands/conversations.rs` (new), `apps/desktop/src-tauri/src/{commands/mod.rs,
  lib.rs}`, `apps/desktop/src/services/{conversations.ts (new), bindings.ts (generated), events.ts}`,
  `apps/desktop/src/features/settings/{PricingSection.tsx (new), SettingsNav.tsx, SettingsPage.tsx}`,
  `apps/desktop/src/features/settings/__tests__/{c1_pricing_settings.test.tsx (new), t1_settings_drawer.test.tsx}`.
- Docs: `docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md}`; the plan's
  Tracker line and Review log (`docs/implementation/plans/C1_connection_and_sync.md`); this handoff.
No mutation probes (flow.md §4: no mutation regime). No tool-recorded state (no architecture graph in this repo).

## Checkpoint SHA
In unit order: U1 `502b501` · U2 `bbaf423` · U3 `f7553a9` · U4 `0868835` · closing (tracker, Review log, handoff,
gate fixes if any): see the last `CHECKPOINT (not approved): C1 close` commit.

## Stamp
Closing gate, run once, one command after the other, on the tree of `0868835` (U4) plus the closing edits (the
Pricing section's 10 s refresh, TypeScript only, tested by its own file and `npm test` below):
| Command | Result |
|---|---|
| `cargo test --workspace` | green: 1998 passed, 0 failed, 0 ignored (all test programs summed) |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `cd apps/desktop && npm run typecheck` | green |
| `npm test` | green: 109 files, 679 tests |
No pre-existing failures. Unit test commands of each unit were green before its checkpoint (U1: integrations
`conversations::` 28/0, suite `c1_conversations_connection::` 1/0; U2: suite 19/0, `p01…c1a` 1/0; U3: rules 11/0, suite
`c1_conversations_sync::` 43/0, `c1_conversations_connection::` 19/0, `cargo check --workspace`; U4: `cargo test -p
psm-desktop` 68/0, `vitest src/features/settings` 20 files / 107 tests, typecheck, sync 44/0; clippy on the touched
crates each time).

## Manual try-out
(The owner's, on the installed app; sessions never launch the built app.)
1. Open Settings → Pricing. The address is filled in. Paste the API key from Pricing (Settings → Service accounts) and
   press Save key: the field turns into "Saved" and the line reads "Connected to <your store> (<your shop>)".
2. Press Test connection: the same line. "Last checked" shows a time that moves every 10 seconds or so.
3. Turn Wi-Fi off for a minute: "Last checked" says "Could not reach Pricing"; every other screen works as before. Turn
   it on again: within at most a few minutes the time returns.
4. Nothing else changes yet: the icon and the conversations arrive with the next step.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
