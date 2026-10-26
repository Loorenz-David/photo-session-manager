---
plan: P15_publish_ui
role: review
round: 2
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-26
actor: opus
---
# P15 re-review r2: CHANGES_REQUESTED (1 MUST)

Fix 1 resolved F1 (list and workspace now refresh on publish progress) and F2 (a repeated failure shows the toast
and popover again). The F3 fix opens the review panel outside the form dialog, which is still open. That dialog is
modal, so the panel is inert. Clicking its confirm button closes the whole batch queue, and nothing is published.
Batch "Publish N confirmed" went from "publishes without showing the summary" to "cannot publish at all". This is
**F4**, triaged MUST. The book answers it (ch04 S-H, 07A MC-10), so there is no owner card.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Review history
- r1 settled task 0, C1(a)/(b), C3(b), C4(a–c), C5(b), C6(a), C7(a–c), header Export/Publish and the bell. Those
  areas are not re-verified here.
- r1 raised three blocking findings: F1, F2 and F3.
- This round checks only the fix seam.

## Perimeter check
- **Gate:**
  - The tracker showed P15 `IMPLEMENTED` with a fix-1 note.
  - Checkpoint `7ace3d5` exists.
  - The review tree is `0fb62e8`: the checkpoint plus the runner's prompt commit, which touches docs only.
    `git status --porcelain` was clean at entry.
- **`git show --stat 7ace3d5`:**
  - `BatchQueue.tsx`, `PublishPopover.tsx` (F3)
  - `p15_publish_ui.test.tsx` (F1/F2/F3)
  - `WorkspacePage.tsx` (F1/F2)
  - `services/events.ts` (F1)
  - the master-plan P15 row, the plan's Review log line, and the fix handoff
- Every code file is on its finding's allowed list. Nothing lies outside the perimeter.
- **Suite, lint and typecheck:** run once, as the prompt directs. The fix stamp named no SHA.

  | Command | Result |
  |---|---|
  | `cargo test --workspace` | 374 passed, 0 failed |
  | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
  | `cargo fmt --all -- --check` | exit 0 |
  | `npm run typecheck` | exit 0 |
  | `npm test` | 6 files, 90 passed |

## Findings resolved
- **F1: resolved as corrected.**
  - `onSessionsChanged` now also listens to `publish_progress`, and so does `onWorkspaceEvents`, which maps it to
    `{kind: 'publish_progress', session_id}`.
  - `useWorkspaceState` filters on `event.session_id` and reloads.
  - The Tauri payload `PublishProgressPayload` carries `session_id` (src-tauri `lib.rs` 28–34). The publish service
    emits on every transition, including both `Failed` paths (publish.rs 365, 395).
  - C2(a) is now driven by progress only. C3(a) starts non-failed and turns failed on a delivered
    `publish_progress`.
  - The test-wiring weakness is a note (below).
- **F2: resolved as corrected.**
  - The effect clears `shownFailure` when `failure` becomes null and when `sessionId` changes. The page is not
    remounted per session, so the second reset is needed.
  - Retry → `publish_retry` sets `step = failed_step` → reload → the failure clears → the marker resets → the next
    failed event reloads → the toast and popover show again.
  - The toast id `publish-failed-<item>` is dismissed on Retry, so the second toast is new.
  - C3(a) asserts two `toast.error` calls and the reason visible again.
  - One narrow race remains. It is a note, DROP.
- **F3: the intent is met, but the fix is defective. See F4.**
  - In isolation, `PublishPopover` with `onConfirmed` is right. It shows the batch preview (summary, removals,
    notes) and sends exactly `preview.summary` for `confirmed`.
  - Where it is mounted makes it unusable.

## Remaining findings

### F4: the batch review panel is inert behind the open form dialog, so "Publish N confirmed" never publishes (B1)
- **route:** production
- **What is wrong:**
  - `BatchQueue` returns `<FormDialog open …/>` and `{reviewOpen && <PublishPopover …/>}` as siblings.
  - `FormDialog` stays open while the review is shown. It is a modal Radix `Dialog` (`components/ui/dialog.tsx`:
    portal, `fixed inset-0 z-50` overlay, `DialogPrimitive.Content` modal).
  - A modal Radix dialog sets `body { pointer-events: none }` and marks every sibling of its portal `aria-hidden`.
    It traps focus and dismisses on any pointerdown outside its content.
  - `PublishPopover` is a plain `<section class="absolute … z-50">` in the app tree. It is outside that layer and
    painted under the later overlay.
  - `FormDialog`'s `onOpenChange` is `open => !open && onClose()`, so an outside pointerdown runs `BatchQueue`'s
    `onClose`. `SessionsPage` then sets `queueIds = []` and the queue and the panel unmount. `publish_confirm` is
    never called.
- **Probe:** a temporary test file, deleted afterwards. It rendered the real `Dialog`/`DialogContent` open with
  `PublishPopover` as a sibling, which is BatchQueue's shape. Results:
  - `document.body.style.pointerEvents === "none"`.
  - The popover's "Publish 1 images" button has an `aria-hidden="true"` ancestor and is not inside `dialog-content`.
  - `fireEvent.pointerDown` on that button called the dialog's `onOpenChange(false)` once. `publishConfirm` was not
    called.
- **Why the test missed it:** C5(a) mocks `FormDialog` as a plain `<div>`, so there is no modal layer. It also uses
  `fireEvent.click`, which ignores pointer-events.
- **Authority:**
  - ch04 S-H, the batch publish.
  - Plan C5(a): "Publish 2 confirmed" → `publish_confirm([A, B], T_AB)`.
  - 07A MC-10 / R18: the confirmation surface shows the summary and removals, and its confirm writes exactly that
    text.
- **Correction:**
  - Show the review inside the form dialog's layer. Either render the preview inline in the queue footer, replacing
    the queue strip/footer while reviewing, or open it as a Radix `Dialog` mounted inside the form dialog's tree,
    which stacks as the top layer. Then send exactly the displayed summary. Closing the form dialog before showing
    the panel also works, if Cancel returns to the queue with its confirmations intact.
  - C5(a) must render the real `FormDialog` root, or a mock that keeps its `Dialog`/`DialogContent`, and reach
    "Publish N images" with `getByRole`. `getByRole` skips `aria-hidden` subtrees, so the test fails on today's
    code.
- **Files the fix may touch:**
  - `apps/desktop/src/features/publish/BatchQueue.tsx`
  - `apps/desktop/src/features/publish/PublishPopover.tsx`
  - `apps/desktop/src/features/form/FormDialog.tsx`, only if a slot is needed
  - `apps/desktop/src/features/publish/__tests__/p15_publish_ui.test.tsx`

## Triage (flow §5)

| Item | Class | Why |
|---|---|---|
| F4: the batch review panel is inert, and its confirm closes the queue | **MUST** | User-visible wrong outcome: batch publish (C5(a), ch04 S-H) cannot complete in the running app. |
| N-r2-1: C2(a)/C3(a) mock `services/events` whole, so the real `publish_progress` wiring is unobserved | DROP → backlog | More tests. The behaviour is correct by reading. |
| N-r2-2: F2 marker race (the worker re-fails the item before Retry's reload lands) | DROP → backlog | Speculative robustness, narrow timing. The row still shows "Publish failed". |

## Verified correct (this round)
- F1 and F2 as above.
- `PublishPopover`'s `onConfirmed ?? onClose` keeps the single-session path unchanged. C1(b) still passes.
- In `BatchQueue`, Cancel closes only the panel. Confirming calls `onClose`, which reloads the list.
- The old inline `publishPreview`/`publishConfirm` path is fully removed. There is no second sender.

## Notes written to backlog
Two rows: `P15 (test, r2)` and `P15 (UI, r2)`.

## Lessons for the plans
- Rows about a surface opened from inside a modal should name the layer: "shown within the form dialog". A test
  that mocks the modal host as a `<div>` cannot see modality defects. Plans for UI rows should say which host
  components a test may not replace.

## Write perimeter
- `docs/implementation/master_plan.md`: P15 tracker row only (→ `CHANGES_REQUESTED`).
- `docs/implementation/plans/P15_publish_ui.md`: one Review log entry appended.
- `docs/implementation/backlog.md`: two P15 rows appended.
- `docs/implementation/handoffs/reviewer/handoff_P15_publish_ui_review_r2.md`: this file.
- **Probe:** `apps/desktop/src/features/publish/__tests__/zz_review_probe_r2.test.tsx` was created, run, and
  deleted. Afterwards, `git status --porcelain` was empty.
- No production code was edited. No state outside the test runner's jsdom was touched. Nothing was committed. No
  narrative was written, because the phase is not approved.
