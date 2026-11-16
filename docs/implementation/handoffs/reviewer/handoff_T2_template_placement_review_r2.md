---
plan: T2_template_placement
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-16
actor: claude opus 5.5 (pipeline-reviewer)
---
# T2 — Photos file themselves · re-review r2

**Verdict: APPROVED_WITH_NOTES.** B1-1 is resolved for the stored (export) Main order. Two leftover items are triaged DROP under flow.md §5, written to the backlog and carried to T3. Nothing needs the owner.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. 10G §6.1 settles the outcome. What remains is P7 wording for the orchestrator (lesson L3).

## Review history
- r1 settled everything except B1-1: P4/P5/P8 mechanisms, P1–P3 store reads, `place_arrival`, `adopt_frame_tx`, the untemplated paths, failed-placement isolation, C1–C8 tests, the UI and H1–H8.
- This round is delta-scoped. It covers the Main order seam (`main_positions`) and its dependents: reorder, classify, delete, arrival, and the container grid that displays the result.

## Perimeter check
- **Gate.** The tracker read IMPLEMENTED with the fix note. Checkpoint `46d5ea8` is in `git log`.
- **`git show --stat 46d5ea8`:** `crates/core/src/services/template_placement.rs`, `crates/testkit/tests/suite/t2_template_placement.rs`, the plan's tracker and Review log, and the fix handoff.
- **`git diff 17a49dc 4ee3396 --stat`:** the same files, plus the runner's two prompt files.
- Every file is inside B1-1's allowed list. `review.rs` and `store.rs` were allowed and are untouched. Nothing is extra.
- The working tree was clean at entry. Code tree for every check: `46d5ea8`.

**Checks** (one cargo command at a time, none in the background):
- `cargo test -p psm-testkit --test suite t2_template_placement::`: 21/21, including the new `t2_b1_…`.
- `cargo test -p psm-core`: 96/96.
- `cargo clippy -p psm-core -p psm-infra -p psm-desktop -p psm-testkit --all-targets -- -D warnings`: green.
- `cargo fmt --all -- --check`: green.
- `npm run typecheck`: green.
- The fixer's p02_review_order / p39_undo_history_core / t1_templates runs were on the same tree. I cite them and did not re-run them. No full suite: the stamp stands, and the change is confined to the templated renumber.

## Findings resolved
**B1-1 (reorder undone or one place off in a templated session): resolved for the stored order.**
- `main_positions` is now the one Main order. `renumber_main` and `place_arrival` both use it.
- **Photos placed by hand.** A Main photo with a row, `place_by_hand = 1` and a position is fixed at that position. Sorted by (position, id), each one goes to `max(wanted, prev + 1)`, capped at `m − (k − j) + 1`.
- **Why nothing panics.** Fixed positions strictly increase and stay ≤ m. The free positions number exactly m − k, the same as the other photos, so `expect("main photo")` cannot fire. The usize arithmetic cannot underflow because k ≤ m.
- **Every other Main photo** fills the free positions by (no slot, slot, previous position, id). That is P7's key with the by-hand term removed, which is right: the only by-hand photos left in that sort have no position.
- **The r1 probe scenario** now gives `reorder(F1, 1)` → F1 1, F2 2, F3 3, then `reorder(F3, 1)` → F3 1, F1 2, F2 3.
- **The new test** asserts both steps plus a later arrival. On the old sort it fails: F1 stayed at 2.
- **Unchanged and still green:** C4(d) (a rowless G1 stays after G2), C7(a) (a classified F4 is fixed at 4), C7(c).
- **Unchanged by reading:** the skip rule for photos placed by hand, Condition's position reuse, events after commit and no history step.
- **The divergence is sound.** The fixer implemented the r1 suggested rule because no settled P7 text existed, and declared that `master_plan_templates.md` P7 is stale. Correct: that file is outside the fixer's perimeter.

## Triage (flow.md §5)
| # | Item | Seen how | Class | Why |
|---|---|---|---|---|
| R2-a | While an earlier container is empty, the screen order after a drag differs from the stored (export) order. A later arrival also puts the template photo back behind the dragged one. | Temporary probe (below). C4(a) holds up to F3: F1 slot 2 pos 1, F3 slot 3 pos 2. `reorder(F1, 2)` → stored F3 1, F1 2 (right), but the view slots are F1 2, F3 3. `ContainerGrid` groups by slot, so the screen shows [1] F1 F3 and the drag looks undone. After F2 arrives: F2 1, F1 2, F3 3. Cause: reorder writes `slot = <target position>`, a position number, not a container number. | **DROP** | Happens only mid-shoot, while an earlier container is empty, and goes away once it fills. The common case (reorder after the shoot, all containers filled) agrees on screen and in store; I checked the cases by reading. No data is lost, and the photo placed by hand keeps its place as the r1 suggested rule says. T3·U4 replaces the drag onto a slotted tile with a container drop. Fixing the grid or slot semantics now would be rework that T3 redoes. Backlog under T2, carried to T3. |
| R2-b | The new rule contradicts the T3 plan. T3·C2(b)'s swap marks both photos placed by hand and renumbers. Under `main_positions` they keep their old positions (F2, F1, F3), not the row's F2, F3, F1, unless the drop writes positions first. | Reading T3 plan task U2 and row C2(b) against `main_positions`. | **DROP** (a plan matter, not a T2 defect) | T2 has no container drop. The orchestrator must settle P7 (L3) before T3 is implemented. Backlog. |

No MUST item remains.

## Remaining findings
None blocking. Notes:
- The two items above (backlog, under T2).
- r1's five notes stand unchanged.

**Carry-forward dispositions**
| Item | Destination |
|---|---|
| R2-a, R2-b, L3 | T3 (orchestrator amends `master_plan_templates.md` P7 and the T3 plan before the T3 implement prompt; the T3 reviewer checks the reorder path that is left, onto a slotless photo) |
| r1 note: the C5(a) rect | backlog (test touch-up when C5 is next edited) |
| r1 note: crash or failed placement leaves the photo rowless | backlog (T4 or a robustness pass) |
| r1 note: a Condition reorder writes `slot` | T3 (differs reads slots) |
| r1 note: the C6(b) map | closed by the r1 record |
| r1 note: the guide | T4 (owns the guide update) |

## Lessons for the plans
- **L3 (supersedes the r1 L1 suggestion).** "A Main photo placed by hand keeps the position the person gave it" fixes reorder but is not enough as P7. It conflicts with T3's swap, which needs order by slot. It also leaves the grid, which groups by slot, out of step with positions whenever reorder's slot (a position number) is not a container number. P7 needs one rule that covers reorder, container drop/swap and display. For example: reorder records the slot of the container it lands in on screen, and Main is ordered by (slot, hand-placed first within a slot, previous position). Or: the grid renders by stored position. Settle it before T3's implement prompt.

## Probe declaration
- Touched: `crates/testkit/tests/suite/t2_template_placement.rs`. I appended one temporary test (`zz_review_r2_probe`, prints only), ran it once, and restored the file with `git checkout`. `shasum` was `de7d90ae…` before and after. `git status --porcelain` was clean afterwards.
- No production file was edited. There were no database or app-state side effects outside the test's temporary folders.

## Write perimeter
- This handoff.
- The Tracker line and a Review log entry in `docs/implementation/plans/T2_template_placement.md`.
- Two lines appended to `docs/implementation/backlog.md` under T2.
- `docs/implementation/narratives/T2_template_placement.md` (new).
- No commits. No archgraph state.
