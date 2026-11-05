---
plan: P31_compose_mode
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer, Claude Opus 5.5)
---
# P31_compose_mode — re-review r2: APPROVED_WITH_NOTES

Gate held: tracker `IMPLEMENTED` with the fix-1 note, and fix checkpoint `f937a9c` is in `git log`. Reviewed on clean tree `24bd3d6`.
Review history: r1 settled C1–C8 except C7(b)'s between-settles state, plus assumptions 1–16 and H1–H8. It raised two B1 findings (F1 saves, F2 Esc over a menu).
This round covers only those two and the seam they changed. Both are resolved. One new item turned up on the changed seam and was triaged DROP. No MUST.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Perimeter check
- `git show --stat f937a9c`: `apps/desktop/src/features/compose/useCompositionSaves.ts`, `ComposeMode.tsx`, `useComposeGesture.ts`,
  `__tests__/p31_gestures.test.tsx`, `docs/implementation/master_plan.md` (P31 row only), `docs/implementation/plans/P31_compose_mode.md`
  (Review log only). All code paths are within F1/F2's allowed files. `machine.ts`, `machine.test.ts` and `lib/keyboard.ts` are untouched. Nothing outside the perimeter changed.
- `f937a9c..24bd3d6` adds only the fix handoff and this round's prompt.
- **Gate divergence (fix handoff, Divergence 1): sound.** The aborted first launch pre-wrote the tracker row. The tree still held review r1's
  `CHANGES_REQUESTED` state, and the runner resumed P31 at the fix on purpose (`8f2c4d7`). The fix applies the r1 corrections as written.
- **Suite (run once, clean `24bd3d6`, tree differs from the fix stamp by docs only, re-run because the prompt asks for it):**
  `cargo test --workspace` 529 passed / 0 failed. `cargo clippy --workspace --all-targets -- -D warnings` clean. `cargo fmt --all -- --check` exit 0.
  `npm run typecheck` exit 0. `npm test` 26 files / 223 passed. No B4. (A scoped `vitest run src/features/compose` gave 91 passed. It was run only to state
  the count in the narrative.)

## Findings resolved
### F1 (B1): resolved as corrected
`settle` now calls `onStored` (workspace row patch, so tiles redraw) on every success. It calls `onSettled` (machine `Saved`, rule 16) only when nothing is
pending, or on a refusal with the last stored row. I traced every case:
- A ok, B pending: tiles take A, B is sent, and the stage stays on B. Then B ok: `Saved(B)` equals `values`, so nothing changes.
- A ok, then B refused: `storedOf` returns A, which `onStored` patched, so the view snaps to A. A is what is stored, so that is correct.
- A refused while B is pending: B is dropped and the view snaps to the stored row. This is C7(c), unchanged since r1.
- A `Saved` that lands after unmount: `dispatch` returns early on a null state. The behaviour is the same as before.
- The extended C7(b) asserts one call before A settles, placement 111.09 between the two settles, and then two calls plus B's tile (`6.4332%`).
  It fails on the r1 behaviour: the fix's red check gave 61.09, which matches my r1 probe.
- Divergence 2 (resolve A first, then B) is sound. Under task 4's one-in-flight rule, B does not exist until A settles. The row's literal is a plan slip
  already in the backlog (r1 note 5). Divergence 3 (the interim tile state is not pinned) is within the correction.

### F2 (B1): resolved as corrected
The keydown handler returns first on `event.defaultPrevented` or on an open `[role="dialog"], [role="alertdialog"], [role="menu"]`.
- The new test (drag, ⋯, Esc on the menu: menu closed, `setComposition` ×1, placement 61.09, editor open) fails on r1 code.
- Divergence 4 (Esc dispatched on the menu element, not on `window`) is sound. It is the real event path past Radix's document capture listener.
- No other `window` keydown listener `preventDefault`s Esc or Enter while composing. `useWorkspaceKeyboard` and `useKeyboard({ d })` are
  disabled while composing, and the workspace has no `PageHeader` with Esc-back. So `defaultPrevented` cannot swallow the editor's own Esc in the app.

## Triage
| Item | Kind | Class | Reason |
|---|---|---|---|
| N1: layer guard mutes compose Esc/Return while a non-modal `role="dialog"` panel is open | new, on the changed seam (B1-class vs 10D "Return finishes; Esc reverts") | **DROP** | No stored value changes and no framing is lost. The key simply does nothing, Done and the card's ✕ work, and the state ends when the panel closes. P32 rewrites this handler's key set and must decide the layer rule anyway. To backlog (P31 → P32). |

No previous blocking item remains. No MUST, so the verdict is `APPROVED_WITH_NOTES`.

## Remaining findings
### N1 (DROP → backlog, route `production`, destination P32)
- **What is wrong.** `LAYER_OPEN` also matches `ItemDetailsCard` (`role="dialog"`, non-modal, no Esc handling of its own while composing). The card opens from
  the compose stage bar's Item details button, or stays open from the workspace when R is pressed. The selector also matches the header's `PublishPopover`
  and `NotificationsDrawer`. While any of them is open, compose Esc does neither the revert nor "close topmost", and Return does not finish.
- **Evidence.** A probe with a temporary test file, now removed: compose, drag +50 (one save), click Item details, Esc on `window`. Result: `setComposition` ×1,
  the card is still open. Enter: the editor is still open.
- **Authority.** 10D §What the photographer sees ("Return finishes; Esc reverts this photo"). Design handoff §Keyboard ("Esc (close topmost)").
- **Suggested correction (for P32).** Decide which layers own the keyboard while composing: only modal dialogs and menus, or Esc first closes the item
  card. Add one behaviour row.
- **Root cause is the r1 correction's selector**, which this reviewer copied from `useBackKeys` without checking the non-modal panels that carry the
  same role. The fix applied it faithfully.

### Notes written to backlog
1. P31 → P32 (re-review r2): N1 above.

### Lessons for the plans
- A "yield to open layers" rule must enumerate the layers. The app uses `role="dialog"` both for modal dialogs and for informational panels (item
  card, header popovers). A role selector copied from page-level back keys is too broad for an editor that owns Esc/Return.
- Plan rows describing a latest-wins queue should use the order the queue allows. C7(b)'s "resolve pB then pA" cannot happen under one-in-flight.
  This is already in the backlog and is restated here so P32/P33 rows that drive saves get it right.

## Probe declaration
One temporary test file, `apps/desktop/src/features/compose/__tests__/zz_review_probe_r2.test.tsx`: created, run twice (the first run's console
output was swallowed, the second used an assertion), deleted. `git status --porcelain` was empty afterwards, before this session's doc edits. No
production file was touched, not even temporarily. No database or app state was involved.

## Write perimeter of this session
- `docs/implementation/handoffs/reviewer/handoff_P31_compose_mode_review_r2.md` (new)
- `docs/implementation/master_plan.md` (P31 tracker row only: `APPROVED`)
- `docs/implementation/plans/P31_compose_mode.md` (re-review r2 Review log entry appended)
- `docs/implementation/backlog.md` (1 row appended)
- `docs/implementation/narratives/P31_compose_mode.md` (new)

No commit (the runner commits). No archgraph state.
