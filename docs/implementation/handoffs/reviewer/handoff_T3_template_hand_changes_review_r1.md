---
plan: T3_template_hand_changes
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-16
actor: claude opus 5.5 (pipeline-reviewer)
---
# T3 — Your changes win, and there is a way back · review r1

**Verdict: CHANGES_REQUESTED.** One blocking finding (B1, route production): undoing a reorder of two extra Main photos in a templated session no longer restores their order. It is a regression from T2 that this phase introduced. Everything else holds: the perimeter, the checks, all 24 rows and their tests, H1–H8.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
This is the first review of this phase. The implementer was Opus. Four units plus a closing checkpoint. No earlier rounds.

## Verified correct
- **Gate.** The plan's Tracker line read IMPLEMENTED. Checkpoints `8057056` (U1), `3d7ea16` (U2), `8db8fe4` (U3), `f4e9432` (U4) and `ab51a43` (closing) are all in `git log`.
- **Perimeter.** I ran `git show --stat` on each checkpoint and `git diff --stat 8057056^ ab51a43`. The result is 24 paths, exactly the handoff's "Files changed" list: the plan's per-unit files, the four disclosed deviations (`commands/items.rs`, `ports/store.rs`, `PhotoTile.tsx`, `useTemplateView.ts`), the plan file and the handoff. Nothing is extra or missing. `domain/template.rs` was declared optional and is unchanged, as disclosed. The tree was clean at review start.
- **Checks** (Review mode, run once each on tree `58f285f` = `ab51a43` + the review prompt, one cargo command at a time):
  - `cargo test -p psm-testkit --test suite t3_template_hand::` 19/19
  - `p39_undo_history_core::` 24/24
  - `t2_template_placement::` 21/21
  - `cargo test -p psm-desktop` 29/29
  - `cargo clippy -p psm-core -p psm-infra -p psm-desktop -p psm-testkit --all-targets -- -D warnings` ok
  - `cargo fmt --all -- --check` ok
  - `npx vitest run src/features/workspace`: 15 files, 83 tests
  - `npm run typecheck` ok
  - The implementer's full-suite stamp stands: Rust 1540, UI 536.
- **U1, differs (P9).** `differs()` returns false for Deleted photos and for photos with no shot index. Otherwise a photo differs in either of two cases:
  - by hand (or no row) and its (classification, Main slot) ≠ the expected place from `template::expected(shot index)`; or
  - `frame_by_hand`, sitting in its expected container, and that container has presets.

  The shot index comes from the same `shot_indexes` placement uses (refactored out of `place_arrival` with no behaviour change; the T2 suite is green), read at the call's `now`. The count is the sum. `place_of` reads a slot for Main photos only, so the slot that assumption 2 keeps in a Condition photo's row is never read as a container.
- **U1, other changes.** `HistoryService::reverse` renumbers Main by P7 inside the transaction for every touched session. `renumber_main` returns early for untemplated sessions. `delete_photo_row` also deletes `photo_places`.
- **U2, drop.** `drop_on_container`:
  - Refusals:
    - `no_template` (via `template_of` = P1)
    - `invalid_value` when n is not a container `final_no`
    - `invalid_value` for a Deleted photo
    - `container_filled` for a slotless photo dropped on a filled container
  - Cases:
    - own slot = n → no write, no step
    - fill → `reclassify` to the end of Main (mirror tag queued) and `put_by_hand(n)`
    - swap → exchange positions and slots; both get `place_by_hand = 1`
  - The occupant is the lowest-position Main photo with slot n.
  - Compositions are untouched.
  - Order of work: snapshot before any write, renumber and touch inside the transaction, step recorded in `finish` after the commit (the ReviewService pattern).
- **U2, `StepKind::Template`.** Each photo's `PhotoState` holds classification, position, the whole place row, the full `Composition` (values or frame-unknown kind/shape/origin with `pending_zoom`) and `denied_at`. `names_photo` covers it.
  - Undo/redo first checks every photo against what the step left (session, classification and row slot) and returns `StepCannotApply` before any write.
  - It then removes the photos from their groups, re-inserts them at their recorded positions (sorted per group), restores denial, composition and row, queues mirror tags on a class change, and the caller renumbers Main.
  - `coalesces` only merges attribute steps, so template steps never merge.
- **U3, back.** `back_to_template`:
  - It acts only on photos that differ (Deleted never), with the current template and shot order.
  - Each one gets row `place_by_hand = 0` and `frame_by_hand = 0`, `reclassify` to its expected class, slot set, and `apply_frame` with presets only when the container has them. Otherwise the composition is unchanged (plan P8 wording; assumption 5).
  - Then Condition goes in shot order (P7) and Main is renumbered.
  - It is one step, "Back to template". When nothing differs, nothing is written and no step is recorded.
  - The snapshot (compositions included) is taken before the first write.
- **U3, commands.** `template_drop` and `template_back` are thin (`hand_action` builds `HandAction` from the wired app and maps to `ReviewResponse`/`CommandError`, which satisfies H8). Both are registered in `specta_builder`, and the bindings were regenerated. `ContainerFilled` maps to `container_filled`.
- **U4.**
  - `TemplateWarning` returns null at count ≤ 0. Otherwise it shows `role="status"`, singular/plural text and the button.
  - The page wires `onTemplateBack` to the action runner with the undo toast. A `data-differs` mark and dot come from `TemplateDiffersContext`.
  - `dropAction` is pure: in a templated view, a known container placeholder or the tile of a Main photo with a known container slot → template drop; else default; untemplated → default.
  - Placeholders are `useDroppable('container:<n>')`, and collision detection prefers tiles, then placeholders. The page asks `dropAction` first.
  - `container_filled` → toast "That container is filled".
  - `useTemplateView` re-reads on `history_changed` for its session.
- **Rows ↔ tests.** I read all 24, and each observes its row's outcome at the row's boundary and would fail on a wrong one:
  - C1(a), (c), (d) and C2(a), (b) each assert a `true` differs plus the exact count.
  - C2(c), (d) compare the whole stored state and assert no undo step.
  - C2(f) and C3(d) compare the full stored state after undo and redo.
  - C3(b) checks kind, origin, the exact rect and `frame_by_hand`.
  - C3(e) would fail if a Deleted photo counted (count 0 before back).
  - C3(f) checks slots, positions and `place_by_hand = 0`.
  - C3(g) goes through the wired IPC and `bindings.ts`.
  - C4(a)–(d) assert exact text and call argument, absence, the attribute on one tile and not the other, and all five `dropAction` cases.

  There are no orphan tests: every T3 test maps to one row, and the scratch probes below were removed.
- **Review-mode reads.**
  - **Hand-placed photos.** No path changes a hand-placed photo's place except `back_to_template` and the person's own actions (drop, classify, reorder, undo/redo). `place_arrival` still skips rows with `place_by_hand = 1` and photos with no row. `renumber_main` changes positions only, never class or slot.
  - **The undo snapshot** includes compositions (`photo_state` → `tx.composition`).
  - **Untemplated sessions** take none of the new paths:
    - drop and back → `no_template`
    - `session_view` → `templated: false`
    - `renumber_main` and `photo_place_mark` return before any write
    - `dropAction` → default
    - the warning renders only when `templated`
- **H1–H8 hold.**
  - H1: no external write.
  - H2: no file touched; mirror tags only, through the existing queue.
  - H3: no pixels.
  - H4: drop and back are one store transaction each; the undo step is recorded after commit, as for every review action.
  - H5: no UI automation.
  - H6: no commerce dependency.
  - H7: no device or platform names in core.
  - H8: the commands are thin.
  - No secrets and no live services.
- **Assumptions.** None contradicts the intention.
  - 1: P7's Main key becomes (no slot, slot, previous position). This is a master-plan rule change, not a book change; see note N1.
  - 2, 4, 5, 6, 7, 8: within 10G §6 and the plan's Delegated choices.
  - 3: the undo/redo of a reorder writes the slot of the restored position. This is the cause of B1 below.

## Findings

### T3-R1-B1 · blocking (B1) · route: production — undo of a reorder among extra Main photos does not restore the order
- **What is wrong.** In a templated session, assumption 3 makes undo and redo of a `Reorder` step write `slot = <restored position>` (`history.rs` `apply`, `StepKind::Reorder`: `tx.photo_place_mark(photo_id, true, false, Some(target))`). The same `reverse` now calls `renumber_main`, which sorts Main by (no slot, slot, position).
  - A photo after the containers (an "extra": a Condition photo the person moved to Main, so it has no slot) that was dragged onto another extra gets a slot from the drag.
  - Undo gives it another slot (its old position number) instead of the none it had.
  - So it still sorts ahead of the slotless extra, and the renumber puts the drag's order back.
  - The step moves to the redo list and Main looks unchanged.
- **Before T3** (`87e60ac` `history.rs` has neither `renumber_main` nor `photo_place_mark`), the same undo restored the positions.
- **Probe** (scratch test appended to `t3_template_hand.rs`, run, file restored by `git checkout`, checksum identical). Fixture: S0, F4 and F5 classified Main (positions 4, 5). `reorder(F5, 4)` gives F4 5, F5 4. `undo` gives **F4 5, F5 4** with slots (None, Some(5)). Expected (4, 5).
- **Authority.** Book 05A MC-30 ("reorder within a group — undo restores the prior position"). Book 10G §6.5. The plan's U1 goal ("Undoing or redoing any older step in a templated session leaves Main in container order").
- **Correction.** Undo of a reorder must leave the photo's place row as it was before the drag: an extra that had no slot has none again. Redo must write what the drag wrote. One way: record the photo's slot before and after in the `Reorder` step and restore it (`photo_place_mark` cannot clear a slot, so use `photo_place_put`). Add one test in `t3_template_hand.rs` at `HistoryService::undo`/`redo` with the probe's input and the exact positions after undo (F4 4, F5 5) and after redo (F5 4, F4 5).
- **Files the fix may touch:** `crates/core/src/services/history.rs`, `crates/core/src/services/review.rs`, `crates/core/src/ports/store.rs` and `crates/infra/src/sqlite/store.rs` (only if a slot-clearing write is needed), `crates/testkit/tests/suite/t3_template_hand.rs`.

## Notes written to backlog
Five notes went to `docs/implementation/backlog.md` under T3:
- **N1 (plan).** `master_plan_templates.md` P7 still has the "placed by hand" sort key and T2 fix 1's absolute-position rule. Assumption 1 replaced both with (no slot, slot, previous position). The coordinator should fold P7 upstream.
- **N2.** A reorder in a templated Main still writes `slot = <target position>`, which can be a number that is no container (an extra dragged to position 4 of a 3-container template gets slot 4). A container photo dragged behind the extras is a visible no-op that marks it by hand (disclosed "known edge"; T2 r2 DROP note).
- **N3.** C2(d) asserts `Error::ContainerFilled`, and the code string `container_filled` is pinned only by the exhaustive match in `items.rs`. No command-level test sees the string the UI keys on.
- **N4.** C4(a) exercises `PhotoPanel`'s fallback (`templateBack` directly). The page's path (`useWorkspaceState.templateBack` → action runner → undo toast) has no UI test.
- **N5 (guide, flow §7A).** `docs/guide/` does not mention drop/back, `StepKind::Template` or `template_drop`/`template_back`. T4 owns the guide update (templates master plan), so this is a carry-forward.

### Carry-forward dispositions
| Note | Destination |
|---|---|
| N1 | coordinator: fold into `master_plan_templates.md` P7 before T4 |
| N2 | T4 (reorder/retake paths) |
| N3, N4 | backlog (test debt, not blocking) |
| N5 | T4 (guide) |

## Lessons for the plans
- A plan that changes how a step is replayed (here: renumbering inside `reverse`) should carry a row per existing step kind that the replay reaches. T3·C1(g) covered classify only, and the reorder kind regressed unseen.
- P7 in the templates master plan is now three rules (T2 fix 1, T2 r2 note, T3 assumption 1). One sentence there would stop the next phase from re-deriving it.

## Mutation-probe declaration
- I edited no production code.
- `crates/testkit/tests/suite/t3_template_hand.rs`: two scratch tests were appended in turn, run, and removed with `git checkout`. The file is byte-identical (`shasum` 25b2d196…6e checked after each).
- Each probe's `TestApp` used its own temp directory; no database or state outside it was touched.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_T3_template_hand_changes_review_r1.md` (this file)
- `docs/implementation/plans/T3_template_hand_changes.md` (Tracker line, Review log entry)
- `docs/implementation/backlog.md` (five T3 notes appended)

No code, no archgraph and no narrative (this is not an approval). Nothing committed.
