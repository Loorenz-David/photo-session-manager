---
plan: composition_corrections
role: plan
round: 1
state: DONE
date: 2026-11-05
actor: Claude Opus 5.5 (pipeline-planner, implementation-planner doctrine under docs/process/flow.md)
---
# Composition corrections after the first try-out (book 10D amended, R38) — planning handoff, round 1

**Summary.** Gate passed: `docs/intention/intention.md` is **RATIFIED** (owner 2026-10-24; the 10D amendment keeps it RATIFIED per changelog 28);
the mechanism inventory stands (10D's contracts MC-27…MC-29 are unchanged by the amendment, which touches only §What the photographer sees and
MC-28 §Thumbnail). I read `git show b75ad61` and `163857a`, 10D in full, R38, changelog 28, flow.md, the master plan (§3–§6.10, §9), the previous
round's handoff (`handoff_composition_plan_r1.md`), the P31–P34 plans and narratives, and the code at `163857a` (`features/compose/*`,
`features/workspace/*`, `features/shell/Header.tsx`, `App.tsx`, `services/compositions.ts`, `lib/keyboard.ts`, the compose test harness and the
tests that touch the changed selectors). Backend verified: `CompositionService::set` (`crates/core/src/services/composition.rs:35–45`) has no
classification rule, and `save_composition` (`crates/infra/src/sqlite/store.rs:975–1000`) clears `needs_review` — so Split-for-any-photo is
frontend-only (a Rust guard row is planned) and the stale Re-frame badge after Apply is a frontend patch bug.
**Two plan files written** (P35, P36; 15 criteria / 40 rows / 30 `[rule]` rows, counts derived from the files), `phases.txt` appended, the master
plan amended. Not committed (as instructed). **No owner card.** No `implementer: fable` anywhere (owner 2026-11-05).

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. Every open point is answered by the amended 10D or recorded as a reversible reading in the plans' Notes (surfaced to
the owner through the narratives' "Choices made for you").

## Write perimeter
- `docs/implementation/plans/P35_composed_display_and_chrome.md` (new) · `docs/implementation/plans/P36_editor_controls_and_overlays.md` (new).
- `docs/implementation/phases.txt` (two lines appended with `printf >>`: P35, then P36).
- `docs/implementation/master_plan.md` (§4 two NOT_STARTED rows; §5 count 36, graph tail, `P35[P34]` · `P36[P35]`, the composition-corrections
  note with the no-Fable rule; §6.10 `EDGE_REVEAL_PX` marked retired and a new P35–P36 block; §9 log entry).
- `docs/implementation/handoffs/reviewer/handoff_composition_corrections_plan_r1.md` (this file).
- **Not touched:** `docs/intention/**`, `STATUS.md`, `backlog.md`, narratives, other plans, prompts, any code, any tool-recorded state. No commit,
  nothing pushed, no pipeline run, no live Capture One or Shopify call, no secret read.

## The phase map
**P35_composed_display_and_chrome** (default implementer, `difficulty: normal`; 7 criteria / 18 rows / 10 `[rule]`). Items 5, 6, 7, 8 + two
fold-ins. A shared `ComposedPicture` (box at the rect's pixel ratio, the turned frame scaled `1/w`, `1/h`, offset `−x/w`, `−y/h`, rotated by
`straighten`, image `rotate(turns·90) scaleX(±1)` — the `OutputPreview` pattern generalised) renders the stage (both Original and Adjusted layers,
the camera original adding its EXIF orientation) and every tile; grid tiles get `.tile-crop-mark` by `leavesFrameOut` (pixel rule or straighten);
a `Compose` button on the stage bar; `COMPOSE_WAITING` shown by the button's hint, the menu entry, R and double-click (toast); Split gate removed
in four places; header Details lifted to `App` like `assignOpen`; Sessions icon in the panel head and the strip; a Radix `Hint` component
(controls that cannot act are `aria-disabled`, never `disabled`). Fold-ins: Apply clears Re-frame; the item card's "Edit details →" opens the form.
~27 paths (18 source, 4 new tests, 5 adjusted tests), ~900 lines.
**P36_editor_controls_and_overlays** (default implementer, `difficulty: high`; 8 criteria / 22 rows / 20 `[rule]`). Items 1, 2, 3, 4 + two fold-ins.
`BrowserOverlay` deleted; `ComposeMode` renders the strip (or, after F, the panel) itself with `onOpen = pick`, the workspace's `folded` untouched
(so leaving restores it); `overlayLines` (thirds, 6×6 grid, golden, Photoshop diagonals; per half in a split), O cycles, remembered; the
straightening grid (slider held or degrees field focused; switch remembered); `StraightenControl` (typed degrees, ↑/↓ 0.1°, ⇧ 1°, comma decimal,
Esc cancels typing only); an `EditorToolbar` on the stage bar (Revert, Loupe, Snapping, Copy/Paste framing, Tools panel, Frame meter), a nudge pad,
`[`/`]` on the stage arrows, and table T of 30 controls with exact hint texts. Fold-ins: Shape/Overlay selects and a pointer-dragged slider give
the keyboard back; the snap guide is drawn at the centre/start/end line it snapped to. ~28 paths (16 source incl. one deletion, 4 new tests,
8 adjusted tests), ~1 300 lines.

## Deviations from the suggested split
- **Item 1 (strip while composing) moved from P35 to P36.** It rewrites `ComposeMode`'s layout and the editor key map (F), which P36 rewrites
  anyway (O, toolbar, prev/next), and all its test churn (`.compose-browser` scopes, the swallowed-`f` rows of P31 C2(c)/P32 C6(a)) sits in the
  editor tests. P35 still changes the tiles shown inside the old edge browser (they are `PhotoTileFace`), so no behaviour is left half-done.
- **Size.** Both phases are ~27–28 paths against the "~≤20 files" guide; about a third are mechanical selector/expectation edits in existing
  tests that R38 makes necessary (listed per plan under Read first). Criteria stay within flow §9 (7 and 8). If the orchestrator prefers smaller
  sessions, P36 splits cleanly at task 7: P36a = strip + overlays + straighten (C1–C4, C7, C8), P36b = toolbar, nudge pad, hints (C5, C6) —
  no plan text needs to move, only the row assignment.

## Grounding worth knowing (orchestrator / reviewer)
- **Tests that change their expectation** are named with line numbers in each plan's Read first (P35: five files — tile rect now read from
  `.composed-frame` `left = −100·x/w %`; split hairline at `50%/0%/100%`; the unreviewed Split entry inverted. P36: eight files — edge-reveal rows
  C8(a)/(b) deleted; `.compose-browser` scope → `.folded-strip`; `f` no longer swallowed; readout → degrees field; snap guides become positions).
- **A sign slip in P34's plan** (C2(b): the ⌥ drag of +3.7 px written as `x = 0.170386`): a drag to the right lowers `x` (P31 C1(b), C7(a)), so
  the value is `0.162818`. The P34 test asserts no value there, so nothing is red; P36 C6(d) asserts the correct one (recorded in P36 Notes).
- **Hints are Radix DOM tooltips**, read in tests as the `role="tooltip"` text after focus (`HINT_DELAY_MS = 400`, rows wait up to 1 s). Reason in
  P35 Notes: the owner asked that hover show name and key; a DOM tooltip behaves the same in WebKit and jsdom and works on `aria-disabled`
  controls. Hazard recorded: Esc while a hint shows closes the hint first (Radix `DismissableLayer` prevents default; the editor ignores a
  prevented Esc).
- **Every expected number is hand-computed from the P31 fixture** (stage 1000×700, frame 2560×1707, default square; `W_t·s = 977.81`): composed
  styles (`149.9707% / −24.9854%`, split `106.0453% / 141.4214% / −3.0227% / −20.7107%`, ratio `2414.0626 / 1207.0313`), overlay lines (thirds
  `391.3333/608.6667`, golden `423.0418/576.9582`, diagonals on the Original box `{24, 32.6047, 952, 634.7906}`), nudge `0.167624`, edge snap guide
  at `x = 174`.
- **jsdom limits** are stated per plan: container-query sizes, the visual hairline, line legibility over a photo, the grid's density and the hint's
  placement are manual try-out items; rows assert the inline styles and attributes that produce them.

## Choices the owner may want to know (recorded in the plans' Notes; for the narratives)
1. The crop mark shows on every grid tile whose framing is smaller than the whole frame or straightened — with the default Square, most tiles.
2. Space shows the camera original with the same framing as the adjusted picture.
3. Clicking the greyed Compose button, R or double-click on a photo still waiting shows the waiting message as a toast too.
4. The Sessions icon sits beside Import and Fold ("export" in the book read as the panel's import icon) and also in the folded strip.
5. The editor always opens with the strip folded; F unfolds it for that session; leaving restores your panel.
6. Default overlay Thirds; Grid is 6×6; Diagonals are Photoshop's 45° corner lines; a split gets overlays per half.
7. "While straightening" = slider held or degrees field focused; the 24 px grid is screen-aligned and covers the stage.
8. Arrow keys get a clickable nudge pad; zoom gets no buttons (gestures, not keys); the Loupe button latches; Snapping is remembered.

## Criteria counts
| Phase | Criteria | Rows | `[rule]` rows |
|---|---|---|---|
| P35_composed_display_and_chrome | 7 | 18 | 10 |
| P36_editor_controls_and_overlays | 8 | 22 | 20 |
| **Total** | **15** | **40** | **30** |

## Folded in (cheap, same files, from the P31–P34 narratives' "Not yet")
P35: the Re-frame badge after "Apply composition to selected" (patch sets `state ok`, `review_reason null`); the item card's disabled "Edit details →".
P36: keys dead after the Shape menu / a slider drag until the photo is clicked (blur on select change and slider pointerup; keyboard-focused slider
keeps focus); the snap guide always drawn at the centre. **Not folded** (not cheap or not in these files): the single save queue for Apply / Split /
Reset (P34), a scroll lost when a shape is picked within ~1 s (P32), the quick second grid Split/Normal (P33), editor keys off while the item card
or a header popover is open (P31/P32).

## Lessons for the planner
- When a book amendment retires a behaviour an earlier plan pinned with rows, list those rows (file:line) as "tests that change their
  expectation" in the new plan; otherwise the implementer either keeps the old behaviour to stay green or edits tests without the reviewer
  knowing it was sanctioned.
- Re-derive the sign of every carried-over number; an earlier plan's value that its test never asserted can be wrong without anything red.
