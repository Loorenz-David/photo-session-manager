---
plan: P39_undo_history_core
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer)
---
# P39 review r1: APPROVED_WITH_NOTES

## Gate
- Checkpoint `bbfede7` exists in `git log`. The orchestrator's C4(h) test edit followed in `a22dc41`, and the plan amendment in `a0dd570`.
- Tracker: the P39 row read **`NOT_STARTED`**, not `IMPLEMENTED`. The implementer had stopped BLOCKED on the C4(h) contradiction and
  intentionally left the row alone. The orchestrator's dated ruling (plan Review log; "Orchestrator note" in the implement handoff, state
  BLOCKED → DONE) resumed the phase "for the reviewer to check the rest as usual" but did not move the row. I took the ruling of the role
  that owns the transition as the phase's real state, and I did not stop on a clerical proxy. Recorded as a process note in the backlog.
  The row is now `APPROVED`.

## Verified correct
- **Perimeter.** `git show --stat bbfede7` lists 22 paths: the plan's 21 expected paths (2 new source, 15 changed, 1 new test, 3 adjusted
  tests) plus this plan's Review log entry. This matches the handoff's Files changed exactly. `a22dc41` touched only the p39 test (C4(h)'s
  last clause, per the amended row) and the implement handoff. Nothing is extra or missing.
- **Stamp (reviewer run, code tree `a22dc41`, clean).** `cargo test --workspace`: 561 passed, 0 failed. `cargo clippy --workspace
  --all-targets -- -D warnings`: clean. `cargo fmt --all -- --check`: clean. `npm run typecheck`: clean.
- **List rules** (`history.rs`). `record` pushes, pops the front past 25, clears redo and emits after the operation lock is released.
  `state` reports the newest entry of each list. The `mc30_*` unit tests cover the limit and the redo clearing (Core mechanisms).
- **Serialisation and drop (D10, D11).** `reverse` holds the operation lock across take-step → `store.transaction` → place-or-drop.
  `record` takes the same lock and runs only after commit, so lock order stays History → store with no inversion. Only `StepCannotApply`
  removes a step. Any other `Err` leaves both lists untouched.
- **Classify/delete reversal.** It checks session and classification, then `remove_at` from the current group, then
  `set_photo_review(…, None)`, so `undo_insert` sees the group without the photo. It re-inserts ascending at `before.position` (clamped),
  then `enqueue_mirror_tag` and one `touch_session`. `active_photo_id` is the first photo. Redo is symmetric.
- **Reorder.** It checks that the photo is ordered and in the session. When `reorder` gives the current positions, nothing is written and
  `change_seq` stays the same (D12, asserted in C4(c)).
- **Move.** It refuses on published-and-clean, on a RAW already in the destination (D6), and on a photo that is not in the expected
  session and classification. It removes from the old groups, then re-inserts ascending by recorded position (D4), and touches both
  sessions.
- **Copy.** Undo refuses on `published_seq >= seq_after` or `publish_confirmed_since` (D5, the new store query, `IS NOT NULL` included).
  It checks `copy.session_id == target` (D7), deletes the `photos` + `compositions` rows only (no file touched, H2), and re-points the
  RAW's tag job at the source row (D9). Redo re-copies, appends, and writes the new ids and `seq_after` back into the step (D8).
- **Recording.** `classify` (no-op records nothing), `delete`, `reorder` (only when changed) and `transfer` build the step inside the
  transaction and record it after commit. The transfer `before` places are read before any removal. One step per source session. The
  target name is title, else SKU, else "Unassigned".
- **Wiring.** `App::build` receives the history store on both rebuild paths (`prepare` and the later rebuild call), and `None` at open.
  The `History` is built on the new App's `EventHub` (the plan's hazard is handled). `handles()` clones it. `TestApp::review()/history()`
  pass it on. `photos_undo` answers the unchanged `ReviewResponse`. `TauriEvents` has the `history_changed` arm. The error map gives
  `nothing_to_redo` and `step_cannot_apply` with `data.label`/`data.reason`. The three new commands are `pub async fn` over
  `spawn_blocking`. Bindings are regenerated.
- **Rows.** Each of the 27 rows has a behaviour test at its boundary that would fail on a wrong outcome: C1–C6 in
  `p39_undo_history_core.rs`, C7(a–c) in `lib.rs p39_tests`. C7(c) received the event on the mock runtime, so no fallback was needed.
  The p02 edits are exactly the planned API renames plus `c3d`'s new assertions. p05 and p27 only gain the field.
  - Changed-input probes, run in a temporary test file that was deleted afterwards (tree verified clean): C4(c)'s
    `to_string()` == `Can't undo Move to Lamp 2: that session is published`; C4(a)'s `out.sessions` ids == `[s, t]`; `Unreview 02` and its
    undo restore Main 1–3.
- **Assumptions.** One step per source matches the recommended Delegated choice. Title → SKU → Unassigned is §Labels. The lock per
  `History` instance is within D10's wording (see note). The clippy `too_many_arguments` allowance is local. None of them contradicts
  MC-30.
- **H1–H8.** No external write (H1). Copy undo deletes database rows, never a RAW (H2). No pixels (H3). Each reversal is one transaction
  (H4). No UI automation (H5). No commerce dependency (H6). No Capture One or Shopify concept in `history.rs` (H7). The logic lives in
  `psm-core` and the commands are thin (H8). No secrets, no live services.

## Findings
None blocking (B1–B4).

## Notes written to backlog
1. **Copy redo with its source row gone** returns `NotFound` instead of `StepCannotApply("that photo is gone")`, so the step stays in the
   redo list (probe-confirmed). It contradicts MC-30's drop rule but cannot be reached from any screen: nothing calls `history_redo` until
   P41. P40 edits the same reversal code. **→ P40.**
2. **Move redo** re-inserts at the recorded position rather than appending when the target changed in between (probe: `[x, b, y]`). This
   fits M36's "exactly as after" and the classify-redo rule, but differs from the plan's wording. **→ P41 / plan wording.**
3. Two row clauses are correct by probe but not asserted by the suite: C4(a)'s session ids and C4(c)'s message text.
4. The D10 operation lock is per `History` instance, so it does not serialize across a settings rebuild (disclosed assumption).
5. Process: the tracker row was stale when a BLOCKED implementer was resumed by an orchestrator ruling.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1 copy-redo `NotFound` | P40 (same `history.rs` reversal code; before P41 exposes redo) |
| 2 move-redo placement wording | P41 (notice/expectation) or a plan amendment by the coordinator |
| 3 unasserted clauses | backlog (touch-when-editing) |
| 4 per-instance lock | P40 (it adds `CompositionService.history: Arc<History>`, same lock) |
| 5 stale tracker on resume | orchestrator procedure |

## Lessons for the plans
- A move-redo rule that says "append" should be stated in the same terms as classify redo ("at `after.position` through
  `undo_insert`"), or the reason it differs should be given. As written, the two reversal rules disagree in the intervening-change case.
- "Steps that cannot apply" should list `NotFound` for **every** row the reversal reads, redo source reads included, not only the undo
  side.

## Probe declaration
- Touched and removed: `crates/testkit/tests/zz_p39_review_probe.rs` (created, run, deleted; `git status` clean afterwards). No
  production file was edited. Each probe used its own temporary `TestApp` database, and no shared state was changed.

## Write perimeter
`docs/implementation/master_plan.md` (P39 tracker row only), `docs/implementation/plans/P39_undo_history_core.md` (Review log entry),
`docs/implementation/backlog.md` (5 rows appended), `docs/implementation/narratives/P39_undo_history_core.md` (new), this handoff. No
commits (the runner commits).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
