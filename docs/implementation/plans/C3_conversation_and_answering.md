---
phase: C3_conversation_and_answering
title: Conversation view and answering — opening a conversation shows it as in Pricing (the pricer's dark bubbles on the left, your light ones on the right, the highlighted quote with its field, photos, the reply line, "edited"); you answer with text, a reply to one message and published photos of that product; a reply shows at once as "Sending…", never twice, and "Couldn't send · Retry" when it fails; Mark answered for when the fix was made in the data
difficulty: high
projection: done (r0 2026-11-20 AMENDMENTS_REQUIRED, amendments applied by the orchestrator; no new round)
depends_on: [C2_header_panel_notifications]
blocked_on: none
---
# C3 — Conversation view and answering

**Tracker:** IMPLEMENTED (2026-11-20, claude opus implementer; 3 units, checkpoints 56f95bb · 3877fd6 · 9076272; rows: rules 4 unit tests (C1), suite c3 31/0, desktop c3_tests 1/0, UI c3 39/0; closing gate green: workspace 2071/0, clippy, fmt, typecheck, npm test 748/0)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `09B` §7 (Conversation), §8 by contract **MC-C2** (`docs/implementation/master_plan_conversations.md` §5). (U1) The outbox, the send with `clientMutationId`, retry, discard, Mark answered, the attachable photos, the view, behind commands. (U2) The conversation view. (U3) The composer and the photo picker. **Not here:** Open session and the visit bar, the quote chip as a link (C4: here the chip is plain and the header shows no Open session button); resolving, topics, blocking, new questions, editing or deleting messages (never, CR1 and 09B §8, §11).

## How to work this plan (flow.md §10, master_plan_conversations.md §3)
One runner phase of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `C3·C<n>(<row>)`; tests named after their row (`c3_c4b_<slug>`). Open details: the smallest option consistent with 09B §7, §8 and MC-C2, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_conversations/C3_conversation_and_answering.md`. **Never call Pricing or Shopify, never launch the built app** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Outbox, send, retry, discard, Mark answered, attachable photos, the view | 8 | 36 | **yes** |
| U2 | The conversation view | 7 | 20 | **yes** (TypeScript) |
| U3 | The composer and the photo picker | 6 | 19 | **yes** (TypeScript) |
| **Total** | | **21** | **75** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `docs/implementation/master_plan_conversations.md` §3, §4, **§5 MC-C2** (and MC-C1.1, .4, .6, .8 it leans on), §6 · book `09B` §3, §7 (Conversation), §8, §10, §12 (CV-3, CV-4, CV-7) · the package (read-only): `API.md` (`POST …/messages`, `PATCH …/cases/{id}`, the error table), `CHAT_UX.md` §2–§5, §7 with `images/02-thread.png`, `images/06-quote-and-reply.png` · the C1 plan (Unit 1 rows C1·C5, Unit 3) and the C2 plan (Unit 2).

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **A reply is saved on this Mac the moment you press send** and sent within a quarter of a second; if the app closes first, it is sent at the next start, once.
- **A failed reply is never sent again by itself;** it shows "Couldn't send" with Retry and Remove.
- **When a reply did arrive but its confirmation was lost,** the conversation shows the delivered message and the "Couldn't send · Retry" line until you press Retry (nothing is sent twice) or Remove.
- **Photos you can attach are the ones already published for that product:** gallery photos in gallery order, then condition photos; both halves of a split photo.
- **Mark answered is offered only on an open conversation;** replying to a resolved one re-opens it as answered, as Pricing does.
- **The conversation counts as read when it is on screen while the window is focused.**
- **A photo opens large inside Session Manager.**

## Unit 1 — Outbox, send, retry, discard, Mark answered, attachable photos, the view

**Unit id:** `C3·U1` · **Rows:** C3·C1–C8 · **Changes app files:** yes

### Goal
`ConversationsService::{view, reply, retry, discard, mark_answered, attachable}` and the outbox flush inside `tick`, by MC-C2; the seven C3 commands.

### Read first
Master plan §5 **MC-C2** (every item), MC-C1.1 · `crates/core/src/services/conversations.rs`, `crates/core/src/ports/conversations.rs`, `crates/infra/src/sqlite/conversations.rs` (as C2 left them; the `conversation_outbox` table exists since migration 019) · `crates/core/src/domain/publish.rs` (`PhotoLink`), `crates/core/src/ports/store.rs` (`photo_links_for_item`, `session_for_item`: how the existing store reads links; the service holds no `Store`, so it reaches them only through the new `ConversationStore` read of task 4), `crates/core/src/services/publish.rs` (the link roles, :1201, :1467: `"gallery"` and `attribute:<condition key>`), `crates/core/src/mechanisms/publish.rs` (the `ready` upload state, :48), `crates/core/src/domain/photo.rs` (preview paths, classification), `crates/infra/src/sqlite/store.rs` (:2359, the session-cover preview precedent) · `crates/testkit/src/fake_pricing.rs` (`drop_answer_next`, `fail_matching`, `timeout_next`, the scripted envelope :470–476, :793–801) · `crates/testkit/tests/suite/p13_publish_runs.rs` (how a test gets published photos with links).

### Files
Edited: `crates/core/src/{domain/conversation.rs, ports/conversations.rs, services/conversations.rs, mechanisms/conversation_rules.rs}`, `crates/infra/src/sqlite/conversations.rs`, `apps/desktop/src-tauri/src/{commands/conversations.rs, lib.rs}`, `apps/desktop/src/services/{bindings.ts (generated), conversations.ts}`, `crates/testkit/src/fake_pricing.rs` (the scripted helper `fail_matching_with`, task 7), `crates/testkit/tests/suite/main.rs`, `docs/guide/{02_architecture.md, 03_rules_and_principles.md}`. New: `crates/testkit/tests/suite/c3_conversations_answer.rs`.

### Core mechanisms (the only functions with unit tests)
`conversation_rules::checked_reply(input) -> Result<NewMessage>` (MC-C2.1: trimming, the 1–4000 rule in Unicode scalar values, the id length, at most 10 distinct media ids). If the filtering and ordering of task 4 is written as a pure `conversation_rules` function (**Delegated choices**), it joins this list with inline unit tests.

### Ordered tasks
1. `checked_reply`; `reply -> PendingView` (MC-C2.1, idempotent on `client_mutation_id`); the flush at the start of `tick` (MC-C2.2; where it sits in `tick`: **Notes**); `retry`, `discard` (MC-C2.3). Each change of an outbox row emits `ConversationsChanged` without notices. The stored `ReplyRef.excerpt` (projection L9) = the target message's first line (up to the first `\n`), at most 80 Unicode scalar values (the fake's rule, `fake_pricing.rs:652`, not a Pricing contract); the composer uses the same rule (U3 task 2). **Hidden and refused states** (projection L5, master plan MC-C2.1, .3, .6, .7): in `no_key`, `no_shop`, `store_unmatched` (the copy is hidden, C2's `visible()`), `view`, `reply`, `retry`, `attachable` and `mark_answered` return `conversations_off` (`data.reason` = the state, snake_case) and write nothing; in `refused` / `store_gone`, "the state's sentence" (MC-C2.1, .3, `off_reason`) = `describe` of that state's error, which is `S.last_error` (written by MC-C1.8 or the connect path). Core holds no Pricing-naming sentence: "Connect Pricing in Settings → Pricing" stays the frontend's.
2. `view(id) -> ConversationView {id, sku, product_title, topic, blocking, status, resolved_by, session_id, messages, pending: Vec<PendingView {client_mutation_id, text, reply_to, attachments: Vec<{media_id, preview_path: Option<String>}>, state, failure, created_at}>, can_send: bool, off_reason: Option<String>}` (MC-C2.6); a pending attachment's `preview_path` is the preview (task 4's rule) of the photo whose link holds that media id, when this Mac has it (through task 4's `ConversationStore` read).
3. `mark_answered` (MC-C2.7), in the lane. Its error carries three cases (projection L6): `Off(state)` → command code `conversations_off`, `data.reason` = the state, message = the state's sentence in `refused` / `store_gone` (task 1), empty in the hidden states, no `PATCH`; `Resolved` → `conversation_resolved`; any other `ProviderError` → the existing code `provider` with `describe(&e)` as message (as C1's `connect_error`, `commands/conversations.rs:88–97`); storage errors as today. `view`, `reply`, `retry` and `attachable` report their `conversations_off` through the same `Off(state)` case (the type's name: **Delegated choices**). A successful Mark answered that changes the copy emits one `ConversationsChanged` without notices (master plan §4 Event).
4. `attachable(id) -> Attachable {photos: Vec<AttachablePhoto {media_id, photo_id, slot, role, preview_path: Option<String>}>, reason: Option<AttachReason {NotOnThisMac, NothingPublished}>}`. **Reaching the links** (projection L2, road (a)): one new `ConversationStore` read, implemented in `crates/infra/src/sqlite/conversations.rs` (`SqliteStore` implements both traits), returns for a session its item's photo links, each with the linked photo's `session_id`, `classification`, `position`, `captured_at` and preview paths (`adjusted_preview_path`, `original_preview_path`); `App::build` and the testkit seam are not touched. **Filtering and ordering stay in core** (in the service or a pure `conversation_rules` function): a link is offered when it has an external id, upload state `ready` (so not `retired`), a gallery or condition role, and a photo that is still in that session and not `deleted`. **Roles** (projection L3; publish writes no other role, `services/publish.rs:1201–1203, 1467–1477`): gallery = `role == "gallery"`; condition = `role` starting with `"attribute:"` (the service does not hold the commerce adapter's condition key). **Preview** (projection L4): `preview_path` = `adjusted_preview_path`, else `original_preview_path` (the session-cover precedent, `infra/src/sqlite/store.rs:2359`); a split photo's `left` and `right` entries carry the same photo's preview. **Order**: gallery links first, then condition links; within each group by `(position, captured_at, photo id)` of the photo (condition photos "in position order", `services/publish.rs:592`), then `left` before `right`.
5. Commands `conversation_get`, `conversation_mark_read`, `conversation_reply {conversation_id, client_mutation_id, text, reply_to_message_id, attachment_media_ids}` (answers the `PendingView`), `conversation_retry`, `conversation_discard`, `conversation_mark_answered` (codes by task 3: `conversations_off`, `conversation_resolved`, `provider`), `conversation_attachable`; every C3 command maps `Off(state)` to `conversations_off` with `data.reason`; bindings; wrappers. `conversations.send` log line (outcome and conversation id only).
6. Guide: the outbox and the lane (`02_architecture.md`); CR1 and CR6 with `checked_reply`, the flush and the port's two writing methods (`03_rules_and_principles.md`).
7. Tests for the rows. `FakePricing` gains one scripted helper (projection L1): `fail_matching_with(path_part, n, status, code, message)` = the next `n` requests whose path contains `path_part` answer `status` with the package envelope (`ErrorEnvelope`, the adapter's own wire type) carrying `code` and `message`; `fail_matching` keeps its behaviour. Its test caller is C3·C4(d) (the message text is the fixture's choice; the package names no wording; API.md error table: 400 `invalid_attachment`).

### Criteria (8 criteria, 36 rows)
Boundary: `ConversationsService` on `TestApp::with_pricing()`, connected, driven by `tick` with `ManualClock` (outcome = the returned value, the `conversation_outbox` and copy rows, the fake's request record and the messages the fake holds, the collected events); `checked_reply` (pure); C3·C8(e) at the command layer (`c3_tests`).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C3·C1(a) | `checked_reply` | text "  Original glass.  " | text "Original glass." | 09B §8 (after trimming) |
| C3·C1(b) | same | text "   "; text of 4000 scalar values; text of 4001 | refused naming `text`; accepted; refused naming `text` | 09B §8 (1–4000); `IntegrationMessageCreate.text` (openapi) |
| C3·C1(c) | same | client id ""; client id of 81 characters; of 80 | refused naming `client_mutation_id`; refused naming `client_mutation_id`; accepted | `clientMutationId` ≤ 80 (openapi); MC-C2.1 |
| C3·C1(d) | same | 11 media ids; 10 media ids; 2 equal media ids | refused naming `attachment_media_ids`; accepted; refused naming `attachment_media_ids` | 09B §8 (at most 10) |
| C3·C2(a) | `reply` | a valid reply on an open conversation | returns the pending row (`PendingView`) in state `sending`; the fake recorded no `POST`; one `ConversationsChanged` without notices | 09B §8 (shows at once); MC-C2.1 |
| C3·C2(b) | `reply` twice | the same `client_mutation_id` (second call with another text) | one outbox row, with the first text; the second call returns it | CR6; MC-C2.1 |
| C3·C2(c) | `reply` | a conversation id not in the copy; a `reply_to_message_id` of another conversation | `not_found`; refused naming `reply_to_message_id`; no row | MC-C2.1 |
| C3·C3(a) | `reply`, `tick` | text "Original glass.", client id `m-1` | exactly one `POST …/messages` whose body has `clientMutationId` `m-1` and that text; the row is `sent` with the fake's message id; the conversation's messages end with that message (author `Me`); status `Answered` | 09B §8; MC-C2.2; CV-3 |
| C3·C3(b) | same | with a reply target and two attachable media ids | the request body carries `replyToMessageId` and the two `attachmentMediaIds` in order | 09B §8; CV-3 |
| C3·C3(c) | `tick` again, 10 s later | the fake logged `message.created` with `authorKind: service` for the reply | the conversation holds that message id exactly once; the emitted event has no notice and `chime` false | 09B §8 (the echo is recognised by its id); MC-C2.4; CV-3 |
| C3·C3(d) | `reply`, `tick` | a resolved conversation | after the send its status is `Answered` and `resolved_by` is none | 09B §8 (replying re-opens it); CHAT_UX §7 rule 3 |
| C3·C3(e) | `view` after (a) | — | the messages end with the sent message; `pending` is empty | MC-C2.6 |
| C3·C4(a) | `reply`, `tick`, then `tick`s over 10 minutes | `timeout_next(1)` | the row is `failed` with failure "Couldn't send"; the fake holds no message; exactly one `POST` was recorded in all | 09B §8; MC-C2.2 (never resent by itself) |
| C3·C4(b) | `retry`, `tick` | after (a) | a second `POST` with the same `clientMutationId`; the row is `sent`; the fake holds exactly one message | CR6; CV-4 |
| C3·C4(c) | `reply`, `tick`, `tick` (+10 s), `retry`, `tick` | `drop_answer_next(1)`: the fake stores the message, the answer is lost | after the second `tick`: the copy shows the message (from the re-read) and `view().pending` still has the row `failed`; after the retry: the row is `sent` with that message's id, the conversation holds that id once, the fake holds one message | MC-C2.5; invariant MC-C2 |
| C3·C4(d) | `reply`, `tick` | `fail_matching_with("/messages", 1, 400, "invalid_attachment", "Not a photo of this product.")`: the fake answers the `POST` 400 `invalid_attachment` with that message | `failed` with "Pricing did not accept this reply: Not a photo of this product." | 09B §8; MC-C2.2 |
| C3·C4(e) | same | the fake answers 404 on the `POST` | `failed` with "This conversation no longer exists" | MC-C2.2 |
| C3·C4(f) | same | the fake answers 401 on the `POST` | `failed` with "Pricing refused the key: check Settings → Pricing"; the connection state is `refused` | 09B §5.4; MC-C2.2 |
| C3·C4(g) | two `reply`s, `tick` | `timeout_next(1)` | both rows `failed` "Couldn't send"; exactly one `POST` was recorded | MC-C2.2 (the rest fail without a request) |
| C3·C5(a) | `reply`, reopen the app, `tick` | the app is dropped before any `tick` | the row survived as `sending`; the first `tick` sends it once with the same `clientMutationId`; the fake holds one message | 09B §8 (survives a restart); CR6 |
| C3·C5(b) | reopen the app, `tick`s | a `failed` row | still `failed`; no `POST` | MC-C2.2 |
| C3·C5(c) | `discard` | a `failed` row; then a `sending` row | the first is deleted and no request is made; the second is refused `invalid_value` and kept | MC-C2.3 |
| C3·C5(d) | `retry` | a `sent` row; a `sending` row | both unchanged; no extra `POST` at the next `tick` beyond the one for the `sending` row | MC-C2.3 |
| C3·C6(a) | `reply` | connection state `refused` (the copy still holds the conversation) | the row is created `failed` with the refused sentence; no request at the next `tick` | MC-C2.1; MC-C1.1 |
| C3·C6(b) | `view` | state `ready`; state `refused` | `can_send` true, `off_reason` none; `can_send` false, `off_reason` the refused sentence | 09B §5.4 |
| C3·C6(c) | `reply` | connection state `no_key` (the key removed; the copy still holds the conversation) | refused `conversations_off` with `data.reason` `no_key`; no outbox row written | 09B §4 (hidden states); MC-C2.1; MC-C1.1 |
| C3·C7(a) | `mark_answered` | an open conversation | one `PATCH` with body `{"status":"answered"}`; the copy's status is `Answered`; one `ConversationsChanged` | 09B §8; MC-C2.7 |
| C3·C7(b) | same | the pricer resolved it on the fake since the last check | the error `conversation_resolved`; the copy's status is `Resolved` | 09B §8 (409) |
| C3·C7(c) | same | `timeout_next(1)` | the error code `provider` with message "Could not reach Pricing"; the copy unchanged | MC-C2.7 |
| C3·C7(d) | a scenario: first connection, a question, `mark_read`, a reply with a photo, a failed reply and its retry, `mark_answered`, a discard | — | every request in the fake's record is a `GET`, a `POST …/messages` or a `PATCH` whose body is `{"status":"answered"}` | CR1; MC-C2.8 |
| C3·C7(e) | `mark_answered` | connection state `refused` (the copy still holds the conversation) | the error `conversations_off` with `data.reason` `refused`; the fake recorded no `PATCH`; the copy unchanged | MC-C2.7 (calls not allowed); MC-C1.1 |
| C3·C8(a) | `attachable` | the conversation's session has a published item: gallery photos 1 and 2 and one condition photo, each with a `ready` link and an external id (gallery role `gallery`, condition role `attribute:custom.damage_report`, the key `TestApp::with_pricing` sets, `test_app.rs:692–701`) | three photos: gallery 1, gallery 2, condition, each with its link's external id as `media_id`; gallery 1 has both an adjusted and an original preview path and its `preview_path` is the adjusted one, the others carry their original path; `reason` none | 09B §8 (published gallery and condition photos) |
| C3·C8(b) | same | plus: a photo with a link without external id, a `retired` link, a link whose photo is now `deleted`, an unpublished photo | none of the four is offered | 09B §8 |
| C3·C8(c) | same | a split photo with `left` and `right` links | two entries, `left` then `right` | master plan §6 |
| C3·C8(d) | same | the session has no published photo; a conversation not on this Mac | no photos with `reason` `NothingPublished`; no photos with `reason` `NotOnThisMac` | 09B §8 ("Publish first to attach photos"); CV-7 |
| C3·C8(e) | `conversation_reply`, then the next `tick` | a conversation not on this Mac, text only | the command answers the pending row; the `POST` is made and the row becomes `sent` | 09B §8 (text replies work for "Not on this Mac"); CV-7 |

### Delegated choices
Transaction method names (also the name of task 4's `ConversationStore` read) · whether `view` resolves pending previews by one query or per row · the name of the C3 error type of task 3 (like C1's `ConnectError`) · whether task 4's filtering and ordering is a pure `conversation_rules` function (then a **Core mechanism** with unit tests) or lives in the service · where a photo with no `position` or `captured_at` sorts within its group (no row depends on it).
- **What `reply` returns** (projection L7): `PendingView` (one frontend type for pending rows; the composer re-reads the view anyway).
- **Unknown ids** (projection L14): `retry` / `discard` of a `client_mutation_id` not in the outbox, and `view` / `attachable` / `mark_answered` of a conversation not in the copy, answer `not_found` (as C3·C2(c) does for `reply`).

### Notes
- Hazard (the heart of MC-C2): the `client_mutation_id` is made once, by the frontend, when send is pressed, and is the outbox row's primary key; nothing in the backend ever makes a new one for an existing row.
- Hazard: the flush and `mark_answered` run in the lane; `reply`, `retry`, `discard` and `view` never do and never call the provider.
- Hazard (H7): core does not look at what a media id looks like; Pricing decides whether an attachment is acceptable (400 `invalid_attachment` → the failure sentence).
- Hazard: a 5xx or a time-out on a `POST` is "unknown outcome"; treat it exactly like MC-C2.2's `Unreachable` (row `failed`), never as "not sent, try a fresh id".
- **The flush and MC-C1.7's schedule** (projection L15): in `tick`, the flush runs after the `calls_allowed` test and **before** the due test (today's `tick` returns early when no check is due, `services/conversations.rs:408–413`), so a reply is sent within a quarter of a second even when no check is due (C3·C3(a), C3·C4). An `Unreachable`, `BadReply`, `Rejected` or `NotFound` from `post_message` changes only outbox rows: no `failures` increment, no `next_check_at` change, no `last_error`. `Refused` / `StoreGone` apply MC-C1.8 as MC-C2.2 says.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::conversation_rules` · `cargo test -p psm-testkit --test suite c3_conversations_answer::` · `cargo test -p psm-testkit --test suite c1_conversations_sync::` · `cargo test -p psm-desktop` · `cargo clippy -p psm-core -p psm-infra -p psm-desktop -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — The conversation view

**Unit id:** `C3·U2` · **Rows:** C3·C9–C15 · **Changes app files:** yes (TypeScript)

### Goal
`ConversationView`, `MessageBubble`, `QuoteBlock` in the panel, replacing C2's placeholder line; the read mark.

### Read first
Book `09B` §6, §7 (Conversation), §8 (Resolved conversations) · `CHAT_UX.md` §2, §3 and the two screenshots · `apps/desktop/src/features/conversations/{ConversationsPanel.tsx, useConversations.ts}` · `apps/desktop/src/services/conversations.ts` · `apps/desktop/src/lib/format.ts` · `docs/intention/evidence/design_handoff.md` (tokens for dark and light surfaces, the accent colour).

### Files
New: `apps/desktop/src/features/conversations/{ConversationView.tsx, MessageBubble.tsx, QuoteBlock.tsx}`, `apps/desktop/src/features/conversations/__tests__/c3_conversation_view.test.tsx`. Edited: `apps/desktop/src/features/conversations/{ConversationsPanel.tsx, useConversations.ts}`, `apps/desktop/src/features/conversations/__tests__/c2_panel_list.test.tsx` (row C2·C15(c) now expects the view), `apps/desktop/src/styles/globals.css`.

### Core mechanisms
None.

### Ordered tasks
1. Selecting a row (or a toast's Open) shows `ConversationView` for `conversationGet(id)`, re-read on `conversations_changed`; the back arrow returns to the list. When `conversationGet` answers the code `conversations_off` (the hidden states, U1 task 1), the view returns to the list, which then shows its connect line (projection L5).
2. Header line: the status dot, the topic, `· Blocking` when blocking, the status in words (`Open`, `Answered`, or for resolved: `Resolved by the pricer` when `resolved_by` is `asker`, `Resolved when the product was activated` when `activation`, else `Resolved`), and `Not on this Mac` when `session_id` is null. (Open session is C4's.)
3. `MessageBubble`: author kind `asker` or `other` → a dark bubble on the left with the author's name and the time under it; `me` → a light bubble on the right labelled `You`; `· edited` after the time when `edited_at` is set; attachments as rounded thumbnails (`thumb_url`) above the text, a click opens `url` large in a dialog, an attachment without URLs shows "Photo not recorded"; the reply line `↩ <author name>: <excerpt>` above the text when `reply_to` is set.
4. `QuoteBlock`: an accent bar, a contrasting fill, the field chip (`field_label` in uppercase, plain text here), the quoted text between “ and ”.
5. Pending rows after the messages: light, right, dimmed with `Sending…`; failed ones with their failure text (the Retry and Remove actions are U3's).
6. Read mark: while the view is shown and `document.hasFocus()`, call `conversationMarkRead(id)` when it opens and whenever a re-read brings a newer message; when the window was not focused, call it at the next `focus` event.
7. Tests for the rows.

### Criteria (7 criteria, 20 rows)
Boundary: `ConversationsPanel` / `ConversationView` rendered in UI tests with `services/conversations.ts` and `services/events.ts` mocked (outcome = what is shown and the service calls).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C3·C9(a) | click a row | conversation `c1` | `conversationGet("c1")` called; the view replaces the list; the back arrow shows the list again | `[rule]` 09B §7 |
| C3·C9(b) | header line | topic Condition, blocking, status open, `session_id` 7 | "Condition", "· Blocking", "Open"; no "Not on this Mac" | `[rule]` 09B §7 |
| C3·C9(c) | same | not blocking, status answered, `session_id` null | "Answered", no "· Blocking", "Not on this Mac" | `[rule]` 09B §7; CV-7 |
| C3·C10(a) | `MessageBubble` | author `asker` "Elin Lindqvist", text, time | a bubble marked received (left, dark) with "Elin Lindqvist" and the time (`activityLabel(created_at)` for the test's fixed `now`) | `[rule]` 09B §7; CHAT_UX §2 |
| C3·C10(b) | same | author `me` | a bubble marked sent (right, light) labelled "You" | `[rule]` 09B §7 |
| C3·C10(c) | same | author `other` "Operations" | received, with "Operations" | `[rule]` 09B §7 |
| C3·C10(d) | same | `edited_at` set; then not set | "· edited" after the time; then absent | `[rule]` 09B §7 |
| C3·C11(a) | `QuoteBlock` | `{field: "condition", field_label: "Condition", text: "Possible desilvering at the lower edge."}` | a quote block with the chip "CONDITION" and the text “Possible desilvering at the lower edge.”; the chip is not a button | `[rule]` 09B §7; CV-1 |
| C3·C11(b) | `MessageBubble` | no quote | no quote block | `[rule]` 09B §7 |
| C3·C12(a) | same | two attachments with `thumb_url` and `url` | two thumbnails above the text with `thumb_url` as source | `[rule]` 09B §7 |
| C3·C12(b) | same | click the first thumbnail | a dialog shows an image whose source is its `url` | `[rule]` 09B §7 |
| C3·C12(c) | same | an attachment with both URLs null | "Photo not recorded" in its place | `[rule]` 09B §7 ("not recorded") |
| C3·C13(a) | same | `reply_to {author_name: "Elin", excerpt: "Is the glass original?"}` | the line "↩ Elin: Is the glass original?" above the text | `[rule]` 09B §7 |
| C3·C14(a) | open the view, document focused | — | `conversationMarkRead(id)` called once | `[rule]` 09B §6 |
| C3·C14(b) | open the view, document not focused; then a window `focus` event | — | not called before the event; called once after it | `[rule]` 09B §6 (while the window is focused) |
| C3·C14(c) | the view open and focused; a `conversations_changed` event; the re-read has one more message | — | the new message is shown; `conversationMarkRead` called again | `[rule]` 09B §6 |
| C3·C15(a) | header line | status resolved, `resolved_by` `asker` | "Resolved by the pricer" | `[rule]` 09B §8 |
| C3·C15(b) | same | `resolved_by` `activation`; then null | "Resolved when the product was activated"; then "Resolved" | `[rule]` 09B §8 |
| C3·C15(c) | the view | `pending` with one `sending` and one `failed` ("Couldn't send") | after the messages: a dimmed sent-side bubble with "Sending…", then one with "Couldn't send" | `[rule]` 09B §8; CHAT_UX §5 |
| C3·C15(d) | the view | no message and no pending | no bubble; the header line is shown | `[rule]` 09B §7 |

### Delegated choices
Exact colours and radii within the handoff's tokens (received dark, sent light, in both themes) · how received/sent is marked in the DOM for the tests (a `data-side` attribute) · the dialog used for the large photo.
- **Message time** (projection L11): `activityLabel(created_at)` from `lib/format.ts` (what C2's row uses, `ConversationRow.tsx:27`); the test asserts that function's output for a fixed `now`.

### Notes
- Hazard: mirror, do not copy: in Pricing's screenshots the pricer is "You" on the right; here the photographer is.
- Images come from the shop's public CDN addresses; nothing is downloaded or cached by the app.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/conversations` · `npm run typecheck`

## Unit 3 — The composer and the photo picker

**Unit id:** `C3·U3` · **Rows:** C3·C16–C21 · **Changes app files:** yes (TypeScript)

### Goal
`Composer` and `PhotoPicker` under the conversation view; Retry, Remove; Mark answered.

### Read first
Book `09B` §8 · `CHAT_UX.md` §4, §5 with `images/03-composer-menu.png` · `apps/desktop/src/features/conversations/{ConversationView.tsx, MessageBubble.tsx}` · `apps/desktop/src/services/conversations.ts` · `apps/desktop/src/services/sessions.ts` (`previewUrl`).

### Files
New: `apps/desktop/src/features/conversations/{Composer.tsx, PhotoPicker.tsx}`, `apps/desktop/src/features/conversations/__tests__/c3_composer.test.tsx`. Edited: `apps/desktop/src/features/conversations/{ConversationView.tsx, MessageBubble.tsx}`, `apps/desktop/src/styles/globals.css`, `docs/guide/01_product.md` (answering, in the conversations paragraph).

### Core mechanisms
None.

### Ordered tasks
1. `Composer`: a text box; **Enter** sends, **Shift+Enter** is a new line; nothing is sent when the trimmed text is empty or longer than 4000 characters (a counter appears from 3800); "characters" are counted as `[...text.trim()].length` (code points = the backend's Unicode scalar values, MC-C2.1; never `.length`, which counts UTF-16 units), for the limit and for the counter (projection L10); on send: `crypto.randomUUID()` → `conversationReply({conversation_id, client_mutation_id, text, reply_to_message_id, attachment_media_ids})`, then the box, the reply target and the chosen photos are cleared and the view is re-read. `can_send` false → the box is disabled with `off_reason`.
2. **Reply** on a message sets the reply target, shown above the box as `↩ <author>: <excerpt>` with ✕; the excerpt = the target message's first line, at most 80 code points (`[...line].slice(0, 80)`), the same rule as the backend's (U1 task 1, projection L9).
3. `conversationAttachable(id)` is read when the view opens and re-read with the view (projection L12); the photo button's state comes from its `reason`. The photo button opens `PhotoPicker` over that answer: thumbnails to tick, at most 10 (further ones cannot be ticked), the chosen ones shown above the box with ✕; `reason` `NothingPublished` → "Publish first to attach photos"; `NotOnThisMac` → the button is disabled.
4. A failed pending bubble offers **Retry** (`conversationRetry(id)`) and **Remove** (`conversationDiscard(id)`).
5. **Mark answered** in the header line of an `open` conversation (`conversationMarkAnswered(id)`); the error code `conversation_resolved` → toast "Pricing has already resolved this" and a re-read; other errors → a toast with their message. A resolved conversation shows the note "Replying re-opens it as answered" above the box.
6. Tests for the rows.

### Criteria (6 criteria, 19 rows)
Boundary: `ConversationView` with its composer rendered in UI tests with `services/conversations.ts` mocked and `crypto.randomUUID` stubbed (outcome = what is shown and the service calls with their arguments).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C3·C16(a) | the text box | type "Original glass.", press Enter | `conversationReply` called once with text "Original glass."; the box is empty | `[rule]` 09B §8 |
| C3·C16(b) | same | type "a", Shift+Enter, "b" | no call; the box holds "a\nb" | `[rule]` 09B §8 |
| C3·C16(c) | same | only spaces, Enter; then 4001 characters, Enter | no call in either case | `[rule]` 09B §8 |
| C3·C17(a) | send | `randomUUID` stubbed to `u-1` | the call's `client_mutation_id` is `u-1` | `[rule]` CR6; MC-C2.1 |
| C3·C17(b) | two sends | `randomUUID` gives `u-1` then `u-2` | the two calls carry `u-1` and `u-2` | `[rule]` 09B §8 (one id per press) |
| C3·C17(c) | send, the mock's view then has a `sending` row | — | a dimmed bubble with "Sending…" and the text is shown | `[rule]` 09B §8; CHAT_UX §5 |
| C3·C18(a) | a failed bubble with client id `u-1` | click **Retry** | `conversationRetry("u-1")` called once; no `conversationReply` call | `[rule]` CR6; CV-4 |
| C3·C18(b) | same | click **Remove** | `conversationDiscard("u-1")` called once | `[rule]` master plan §6 |
| C3·C19(a) | click **Reply** on the pricer's message `m9` | `m9` by "Elin Lindqvist" with text "Is the glass original?" | "↩ Elin Lindqvist: Is the glass original?" above the box with ✕ | `[rule]` 09B §8 |
| C3·C19(b) | then type and Enter | — | the call's `reply_to_message_id` is `m9`; the reply line is gone afterwards | `[rule]` 09B §8 |
| C3·C19(c) | click ✕ then send | — | the call's `reply_to_message_id` is null | `[rule]` 09B §8 |
| C3·C20(a) | the photo button | attachable: three photos | the picker shows three thumbnails; ticking two and sending calls `conversationReply` with their two `media_id`s in the order ticked | `[rule]` 09B §8; CV-3 |
| C3·C20(b) | same | twelve attachable photos; tick ten | the other two cannot be ticked | `[rule]` 09B §8 (at most 10) |
| C3·C20(c) | same | `reason` `NothingPublished` | "Publish first to attach photos"; no thumbnail | `[rule]` 09B §8 |
| C3·C20(d) | same | `conversationAttachable` answers `reason` `NotOnThisMac` (the view's `session_id` is not what disables the button) | the photo button is disabled; typing and Enter still calls `conversationReply` | `[rule]` 09B §8; CV-7 |
| C3·C21(a) | header line | status open | a **Mark answered** button; a click calls `conversationMarkAnswered(id)` once | `[rule]` 09B §8 |
| C3·C21(b) | same | status answered; status resolved | no **Mark answered** button in either | `[rule]` 09B §8 |
| C3·C21(c) | click **Mark answered** | the mock rejects with code `conversation_resolved` | a toast "Pricing has already resolved this"; `conversationGet` called again | `[rule]` 09B §8 |
| C3·C21(d) | the view | status resolved; then `can_send` false with `off_reason` | the note "Replying re-opens it as answered" and an enabled box; then a disabled box showing the reason | `[rule]` 09B §8, §5.4 |

### Delegated choices
The composer's look within the handoff (Pricing's rounded box with a round send button is the reference) · where the counter sits.
- **Trimming before the call** (projection L16): whether the composer trims the text before calling `conversationReply` is the implementer's (the backend trims, MC-C2.1; no row depends on it).

### Notes
- Hazard: the id is made at the press, before the call, and never again for that message: Retry calls `conversationRetry` with the row's id, it does not call `conversationReply`.
- Hazard: do not disable the box while a previous reply is `sending`; replies queue in order.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/conversations` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core --lib mechanisms::conversation_rules` · `cargo test -p psm-testkit --test suite c3_conversations_answer::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/conversations` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: one `client_mutation_id` per press, made in the composer, never regenerated in the backend or on Retry; no code path sends a `failed` row without `retry`; the only provider writes in the tree are the port's `post_message` and `mark_answered`; `reply`, `retry`, `discard`, `view` make no provider call. Notes go to `docs/implementation/backlog_conversations/C3_conversation_and_answering.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app with a test question, never by a session)
1. Open the conversation from the panel: the pricer's question is a dark bubble on the left with the quote block; the header count goes down.
2. Type an answer and press Enter: it shows at once as "Sending…", then as sent; Pricing shows it and the conversation reads "Answered". No chime for your own reply.
3. Press the photo button, tick two published photos, send: Pricing shows the photos.
4. Turn Wi-Fi off, send a reply: "Couldn't send". Turn Wi-Fi on, press Retry: Pricing shows exactly one message.
5. On another open question press **Mark answered**: it reads "Answered" in both apps.

## Review log

- **2026-11-20, orchestrator: projection r0 amendments applied** (`handoffs/reviewer/handoff_C3_conversation_and_answering_projection_r0.md`, AMENDMENTS_REQUIRED, 0 owner cards; master plan v1.3). L1 `crates/testkit/src/fake_pricing.rs` added to U1 Files and the scripted helper `fail_matching_with(path_part, n, status, code, message)` answering the package envelope (U1 task 7); C3·C4(d) uses it. L2 road (a): one new `ConversationStore` read in `crates/infra/src/sqlite/conversations.rs` (a session's item's links with each photo's session, classification, position, captured time, previews), filtering and ordering in core (U1 task 4; master plan §4 Ports). L3 gallery = `role == "gallery"`, condition = `role` starting with `"attribute:"`; Read-first citation fixed to `crates/core/src/services/publish.rs` (:1201, :1467), `mechanisms/publish.rs` kept for `ready` (:48). L4 `preview_path: Option<String>` = adjusted, else original; split halves share the preview; order `(position, captured_at, photo id)` then `left` before `right`; C3·C8(a) asserts the adjusted path wins. L5 hidden states `no_key` / `no_shop` / `store_unmatched` → `conversations_off` (`data.reason`), nothing written; `refused` / `store_gone` sentence = `describe` of the state's error = `S.last_error` (U1 task 1; master plan MC-C2.1, .3, .6, .7); U2 task 1 returns to the list on `conversations_off`; row **C3·C6(c)** added. L6 Mark answered's error cases `Off(state)` → `conversations_off`, `Resolved` → `conversation_resolved`, other → `provider` with `describe` (U1 tasks 3, 5); C3·C7(c) names the code; row **C3·C7(e)** added; master plan §4 Event names Mark answered. L7 delegated: `reply` returns `PendingView`. L8 C3·C2(a) "the fake recorded no `POST`". L9 excerpt = the target's first line, at most 80 scalar values, in core and the composer (U1 task 1, U3 task 2); C3·C19(a) asserts "↩ Elin Lindqvist: Is the glass original?". L10 composer counts `[...text.trim()].length` (U3 task 1). L11 delegated: `activityLabel(created_at)` (U2; C3·C10(a)). L12 `conversationAttachable` read with the view; the photo button follows its `reason` (U3 task 3; C3·C20(d)). L13 C3·C1(c), (d) name the field in every refusal. L14 delegated: `not_found` throughout. L15 the flush runs after `calls_allowed` and before the due test; non-`Refused`/`StoreGone` `post_message` errors change only outbox rows (U1 Notes). L16 delegated: trimming in the composer. Counts: U1 8 criteria / 36 rows; total 21 / 75 (recounted from the four plan files: 12 units, 73 criteria, 297 rows). Outcomes otherwise unchanged, so no new projection round (`projection: done`).
- **2026-11-20, implementer (claude opus), implement r1.** Built the three units in order with a checkpoint after each
  (U1 `56f95bb`, U2 `3877fd6`, U3 `9076272`); closing gate run once, green. **U1:** `conversation_rules::{checked_reply,
  reply_excerpt}`; `ConversationsService::{view, reply, retry, discard, mark_answered, attachable}` and the flush at the
  start of `tick` (after the `ready` test, before the due test); `ConversationError {Off {state, message}, Resolved,
  Provider, Storage}`; `ConversationStore::{outbox_accept, outbox_row, outbox_sent, outbox_move, outbox_discard,
  session_links}`; the seven commands with DTOs and wrappers; `FakePricing::fail_matching_with`; guide 02 and 03.
  **U2:** `ConversationView`, `MessageBubble` (+ `PendingBubble`, `ReplyLine`), `QuoteBlock`, the read mark; C2·C15(c)
  now expects the view. **U3:** `Composer`, `PhotoPicker`, Reply / Retry / Remove, Mark answered; guide 01.
  **Assumptions** (smallest option, 09B §7–§8, MC-C2): (1) a `Refused` / `StoreGone` answer to a send fails that row
  **and the later `sending` rows of the same flush** with the state's sentence, without a request (MC-C2.2 names only
  the row; the rows after it cannot be sent in that state, and MC-C2.1 makes new replies `failed` there too);
  (2) `discard` is allowed in every state (MC-C2.1 does not list it among the `conversations_off` methods; it is local);
  (3) `reply` validates first, then returns an existing row for the same id before checking the conversation;
  (4) the attachable filter and order live in the service (not a `conversation_rules` function, so no extra unit
  tests); a missing position or capture time sorts last in its group; `session_links` reads only `shopify` links like
  `photo_links_for_item`; (5) the outbox is ordered by `created_at` then insertion order (`rowid`), so two replies of
  the same millisecond leave in the order they were pressed (was `client_mutation_id`); (6) `mark_answered` on a 409
  keeps the stored `resolved_by` (the next pull fills it); a re-read that answers 404 deletes the conversation;
  (7) `retry` returns the row as a `PendingView`; `conversation_mark_answered` and `conversation_discard` return
  nothing; DTO `reason` / `role` / `slot` / `state` are snake_case strings (`not_on_this_mac`, `nothing_published`);
  (8) the composer trims the text before the call and clears the box at the press (the call follows); a failed call
  shows a toast; Reply is offered on every message; the photo button opens the picker, which says "Publish first to
  attach photos" when nothing is published; the photo button is disabled when `can_send` is false. **Deviations:**
  none from the rows. `useConversations.ts` (listed as edited in U2) needed no change. **Observations:** the inline
  desktop test `p50_tests::c8a_support_and_secret_commands_have_their_wire_shapes` failed once during U1 because the
  OS handed it a random port inside its own passive range 50000–50009 (pre-existing flaky test, not touched; it passed
  alone and in the closing gate). No test, session or tool called Pricing or Shopify; the app was never launched.

