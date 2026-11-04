---
plan: P31_compose_mode
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer, Claude Opus 5.5)
---
# P31_compose_mode — review r1: CHANGES_REQUESTED

Gate held: tracker `IMPLEMENTED`, checkpoint `7d98764` in `git log`. Reviewed on clean tree `c88ba28`.

## Verified correct
- **Perimeter.** `git show --stat 7d98764` lists exactly the handoff's 25 paths. The two files outside "Files expected to change" are
  declared: `useWorkspaceState.ts` (`patchPhoto`, the plan's suggested seam) and `__tests__/harness.tsx`. `7d98764..c88ba28` adds only the
  implement handoff and the review prompt.
- **Suite (run once, clean `c88ba28`).** `cargo test --workspace` 529 passed / 0 failed; `cargo clippy --workspace --all-targets -- -D warnings`
  clean; `cargo fmt --all -- --check` exit 0; `npm run typecheck` exit 0; `npm test` 222 passed (26 files). No B4.
- **C1(a).** `geometry.test.ts` runs every op of `crates/core/src/mechanisms/composition_cases.json` (count guard 11 / 5 / 13) at the Rust
  tolerances, with exact reason strings. Loading it as a static JSON import instead of `fs` is fine: it is the same single file.
- **C1(b).** I checked the view maths by hand: frame box 652×652 at (174, 24), s = 0.381956, centre (500, 350). The implementer's zoom values
  (`w = 0.404433`, `x = 0.297784`) are the correct arithmetic from the unrounded `w₀`. The plan's numbers come from `0.6668`.
- **C1(c).** `machine.ts` is pure (no DOM, no React). Each of rules 1–16 has its own row in `machine.test.ts` that asserts phase, live, values and
  effects. The `Resize` extension is declared.
- **Binding.** `View` is coalesced to one rAF write (C3(a) counts the calls). Wheel and WebKit gesture defaults are prevented. The
  document-level `gesturestart` guard exists only while the editor is mounted (C5(b)). Pointer capture and the settle interval behave as task 3
  describes. An unmount mid-gesture commits what was on screen.
- **Rows C2(a)–(e), C3(a)–(b), C4(a)–(d), C5(a)–(c), C6(a)–(d), C7(a), C7(c), C8(a)–(c)** each observe the row's outcome at the row's boundary
  (DOM transform, `setComposition` calls, menu `aria-disabled`, `localStorage`, tile styles), and each would fail on a wrong outcome. C4(d)
  zooms in first, which keeps the row's intent: a zoom-out from the default square changes nothing.
- **Assumptions 1–16.** None contradicts the intention. Assumption 1 (the `panBy` sign) and assumption 3 (`FINGERPRINT_TOLERANCE`) are plan
  slips, recorded as plan lessons in the backlog.
- **H1–H8.** The phase is frontend only, with no Rust and no bindings change. CSS display transforms are not pixel processing (H3). The machine
  and geometry are plain TS (H8). No external write and no live service in tests.

## Findings

### F1: blocking (B1), route `production`. A successful earlier save snaps the stage back while a newer save is queued
- **What is wrong.** `useCompositionSaves` hands every successful answer to the machine as `Saved`. Suppose save A is in flight, B is queued,
  and the machine holds `values = B`. A's answer differs from `values`, so rule 16 sets `values = live = A` and the stage jumps back to A until
  B's answer arrives. A drag started in that window starts from A, so B's change is lost.
- **Evidence.** Probe with a temporary test file, now removed. Two quick +50 px drags, then resolve A. The placement becomes 61.1 px instead of
  111.1 px. A following +10 px drag starts at 71.1 px. The existing C7(b) test only looks after both answers and one coalesced flush, so it
  cannot see this.
- **Authority.** Plan row C7(b): "the view and tile 1 show B's rect (pA's stale answer is ignored)". The plan's Goal: "a save that settles late
  … one defined outcome".
- **Correction.** Dispatch the machine's `Saved` only when nothing newer is queued for that photo (or only for a refusal). The tile may still
  take the stored row. Extend C7(b) to check the stage placement after A settles while B is still in flight.
- **Files the fix may touch.** `apps/desktop/src/features/compose/useCompositionSaves.ts`, `apps/desktop/src/features/compose/ComposeMode.tsx`,
  `apps/desktop/src/features/compose/machine.ts` (only if rule 16's input changes, with `__tests__/machine.test.ts` rule 16 accordingly),
  `apps/desktop/src/features/compose/__tests__/p31_gestures.test.tsx`.

### F2: blocking (B1), route `production`. Esc that closes a menu over the editor also reverts the photo
- **What is wrong.** The `window` keydown handler in `useComposeGesture` dispatches `KeyEsc` / `KeyReturn` without checking for an open layer.
  Radix closes the ⋯ menu on the same Esc but does not stop propagation.
- **Evidence.** Probe: drag +50 (saved), open ⋯, press Esc. The menu closes **and** a second `setComposition` is sent with the entry rect, and
  the placement goes back to 11.1 px. The framing is discarded, and Esc-revert leaves nothing to undo it. The same handler serves the
  right-click menu in the edge browser, the move/copy/assign pickers and the product form, all reachable while composing. Enter on a menu item
  also exits the editor.
- **Authority.** 10D §What the photographer sees ("Gestures and keys (stage only, while composing)"); design handoff §Keyboard ("Esc (close
  topmost)"); the app's own `useBackKeys` guard in `lib/keyboard.ts`.
- **Correction.** The compose keyboard ignores keys while a `[role="dialog"], [role="alertdialog"], [role="menu"]` layer is open (the guard
  `useBackKeys` already uses) or when the event was already handled. Add one behaviour test: drag, open ⋯, Esc → the menu is closed, there is
  exactly one `setComposition`, and the placement is unchanged.
- **Files the fix may touch.** `apps/desktop/src/features/compose/useComposeGesture.ts`, and one of
  `apps/desktop/src/features/compose/__tests__/p31_compose_mode.test.tsx` / `__tests__/p31_gestures.test.tsx`.

No other blocking findings. No B2: every criterion row has a test that can fail. F1's test gap belongs to F1's behaviour defect and is fixed
with it.

## Notes written to backlog
1. P31 (review r1): the editor half of the P30 → P31 row is resolved. No `set` is sent when nothing moved (Return/Done/Esc without a change,
   no-op gestures, `1e-9` comparisons). The backend half is open.
2. P31 → P32: the compose keyboard swallows Space/A/C/F/I/D/arrows even when a button in a non-modal layer has focus, so Space does not
   activate it.
3. P31 → P32: ⌘[ still leaves the workspace while composing. The visible state is committed, so nothing is lost. Decide whether it should be off.
4. P31 plan lesson: `FINGERPRINT_TOLERANCE = 0.002` in task 0 has no use in the contract.
5. P31 plan lesson: wrong literals in C1(b)/C4(a)/C7(a), the C4(d) start state, the C7(b) two-in-flight assumption, and the `panBy` signs of rules 2/7.

## Lessons for the plans
- A latest-wins save queue needs a row that looks at the screen **between** the two settles, not only after both. The plan's rule 16 was
  written for refusals only ("a refused save returned the stored composition"), and its interaction with a queued newer save was never stated.
- An editor that owns Esc/Enter while other layers can open on top of it needs an explicit "topmost layer wins" rule and a row for it.

## Probe declaration
One temporary test file, `apps/desktop/src/features/compose/__tests__/zz_review_probe.test.tsx`: created, run, deleted. `git status --porcelain`
was empty afterwards (before this handoff and the tracker/Review log/backlog edits). No production file was touched, not even temporarily. No
database or app state was involved.

## Write perimeter of this session
`docs/implementation/handoffs/reviewer/handoff_P31_compose_mode_review_r1.md` (new); `docs/implementation/master_plan.md` (P31 tracker row
only); `docs/implementation/plans/P31_compose_mode.md` (Review log entry appended); `docs/implementation/backlog.md` (5 rows appended).
No commit (the runner commits).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
