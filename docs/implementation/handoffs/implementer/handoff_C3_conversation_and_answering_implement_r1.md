---
plan: C3_conversation_and_answering
role: implement
round: 1
state: DONE
date: 2026-11-20
actor: claude (opus, pipeline-implementer)
---
# Handoff — C3_conversation_and_answering, implement r1

**Gate:** the intention is RATIFIED (2026-10-24; 09B RATIFIED 2026-11-20). `depends_on: [C2_header_panel_notifications]`, and that plan's Tracker line reads APPROVED (2026-11-20). This plan's Tracker line read NOT_STARTED. I worked the three units in order and made a `CHECKPOINT (not approved):` commit after each unit's test commands passed. Then I ran the closing gate once, at the end.

## What was built
**For the photographer:**
- **Opening a conversation** from the panel, or from a toast's Open, now shows the real conversation in place of C2's placeholder:
  - The header line has the status dot, the topic, "· Blocking", the status in words, and "Not on this Mac" when no session on this Mac has the product. The status reads Open, Answered, Resolved by the pricer, Resolved when the product was activated, or Resolved.
  - The pricer's messages (and those of other Pricing accounts) are dark bubbles on the left, with the author's name and the time underneath.
  - The photographer's own messages are light bubbles on the right, labelled "You".
  - A message can carry a highlighted quote block with its field chip (for example "CONDITION"), photo thumbnails that open large inside Session Manager ("Photo not recorded" when a photo has no address), the reply line "↩ Elin: …", and "· edited".
  - The back arrow returns to the list.
- **Read marks:** a conversation counts as read while it is on screen and the window is focused. It is marked when it opens, again when a newer message arrives, and at the next window focus if it opened in the background.
- **Answering:**
  - Enter sends and Shift+Enter starts a new line. A reply holds at most 4000 characters, and a counter appears from 3800.
  - **Reply** on a message shows "↩ Name: first line" above the box, with ✕ to cancel.
  - The photo button opens the product's **published** photos: gallery photos in gallery order, then condition photos, with both halves of a split photo. Up to ten can be ticked, and they travel in the order ticked.
  - When the product has nothing published, the picker says "Publish first to attach photos". For a conversation "Not on this Mac" the photo button is disabled, but text replies still work.
- **A reply shows at once** as a dimmed "Sending…" bubble. It is saved on this Mac at the press and sent within a quarter of a second, even after a restart, and never twice.
  - A failure shows "Couldn't send", or Pricing's own words ("Pricing did not accept this reply: …", "This conversation no longer exists"), with **Retry** and **Remove**.
  - A failed reply is never sent again by itself.
  - If Pricing stored the reply but the confirmation was lost, the delivered message appears, and the "Couldn't send" line stays until Retry (which confirms it without a second message) or Remove.
- **Mark answered** appears in the header of an open conversation. If Pricing had already resolved it, a toast says "Pricing has already resolved this" and the view reloads. A resolved conversation shows "Replying re-opens it as answered" above the box.
- When the key is refused or the store is gone, the box is disabled and shows the reason. When there is no key, no shop, or the store doesn't match, opening a conversation returns to the list, which asks to connect.

**In the code:**
- **Core rules:** `conversation_rules::checked_reply` (trim, 1–4000 scalar values, id 1–80, at most 10 distinct media ids, refusals name the field) and `reply_excerpt`.
- **Service:** `ConversationsService::{view, reply, retry, discard, mark_answered, attachable}`, and the outbox flush at the start of `tick` (after the `ready` test, before the due test).
  - A send's outcome changes only outbox rows and the copy (no failure count, no schedule, no `last_error`), except 401 and store gone, which apply MC-C1.8.
  - The error type is `ConversationError {Off {state, message}, Resolved, Provider, Storage}`.
  - Every outbox change emits `ConversationsChanged` without notices.
- **Storage:** `ConversationStore::{outbox_accept, outbox_row, outbox_sent, outbox_move, outbox_discard, session_links}` in `crates/infra/src/sqlite/conversations.rs`.
- **Commands:** `conversation_get`, `conversation_mark_read`, `conversation_reply`, `conversation_retry`, `conversation_discard`, `conversation_mark_answered`, `conversation_attachable`, with codes `conversations_off` (`data.reason`), `conversation_resolved` and `provider`. Bindings are regenerated, and the wrappers are in `services/conversations.ts`.
- **Test fake:** `FakePricing::fail_matching_with(path_part, n, status, code, message)`.
- **UI:** `ConversationView`, `MessageBubble` (with `PendingBubble` and `ReplyLine`), `QuoteBlock`, `Composer`, `PhotoPicker`.
- **Guide:** updated `01_product.md` (answering), `02_architecture.md` (the outbox and the lane) and `03_rules_and_principles.md` (CR1, CR6).

## Tests → criterion rows
**Unit 1** — the C1 rows are unit tests in `crates/core/src/mechanisms/conversation_rules.rs`; the rest are in `crates/testkit/tests/suite/c3_conversations_answer.rs` unless noted.

| Row | Test | Asserts the row's exact outcome |
|---|---|---|
| C3·C1(a) | `c3_c1a_the_text_is_trimmed` | yes |
| C3·C1(b) | `c3_c1b_the_text_is_1_to_4000_scalar_values` (é × 4000 / 4001) | yes |
| C3·C1(c) | `c3_c1c_the_client_id_is_1_to_80_characters` | yes |
| C3·C1(d) | `c3_c1d_at_most_ten_distinct_media_ids` | yes |
| C3·C2(a) | `c3_c2a_a_reply_is_pending_at_once_without_a_request` | yes |
| C3·C2(b) | `c3_c2b_the_same_client_id_twice_keeps_one_row_with_the_first_text` | yes |
| C3·C2(c) | `c3_c2c_an_unknown_conversation_or_a_foreign_reply_target_is_refused` | yes |
| C3·C3(a) | `c3_c3a_the_next_tick_sends_once_with_the_client_id` | yes |
| C3·C3(b) | `c3_c3b_the_reply_target_and_the_photos_travel_in_order` (media ids from `attachable`) | yes |
| C3·C3(c) | `c3_c3c_the_echo_of_the_reply_is_recognised_by_its_id` | yes |
| C3·C3(d) | `c3_c3d_replying_to_a_resolved_conversation_reopens_it_as_answered` | yes |
| C3·C3(e) | `c3_c3e_the_view_after_the_send_ends_with_the_message_and_no_pending` | yes |
| C3·C4(a) | `c3_c4a_a_timeout_fails_the_reply_and_it_is_never_resent_by_itself` (60 checks, 10 min) | yes |
| C3·C4(b) | `c3_c4b_retry_sends_the_same_client_id_once_more` | yes |
| C3·C4(c) | `c3_c4c_a_lost_answer_shows_the_message_and_the_failed_row_until_retry` | yes |
| C3·C4(d) | `c3_c4d_a_refused_attachment_fails_with_the_providers_words` | yes |
| C3·C4(e) | `c3_c4e_a_404_fails_as_no_longer_existing` | yes |
| C3·C4(f) | `c3_c4f_a_401_fails_with_the_refused_sentence_and_refuses_the_connection` | yes |
| C3·C4(g) | `c3_c4g_after_a_timeout_the_other_replies_fail_without_a_request` | yes |
| C3·C5(a) | `c3_c5a_a_reply_survives_a_restart_and_is_sent_once` | yes |
| C3·C5(b) | `c3_c5b_a_failed_reply_stays_failed_after_a_restart` | yes |
| C3·C5(c) | `c3_c5c_discard_removes_a_failed_reply_and_refuses_a_sending_one` | yes |
| C3·C5(d) | `c3_c5d_retry_leaves_a_sent_or_sending_reply_unchanged` | yes |
| C3·C6(a) | `c3_c6a_a_reply_while_refused_is_created_failed_and_never_sent` | yes |
| C3·C6(b) | `c3_c6b_the_view_says_whether_a_reply_can_be_sent` | yes |
| C3·C6(c) | `c3_c6c_without_a_key_a_reply_is_refused_as_off_and_nothing_is_written` (service `Off {state: NoKey}`; the `data.reason` mapping is in `conversation_error`) | yes, at the service boundary the row names |
| C3·C7(a) | `c3_c7a_mark_answered_patches_once_and_updates_the_copy` | yes |
| C3·C7(b) | `c3_c7b_mark_answered_on_a_conversation_resolved_meanwhile` | yes |
| C3·C7(c) | `c3_c7c_mark_answered_when_pricing_is_unreachable` (`Provider` → command code `provider`; message asserted through `describe`) | yes |
| C3·C7(d) | `c3_c7d_the_only_writes_are_replies_and_mark_answered` (the whole record since build) | yes |
| C3·C7(e) | `c3_c7e_mark_answered_while_refused_is_off_without_a_request` | yes |
| C3·C8(a) | `c3_c8a_published_gallery_and_condition_photos_are_offered` | yes |
| C3·C8(b) | `c3_c8b_unpublished_retired_and_deleted_photos_are_not_offered` | yes |
| C3·C8(c) | `c3_c8c_both_halves_of_a_split_photo_are_offered_left_then_right` | yes |
| C3·C8(d) | `c3_c8d_nothing_published_or_not_on_this_mac` | yes |
| C3·C8(e) | `apps/desktop/src-tauri/src/lib.rs` `c3_tests::c3_c8e_a_text_reply_works_for_a_conversation_not_on_this_mac` | yes |

**Unit 2** — all in `apps/desktop/src/features/conversations/__tests__/c3_conversation_view.test.tsx`; each test is named `c3_<row>_…`:
- C3·C9(a), (b), (c)
- C3·C10(a) (time = `activityLabel(created_at, NOW)` with a fixed `Date`), (b), (c), (d)
- C3·C11(a), (b)
- C3·C12(a), (b), (c)
- C3·C13(a)
- C3·C14(a), (b), (c)
- C3·C15(a), (b), (c), (d)

`c2_panel_list.test.tsx` row C2·C15(c) now expects the view (`c2_c15c_a_row_opens_its_conversation_and_the_back_arrow_returns`).

**Unit 3** — all in `__tests__/c3_composer.test.tsx`; each test is named `c3_<row>_…`:
- C3·C16(a), (b), (c)
- C3·C17(a), (b), (c)
- C3·C18(a), (b)
- C3·C19(a), (b), (c)
- C3·C20(a), (b), (c), (d)
- C3·C21(a), (b), (c), (d)

C3·C20(b) asserts that the 11th and 12th photos are disabled and that the send carries exactly the ten ticked ids. jsdom changes a disabled checkbox when it is clicked directly, so the test does not assert a direct click on a disabled box.

Every test in the three new test files maps to a row; there are no orphan tests.

## Assumptions
1. **Send refused (401) or store gone during a send:** the send fails that reply **and the later "sending" replies of the same flush**, all with the state's sentence and without a request. MC-C2.2 names only the one row, but no later row can be sent in that state, and MC-C2.1 already creates new replies failed there.
2. **Remove (`discard`) works in every connection state.** MC-C2.1 doesn't list it among the `conversations_off` methods, and it is local only.
3. **Order of checks in `reply`:** `reply` checks the input first, then returns an existing row with the same id before it checks the conversation.
4. **Attachable photos:**
   - The filter and the order are a private function of the service, not a `conversation_rules` function, so they have no unit tests of their own; rows C3·C8 cover them.
   - A photo with no position or capture time sorts last in its group.
   - `session_links` reads only the `shopify` integration's links, as `photo_links_for_item` does.
5. **Outbox order:** `created_at`, then insertion order (`rowid`), so two replies made in the same millisecond leave in the order they were pressed. The C1 order was `created_at`, then `client_mutation_id`.
6. **Mark answered after a 409:** the stored `resolved_by` is kept, and the next pull fills it in. If the re-read answers 404, the conversation is deleted, as in the pull.
7. **Return shapes:**
   - `retry` answers the row as a `PendingView`; `conversation_mark_answered` and `conversation_discard` answer nothing.
   - DTO fields `reason`, `role`, `slot` and `state` are snake_case strings (`not_on_this_mac`, `nothing_published`, `gallery`, `condition`, `left`, `right`, `sending`, `failed`, `sent`).
8. **Composer behaviour:**
   - It trims the text before the call and clears the box, the reply target and the chosen photos at the press; the call follows.
   - A refused call shows a toast with its message.
   - **Reply** is offered on every message.
   - The photo button opens the picker, which says "Publish first to attach photos" when nothing is published.
   - The photo button is disabled when `can_send` is false or the answer is `not_on_this_mac`.
9. **Previews:** a pending reply's photos show their previews when this Mac has them, read from the conversation's session.

## Deviations
None from the plan's rows or contracts. `useConversations.ts`, which U2 lists as edited, needed no change.

## Files changed
**Production:**
- `crates/core/src/{domain/conversation.rs, ports/conversations.rs, services/conversations.rs, mechanisms/conversation_rules.rs}`
- `crates/infra/src/sqlite/conversations.rs`
- `apps/desktop/src-tauri/src/{commands/conversations.rs, lib.rs}`
- `apps/desktop/src/services/{bindings.ts (generated), conversations.ts}`
- `apps/desktop/src/features/conversations/{ConversationView.tsx (new), MessageBubble.tsx (new), QuoteBlock.tsx (new), Composer.tsx (new), PhotoPicker.tsx (new), ConversationsPanel.tsx}`
- `apps/desktop/src/styles/globals.css`

**Tests:**
- `crates/testkit/src/fake_pricing.rs` (`fail_matching_with`)
- `crates/testkit/tests/suite/{main.rs, c3_conversations_answer.rs (new)}`
- `apps/desktop/src/features/conversations/__tests__/{c3_conversation_view.test.tsx (new), c3_composer.test.tsx (new), c2_panel_list.test.tsx}`
- the `c3_tests` module in `apps/desktop/src-tauri/src/lib.rs`

**Docs:**
- `docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md}`
- `docs/implementation/plans/C3_conversation_and_answering.md` (Tracker line and Review log only)
- this handoff

No mutation probes, no files outside this perimeter, no tool-recorded state.

## Checkpoint SHA
- U1: `56f95bb` — CHECKPOINT (not approved): C3·U1 outbox, send, retry, discard, Mark answered, attachable photos, the view
- U2: `3877fd6` — CHECKPOINT (not approved): C3·U2 the conversation view
- U3: `9076272` — CHECKPOINT (not approved): C3·U3 the composer and the photo picker
- Close: the commit that follows this handoff (Tracker line, Review log, handoff), named in the final message.

## Stamp
Run once on the clean tree at `9076272`, one command at a time:

| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0; 2071 passed, 0 failed (suite program 1150/0) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `npm test` | exit 0; 115 files, 748 tests passed |

**Unit commands while working:**
- `mechanisms::conversation_rules`: 21/0
- `c3_conversations_answer::`: 31/0
- `c1_conversations_sync::`: 44/0
- `psm-desktop`: 80 tests, 79 passed and 1 failed. The failure was the pre-existing flaky `p50_tests::c8a_support_and_secret_commands_have_their_wire_shapes`: the operating system handed it a random port inside its own passive range 50000–50009. It passed alone, and again in the closing gate. Not touched.
- Unit clippy: clean
- `npx vitest run src/features/conversations`: 67/0

## Manual try-out
On the installed app, with a test question the owner creates in Pricing. Sessions never launch the app.
1. Open the conversations panel and click the question. It shows as a dark bubble on the left with the quote block, and the header count goes down.
2. Type an answer and press Enter. It shows at once as "Sending…", then as your light bubble on the right. Pricing shows it, and the conversation reads "Answered". There is no chime for your own reply.
3. Press the photo button, tick two published photos and send. Pricing shows the two photos.
4. Turn Wi-Fi off and send a reply: it shows "Couldn't send". Turn Wi-Fi on and press Retry: Pricing shows exactly one message.
5. On another open question, press **Mark answered**. It reads "Answered" in both apps.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
