---
phase: P15_publish_ui
title: Publish UI — confirmation popover, progress, failure and retry, batch queue, notifications (frontend)
difficulty: normal
projection: no
depends_on: [P14_publish_update, P11_form_ui]
---
# P15 — Publish UI: confirmation popover, progress, failure and retry, batch queue, notifications (frontend)

## Goal
Surface publishing in the UI from the design: the Publish popover as the confirmation surface (thumbnails, counts,
notes, "Publish N images" → `publish_confirm` with the exact summary shown), live progress on rows and in the header,
failure (sticky toast with Retry, popover with the reason, per-photo badges), batch "Review & publish N" → form queue
("Confirm & next", "Publish N confirmed"), the notifications drawer with Retry/Open, and the header action switching
from Export to Publish when an integration is configured. **Not in this phase:** settings/health screens (P16).
Thin frontend phase — **refine at prompt time**.

## Read first
master plan §6.7 (P13 commands, `publish_progress`), §6.9 · book `01` (H1), `14`, `07` §Publish, §Failure and retry,
`04` S-G, S-H, S-J, S-K · contracts `07A` MC-10 (the popover is the confirmation surface; summary text) · evidence
`design_handoff.md` §Publish, §Sessions list (status pills) · `claude_design/NOTES.md` §Flows (Publish one, Publish many,
Publish steps, Failure), §Screens 8; `COMPONENTS.md` §Sessions page (Failure popover, Batch bar), §Workspace (Publish
popover, Failure notice), §Product details form (Batch queue strip, Footer); screens `desktop/06–08`, `17–18`, `30`, `35`.

## Files expected to change
`apps/desktop/src/features/publish/{PublishPopover.tsx, PublishProgress.tsx, FailurePopover.tsx, PhotoUploadBadge.tsx,
BatchQueue.tsx, NotificationsDrawer.tsx, usePublish.ts, __tests__/p15_publish_ui.test.tsx}`, `apps/desktop/src/services/publish.ts`,
`apps/desktop/src/features/{shell/Header.tsx (Publish action, bell count), sessions/{BatchBar.tsx, StatusPill.tsx (Retry
click), SessionsPage.tsx, SessionRow.tsx (row Retry, Ready · Publish)}, workspace/{PhotoTile.tsx (badges), WorkspacePage.tsx},
form/FormDialog.tsx (queue strip, footer buttons)}`.

Backend read additions (task 0, added 2026-10-26 after implement r1 BLOCKED):
`crates/core/src/domain/{session.rs (`LatestPublish` on the session record and view), publish.rs (`Notification.item_id`)}`,
`crates/core/src/ports/store.rs`, `crates/core/src/services/sessions.rs` (view carries photo `upload_state`),
`crates/infra/src/sqlite/store.rs`, `apps/desktop/src-tauri/src/commands/{sessions.rs, publish.rs}` (DTO fields),
`apps/desktop/src/services/bindings.ts` (regenerated), `crates/testkit/tests/p15_publish_backend.rs` (new),
`crates/testkit/src/fake_commerce.rs` (only if a failure needs scripting). No migration: every value is already stored.

## Core mechanisms (unit tests allowed)
None.

## Ordered tasks
0. **Backend reads (thin; no new publish logic except (c)).** Verified 2026-10-26 against the tree at `e3848bd`:
   (a) `publish_items.id`, `.error`, `.failed_step` are stored (07 §Failure and retry: "marks the item `failed` with
   the reason") and the session-list query already reads the latest confirmed item's step/failed_step/progress
   (`sqlite/store.rs` ~l.981) but drops its id and error. Expose `publish: Option<LatestPublish{item_id, step,
   failed_step, error, uploaded, total}>` on `SessionRowResponse` and on `session_get`'s `SessionResponse` (same
   latest-confirmed-item rule). (b) `photo_links.upload_state` is stored per photo/item/role (07A MC-11) but
   `session_get` photos omit it. Add `upload_state: Option<String>` to `PhotoResponse`: the state of the photo's link
   for the configured integration (role `main` or `attribute:<condition key>` per its classification); `null` when
   no link or no integration. (c) **Genuinely missing in core:** the store inserts only `published` notifications
   (`complete_publish_item`), and notification rows carry no publish item id, so the drawer cannot list a failure or
   retry it (ch12: "Notifications drawer lists publish history with Retry and Open"; ch04 S-J). Minimal addition: in
   the same transaction as `set_publish_step(.., Failed, ..)`, insert one notification `kind = 'failed'`, text
   `Publish failed · <error>`, with the item's `session_id`/`run_id`; `Notification` and `notifications_list` gain
   `item_id` (join `publish_items` on `run_id` + `session_id`, one item per session per run; no migration). The bell
   count = sessions whose latest item is `failed` (from (a)), not a count of notification rows.
1. `usePublish`: `publish_preview(ids)` → popover data; `publish_confirm(ids, summary)`; subscribe to
   `publish_progress`; `publish_retry(item)`; `notifications_list`.
2. Popover per `COMPONENTS.md`; progress labels per MC-7 (`Publishing n%` from `session_changed`/`publish_progress`);
   failure toast (sticky, Retry) and popover; badges from `session_get` photo `upload_state` for the integration.
3. Batch: selection bar → form queue mode (strip, "Confirm & next" marks the item confirmed, "Publish N confirmed"
   → `publish_confirm` for confirmed sessions only).
4. Notifications drawer (bell count = failed items).
5. Tests.

## Criteria (7 criteria, 14 rows; 13 `[rule]` rows)
| Row | Boundary | Input (services mocked) | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `PublishPopover` render | `publish_preview` → 3 main, 1 condition, 2 unreviewed, 1 removal, `create_item`, details 3/8, summary text T | main thumbnails in order, condition thumbnails, "Product details 3/8", the text T rendered verbatim, button "Publish 4 images" | `[rule]` ch07 §Publish (confirmation surface) |
| C1(b) | click "Publish 4 images" | — | `publish_confirm([S], T)` called with the **same** T the popover showed | M17 (summary recorded = text shown) |
| C2(a) | `publish_progress{step: upload, uploaded: 2, total: 4}` then `session_changed` with label `Publishing(42)` | — | row and header read "Publishing 42%"; progress bar value 42 | `[rule]` ch07 (live progress) |
| C3(a) | `WorkspacePage` with `session_get` mocked → session `publish {item_id: 7, step: failed, failed_step: upload, error: "Upload timed out"}` | — | sticky toast with "Retry"; clicking calls `publish_retry(7)`; failure popover shows "Upload timed out" and "Nothing was overwritten" | `[rule]` ch04 S-J |
| C3(b) | `SessionsPage` row with `session_list` mocked → label `DescriptionFailed`, `publish.item_id = 9` | — | row reads "Description failed"; its Retry calls `publish_retry(9)` | `[rule]` ch07 (list rows) |
| C4(a–c) | `WorkspacePage` photo tile with `session_get` mocked → photo `upload_state` `ready` / `none` with session `publish.step = failed` / `uploaded` | — | badge "On Shopify" / "Not uploaded" / "Uploading" (3 rows) | `[rule]` ch04 S-J (badges) |
| C5(a) | tick rows A, B, C → "Review & publish 3" → form queue; "Confirm & next" on A and B; "Publish 2 confirmed" | — | `publish_confirm([A, B], T_AB)` — C excluded | `[rule]` ch04 S-H |
| C5(b) | queue strip | after confirming A | A shows ✓ and "n/N filled" | `[rule]` evidence §Form (batch queue strip) |
| C6(a) | `NotificationsDrawer` | `notifications_list` → one `published`, one `failed` `{item_id: 11, session_id: 4}` | two rows; the failed one has Retry → `publish_retry(11)` and Open → navigates to `#/session/4` | `[rule]` ch12 §Notifications drawer |
| C7(a) | `session_list` / `session_get` (Rust, `TestApp` + `FakeCommerce`) | one confirmed item whose upload fails with fixture message M; and a session never published | row and session `publish = {item_id: <that item>, step: failed, failed_step: upload, error: M}`; never published → `null` | `[rule]` ch07 §Failure and retry |
| C7(b) | `session_get` photos | run where photo P1 uploaded to `ready` and P2's upload failed; and no integration configured | P1 `upload_state = "ready"`, P2 `"none"` or `null` as stored; no integration → both `null` | `[rule]` ch04 S-J (badges), 07A MC-11 |
| C7(c) | `notifications_list` | the failure of C7(a), then `publish_retry` succeeding | first: one `failed` row, text `Publish failed · M`, `item_id` = the item; after retry: that row plus one `published` row, both with the same `item_id`, newest first | `[rule]` ch12 §Notifications drawer |

## Delegated choices
Popover vs. sheet on narrow widths, throttling of progress re-renders, exact iconography, where the "Open" of a
notification lands (workspace).

## Manual try-out
1. Open a Ready session with Shopify configured; click Publish: the popover lists the photos, counts and notes; click
   "Publish N images". The row shows "Publishing n%", then "Published"; the bell stays quiet.
2. Switch the Mac's Wi-Fi off and publish another: "Publish failed" with a sticky toast; Wi-Fi on → Retry → finishes.
3. Tick three rows → "Review & publish 3" → confirm two → "Publish 2 confirmed".

## Notes
- **Unreviewed photos are excluded from publish** (owner decision N2, R19; ch07 §Publish: the popover note is
  "n unreviewed photos will not be published"; 05A MC-7: `ready` requires no unreviewed photo). Ch04 S-G's "will
  publish in Main" was stale and is corrected (changelog 20). The implementer's r1 assumption is right.
- Estimated size: ~20 files, ~2 000 lines.

## Review log
- 2026-10-26, codex implement r1 continuation: Exposed the latest confirmed publish item's id, step, failed step, error and progress on list and workspace reads; exposed each photo's configured-integration upload state; failed transitions now insert a linked notification atomically. Wired row Retry, workspace sticky failure toast and reason popover, upload badges, drawer Retry/Open, and bell count from latest failed items. The existing popover and batch queue remain the confirmation surfaces. Assumptions: missing photo links display "Not uploaded" during a failed run; each failed transition adds a history row, so repeated failures remain visible; a successful Retry dismisses that item's sticky toast; the drawer opens the workspace hash. Deviations: refreshed three older UI test fixtures with the newly required generated DTO fields. No migration or new publish mutation semantics. P15 tests: 11 UI plus 3 Rust, one per criterion row.
- 2026-10-26, codex implement r1: Built the confirmation popover (exact preview summary forwarded), progress display, batch form queue with confirmed-only publish, photo badge component, and notification history drawer. Eight P15 UI behavior tests pass. Assumed the ch07 rule governs unreviewed photos: they are excluded, matching the P13 preview and frozen-run behavior; ch04 S-G's older wording says they publish in Main. Delegated queue form count uses `form_get` for each item; confirmed sessions are sent in confirmation order. No mechanism or backend contract was changed.
- **Blocking boundary gap:** P15's C3(a), C4 integration, and C6(a) cannot be completed within this plan's frontend file perimeter. `session_get` has neither photo `upload_state` nor failed publish item ID/reason. `notifications_list` returns `id`, `session_id`, `run_id`, `kind`, `text`, `created_at`, omitting the publish item ID needed by `publish_retry(item_id)`; the store currently inserts only `published` notifications. The backend command/DTO/store changes must be planned before these rows can be implemented and tested at their named boundaries. C3(b)'s StatusPill behavior is covered in isolation, but the list cannot pass a retry target until the backend exposes it.
- 2026-10-26 (orchestrator): implement r1 BLOCKED on backend data, resolved by plan amendment. Verified against
  `e3848bd`: failed item id/error and per-photo `upload_state` are stored but not exposed (task 0 (a), (b): DTO
  fields only); failed notifications and the notification item id are genuinely absent (task 0 (c): one insert in
  the failure transaction + a join, ch12/S-J). Perimeter extended (backend reads, `SessionRow.tsx`). C3(a), C3(b),
  C4, C6(a) restated at their boundaries with exact mocked inputs; new C7(a)–(c) (Rust) for the backend reads.
  7 criteria, 14 rows. Unreviewed exclusion confirmed (N2/R19); ch04 S-G corrected (changelog 20). Attempt
  archived. **The next implementer continues from the tree at checkpoint `e3848bd`**: keep the components and tests
  that fit, do task 0 first, then wire `FailurePopover`, `PhotoUploadBadge`, row Retry and the drawer Retry to it.
- 2026-10-26, opus review r1: **CHANGES_REQUESTED** (3 blocking, 7 notes in backlog). Gate held (tracker
  `IMPLEMENTED`, checkpoint `d5fc553`); `git show --stat d5fc553` = the 21 declared code files + 3 docs, exactly.
  Suite 374 Rust / 90 UI, clippy, fmt, typecheck green on `549129c` (clean). Verified correct: C1(a), C1(b), C3(b),
  C4(a–c), C5(b), C6(a), C7(a–c) (production and test); task 0 (latest confirmed item on list and workspace reads,
  upload state from the `gallery` / `attribute:<key>` link only with a configured integration, failed notification
  inserted in the failure transaction, `item_id` joined); header Export ↔ Publish switch; bell = latest-failed
  sessions; assumptions 1–4 consistent with the book; H1–H8 hold except as F3.
  **F1 (B1, route production):** publishing emits only `publish_progress` (core never emits `session_changed` for
  publish), but the Sessions list and the workspace view reload only on `session_changed`/`photo_added`/
  `photo_changed`. Rows never show live "Publishing n%", "Publish failed" or "Published", and a failure while the
  workspace is open shows no sticky toast, popover or badges until the page is re-entered (ch07 §Publish "rows show
  live progress"; ch04 S-J; C2(a), C3(a)). C2(a)'s test injects a `session_changed` the backend never sends; probe
  without it: row stays at 10%, header reaches 42%.
  **F2 (B1, route production):** `WorkspacePage` keys the toast/popover on `shownFailure.current === item_id` and never
  resets it, so a Retry that fails again (same item) shows no sticky toast or popover (ch04 S-J).
  **F3 (B1, route production):** `BatchQueue` fetches `publish_preview(confirmed)` at the click and sends its summary
  without showing it; the photographer never sees the batch's gallery-removal counts, unreviewed notes or summary,
  yet `confirmed_summary` records it as shown (07A MC-10 "the exact text shown"; R18 removal count shown before
  confirm; M17). Lessons: C2(a)'s input named an event the backend does not emit for publish; the plan left the
  batch confirmation surface undefined against MC-10/R18.
- 2026-10-26, codex fix r2: Resolved F1 by refreshing the Sessions list and matching workspace view on `publish_progress`; C2(a) now delivers that event alone. Resolved F2 by clearing the shown-failure marker when failure clears and when the workspace changes sessions; C3(a) now covers non-failed → failed → Retry → failed for the same item. Resolved F3 by opening the existing `PublishPopover` for the confirmed batch ids, displaying its summary, per-item counts and notes before confirmation, and sending that displayed summary; C5(a) confirms through this surface and excludes the unconfirmed session. No divergences. Focused UI tests 11/11; full Rust 374, UI 90, clippy, fmt and typecheck green.
