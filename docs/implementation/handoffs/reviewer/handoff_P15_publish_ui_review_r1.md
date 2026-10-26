---
plan: P15_publish_ui
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-26
actor: opus
---
# P15 review r1 (FIX2 relaunch): APPROVED_WITH_NOTES

Fix 2 resolves F4. The batch review is now a Radix dialog nested inside the form dialog's content, so it is the top
modal layer. Its confirm publishes the confirmed sessions with the displayed summary and does not dismiss the queue.
The rewritten C5(a) renders the real `FormDialog`. It fails on the pre-fix production and passes on the fix. All
other rows, F1 and F2 were settled by the earlier rounds on code this fix did not touch. Suite, lint and typecheck are
green. One non-blocking note went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Review history
- Original review r1 (`ace737d`): settled task 0, C1(a)/(b), C3(b), C4(a–c), C5(b), C6(a), C7(a–c), the header
  Export/Publish switch and the bell. It raised F1, F2 and F3.
- Re-review r2 (archived `archive/P15_publish_ui_20261026-081128/handoff_P15_publish_ui_review_r2.md`): F1 and F2
  resolved. F4 was raised as a MUST, because the F3 fix had made the batch review inert. FIX2_AUTHORISED followed.
- Review r1 of the relaunch at 08:10 was BLOCKED at the gate (stale handoff, no code). Its file was overwritten by this
  one. Content: `git show 8d47e0b:docs/implementation/handoffs/reviewer/handoff_P15_publish_ui_review_r1.md`.
- This round has full depth on the F4 seam. Because the code delta since `7ace3d5` is three files, settled rows were
  re-read against the current tree but not re-probed.

## Gate and perimeter
| Check | Observed | Result |
|---|---|---|
| Tracker P15 = `IMPLEMENTED` | yes (fix-2 note) | pass |
| Checkpoint `c4b056d` in `git log` | yes. HEAD `e786ce5` = checkpoint + runner commits (handoff, review prompt) | pass |
| `git show --stat c4b056d` vs declared files | `BatchQueue.tsx`, `PublishPopover.tsx`, `p15_publish_ui.test.tsx`, `master_plan.md`, `plans/P15_publish_ui.md`: exactly the declared list | pass |
| `git diff --stat 7ace3d5 HEAD -- apps crates` | the same 3 code files only. `FormDialog.tsx` untouched, as declared | pass |
| `git status --porcelain` at entry | clean | pass |

## Suite (run once, master plan commands, tree `e786ce5` clean)
| Command | Result |
|---|---|
| `cargo test --workspace` | 374 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` (apps/desktop) | exit 0 |
| `npm test` (apps/desktop) | 6 files, 90 passed |

## Verified correct
- **F4 resolved (C5(a), ch04 S-H, 07A MC-10/R18, H1).**
  - `BatchQueue` passes `<PublishPopover modal …/>` inside `publishSlot`. `FormDialog` renders the slot inside its
    `DialogContent` (`FormDialog.tsx` l.60, l.204), so the review's Radix `Dialog` is a React descendant of the
    form dialog's content.
  - Its `DialogContent` portals to `body` after the form dialog's portal. The result: it paints above, it is the
    highest DismissableLayer, and Escape and outside presses go to it only.
  - The outer layer ignores pointerdown inside it, for two independent reasons: the event is inside its React tree,
    and the outer layer sits below the highest layer that disables outside pointer events.
  - `hideOthers` marks the outer portal `aria-hidden` while the review is open. The outer focus scope is paused.
- **Confirm path.** `confirm()` sends `publishConfirm(confirmed, preview.summary)`, where `preview` comes from
  `publishPreview(confirmed)` and is the object rendered in the body. On success it calls `onConfirmed`, which is
  `BatchQueue`'s `onClose`, so `SessionsPage` clears the queue and selection and reloads. On error it shows a toast
  and the dialog stays open.
  - Cancel and Escape call `setReviewOpen(false)` only. `confirmed` lives in `BatchQueue`, so the ✓ marks survive.
  - `onOpenChange` is ignored while `busy` (the assumption).
- **The single-session popover is unchanged** (`modal` defaults to false). Header and `SessionsPage` still render the
  `<section role="dialog">` form. C1(a)/(b) still exercise it.
- **C5(a) test discriminates.**
  - It renders the real `FormDialog`/`Dialog`. It reaches "Confirm & next", "Publish 2 confirmed" and "Publish 4
    images" by role, the last one within the dialog named "Review publish". Every press is pointerdown + click.
  - It asserts `publish_confirm([1, 2], 'T_AB')` (session 3 excluded), `publishPreview([1, 2])`, the summary and the
    removal note shown before confirming, and `onClose` called exactly once. The last assertion also catches an outer
    dismissal on pointerdown, which would make two calls.
  - **Variation:** I ran the new test file against the pre-fix production (scratch `git worktree` at `7ace3d5` with
    the new test copied in, since removed). Result: `C5(a)` fails with `Unable to find role="dialog" and name "Review
    publish"`, and the other 10 pass. On `c4b056d` it passes.
- **C5(b).** It now uses the real dialog. It awaits the ✓ because the real "Confirm & next" runs `state.flush()` first.
  The outcome is unchanged.
- **Test fixture change.** `@/services/attributes` is now partially mocked (`importOriginal` + `getForm`) because
  the real `FormDialog` imports its other functions. Over-engineering test: this follows the r2 correction, which
  required the real host component. It is not a defect.
- **All other rows** (task 0, C1(a)/(b), C2(a), C3(a)/(b), C4(a–c), C6(a), C7(a–c)): the production and test files
  are byte-identical to the tree on which r1 and r2 verified them. I re-read them against the current tree and found
  no change in their behaviour.
- **Assumptions.** The fix-2 assumption (close requests ignored while the confirm is in flight) does not contradict
  the book. The earlier assumptions also hold, and the narrative reports them: unreviewed photos excluded (N2/R19,
  ch07 §Publish), "Not uploaded" for a missing link during a failed run, one history row per failed transition, a
  successful Retry dismisses the sticky toast, Open lands on the workspace hash, the per-item fill count is
  re-read on confirm, and confirmed ids are sent in confirmation order.
- **H1–H8.**
  - H1: nothing is sent without the review being shown, and the summary sent equals the summary shown.
  - H8: the fix is UI-only, with no business logic in components. Confirm goes through `services/publish`.
  - H2–H7: not touched by this delta. The H7 badge-label note stays in the backlog from r1.
  - There are no secrets and no live services in tests.

## Findings
None blocking.

## Notes written to backlog
- `P15 (UI, fix 2)`: the review's Cancel button stays enabled while "Publish N images" is in flight. Close requests
  through `onOpenChange` are ignored, but Cancel calls `onClose` directly. Outcome: the review closes, the publish
  still proceeds, and the queue then closes. Disable Cancel while `busy`.

## Carry-forward dispositions (open P15 notes)
| Note (backlog row) | Destination |
|---|---|
| P15 (test): C2(a) header observed via test-local `HeaderProgress` | backlog, test debt (no phase) |
| P15 (test): C7 at service, not command, boundary; `Debug` step names | backlog, P16 if commands are touched |
| P15 → P16: `SessionResponse.publish` only from `session_get` | P16 |
| P15 (UI, from P13): raw epoch in "Published · …" text | P16 (settings/notifications polish) |
| P15 → H7: hard-coded "On Shopify" | post-MVP second integration |
| P15 (perf): `usePublish` refetch per progress event | backlog, revisit with large lists |
| P15: `confirm` does not dedupe ids | backlog |
| P15 (test, r2): events module mocked whole | backlog, test debt |
| P15 (UI, r2): F2 marker race | backlog |
| P15 (UI, fix 2): Cancel live while busy | backlog |

## Lessons for the plans
- Already recorded in r2: UI rows about a surface opened from inside a modal should name the layer, and they should
  say which host components a test may not replace. This round confirms that rendering the real host was enough to
  make the row discriminating.
- Runner (from the gate-blocked 08:10 round): relaunch-after-FIX2 needs a resume point, and stale handoffs must not
  be read as `DONE`. This phase got through because the orchestrator archived the files and relaunched by hand.

## Write perimeter
- `docs/implementation/master_plan.md`: the P15 tracker row only (→ `APPROVED`, 2026-10-26, opus).
- `docs/implementation/plans/P15_publish_ui.md`: one Review log entry appended.
- `docs/implementation/backlog.md`: one P15 row appended.
- `docs/implementation/narratives/P15_publish_ui.md`: created.
- `docs/implementation/handoffs/reviewer/handoff_P15_publish_ui_review_r1.md`: this file. It replaces the
  gate-blocked 08:10 file, which is in git at `8d47e0b`.
- **Probe:** a scratch `git worktree` at `/tmp/p15probe` (detached `7ace3d5`, with the current test file copied in and
  `node_modules` symlinked). It was run, then removed with `git worktree remove --force` and `git worktree prune`.
  `git worktree list` shows only the main tree. The main working tree was not touched by the probe.
- I edited no production or test code. I committed nothing. There is no archgraph delta.
