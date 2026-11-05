---
plan: P38_editor_polish
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-05
actor: opus
---
# P38 editor polish — review r1

**Verdict: APPROVED_WITH_NOTES.** All 12 criterion rows hold at their boundaries, and each has a test that fails on a wrong outcome.
Perimeter = declared files. Suite, lint and typecheck are green. H1–H8 hold. No assumption contradicts 10D. 3 notes went to the
backlog. No blocking findings.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Gate.** Tracker row `IMPLEMENTED` (2026-11-05, codex). Checkpoint `1b7a328` exists (`CHECKPOINT (not approved): implement P38 editor
  polish`), and the handoff resolves it by `git log -1 -- <handoff>`.
- **Perimeter.** `git show --stat 1b7a328` lists 13 paths, equal to the handoff's 13 declared files. The plan's expected `ComposedPicture.tsx`
  is unchanged: only the shared `.composed-split` CSS needed to change (disclosed). `p36_controls.test.tsx` is unchanged and green
  (disclosed: its `Nudge` click now opens the popover). No test outside the plan's list changed.
- **Evidence (reviewer run, code tree = `1b7a328`; the only later commit `fa66dc4` is the review prompt).** `npm run typecheck` passes.
  `npm test` 355/355. `cargo test --workspace` has 0 failures. `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo fmt --all -- --check` are clean.
- **C1(a)** `ComposeFrame.tsx:33–36`: two `<line>`s render, edge (`strokeWidth={4}`) before line (`strokeWidth={2}`), at `middle`, from
  `frameBox.top` to the bottom, only when `kind === 'split'`. The test reads the class order, `x1/x2` against the rendered rect's
  `x + width/2` (1e-6), `y1/y2`, and the `stroke-width` attribute, then checks absence after Normal. It would fail on a wrong order, width,
  or position.
- **C2(a–c)** `EditorToolbar.tsx:13–17` and `StraightenControl.tsx:63`: `fill` follows the state, and `data-on={on || undefined}` sits beside
  `aria-pressed`/`aria-checked`. Each test reads before and after state, `data-on` and `svg[fill]`, plus a persisted `localStorage` `'0'`
  for C2(b) and C2(c), and no save for Loupe.
- **C3(a)/(b)** `CompositionSection.tsx:105–112`: Radix `DropdownMenu` with an icon-only trigger and three `Item`s in the planned order, each
  `onSelect` calling `set(turn left | turn right | flip)`. The tests check `aria-haspopup="menu"`, `aria-expanded`, that the retired
  buttons are absent, the item names in order, one save with `quarter_turns 1` and the P32 `ROTATED` rect (1e-6), and that the menu is
  gone after a pick.
- **C4(a–d)** `CompositionSection.tsx:113–119`: `Popover` `modal={false}`, controlled by local state. The content is
  `#compose-nudge-pad role="group" aria-label="Nudge"` (the props spread after Radix's `role="dialog"`, so `group` wins, and the DOM
  confirms it), with `onOpenAutoFocus` prevented. The arrows are unchanged from P37. The tests check the pad survives each of the four
  clicks, the three exact P37 values plus the clamped `Nudge up`, outside `pointerdown` dismissal without a save, and Esc closing the pad
  with the save count staying 1. A revert would add a call, so C4(d) discriminates: `useComposeKeys.ts:41` skips `defaultPrevented`, and
  Radix prevents the Esc it uses to dismiss.
- **C5(a)** `StraightenControl.tsx` renders `children` after the switch inside `.compose-degrees`. The test checks that
  `row.children.slice(3)` equals exactly `[switch, Rotate and flip, Nudge]`, that each is icon-only, that each is a direct child, and that
  no drawer classes remain.
- **C6(a)** adjusted `p37_editor_chrome.test.tsx`: 27 array rows plus `Show Tools panel` and `Fold panel` = 29 = T3. The texts are T2's,
  verbatim, with the three rotate/flip rows removed. The pad entries are scoped to `#compose-nudge-pad`, which is portalled, so that is
  the right "place". The `Nudge` † action opens the pad. Each card is read with an exact `toBe`.
- **P32 C3(c)** now picks `menuitem`s `Rotate 90° right/left` and `Flip horizontal`. The same saved values (`ROTATED` 1e-9, `quarter_turns`
  1 → 0 → 3, flip rect) stand.
- **Changed-input probes (scratch test file, run, then deleted; tree back to clean `git status`).**
  (1) ArrowRight while the pad is open saves `x = 0.167624`, and the pad stays open: editor keys stay on.
  (2) ArrowRight while the rotate menu is open saves nothing (the `role="menu"` matches `LAYER_OPEN`, as planned).
  (3) After Esc closes the pad, a second Esc reverts (2 saves): the P36 revert still works once no layer is open.
  (4) Clicking the trigger again closes the pad: the custom outside handler excludes the trigger.
- **Retired names.** No `.compose-drawer*`, `#compose-rotate-flip`, `rotateFlipOpen`/`onRotateFlipOpen` props or `ComposeMode` drawer
  state remain in `src/` outside tests. `nudgeOpen` survives only as `CompositionSection`'s local popover state, which satisfies Notes (6):
  it starts closed on every fresh render.
- **Assumptions.** (a) The suggested translucent black edge and the existing tokens: within Delegated choices. (b) The pad keeps focus on the
  editor: Delegated (`onOpenAutoFocus`). (c) A document capture `pointerdown` handler closes the pad on an outside press: an addition
  beyond Radix's own dismissal. Its observable outcome matches 10D ("closed by a click outside"), and probe (4) shows it does not break the
  trigger toggle. None contradicts 10D.
- **H1–H8.** Frontend-only CSS and DOM change. No external write, no RAW access, no pixel processing, no UI automation, no core,
  Tauri-command or Rust change. No secrets and no live services.

## Findings
None blocking. No should-fix.

## Notes written to backlog
1. P38 (by reading): `Popover.Content` gets `id="compose-nudge-pad"`, which overrides Radix's generated content id. The trigger's
   `aria-controls` then points at an id that does not exist. This was plan-induced (registry), and screen readers lose the trigger-to-pad
   link.
2. P38: the document capture-phase `pointerdown` handler duplicates Radix's own outside-dismiss and finds the trigger with a selector
   (`[aria-label="Nudge"][data-state]`). The test-environment timing it works around (Radix registers its listener on the next tick) does
   not exist in the app. It is harmless today, but it is a second dismissal path to keep in step.
3. P38 (not verified in WebKit): line contrast, filled-icon look, and menu/pad placement and z-order over the stage were checked by DOM
   shape only. The owner's try-out step 7 is the first visual check.

## Lessons for the plans
- A plan that pins a Radix content `id` should name the side effect on the trigger's `aria-controls`, or assert the pad by role and label
  instead of by id.

## Probe declaration
Files touched by probes: `apps/desktop/src/features/compose/__tests__/zz_review_probe.test.tsx`, created, run and deleted. No production
file was touched. `git status --porcelain` was clean after deletion. No database or app state changed.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_P38_editor_polish_review_r1.md` (this file)
- `docs/implementation/narratives/P38_editor_polish.md`
- `docs/implementation/plans/P38_editor_polish.md` (Review log entry)
- `docs/implementation/master_plan.md` (P38 tracker row only)
- `docs/implementation/backlog.md` (3 rows appended)
