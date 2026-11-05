---
phase: P38_editor_polish
title: Editor polish — a split middle line visible on any photo, on/off icons that show their state (filled accent on, outline off), Rotate and flip as one icon with an ordinary dropdown menu, the nudge pad as a dropdown from one arrows icon, both on one row with the straighten control, no inline drawers and no words
difficulty: normal
projection: no
depends_on: [P37_editor_chrome]
---
# P38 — Editor polish: the visible split line, filled on/off icons, the Rotate and flip menu, the Nudge dropdown, one row

## Goal
The owner's third round of editor corrections (10D §What the photographer sees as amended `3e4b760`, R40, changelog 30). Only how the
editor looks and how two groups of controls open changes; nothing the editor *saves* changes. (1) The **split's middle line** is visible on
any photo: a light line with a dark edge, at least 2 screen px in the editor and on the stage, and visible on tiles too. (2) Every **on/off
icon** shows its state at a glance: a filled accent icon when on, a plain outline when off — Loupe, Snapping and the straightening grid.
(3) **Rotate and flip** becomes one small icon with an ordinary dropdown menu (Rotate 90° left, Rotate 90° right, Flip horizontal) that
closes after a pick. (4) The **nudge pad** becomes a dropdown from one arrows icon: it stays open while its arrows are clicked and closes on
a click outside. (5) The two icons sit **on one row with the straighten control**, beside the grid icon; no inline drawers, no words
beside these icons. P37's hover-card table stays and is adjusted (T3 below).
**Not in this phase:** any key, gesture, save, overlay or straighten behaviour (P31–P37 stand; their tests change only where this phase
changes a drawer into a menu or a popover); the undo history (P39–P41); colour and light tools; any backend change.

## Read first
Book: `10D_composition_and_framing.md` §What the photographer sees (bullet "A Tools panel …": "one small rotate icon with a dropdown …
No inline drawer, no words beside the icon"; "On and off must be told apart at a glance … the same holds for every on/off icon in the
editor (Loupe, Snapping)"; "The nudge arrows … in a dropdown from one arrows icon … closed by a click outside; it stays open while its
arrows are clicked. The rotate and nudge icons sit on one row with the straighten control, no words"; bullet "Split: … The middle line, in
the editor, on the stage and on tiles, is visible on any photo … a light line with a dark edge, at least 2 screen pixels in the editor and
on the stage") · `15_decisions.md` R39, R40 · `16_changelog.md` entry 30 · process `flow.md` §4, §6, §9 · master plan §3, §6.10 (P37
block and this phase's P38 block) · plans `P37_editor_chrome.md` (tasks 4–6, table T2, Notes (1), (7), (8); C5(a) `ROTATED`, C7(b) nudge
values), `P35_composed_display_and_chrome.md` (task 1 `Hint`, the `aria-disabled` rule), `P31_compose_mode.md` (fixture: stage `1000 × 700`,
frame `2560 × 1707`, default square `frameBox {174, 24, 652, 652}`, `W_t·s = 977.81`) and the narrative `narratives/P37_editor_chrome.md`.
Code at `3e4b760` (line numbers as of that commit):
- `features/compose/ComposeFrame.tsx:33` (the split hairline `<line className="compose-middle">`), `compose.css:16–17` (`.compose-frame
  [data-kind="split"]`, `.compose-middle` 1 px `--hair-strong`).
- `features/compose/ComposedPicture.tsx:12` (`<span className="composed-split" style={{ left: '50%' }}>`), `features/workspace/
  workspace.css:95` (`.composed-split` 1 px white, opacity .9) — the stage and every tile use this one component (P35).
- `features/compose/EditorToolbar.tsx:12–20` (`button(card, icon, action, pressed?, unavailable?)`; Loupe and Snapping carry `aria-pressed`).
- `features/compose/StraightenControl.tsx:66–70` (`.compose-degrees`: label, `#compose-straighten-degrees`, the `°` span, the
  `Straightening grid` switch with `aria-checked`).
- `features/compose/CompositionSection.tsx:41–44` (props `rotateFlipOpen`, `onRotateFlipOpen`, `nudgeOpen`, `onNudgeOpen`), `:92–100`
  (the `Rotate and flip` drawer: toggle with `<span>Rotate and flip</span>`, body `#compose-rotate-flip` with `Rotate left` / `Rotate right` /
  `Flip`), `:102` (`<StraightenControl …>`), `:103–110` (the `Nudge` drawer: toggle with `<span>Nudge</span>`, body `#compose-nudge-pad
  [role="group"]`, the four `Nudge up|left|down|right` buttons, `controller.dispatch({ type: 'Nudge', dx: -x·step, dy: -y·step })`).
- `features/compose/ComposeMode.tsx:81–82` (`rotateFlipOpen`, `nudgeOpen` state), `:202` (passed to `CompositionSection`).
- `features/compose/useComposeGesture.ts:36` (`LAYER_OPEN = '[role="dialog"], [role="alertdialog"], [role="menu"]'`) and
  `useComposeKeys.ts:41` (editor keys are off while a layer matching it is open; a prevented key is ignored) — **the nudge dropdown must not
  match it** (task 5), the rotate menu must (task 4).
- `components/ui/popover.tsx` (Radix `Popover` wrappers, `radix-ui` 1.8.1; `DropdownMenu` comes from the same package), `components/ui/
  tooltip.tsx` (`Hint`, `HintCard`).
- `compose.css:45–56` (`.compose-drawer*`, `.compose-nudge-pad` grid areas), `:57–59` (`.compose-degrees`), `:76–78` (`.compose-tool`).
- Tests that **change their expectation** in this phase (behaviour changed by R40 / changelog 30): `features/compose/__tests__/
  p37_editor_chrome.test.tsx` C2(a) table `:33` (the `Rotate and flip` card text stays), `:34–36` (the three `Rotate left` / `Rotate right` /
  `Flip` entries are deleted: menu entries carry words, no card), `:65` (`await click('Rotate and flip'); … rows.slice(6,9)` deleted), `:67`
  (`click('Nudge')` now opens the dropdown; the pad rows stay); C5(a) `:104–…` (deleted: C3 below replaces it); C7(a) `:128–134` and C7(b)
  `:136–140` (deleted: C4 below replaces them); C8(a) `:142–147` (deleted: no drawer state exists any more); `p32_controls.test.tsx` C3(c) `:87`
  and `:110` (`click('Rotate and flip')` then pick the **menu item** `Rotate 90° right` / `Rotate 90° left` / `Flip horizontal` by
  `getByRole('menuitem', { name })` instead of clicking a button named `Rotate right` …); `p36_controls.test.tsx` C6(h) `:80–90` (`click(…,
  'Nudge')` now opens the dropdown; the `Nudge right` clicks are unchanged). Unchanged and must keep passing: `p33_split_editor.test.tsx`
  (`middle()` `:26` = `.compose-middle`, present while split, absent when Normal), `p33_split_grid.test.tsx` and `p35_composed_display.test.tsx`
  (`.composed-split` at `left 50%` on tiles and the stage), `p36_straighten.test.tsx` (the switch by role and `aria-label`), every other
  P31–P37 test.

## Files expected to change
Frontend (changed): `apps/desktop/src/features/compose/{ComposeFrame.tsx, ComposedPicture.tsx, EditorToolbar.tsx, StraightenControl.tsx,
CompositionSection.tsx, ComposeMode.tsx, compose.css}`, `apps/desktop/src/features/workspace/workspace.css`.
Tests (new): `apps/desktop/src/features/compose/__tests__/p38_editor_polish.test.tsx`.
Tests (changed): `p37_editor_chrome.test.tsx`, `p32_controls.test.tsx`, `p36_controls.test.tsx` (as listed in Read first).
Estimated: 8 source paths, 1 new test, 3 adjusted tests.

## Core mechanisms (unit tests allowed)
None. Everything is DOM shape, attributes and saved values at the editor's boundary.

## Ordered tasks
0. **Registry first (master plan §6.10, P38 block).** New CSS classes `.compose-middle-edge`, `.compose-menu`, `.compose-menu-item`,
   `.compose-dropdown`; the toggle marker `data-on` on `.compose-tool`; the SVG attributes `strokeWidth` `4` (edge) and `2` (line) on the
   split lines; the Radix roles: the rotate menu is `role="menu"` with `role="menuitem"` entries named exactly `Rotate 90° left`, `Rotate 90°
   right`, `Flip horizontal`; the nudge dropdown's content **is** `#compose-nudge-pad` with `role="group"` and `aria-label="Nudge"` (never
   `role="dialog"`: `LAYER_OPEN` would switch the editor keys off while it is open). Retired: `.compose-drawer`, `.compose-drawer-toggle`,
   `.compose-drawer-body`, `#compose-rotate-flip`, the buttons named `Rotate left`, `Rotate right`, `Flip`, the props `rotateFlipOpen`,
   `onRotateFlipOpen`, `nudgeOpen`, `onNudgeOpen` and the matching `ComposeMode` state, the `<span>` words of the two toggles. Accessible names
   kept: `Rotate and flip`, `Nudge`, `Nudge up|left|down|right`, `Straightening grid`, `Loupe`, `Snapping`.
1. **Split line in the editor** (`ComposeFrame.tsx:33`, `compose.css`): while `kind === 'split'` two lines at `x = middle` from the frame's
   top to its bottom, in this DOM order: `<line className="compose-middle-edge" strokeWidth={4} …>` then `<line className="compose-middle"
   strokeWidth={2} …>` (SVG presentation attributes, so the widths are the element's own, not the stylesheet's). CSS: `.compose-middle-edge
   { stroke: rgba(0, 0, 0, .6); shape-rendering: crispEdges }`, `.compose-middle { stroke: #fff; shape-rendering: crispEdges }` (the `stroke-width`
   rules of `:17` go). Dark edge under a light line: visible on a white wall and on a black cloth.
2. **Split line on the stage and tiles** (`ComposedPicture.tsx:12`, `workspace.css:95`): the span stays (`.composed-split`, `left: 50%`, the
   P33/P35 rows keep passing); its CSS becomes `width: 2px; margin-left: -1px; background: #fff; opacity: 1; box-shadow: 0 0 0 1px rgba(0, 0, 0,
   .6)` — the same component draws the stage and every tile, so tiles get the edge too (10D: "on tiles … visible on any photo"). jsdom cannot
   measure this; it is a manual try-out item (step 1).
3. **On/off icons show their state** (`EditorToolbar.tsx:12–20`, `StraightenControl.tsx:70`, `compose.css`): every toggle button renders its
   lucide icon with `fill={on ? 'currentColor' : 'none'}` and carries `data-on={on || undefined}` beside its `aria-pressed` (Loupe, Snapping) or
   `aria-checked` (Straightening grid). CSS: `.compose-tool[data-on] { color: var(--color-accent); border-color: var(--color-accent) }` and
   `.compose-tool[data-on] svg { fill: currentColor }`. Nothing about what the toggles *do* changes (P34/P36 rows keep passing).
4. **Rotate and flip as a dropdown menu** (`CompositionSection.tsx:92–100`): Radix `DropdownMenu` (`import { DropdownMenu } from 'radix-ui'`).
   Trigger: `<DropdownMenu.Trigger asChild><button type="button" className="compose-tool" aria-label="Rotate and flip"><RotateCw aria-hidden />
   </button></DropdownMenu.Trigger>` wrapped in `Hint` with the card `Rotate and flip — Quarter turns and a mirror for the photo` (Radix sets
   `aria-haspopup="menu"` and `aria-expanded`). Content (portal, `className="compose-menu"`): three `DropdownMenu.Item`s (`className=
   "compose-menu-item"`), each an icon plus its words, in this order and with exactly these accessible names: `Rotate 90° left` (`RotateCcw`,
   `onSelect → set({ kind: 'turn', direction: 'left' })`), `Rotate 90° right` (`RotateCw`, `direction: 'right'`), `Flip horizontal`
   (`FlipHorizontal2`, `set({ kind: 'flip' })`). The menu closes after a pick (Radix default) and on a click outside; while it is open its
   `role="menu"` matches `LAYER_OPEN`, so the editor's keys are off — the menu owns the keyboard, as any menu does. No hover cards on the items
   (they carry words).
5. **Nudge as a dropdown** (`CompositionSection.tsx:103–110`): Radix `Popover` with `modal={false}`. Trigger: `<Popover.Trigger asChild><button
   type="button" className="compose-tool" aria-label="Nudge"><Move aria-hidden /></button></Popover.Trigger>` with the card `Nudge · ← ↑ → ↓ —
   Moves the frame one screen pixel, ⇧ ×10`. Content (portal): `<Popover.Content id="compose-nudge-pad" role="group" aria-label="Nudge"
   className="compose-nudge-pad compose-dropdown" onOpenAutoFocus={event => event.preventDefault()} sideOffset={4}>` holding the four icon-only
   buttons exactly as P37 task 6 (DOM order `Nudge up`, `Nudge left`, `Nudge down`, `Nudge right`; the inverted-T grid areas; `Nudge` dispatch with
   `step = shiftKey ? 10 : 1`; their P37 cards). Clicks inside the content never close it (Radix dismisses on `pointerdown` outside and on Esc
   only); the editor's keys stay on while it is open (`role="group"`). Esc closes it: Radix prevents the default, so `useComposeKeys` ignores
   that Esc and nothing reverts (P36 Notes (9) pattern).
6. **One row** (`StraightenControl.tsx:66–70`, `CompositionSection.tsx`): `StraightenControl` takes `children?: ReactNode` and renders them
   inside `.compose-degrees` **after** the `Straightening grid` switch; `CompositionSection` passes the two triggers (task 4, then task 5) as
   those children, so the degrees row reads: label · field · `°` · grid icon · rotate icon · nudge icon, all `.compose-tool` icons of 28 px.
   The two drawers, their state in `ComposeMode` (`:81–82`, `:202`) and their props go.
7. **Hover-card table T3** (`p37_editor_chrome.test.tsx` C2(a)): T2 minus the three rotate/flip entries (29 entries); the `Rotate and flip`
   trigger keeps its card; the `Nudge` trigger keeps its card and the † action `click Nudge` now opens the dropdown. Every other P37 card is
   unchanged.
8. **CSS** (`compose.css`, `workspace.css`): tasks 1–3; `.compose-menu` (popover surface: `background var(--color-bg)`, hairline border, `var(--r)`,
   `var(--shadow-sm)`, 4 px padding, `z-index 100`), `.compose-menu-item` (28 px rows, icon + words, highlighted on `[data-highlighted]`), `.compose-
   dropdown` (the pad's surface, same tokens as the menu); drop `.compose-drawer`, `.compose-drawer-toggle`, `.compose-drawer-body` (`:45–50`) and
   the pad's `border-top`. The `.compose-nudge-pad` grid areas stay.
9. Tests: `p38_editor_polish.test.tsx` (C1–C6), the three adjusted tests, `npm run typecheck`, `npm test`.

## Criteria (6 criteria, 12 rows; 11 `[rule]` rows — UI rows with no ledger measurement behind them, master plan §3; C3(b) traces MC-27)
Fixture: P31's (above) through the compose harness (`twoPhotos`, `layoutStage`, `openWorkspace`, `compose`); numbers are hand-computed from it
(P37 C5(a), C7(b)). Hover cards are read as in P37 (focus → the `role="tooltip"` text, whitespace collapsed, `toBe`). "Icon-only" = `textContent
.trim() === ''` and exactly one `svg` descendant. Radix menus and popovers are opened with `fireEvent.pointerDown` + `click` on the trigger (the
P35 `Hint` rows show the harness pattern) and read by role.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | composing on photo 1 | press `s` (Split); read the frame SVG; press `s` again | while split: exactly one `line.compose-middle-edge` and exactly one `line.compose-middle`, the edge **before** the line in DOM order; with `rect` = the `.compose-frame` element's `x, y, width, height` attributes (the 2:1 frame of P33 C2(a)), both lines have `x1 === x2 === rect.x + rect.width / 2` (± 1e-6), `y1 === rect.y`, `y2 === rect.y + rect.height`, and `stroke-width` attributes `"4"` (edge) and `"2"` (line); after Normal: neither line exists (P33 C2(b) keeps passing) | `[rule]` 10D ("The middle line, in the editor … a light line with a dark edge, at least 2 screen pixels"); R40 |
| C2(a) | composing on photo 1 | read the toolbar button `Loupe`; click it; read again | before: `aria-pressed="false"`, no `data-on`, its `svg` has `fill="none"`; after: `aria-pressed="true"`, `data-on` present, `svg[fill="currentColor"]`; the loupe behaviour of P34/P36 is unchanged (no save) | `[rule]` 10D ("every on/off icon in the editor (Loupe, Snapping) … on is a filled accent icon, off a plain outline one"); R40 |
| C2(b) | composing, `compose.snap` absent (default on) | read `Snapping`; click it | before: `aria-pressed="true"`, `data-on`, `svg[fill="currentColor"]`; after: `aria-pressed="false"`, no `data-on`, `svg[fill="none"]`, `localStorage['compose.snap'] === '0'` (P36 C6(d) keeps passing) | `[rule]` 10D (as C2(a)) |
| C2(c) | composing, `compose.straightenGrid` absent | read the switch `Straightening grid`; click it | before: `aria-checked="true"`, `data-on`, `svg[fill="currentColor"]`; after: `aria-checked="false"`, no `data-on`, `svg[fill="none"]`, `localStorage['compose.straightenGrid'] === '0'` (P37 C6(a) keeps passing) | `[rule]` 10D ("the straightening grid … on is a filled accent icon, off a plain outline one") |
| C3(a) | composing on the default square | read the Composition section; open `Rotate and flip` | before: the button `Rotate and flip` is icon-only with `aria-haspopup="menu"` and `aria-expanded="false"`; no `role="menu"` in the document; no element `#compose-rotate-flip`, no `.compose-drawer`, and no button named `Rotate left`, `Rotate right` or `Flip` anywhere; after: exactly one `role="menu"` holding exactly three `menuitem`s with accessible names, in DOM order, `Rotate 90° left`, `Rotate 90° right`, `Flip horizontal` | `[rule]` 10D ("one small rotate icon with a dropdown opens an ordinary menu (Rotate 90° left, Rotate 90° right, Flip horizontal) … No inline drawer, no words beside the icon"); R40 |
| C3(b) | from C3(a), menu open | pick the menu item `Rotate 90° right` | exactly one `setComposition(1, v)` with `quarter_turns 1` and rect `(0, 0.166602, 1, 0.666797) ± 1e-6` (P32 C3(c) `ROTATED`); afterwards no `role="menu"` in the document and `Rotate and flip` reads `aria-expanded="false"` (closes after a pick) | MC-27 §Values (`quarter_turns`; the default rect on the turned frame); 10D ("that closes after a pick") |
| C4(a) | composing on the default square | read `Nudge`; open it | before: the button `Nudge` is icon-only, no `#compose-nudge-pad`, no button named `Nudge up|left|down|right` anywhere, no `.compose-drawer`; after: `#compose-nudge-pad[role="group"][aria-label="Nudge"]` exists, is **not** `role="dialog"`, and holds exactly four icon-only buttons in DOM order `Nudge up`, `Nudge left`, `Nudge down`, `Nudge right` | `[rule]` 10D ("The nudge arrows are a small keyboard-arrow pad … in a dropdown from one arrows icon, shown only when the photographer opens it") |
| C4(b) | from C4(a), pad open | click `Nudge right`; Shift-click `Nudge right`; click `Nudge left`; click `Nudge up` | exactly three `setComposition(1, v)` calls, each `y = 0`: `x = 0.167624`, `0.177851`, `0.176828` (± 1e-5; P37 C7(b)); the `Nudge up` click adds no call; **after every click `#compose-nudge-pad` is still in the document** (stays open while its arrows are clicked) | `[rule]` 10D ("it stays open while its arrows are clicked"; "arrows nudge one screen pixel (Shift ×10)"); P37 C7(b) |
| C4(c) | from C4(a), pad open | `fireEvent.pointerDown` on `.compose-stage` (outside the pad) | `#compose-nudge-pad` is gone; no `setComposition` call was made by the dismissal | `[rule]` 10D ("closed by a click outside") |
| C4(d) | composing, pad opened again, after one `Nudge right` | press `Escape` on `document.activeElement` (the trigger or body) | the pad is gone **and** no further `setComposition` call happened (Esc closed the layer; the editor did not revert: the single nudge save stays the last call) | `[rule]` 10D ("Esc reverts" is the editor's; a dismissed layer swallows its Esc — P36 Notes (9)) |
| C5(a) | composing | read `.compose-degrees` (the parent of `#compose-straighten-degrees`) | its children after the `°` span are, in DOM order, the switch `Straightening grid`, the button `Rotate and flip`, the button `Nudge` — each icon-only, each `button.parentElement === degreesField.parentElement`; the Composition section holds no `.compose-drawer`, `.compose-drawer-toggle` or `.compose-drawer-body` | `[rule]` 10D ("The rotate and nudge icons sit on one row with the straighten control, no words"); R40 |
| C6(a) | composing on photo 1 (table T3) | for each of the 29 entries: perform the † action if any, find the control by role and accessible name inside its place, focus it | the control exists exactly there and its tooltip text equals the entry's hover text, for all 29 entries (replaces P37 C2(a)) | `[rule]` 10D ("Hovering any editor control shows a small card with its name, its key and one short line of what it does"); R39, R40 |

**Table T3** (C6(a)) = P37's T2 with these changes only: the entries `Rotate left`, `Rotate right` and `Flip` are removed (they are menu entries
with words now, and a menu item has no hover card); the `Rotate and flip` entry's † action is none (the trigger is always visible); the
`Nudge` pad entries' † action is `[open Nudge]` (click the trigger). Count: 32 − 3 = **29** (1 + 1 + 17 + 6 + 2 + 2). Every text is T2's verbatim.

## Delegated choices
The icons (`RotateCw` for the trigger or any rotate glyph; `Move` for the nudge trigger); the dark edge's exact colour and alpha (a dark edge
is the contract, `rgba(0,0,0,.6)` the suggestion) and the menu/pad surface styling; whether `data-on` also tints the border; the menu's and
pad's `side`/`align` (default `bottom`, `start`); whether the items use `onSelect` or `onClick`; `onOpenAutoFocus` prevention on the pad (keep
the stage's keyboard where it was); whether `StraightenControl` renders its children through a `trailing` prop or `children` (the DOM contract
is C5(a)).

## Manual try-out
1. Open a dark photo (a black cloth or a night shot) and press S for Split: the middle line is a light line with a dark edge, clearly
   visible; press Return and look at the stage and the tile in the strip: the line is visible there too. Open a very light photo (a white
   wall): still visible.
2. Press R. Hover the Loupe icon above the photo and click it: the icon fills with the accent colour; click again: it is an outline again.
   The Snapping icon is filled (on) by default; the grid icon beside the degrees field is filled; click each: outline.
3. In Composition, the degrees row ends with three icons: grid, rotate, arrows. Click the rotate icon: a small menu opens with "Rotate 90°
   left", "Rotate 90° right", "Flip horizontal"; pick one: the photo turns or mirrors and the menu is gone. Open it and click elsewhere: gone.
4. Click the arrows icon: a small pad opens (↑ above ← ↓ →). Click → several times and ⇧-click once: the frame moves and the pad stays
   open. Click on the photo: the pad closes. Open it again and press Esc: it closes and the framing is not reverted.
5. No drawer, no "Rotate and flip" or "Nudge" words remain in the section; hovering the two icons still gives their cards.

## Notes
- **Planner readings (reversible; for the narrative's "Choices made for you")**:
  (1) The dark edge is a translucent black halo (`rgba(0,0,0,.6)`) 1 px around a 2 px white line in the editor; on the stage and tiles the
  same 2 px white line gets a 1 px dark shadow. Tiles get the edge too, though 10D's 2 px minimum names only the editor and the stage.
  (2) "Filled accent icon" = the lucide outline filled with `currentColor` and the button coloured with the accent (`--color-accent`).
  (3) The rotate menu's three entries show an icon and their words (a menu entry is text by nature; "no words" is about the icon row).
  (4) The nudge pad closes on a click outside and on Esc (as every dropdown does); Esc there does not revert the photo.
  (5) The pad does not steal keyboard focus when it opens, so arrow keys keep nudging exactly as before.
  (6) Neither dropdown is remembered: both are closed whenever the section renders fresh (editor open, panel shown again).
- **Tests that change their expectation** are listed in Read first with line numbers (P37 C2(a) table, C5(a), C7(a)/(b), C8(a); P32 C3(c);
  P36 C6(h)). No other test may change; a test that goes red for any other reason is a defect of this phase.
- **Hazards**: `LAYER_OPEN` (`useComposeGesture.ts:36`) — a Radix `Popover.Content` is `role="dialog"` by default, which would switch every
  editor key off while the pad is open; C4(a) pins `role="group"`. The rotate menu's `role="menu"` is meant to match it. Radix closes an open
  tooltip or layer on Esc before the editor sees it (P36 Notes (9)); C4(d) asserts the editor did not revert. Radix portals render outside
  `.composition-section`: rows find the menu and the pad by role over the document, the triggers inside the section. A `DropdownMenu.Item`
  fires `onSelect` on click and on Enter; the row uses `click`.
- **C1(a)'s middle**: read from the rendered `.compose-frame` rect (`x + width / 2`), not from a constant — the split re-fit depends on the
  square's centre and area (P33 C2(a)), and the lines must sit on the frame's exact middle whatever the rect.
- **jsdom limits**: the line's look over a photo, the filled icon's colour, the menu's and pad's placement are manual try-out items; rows
  assert DOM shape, attributes and saved values.
- Registry additions made by the planner in master plan §6.10 (P38 block). No intention-book change; no backend change; no new `localStorage`
  key.
- Estimated size: ~12 paths (8 source, 1 new test, 3 adjusted tests), ~450 lines (≈ 200 features and CSS, ≈ 250 tests).

## Review log

- **2026-11-05, implement r1 (codex).** Built the two-layer split line, 2 px stage/tile split marker, filled accent state for Loupe/Snapping/grid, a Radix rotate menu and nudge popover, and moved their icon triggers onto the degrees row. Replaced the retired drawer expectations and covered the 12 P38 boundary rows (the 29-card T3 check lives in the adjusted P37 test). Assumptions: use the suggested translucent black edge and token-based menu surfaces; keep the existing popover focus so editor arrows continue working; add a document capture pointer handler for nudge outside dismissal because the stage pointerdown did not dismiss Radix in the DOM boundary test. No deviations from the plan. Full frontend 355/355, Rust suite, clippy, fmt and typecheck pass. Visual contrast and placement await the owner's manual try-out.

