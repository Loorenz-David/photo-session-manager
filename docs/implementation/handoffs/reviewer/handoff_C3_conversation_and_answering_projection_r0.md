---
plan: C3_conversation_and_answering
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-20
actor: Claude Opus 5.5 (pipeline-projectionist)
---
# Projection of C3 — Conversation view and answering (round 0)

The plan for answering conversations is sound at its heart. The "never send a reply twice" design is fully
determined, and the fake Pricing server already behaves the way the real Pricing server documents: a resent reply
returns the same message, a lost answer still stores it, a resolved case refuses Mark answered. Before the implementer
starts, the plan needs a handful of paragraph-sized fixes. One test case cannot be built with today's fake server.
The plan does not say how the conversations service reaches the published photos it must offer. Which preview image
and which link role count as "condition photo" are left open. And it does not say what the view does when Pricing is
not connected at all. Nothing here needs you: every gap can be closed from the book and the code, and the
orchestrator folds the amendments into the plan before the implementer prompt is compiled.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner: every ledger row is a plan gap or a delegation the orchestrator can settle from book 09B,
`master_plan_conversations.md` and the code.

## Decision ledger

Classification: **plan gap** → amendment proposed (the home artifact is named); **free choice** → explicit
delegation proposed. No intention gap was found (09B answers every semantic question the plan raises; where it is
silent, the home is the master plan's contract §5, a planning artifact).

| # | Decision point (where the implementer would have to stop and choose) | Class | Proposed routing |
|---|---|---|---|
| L1 | **C3·C4(d) fixture cannot be produced.** The row needs the fake to answer the `POST` with 400 `invalid_attachment`, message "Not a photo of this product.". `FakePricing::fail_matching` always answers `envelope(status, code_for(status), "Scripted failure.")` (`crates/testkit/src/fake_pricing.rs:470-476, 793-801`; `code_for(400)` = `bad_request`, :110-120). The fake's `POST` route never validates media ids (:615-695). `crates/testkit/src/fake_pricing.rs` is **not** in U1's Files. Written as-is, the test would either fail or be bent to assert "Pricing did not accept this reply: Scripted failure." | plan gap (rule 17 producibility) | Plan U1 **Files**: add `crates/testkit/src/fake_pricing.rs` (edited). Plan U1 task 7 / Notes: add one scripted helper, e.g. `fail_matching_with(path_part, n, status, code, message)`, answering the package envelope (`ErrorEnvelope`, the adapter's own wire type). Grounding: API.md error table line 28 (400 `invalid_attachment` = "attachment isn't a photo of this product"); the adapter maps any enveloped non-409 4xx on a write to `Rejected {code, message}` (`adapter.rs:104-117`, unit test :978-991) and `describe` gives "Pricing did not accept this reply: {message}" (:372-374). The message text itself is the fixture's choice (the package names no wording). |
| L2 | **How `attachable` (and `view`'s pending `preview_path`) reach photos and links.** `ConversationsService` holds only `Arc<dyn ConversationStore>` (`services/conversations.rs:134-144`), with no `Store`. The plan's Read-first points at `Store::photo_links_for_item`, `session_for_item`, `photos_in_session` (`ports/store.rs:341-342, 439`), which the service cannot call. Two roads: (a) a new `ConversationStore` method implemented in `crates/infra/src/sqlite/conversations.rs` (in Files; `SqliteStore` implements both traits); (b) add `Arc<dyn Store>` to the constructor, which edits `App::build` in `crates/integrations/src/lib.rs` and the testkit seam. Neither file is in Files. | plan gap | Plan U1 task 4: name road (a). One `ConversationStore` method returns, for a session, its item's links with each linked photo's `session_id`, `classification`, `position`, `captured_at`, preview paths. Filtering and ordering stay in core (a pure function in `conversation_rules`, or in the service; if pure and listed, it joins **Core mechanisms**). Add it to master plan §4 Ports ("its methods are the transactions §5 names" plus this read). |
| L3 | **Which link role is "condition".** Publish writes exactly two role families: `"gallery"` and `format!("attribute:{}", condition_key)` (`services/publish.rs:1201-1203, 1467-1477`). The key comes from the commerce adapter's `condition_photos_key()` (`ports/commerce.rs:62`), which the conversations service does not hold. The plan says "a gallery or condition role". Its Read-first cites `mechanisms/publish.rs` for "the link roles", but that file holds only `"ready"` (:48); the roles live in `services/publish.rs`. | plan gap | Plan U1 task 4: gallery = `role == "gallery"`; condition = `role` starting with `"attribute:"` (publish writes no other attribute role). Fix the Read-first citation to `crates/core/src/services/publish.rs` (link roles, :1201, :1467). The test fixture uses `attribute:custom.damage_report` (`TestApp::with_pricing` sets that key, `test_app.rs:692-701`). |
| L4 | **Which preview, its type, split halves, ordering keys.** `Photo` has `original_preview_path`, `adjusted_preview_path`, `thumbnail_path`, each `Option` (`domain/photo.rs:12-14`). The plan's `AttachablePhoto.preview_path` is untyped, and C3·C8(a) asserts "its preview path", so there is no single exact outcome. A split photo has one photo row (no per-slot preview). "Main order" / "condition order" have no key. | plan gap | Plan U1 tasks 2 and 4: `preview_path: Option<String>` = `adjusted_preview_path`, else `original_preview_path` (the session-cover precedent, `infra/src/sqlite/store.rs:2359`); `left` and `right` carry the same photo's preview. Order within each group: `(position, captured_at, photo id)` (condition photos "in position order", `services/publish.rs:592`), then `left` before `right`. C3·C8(a): the fixture sets both an adjusted and an original path on one photo, so the row asserts which wins. |
| L5 | **The "state's sentence" and `off_reason` in the hidden states.** MC-C2.1 / .3 say a reply or retry made while calls are not allowed becomes `failed` "with the state's sentence", and C3 adds `off_reason`. Sentences exist only through `describe(ProviderError)`: refused → `describe(Refused)`, store_gone → `describe(StoreGone)`. For `no_key`, `no_shop`, `store_unmatched` there is no `ProviderError`, and core may not hold a Pricing-naming sentence (H7; "Connect Pricing in Settings → Pricing" is frontend-held, §4 Words). C2's `visible()` already hides the copy in those three states (`services/conversations.rs:777-784`), and 09B §4 says the panel then shows only "Connect Pricing in Settings → Pricing". | plan gap (master plan §5) | Amend master plan MC-C2.1, .3, .6 (and plan U1 tasks 1–4): in `no_key` / `no_shop` / `store_unmatched`, `view`, `reply`, `retry` and `attachable` return `conversations_off` (`data.reason` = the state) and write nothing. In `refused` / `store_gone`, the sentence is `describe` of that state's error (= `S.last_error`, written by MC-C1.8 / the connect path). The U2 view, on `conversations_off`, returns to the list (the panel then shows its connect line). Optional row: C3·C6(c), state `no_key` → `reply` refused `conversations_off`, no row written. |
| L6 | **Mark answered's error surface.** The codes `conversation_resolved` and `conversations_off` are named in master plan §4, but neither exists in `CommandError`'s mapping (`commands/items.rs:78-79` region; `connect_error` uses code `provider`, `commands/conversations.rs:88-97`). The service's return type for MC-C2.7 is unstated. "Other errors → the error's sentence" has no code. **No row covers MC-C2.7's first branch** (calls not allowed → `conversations_off`, no `PATCH`). C3·C7(a) asserts one `ConversationsChanged`, but §4 Event's emission list does not name Mark answered. | plan gap | Plan U1 task 3/5: `mark_answered` returns an error carrying three cases: `Off(state)` → `conversations_off` (message = the L5 sentence, or empty in hidden states); `Resolved` → `conversation_resolved`; other `ProviderError` → the existing code `provider` with `describe(&e)` as message (as `connect_error`). Add row **C3·C7(e)**: state `refused` → `conversations_off` with `data.reason` `refused`; the fake recorded no `PATCH`. Master plan §4 Event: add "when Mark answered changes the copy". (This makes U1 8 criteria / 35 rows.) |
| L7 | **What `reply` returns.** "Returns the pending row" (C3·C2(a), C3·C8(e)): `OutboxMessage` (registry) or `PendingView` (C3's view shape, with attachment previews)? | free choice | Delegate explicitly in U1 **Delegated choices**; recommend `PendingView` (one frontend type for pending rows; the composer re-reads the view anyway). |
| L8 | **C3·C2(a) "the fake recorded no request yet"** is false for any connected fixture: the first connection's `GET`s are recorded (`FakePricing::requests`). | plan gap (wording) | Reword: "the fake recorded no `POST`" (or: requests cleared before `reply`, none after). |
| L9 | **The reply excerpt.** MC-C2.1 stores a `ReplyRef {message_id, author_name, excerpt}` for the pending row, but no excerpt rule exists (Pricing computes its own for delivered messages, API.md line 89). The composer's reply line has the same gap. C3·C19(a) asserts "↩ Elin Lindqvist: …" with "…" as an unexplained placeholder. | plan gap (C3·C19(a)) + free choice (rule) | Plan: the excerpt = the target message's first line, at most 80 Unicode scalar values (the fake's rule, `fake_pricing.rs:652`; not a Pricing contract), in core and in the composer. C3·C19(a): use a short target text and assert the exact line, e.g. "↩ Elin Lindqvist: Is the glass original?". |
| L10 | **Composer length counting.** Task U3·1 says "longer than 4000 characters". In TypeScript `.length` counts UTF-16 units, while the backend counts Unicode scalar values (MC-C2.1). A 2100-emoji reply would be blocked by the composer but accepted by the backend. Rows use ASCII, so no row catches it. | free choice → decide | Plan U3 task 1: count `[...text.trim()].length` (code points = scalar values for valid strings), also for the counter from 3800. |
| L11 | **Message time format** (C3·C10(a) "the time"). `lib/format.ts` offers `activityLabel` and `dateTimeLabel`. | free choice | Delegate in U2 **Delegated choices**, recommending `activityLabel(created_at)` (what C2's row uses, `ConversationRow.tsx:27`). The test asserts that function's output for a fixed `now`. |
| L12 | **Where the photo button's "disabled" comes from** (C3·C20(d)): `attachable().reason = NotOnThisMac` (requires fetching `conversationAttachable` when the view opens) or `view.session_id == null`. | plan gap (minor) | Plan U3 task 3: `conversationAttachable(id)` is read when the view opens (and re-read with the view), and the button state comes from its `reason`. The row's fixture then has one cause. |
| L13 | **C3·C1(c), C3·C1(d) "refused"** without the field, unlike (b). | plan gap (wording) | Reword each "refused" to "refused naming `client_mutation_id`" / "… `attachment_media_ids`". |
| L14 | **Unknown ids.** `retry` / `discard` of a `client_mutation_id` not in the outbox; `view` / `attachable` / `mark_answered` of a conversation not in the copy. | free choice | Delegate, recommending `not_found` throughout (as C3·C2(c) does for `reply`). |
| L15 | **The flush versus MC-C1.7's schedule.** Today's `tick` returns early when no check is due (`services/conversations.rs:408-413`). C3·C3(a) and C3·C4(*) rely on the flush running in a `tick` where no check is due ("sent within a quarter of a second"). MC-C2.2 also does not say whether an `Unreachable` on a `POST` touches `failures`, `next_check_at` or `last_error`. | plan gap (minor) | Plan U1 Notes: the flush runs after `calls_allowed` and **before** the due test. An `Unreachable`/`BadReply`/`Rejected`/`NotFound` from `post_message` changes only outbox rows (no backoff, no `last_error`). `Refused`/`StoreGone` apply MC-C1.8 as MC-C2.2 already says. |
| L16 | **Whether the composer trims before calling `conversationReply`.** The backend trims (MC-C2.1). | free choice | Delegate (no row depends on it). |

## Reality checks

- **Paths.** Every file in U1–U3 Files exists or is marked new. Verified present: the four core files,
  `infra/src/sqlite/conversations.rs`, `commands/conversations.rs`, `src-tauri/src/lib.rs`, `services/bindings.ts`,
  `services/conversations.ts`, `services/events.ts`, `tests/suite/main.rs`, `docs/guide/01/02/03_*.md`,
  `styles/globals.css`, `c2_panel_list.test.tsx`, `design_handoff.md`, `backlog_conversations/`. **Missing from Files:**
  `crates/testkit/src/fake_pricing.rs` (L1).
- **Citations.** `mechanisms/publish.rs` does not hold "the link roles" (L3). Everything else resolves and says what
  the plan claims. Checked: `ports/store.rs` (`photo_links_for_item` :439, `session_for_item` :341, `upsert_photo_link`
  :427); `domain/publish.rs` `PhotoLink` (:242, `slot`, `external_id`, `upload_state`); fake helpers
  `drop_answer_next` :489, `fail_matching` :470, `timeout_next` :479; p13's direct link creation (:252); migration 019's
  `conversation_outbox` (:53); `ConversationStore::outbox_insert` / `outbox` exist (`ports/conversations.rs:88-90`).
  The package's `CHAT_UX.md` §2–§5, §7 and `images/02-thread.png`, `03-composer-menu.png`, `06-quote-and-reply.png`
  exist.
- **Prior-phase outputs, verified in code.** `ConversationsLane`, `in_lane`, `calls_allowed` (service :43-53,
  :395-457); `delete_conversation` already deletes outbox rows not `sent` (infra :406-425, as MC-C1.6 says);
  `connect_store(reset)` empties the outbox (:318-324); `apply` returns `changed` by full comparison (:397), so C3·C3(c)'s
  emission exists because the fake's `POST` bumps the case's `updatedAt` (`fake_pricing.rs:688` `touch`) and the
  re-read differs; `TestApp::kill_and_restart` keeps the fake and the fake Keychain (C1 tests restart with Pricing,
  `c1_conversations_sync.rs:496`); C2's `c2_tests` harness gives C3·C8(e) a command-layer path (`lib.rs:729-790`).
- **Not yet existing, which the plan must create** (correctly in scope): the commands, the codes `conversations_off` and
  `conversation_resolved` (L6), the outbox state-change and "sent + append" transactions (delegated names).
- **Counts** derived from the tables: U1 8 criteria / 34 rows, U2 7 / 20, U3 6 / 19, total 21 / 73. These match the
  plan header and master plan §2. L6 adds one row (U1 35, total 74; master plan §2 totals then 296).

## Criteria decidability (two questions per row)

**Q1** = the test can be written now with one exact outcome at the named boundary. **Q2** = the fixture is a shape the
installed dependencies (here: the real adapter over `FakePricing`) can produce.

| Rows | Q1 | Q2 | Note |
|---|---|---|---|
| C3·C1(a), (b) | yes | n/a (pure) | |
| C3·C1(c), (d) | yes after L13 | n/a | |
| C3·C2(a) | yes after L8 | yes | |
| C3·C2(b), (c) | yes | yes | |
| C3·C3(a)–(e) | yes | **yes**: `POST` route `fake_pricing.rs:615-695` stores the message, sets `answered` (also from `resolved`, matching API.md :112-113), logs `message.created` `authorKind: service` (:689-693); the adapter sends `clientMutationId` (`adapter.rs:327-332`) and maps `isMe` → `Me` (:147-148) | (c) relies on the `updatedAt` bump (Reality checks) |
| C3·C4(a), (g) | yes | **yes**: `timeout_next` records the request, then `Error::Timeout` before routing (:763-787), so nothing is stored | L15 for schedule side effects |
| C3·C4(b) | yes | **yes**: replay of `(case, clientMutationId)` returns the stored message with 201 (:633-637; API.md :110-111) | |
| C3·C4(c) | yes | **yes**: `drop_answer_next` routes then returns `Error::Timeout` (:813-816); the stored message's `message.created` reaches the next pull | needs the flush to be the tick's first request (L15) |
| **C3·C4(d)** | yes | **no**: the fake cannot answer 400 `invalid_attachment` with a chosen message (L1) | blocks compilation until L1 is routed |
| C3·C4(e) | yes | yes: `fail_matching("/messages",1,404)` → 404 → `NotFound` on a write (`adapter.rs:107`) → "This conversation no longer exists" (:370) | |
| C3·C4(f) | yes | yes: `fail_matching(…,401)` → `Refused` (:106) | |
| C3·C5(a)–(d) | yes | yes | (a) via `kill_and_restart` |
| C3·C6(a), (b) | yes | yes: 401 on `/events` → `refused` (C1 path) with the copy kept | hidden states: L5 |
| C3·C7(a) | yes | yes: `PATCH` route records body `{"status":"answered"}` (`FakeRequest.body`), answers the case (:592-614) | emission: L6 |
| C3·C7(b) | yes | yes: `FakePricing::resolve` (:420) then `PATCH` → 409 `case_resolved` (:607-609) → `Resolved` (`adapter.rs:110`) | |
| C3·C7(c) | yes after L6 (code) | yes: `Unreachable` → "Could not reach Pricing" (:361) | |
| C3·C7(d) | yes | yes: every request is recorded with method, path, body (:763-770) | |
| C3·C8(a)–(c) | **no until L3, L4** (preview field, role predicate, ordering key) | yes: links via `Store::upsert_photo_link` (p13 :252) | |
| C3·C8(d), (e) | yes | yes | |
| C3·C9–C3·C15 (all 20) | yes, except C3·C10(a) "the time" (L11) | n/a (mocked services) | C3·C15(c)'s pending order follows MC-C2.6 |
| C3·C16–C3·C18, C3·C19(b), (c), C3·C20(a)–(c), C3·C21 | yes | n/a | C3·C21(c): `ServiceError.code` exists (`services/sessions.ts:20-30`) |
| C3·C19(a) | **no until L9** ("…" is not an exact outcome) | n/a | |
| C3·C20(d) | yes after L12 | n/a | |

## Trace verification (both directions)

- **Row → ledger.** Every row's trace cell resolves and supports the row. MC-C2.1 → C1, C2, C6; .2 → C3(a), (b), C4,
  C5(a), (b); .3 → C5(c), (d); .4 → C3(c); .5 → C4(c); .6 → C3(e), C15(c); .7 → C7(a)–(c); .8 / CR1 → C7(d); CR6 → C2(b),
  C4(b), C5(a), C17(a), C18(a); CV-1 (chip plain here) → C11(a); CV-3 → C3, C20(a); CV-4 → C4(b), C18(a); CV-7 → C8(d),
  (e), C9(c), C20(d); master plan §6 split / Remove → C8(c), C18(b). `[rule]` appears only on U2/U3 rendering rows, as
  §3 allows.
- **Ledger → row.** Every MC-C2 item the phase claims has a row, except **MC-C2.7's `conversations_off` branch**
  (L6 adds C3·C7(e)). Invariant MC-C2 is served by C3·C4(b), (c), C3·C5(a) (same id), C3·C4(a) / C5(b) (no unasked resend),
  C3·C3(c) / C4(c) (id once) and C3·C3(a) / C4(c) (`sent` carries the id).

## Write perimeter (this session)
- Written: this file only.
- Not touched: the plan, the master plans, the intention, code, tests, tracker rows, archgraph. No commit.

## Appendix — non-authoritative
None. The skeleton was discarded on purpose. The ledger rows above carry everything the amendments need.
