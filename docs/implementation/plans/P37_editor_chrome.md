---
phase: P37_editor_chrome
title: Editor chrome — an icon toolbar above the stage, a hover card with name, key and one line for every editor control, the Tools panel's hide button in its own header with a left rail when hidden, Rotate and flip in a folded drawer, Turn as an icon beside Shape, the straightening grid as a grid icon beside the degrees field, the nudge arrows as a folded keyboard pad, the frame meter without a button
difficulty: normal
projection: no
depends_on: [P36_editor_controls_and_overlays]
---
# P37 — Editor chrome: icon toolbar, hover cards, the Tools panel's own hide button and rail, the Rotate and flip drawer, the Turn and grid icons, the folded nudge pad, the frame meter without a button

## Goal
The owner's second round of editor corrections (10D §What the photographer sees as amended `3c7623e`, R39, changelog 29). Everything the
photographer sees in the editor's chrome changes; nothing the editor *does* changes. (1) The toolbar above the stage shows **icons, not names**:
Revert, Loupe, Snapping, Copy framing, Paste framing, each keeping its accessible name. (2) Every editor control's **hover card** gives its name,
its key and one short line of what it does ("Loupe · Z — 100 % view under the pointer"); the texts of table T2 are the contract. (3) The **frame
meter** loses its toolbar button and keeps ⌥⌘P. (4) The **Tools panel's hide button** moves from the toolbar into the panel's own header; a hidden
panel leaves a slim **rail** on the left edge whose button (with a hover card) shows it again; ⌘T and `tools.hidden` are unchanged. (5) Rotate 90°
left/right and Flip move into a folded **Rotate and flip** drawer of the Composition section, as icons, closed at every editor open; **Turn frame**
(X) becomes an icon beside the Shape menu. (6) The **straightening grid** switch becomes a grid icon beside the degrees field (still a switch,
still remembered). (7) The nudge buttons become a small **keyboard-arrow pad** (← ↑ → ↓ as on a keyboard, the inverted T) folded behind one
arrows icon, closed at every editor open; Shift ×10 kept.
**Not in this phase:** any key, gesture, save, overlay, straighten or strip behaviour (P31–P36 stand and their tests keep passing, adjusted only
where this phase changes a visible name into an icon or a hint text into a card); the Sessions, Import and Compose buttons (workspace chrome,
P35); colour and light tools; the save-queue unification and the typed-degrees-then-drag note (P36 narrative "Not yet"). No backend change.

## Read first
Book: `10D_composition_and_framing.md` §What the photographer sees (bullets "A Tools panel …" — the header's hide button and the rail, the Turn
icon beside Shape, the Rotate and flip drawer, the grid icon, the arrow pad; "The toolbar above the stage …" — icons, the hover card with its
example, the frame meter without a button; "Every shortcut has a control") · `15_decisions.md` R38, R39 · `16_changelog.md` entry 29 (readings:
the meter is a measuring aid and loses its button; the pad and the drawer start closed each time the editor opens) · process `flow.md` §4, §6,
§9 · master plan §3 (trace conventions), §6.10 (P35/P36 block and this phase's P37 block) · plans `P35_composed_display_and_chrome.md` (task 1:
`Hint`, the `aria-disabled` rule, why a DOM tooltip), `P36_editor_controls_and_overlays.md` (task 7 and table T — superseded here by T2; Notes (7),
(8), (9); C6(h) nudge values `0.167624` / `0.177851`), `P32_composition_tools.md` (C3(b) Turn, C3(c) Rotate and Flip: `ROTATED` = the square on
the turned frame), `P31_compose_mode.md` (fixture: stage `1000 × 700` at `(0, 0)`, frame `2560 × 1707`, default square `frameBox {174, 24, 652,
652}`, `W_t·s = 977.81`) and the narrative `narratives/P36_editor_controls_and_overlays.md` ("Choices made for you", "Not yet").
Code at `3c7623e` (line numbers as of that commit):
- `components/ui/tooltip.tsx:4–13` (`HINT_DELAY_MS`, `Hint({ text, children })`: Radix Tooltip, own provider, content `.hint-tooltip`).
- `features/compose/EditorToolbar.tsx:13–23` (the button factory; name as text content; `Tools panel :21`, `Frame meter :22`).
- `features/compose/ToolsPanel.tsx:17–33` (`aside.tools-panel[aria-label="Tools"] hidden={hidden}`; sections with `data-open`),
  `useToolsPanelState.ts:4, 49–58` (`TOOLS_HIDDEN_KEY = 'tools.hidden'`, `hidden`, `toggleHidden`; the guarded `localStorage` pattern).
- `features/compose/CompositionSection.tsx:83–91` (the Shape `<label>` wrapping its `<select id="compose-shape">`), `:93–97` (Overlay), `:99–112`
  (`.compose-buttons`: Turn frame, Rotate left, Rotate right, Flip — text buttons with icons), `:114` (`<StraightenControl>`), `:115–118`
  (`.compose-nudge`: four text buttons), `:122–132` (Centre, Reset, Done).
- `features/compose/StraightenControl.tsx:48–61` (`.compose-degrees`: label, `#compose-straighten-degrees`, the `°` span), `:62` (the switch
  `Straightening grid` with text).
- `features/compose/ComposeMode.tsx:87` (`tools = useToolsPanelState()`), `:185–187` (`useComposeKeys` — `onToggleTools`, `onToggleMeter`),
  `:191–202` (`<ToolsPanel hidden …>`), `:215` (`<EditorToolbar … tools onTools meter onMeter>`), `:219` (`{meter && <FrameMeter>}`).
- `features/compose/useComposeKeys.ts:49–51` (⌘T), `:59–60` (⌥⌘P) — unchanged.
- `features/workspace/Stage.tsx:76–78` (Done), `:87–89` (Previous photo), `:97–99` (Next photo); `FoldedStrip.tsx:14–16` (Unfold panel);
  `PhotoPanel.tsx:33–35` (Fold panel) — their `Hint` texts become cards.
- `features/compose/compose.css:38–43` (`.editor-toolbar`, `.compose-nudge`, `.compose-degrees`), `:62` (`.compose-readout`, dead since P36),
  `:37` (`.compose-straighten-grid + path`, dead — backlog), `:63` (`.compose-buttons`); `features/workspace/workspace.css:8` (`.tools-panel`),
  `:18–20` (`.stage-icon-button`), `:25` (`.hint-tooltip`), `:97` (`.folded-strip` — the pattern for the rail).
- Tests that **change their expectation** in this phase (behaviour changed by R39 / changelog 29): `features/compose/__tests__/
  p36_controls.test.tsx` C5(a) `:12–42` (deleted: C2(a) and table T2 replace it), C5(b) `:48` (`'Paste framing (⌥⌘V)'` → the T2 card text),
  C6(f) `:111–118` and C6(g) `:119–127` (deleted: the stage-bar `Tools panel` and `Frame meter` buttons are retired; C4 and C3 replace them),
  C6(h) `:128–137` (opens the pad with the `Nudge` toggle before clicking `Nudge right`); `p32_controls.test.tsx` C3(c) `:85–113` (opens the drawer
  with `Rotate and flip` before `Rotate right :87`, `Rotate left :97, :99` and, after the second `editor(...)`, before `Flip :109`). Unchanged and
  must keep passing: `p32_tools_panel.test.tsx` (`panel()`, `shown()` `:6–8`; ⌘T `:43–49`), `p36_straighten.test.tsx` (the switch by
  `[role="switch"][aria-label="Straightening grid"]` `:24–31`), `p36_strip_overlays.test.tsx` (`[aria-label="Fold panel"]` `:29, :34`),
  `p34_meter.test.tsx` (⌥⌘P `:12, :22`), `p35_chrome.test.tsx` (text hints `Compose (R)`, `Details (D)`, `Sessions`), and every other P31–P36 test.

## Files expected to change
Frontend (changed): `apps/desktop/src/components/ui/tooltip.tsx`, `apps/desktop/src/features/compose/{EditorToolbar.tsx, ToolsPanel.tsx,
CompositionSection.tsx, StraightenControl.tsx, ComposeMode.tsx, compose.css}`, `apps/desktop/src/features/workspace/{Stage.tsx, FoldedStrip.tsx,
PhotoPanel.tsx, workspace.css}`.
Tests (new): `apps/desktop/src/features/compose/__tests__/p37_editor_chrome.test.tsx`.
Tests (changed): `p36_controls.test.tsx`, `p32_controls.test.tsx` (as listed in Read first).
Estimated: 11 source paths, 1 new test, 2 adjusted tests.

## Core mechanisms (unit tests allowed)
None. The hover text is asserted where the photographer reads it (the `role="tooltip"` element); everything else is DOM shape and saved values.

## Ordered tasks
0. **Registry first (master plan §6.10, P37 block).** `components/ui/tooltip.tsx` exports `type HintCard = { name: string; key?: string;
   description: string }` and `Hint({ text, children })` now takes `text: string | HintCard`. A string renders as today (the workspace's `Compose
   (R)`, `Details (D)`, `Sessions`, `Import photos` keep working). A card renders inside `Tooltip.Content` as `<span class="hint-card"><strong
   class="hint-name">{name}</strong>` + (when `key` is set) ` · <kbd class="hint-key">{key}</kbd>` + ` — <span class="hint-description">
   {description}</span></span>`, so that the tooltip's text content, whitespace collapsed to single spaces and trimmed, is exactly
   **`name · key — description`** (U+00B7 middle dot, U+2014 em dash, one space each side) or **`name — description`** without a key. Accessible
   names fixed here: `Hide Tools panel`, `Show Tools panel`, `Rotate and flip`, `Nudge`. CSS: `.tools-panel-head`, `.tools-panel-title`,
   `.tools-rail`, `.compose-drawer`, `.compose-drawer-toggle`, `.compose-drawer-body`, `.compose-nudge-pad`, `.hint-card`, `.hint-name`,
   `.hint-key`, `.hint-description`. Retired: the stage-bar buttons `Tools panel` and `Frame meter`, `EditorToolbar`'s props `tools`, `onTools`,
   `meter`, `onMeter`, the `.compose-buttons` row of `CompositionSection`, the text content of every icon control named below. `localStorage` keys
   unchanged (`tools.hidden`, `tools.sections`, `compose.*`); no new key — the drawer and the pad are not remembered.
1. **Icon toolbar** (`EditorToolbar.tsx`): five buttons in this order — `Revert`, `Loupe`, `Snapping`, `Copy framing`, `Paste framing` — each
   `<button type="button" class="compose-tool" aria-label={name}>` holding exactly one lucide icon (`aria-hidden`) and **no text**; `aria-pressed`
   on Loupe and Snapping, `aria-disabled` on Paste framing while the clipboard is empty (P35 rule), hover cards from T2. The `Tools panel` and
   `Frame meter` buttons go, with their props; `ComposeMode.tsx:215` drops `tools`/`onTools`/`meter`/`onMeter`; `useComposeKeys` keeps ⌘T and
   ⌥⌘P (`ComposeMode.tsx:185–187` unchanged) and `{meter && <FrameMeter>}` stays.
2. **Tools panel header and rail** (`ToolsPanel.tsx`, `ComposeMode.tsx`, `workspace.css`): `ToolsPanel` gains `onToggleHidden: () => void`
   (`tools.toggleHidden`). Shown (`hidden === false`): `<aside class="tools-panel" aria-label="Tools">` whose **first child** is `<div
   class="tools-panel-head">` with `<span class="tools-panel-title">Tools</span>` and the icon button `aria-label="Hide Tools panel"`
   (`stage-icon-button`, `PanelLeftClose`, card T2); the sections follow as today. Hidden (`hidden === true`): the aside is **not rendered**; in
   its place `<aside class="tools-rail" aria-label="Tools">` holding exactly one button, `aria-label="Show Tools panel"` (`PanelLeftOpen`, card
   T2). Both buttons call `onToggleHidden`. The rail is a flex child of `main.workspace` before `.stage-column` (as the panel is), slim (width
   Delegated, ≤ 40 px), `border-right: 1px solid var(--hair)`, the button at its top — the mirror of `.folded-strip` on the right. The `hidden`
   attribute is no longer used. `useToolsPanelState` is unchanged: `tools.hidden` `'1'` is written when hidden and removed when shown, read at
   mount, so a hidden panel is hidden at the next editor open and ⌘T toggles exactly as today.
3. **Composition section row for Shape and Turn** (`CompositionSection.tsx:83–91, 99–102`): the Shape field becomes `<div class="compose-field">
   <label htmlFor="compose-shape">Shape</label><select id="compose-shape">…</select><button aria-label="Turn frame">…</button></div>` — the
   select and the Turn button are **children of the same element**; Turn is icon-only (`RectangleVertical` or Delegated, `aria-hidden`), same
   `onClick` (orientation named ⇄ turned, through `set`), card T2. The select keeps its `blur()` on change (P36 task 8).
4. **Rotate and flip drawer** (`CompositionSection.tsx:103–111`, `ComposeMode.tsx`, `compose.css`): the `.compose-buttons` row is replaced by
   `<div class="compose-drawer" data-open={open}>` with a toggle `<button type="button" class="compose-drawer-toggle" aria-expanded={open}
   aria-controls="compose-rotate-flip" aria-label="Rotate and flip">` (an icon plus a chevron, Delegated; card T2) and, **only while open**,
   `<div id="compose-rotate-flip" class="compose-drawer-body">` with three icon-only buttons in this order: `aria-label="Rotate left"`
   (`RotateCcw`), `aria-label="Rotate right"` (`RotateCw`), `aria-label="Flip"` (`FlipHorizontal2`), the same `set` calls as today
   (`{ kind: 'turn', direction }`, `{ kind: 'flip' }`), cards T2. The open state lives in **`ComposeMode`** (`useState(false)`, passed as
   `rotateFlipOpen` / `onRotateFlipOpen`): it starts closed at every editor open (every `ComposeMode` mount), and is kept while the editor switches
   photos (`[` `]`, the strip), while the Tools panel is hidden and shown, and while the Composition section is folded and unfolded (Notes (1)).
5. **Straightening grid icon** (`StraightenControl.tsx:48–62`): the switch moves **into `.compose-degrees`** (the same parent as
   `#compose-straighten-degrees`, after the `°` span): `<button type="button" role="switch" aria-checked={grid} aria-label="Straightening grid"
   class="compose-tool">` holding one icon (`Grid3x3`, `aria-hidden`) and no text; `onClick → onGrid(!grid)` and `compose.straightenGrid` as today;
   card T2. Nothing else in the control changes (the slider, the field, the P36 key rules).
6. **Nudge pad** (`CompositionSection.tsx:115–118`, `ComposeMode.tsx`, `compose.css`): `.compose-nudge` becomes a drawer like task 4: toggle
   `<button class="compose-drawer-toggle" aria-expanded={open} aria-controls="compose-nudge-pad" aria-label="Nudge">` (`Move` icon, card T2) and,
   **only while open**, `<div id="compose-nudge-pad" class="compose-nudge-pad" role="group" aria-label="Nudge">` with four icon-only buttons in
   **DOM order** `Nudge up`, `Nudge left`, `Nudge down`, `Nudge right` (`ArrowUp`, `ArrowLeft`, `ArrowDown`, `ArrowRight`), laid out by CSS grid
   as a keyboard's inverted T (`grid-template-areas: ". up ." "left down right"`); each click dispatches `Nudge` exactly as today (`dx = −x·step`,
   `dy = −y·step`, `step = event.shiftKey ? 10 : 1`), cards T2. The open state lives in `ComposeMode` (`nudgeOpen` / `onNudgeOpen`), closed at
   every editor open, kept otherwise (as task 4).
7. **Hover cards everywhere** (`CompositionSection.tsx`, `StraightenControl.tsx`, `EditorToolbar.tsx`, `ToolsPanel.tsx`, `Stage.tsx:76–99`,
   `FoldedStrip.tsx:14–16`, `PhotoPanel.tsx:33–35`): every control of table T2 passes its card to `Hint`; no editor control keeps a plain string
   or a `title`. The workspace-only hints (`Compose (R)` / `COMPOSE_WAITING`, `Details (D)`, `Sessions`, `Import photos`) stay strings.
8. **CSS** (`compose.css`, `workspace.css`): `.hint-card` (name bold, key as `kbd` in the tooltip's font, description muted or plain — Delegated),
   `.tools-panel-head` (height 40 px like a section header, title + button, bottom hairline), `.tools-rail`, `.compose-drawer*`,
   `.compose-nudge-pad` (3 × 2 grid, 28 px buttons), icon-only `.compose-tool` padding; the `.editor-toolbar` keeps its row. While there, drop the
   dead `.compose-readout` and `.compose-straighten-grid + path` rules (backlog, no behaviour).
9. Tests: `p37_editor_chrome.test.tsx` (C1–C8), the two adjusted tests, `npm run typecheck`, `npm test`.

## Criteria (8 criteria, 12 rows; 10 `[rule]` rows — UI rows with no ledger measurement behind them, master plan §3)
Fixture: P31's (above) through the compose harness (`twoPhotos`, `layoutStage`, `openWorkspace`, `compose`); numbers are hand-computed from it.
Hover cards are read as in P36: focus the control (after blurring the previous one) → the `role="tooltip"` element, waiting up to 1 s; its text
is `textContent.replace(/\s+/g, ' ').trim()`. "Icon-only" = `textContent.trim() === ''` and exactly one `svg` descendant. `localStorage` is
cleared of `tools.hidden` before each test unless the row sets it.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | composing on photo 1 | read `.stage-bar .editor-toolbar`; look for buttons named `Frame meter` and `Tools panel` in the document | the toolbar holds exactly five buttons, in DOM order with accessible names `Revert`, `Loupe`, `Snapping`, `Copy framing`, `Paste framing`, each icon-only; `Loupe` and `Snapping` carry `aria-pressed` (`false`, `true`), `Paste framing` carries `aria-disabled="true"` (empty clipboard); no button named `Frame meter` or `Tools panel` anywhere; the stage bar still holds a button `Done` | `[rule]` 10D ("The toolbar above the stage shows icons, not names … Revert, Loupe, Snapping, Copy framing … Paste framing"); R39 |
| C2(a) | composing on photo 1 (table T2; one test iterating every entry, performing the listed action before entries that need it) | for each entry: find the control by role and accessible name inside its place; focus it | the control exists exactly there and its tooltip text equals the entry's hover text, for all 32 entries | `[rule]` 10D ("Hovering any editor control shows a small card with its name, its key and one short line of what it does"); R39 |
| C3(a) | composing on photo 1 | look for a button named `Frame meter` anywhere; press ⌥⌘P; press ⌥⌘P | none exists; `.frame-meter` present after the first press and absent after the second (P34 C7(a) keeps passing) | `[rule]` 10D ("The frame meter … keeps its key (⌥⌘P) and has no button"); changelog 29 reading |
| C4(a) | composing, `tools.hidden` absent | read `main.workspace`'s first child; click the button `Hide Tools panel` | before: `aside.tools-panel` is `main.workspace`'s first child and **its** first child is `.tools-panel-head` holding the button `Hide Tools panel` and the text `Tools`; after: no `.tools-panel`; `aside.tools-rail[aria-label="Tools"]` is `main.workspace`'s first child, its next sibling is `.stage-column`, and it holds exactly one button, named `Show Tools panel`; `localStorage['tools.hidden'] === '1'`; `.compose-stage` still present | `[rule]` 10D ("Its own header carries the button that hides it (⌘T); hidden, it leaves a slim rail on the left edge with the button that shows it again"); R39 |
| C4(b) | from C4(a), hidden | click `Show Tools panel`; press ⌘T; press ⌘T | after the click: `aside.tools-panel` is the first child again with `#compose-shape` inside, no `.tools-rail`, `localStorage['tools.hidden']` is not `'1'`; first ⌘T → the rail again (`tools.hidden === '1'`), second ⌘T → the panel (P32 C2(a) keeps passing unchanged) | `[rule]` 10D (as C4(a)); master plan §6.10 (`tools.hidden`, ⌘T unchanged) |
| C4(c) | `localStorage['tools.hidden'] = '1'` before the editor opens | press `r`; press Return | while composing: `.tools-rail` is `main.workspace`'s first child and no `.tools-panel` exists (remembered); after Return: neither `.tools-rail` nor `.tools-panel` exists and `main.workspace`'s first child is `.stage-column` | `[rule]` 10D ("remembered across launches … (⌘T)"); P32 C2(a) |
| C5(a) | composing on the default square | read the Composition section; click `Rotate and flip`; click `Rotate right` | before: a button `Rotate and flip` with `aria-expanded="false"`, no element `#compose-rotate-flip`, and no button named `Rotate left`, `Rotate right` or `Flip` anywhere; after the toggle: `aria-expanded="true"` and `#compose-rotate-flip.compose-drawer-body` holding exactly three icon-only buttons in DOM order `Rotate left`, `Rotate right`, `Flip`; the click → exactly one `setComposition(1, v)` with `quarter_turns 1` and rect `(0, 0.166602, 1, 0.666797) ± 1e-6` (P32 C3(c) `ROTATED`: the square on the turned frame 1707 × 2560) | MC-27 §Values (`quarter_turns` 0–3 clockwise; default rect of the shape on the turned frame); 10D ("Rotate 90° left/right and flip … sit as icons in a folded Rotate and flip drawer") |
| C5(b) | composing on the default square | find the button `Turn frame`; click it | the button is icon-only and `button.parentElement === document.querySelector('#compose-shape').parentElement`; the click → exactly one `setComposition(1, v)` with `orientation 'turned'`, `shape '1:1'` and the rect unchanged `(0.166601562, 0, 0.666796875, 1) ± 1e-6` (a square has no turned form — `refit` re-shapes to the same ratio about the same centre and area; P32 C3(b) keeps asserting the 4:5 case) | MC-27 §Values (`orientation` through the shape's turned form); 10D ("the Shape menu with a Turn icon beside it (portrait/landscape, X)") |
| C6(a) | composing, `compose.straightenGrid` absent | find the switch `Straightening grid`; click it | the switch has `role="switch"`, is icon-only, and `switch.parentElement === document.querySelector('#compose-straighten-degrees').parentElement`; before the click `aria-checked="true"`; after: `aria-checked="false"` and `localStorage['compose.straightenGrid'] === '0'` (the grid's behaviour stays P36 C3(a)'s, which keeps passing) | `[rule]` 10D ("Beside it a grid icon switches the straightening grid"); R39 |
| C7(a) | composing on the default square | read the Composition section; click `Nudge` | before: a button `Nudge` with `aria-expanded="false"`, no `#compose-nudge-pad`, and no button named `Nudge left`, `Nudge right`, `Nudge up` or `Nudge down` anywhere; after: `aria-expanded="true"` and `#compose-nudge-pad[role="group"]` holding exactly four icon-only buttons in DOM order `Nudge up`, `Nudge left`, `Nudge down`, `Nudge right` | `[rule]` 10D ("The nudge arrows are a small keyboard-arrow pad (← ↑ → ↓ laid out as on a keyboard), folded behind one arrows icon and shown only when the photographer opens it") |
| C7(b) | from C7(a), pad open | click `Nudge right`; Shift-click `Nudge right`; click `Nudge left`; click `Nudge up` | exactly three `setComposition(1, v)` calls, each with `y = 0`: `x = 0.167624 ± 1e-5` (0.166601562 + 1/977.81); `x = 0.177851 ± 1e-5` (+ 10/977.81); `x = 0.176828 ± 1e-5` (− 1/977.81); the `Nudge up` click adds **no** call (the default square fills the height, `y = 0`, so the pan is clamped and the machine's commit sees an unchanged rect — `machine.ts:337–344`, `:193`) | `[rule]` 10D ("arrows nudge one screen pixel (Shift ×10)") + control rule; P36 C6(h) |
| C8(a) | composing on photo 1 of twoPhotos | click `Rotate and flip`; click `Nudge`; press `]`; press ⌘T twice; click the `Composition` section header twice; press Return; press `r` | after `]`, after the two ⌘T and after the two header clicks: both `Rotate and flip` and `Nudge` still read `aria-expanded="true"` (and `#compose-rotate-flip`, `#compose-nudge-pad` exist); after Return + `r`: both read `aria-expanded="false"` and neither body exists | `[rule]` changelog 29 reading ("the arrow pad and the drawer start closed each time the editor opens"); 10D ("closed at first") |

**Table T2** (C2(a)) — place · role and accessible name · hover text (the tooltip's normalised text; `·` U+00B7, `—` U+2014). Entries marked
† need the action in brackets first.
Tools panel head (`.tools-panel-head`): button `Hide Tools panel` · `Hide Tools panel · ⌘T — Folds the panel into a rail on the left`.
Rail (`.tools-rail`) †[click `Hide Tools panel`; read; click `Show Tools panel`]: button `Show Tools panel` · `Show Tools panel · ⌘T — Brings
the Tools panel back`.
Composition section (`.composition-section`): button `Normal` · `Normal · S — One frame, one file` — button `Split` · `Split · S — One 2:1 frame
cut into two square files, Left and Right` — combobox `Shape` · `Shape · 1–6, 0 — Original, Square, 4:5, 3:2, 2:3, 16:9 or Free` — button
`Turn frame` · `Turn frame · X — Portrait or landscape, same centre and area` — button `Rotate and flip` · `Rotate and flip — Quarter turns
and a mirror for the photo` — †[click `Rotate and flip`] button `Rotate left` · `Rotate 90° left — Turns the photo anticlockwise` — button
`Rotate right` · `Rotate 90° right — Turns the photo clockwise` — button `Flip` · `Flip horizontally — Mirrors the photo left to right` —
slider `Straighten` · `Straighten — Tilts the photo; double-click levels it` — textbox `Straighten degrees` · `Straighten degrees · ↑ ↓ — Typed
degrees; ↑ ↓ step 0.1°, ⇧ 1°` — switch `Straightening grid` · `Straightening grid — A fine grid over the photo while straightening` — combobox
`Overlay` · `Overlay · O — Guides inside the frame; O cycles them` — button `Nudge` · `Nudge · ← ↑ → ↓ — Moves the frame one screen pixel, ⇧
×10` — †[click `Nudge`] button `Nudge up` · `Nudge up · ↑ — One screen pixel up, ⇧ ×10` — button `Nudge left` · `Nudge left · ← — One screen
pixel left, ⇧ ×10` — button `Nudge down` · `Nudge down · ↓ — One screen pixel down, ⇧ ×10` — button `Nudge right` · `Nudge right · → — One
screen pixel right, ⇧ ×10` — button `Centre` · `Centre · . — Centres the frame on the photo` — button `Reset` · `Reset · ⌘⌥R — Back to the
default framing` — button `Done` · `Done · Return — Keeps the changes and leaves the editor`.
Stage bar (`.stage-bar`): button `Revert` · `Revert · Esc — Back to how this photo was when the editor opened` — button `Loupe` · `Loupe · Z —
100 % view under the pointer` — button `Snapping` · `Snapping · ⌥ — Snaps to the centre and the image edges; hold ⌥ to suspend` — button `Copy
framing` · `Copy framing · ⌥⌘C — Takes this photo's composition` — button `Paste framing` · `Paste framing · ⌥⌘V — Puts the copied composition
on this photo, refitted to its frame` — button `Done` · `Done · Return — Keeps the changes and leaves the editor`.
Stage body (`.stage-body`): button `Previous photo` · `Previous photo · [ — The photo before, keeping the changes` — button `Next photo` ·
`Next photo · ] — The photo after, keeping the changes`.
Browser: in `.folded-strip` button `Unfold panel` · `Unfold panel · F — Opens the full photo panel beside the editor`; †[press `f`] in
`aside.photo-panel` button `Fold panel` · `Fold panel · F — Folds the panel to the strip`.
(32 entries: 1 + 1 + 20 + 6 + 2 + 2. Controls without a key — Rotate and flip, Rotate left, Rotate right, Flip, Straighten, Straightening grid —
show `name — description`. The frame meter has no control (10D's exception). `Loupe`'s text is the book's example verbatim.)

## Delegated choices
The icons (suggested, all in lucide-react 1.61.0: `Undo2` Revert, `ZoomIn` Loupe, `Magnet` Snapping, `Copy`, `ClipboardPaste`, `PanelLeftClose` /
`PanelLeftOpen`, `RectangleVertical` Turn frame, `RotateCcw` / `RotateCw` / `FlipHorizontal2`, `Grid3x3`, `Move` Nudge, `ArrowUp` / `ArrowLeft` /
`ArrowDown` / `ArrowRight`; the drawer toggles' chevron) — accessible names and hover texts are fixed; the rail's width and the Show button's
position in it; the `.hint-card` typography; whether the drawer toggles also show a short label beside their icon (the accessible name stays the
`aria-label`); the pad's button size; whether `Hint` renders the card through a small `hintText(card)` helper or inline JSX (the normalised text
is the contract); keeping or dropping the `Tools` title text in the panel head (the head must hold the text `Tools` per C4(a) — keep it).

## Manual try-out
1. Press R. Above the photo there are five icons and Done; no "Frame meter", no "Tools panel". Hover each icon: a small card gives the name, the
   key and one line, for example "Loupe · Z — 100 % view under the pointer". Press ⌥⌘P: the frame meter appears; ⌥⌘P again hides it.
2. The Tools panel on the left has its own header, "Tools", with a hide button at the right. Click it: the panel folds into a thin rail on the
   left edge with one button. Hover it: "Show Tools panel · ⌘T — Brings the Tools panel back". Click it: the panel is back. Press ⌘T twice: the
   same. Hide the panel, press Return, press R: the rail is still there (remembered).
3. In Composition, the Shape menu has a small turn icon beside it: click it on a 4:5 frame and the frame turns. Below, "Rotate and flip" is a
   closed drawer: open it to see the three icons; Rotate right turns the photo under the frame. Press Return and R: the drawer is closed again.
4. Beside the degrees field a grid icon switches the straightening grid: click it off, click into the degrees field: no grid. Quit and relaunch:
   still off.
5. The nudge arrows are hidden behind one arrows icon. Click it: a small pad with ↑ on top and ← ↓ → below, as on a keyboard. Click → once
   (one pixel) and with ⇧ held (ten). Switch to the next photo with ]: the pad stays open. Press Return and R: it is closed again.
6. Hover every control in the Composition section, above the photo, the ‹ › arrows, and the strip's unfold button: each card reads name · key —
   one line (controls without a key: name — one line).

## Notes
- **Planner readings (reversible; for the narrative's "Choices made for you")**:
  (1) The Rotate and flip drawer and the nudge pad start closed every time the editor opens and are not remembered; within one editing session
  they stay as you left them while you switch photos, hide and show the Tools panel, or fold and unfold the Composition section.
  (2) The hidden Tools panel's rail is a thin strip on the left edge holding only the Show button, the mirror of the photo strip on the right.
  (3) The hover texts of table T2 are the planner's wording; the book fixes only the shape (name · key — one short line) and the Loupe example,
  which is used verbatim. Any line can be changed by saying so; the table is the contract the tests assert.
  (4) Controls without a key (Rotate 90° left/right, Flip, the straighten slider, the straightening grid, the Rotate and flip drawer) show
  `name — description`; the pad's and the drawer's toggles list the keys of what they hold.
  (5) The ‹ › stage arrows and the panel's Fold/Unfold buttons are the same buttons outside the editor, so their cards show there too.
  (6) The frame meter is reachable by ⌥⌘P only and is not in the hover table (10D's one exception).
  (7) The pad's DOM order is up, left, down, right (the inverted T read row by row); the keyboard layout is CSS (`grid-template-areas`), a
  manual try-out item, as jsdom does not lay out.
  (8) Icon-only buttons keep their accessible names through `aria-label`, so every existing test that finds a control by role and name keeps
  working once the drawer or pad it sits in is open.
- **Tests that change their expectation** are listed in Read first with line numbers (P36 C5(a) deleted in favour of C2(a)/T2; P36 C5(b) text;
  P36 C6(f)/(g) deleted in favour of C4/C3; P36 C6(h) and P32 C3(c) open the pad/drawer first). No other test may change; a test that goes red
  for any other reason is a defect of this phase.
- **Hazards from P35/P36 reviews**: a hint row that reads `toHaveTextContent` (substring) hides a wrong card — C2(a) compares the normalised text
  with `toBe`; a toggle row asserts both states; a "no button named X" clause is checked with `queryByRole('button', { name })` over the whole
  document (`null`), never by class. Radix closes an open tooltip on Esc before the editor sees it (P36 Notes (9)) — unchanged. Blurring the Shape
  select on change (P36 task 8) must survive the Shape row's restructure (P36 C7(a) keeps passing).
- **C5(b)'s rect**: `orientation` of a `1:1` shape has no visible effect (ratio 1 = 1/1), so the saved rect equals the default square; the row
  asserts the saved `orientation 'turned'` and that nothing else moved. P32 C3(b) remains the proof on a 4:5 frame.
- **C7(b)'s vertical click**: the default square has `h = 1`, so both vertical nudges are clamped (`settle` → `clampRect`) and `commit` emits
  nothing when `live` equals the stored rect (`machine.ts:193`): the row asserts the absence of a fourth call. The up/down buttons' direction is
  therefore proven by DOM order and the shared handler only; a landscape fixture with `16:9` would move `y`, but its scale differs from the
  square's (`s = 952/2560`) and is not worth a second fixture here.
- **jsdom limits**: the icons' look, the rail's width, the pad's inverted-T layout and the card's typography are manual try-out items; rows
  assert DOM shape, attributes, text and saved values.
- Registry additions made by the planner in master plan §6.10 (P37 block). No intention-book change; no backend change; no new `localStorage` key.
- Estimated size: ~14 paths (11 source, 1 new test, 2 adjusted tests), ~600 lines (≈ 300 features and CSS, ≈ 300 tests).

## Review log


- **2026-11-05, implement r1 (codex).** Built the five-icon stage toolbar, exact T2 hover cards, a Tools header and remembered left rail, closed Rotate and flip and Nudge drawers, the Turn icon beside Shape, and the grid icon beside degrees. Adjusted only the P36 and P32 tests named in Read first; added 12 P37 boundary tests. Assumptions: rail width 36 px; drawer toggles have visible short labels plus icons; hover card typography stays the existing tooltip's font. No deviations from the plan. Rust suite, clippy, fmt, typecheck and UI suite (348 tests) passed; the first full UI run encountered the previously documented intermittent P33 C4(d) failure and passed on rerun.
