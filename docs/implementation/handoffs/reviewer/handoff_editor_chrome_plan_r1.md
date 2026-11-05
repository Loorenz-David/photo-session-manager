---
plan: editor_chrome
role: plan
round: 1
state: DONE
date: 2026-11-05
actor: Claude Fable 5.1 (pipeline-planner, implementation-planner doctrine under docs/process/flow.md)
---
# Editor chrome after the second try-out (book 10D amended `3c7623e`, R39, changelog 29) — planning handoff, round 1

**Summary.** Gate passed: `docs/intention/intention.md` is **RATIFIED** (owner 2026-10-24; the 10D amendment keeps it RATIFIED per changelog 29);
the mechanism inventory stands (MC-27…MC-29 are untouched by the amendment, which moves only 10D §What the photographer sees). I read `git show
3c7623e`, 10D in full, R38 and R39, changelog 28–29, flow.md, the master plan (§3–§6.10, §9), the previous planner handoff
(`handoff_composition_corrections_plan_r1.md`), the P36 plan (incl. its Review log) and narrative, and the code at `3c7623e` (`features/compose/*`
— `EditorToolbar`, `ToolsPanel`, `CompositionSection`, `StraightenControl`, `ComposeMode`, `useComposeKeys`, `useToolsPanelState`,
`useComposePrefs`, `compose.css`, `machine.ts` Nudge/commit, `refit.ts` —, `components/ui/tooltip.tsx`, `features/workspace/{Stage, FoldedStrip,
PhotoPanel}.tsx`, `workspace.css`, the compose harness and every test that names these controls). **One plan file written** (P37; 8 criteria /
12 rows / 10 `[rule]` rows / table T2 of 32 hover cards — counts derived from the file), `phases.txt` appended, the master plan amended. Not
committed (as instructed). **No owner card.** No `implementer: fable` (owner 2026-11-05).

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. Every open point is answered by the amended 10D or recorded as a reversible reading in the plan's Notes (surfaced to the
owner through the narrative's "Choices made for you").

## Write perimeter
- `docs/implementation/plans/P37_editor_chrome.md` (new).
- `docs/implementation/phases.txt` (one line appended with `printf >>`: `P37_editor_chrome`).
- `docs/implementation/master_plan.md` (§4 one NOT_STARTED row; §5 count 37, graph tail `→ P37`, `P37[P36]`, an editor-chrome note with the
  no-Fable rule; §6.10 a new P37 block; §9 log entry).
- `docs/implementation/handoffs/reviewer/handoff_editor_chrome_plan_r1.md` (this file).
- **Not touched:** `docs/intention/**`, `STATUS.md`, `backlog.md`, narratives, other plans, prompts, any code, any tool-recorded state. No commit,
  nothing pushed, no pipeline run, no live Capture One or Shopify call, no secret read.

## The phase
**P37_editor_chrome** (default implementer, `difficulty: normal`, `depends_on: [P36]`; 8 criteria / 12 rows / 10 `[rule]`). All seven owner
items, frontend only, ~14 paths (11 source, 1 new test, 2 adjusted tests), ~600 lines:
- **C1** icon toolbar: five icon-only buttons `Revert, Loupe, Snapping, Copy framing, Paste framing` (accessible names kept); the stage-bar
  `Tools panel` and `Frame meter` buttons retired with their `EditorToolbar` props.
- **C2** hover cards: `Hint` takes `text: string | HintCard` (`{ name, key?, description }`); the tooltip's normalised text is exactly `name · key
  — description` (or `name — description` without a key). **Table T2 (32 entries) is the contract**; the Loupe line is the book's example
  verbatim. Workspace-only hints (`Compose (R)`, `Details (D)`, `Sessions`, `Import photos`) stay strings.
- **C3** frame meter: no button anywhere; ⌥⌘P unchanged (P34 C7(a) keeps passing).
- **C4** Tools panel: `.tools-panel-head` ("Tools" + `Hide Tools panel`) as the panel's first child; hidden → the panel is not rendered and
  `aside.tools-rail` with one `Show Tools panel` button takes its place as `main.workspace`'s first child; `tools.hidden` and ⌘T unchanged;
  remembered at the next open; no rail outside the editor.
- **C5** `Rotate and flip` drawer (`aria-expanded`, body `#compose-rotate-flip` with icon buttons `Rotate left`, `Rotate right`, `Flip`), Rotate
  right saves `quarter_turns 1` with P32's `ROTATED` rect; `Turn frame` icon-only and a sibling of `#compose-shape`.
- **C6** `Straightening grid` switch icon-only, a sibling of `#compose-straighten-degrees`, `compose.straightenGrid` as today.
- **C7** `Nudge` drawer (`#compose-nudge-pad[role="group"]`, DOM order up, left, down, right; inverted-T by CSS grid); right / ⇧ right / left save
  `x = 0.167624 / 0.177851 / 0.176828`; a vertical nudge on the default square is clamped and saves nothing (`machine.ts:193`).
- **C8** both drawers start closed at every editor open and stay as left while switching photos, hiding/showing the panel or folding the section
  (state held in `ComposeMode`).

## Grounding worth knowing (orchestrator / reviewer)
- **Tests that change their expectation** are named with line numbers in the plan's Read first: `p36_controls.test.tsx` C5(a) `:12–42` (deleted —
  C2(a)/T2 replaces it), C5(b) `:48` (substring `'Paste framing (⌥⌘V)'` → the card text), C6(f) `:111–118` and C6(g) `:119–127` (deleted — C4 and C3
  replace them), C6(h) `:128–137` (opens the pad first); `p32_controls.test.tsx` C3(c) `:85–113` (opens the drawer before `Rotate right`, `Rotate
  left`, `Flip`). Every other P31–P36 test must keep passing unchanged: `p32_tools_panel` (`panel()`/`shown()` work with the panel not rendered
  when hidden), `p36_straighten` (switch by role + `aria-label`), `p36_strip_overlays` (`[aria-label="Fold panel"]`), `p34_meter` (⌥⌘P),
  `p35_chrome` (string hints).
- **Why the hidden panel is not rendered** (instead of `hidden`): the rail must take the panel's place as the first flex child of
  `main.workspace`; two siblings (a hidden aside plus a rail) would make "first child" ambiguous for the rows and the CSS. `useToolsPanelState`
  is untouched, so ⌘T, `tools.hidden` and the remembered state are exactly P32's.
- **Why the drawer state lives in `ComposeMode`**: `CompositionSection` unmounts when the section folds or the panel hides; the owner asked only
  that the drawer and pad be closed "at first", so the state is per editor open (one `ComposeMode` mount), not per section render.
- **Every expected number is hand-computed from the P31 fixture** (`W_t·s = 977.81`): nudge `0.167624`, `0.177851`, `0.176828`; `ROTATED` =
  `(0, 0.166602, 1, 0.666797)`; the square's own rect `(0.166601562, 0, 0.666796875, 1)`. The vertical-nudge "no save" follows from
  `machine.ts:337–344` (`settle` clamps) and `:193` (`commit` with an unchanged rect emits nothing).
- **Icons** are suggested, not fixed (all verified present in lucide-react 1.61.0: `Undo2`, `ZoomIn`, `Magnet`, `Copy`, `ClipboardPaste`,
  `PanelLeftClose`, `PanelLeftOpen`, `RectangleVertical`, `RotateCcw`, `RotateCw`, `FlipHorizontal2`, `Grid3x3`, `Move`, `ArrowUp/Left/Down/Right`).
- **jsdom limits** stated in the plan: icon look, rail width, the pad's inverted-T layout and the card's typography are manual try-out items; the
  P35 WKWebView probe (backlog) can measure layout if something looks off.

## Choices the owner may want to know (recorded in the plan's Notes; for the narrative)
1. The Rotate and flip drawer and the nudge pad start closed every time the editor opens and are not remembered; within one editing session they
   stay as left while switching photos, hiding/showing the Tools panel or folding the section.
2. The hidden Tools panel leaves a thin rail on the left holding only the Show button — the mirror of the photo strip on the right.
3. The hover texts are the planner's wording (the book fixes the shape and the Loupe example); any line can be changed by saying so.
4. Controls without a key (Rotate 90° left/right, Flip, the straighten slider, the straightening grid, the drawer toggle) show name — one line.
5. The ‹ › stage arrows and the panel's Fold/Unfold buttons show their cards outside the editor too (same buttons).
6. The frame meter is reachable by ⌥⌘P only.

## Criteria counts
| Phase | Criteria | Rows | `[rule]` rows | T2 entries |
|---|---|---|---|---|
| P37_editor_chrome | 8 | 12 | 10 | 32 |

## Folded in (cheap, same files)
Dropping the dead CSS rules `.compose-readout` and `.compose-straighten-grid + path` (backlog note on P36, no behaviour, no row). **Not folded**
(not in these files or not cheap): the typed-degrees-then-drag drop and the single save queue (P36 narrative "Not yet"); the flaky P33 stage-bar
place-count test under the full suite (backlog N1).

## Lessons for the planner
- When a book amendment turns named controls into icons, list every test that finds a control by **visible text** (not only by role and
  accessible name) and every hint assertion that uses a substring match — both go red or, worse, stay green against the wrong card.
- A drawer that hides controls makes every existing test of those controls depend on opening it first; name the open step per affected test.
