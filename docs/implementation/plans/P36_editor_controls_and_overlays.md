---
phase: P36_editor_controls_and_overlays
title: Editor controls and overlays — the photo browser stays as the single strip while composing (F folds and unfolds it), alignment overlays cycled with O, a straightening grid with its own switch, typed and stepped straighten, a clickable control with a name-and-key hint for every editor shortcut, focus handed back to the editor, the snap guide drawn where it snapped
difficulty: high
projection: no
depends_on: [P35_composed_display_and_chrome]
---
# P36 — Editor controls and overlays: the single strip while composing, overlays (O), the straightening grid, typed straighten, a control and hint for every shortcut, focus back to the editor, the snap guide where it snapped

## Goal
The editor half of the owner's corrections after the first try-out (10D as amended `b75ad61`/`163857a`, R38, changelog 28).
(1) **While composing the right photo browser is its single folded strip and stays put** — the P31 hidden overlay with edge reveal is removed;
clicking a photo in the strip moves the editor to it keeping the changes; **F** unfolds the full panel (beside the stage) and folds it again;
leaving the editor puts the panel back as it was. (2) **Alignment overlays** inside the frame — Off, Thirds, Grid, Golden ratio, Diagonals —
picked in the Composition section, **O** cycles them, remembered across launches; while straightening, a **fine square grid** covers the image,
with its own on/off switch (remembered). (3) The straighten control takes a **typed number of degrees** and **↑/↓ steps** (0.1°, Shift 1°)
besides its slider, range −45…45. (4) **Every editor shortcut has a clickable control** in the Tools panel or above the stage, and every
control's hint shows its name and key ("Split (S)"), through P35's `Hint`. Folded in (cheap, same files, P32/P34 narratives' "Not yet"): after
the Shape menu, the Overlay menu or a pointer drag of the straighten slider the **keyboard goes back to the editor** (keys used to do nothing
until the photo was clicked), and the **snap guide is drawn at the line it snapped to** (centre or image edge), not always at the centre.
**Not in this phase:** zoom buttons (pinch and ⌘-wheel are gestures, not keys), colour and light tools, the save-queue unification (P34 Not yet).
No backend change.

## Read first
Book: `10D_composition_and_framing.md` §What the photographer sees (bullets "Compose …" — strip, F; "A Tools panel …" — straighten typed/steps,
Overlay, straightening grid; "Every shortcut has a control"; "Gestures and keys" — the full key list incl. "F stays and folds or unfolds the
strip") · `15_decisions.md` R38 · `16_changelog.md` entry 28 (readings: the overlay set and O to cycle them, Photoshop's crop-overlay key;
0.1°/1° steps) · evidence `design_handoff.md` §Keyboard, §Tokens · process `flow.md` §4, §6, §9 · master plan §6.10 (P35/P36 block) ·
plans `P31_compose_mode.md` (fixture: stage `1000 × 700` at `(0, 0)`, frame `2560 × 1707`, default square, `frameBox {174, 24, 652, 652}`,
`s = 0.381956`, `W_t·s = 977.81`; the interaction contract; rows C2(a), C2(c), C8(a)–(b)), `P32_composition_tools.md` (the re-fit contract; C3(d)
straighten re-fit (8) `h = 0.816 ± 0.005`; C5(a) nudge `x += 1/977.81`; C6(b) a key inside a control belongs to it), `P33_split_ui.md` (split
frame box `952 × 476` at `(24, 112)`), `P34_composition_polish.md` (loupe, snapping `SNAP_PX = 6`, clipboard fixture and C3(a) re-fit
`(0.26002, 0.21998, 0.47995, 0.40004)`, meter; the orchestrator ruling on C2(b): a snap back to the entry rect emits no save),
`P35_composed_display_and_chrome.md` (`Hint`, `aria-disabled` rule, the Sessions icon on `PhotoPanel`/`FoldedStrip`) and the narratives
`narratives/P32…P34` ("Not yet").
Code at `163857a` (line numbers as of that commit; P35 changes some of these files first — re-read them at the gate):
- `features/compose/ComposeMode.tsx` (`stepPhoto :119–127`, `pick :105–116`, keys `:173–176`, `<Stage compose={…}> :193–210`, `<BrowserOverlay> :211`,
  the loupe/meter state `:69–71`), `BrowserOverlay.tsx` (deleted by this phase), `compose.css:35–39` (`.compose-browser`).
- `features/compose/useComposeKeys.ts:46–82` (the editor's key map; `F` is swallowed today), `useComposeGesture.ts:230–244` (Esc / Return / the
  swallowed set; `LAYER_OPEN :35`), `machine.ts:255–261` (the snap inside the drag rule: `altKey` suspends) and `:358–361` (`SetValues` causes;
  `adjusting` phase for a live slider), `snapping.ts:7–20` (guides are booleans), `ComposeFrame.tsx:26–27` (guides drawn at `width/2`,
  `height/2` — the always-centre bug), `ComposeStage.tsx:65–69` (the overlay SVG and handles).
- `features/compose/CompositionSection.tsx` (the controls; the slider's `change` listener `:59–66`; the readout `:127–129`), `ToolsPanel.tsx`,
  `useToolsPanelState.ts` (the `localStorage` pattern to copy: guarded reads and writes), `useClipboardComposition.ts` (`useClipboardComposition()`
  hook for the Paste control's state), `Loupe.tsx`, `FrameMeter.tsx`.
- `features/workspace/Stage.tsx` (`.stage-tools :67–77`; the prev/next `.stage-nav` buttons `:80–92` call `onStep`), `WorkspacePage.tsx` (the
  right column `:423–427`: `composing ? null : …`; `folded` state `:133`), `PhotoPanel.tsx`, `FoldedStrip.tsx`, `lib/keyboard.ts:4–7` (`isTyping`:
  INPUT, TEXTAREA, SELECT — why a focused select or range takes every key).
- Tests that **change their expectation** in this phase (behaviour changed by R38 / changelog 28): `features/compose/__tests__/
  p31_browser_and_tiles.test.tsx` C8(a), C8(b) (the edge reveal is retired: delete those cases; P36 C1 replaces them); `p31_compose_mode.test.tsx:21`
  and P31 C2(a)'s "only inside `.compose-browser`" (now: the strip is a flex sibling); P31 C2(c) and P32 C6(a) (`f` is no longer swallowed: it
  folds/unfolds the strip; the other keys stay swallowed); `p31_gestures.test.tsx:302, 356`, `p32_keys.test.tsx:118`, `p32_reframe.test.tsx:63`
  (the `.compose-browser` scope becomes `.folded-strip`; the Re-frame badge is grid-only, so `p32_reframe` reads it after `f` unfolds the panel);
  `p32_controls.test.tsx` C3(d) (the readout `10.0°` becomes the degrees field's value `10.0`); `snapping.test.ts` (guides become positions);
  `p34_loupe_snap.test.tsx` C2(b) (the guide also asserts its position).

## Files expected to change
Frontend (new): `apps/desktop/src/features/compose/{overlays.ts, EditorToolbar.tsx, StraightenControl.tsx, useComposePrefs.ts}`.
Frontend (changed): `features/compose/{ComposeMode.tsx, ComposeStage.tsx, ComposeFrame.tsx, CompositionSection.tsx, useComposeKeys.ts,
useComposeGesture.ts, snapping.ts, machine.ts (guide type only), compose.css}`, `features/workspace/{Stage.tsx, WorkspacePage.tsx}`; deleted:
`features/compose/BrowserOverlay.tsx`.
Tests (new): `features/compose/__tests__/{overlays.test.ts, p36_strip_overlays.test.tsx, p36_straighten.test.tsx, p36_controls.test.tsx}`.
Tests (changed): the eight listed in Read first. Estimated: 16 source paths (one deleted), 4 new tests, 8 adjusted tests.

## Core mechanisms (unit tests allowed)
- `features/compose/overlays.ts` — `overlayLines(kind, box, split) → { x1, y1, x2, y2 }[]` in stage pixels (C2(a)).
- `features/compose/snapping.ts` — `snap(rect, threshold) → { rect, guides: { x: SnapLine | null, y: SnapLine | null } }`, `SnapLine = 'centre'
  | 'start' | 'end'` (C8(a)).

## Ordered tasks
0. **Registry first (master plan §6.10, amended).** Keys while composing: `o` cycles the overlay; `f` folds/unfolds the browser (no longer
   swallowed). `localStorage` (guarded like `useToolsPanelState`): `compose.overlay` ∈ `off | thirds | grid | golden | diagonals` (missing or
   invalid → `thirds`), `compose.straightenGrid` (`'0'` = off; default on), `compose.snap` (`'0'` = off; default on). Constants
   `OVERLAY_ORDER = ['off', 'thirds', 'grid', 'golden', 'diagonals']` (O goes to the next, wrapping), `STRAIGHTEN_GRID_PX = 24`,
   `STRAIGHTEN_STEP = 0.1`, `STRAIGHTEN_STEP_SHIFT = 1`. CSS `.compose-overlay-guides[data-overlay]`, `.compose-straighten-grid`, `.editor-toolbar`,
   `.compose-nudge`. Retired: `EDGE_REVEAL_PX`, `.compose-browser`, `BrowserOverlay`.
1. **The strip while composing** (`ComposeMode.tsx`, `WorkspacePage.tsx`): delete `BrowserOverlay`; `ComposeMode` renders the right column itself,
   after `.stage-column`, from its own `browserFolded` state (initial `true` at every mount): folded → `<FoldedStrip>`, unfolded → `<PhotoPanel>`,
   both with `onOpen = pick` (a tile click moves the editor, keeping the changes) and P35's Sessions toggle passed through; their Unfold / Fold
   buttons toggle `browserFolded`; `useComposeKeys` maps `f` (no modifier) to the same toggle. `WorkspacePage` keeps rendering nothing on the right
   while composing and its own `folded` is untouched, so leaving the editor shows the panel as it was before. Nothing reacts to the window's right
   edge any more.
2. **Preferences** (`useComposePrefs.ts`): `{ overlay, setOverlay, cycleOverlay, straightenGrid, setStraightenGrid, snap, setSnap }` read once
   at mount and written on change (guarded).
3. **Overlays** (`overlays.ts`, `ComposeFrame.tsx`): inside the frame's SVG a `<g class="compose-overlay-guides" data-overlay={kind}>` of
   `<line>`s from `overlayLines(kind, drawnBox, kind === 'split')`, computed per **half** for a split (each half is its own picture): `thirds` —
   lines at 1/3 and 2/3 of the width and of the height; `grid` — lines at k/6, k = 1…5, both ways; `golden` — at `1/φ² = 0.381966` and `1/φ =
   0.618034` both ways; `diagonals` — from each corner a 45° line into the box until it meets the far long side (Photoshop's Diagonal), duplicates
   removed (a square box has exactly its two corner-to-corner diagonals); `off` — no `<g>` children. Vertical lines span the box's (half's) top to
   bottom, horizontal ones its left to right. `pointer-events: none`; colour Delegated (legible over light and dark photos).
4. **Overlay control** (`CompositionSection.tsx`): `<select id="compose-overlay">` labelled `Overlay` with options `Off`, `Thirds`, `Grid`, `Golden
   ratio`, `Diagonals` (values as `OVERLAY_ORDER`); `o` in `useComposeKeys` calls `cycleOverlay`.
5. **Straighten control** (`StraightenControl.tsx`, replacing the slider block of `CompositionSection`): the slider (`#compose-straighten`, as P32)
   + a text field `<input id="compose-straighten-degrees" inputmode="decimal">` labelled `Straighten degrees` showing `straighten.toFixed(1)` + a
   `°` suffix + the switch `<button role="switch" aria-checked>` named `Straightening grid`. Field rules: **Enter** or **blur** commits the parsed
   text (`,` accepted as the decimal mark, `−` U+2212 as minus) through `SetValues { refit(values, { kind: 'straighten', degrees }) }` (refit
   clamps to ±45 and rounds to 0.1°); unparsable text commits nothing and restores the current value; Enter also blurs the field; **Esc** restores
   the current value, blurs, and is stopped from reaching the editor (no revert); **↑/↓** step the current value by `STRAIGHTEN_STEP` (Shift:
   `STRAIGHTEN_STEP_SHIFT`), each step one `SetValues` (`cause 'set'`), the field keeping focus; keys inside the field never reach the editor's
   key map (`isTyping`).
6. **Straightening grid** (`ComposeStage.tsx`): "straightening" = the slider is held (pointerdown on it until pointerup / pointercancel) **or** the
   degrees field has focus; while straightening and `straightenGrid` is on, the overlay SVG carries a `<pattern patternUnits="userSpaceOnUse"
   width={STRAIGHTEN_GRID_PX} height={STRAIGHTEN_GRID_PX}>` of thin lines and a `<rect class="compose-straighten-grid">` filling the whole stage
   with it (screen-aligned: it does not rotate with the photo, so tilted lines can be compared with it).
7. **Every shortcut has a control** (`EditorToolbar.tsx`, `CompositionSection.tsx`, `Stage.tsx`): the controls and hints of table T (C5(a)), all
   through P35's `Hint`, unavailable ones `aria-disabled` (never `disabled`). New controls and their actions:
   **stage bar** (`.stage-bar .editor-toolbar`, before Done, while composing; `Stage`'s `compose` prop gains `toolbar`): `Revert` → `KeyEsc`
   (as Esc); `Loupe` toggle (`aria-pressed`) → the loupe shows while the toggle is on **or** Z is held; `Snapping` toggle (`aria-pressed`,
   `compose.snap`) → when off every drag behaves as with ⌥ held; `Copy framing` → as ⌥⌘C; `Paste framing` → as ⌥⌘V, `aria-disabled` while the
   clipboard is empty; `Tools panel` toggle (`aria-pressed` = shown) → as ⌘T; `Frame meter` toggle (`aria-pressed`) → as ⌥⌘P.
   **Composition section**: a nudge pad `.compose-nudge` of four buttons `Nudge left|right|up|down` → `Nudge` exactly as the arrow keys (Shift-click
   ×10); `Revert` is not duplicated there. **Stage body**: while composing the `.stage-nav` buttons call `stepPhoto` (exactly `[` / `]`: display
   sequence, no wrap) instead of the workspace step. Existing controls lose `title` and gain `Hint` with the table's text.
8. **Focus back to the editor** (fold-in): after a `change` of `#compose-shape` or `#compose-overlay` the select is blurred; after a **pointer**
   release on `#compose-straighten` (pointerup) the slider is blurred. A slider focused with the keyboard keeps focus (P32 C6(b): its arrows
   belong to it). The degrees field follows task 5.
9. **Snap guide where it snapped** (fold-in, `snapping.ts`, `machine.ts`, `ComposeFrame.tsx`): `snap` reports which line won per axis (`centre`,
   `start` = the image's left/top edge, `end` = its right/bottom edge); `ComposeFrame` draws the x guide at `frameBox.left + frameBox.width/2`
   (centre), `frameBox.left` (start) or `frameBox.left + frameBox.width` (end), the y guide likewise; `useComposeGesture`'s publish check compares
   the new guide values (not booleans).
10. Tests (unit + component), the eight adjusted tests, `npm run typecheck`, `npm test`.

## Criteria (8 criteria, 22 rows; 20 `[rule]` rows — UI rows with no ledger measurement behind them, master plan §3)
Fixture: P31's (above) through the compose harness; numbers are hand-computed from it. Hints are read as in P35 (focus or pointer move →
`role="tooltip"` text, waiting up to 1 s). Line sets are compared with tolerance 0.01 px, order-free.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `<App>` workspace, panel unfolded, photo 1 active | press `r`; `pointermove` on `main.workspace` at `clientX = innerWidth − 5`; click tile 2 in the strip | `.folded-strip` is a child of `main.workspace` (not inside `.stage-column`) holding every photo's tile; no `aside.photo-panel`, no `.compose-browser`, nothing with `data-open` after the edge move; the tile click keeps composing and the `.compose-image img` src becomes photo 2's preview | `[rule]` 10D ("shows as its single strip … stays put: no hiding, no edge reveal … Clicking a photo in the strip moves the editor to it") |
| C1(b) | composing (from C1(a)) | press `f`; click tile 1 in the panel; click `Fold panel`; press Return | after `f`: `aside.photo-panel` beside the stage, no `.folded-strip`, still composing; tile 1 → editor on photo 1; Fold → the strip again; Return → the editor closes and the panel is unfolded (as before Compose); a second run that folds with `f` **before** Compose ends folded after Return | `[rule]` 10D ("F still unfolds and folds the panel"; 163857a "F stays and folds or unfolds the strip") |
| C2(a) | `overlays.test.ts`, `overlayLines` | (1) `thirds`, box `{174, 24, 652, 652}`; (2) `grid`, same; (3) `golden`, same; (4) `diagonals`, same; (5) `diagonals`, box `{24, 32.6047, 952, 634.7906}` (Original on 2560×1707); (6) `thirds`, box `{24, 112, 952, 476}`, split; (7) `off` | (1) verticals `x = 391.3333, 608.6667` (y 24→676), horizontals `y = 241.3333, 458.6667` (x 174→826); (2) verticals `x = 282.6667, 391.3333, 500, 608.6667, 717.3333`, horizontals `y = 132.6667, 241.3333, 350, 458.6667, 567.3333`; (3) `x = 423.0418, 576.9582`, `y = 273.0418, 426.9582`; (4) exactly two lines `(174, 24)–(826, 676)`, `(826, 24)–(174, 676)`; (5) four lines `(24, 32.6047)–(658.7906, 667.3953)`, `(976, 32.6047)–(341.2094, 667.3953)`, `(24, 667.3953)–(658.7906, 32.6047)`, `(976, 667.3953)–(341.2094, 32.6047)`; (6) verticals `x = 182.6667, 341.3333, 658.6667, 817.3333` (y 112→588), horizontals `y = 270.6667, 429.3333` drawn in each half (x 24→500 and 500→976): 8 lines; (7) `[]` | `[rule]` 10D ("Off, Thirds, Grid, Golden ratio, Diagonals"); changelog 28 reading (Photoshop's overlays) |
| C2(b) | composing, `localStorage` empty | read the overlay; press `o` four times; then choose `Golden ratio` in `#compose-overlay`; leave (Return) and re-open (r); separately `localStorage['compose.overlay'] = 'bogus'` before render | initial `.compose-overlay-guides[data-overlay="thirds"]` with 4 lines; `o` → `grid` (10 lines) → `golden` (4) → `diagonals` (2) → `off` (0 lines); the select → `golden`, `localStorage['compose.overlay'] === 'golden'`, and after re-opening still `golden`; `bogus` → `thirds` | `[rule]` 10D ("O cycles them) and is remembered") |
| C3(a) | composing, grid switch on (default) | focus `#compose-straighten-degrees`; blur; `pointerdown` on `#compose-straighten`, then `pointerup`; click `Straightening grid`; focus the field again; remount | `.compose-straighten-grid` absent at rest; present while the field has focus (its pattern `width="24"`, `height="24"`, `patternUnits="userSpaceOnUse"`), absent after blur; present between pointerdown and pointerup on the slider; the switch → `aria-checked="false"`, `localStorage['compose.straightenGrid'] === '0'`, and focusing the field shows no grid; after a remount the switch still reads off | `[rule]` 10D ("while straightening, a fine square grid covers the image … with its own on/off switch") |
| C4(a) | composing on the default square | type `10` in `#compose-straighten-degrees`, press Enter | exactly one `setComposition(1, v)` with `straighten 10` and rect shrunk about the centre to `h = 0.816 ± 0.005` (P32 re-fit (8)); the field reads `10.0`; the slider's value `10`; `document.activeElement` is not the field | 10D ("The straighten control takes a typed number of degrees"); MC-27 §Values (straighten in degrees) |
| C4(b) | composing on the default square, the field focused | press `ArrowUp`; `Shift+ArrowUp`; `ArrowDown` (each answered at once by the mock) | exactly three `setComposition` calls with `straighten` `0.1`, `1.1`, `1.0` in that order, each rect centred at `(0.5, 0.5) ± 1e-6` (a nudge would keep straighten `0` and move the centre); the field keeps focus and reads `1.0` | `[rule]` 10D ("and the up/down arrows (0.1° a step, Shift 1°)"); changelog 28 reading |
| C4(c) | composing on the default square, the field focused | type `50` + Enter; then type `abc` + Enter; then type `-3,5` + Enter; then type `7` and press Escape | `50` → one save with `straighten 45` (clamped); `abc` → no save, the field reads `45.0`; `-3,5` → one save `straighten -3.5`; `7` + Escape → no save, the field reads `-3.5`, it is blurred, and the editor did **not** revert (no further `setComposition`, `.compose-stage` present) | MC-27 §Values ("straighten: degrees, −45…45"); design handoff §Keyboard (keys inside a field belong to it) |
| C5(a) | composing on photo 1 (table T below; one test iterating every entry) | for each entry: find the control by role and accessible name inside its place; focus it | the control exists exactly there and its tooltip text equals the entry's hint, for all 30 entries | `[rule]` 10D ("each editor action reachable by a key also has a button or menu entry in the Tools panel or above the stage, and hovering any of them shows its name and key") |
| C5(b) | composing, empty clipboard (fresh module state) | click `Paste framing`; click `Copy framing` | before Copy: `Paste framing` has `aria-disabled="true"`, its tooltip still reads `Paste framing (⌥⌘V)`, and the click sends no `setComposition`; after Copy: no `aria-disabled` (its effect is C6(e)) | `[rule]` 10D ("every shortcut has a control"); P35 `aria-disabled` rule |
| C6(a) | composing; drag `(500, 350) → (550, 350)` and release (save `x = 0.115466`) | click `Revert` (stage bar) | one more `setComposition(1, v)` with `v.rect.x = 0.166601562` (the entry), exactly what Esc sends (P31 C5) | `[rule]` 10D ("Esc reverts this photo to how it was when the editor came to it") + "every shortcut has a control" |
| C6(b) | composing on photo 1 of twoPhotos | click `Next photo`; click `Next photo` again; click `Previous photo` | first → the editor on photo 2 (still composing); second → nothing changes (last photo, no wrap); third → photo 1 | `[rule]` 10D ("[ ] previous/next photo keeping changes") + control rule |
| C6(c) | composing; pointer at `(600, 300)` on the stage | click `Loupe`; click it again | after the first click `aria-pressed="true"` and `.compose-loupe` present at `left 616px, top 316px`; after the second `aria-pressed="false"` and no loupe; `setComposition` never called | `[rule]` 10D ("hold Z for a 100 % loupe that changes nothing") + control rule |
| C6(d) | composing; `Snapping` reads `aria-pressed="true"` | click `Snapping`; drag `(500, 350) → (503.7, 350)` and release; click `Snapping` again; drag `(500, 350) → (503.7, 350)` (pointer still down) | after the first click `aria-pressed="false"`, `localStorage['compose.snap'] === '0'`; the drag shows no `.compose-guide` and saves `x = 0.162818 ± 1e-6` (0.166601562 − 3.7/977.81: a drag to the right moves the photo right, so the frame's `x` falls — P31 C1(b), C7(a); see Notes); with snapping back on the move shows `.compose-guide[data-axis="x"]` | `[rule]` 10D ("Snapping to the centre and the image edges; ⌥ suspends it") + control rule |
| C6(e) | composing on photo 1 with values `V` (P34 C3(a) fixture: a 4:5 rect, straighten 2) | click `Copy framing`; `]` to photo 2 (frame 1707×2560); click `Paste framing` | one `setComposition(2, V')` with rect `(0.26002, 0.21998, 0.47995, 0.40004)` to 4 decimals, shape, orientation, turns, flip, straighten equal to `V` (= P34 C3(a)) | `[rule]` 10D (copy/paste — R37 phasing) + control rule |
| C6(f) | composing | click `Tools panel`; click it again | first: `.tools-panel` has `hidden`, the button `aria-pressed="false"`, `localStorage['tools.hidden'] === '1'`; second: shown, `aria-pressed="true"` | `[rule]` 10D ("hidden with ⌘T") + control rule |
| C6(g) | composing | click `Frame meter`; click it again | `.frame-meter` present with `aria-pressed="true"`, then absent | `[rule]` 10D §Performance targets (the meter, P34) + control rule |
| C6(h) | composing on the default square | click `Nudge right`; Shift-click `Nudge right` | two `setComposition` calls: `x = 0.167624 ± 1e-6` (0.166601562 + 1/977.81), then `x = 0.177852 ± 1e-6` (+ 10/977.81), `y = 0` | `[rule]` 10D ("arrows nudge one screen pixel (Shift ×10)") + control rule |
| C7(a) | composing on the default square | choose `4:5` in `#compose-shape` (fire `change`); press `x` | two `setComposition` calls: the 4:5 re-fit, then `orientation 'turned'` (the `x` key reached the editor); `document.activeElement` is not the select | `[rule]` fold-in: P32 narrative "Not yet" ("that control keeps the keyboard … until you click the photo") |
| C7(b) | composing on the default square | `pointerdown` on `#compose-straighten`; set it to `10` (`input`, then `change`); `pointerup`; press `x` | one save `straighten 10` (P32 C3(d)), then one save with `orientation 'turned'`; the slider is not focused after pointerup; P32 C6(b) (keyboard on a focused slider) still passes unchanged | `[rule]` fold-in: P32 narrative "Not yet"; design handoff §Keyboard |
| C8(a) | `snapping.test.ts`, threshold `6/977.81` both axes | `(0.1704, 0, 0.6668, 1)`; `(0.004, 0, 0.6668, 1)`; `(0.3292, 0, 0.6668, 1)` (right edge `0.996`); `(0.1, 0, 0.6668, 1)` | `x 0.1666`, guide x `centre`; `x 0`, guide x `start`; `x 0.3332`, guide x `end`; unchanged, guides `null` (y is `null` in all: `h = 1` sits on both edges at distance 0, which never snaps) | `[rule]` fold-in: P34 narrative "Not yet" ("the guide line is always drawn at the frame's centre"); 10D ("Snapping to the centre and the image edges") |
| C8(b) | composing on the default square | drag `(500, 350) → (659, 350)` (proposed `x = 0.003993`, within `6/977.81 = 0.006136` of the left edge) | during the move `.compose-guide[data-axis="x"]` has `x1 = x2 = 174` (the frame's left edge, not 500); on release one save with `x = 0` | `[rule]` fold-in (as C8(a)) |

**Table T** (C5(a)) — place · role and accessible name · hint text:
Tools panel (`.tools-panel`): button `Normal` · `Normal (S)` — button `Split` · `Split (S)` — combobox `Shape` · `Shape (1–6, 0)` — button `Turn
frame` · `Turn frame (X)` — button `Rotate left` · `Rotate 90° left` — button `Rotate right` · `Rotate 90° right` — button `Flip` · `Flip
horizontally` — slider `Straighten` · `Straighten (double-click to level)` — textbox `Straighten degrees` · `Straighten in degrees (↑ ↓ 0.1°, ⇧ 1°)`
— switch `Straightening grid` · `Straightening grid` — combobox `Overlay` · `Overlay (O)` — buttons `Nudge left`, `Nudge right`, `Nudge up`, `Nudge
down` · `Nudge left (←)`, `Nudge right (→)`, `Nudge up (↑)`, `Nudge down (↓)` — button `Centre` · `Centre (.)` — button `Reset` · `Reset (⌘⌥R)` —
button `Done` · `Done (Return)`.
Stage bar (`.stage-bar`): button `Revert` · `Revert (Esc)` — button `Loupe` · `Loupe (hold Z)` — button `Snapping` · `Snapping (hold ⌥ to suspend)`
— button `Copy framing` · `Copy framing (⌥⌘C)` — button `Paste framing` · `Paste framing (⌥⌘V)` — button `Tools panel` · `Tools panel (⌘T)` —
button `Frame meter` · `Frame meter (⌥⌘P)` — button `Done` · `Done (Return)`.
Stage body (`.stage-body`): button `Previous photo` · `Previous photo ([)` — button `Next photo` · `Next photo (])`.
Browser: in `.folded-strip` button `Unfold panel` · `Unfold panel (F)`; after `f`, in `aside.photo-panel` button `Fold panel` · `Fold panel (F)`.
(30 entries. Keys without a control of their own, by reading: drag / pinch / ⌘-wheel are gestures; 1–6 and 0 are the Shape menu's entries; Esc
mid-gesture cancels the gesture; ⌘[ is swallowed, not an action.)

## Delegated choices
Icons and grouping of the stage-bar toolbar and the nudge pad (accessible names and hints are fixed); the overlay and grid colours and stroke
widths; whether the Overlay control is a select (default) or a segmented menu with the same accessible name `Overlay`; whether the straighten
field commits on blur when the text is unchanged (it must not save identical values); the guide stroke; the `° ` suffix styling; how
`useComposePrefs` shares state between the section and the stage (context or props).

## Manual try-out
1. Press R: the right side becomes the thin strip of photos and stays there; nothing slides in from the edge. Click a photo in the strip: the
   editor moves to it. Press F: the full panel opens beside the editor; F again folds it. Press Return: the panel is as you left it before R.
2. Thirds lines sit inside the frame. Press O repeatedly: Grid, Golden ratio, Diagonals, Off, Thirds. Pick one in the Overlay menu, quit and
   relaunch: it is remembered. Split a photo: each half gets its own lines.
3. Click into the degrees field: a fine grid covers the photo; type 2.5 and Enter: the photo tilts 2.5°. Press ↑ and ↓ (0.1°) and ⇧↑ (1°).
   Turn "Straightening grid" off: no grid next time. Drag the slider: the grid shows while you hold it.
4. Hover every button in the Tools panel and above the photo: each hint gives its name and key (for example "Split (S)", "Loupe (hold Z)").
   Click Loupe, Snapping, Copy framing, Paste framing, Tools panel, Frame meter, Revert, the nudge arrows and the ‹ › arrows: each does what its
   key does.
5. Pick a shape in the Shape menu and press X right away: the frame turns (no need to click the photo first). Drag the photo so its left edge
   meets the frame's left edge: the guide line appears on that edge, not in the middle.

## Notes
- **Planner readings (reversible; for the narrative's "Choices made for you")**:
  (1) The editor always opens with the strip folded; F unfolds it for that editing session only; leaving the editor restores the panel you had.
  (2) The default overlay is Thirds (Capture One's and Photoshop's crop default); O cycles Off → Thirds → Grid → Golden ratio → Diagonals.
  (3) Grid = a 6 × 6 grid; Golden ratio = lines at 38.2 % and 61.8 %; Diagonals = Photoshop's 45° corner lines (the two diagonals on a square).
  (4) In a split each half gets its own overlay (each half is its own picture).
  (5) "While straightening" = while the slider is held or the degrees field has the keyboard; the grid is screen-aligned with 24 px squares and
  covers the whole stage.
  (6) The degrees field accepts a comma as decimal mark; Esc in it cancels the typing and does not revert the photo.
  (7) A nudge pad gives the arrow keys a clickable form; zoom has no buttons because pinch and ⌘-scroll are gestures, not keys.
  (8) The Loupe button keeps the loupe on until clicked again (Z still shows it while held); the Snapping switch is remembered.
  (9) Esc while a hint is showing closes the hint first (Radix closes its layer on Esc); a second Esc reverts as before.
- **A sign slip in P34's plan, corrected here**: P34 C2(b) wrote the ⌥ drag of `+3.7 px` as `x = 0.166601562 + 3.7/977.81 = 0.170386`; a drag to
  the right moves the photo right under the fixed frame, so `x` falls to `0.162818` (P31 C1(b): a `−50/977.81` pan for a `+50 px` drag; P31 C7(a)).
  The P34 test asserts no value for that save, so nothing red today; C6(d) asserts the right one.
- **Why the strip is rendered by `ComposeMode`**: its tile click must go through the editor's `pick` (commit first, `SwitchPhoto`), which only
  `ComposeMode` holds; the workspace's own `folded` stays untouched, which gives "back as it was" for free.
- **jsdom limits**: the SVG lines and the pattern are asserted by their attributes; whether they look right over a photo, the grid's density
  and the hint's position are for the manual try-out. Tooltips are Radix DOM nodes, so they are checkable in jsdom (P35).
- **Hazards from P31–P34 reviews**: assert the exact values listed (hand-computed from the fixture), never `toContain` or a value read back
  from the implementation; a toggle row asserts both states; a "no save" row asserts the absence of the call after the action settles.
- Registry additions made by the planner in master plan §6.10 (P36 block). No intention-book change; no backend change.
- Estimated size: ~28 paths (16 source incl. one deletion, 4 new tests, 8 adjusted tests), ~1 300 lines (≈ 600 features, ≈ 700 tests).

## Review log

- **2026-11-05 implement r1 (codex).** Replaced the edge-reveal browser with the folded strip and F toggle; added alignment overlays, remembered overlay/grid/snap preferences, typed and stepped straightening, the straightening grid, shortcut controls and hints, focus handoff, and guides at the snapped line. Added one boundary test per 22 criterion rows and adjusted eight earlier tests for retired behaviour. Full Rust and frontend suites, clippy, fmt, and TypeScript are green.
- **Assumptions.** The plan's `C4(a)` height `0.816 ± 0.005` repeats an arithmetic slip corrected in P32 review; the existing refit contract and P32 test yield `1/(cos 10° + sin 10°) ≈ 0.8632`, so the P36 test asserts that value. For `C6(d)`, after the first unsnapped `+3.7 px` drag the photo has moved away from centre; repeating the drag cannot centre-snap within six pixels. The test clicks Revert before the second drag so it starts at the entry rect. `C1(a)`'s “nothing with data-open” is read as no edge browser with `data-open`, since the existing Tools section uses `data-open` for its own fold state.
- **Deviations.** The three criterion fixture corrections above are disclosed for review; no backend or other-phase product behaviour changed. The checkpoint includes this Review log and the tracker row because the wrapper requires one phase-perimeter commit.
