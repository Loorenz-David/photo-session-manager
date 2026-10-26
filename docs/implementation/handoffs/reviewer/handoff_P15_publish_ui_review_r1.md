---
plan: P15_publish_ui
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-26
actor: opus
---
# P15 review r1: CHANGES_REQUESTED

The backend reads (task 0) are right, and so are the popover, row Retry, badges, drawer and batch queue mechanics.
Every criterion row has a test that can fail. There are three blocking defects, all where the UI meets the live
publish:
- **F1:** the Sessions list and the workspace never refresh on publish events. The only event publishing emits is
  `publish_progress`.
- **F2:** if Retry fails again, no second failure toast appears.
- **F3:** batch "Publish N confirmed" records a summary the photographer never saw.

The book answers all three (ch07, ch04 S-J, 07A MC-10, R18), so no owner card is needed.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Verified correct
- **Gate:** tracker `IMPLEMENTED`. Checkpoint `d5fc553` is in `git log`. The review tree is `549129c`, which is the
  checkpoint plus the runner's review-prompt commit. `git status --porcelain` was clean at entry.
- **Perimeter:** `git show --stat d5fc553` lists exactly the handoff's declared files: 21 code/test files plus
  `master_plan.md`, the plan file and the handoff. The three fixture-only edits to P07/P08/P11 tests are disclosed
  deviations. They add `upload_state: null` / `publish: null` to typed fixtures, and nothing else changed in them.
  The earlier partial checkpoint `e3848bd` holds the remaining P15 frontend files, and all of them are inside the
  plan perimeter.
- **Suite, lint and typecheck (tree `549129c`, clean):**

  | Command | Result |
  |---|---|
  | `cargo test --workspace` | 374 passed, 0 failed |
  | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
  | `cargo fmt --all -- --check` | exit 0 |
  | `npm run typecheck` | exit 0 |
  | `npm test` | 6 files, 90 passed |

  The implementer's stamp named no tree identity, so I re-ran it once (review entry, flow B4).
- **Task 0 / C7(a):**
  - `latest_publish` uses the same "latest confirmed item" query as the list record, and it now carries `id` and
    `error`.
  - The list row and `SessionsService::view` both expose it.
  - A session that was never published gives `None`.
  - The test asserts item id, `failed`, `upload` and the fake's message on both reads, plus `None` for the other
    session. It would fail on a wrong id, step or reason.
- **C7(b):**
  - Upload state comes from the photo's link for the role `gallery` (main) or `attribute:<condition_photos_key>`
    (condition). These are the role strings the publish service writes (publish.rs 478–482, 567–571).
  - When no commerce adapter is set it is `None`. `app.commerce` is `Some` only when settings configure an
    integration (integrations/lib.rs `commerce_from_settings`).
  - The test asserts `ready`/`none` as stored, and `[None]` with no integration.
- **C7(c):**
  - The failed notification is inserted in the same transaction as `step = failed`, which also sets
    `last_activity_at`.
  - `retry` writes `step = failed_step` (not `Failed`), so it adds no notification.
  - `item_id` is joined on `(run_id, session_id)`.
  - The test asserts the text `Publish failed · <reason>`, the item id, and newest-first published-then-failed
    history after a retry.
- **C1(a)/(b):** the popover renders main thumbnails in order, the condition thumbnail, "Product details 3/8", the
  unreviewed note (N2), the removal count, the create note, the summary verbatim and "Publish 4 images". Confirm
  sends `preview.summary` unchanged. The tests assert each of these.
- **C3(b):** a `DescriptionFailed` row is a latest item with `failed` / `failed_step = describe`
  (`mechanisms/status.rs`), so `publish_retry(item_id)` is valid. The row passes `row.publish.item_id`, and the test
  asserts the label and `publish_retry(9)`.
- **C3(a)** (on initial load): the sticky toast is `duration: Infinity` with a Retry action. The popover shows the
  reason and "Nothing was overwritten", and Retry calls `publish_retry(7)`. This is correct when the failure is
  already stored as the page opens. See F1/F2 for the live case.
- **C4(a–c):** the badge appears when there is an upload state, or during a failure. `ready` shows "On Shopify",
  `none` (or missing during a failure) shows "Not uploaded", anything else shows "Uploading". The table test
  discriminates all three labels.
- **C5(a)/(b):**
  - Only confirmed ids, in confirmation order, are sent. C is excluded.
  - The strip shows ✓ and `n/N filled` from `form_get`.
  - The real `FormDialog` renders `queueStrip` (l.146) and "Confirm & next" after a flush (l.207). The tests mock
    `FormDialog` and drive those same props.
  - See F3 for the summary.
- **C6(a):** both rows render. The failed row's Retry calls `publish_retry(11)`, and Open sets `#/session/4` and
  calls `onOpen(4)`.
- **Header:** `integration === null` shows Export and a configured integration shows Publish (H6, ch07 §Completion).
  The bell shows the number of sessions whose latest item is `failed` (plan task 0 (c)).
- **Assumptions 1–4:** none contradicts the book. Assumption 1 ("Not uploaded" for a missing link during a failure)
  matches S-J. Assumption 2 (one notification per failed transition) matches ch12's "publish history".
  Assumption 3 (toast dismissed on Retry, Open lands on the workspace) is a delegated choice. Assumption 4
  (unreviewed photos excluded) is N2/R19.
- **H1–H8:**
  - H2, H3 and H5: no RAW, pixel or other-app operations were added.
  - H4: the failure notification is atomic with the step.
  - H6: with no integration there are no badges and Export is the action.
  - H7: core adds no platform names. It uses only the existing `gallery` role string.
  - H8: role derivation lives in `SessionsService`, and the commands only map.
  - H1 is respected for single publishes. The batch path is F3.
- **Tests vs. flow §4:** one test per row at a UI or service boundary. There are no call counts on internal
  collaborators and no snapshots. Nothing fails the over-engineering test.

## Findings

### F1: the Sessions list and the workspace never refresh on publish events (B1)
- **route:** production
- **What is wrong:**
  - While publishing, core emits only `AppEvent::PublishProgress`, mapped to the Tauri event `publish_progress`.
    `SessionChanged` is emitted only by `previews.rs:191`.
  - `SessionsPage` reloads only on `onSessionsChanged` (`session_changed`, `photo_added`).
  - The workspace (`useWorkspaceState`) reloads only on `onWorkspaceEvents` (`photo_added`, `photo_changed`,
    `session_changed`).
  - Only `usePublish` listens to `publish_progress`. It drives the header pill and the bell, but not the list or the
    workspace view.
  - So in the running app, list rows never move through "Publishing n%", "Publish failed" or "Published" while you
    watch. A failure while the workspace is open shows no sticky toast, no reason popover and no badges; they appear
    only when the page is re-entered.
- **Why the test missed it:** the C2(a) test injects a `session_changed` that the backend never sends during a
  publish.
- **Probe (temporary test file, deleted afterwards):** I delivered only `publish_progress`. The header reached "Publishing 42%";
  the row stayed at "Publishing 10%".
- **Authority:**
  - ch07 §Publish: "Publishing never blocks the workspace; rows show live progress."
  - ch04 S-J: the row shows "Publish failed"; a sticky toast offers Retry; the popover shows the reason; badges.
  - Plan C2(a) and C3(a) outcomes; task 2 ("`Publishing n%` from `session_changed`/`publish_progress`").
- **Correction:**
  - Make the list and the workspace view reload on `publish_progress` for their session(s), as `usePublish` already
    does. Emitting `SessionChanged` from the publish service on each transition also works, but
    `crates/core/src/services/publish.rs` is outside this plan's perimeter, so prefer the frontend fix.
  - Drive C2(a) with `publish_progress` only.
  - Make C3(a) start from a non-failed view that becomes failed after a `publish_progress{step: failed}`.
- **Files the fix may touch:**
  - `apps/desktop/src/features/sessions/SessionsPage.tsx`
  - `apps/desktop/src/features/workspace/WorkspacePage.tsx`
  - `apps/desktop/src/features/workspace/useWorkspaceState.ts`
  - `apps/desktop/src/services/events.ts` or `apps/desktop/src/services/publish.ts`
  - `apps/desktop/src/features/publish/usePublish.ts`
  - `apps/desktop/src/features/publish/__tests__/p15_publish_ui.test.tsx`

### F2: a Retry that fails again shows no toast or popover (B1)
- **route:** production
- **What is wrong:**
  - `WorkspacePage` shows the sticky toast and opens the reason popover only when `failure.item_id` differs from
    `shownFailure.current`, and it never resets that ref.
  - Retry keeps the same publish item. After a Retry that fails again (Wi-Fi still off), the item fails under the
    same id, so the effect returns early: no sticky toast, no popover.
  - The ref is also kept across sessions, because the page is not remounted per session.
- **Authority:** ch04 S-J ("Upload times out: … a sticky toast offers Retry, the popover shows the reason").
- **Correction:** clear the "already shown" marker when the session's latest item leaves `failed` (or key it on the
  failure instance, not the item id), so that each new failure shows the toast and popover again. Add the
  fail → Retry → fail case to the C3(a) test.
- **Files the fix may touch:**
  - `apps/desktop/src/features/workspace/WorkspacePage.tsx`
  - `apps/desktop/src/features/publish/__tests__/p15_publish_ui.test.tsx`

### F3: batch "Publish N confirmed" records a summary the photographer never saw (B1)
- **route:** production
- **What is wrong:**
  - `BatchQueue.publish()` calls `publish_preview(confirmed)` at the click and passes its `summary` straight to
    `publish_confirm`. Nothing from that preview is rendered.
  - In a batch, the photographer never sees the gallery-removal count (N1), the unreviewed-exclusion note (N2) or the
    summary text.
  - Yet the run's `confirmed_summary` records that text as the one shown. The run then removes the announced
    platform media.
- **Authority:**
  - 07A MC-10: "Confirming (popover, or 'Publish N confirmed') writes … `confirmed_summary` = the exact text shown"
    (M17).
  - R18: "the removal count is shown on the confirmation popover before confirm".
  - H1.
- **Correction:**
  - Before anything is sent, show the confirmed set's preview (summary, per-item removal counts and notes). Then send
    exactly that text. The smallest option is that "Publish N confirmed" opens the existing `PublishPopover` for the
    confirmed ids, and its "Publish N images" confirms. Showing the preview inline in the queue footer also
    satisfies the book.
  - Update C5(a)'s test to confirm through that surface, still `([A, B], T_AB)`.
- **Files the fix may touch:**
  - `apps/desktop/src/features/publish/BatchQueue.tsx`
  - `apps/desktop/src/features/publish/PublishPopover.tsx`
  - `apps/desktop/src/features/publish/__tests__/p15_publish_ui.test.tsx`

## Notes written to backlog
Seven rows under P15 in `docs/implementation/backlog.md`:
1. C2(a)'s header half is observed through a test-local stand-in.
2. The C7 tests call the service, not the commands, and the command layer uses `Debug`-formatted step names.
3. `SessionResponse.publish` is `null` from every command except `session_get`.
4. The drawer shows `Published · <epoch ms>` raw (text from P13).
5. "On Shopify" is hard-coded (H7 spirit).
6. `usePublish` re-fetches the list and history on every progress event.
7. The notification join assumes one item per session per run, and `confirm` does not dedupe ids.

## Lessons for the plans
- C2(a)'s input named a `session_changed` that P13's publish service never emits. Rows about live UI should name
  the event the backend actually sends (master plan §6.7 lists both events, but not which service emits which).
- The plan named the popover as the confirmation surface but left the batch path's surface undefined against 07A
  MC-10 / R18. C5(a) specified the call, not what is shown before it.

## Write perimeter
- `docs/implementation/master_plan.md`: P15 tracker row only.
- `docs/implementation/plans/P15_publish_ui.md`: Review log entry appended.
- `docs/implementation/backlog.md`: seven P15 rows appended.
- `docs/implementation/handoffs/reviewer/handoff_P15_publish_ui_review_r1.md`: this file.
- **Probe:** one temporary test file, `apps/desktop/src/features/publish/__tests__/zz_review_probe.test.tsx`, was
  created, run once, and deleted. `git status --porcelain` afterwards showed only the documents above.
- No production code was edited. No database or state outside temp dirs was touched. Nothing was committed.
