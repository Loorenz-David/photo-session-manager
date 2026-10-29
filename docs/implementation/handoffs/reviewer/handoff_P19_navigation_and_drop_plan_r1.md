---
plan: P19_navigation_and_drop
role: plan
round: 1
state: DONE
date: 2026-10-29
actor: Claude Fable 5.1 (pipeline-planner, implementation-planner doctrine under docs/process/flow.md)
---
# P19_navigation_and_drop — planning handoff, round 1

**Summary.** Gate passed: `docs/intention/intention.md` reads **RATIFIED** (2026-10-24) and the book carries the
inventory's contract chapters (05A/07A/10A; P00–P18 APPROVED on them). I read flow.md, the master plan (§4, §5, §6.1,
§6.9, §7.2, §9), the second real run's findings R7/R8, book 01/14/06/12, the design handoff and `COMPONENTS.md`, the
four frontend routes (`App.tsx:15`), every dnd-kit site in `apps/desktop/src`, the installed `@dnd-kit/core` 6.3.1
drop-animation source, and the P07/P08/P16 tests the new rows reuse. One plan file written, `phases.txt` appended,
master plan §4 row, §5 order/`depends_on` and §9 log entry added. 3 criteria / 9 rows, all `[rule]` rows tracing to
R7 or R8 plus the book chapter they lean on. Nothing needs the owner.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. Escape-as-Back on Settings and Health only (the handoff defines Esc as "close the topmost layer", so the
workspace keeps that meaning) and Health's parent = Settings are recorded as delegated choices in the plan, not cards.

## Write perimeter
- `docs/implementation/plans/P19_navigation_and_drop.md` (new) — committed.
- `docs/implementation/phases.txt` (one line appended) — committed.
- `docs/implementation/master_plan.md` (§4 one row; §5 phase count, graph tail `→ P19`, `P19[P18]`; §9 one entry) — committed.
- `docs/implementation/handoffs/reviewer/handoff_P19_navigation_and_drop_plan_r1.md` (this file) — **not committed**
  (the task restricted the commit to the three paths above); the runner's next `commit_artifacts` or the orchestrator
  may commit it.
- No code, no tool-recorded state. Nothing pushed; the pipeline was not run.

## Grounding worth knowing
- **R8 cause (verified in the installed library):** dnd-kit's `DragOverlay` drop animation measures the dragged
  sortable tile's layout rect the moment `active` clears (`core.esm.js:3755–3800`, default transform-agnostic measure
  `:2478–2490`). `useWorkspaceState.run` (`:124–139`) awaits `reorderPhoto` before `reload()`, so at that moment the
  tile is still at its old index → the preview flies back, then the grid jumps. The plan's fix applies the new order
  to local state synchronously in the drop's commit (task 4); `dropAnimation={null}` is the fallback only if the
  manual try-out still shows a return. Rows C3(a)–(c) assert the DOM order right after release, which is exactly what
  the animation reads (jsdom cannot observe the animation itself).
- **Other dnd-kit sites:** `CustomizeMode.tsx:122–127` reorders synchronously with no overlay — no change;
  `SessionsPanel`/`PhotoGrid` droppables carry no preview; the folded strip shares the workspace context.
- **R7 parents:** `#/` root · `#/session/:id` (incl. Unassigned `1`) → `#/` · `#/settings` → `#/` · `#/health` →
  `#/settings` (Health is reached only from Settings today, `SettingsPage.tsx:55`). Only `COMPONENTS.md` "Header bar"
  gives header guidance ("Back to Sessions, item type, SKU and ⓘ; …"); Health/Settings are "Not in the handoff".
- **Ungrounded / judgment:** none of the rows depends on an external fixture beyond dnd-kit's measuring order
  (grounded above). The accessible name `Back to <parent>` and the "open layer" selector set (`dialog`,
  `alertdialog`, `menu`) are plan choices, listed under Delegated choices.

## Counts (derived)
Criteria 3 (C1 header 4 rows, C2 keyboard 2 rows, C3 drop 3 rows) = 9 rows; `[rule]` rows 9; files expected to
change 10 (+2 optional test files); frontmatter `difficulty: normal`, `projection: no`, `depends_on: [P18_pending_state_ui]`.
