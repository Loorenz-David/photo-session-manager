---
plan: P31_compose_mode
role: implement
round: 1
state: DONE
date: 2026-11-05
actor: fable (pipeline-implementer, Claude Fable 5.1)
---
# P31_compose_mode — implement r1

Checkpoint `7d98764` on a clean tree (`git status --porcelain` empty after the commit). Gate held: intention RATIFIED, P28 APPROVED,
P31 was NOT_STARTED. No owner decision needed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## What was built
**Product terms.** Compose mode (book 10D): pressing **R**, double-clicking the stage photo, or choosing **Compose…** in the ⋯ or right-click
menu turns the stage into the editor. The square frame stays centred and fills the stage (24 px margin); the adjusted preview moves under it.
Drag or two-finger swipe moves the photo; pinch, ⌘-wheel or ctrl-wheel zooms the composition around the pointer; the composition never passes
the photo's edge and never shrinks below 5 % of the frame's short side. Every change saves by itself when the gesture ends (one save in flight
per photo, the newest waiting behind it, a late answer never touching another photo); a refused save shows "Not saved: <reason>" and the view
snaps back to what is stored. **Esc** mid-gesture cancels that gesture; **Esc** while idle returns the photo to how it was when the editor came to
it; **Return** or **Done** finishes. A, C, Delete, Space, F, I, D, the stage swipes, the webview's page zoom and the two-finger back swipe are off
while composing. The photo browser hides and slides in **over** the image when the pointer reaches the window's right edge (never during a drag),
hides when the pointer leaves it; clicking a photo there moves the editor to it keeping the changes. Grid tiles show the 640 px thumbnail with the
composition drawn over it and an amber **Re-frame** corner when the composition needs review. The first time the editor opens, one line says
"Crops made in Capture One are not used." Compose is unavailable (menu entry disabled, R does nothing) on a photo without an adjusted preview or
a materialised frame.

**Code.** New `apps/desktop/src/services/compositions.ts`; new `apps/desktop/src/features/compose/`: `geometry.ts` (TS mirror of P28's contract
+ view maths), `machine.ts` (pure `reduce`, rules 1–16 in its doc comment), `useComposeGesture.ts` (DOM binding: pointer / wheel / WebKit
gesture / blur / visibility / keyboard → machine; `View` coalesced to one `requestAnimationFrame` write; settle interval; document-level
`gesturestart` prevented), `useCompositionSaves.ts`, `ComposeMode.tsx`, `ComposeStage.tsx`, `ComposeFrame.tsx` (SVG mask + border),
`BrowserOverlay.tsx`, `compose.css`. Changed: `WorkspacePage.tsx`, `Stage.tsx`, `PhotoMenu.tsx`, `PhotoTile.tsx`, `useWorkspaceState.ts`
(`patchPhoto`), `workspace.css`, `tokens.css` (`--warn`). No Rust, no bindings change.

## Tests → criterion rows
All in `apps/desktop/src/features/compose/__tests__/` (shared fixtures in `harness.tsx`). 90 new tests; frontend suite 132 → 222.

| Row | Test | Boundary asserted |
|---|---|---|
| C1(a) | `geometry.test.ts` › "C1(a) …" — one `it` per case of `crates/core/src/mechanisms/composition_cases.json` (43: 11 default_rect, 4 pixel_rect, 1 split_halves, 13 validate, 12 fingerprint, 2 inside) + a count guard | same tolerances as the Rust test (`1e-9`, exact integers, reason strings) |
| C1(b) | `geometry.test.ts` › "C1(b) …" (5 tests: frame box/scale/centre; pan; zoom in; zoom ×0.5 clamp; zoom ×20 clamp) | the view numbers of the row (zoom values corrected — see Deviations) |
| C1(c) | `machine.test.ts` › "rule 1" … "rule 16" (16 tests) + "extension: Resize" (declared candidate) | phase, `live`, `values`, effect list per rule |
| C2(a) | `p31_compose_mode.test.tsx` › C2(a) | R, dblclick, ⋯ entry, right-click entry → `.compose-stage`, transform `translate(11.09px, 24px) rotate(0deg)` ±0.5, overlay, Done, no swipe layer/pill/review buttons, panel only inside `.compose-browser` |
| C2(b) | › C2(b) | Enter and Done → editor gone, swipe layer back, photo 1 active, `setComposition` not called |
| C2(c) | › C2(c) | a c ⌫ Space f i d → no classify/delete, no pill, no strip, no item card, no dialog, editor still open; mode still Adjusted after leaving |
| C2(d) | › C2(d) | no preview / no rect → R does nothing; `Compose…` `aria-disabled="true"` |
| C2(e) | › C2(e) | line on first open only; `localStorage['compose.hintShown'] === '1'` |
| C3(a) | `p31_gestures.test.tsx` › C3(a) | one rAF between two moves; transform `translate(61.09px, 24px)`; one `setComposition(1, …)` with the exact values |
| C3(b) | › C3(b) | saves `x == 0` then `x == 0.333203125`, `y == 0` |
| C4(a) | › C4(a) | ctrl-wheel: `defaultPrevented`, image width ≈ 1612 px, one save with the zoomed rect after 130 ms |
| C4(b) | › C4(b) | ⌘-wheel: same outcome |
| C4(c) | › C4(c) | gesture events + duplicate ctrl-wheel (prevented, ignored) → one save with the zoomed rect |
| C4(d) | › C4(d) | zoom-out clamp → default square; zoom-in clamp → `h == 0.05`, `w == 0.03334` (after a first zoom-in so both clamps are real saves) |
| C5(a) | › C5(a) | wheel `deltaX −50` → image +50 px, one save `x = 0.115466`, `defaultPrevented` |
| C5(b) | › C5(b) | `gesturestart` on `document.body` prevented while composing, not after leaving |
| C5(c) | › C5(c) | wheel during a drag: prevented, one save with the drag's rect |
| C6(a) | › C6(a) | Esc mid-drag: transform back to 11.09, no save, editor open |
| C6(b) | › C6(b) | pointercancel / blur / hidden / lostpointercapture → one save `x = 0.115466`; a following move changes nothing |
| C6(c) | › C6(c) | second non-primary pointer → one save `x = 0.115466` |
| C6(d) | › C6(d) | Esc idle after a drag → second save with the entry values; transform back |
| C7(a) | › C7(a) | pending `p1`, switch via the browser, drag photo 2; calls `(1, 0.115466)`, `(2, 0.115466)`; late `p1` updates tile 1 only (`left 11.5467%`); no `getSessionView` |
| C7(b) | › C7(b) | two quick drags, `pB` then `pA` resolved → B's rect shown (`6.4332%`), exactly two calls |
| C7(c) | › C7(c) | `ServiceError composition_invalid/off_shape` → `toast.error('Not saved: off_shape')`, transform back to 11.09, editor open |
| C8(a) | `p31_browser_and_tiles.test.tsx` › C8(a) | edge pointermove → `data-open="true"` with every tile; pointerleave → `"false"`; inside `.stage-column`, not a `main` child |
| C8(b) | › C8(b) | edge during a drag stays `"false"`; after release opens |
| C8(c) | › C8(c) | tile 1 thumb src, `aspectRatio "2560 / 1707"`, frame `16.6602% / 0% / 66.6797% / 100%`; tile 2 no overlay; tile 3 `"1707 / 2560"` + `Re-frame` |

Reverse map: every test in the five files is listed above; the only test outside a row is `machine.test.ts` › "extension: Resize", declared as a
candidate criterion in the plan's Review log (the machine needs the stage box; a window resize must re-place the frame).

## Assumptions
(Full list with rationale in the plan's Review log, "Judgment calls".)
1. `panBy(rect, delta)` moves the rect by `delta`; the machine negates the pointer displacement (photo follows the pointer) and passes wheel
   deltas as they are (natural scrolling). The literal argument expressions of rules 2 and 7 in the plan have the opposite sign, but every
   stated outcome (rule 2's parenthetical, C1(b), C3(a), C5(a)) holds and the two literals cannot both hold under either sign.
2. `Composition = CompositionDto_Serialize`; `CompositionValues` = the DTO with a non-null `Rect`.
3. `FINGERPRINT_TOLERANCE = 0.002` from task 0 is not defined: the P28 contract (and the shared table) use `1e-6`; an unused constant is dead code.
4. Machine extension `Resize { stage }` (recorded; tested).
5. Esc never navigates back on the workspace page (`PageHeader` has `escapeGoesBack = false`), so no `role="dialog"` and no `lib/keyboard.ts`
   change; ⌘[ is not suppressed while composing.
6. `Blur`/`Hidden` while wheeling commit too; `GestureStart` while wheeling-pan commits the pan first.
7. First-time line = a caption inside the stage for the first editor session; SVG `<mask>` dim (`--color-bg` 72 %), border in `--color-text`.
8. The `.compose-image` wrapper's width/height/transform are written imperatively only (never through React), so a late save answer cannot snap
   the image mid-gesture; the inner `<img>` carries turn/flip and is sized with container units.
9. One save in flight per photo + pending (task 4); pending dropped after a refusal; `lastStored` = the workspace photo's composition.
10. Prev/next arrows stay while composing; any change of the active photo (arrows, a capture arriving) dispatches `SwitchPhoto`; an uncomposable
    new photo returns the normal stage; an unmount mid-gesture commits what was on screen.
11. `Compose… (R)` is the first menu entry; `.tile-review` is bottom-right, grid tiles only; `--warn` `oklch(0.74 0.16 72)` / `oklch(0.8 0.15 78)`.
12. `sameRect` uses `1e-9` so a no-op gesture never saves an ulp-different rect (which would bump `revision` and dirty a published session).
13. `--photo-panel-width` on `.workspace` drives both `.photo-panel` and `.compose-browser`; the overlay hides the fold button by CSS.

## Deviations
- Files outside "Files expected to change": `apps/desktop/src/features/workspace/useWorkspaceState.ts` (+`patchPhoto`, the plan's suggested
  seam) and `apps/desktop/src/features/compose/__tests__/harness.tsx` (shared test fixtures). `apps/desktop/src/lib/keyboard.ts` untouched.
- `geometry.test.ts` loads the shared table with a static JSON import, not `fs` (the frontend tsconfig has no Node types). Same single file.
- Plan arithmetic the tests do **not** reproduce: C1(b)/C4(a) zoom `w = 0.404433`, `x = 0.297784` (plan: `0.404440` / `0.297780`, from the rounded
  `w₀ = 0.6668`); C7(a) `left 11.5467%` (plan `11.5466%`); C4(d) zooms in first (from the default square a zoom-out changes nothing, so rule 3 saves
  nothing); C7(b) resolves `pB` before `pA` as written, although under task 4's one-in-flight rule B is only sent after A settles — the asserted
  outcome (B's rect, two calls) holds. Coordinator: the plan literals may be corrected.
- Tracker actor written as `fable` (the prompt template said `codex`; the plan's `implementer: fable`).

## Files changed (checkpoint `7d98764`, 25 paths)
New: `apps/desktop/src/services/compositions.ts`; `apps/desktop/src/features/compose/{geometry.ts, machine.ts, useComposeGesture.ts,
useCompositionSaves.ts, ComposeMode.tsx, ComposeStage.tsx, ComposeFrame.tsx, BrowserOverlay.tsx, compose.css}`;
`apps/desktop/src/features/compose/__tests__/{harness.tsx, geometry.test.ts, machine.test.ts, p31_compose_mode.test.tsx, p31_gestures.test.tsx,
p31_browser_and_tiles.test.tsx}`.
Changed: `apps/desktop/src/features/workspace/{WorkspacePage.tsx, Stage.tsx, PhotoMenu.tsx, PhotoTile.tsx, useWorkspaceState.ts, workspace.css}`,
`apps/desktop/src/styles/tokens.css`, `docs/implementation/master_plan.md` (P31 tracker row only), `docs/implementation/plans/P31_compose_mode.md`
(Review log entry).
Mutation-probe files: none (flow §4: no mutation regime). Temporary probe files (`__tests__/zz_probe.test.ts`, two `console.log` lines) were
created and removed before the commit; nothing of them is in the tree.

## Checkpoint SHA
`7d98764` — `CHECKPOINT (not approved): P31_compose_mode implement r1 — Compose mode` (parent `db2ce67`).

## Stamp
Tree `7d98764`, clean. Node 22.22.3 via `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH` (master plan §7.1).
- `cd apps/desktop && npm run typecheck` → exit 0.
- `cd apps/desktop && npx vitest run` (= `npm test`) → **26 files, 222 passed, 0 failed** (baseline before this phase: 132 passed; +90).
- `cd apps/desktop && npx vite build` → built (pre-existing chunk-size warning only; not part of the stamp).
- `cargo test --workspace` → **529 passed, 0 failed** across every test binary (no Rust file changed; equals the P30 approval count).
- `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`, no warnings. `cargo fmt --all -- --check` → exit 0.
Failure-ID delta against the P30 baseline: none (∅ → ∅). Nothing pre-existing failing.

## Manual try-out
Preconditions: the app running (`cd apps/desktop && npm run tauri -- dev`), a session with photos that have adjusted previews (P28 materialises
the frame on the first render — tiles show the drawn frame once it exists).
1. Open the session, select a photo and press **R** (or double-click it, or ⋯ → Compose…). The stage becomes the editor: a square frame, the
   photo under it, the right panel gone. Drag the photo: it follows the pointer; let go, then move the pointer to the window's right edge: the
   browser slides in over the image and the tile shows the new framing. Hold ⌘ and scroll (or pinch): the composition zooms around the pointer;
   you cannot zoom out past the photo's edge nor in to a sliver. Two-finger swipe moves the photo; nothing in the window zooms or navigates back.
2. Start a drag and press **Esc** before letting go: the photo snaps back to where the drag started. Press **Esc** again: it returns to how it was
   when you opened the editor. **Return** or **Done** leaves. A / C / Delete / Space / F / I / D do nothing while the editor is open.
3. Drag, then at once reveal the browser and click another photo, and drag that one: both keep their framing (check both tiles).
4. The first time you opened the editor a line said "Crops made in Capture One are not used."; it does not come back (clear it with
   `localStorage.removeItem('compose.hintShown')` in the webview console to see it again).
5. Tiles show the frame drawn over the thumbnail; a photo rotated in Capture One after its first render shows an amber "Re-frame" corner.
Not verified live in this session (no interactive window): the feel of the drag, the WebKit gesture path and the CSS container-query sizing of
the tile overlay — the owner's step 1 and 5 are the checks.
