---
plan: P32_composition_tools
role: implement
round: 1
state: DONE
date: 2026-11-05
actor: fable (pipeline-implementer, claude)
---
# Handoff — P32_composition_tools, implement r1

⚠ OWNER DECISIONS REQUIRED (0): none.

## What was built
The editor's tools, the second half of 10D's composition editor for Normal compositions:
- **A Tools panel on the left of the editor**, present only while composing. Its sections fold open and closed on their title and remember it
  across launches; ⌘T hides and shows the whole panel (remembered). **Composition** is the first and only section; the panel is a list of
  `{ id, title, content }` so colour and light tools can be appended later.
- **The Composition section**: Normal | Split (Split disabled until P33), the Shape menu (Original, Square (1:1), 4:5, 3:2, 2:3, 16:9, Free),
  Turn frame (X), Rotate left / right, Flip, a straighten slider (−45…45, 0.1° steps, readout `n.n°`, double-click levels), the output preview
  at the composition's real proportions (the same adjusted preview clipped by CSS), and Centre, Reset, Done.
- **The re-fit rule** (one pure function): a shape, turn, rotation, flip or straighten change keeps the composition's centre and area where the
  new ratio and the frame allow, shrinking about the centre until the rect fits the frame and, with a tilt, the rotated image. Every result
  passes the backend's validation.
- **Trim handles** on the frame's corners and edges: while held, the overlay frame shrinks (a locked shape keeps its ratio, a free shape moves
  one edge); on release the composition is saved once and the frame glides back to fill the stage.
- **Keys while composing**: arrows nudge the frame one screen pixel (Shift ×10), `.` centres, 1–6 and 0 pick the shape, X turns the frame,
  `[` `]` move to the previous / next photo keeping the changes (no wrap), ⌘⌥R resets to the default, ⌘T toggles the panel; every other key —
  the workspace's and ⌘[ Back among them — does nothing; a key inside a control belongs to the control.
- **The Re-frame confirmation**: a photo in `needs_review` opens with a banner naming the cause ("Changed in Capture One: straighten here
  instead" for a geometry change; "Capture One changed this photo's frame — check the framing and press Return" for a frame change) and its
  stored framing re-fitted to the photo's current frame; Return (or Done, or any save) confirms and clears the flag. Export and Publish refusals
  (`needs_review`) surface the backend's own message ("Re-frame first: Main 02") as a toast; the publish popover stays open.
- Machine: phases `trimming` and `adjusting` (a live slider), events `SetValues { values, cause }`, `Nudge`, `TrimStart/Move/End`, a
  `needsConfirm` flag; every control and key runs through it and saves through P31's queue. No Rust, no bindings change.

## Tests → criterion rows
| Row | Test (file › name) | Shape of the assertion |
|---|---|---|
| C1(a) | `refit.test.ts` › (1)…(11) + clamp/round | exact rects (1e-9 / 1e-5 for bisection), every result passes `validate`; (8) also shows a larger rect fails |
| C1(b) | `machine.test.ts` › "P32 C1(b)" — 10 rows (SetValues idle / wheeling / dragging, Nudge, Trim start/move/end, Trim Esc, Trim Blur, free e-edge, SetValues live/set/Esc/Blur, needs_review entry + confirm) | exact effect lists, phases, rects |
| C2(a) | `p32_tools_panel.test.tsx` › C2(a) | panel first in `main.workspace`, section titled Composition `data-open` true → click → false + `tools.sections`; remount via hash → still false; ⌘T → `hidden` + `tools.hidden === '1'`; ⌘T → shown |
| C2(b) | `p32_tools_panel.test.tsx` › C2(b) | no `aside.tools-panel` when not composing; `aside.photo-panel` right of the stage column; panel gone again after Return |
| C3(a) | `p32_controls.test.tsx` › C3(a) | one save `{shape '4:5', orientation 'named', rect (0.23328125, 0, 0.5334375, 1)}`; `.compose-frame` ≈ 521.6 × 652 at (239.2, 24) |
| C3(b) | › C3(b) | Turn frame → save `{orientation 'turned', shape '4:5', rect (0.1666015625, 0.1, 0.666796875, 0.8)}`, frame 815 × 652; `x` key → back to named |
| C3(c) | › C3(c) | right → `quarter_turns 1`, rect (0, 0.1666, 1, 0.6668), inner `rotate(90deg)`, placement (174, −138.9); left ×2 → `3`, `rotate(270deg)`; Flip on free (0.1, 0, 0.5, 0.75) → `horizontal`, `x = 0.4`, `scaleX(-1)` |
| C3(d) | › C3(d) | `input` 10 → transform ends `rotate(10deg)`, no save, readout `10.0°`; `change` → one save straighten 10, rect centred, `h = 1/(cos10°+sin10°)`; double-click → straighten 0, rect unchanged |
| C3(e) | › C3(e) | `.output-preview` `aspectRatio` `1707 / 1707` → `1365.6 / 1707`; its `img.src` equals the stage image's; exactly two such `img`s |
| C4(a) | › C4(a) | 8 handles; se drag: frame 552 × 552 at (174, 24) while held, image unchanged, no save; release → one save (0.166602, 0, 0.564526, 0.846626), frame 652 × 652, `data-gliding="true"`, image width 977.81/h, cleared on `transitionend` |
| C4(b) | › C4(b) | free e-edge −100 px → one save `w = 0.564526`, `h = 1`, `x` unchanged |
| C4(c) | › C4(c) | Esc mid-trim → frame 652 × 652, no save, editor open, no glide |
| C5(a) | `p32_keys.test.tsx` › C5(a) | ArrowRight → `x += 1/977.81`; Shift → `+10/977.81`; ArrowDown → no save (`y` 0 on every call); ArrowLeft → back one px |
| C5(b) | › C5(b) | `.` and Centre after a drag → one save each with `x = 0.166601562`, `y = 0` |
| C5(c) | › C5(c) | 7 saves with shapes original, 1:1, 4:5, 3:2, 2:3, 16:9, free in order; rects re-fitted from the previous; select shows `free` |
| C5(d) | › C5(d) | `]` `]` `]` `[` `[` `[` show photos 2, 5, 5, 2, 1, 1; `p1` resolves → stage 61.09, tile 1 `11.5467%`, tiles 2/5 unchanged, one save, `getSessionView` not re-called |
| C5(e) | › C5(e) | ⌘⌥R and Reset → `resetComposition(1)` once each, view back to `translate(11.09px, 24px)`, no extra save |
| C6(a) | › C6(a) | a c ⌫ Space f i d n ⌘[ → no classify/delete, no dialog/card/pill/strip, hash unchanged, editor open, no save |
| C6(b) | › C6(b) | ArrowRight inside `#compose-straighten` → `defaultPrevented === false`, no save |
| C7(a) | `p32_reframe.test.tsx` › C7(a) | banner frame text; image 652 × 977.81 at (174, −138.9) (the re-fitted square), preview `1707 / 1707`, no save |
| C7(b) | › C7(b) | tile Re-frame corner present; Enter → one save `(0, 0.1666, 1, 0.6668)`; editor closed, banner and corner gone |
| C7(c) | › C7(c) | geometry banner text; placement 11.09; Enter → one save with the stored values; corner gone |
| C7(d) | › C7(d) | vertical drag → one save `y = 0.1666 + 50/977.81`, `x = 0`, `w = 1`; banner gone; Enter sends nothing more |
| C8(a) | › C8(a) | Export → `runExport(5)` rejects → `toast.error` with a text containing `Re-frame first: Main 02` |
| C8(b) | › C8(b) | Publish → popover → confirm rejects → the same toast; the `Review publish` dialog is still the same node |

Every test in the five new files and the ten new `machine.test.ts` rows maps to a row above; no orphan tests.

## Assumptions
(Also in the plan's Review log entry, numbered the same.)
1. `Nudge{dx: 1}` keeps P31's pointer convention (`x −= 1/(W_t·s)`, C1(b)); the key layer sends ArrowRight as `Nudge{dx: −1}` so the frame moves
   right over the photo (C5(a)).
2. ArrowDown with `h = 1` sends no save: the clamped rect equals the committed one (P31 rule 3); the test asserts 2 saves for the three presses.
3. Straighten-live is a machine phase `adjusting { startValues }`: React `onChange` (native `input`) → `SetValues{cause:'live'}` (view only);
   a native `change` listener → `SetValues{cause:'set'}` (one save); double-click → 0. Esc mid-slide restores; Blur / pointer down / wheel commit.
4. Quarter turn on a free rect turns the rect with the content; `original` → whole frame. Flip also runs the straighten guard (no effect at θ = 0).
   When even the minimum size at the rect's centre is outside the tilted image, the centre moves to the frame's centre before the bisection.
5. `TrimMove { pointerId, point }` (the handle is in the phase); `PointerMove/Up/Cancel/LostCapture` of the trimming pointer act as `TrimMove/End`;
   a handle's pointer down is recognised by the stage's native listener (`closest('.compose-handle')`).
6. The glide is detected in the binding (`trimming → idle`, `live ≠ startRect`): `data-gliding="true"` until the image's `transitionend` (400 ms fallback).
7. Controls, frame box, handles, output preview and the inner turn/flip render from the machine's published snapshot (`useSyncExternalStore`;
   not published during the moves of a drag, pinch or wheel); the image transform stays imperative.
8. Reset calls `resetComposition`, patches the row and sends `Saved` to the machine; `entry` is left alone (Esc still reverts to the editor's entry).
9. Done stays in the stage bar as well as in the panel (the panel can be hidden).
10. ⌘ combinations other than ⌘T / ⌘⌥R / ⌘[ are left to the app; ⌥ alone is left for P34; Tab keeps moving the focus; `code === 'KeyR'` also matches ⌘⌥R.
11. The banner reads the workspace row's `state`, so it disappears when the `ok` answer is patched in; the toasts check `error.code` duck-typed.
12. `[`/`]` back onto a photo whose save is in flight show the stored row until the answer lands (then rule 16 snaps) — P31 C7(a) semantics.
13. A side within 1e-12 of the whole frame snaps to exactly 1 in the re-fit (so `x`/`y` clamp to exactly 0).

## Deviations
- Plan arithmetic asserted at the contract's exact values, not the plan's rounded ones (recorded in the Review log): re-fit (4)/C3(b)
  `w = 0.666796875, h = 0.8, x = 0.1666015625, y = 0.1`; re-fit (8)/C3(d) `h = 1/(cos 10° + sin 10°) = 0.8632` (plan 0.816); re-fit (10)/C4(a)
  `h = 1 − 100/652 = 0.846626`; C1(b) "TrimStart, Blur → Save" tested with a `TrimMove` in between (nothing to save otherwise); C7(d)
  `y = 0.166601562 + 50/977.81` under the unchanged machine rule 2 (the plan's `− 50/652` has the wrong side and sign).
- `useComposeGesture.ts` exports `LAYER_OPEN` (shared with `useComposeKeys`). `trim` lives in `refit.ts` (Delegated). Tracker actor `fable`.
- No files outside the plan's list were created or changed.

## Files changed (write perimeter = the checkpoint commit)
New: `apps/desktop/src/features/compose/{refit.ts, useComposeKeys.ts, useToolsPanelState.ts, ToolsPanel.tsx, CompositionSection.tsx,
OutputPreview.tsx, Handles.tsx}`, `apps/desktop/src/features/compose/__tests__/{refit.test.ts, p32_tools_panel.test.tsx, p32_controls.test.tsx,
p32_keys.test.tsx, p32_reframe.test.tsx}`.
Changed: `apps/desktop/src/features/compose/{machine.ts, useComposeGesture.ts, ComposeMode.tsx, ComposeStage.tsx, ComposeFrame.tsx, compose.css}`,
`apps/desktop/src/features/compose/__tests__/machine.test.ts`, `apps/desktop/src/features/workspace/{WorkspacePage.tsx, workspace.css}`,
`apps/desktop/src/features/shell/Header.tsx`, `apps/desktop/src/features/publish/PublishPopover.tsx`, `docs/implementation/master_plan.md`
(P32 tracker row only), `docs/implementation/plans/P32_composition_tools.md` (Review log entry). 25 paths.
Mutation-probe files: none (no mutation regime, flow §4). One throwaway probe `apps/desktop/src/zz_probe_order.test.ts` (jsdom listener order)
was created, run and deleted before any production edit; not in any commit. No Rust, no bindings, no intention-book change.

## Checkpoint SHA
`c0cc169` — `CHECKPOINT (not approved): P32_composition_tools implement r1 …`; `git status --porcelain` empty after the commit (this handoff is
the only file written since).

## Stamp (tree `c0cc169`, clean)
- `cd apps/desktop && npm run typecheck` → exit 0.
- `cd apps/desktop && npm test` → 31 files / 268 passed (223 before this phase + 45 new).
- `cargo test --workspace` → 529 passed / 0 failed (37 `test result: ok` lines summed); `cargo clippy --workspace --all-targets -- -D warnings`
  → finished, no warnings; `cargo fmt --all -- --check` → exit 0. Run on the working tree just before the checkpoint; the Rust sources are
  byte-identical to `c0cc169` (this phase changes no Rust file), so the stamp binds to that tree.
Pre-existing failures: none.

## Manual try-out
1. `cd apps/desktop && PSM_DATA_DIR=/tmp/psm-dev npm run tauri -- dev` (Node 22 via nvm). Open a session with adjusted previews, select a
   photo and press R. A Tools panel sits on the left with "Composition" open. Click its title: it folds; quit and relaunch: still folded.
   ⌘T hides the whole panel; ⌘T again brings it back.
2. Pick 4:5 in Shape: the frame becomes upright 4:5 around the same centre; press X: it turns to 5:4; press 2 to go back to square.
   Rotate right: the photo turns, the frame stays; Flip mirrors it. Drag the straighten slider to about 5°: the photo tilts under the frame
   and the frame shrinks just enough to avoid empty corners; double-click the slider to level it. The small output preview in the panel
   shows the result at its real proportions throughout.
3. Drag a corner handle inward: the frame shrinks while you hold; let go: it glides back to fill the stage, now showing a tighter crop.
   Press `.` to centre, arrows to nudge by a pixel (Shift for ten), ⌘⌥R to reset.
4. Press `]` and `[` to move through the photos without leaving the editor; your changes stay (watch the tiles via the edge reveal).
5. On a photo you straightened in Capture One (P28 step 3), open the editor: a banner says "Changed in Capture One: straighten here instead";
   press Return to confirm; the amber "Re-frame" corner on its tile disappears and Export/Publish stop refusing. A photo rotated 90° in
   Capture One shows the other banner and its framing fitted to the turned frame. While a photo still needs review, Export shows
   "Re-frame first: Main 02" as a toast and the Publish confirmation refuses with the same text.

⚠ OWNER DECISIONS REQUIRED (0): none.
