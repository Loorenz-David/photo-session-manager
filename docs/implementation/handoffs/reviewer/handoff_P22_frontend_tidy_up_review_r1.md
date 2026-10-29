---
plan: P22_frontend_tidy_up
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-29
actor: opus
---
# P22 review, round 1: CHANGES_REQUESTED

**Gate.** The tracker showed `IMPLEMENTED`. The checkpoint `a55f97468704b4f87cb44989066151202afb79e5` exists (resolved with the
handoff's `git log -1 -- <handoff>` command).

## Verified correct
- **Perimeter.** `git show --stat a55f974` lists the same 26 paths as the handoff's Files changed. Two of them are outside
  the plan's list, and both are declared as deviations and justified: `lib/routes.ts` (delegated choice: where
  `routeLabel` lives) and `p20_drop_continuity.test.tsx` (its held-overlay assertion `01` → `03` is the old behaviour
  that P22 task 3 replaces; it is still read mid-drag).
- **B4 (all green, run once on tree `a55f974` + the runner's prompt commit).** `cargo test --workspace`: 421 passed, 0
  failed. `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. `cargo fmt --all -- --check`: exit 0.
  `npm run typecheck`: exit 0. `npm test`: 15 files, 124 tests passed.
- **C1(a).** The drawer strips the ` · \d{13}$` suffix and renders `<time dateTime=ISO>` with `dateTimeLabel` (medium
  date, short time, local zone). The test renders a new row and a legacy row. It needs both paragraphs to equal the
  expected text, checks the `datetime` attribute, and checks that no 13-digit run remains, so it fails if the strip or
  the formatter is missing. The ISO literal `2025-10-25T00:00:00.000Z` is right for `1761350400000` (checked with
  `date -u -r`).
- **C2(a).** On focus, the secret is cleared only when it starts with `••••` and equals the value last loaded or saved
  (`loadedSecret` ref). On blur, an empty field gets the mask back. The test covers focus `''`, blur `••••1234`, the
  saved patch keeping the mask, and a typed `NEW-SECRET-77` reaching the patch. The pre-P22 code fails on the focus
  assertion.
- **C2(b).** `commandBack` now also requires `!isTyping(event.target)`. The test sends the key on the focused input
  (the hash stays `#/settings`), then on `window` (the hash becomes `#/`). The old code fails the first half.
- **C3(a).** A `commandsInFlight` ref goes up before the optimistic update. Its decrement runs after `run`'s own
  `reload()`, which is now in `finally`, so it also runs after a failure. While the count is above zero, the event
  handler skips the reload, and `photo_added` still sets the active photo at once. The test checks that no
  `getSessionView` call happens during the event, that the order `[2, 5, 1]` holds before and after the resolve, and
  that a call happens after the resolve. Without the deferral the event reload would return `original`, and the first
  order assertion would fail.
- **C3(b).** The projection matches `useDrop` for the cases the row covers: a same-group tile, the other group (end of
  target, number = length + 1), the photo's own tile (unchanged), and Cover only for Main number 1. The inert
  `flushSync` and its comment are gone. The 200 ms glide and the hidden source are unchanged (the P20 suite passes).
  The test reads `.drag-overlay` during the hold: `03` with no Cover, `02` with no Cover, `01` with Cover, no command on
  the own-slot release, and `01` with Cover on the second drag. The old code shows `01` over slot 3, so it fails.
- **C4(a).** `act` chains every action on a promise in a ref. Each action toasts `<failure>: <message>` when it fails,
  and always reloads in `finally`. The test checks one call before `pA` resolves, the second call
  `(7, 'custom.maker', 'Oak & Co')` after it resolves, then a toast containing `nope` and a new `getForm` call after
  `pB` rejects. The old shared `useAction` drops the second click, so it fails.
- **C4(b).** For `text`, `textarea`, `url`, `int` and `decimal`, `valueOf` seeds an invalid stored value with the same
  verbatim rule `MetafieldInput` uses. Nothing is written until the photographer edits the field (it is not a draft).
  The marker stays until an edit (`invalid = !edited && …`). The test checks the input value `12,5`, the marker text,
  the single write `'12.5'` on blur, and that the marker is gone after the re-fetch.
- **C4(c).** `FormDialog.changeOpen(false)` (Escape and outside click), the X button and Save & close all run
  `saveAndClose`, which is guarded against a second dismissal. It checks `isEdited(title)` before the flush, flushes,
  calls `onItemChanged`, then calls `close`. A debounced Title write without a blur still counts: the draft stays until
  the next `reload`. The test checks the immediate Escape case, including that `onItemChanged` comes before
  `onOpenChange(false)`.
- **C4(d).** An emptied barcode now sends `''`, and an unchanged one still sends `null`. The test checks
  `updateIdentity(7, null, '')`, `onSaved` called with the returned item, and the popover closing.
- **C5(a).** `PageHeader` takes its parent from `parentRoute(window.location.hash)` and its label from `routeLabel`.
  `back` and `goHome` are removed from every caller, and `.health-subtitle` is gone. `App` re-renders on every hash
  change (`useRoute`), so the header always reads the current hash. The test covers four routes, with label,
  destination and no Back control at the root. The P19 navigation suite passes unchanged.
- **Task 2, in passing (backlog 145).** Both dialogs now show the heading only when `shopFailedFor === currentQuery`,
  so a new query cannot show `shop unavailable`. Verified by reading, as the plan asks.
- **Assumptions** (none contradicts the intention). The session route is labelled `Workspace`; no screen shows it,
  because no route has a session as its parent. The date format is the plan's default. A popover closes before its
  queued action finishes (a delegated choice). A second dismissal while closing is ignored. Nothing was tried live.
- **H1–H8.** Untouched. The change is frontend only: no new external write, no core or Tauri change, and the secret is
  never logged.

## Findings

### F1 (B1, `route: production`): the gliding copy reads `00` over a to-review photo
- **What is wrong.** In `WorkspacePage.tsx`, when a Main photo is held over an unreviewed tile, `target` is `'main'`
  (`useDrop` treats an unreviewed tile as Main) and the group is the same, so
  `projectedNumber = ordered.main.findIndex(photo => photo.id === overPhoto.id) + 1`. Unreviewed photos are not in
  `ordered.main`, so this gives `0` and the overlay renders `00`. On release, `useDrop` maps index −1 to
  `group.length`, so the photo lands at the end of Main with a different number. Reviewer probe (temporary test file,
  deleted): Main `1, 2` plus unreviewed `5`, tile 1 held over tile 5 → overlay `00`, release → `reorderPhoto(1, 2)`,
  landing as `02`.
- **Authority.** Plan task 3: "over a tile of the same group → that tile's index + 1 in the group's display order
  (**what `useDrop` will pass to `reorder`**)". Owner, 2026-10-29: "one component, no pop". ch06 §Order: unreviewed
  photos sit at the end of Main, and Main is numbered contiguously from 01. `00` is never a valid slot.
- **Correction.** Compute the same-group projection as `useDrop` does: `index === -1 ? group.length : index + 1`. The
  simplest way is to share one helper for the projected slot between the projection and `useDrop`.
- **Files the fix may touch.** `apps/desktop/src/features/workspace/WorkspacePage.tsx`. Optionally, add one hover over
  an unreviewed tile to `apps/desktop/src/features/workspace/__tests__/p22_workspace_tidy.test.tsx`, next to C3(b).

## Notes written to backlog
- (P19 origin) Dropping a to-review photo back on its own slot approves it to Main, because `useDrop` classifies before
  it checks for the photo's own tile (reviewer probe: `classifyPhoto(6, 'main')`).
- The in-flight counter defers event reloads only. With two quick drags, the first command's own closing reload can
  replace the second drag's optimistic order.
- The `shopDone` check no longer tests whether the search finished. Returning to an earlier query shows its old hits,
  and during an outage it says "n in the shop".
- Process: the implementer edited criterion row C1(a)'s ISO literal. The value is correct, but rows belong to the
  planner and coordinator.
- Not verified live: the drag overlay, the deferred reload and Escape after a Title edit were tested in jsdom only.

## Lessons for the plans
- When a row projects a value that another function also computes (here, the overlay number and `useDrop`'s
  position), name the shared function, or list the full case table for the target set: "same group" hid the
  unreviewed tiles shown inside Main.
- Compute ISO literals from the millisecond value when writing a row.

## Probe declaration
Two temporary test files, both `apps/desktop/src/features/workspace/__tests__/zz_review_probe.test.tsx`, each created,
run and deleted. No production file was edited. Afterwards `git status --short` showed nothing beyond this review's own
writes. No database or app state was touched.

## Write perimeter
`docs/implementation/master_plan.md` (P22 tracker row only), `docs/implementation/plans/P22_frontend_tidy_up.md` (Review
log entry), `docs/implementation/backlog.md` (5 rows appended), this handoff. No commits.

## ⚠ OWNER DECISIONS REQUIRED (0)
none
