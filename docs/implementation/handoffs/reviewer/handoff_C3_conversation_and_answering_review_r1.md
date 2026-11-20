---
plan: C3_conversation_and_answering
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-20
actor: opus (pipeline-reviewer)
---
# Review: C3_conversation_and_answering, round 1

**Summary.** The production code is correct on all 75 rows and matches MC-C2, 09B §7–§8 and H1–H8. One test is the
problem. The test for C3·C16(b) ("Shift+Enter is a new line") cannot fail if Shift+Enter sends the reply. That makes it
a B2 under flow.md §5 (blocking finding B2-1). The fix touches only one test file. Nothing needs the owner.

**Gate.** The plan's Tracker line read `IMPLEMENTED`. The checkpoints `56f95bb`, `3877fd6`, `9076272` and the close
`c4e17d0` are all in `git log`. The intention is RATIFIED.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Evidence identity
- **Tree:** HEAD `75599e5` (the review-prompt commit), `git status --porcelain` clean at start. The code tree is the same as `9076272`.
- **Runs:** each Review-mode command once, one cargo process at a time, never piped into `head`:

| Command | Result |
|---|---|
| `cargo test -p psm-core --lib mechanisms::conversation_rules` | 21 passed / 0 failed |
| `cargo test -p psm-testkit --test suite c3_conversations_answer::` | 31 / 0 |
| `cargo test -p psm-desktop` | 80 / 0 (the P50 port flake did not recur) |
| `cargo clippy -p psm-core -p psm-infra -p psm-desktop -p psm-testkit --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npx vitest run src/features/conversations` | 5 files, 67 / 0 |
| `npm run typecheck` | exit 0 |

- The implementer's full-suite stamp stands: workspace 2071/0 and npm 748/0 on the clean `9076272`.

## Verified correct
**Perimeter.** `git diff --stat 56f95bb^ 9076272` lists 25 paths. They are exactly the handoff's declared production, test and guide files. The
close commit `c4e17d0` adds only the handoff and the plan's Tracker line and Review log. The U2 Files list names
`useConversations.ts`. It was not changed, and the handoff says so.

**Core (U1), read against MC-C2.**
- **`checked_reply`** (MC-C2.1): trims; checks 1–4000 Unicode scalar values (`chars().count()`); checks an id of 1–80 characters; allows at most 10 media ids with none repeated (a `HashSet`). A refusal is `invalid_value` naming the field.
- **`reply_excerpt`** (projection L9): the first line, at most 80 scalar values. The composer's `replyExcerpt` uses the same rule in code points.
- **`reply`:**
  - Hidden states: `Off` before anything is written.
  - Idempotence: through `outbox_accept`, a single transaction that returns the existing row.
  - Checks: the conversation is in the copy (else `not_found`); the reply target is a message of that conversation (else `invalid_value("reply_to_message_id")`).
  - Refused / store gone: a new row is `failed` with the state's sentence (`S.last_error`).
  - Emits one `ConversationsChanged` without notices, only when it inserted a row.
- **Flush** (MC-C2.2, L15):
  - Placement: after `calls_allowed`, before the due test.
  - Each row: `post_message` in `in_lane` (calls re-checked for the same store), and a re-check that the row is still `sending`.
  - Outcomes, as the contract's table: `Ok` → `outbox_sent` (row `sent` with the server id, the message appended unless its id is already there, `answered`, `resolved_by` null, `last_activity_at`); `Unreachable`/`BadReply` → this row and every later row of the flush `failed` "Couldn't send" without a request; `Rejected` / `NotFound` → that row with `describe`.
  - `Refused`/`StoreGone` → MC-C1.8 through `record_error` (plus assumption 1).
  - No `failures`, schedule or `last_error` change on the other errors.
  - After a refused flush, the due check that follows is stopped by `in_lane`'s `calls_allowed` re-check, so no request is made.
- **`retry` / `discard`** (MC-C2.3):
  - `retry` moves `failed` → `sending` (or `failed` again with the sentence) with the same id. Any other state is unchanged.
  - `discard` deletes only a `failed` row (`DELETE … AND state='failed'`). Any other state → `invalid_value`.
- **`view`** (MC-C2.6): the copy's messages, then the `sending`/`failed` rows in outbox order. `can_send` ⇔ `ready`; `off_reason` = the state's sentence.
- **`mark_answered`** (MC-C2.7, L6):
  - `Off` unless `ready`: message empty in the hidden states, the sentence in `refused`/`store_gone`.
  - The lane is taken, and the store is re-checked inside it.
  - `Ok` → the MC-C1.4 replace (`store.apply` keeps `read_mark`).
  - 409 → `conversation` is re-read and applied, then `Resolved`.
  - Other errors → `Provider`; `Refused`/`StoreGone` also record the state.
  - Emits when the copy or the state changed.
- **`attachable`** (09B §8, L2–L4):
  - Source: `session_links` (infra, `integration='shopify'`, the session's item).
  - Filter: an external id, upload state `ready`, role `gallery` or `attribute:*`, a photo still in that session and not `deleted`.
  - Order: gallery before condition, then position, capture time, photo id (missing values last), then `single`/`left`/`right`.
  - Preview: adjusted, else original.
  - Reasons: `NotOnThisMac` when the SKU matches no session; `NothingPublished` when the list is empty.
- **Review-mode reading checks:**
  - **One id per press:** `crypto.randomUUID()` in `Composer.send`. The backend never makes an id: `reply` takes the input's id, the flush sends the stored one, `retry` keeps it. Retry in the UI calls `conversationRetry(p.client_mutation_id)` and never `conversationReply`.
  - **No path sends a `failed` row without `retry`:** the flush filters `sending` and re-checks inside the lane.
  - **The only provider writes** are `post_message` (called only by the flush) and `mark_answered` (called only by `ConversationsService::mark_answered`). The port's write surface is unchanged since C1.
  - **No provider call:** `reply`, `retry`, `discard` and `view` call only the pure `describe`.

**Commands.** All seven are thin: a DTO mapping plus `conversation_error`, which maps `Off` → `conversations_off` with
`data.reason` = `state.as_str()` (snake_case), `Resolved` → `conversation_resolved`, `Provider` → `provider` with
`describe`. They are registered in `specta_builder`, which is the runtime `invoke_handler`.

**UI (U2, U3).**
- **View:**
  - The view replaces the list. `conversations_off` returns to the list.
  - Header words come from `statusWords`; Mark answered shows only on `open`.
  - Bubbles: `data-side`, "You", `activityLabel`, "· edited", thumbnails above the text, a dialog with `url`, "Photo not recorded", the reply line.
  - The quote chip is plain text in uppercase.
  - Pending bubbles come after the messages, dimmed.
- **Read mark:** called on open when focused, on a newer message, and at the next `focus` when the view opened unfocused.
- **Composer:**
  - Enter sends; Shift+Enter does not (`!event.shiftKey`, IME guarded).
  - Counts `[...text.trim()].length`, with the 4000 limit and the counter from 3800.
  - Reply target with ✕; picker at most 10, in the order ticked.
  - "Publish first to attach photos"; the photo button is disabled on `not_on_this_mac`.
  - Box disabled with `off_reason` when `can_send` is false; the resolved note above the box.
  - The box is not locked while earlier replies are sending.

**Rows.** 74 of the 75 rows have a test at their boundary that asserts the exact outcome and fails on a wrong one.
Judged by reading. Two UI tests were also run with a changed input in a scratch test file, deleted afterwards:
- The C3·C16(c) shape with a valid text goes red, so "nothing is sent" can fail.
- The C3·C14(b) shape with the window focused goes red, so "not called before focus" can fail.
Two notes on how the fixtures discriminate:
- C3·C21(c)'s rejection message equals the toast. That case is equivalent: the adapter's own `describe(Resolved)` is the same sentence (`adapter.rs:371`).
- C3·C1(b) uses `é`, which is 2 bytes in UTF-8, so a byte-length rule would fail it.

**Assumptions.** None contradicts 09B or MC-C2. Notes on how they read:
- Assumption 1 (Refused/StoreGone fail the later rows of the flush) is consistent with MC-C2.1 (replies are created `failed` in those states).
- Assumption 2 (`discard` in every state) is allowed: MC-C2.1 does not list `discard` among the `conversations_off` methods.
- Assumptions 3–9 are within the Delegated choices.

**Hard constraints.**
- H1: a write needs the photographer's press of send or Mark answered, and is recorded in the outbox.
- H2, H3, H5: not touched.
- H4: the client id survives a restart (row C3·C5(a)) and is reused by retry.
- H6: conversations stay optional.
- H7: media ids are opaque strings, as the plan and C1's domain define them. Role strings follow publish's own link roles. `integration='shopify'` sits in infra, as `photo_links_for_item` does.
- H8: the commands are thin.
- **Secrets / live services:** the log line carries the outcome and the conversation id only. Every test runs over `FakePricing`.

## Findings
### B2-1 (blocking) · route: production (test-only change)
- **What is wrong.** `c3_c16b_shift_enter_is_a_new_line` (`apps/desktop/src/features/conversations/__tests__/c3_composer.test.tsx:165`) cannot fail when Shift+Enter sends:
  - `Composer.send` calls `conversationReply` inside `Promise.resolve().then(…)`, a microtask.
  - The test asserts `not.toHaveBeenCalled()` synchronously, right after `fireEvent.keyDown`, so the microtask has not run yet.
  - Before asserting the box's value, it re-types the box to `a\nb`. That hides the box being cleared by a send.
- **Evidence.** A scratch copy of this test with `enter(false)` (plain Enter, which really sends in the current code) passed both assertions. The scratch file was deleted, and `git status` was clean afterwards.
- **Authority.** flow.md §5 B2 ("its test cannot fail on a wrong outcome"); plan row C3·C16(b) (outcome "no call; the box holds a\nb"); 09B §8 ("Shift+Enter is a new line").
- **Why it matters.** A Shift+Enter that sends delivers a half-written reply to the pricer. Pricing messages cannot be edited or deleted (09B §8, §11).
- **Correction.** After `enter(true)`, flush a microtask (`await Promise.resolve()`, as C3·C16(c) already does). Then assert that `conversationReply` was not called **and** that the box still holds `a`. Then type `a\nb` and assert the value as today. Production code is correct and must not change.
- **Files the fix may touch:** `apps/desktop/src/features/conversations/__tests__/c3_composer.test.tsx` only.

## Notes written to backlog
In `docs/implementation/backlog_conversations/C3_conversation_and_answering.md` (the location named by master plan §3
and the plan's Review mode):
1. The composer clears the typed text at the press, so a refused `conversation_reply` loses the answer. Restore it in the `.catch`.
2. MC-C2 gap: a `sending` row stays "Sending…" with no failure when a check (not a flush) turns the state `refused`/`store_gone`. After a reconnect it leaves by itself.
3. Contract folds into MC-C2: implementer assumptions 1, 2, 5 and 6.
4. Stale guide line: `02_architecture.md` says every C3 call is `conversations_off` in the hidden states, but `discard` is not.

## Write perimeter of this session
- `docs/implementation/plans/C3_conversation_and_answering.md`: the Tracker line and one Review log entry.
- `docs/implementation/backlog_conversations/C3_conversation_and_answering.md` (new).
- This handoff.
- Probes: two scratch test files, `apps/desktop/src/features/conversations/__tests__/zz_review_probe.test.tsx` and
  `zz_review_probe2.test.tsx`. Each was created, run once and deleted, and `git status --porcelain` was clean after each.
- No production file was touched. No database, archgraph or other tool state was changed. Nothing was committed.

## For the narrative at approval (plain words, for the re-reviewer)
Choices made for the owner, each one sentence:
- If Pricing refuses the key in the middle of sending, every reply still waiting in that batch also shows the refusal and needs Retry.
- Remove works even when Pricing is not connected.
- Two replies sent within the same millisecond leave in the order they were pressed.
- After Mark answered finds the question already resolved, the "Resolved by …" words appear at the next check.
- Retry shows the reply again as "Sending…".
- The composer empties as soon as you press send.
- Reply is offered on every message.
- The photo button is disabled when replies can't be sent.

Plus the planning points already listed in the plan.

## Lessons for the plans
- A UI row whose outcome is "nothing is called", for a component that calls its service in a microtask, needs the test to flush microtasks before asserting. Plans with "no call" rows could say this in their Notes (C4 has several "calls neither" rows under MC-C3).
