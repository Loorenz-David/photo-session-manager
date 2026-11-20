---
plan: C1_connection_and_sync
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-20
actor: claude (opus, pipeline-reviewer)
---
# Review: C1_connection_and_sync, round 1

**Gate.** The plan's tracker line read `IMPLEMENTED`. Checkpoints `502b501` (U1), `bbaf423` (U2), `f7553a9` (U3), `0868835` (U4) and the close `9b3ac95` are all in `git log`. Process authority: `docs/process/flow.md` (no mutation probes, no named-mutation audit, blocking only for B1–B4). No production file was edited, not even temporarily.

## Verified correct
**Perimeter.** `git show --stat` of the five commits, read in unit order. The paths are exactly the handoff's **Files changed** list. That list includes the declared deviations: U2 added the outbox methods to the port, U4 added `key_saved`/`describe` to the core service, `chrono` was added to `psm-integrations`, and `p01_storage_identity.rs` and `t1_settings_drawer.test.tsx` were updated to follow the change. No path is extra or missing.

**Checks (the plan's Review mode, run once each, one cargo command at a time):**
| Command | Result |
|---|---|
| `cargo test -p psm-core --lib mechanisms::conversation_rules` | 11 passed |
| `cargo test -p psm-integrations --lib conversations::` | 28 passed |
| `cargo test -p psm-testkit --test suite c1_conversations_connection::` | 19 passed |
| `cargo test -p psm-testkit --test suite c1_conversations_sync::` | 44 passed |
| `cargo test -p psm-desktop` | 68 passed (incl. 6 `c1_tests`) |
| `npx vitest run src/features/settings` | 20 files, 107 passed |
| `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit -p psm-desktop --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green (exit 0) |
| `npm run typecheck` | green |
The implementer's full-suite stamp (workspace 1998/0, npm test 679/0) stands. Nothing contradicts it.

**Criterion rows (112).** For each row I read the production code that makes it true, then its test.
- **U1 adapter (C1–C6).** `adapter.rs` maps every status the rows and the delegated choices name: 401/403 → `Refused`, 5xx and transport errors → `Unreachable`, 404 → `StoreGone` on lists, `None` on one conversation and `NotFound` on writes, 409 `case_resolved` → `Resolved`, any other envelope → `Rejected`. Times parse as RFC 3339. A message that is deleted or has no text is dropped. `isMe` → `Me`, `user` → `Asker`. A non-photographer case is dropped from lists and gives `None` on one read. `for_us` requires the photographer recipient and a known type. The tests compare the whole value or the whole request list, so a wrong mapping fails them. `checked_server` refuses all five bad forms. C1·C6(b) uses a raw subscriber and checks errors, sentences, the log and the request record.
- **U2 connection (C7–C10).** `ConversationsService::derive` follows MC-C1.2, plus the server rule of MC-C1.3. `connect` runs one transaction per outcome (`record_error`, `store_unmatched`, `connect_store` with reset when the store id differs). `SettingsFileSlot` refuses an invalid server and writes nothing, and stores the default address as none. `prepare` validates and normalises the slot and never rebuilds for it. C1·C7(c) compares the file bytes. C1·C8(c) only passes if `RedactingSecrets::set` registered the key: the redaction is registry-based only, which I checked in `crates/infra/src/logging.rs`. C1·C10(d) reopens the app and checks that neither server gets a request and that the cursor stays 40.
- **U3 sync (C11–C18).** The first connection pages, then lists, then reads the named ids the list did not return, and rejects `next_cursor ≤ after` when events are present. The pull skips ids ≤ cursor, moves only the cursor for events not for us, deletes on `deleted: true`, defers to a later event of the same conversation, and re-reads otherwise. `resolved_by` follows MC-C1.6. Backoff is `min(10 s·2^(n−1), 300 s)`. Notices are marked per conversation, ordered by first event, and filtered by the copy. Emission follows MC-C1.9 plus assumption 7. The tests assert whole request lists (C11(b), (g), (h), C13(d)), exact cursors, copy equality and exact event lists, all at both bounds where the row gives two (C14(b), C15(h)).
- **U4 commands and UI (C19–C21).** The commands run in `spawn_blocking` and return `ConversationsSettingsDto` with only `key_saved` (no key). `Unreachable`/`BadReply` → `CommandError` `provider` in the adapter's words. The bridge forwards `notice_lines`. The UI tests drive the mocked service and assert the shown words and the call arguments. C1·C21(a) holds a request on the fake's latch while the sessions list and `connection()` answer. `connection()` takes no lane.

Every row's test observes the row's outcome at the row's boundary and would fail on a wrong outcome. No B2.

**Review-mode reading checks.**
- `crates/core` production code has no Pricing word, URL or `photographer`. The only hit is the test variable `pricer` (note 5).
- The cursor is written in the same SQLite transaction as the change it follows at every site: `apply`, `delete_conversation` and `set_cursor` take the cursor, and `finish_first_sync` writes the cursor with `first_sync_done`.
- No provider request when MC-C1.1 is false. `tick` and every `in_lane` section re-evaluate `calls_allowed` (key held, shop matching, connected server, state `ready`) inside the lane before the request. Only `connect` calls `stores()`, and only with a shop and a key.
- The key appears in no DTO, log call, error text or request record. `PricingRequest`'s `Debug` prints `<redacted>`. Transport errors go through `without_url`. The fake keeps only `authorized: bool`.
- Every fixture matches its source. `stores.json`, `cases_list.json`, `events_page.json` and `error_envelope.json` are the package's examples (elided ids filled in). `case.json`, `message_created.json` and `events_other_types.json` are built from the schema as the provenance list says.
- No real host is called. A real transport is used only when no test seam is registered, and it is never reached without a key and a matched store. No test registers a real key.
- `conversations_background` is spawned in `App::build` under `run_startup`, the same gate the shipped storage and research threads use. `shutdown` joins it within `BACKGROUND_JOIN_BOUND`.
- The guide (`01`–`04`) has the service, adapter, thread, tables, event, settings slot and change-impact rows.

**Hard constraints H1–H8.**
- H1: `post_message` and `mark_answered` have no production caller (only adapter tests).
- H2, H3, H5: not touched.
- H4: each transaction carries its cursor, and the first connection restarts idempotently (C1·C11(f), C13(a)).
- H6: the service is always present and silent without a shop or key (C1·C9(d)).
- H7: core uses neutral vocabulary only.
- H8: the commands are thin and core has no Tauri dependency.
- Secrets and no-live-services rules hold. No B3.

**Assumptions** (handoff §Assumptions 1–15). None contradicts book 09B or the plan. Assumption 2 (refused before any store → `refused`) is what row C1·C9(f) asks for. Assumption 4 (first connection (c) keyed on this attempt's list) keeps the MC-C1 invariant more strictly than the literal text. Assumption 7 (emit when a stored error is cleared) matches the "Last checked" behaviour of 09B §4. Assumptions 2, 4 and 7 are written to the backlog as contract folds. All user-visible ones are in the narrative in plain words. No B1.

## Findings
None blocking.

## Notes written to backlog
To `docs/implementation/backlog_conversations/C1_connection_and_sync.md`. This is the location the plan's Review mode and master plan §3 name for this milestone, not `docs/implementation/backlog.md`.
1. MC-C1.5(c) contract fold: the first connection reads the ids this attempt's list did not return, and drops deleted ids from a left-over copy.
2. MC-C1.2 contract fold: the "no saved store and `S.state = refused`" branch.
3. MC-C1.9 contract fold: a completed check emits when it cleared a stored error.
4. `complete_check` and the notice reads run outside the lane (small impact today; C3 adds writes).
5. The test variable `pricer` in core's rules tests (H7 wording; rename to `asker`).

## Files written by this review
- `docs/implementation/handoffs/reviewer/handoff_C1_connection_and_sync_review_r1.md` (this file)
- `docs/implementation/narratives/C1_connection_and_sync.md`
- `docs/implementation/backlog_conversations/C1_connection_and_sync.md` (new)
- `docs/implementation/plans/C1_connection_and_sync.md`: the Tracker line (→ APPROVED) and one Review log entry

No production or test file was touched. No probes. No tool-recorded state. Nothing committed (the runner commits).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
