---
plan: P39_undo_history_core
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-05
actor: pipeline-projectionist (Claude Opus 5.5)
---
# Projection of P39_undo_history_core (round 0)

The undo-history plan can be built, and most of its 22 test rows can be written today with one exact answer each. It still needs a round of paragraph-sized fixes before the implementer starts. One existing test cannot keep its expectation as the plan promises. One new row expects the wrong answer. Two existing test files will stop compiling. Several undo rules would misbehave in situations no row exercises: undoing a multi-photo move puts the photos back in the wrong order, and undoing a copy while that session is mid-publish breaks the publish. None of this needs the owner. All of it is plan-level and the coordinator can fold it in, after which the implementer prompt can be compiled.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. Every gap below is a plan amendment or a written delegation.

## Gate check
- Intention header **RATIFIED** (`docs/intention/intention.md:3`). Upstream gate `handoff_polish_and_undo_plan_r1.md` is `state: DONE` with `OWNER DECISIONS REQUIRED (0)`.
- Plan frontmatter `projection: yes` (flow.md §2: the gate is on).
- Code basis: `git diff --stat 3e4b760 HEAD -- crates apps/desktop/src-tauri` is empty, so every `file:line` the plan cites is valid at HEAD `8aa6c45` (spot-checked below).

## Decision ledger

| # | Decision point (where the artifacts stop determining the next step) | Class | Proposed routing |
|---|---|---|---|
| D1 | **p02 `c3d_undo_clamps_prior_position` (`p02_review_order.rs:193–206`) cannot keep its assertions.** Sequence: `main(a,b,c)`, delete c, classify a→Condition, classify b→Condition, one undo. Under the history, the newest step is *b → Condition*, not the delete, so the first undo gives `Main == [(b,1)]`, `Condition == [(a,1)]`. The test asserts `Main == [(c,1)]`. The plan's Read-first ("their assertions are unchanged (the newest step is the last delete …)") and Notes ("No other test may change") contradict the code. An implementer either stops with BLOCKED or edits silently. | plan gap (blocking) | Amend Read-first/Notes: c3d's expectation changes to the newest-step outcome (`Main == [(b,1)]`, `Condition == [(a,1)]`). The clamp it used to prove moves to C2(b). Alternatively, retire c3d in favour of C2(b). Either way, name it. |
| D2 | **`PhotosService` gains `history: &'a History`, which breaks two struct literals in tests the plan says stay untouched:** `crates/testkit/tests/p05_capture_arrival.rs:76` (`fn service`) and `crates/testkit/tests/p27_retire_shape_on_arrival.rs:86`. A third, production literal is `commands/photos.rs:171` (`photos_import`). The plan lists only the `:192` assign site, and cites it with the wrong range (R2). | plan gap (blocking compile) | Amend Files/Notes: add both test files as mechanical edits (`history: a.app.history.as_ref()`, no assertion change). Name both desktop construction sites (`photos_import` `:171`, `photos_assign_unassigned` `:192`). |
| D3 | **C2(e)'s expected outcome is wrong.** `main(s,[a])` records `Approve 01` through `R.classify`. After the `Condition 01` step is dropped, `H.state(s) == { Some("Approve 01"), None }`, not `{ None, None }`. | plan gap (row defect) | Amend C2(e)'s outcome to `H.state(s) == { undo: Some("Approve 01"), redo: None }`. |
| D4 | **Move undo replay order puts photos back out of order (M36 "order drift").** `transfer` reads every photo before removing any (`review.rs:168–171`), so the natural `before.position` is the **pre-transfer** position. With that, the plan's "for each photo (reverse source order) … `undo_insert(before.position)`" misplaces photos: `[a,b,c,d]`, move `[b,c]`, undo → `c` inserted at 3 into `[a,d]` = `[a,d,c]`, then `b` at 2 = `[a,b,d,c]`. Ascending order gives `[a,b,c,d]`. Reverse order is correct only if `before.position` means "position at the moment of its own sequential removal", and the plan does not say that. No row moves more than one photo (C4(a) moves only `b`). Classify's "for each PhotoPlace in reverse order" has the same ambiguity, but nothing in P39 records a multi-photo classify (D18). | plan gap (silent failure) | Amend §Reversal rules: `before.position` = position read before the operation. Undo inserts per group in **ascending `before.position`** (forward source order). Removal order from the target is free. Add row **C4(f)**: `main(s,[a,b,c,d])`, `main(t,[x])`, `R.move_photos(&[b,c], t)`, `H.undo(s)` → `state(s,Main) == [(a,1),(b,2),(c,3),(d,4)]`, `state(t,Main) == [(x,1)]`. Then `H.redo(s)` → `state(t,Main) == [(x,1),(b,2),(c,3)]`. Trace M12, M36. |
| D5 | **Copy undo during an unfinished publish breaks that publish.** The refusal `published_at.is_some() && published_seq >= seq_after` only sees **completed** publishes. A publish confirmed after the copy but still running (or failed, awaiting Retry) has the copy's id in `publish_items.frozen_main/condition`. `PublishService::selected` (`services/publish.rs:95–104`) calls `self.store.photo(*id)?`, so a deleted copy row → `NotFound` → the target's publish fails at prepare/upload, and on every retry, because the frozen list never changes. The plan's claim that "nothing else references a never-published copy" also misses `photo_links` rows written by a partly-uploaded failed run. SQLite foreign keys are not enforced (no `PRAGMA foreign_keys` in `crates/infra/src`), so `delete_photo_row` succeeds and leaves them orphaned. | plan gap (silent failure) | Amend Copy undo: also refuse when **any** `publish_items` row of `target` has `confirmed_change_seq >= seq_after` (a publish confirmed after the copy, whether running, failed or done), reason `that session is published` (or a new reason added to master §6.6). Name the new `StoreTx` read (e.g. `publish_confirmed_since(session_id, seq) -> bool`) in Task 3 and master §6.4. Optional row: the copy-undo refusal with an inserted confirmed publish item (`TestApp::insert_publish_item` exists, `test_app.rs:339`). |
| D6 | **Move undo has no "already has this photo" refusal.** `photos` has `UNIQUE(session_id, raw_path)` (`002_domain.sql:51`). Scenario: move b s→t; in t, copy b back to s (a step in t's history); ⌘Z in s → `set_photo_review` violates the constraint → raw DB error, rolled back. Under D11 the step stays on top, so every later ⌘Z in s fails the same way, and earlier steps can never be reached. | plan gap | Amend Move undo: check the source for a row on the same RAW (as `transfer` does, `review.rs:175–186`) → `StepCannotApply` reason `that session already has this photo`. Add that reason to master §6.6's list (R4). |
| D7 | **Copy undo does not check that the copy is still in `target`.** If a later step in t's history moved the copy on to u, the plan's rule deletes the row from **u** while touching only `target`, so u's `change_seq` is not incremented (MC-6 / M13 violated). | plan gap | Amend: `copy.session_id == target`, else `that photo already changed`. |
| D8 | **Copy redo must rewrite `seq_after`, not only the ids.** If a publish completes after the undo and the target is then dirtied, redo is allowed and creates fresh copies at a higher seq. The stale `seq_after` then refuses the next undo of copies that were never published. | plan gap | Amend Copy redo: "new ids **and** the new `seq_after` (target `change_seq` after the redo's `touch_session`) are written back into the step". |
| D9 | **Copy undo and colour tags.** Tags are per RAW. After the copy row goes, the surviving source row's classification should be mirrored (MC-30: "mirror colour tags … as classification does"). A queued `mirror_tag` job whose payload names the deleted copy (`enqueue_mirror_tag` coalesces per RAW, `store.rs:467–476`) fails in `mirror.rs` at `self.store.photo(photo_id)?`. | plan gap | Amend Copy undo: `enqueue_mirror_tag(raw, source_photo_id, at)` per removed copy. This repoints any queued job to the surviving row. |
| D10 | **What the history lock covers while a transaction runs.** Holding ⌘Z autorepeats, and each `history_undo` runs on its own `spawn_blocking` thread. "Peek → transaction → move" can apply one step twice, or drop the wrong one. "Take → transaction → place" can apply step N‑1 before step N (outside LIFO order), so a correct step fails its precondition and is dropped. The plan says only "`Mutex` inside". | plan gap (concurrency) | Amend task 1: undo/redo are serialised per `History`. One operation lock is held across take → `store.transaction` → place/drop. Lock order is History → store, and recording never happens inside a transaction (already required), so there is no inversion. `HistoryChanged` is emitted after the list lock is released. |
| D11 | **Which errors drop the step.** §Reversal rules: "a failed precondition returns `Err` … the step is then **removed**". Master §6.10: "a failed precondition drops the step". Neither says what happens on a DB/IO error. | plan gap (small) | Amend: only `Error::StepCannotApply` drops the step. Any other `Err` leaves both lists unchanged and is returned. |
| D12 | **A reversal that changes no position.** C4(c)'s `r2` (reorder of the only remaining photo) yields the same list. The plan's reorder undo always calls `touch_session`, which makes the *published, clean* `s` dirty with no photo change (M13). `ReviewService::reorder` itself skips no-ops (`review.rs:80–82`). | plan gap (small) | Amend: when the computed positions equal the current ones, skip `touch_session` (and the mirror). The step still moves to the other list. No row change needed, since C4(c) asserts only `r2.label`. |
| D13 | **C2(a)'s mirror clause cannot fail.** `jobs == j + 1` holds whether or not the undo enqueues a mirror, because the classify's job for u's RAW is still queued and `enqueue_mirror_tag` coalesces onto it (`store.rs:469–473`). Deleting the undo's `enqueue_mirror_tag` leaves the row green (flow.md §5 B2). | plan gap (row defect) | Amend C2(a): advance the clock (e.g. `a.clock.advance(1000)`) before `H.undo`, then assert the queued `mirror_tag` row for u's RAW has `next_attempt_at`/`updated_at` == the undo's `now` (one direct `SELECT` on `jobs`, like p02's `state`). Keep `jobs == j + 1`. |
| D14 | **No row observes the Tauri `history_changed` emission.** `TauriEvents::emit` has a catch-all `_ => {}` arm (`apps/desktop/src-tauri/src/lib.rs`, the `AppEvent` match ~`:35–106`), so leaving out the new arm is silent. C6(a) checks only the core `AppEvent`. P41's header icons and menu depend on this event. | plan gap (coverage) | Amend C7(a): listen for `history_changed` on the mock webview and assert one payload `{session_id, undo, redo}` after `history_undo`. Or record explicitly that P41 owns this proof. |
| D15 | **C6(a)'s second clause has no host.** "after C4(c)'s blocked undo one more `HistoryChanged` is emitted …" relies on C4(c)'s fixture, which C6(a)'s input does not contain. | plan gap (small) | Move it into C4(c)'s outcome (read `a.events` after `r1`), or make it a separate row C6(b). |
| D16 | **Label for classifying into Unreviewed.** `photos_classify` accepts `unreviewed` (`commands/photos.rs:18–36`; `ReviewService::classify` does not reject it), but §Labels defines only main/condition/deleted. | free choice | Delegate in writing (e.g. `Unreview 03` / `Unreview photo`), or pin it in §Labels. |
| D17 | **C5(a)'s arrival clause cannot fail.** `a.add_photo` writes straight to the store (`test_app.rs:56–67`) and never goes through `ArrivalService`, so an implementation that recorded arrivals would still pass. | plan gap (row weakness, low) | Use the real arrival path (`a.watcher_event(path)` / `a.deliver_hook`, p05 pattern), or drop the arrival clause. |
| D18 | **Selection labels (`Approve 3 photos`, `Delete 3 photos`) can't be produced in P39.** `classify` takes one id, and no selection command exists, so every `Classify` step has one photo. | free choice | Delegate: an n>1 label branch may exist without a row, or be deferred to the phase that adds selection classify. State which, so the reviewer doesn't flag it as dead code. |

## Reality-check findings
- **R1 (count).** "Estimated: 2 new source paths, 12 changed" does not match the plan's own Files list: it names **15** changed paths (core 7: `services/{mod,review,photos}.rs`, `ports/store.rs`, `domain/photo.rs`, `error.rs`, `events.rs`; infra 1; integrations 1; desktop 5: `commands/{mod,photos,items}.rs`, `lib.rs`, `bindings.ts`; testkit 1). With D2, add 2 more changed tests. Notes' "~16 paths" needs recomputing too.
- **R2 (citation).** `commands/photos.rs:166–181` is labelled `photos_assign_unassigned`, but those lines are `photos_import` (`:163–182`). `photos_assign_unassigned` is `:184–204`. Both build `PhotosService` (see D2).
- **R3 (hazard text).** "its hub has no target after `transfer_target_to`" is inaccurate. `transfer_target_to` *copies* the target, and the old hub keeps it (`integrations/src/lib.rs:802–808`). The instruction itself (build `History` with `next.events`) is still right.
- **R4 (registry).** Master §6.6's `step_cannot_apply` reason list (`master_plan.md:339–340`) lacks `that session already has this photo`, which the plan's Move/Copy redo rules use (and D6 adds for Move undo).
- **R5 (claim).** "`Session.undo_slot` … reads NULL from now on" is true only for slots written after P39. Existing databases keep whatever the last delete wrote, since no migration clears it. This is harmless because the field is no longer read. Rephrase it as "is no longer read".
- **Verified as the plan states:** `Store::transaction` rolls back on `Err` (`infra/src/sqlite/store.rs:1245–1255`; a dropped rusqlite `Transaction` rolls back). Only `review.rs` calls `set_photo_review`/`set_positions` (grep over `crates/*/src`), and no code deletes `photos` rows today. `copy_photo` copies the `compositions` row (`:514–525`). `add_photo` rows get a default composition (`store.rs` `photo_insert_or_get` → `insert_default_composition`), so C4(b)'s `compositions` count is discriminating. `enqueue_mirror_tag` coalesces per RAW (`:467–476`). `App::build` is reached only from `open_with_adapter` (`:332`) and `prepare` (`:160`). Desktop rebuilds go through `state.rs:25–27` → the same pair. `TestApp::kill_and_restart` → `open_with_adapter` (a fresh history, as C5(c) needs). `commands_are_async` scans `src/commands/*` (`lib.rs:282–299`), so the new `history.rs` is counted. `CommandError {code, message, data}` (`commands/items.rs:27–32`) carries `data.label`/`data.reason`. The p28 mock-webview pattern (`lib.rs:~424–470`) moves a `TestApp`'s `App` into `AppState`, so steps recorded before the move are visible to C7.

## Criteria decidability (both questions per row)
"Writable now" = a test can be written from the artifacts with one exact outcome. "Producible" = the fixture/shape exists with this repo's dependencies.

| Row | Writable now | Producible | Note |
|---|---|---|---|
| C1(a) | yes | yes | Computed: 25 undos leave `[(u1,1)]`, the 26th → `NothingToUndo`. |
| C1(b) | yes | yes | `c` must be added (unreviewed) before use. |
| C1(c) | yes | yes | |
| C1(d) | yes | yes | Label uses U+2192 `→`. |
| C2(a) | **no**: mirror clause can't fail | yes | D13. |
| C2(b) | yes | yes | Computed: move refused (`change_seq == published_seq`); delete undo clamps 3→1. |
| C2(c) | yes | yes | |
| C2(d) | yes | yes | Computed: undo puts b at Main 2, redo at Condition 1, `seq +1`. |
| C2(e) | **no**: wrong expected state | yes | D3. |
| C3(a) | yes | yes | |
| C3(b) | yes | yes | |
| C4(a) | yes | yes | `<t's title>` = `"s2"` with p02's `session()` (`create_local(name, name, …)`). |
| C4(b) | yes | yes | Composition count is meaningful (see R-verified). |
| C4(c) | yes | yes | D12 affects `seq`, not asserted. D15 if the event clause moves here. |
| C4(d) | yes | yes (`mark_published` sets `published_seq = change_seq` ≥ `seq_after`) | Covers completed publishes only (D5). |
| C4(e) | yes | yes | |
| C5(a) | partly: arrival clause can't fail | yes | D17. `settings_set` needs `&mut a`. |
| C5(b) | yes | yes (`sku_prefix` is a runtime key, `settings.rs:47`; used the same way in p21/p25) | |
| C5(c) | yes | yes | |
| C6(a) | first clause yes; second clause has no fixture | yes (`AppEvent` has no `PartialEq`, so use `matches!`/destructure) | D15. |
| C7(a) | yes | yes (Tauri `tauri::test` mock runtime as in p28. `Option<String>` serialises `null`) | D14 (event not covered). |
| C7(b) | yes | yes | Computed: delete undone → Main 1, active a; then classify undone → Unreviewed, active a; then `nothing_to_undo`. |

Externally-owned shapes (charter rule 17) checked: rusqlite rollback-on-drop, the SQLite UNIQUE violation path (D6), Tauri mock IPC JSON bodies and `CommandError` serialisation. All are grounded in code already in this repo. No new external fixture is introduced.

## Trace verification
- Every row has a trace cell. Each resolves to MC-30 / MC-5 / MC-6 / MC-2 text, or to M12 / M36 / `[rule]` §6.7, that says what the row asserts. Spot-read in `05A_mechanism_contracts.md:143–165, 169–183, 278–320` and `14_measurement_ledger.md:48`.
- M36's named defect family "order drift" has **no serving row for multi-photo steps** (D4 → C4(f)). "redo kept after a new step" → C1(b). "partial undo" → C4(c)/C2(e). "lost or extra steps" → C1(a)/C1(d).
- M13 (dirty) is listed in Read-first. Rows trace MC-6 (`change_seq`) instead, which is acceptable because MC-6 is M13's contract. D12 is the one place the plan's rules would break M13.

## Write perimeter
- Created: `docs/implementation/handoffs/reviewer/handoff_P39_undo_history_core_projection_r0.md` (this file).
- Nothing else: no plan, intention, master plan, code or test edits; no archgraph state; no commits.
