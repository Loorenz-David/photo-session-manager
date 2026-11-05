---
phase: P39_undo_history_core
title: Undo history core — the per-session 25-step history in the Rust services (classify, delete, reorder, move, copy as steps), undo and redo as whole transactions with MC-5 position math and colour-tag mirroring, steps that cannot apply dropped with a reason, the history_changed event and the history_get / history_undo / history_redo commands; the single delete slot retired
difficulty: high
projection: yes
depends_on: [P37_editor_chrome]
---
# P39 — Undo history core: the history, the review steps, undo and redo, the commands

## Goal
The backend half of R41 (05A MC-30, invariant M36, changelog 30) for the **review steps**: a `History` kept by the application services —
one list per session, at most 25 steps, kept while the app runs (across a settings rebuild, cleared on quit) — in which `ReviewService`
records every classify (A, C, Delete, Move to Main, swipe, drag between groups), reorder, move/Assign and copy as one **step**; a
`HistoryService` that undoes the newest step and redoes the newest undone one, each as **one SQLite transaction**, with MC-5's position math
(`undo_insert`, clamped `min(k, n_g + 1)`), MC-6's `change_seq + 1`, and the colour-tag mirror job of chapter 06; a step that can no longer
apply (its photo is gone, the session it would write into is published) changes nothing, is dropped from the list and is reported with
its reason; `AppEvent::HistoryChanged`; the Tauri commands `history_get`, `history_undo`, `history_redo`. MC-5's single delete slot
(`UndoSlot`, `sessions.undo_slot`, `ReviewService::undo`) is retired; `photos_undo` stays as a thin compatibility wrapper over the history
until P41 rewires the frontend.
**Not in this phase:** composition and attribute steps (P40 — the `History` type is designed to hold them, see §Step model); any frontend
change (P41; the frontend's ⌘Z keeps working through `photos_undo`); the Edit menu (P41); the header icons and notices (P41).

## Read first
Book: `05A_mechanism_contracts.md` **MC-30** (whole), MC-5 (the operations table incl. the two MC-30 rows, M12), MC-6 (`change_seq`,
"undo" in the photo-change list), MC-2 §Move and copy (refusals: `target_has_raw`, published target) · `06_photo_review_and_ordering.md`
§Classification (undo, toast) and §Mirror to the editing provider · `14_measurement_ledger.md` M12, M13, M36 · `15_decisions.md` R41 ·
`16_changelog.md` entry 30 (readings: one history per session kept while the app runs; the step list; a step that cannot apply is dropped
with a notice) · process `flow.md` §4, §6, §9 · master plan §3, §6.4 (`Store`/`StoreTx`), §6.6 (codes), §6.7 (commands, events), §6.9,
§6.10 (this phase's P39–P41 block), §7.5 (`TestApp`) · plan `P02_review_order.md` (MC-5 rows C1–C4 and the fixtures its tests use).
Code at `3e4b760` (line numbers as of that commit):
- `crates/core/src/services/review.rs` — `ReviewService::new(store, clock)` `:26`; `classify` `:40–66` (`set_positions(remove_at)`, `n_g + 1`,
  `set_photo_review`, `enqueue_mirror_tag`, `touch_session`); `reorder` `:68–85` (`position::reorder`, no-op when unchanged); `delete`
  `:87–113` (**the `UndoSlot` and `set_undo_slot` go**); `undo` `:115–146` (**removed**: its body is the model for the classify/delete
  reversal — `undo_insert(&old, id, prior.unwrap_or(1))`, `set_photo_review`, `enqueue_mirror_tag`, `touch_session`, `active = Some(id)`);
  `transfer` `:156–228` (the published-target rule `:161–165`, `TargetHasRaw` `:171–186`, the source-order sort `:187–194`, per-photo
  `remove_at` + append `:195–211`, `copy_photo` `:206`, `touch_session` for every touched session `:216–218`); `group()` `:240–252`.
- `crates/core/src/mechanisms/position.rs:10–38` (`classify_into`, `remove_at`, `reorder`, `undo_insert`).
- `crates/core/src/ports/store.rs:35–56` (`StoreTx`: `enqueue_mirror_tag`, `session`, `photo`, `photos`, `set_photo_review`, `set_positions`,
  `copy_photo`, `set_undo_slot` (**removed**), `touch_session`), `:95–100` (`Store::transaction`, `item`, `session`).
- `crates/infra/src/sqlite/store.rs:467–476` (`enqueue_mirror_tag`: one queued `mirror_tag` job per RAW), `:514–525` (`copy_photo`: inserts the
  photo **and** its `compositions` row), `:526–538` (`set_undo_slot`, **removed**), `:539–547` (`touch_session`: `change_seq + 1`,
  `last_activity_at`), `:1245–1255` (`transaction`: the closure's `Err` drops the `rusqlite::Transaction` → rollback, so "nothing is partly
  applied" is the transaction's property).
- `crates/core/src/domain/photo.rs:66–70` (`UndoSlot`, **removed**), `domain/session.rs:12` (`Session.undo_slot` — the column stays, the field
  is kept and reads NULL from now on), `crates/core/src/error.rs:43–44` (`NothingToUndo`), `events.rs` (`AppEvent`).
- `crates/core/src/services/photos.rs:12–17, 42` (`PhotosService` fields; `assign_unassigned` → `ReviewService::new(...).move_photos` — Assign
  product is a move and must record).
- `crates/integrations/src/lib.rs:72–94` (`App` fields), `:107–124` (`AppHandles`), `:127–136` (`SettingsSetContext`), `:144–176` (`prepare`:
  a runtime change calls `App::build(…)` for a **new** App), `:268–286` (`apply_settings`: `events.transfer_target_to(&next.events)` then
  `mem::replace` — the pattern the history follows), `:287–306` (`handles()`), `:342–352` (`build` signature), `:425–432` (`events`,
  `CompositionService` construction), `:793–830` (`EventHub`: `set_target`, `transfer_target_to`, `impl EventSink`).
- `apps/desktop/src-tauri/src/commands/photos.rs:55–61` (`with_review`), `:109–121` (`photos_undo`), `:166–181` (`photos_assign_unassigned`
  builds `PhotosService`); `commands/items.rs:40–104` (`CommandError` code map); `lib.rs:110–160` (`collect_commands!`), `:35–106`
  (`TauriEvents::emit` — add `history_changed`), `:208–232` (`commands_are_async`: every `#[tauri::command]` is `pub async fn`), `:380–470`
  (`p28_tests`: the mock-webview pattern for a command-boundary row).
- `crates/testkit/src/test_app.rs:53–55` (`review()`), `:68–77` (`mark_published`: `published_at = now, published_seq = change_seq`), `:161`
  (`kill_and_restart`); `crates/testkit/tests/p02_review_order.rs:1–40` (helpers `app`, `session`, `add`, `state`, `seq`, `main`).
- Tests that **change their expectation** in this phase: `crates/testkit/tests/p02_review_order.rs` `c3b_undo_restores_position_and_active`
  `:168–179`, `c3c_only_last_delete_undoes` `:181–191`, `c3d_undo_clamps_prior_position` `:193–206`, `c3e_empty_undo_fails_without_change`
  `:208–214`, `c3f_unreviewed_undo_stays_unpositioned` `:216–223` and the MC-6 row at `:341` — every `a.review().undo(s)` becomes
  `a.history().undo(s)`; **their assertions are unchanged** (the newest step is the last delete; `NothingToUndo` on an empty list; `c3c`'s
  single undo still restores only the last deleted photo). `crates/testkit/tests/p01_storage_identity.rs:91` lists the `undo_slot` column —
  the column stays, the test is untouched. Every other Rust test keeps passing unchanged; `apps/desktop` tests are untouched (the frontend
  still calls `photos_undo`).

## Files expected to change
Core (new): `crates/core/src/services/history.rs`. Core (changed): `crates/core/src/services/{mod.rs, review.rs, photos.rs}`,
`crates/core/src/ports/store.rs`, `crates/core/src/domain/photo.rs`, `crates/core/src/error.rs`, `crates/core/src/events.rs`.
Infra (changed): `crates/infra/src/sqlite/store.rs`. Integrations (changed): `crates/integrations/src/lib.rs`.
Desktop (new): `apps/desktop/src-tauri/src/commands/history.rs`. Desktop (changed): `apps/desktop/src-tauri/src/commands/{mod.rs, photos.rs,
items.rs}`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/bindings.ts` (regenerated).
Testkit (changed): `crates/testkit/src/test_app.rs`. Tests (new): `crates/testkit/tests/p39_undo_history_core.rs`. Tests (changed):
`crates/testkit/tests/p02_review_order.rs` (as listed).
Estimated: 2 new source paths, 12 changed, 1 new test, 1 adjusted test.

## Core mechanisms (unit tests allowed)
`crates/core/src/services/history.rs` — the list rules only, as inline `#[cfg(test)]` tests named `mc30_<slug>`: `record` drops the oldest
beyond `HISTORY_LIMIT`, `record` clears the redo list, `undo`/`redo` move one step between the lists, `state` labels (newest of each list).
Everything else is a behaviour row at the service or command boundary.

## The step model (master plan §6.10 names; designed here for P39 and P40 together, so P40 adds variants, not shape)
```rust
pub const HISTORY_LIMIT: usize = 25;
pub type HistoryStore = Arc<Mutex<HashMap<i64 /* session_id */, SessionHistory>>>;
pub struct History { store: HistoryStore, events: Arc<dyn EventSink> }          // one per App; the store is carried across a settings rebuild
pub struct SessionHistory { undo: VecDeque<Step>, redo: Vec<Step> }
pub struct Step { pub label: String, pub kind: StepKind }
pub enum StepKind {
    Classify { session_id: i64, photos: Vec<PhotoPlace> },                       // classify, Move to Main, delete; one photo or a selection
    Reorder  { session_id: i64, photo_id: i64, from: i64, to: i64 },
    Move     { source: i64, target: i64, photos: Vec<PhotoPlace> },               // move, Assign product, drag onto a session; recorded in `source`
    Copy     { source: i64, target: i64, seq_after: i64, copies: Vec<(i64 /* source photo */, i64 /* copy id */)> },
    Composition { session_id: i64, photos: Vec<CompositionChange> },              // P40
    Attribute   { session_id: i64, item_id: i64, fields: Vec<FieldChange> },      // P40
}
pub struct Place { pub classification: Classification, pub position: Option<i64> }
pub struct PhotoPlace { pub photo_id: i64, pub before: Place, pub after: Place }
pub struct HistoryState { pub undo: Option<String>, pub redo: Option<String> }  // the labels of the steps ⌘Z / ⌘⇧Z would act on
pub enum Direction { Undone, Redone }
pub struct HistoryOutcome { pub label: String, pub direction: Direction, pub sessions: Vec<SessionDto>, pub active_photo_id: Option<i64> }
pub struct HistoryService<'a> { store: &'a dyn Store, clock: &'a dyn Clock, history: &'a History }
```
`History::new(events)`, `History::with_store(store: HistoryStore, events)`, `History::store(&self) -> HistoryStore`, `History::record(session_id,
Step)` (push to `undo`, pop the front beyond 25, **clear `redo`**, emit `HistoryChanged`), `History::state(session_id) -> HistoryState`.
`HistoryService::undo(session_id) -> Result<HistoryOutcome>`, `redo(session_id)`. A step's `label` is computed **at record time** (below).
Errors: `Error::NothingToUndo` (kept), **new** `Error::NothingToRedo` (`nothing_to_redo`), **new** `Error::StepCannotApply { direction: String
("undo" | "redo"), label: String, reason: String }` with `Display` `Can't {direction} {label}: {reason}` (code `step_cannot_apply`, data `label`,
`reason`). **New event** `AppEvent::HistoryChanged { session_id, undo: Option<String>, redo: Option<String> }` → Tauri `history_changed`.

**Labels** (planner reading, Notes (3)): numbers are the photo's **position** in Main or Condition, zero-padded to 2 (`03`); an unpositioned
photo has no number. Classify to main → `Approve 03` (03 = the position after); to condition → `Condition 02`; to deleted → `Delete 03` (03 =
the position before; unpositioned → `Delete photo`); a selection → `Approve 3 photos` / `Delete 3 photos`. Reorder → `Reorder 03 → 01`. Move →
`Move to <name>`, `Move 2 photos to <name>`; copy → `Copy to <name>` …; `<name>` = the target session's item **title**, else its SKU, and
`Unassigned` for session 1. (P40: `Compose 03` / `Compose photo`, `Apply framing to 4 photos`, `Edit <label>`, `Accept all suggestions`, `Clear
<label>`.)

**Reversal rules** (each `undo`/`redo` is one `store.transaction`; a failed precondition returns `Err` inside the closure, so the transaction
rolls back and nothing is partly applied; the step is then **removed** from its list and `HistoryChanged` is emitted, and the error is returned):
- `Classify` undo: for each `PhotoPlace` in reverse order — `tx.photo(id)` (`NotFound` → reason `that photo is gone`), `photo.session_id ==
  session_id` and `photo.classification == after.classification` (else `that photo already changed`); if the current classification is
  ordered, `set_positions(remove_at(group))`; if `before.classification` is ordered, `set_positions(undo_insert(group, id, before.position
  .unwrap_or(1)))` and the photo's position is the one `undo_insert` gave it (clamped `min(k, n_g + 1)`), else `None`; `set_photo_review`;
  `enqueue_mirror_tag`; then `touch_session(session_id)` once. `active_photo_id` = the first photo of the step. Redo = the same with `before`
  and `after` swapped (the photo must read `before.classification`; it is placed at `after.position` through `undo_insert`, which equals
  `n_g + 1` when nothing intervened).
- `Reorder` undo: the photo must be ordered in `session_id` (else `that photo already changed`); `set_positions(position::reorder(group, id,
  from))`; `touch_session`. Redo: `reorder(group, id, to)`.
- `Move` undo (photos back into `source`): `tx.session(source)`; refuse when `published_at.is_some() && change_seq <= published_seq` (the
  `transfer` rule, reason `that session is published`); for each photo (reverse source order): must be in `target` with `after.classification`;
  `remove_at` in the target group when ordered; place in the source group with `undo_insert(before.position.unwrap_or(1))` when `before` is
  ordered; `set_photo_review(id, source, before.classification, position)`; `touch_session(source)` and `touch_session(target)`. Redo: the
  `transfer` rules again (published-and-clean target → `that session is published`; a RAW already in the target → `that session already has this
  photo`); the photos are appended to their groups in source order (= `after`).
- `Copy` undo (copies removed from `target`): `tx.session(target)`; refuse when `published_at.is_some() && published_seq >= seq_after` (a
  publish completed after the copy, so the copies may carry links; reason `that session is published`); for each copy: `tx.photo(copy_id)`
  (`NotFound` → `the copy is gone`); `remove_at` in its group when ordered; **new** `StoreTx::delete_photo_row(copy_id)` (the `photos` row and
  its `compositions` row; nothing else references a never-published copy); `touch_session(target)`. Redo: `copy_photo` again (new ids are
  written back into the step), appended at the end of the target group, `touch_session(target)`; the `transfer` refusals apply.
- After a successful undo the step moves to `redo`; after a redo it moves back to `undo`; `HistoryChanged` is emitted in both cases.
- Recording: `classify` records when the classification changes (a no-op records nothing); `delete` records with `before = (prior, position)`
  and `after = (Deleted, None)`; `reorder` records when `old != next`; `transfer` records one `Move` or `Copy` step **in the source session's
  history** (the first photo's session; a set spanning several sessions is recorded under each source with its own photos — Delegated).
  Recording happens **after** the transaction committed, never inside it.

## Ordered tasks
0. **Registry first (master plan §6.10, P39–P41 block).** The names above; `services/mod.rs` gains `pub mod history`. `ReviewService::new(store,
   clock, history: &History)` (every caller passes one: `with_review`, `PhotosService` (new field `history: &'a History`), `TestApp::review()`).
   `App` and `AppHandles` gain `history: Arc<History>`; `SettingsSetContext` gains `history_store: HistoryStore`; `App::build` gains the parameter
   `history: Option<HistoryStore>` (`None` at open → a fresh store; `prepare` passes `Some(self.history_store)` so a runtime settings change keeps
   every session's steps — MC-30 "kept while the app runs"); `History::with_store(store, events.clone())` is built right after `events`
   (`lib.rs:425`). Removed: `UndoSlot`, `StoreTx::set_undo_slot`, `ReviewService::undo`. Kept: the `sessions.undo_slot` column (migrations are
   never rewritten), `Session.undo_slot` (reads NULL), `Error::NothingToUndo`.
1. **`history.rs`** (core): the types, `History` (`Mutex` inside; `record`, `state`, and the private list moves), the unit tests of §Core
   mechanisms, `HistoryService` with `undo`/`redo` implementing §Reversal rules through `store.transaction` + `position::*`; `HistoryOutcome
   .sessions` = the `SessionDto`s of every touched session (as `ReviewResult.sessions`). The label functions live here too (`fn label_classify
   (…)`, …; the item name is read inside the recording service's transaction with `tx.session(target)` → `tx.item(item_id)`).
2. **`ReviewService` records** (`review.rs`): `classify`/`delete`/`reorder`/`transfer` compute `before`/`after` places inside their transaction
   (they already read the photo and the group) and call `history.record(session, step)` after the commit. `delete` drops the slot code.
   `transfer` keeps every refusal exactly as today.
3. **Store** (`ports/store.rs`, `sqlite/store.rs`): remove `set_undo_slot`; add `StoreTx::delete_photo_row(id: i64) -> Result<()>` (`DELETE FROM
   compositions WHERE photo_id=?; DELETE FROM photos WHERE id=?`).
4. **Events and errors** (`events.rs`, `error.rs`, `commands/items.rs`, `lib.rs` `TauriEvents`): `HistoryChanged` → `history_changed {session_id,
   undo, redo}`; `nothing_to_redo`; `step_cannot_apply` with `data.label`, `data.reason`.
5. **Commands** (`commands/history.rs`, `mod.rs`, `lib.rs`): `history_get(session_id) -> HistoryStateDto { undo: Option<String>, redo:
   Option<String> }`; `history_undo(session_id) -> HistoryOutcomeDto { label: String, direction: "undone" | "redone", sessions:
   Vec<SessionResponse>, active_photo_id: Option<i64> }`; `history_redo(session_id)` (same DTO); all `pub async fn` over `spawn_blocking`
   with `HistoryService::new(app.store.as_ref(), &SystemClock, app.history.as_ref())`. `photos_undo(session_id)` now calls `HistoryService::undo`
   and answers `ReviewResponse { sessions, active_photo_id }` unchanged in shape (compatibility until P41; `nothing_to_undo` still surfaces).
   Regenerate `bindings.ts` (`cargo test -p psm-desktop export_bindings`).
6. **App wiring** (`integrations/src/lib.rs`): task 0's fields and the `prepare`/`build` thread; `handles()` clones `history`.
7. **Testkit** (`test_app.rs`): `review()` passes `self.app.history.as_ref()`; new `history() -> HistoryService<'_>`; `TestApp.events`
   (`CollectingEventSink`) already receives every `AppEvent`.
8. Tests: `p39_undo_history_core.rs` (C1–C7), the p02 call-site edits, `cargo test --workspace`, clippy `-D warnings`, fmt, `npm run typecheck`
   (bindings unchanged for the frontend's existing calls; the new commands appear).

## Criteria (7 criteria, 22 rows; 1 `[rule]` row)
Fixture: `TestApp::new()` with p02's helpers (`session(name)` creates an item + session; `main(s, names)` adds photos classified Main in order;
`add(s, name, class)`; `state(s, class)` = `[(id, position)]` ordered; `seq(s)` = `change_seq`). "Jobs" are read with `store.count_jobs("mirror_tag",
"queued", None)`. Events are read from `a.events.0.lock()`. `H` = `a.history()`, `R` = `a.review()`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `History` list rules (service) | 26 unreviewed photos `u1..u26`; `R.classify(u_i, Main)` for i = 1..26; then `H.undo(s)` 25 times; then once more | after the 25 undos `state(s, Main) == [(u1, 1)]` and `state(s, Unreviewed)` holds `u2..u26` with `None` (the 26th step dropped the oldest, so step 1 is the one that stays); the 26th undo → `Err(NothingToUndo)` and nothing changes | MC-30 ("At most 25 steps per session; recording the 26th drops the oldest"); M36 |
| C1(b) | list rules | `main(s, [a, b])`; `H.undo(s)`; `R.classify(c, Condition)` (c unreviewed); `H.redo(s)` | after the undo `H.state(s).redo == Some("Approve 02")`; after the new step `H.state(s) == { undo: Some("Condition 01"), redo: None }`; the redo → `Err(NothingToRedo)`, `state(s, Main) == [(a, 1)]` unchanged | MC-30 ("A new step clears the redo list"); M36 ("redo kept after a new step") |
| C1(c) | list rules | sessions `s`, `t`; `main(s, [a])` | `H.state(s).undo == Some("Approve 01")`, `H.state(t) == { None, None }`; `H.undo(t)` → `Err(NothingToUndo)` | MC-30 ("One history per session") |
| C1(d) | `History::state` | `main(s, [a, b])`; `R.reorder(b, 1)`; `H.undo(s)`; `H.undo(s)` | states in order: `{ Some("Reorder 02 → 01"), None }` → `{ Some("Approve 02"), Some("Reorder 02 → 01") }` → `{ Some("Approve 01"), Some("Approve 02") }` | MC-30 ("⌘Z undoes the open session's newest step; ⌘⇧Z redoes the newest undone step") |
| C2(a) | classify undo | `main(s, [a, b, c])`; `u` unreviewed; `n = seq(s)`, `j = jobs`; `R.classify(u, Main)`; `out = H.undo(s)` | `out.direction == Undone`, `out.label == "Approve 04"`, `out.active_photo_id == Some(u)`; `state(s, Main) == [(a,1),(b,2),(c,3)]`, `state(s, Unreviewed) == [(u, None)]`; `seq(s) == n + 2` (the classify and the undo); `jobs == j + 1` (the RAW's one queued `mirror_tag` job, re-queued by the undo) | MC-5 (undo of a classify), MC-6 ("undo" is a photo change), ch06 §Mirror; M12, M36 |
| C2(b) | delete undo clamps (after a dropped step) | `s`, `t` (empty); `main(s, [a, b, c])`; `R.delete(c)` (prior place Main 3); `R.move_photos(&[a, b], t)`; `a.mark_published(s)`; `r1 = H.undo(s)`; `out = H.undo(s)` | `r1` is `Err(StepCannotApply { reason: "that session is published", .. })` (the move cannot come back into the published, clean `s`) and is dropped; `out.label == "Delete 03"`, `out.active_photo_id == Some(c)`; `state(s, Main) == [(c, 1)]` (k = 3 clamped to `n_g + 1 = 1`), `state(s, Deleted) == []`, `state(t, Main) == [(a,1),(b,2)]` unchanged | MC-5 ("insert at `min(k, n_g + 1)`"); MC-30 (a dropped step; the next ⌘Z goes on); M12 (positions `1..n` after every undo) |
| C2(c) | delete undo of an unreviewed photo | `u` unreviewed; `R.delete(u)`; `out = H.undo(s)` | `out.label == "Delete photo"`, `state(s, Unreviewed) == [(u, None)]`, `out.active_photo_id == Some(u)` | MC-5 ("unreviewed stays null"); M12 |
| C2(d) | classify redo | `main(s, [a, b])`; `R.classify(b, Condition)`; `H.undo(s)`; `n = seq(s)`; `out = H.redo(s)` | `out.direction == Redone`, `out.label == "Condition 01"`; `state(s, Main) == [(a, 1)]`, `state(s, Condition) == [(b, 1)]`; `seq(s) == n + 1`; `H.state(s) == { Some("Condition 01"), None }` | MC-30 (redo), MC-6; M36 ("redoing them leaves them exactly as after") |
| C2(e) | classify step that cannot apply (a non-step change intervened) | `main(s, [a])`; `R.classify(a, Condition)`; a direct non-step write `store.transaction(|tx| tx.set_photo_review(a, s, Main, Some(1)))` (what a later action that is not a step would do); `r = H.undo(s)` | `r` is `Err(StepCannotApply { direction: "undo", label: "Condition 01", reason: "that photo already changed" })`; `state(s, Main) == [(a, 1)]` and `state(s, Condition) == []` unchanged; the step is dropped: `H.state(s) == { None, None }` | MC-30 ("its photo was removed by a later action that is not a step … nothing changes, it is dropped from the list, and the app says why"); M36 ("a step that cannot apply changes nothing") |
| C3(a) | reorder undo | `main(s, [a, b, c])`; `R.reorder(c, 1)`; `n = seq(s)`; `H.undo(s)` | `state(s, Main) == [(a,1),(b,2),(c,3)]`; `seq(s) == n + 1`; `H.state(s).redo == Some("Reorder 03 → 01")` | MC-30 ("reorder within a group → the prior position"); MC-6; M12 |
| C3(b) | reorder redo | from C3(a); `H.redo(s)` | `state(s, Main) == [(c,1),(a,2),(b,3)]` | M36 |
| C4(a) | move undo | `s`, `t`; `main(s, [a, b, c])`; `main(t, [x])`; `R.move_photos(&[b], t)`; `(ns, nt) = (seq(s), seq(t))`; `out = H.undo(s)` | `H.state(s).undo` before the undo `== Some("Move to <t's title>")` and `H.state(t).undo == Some("Approve 01")` (the move is recorded in the source only); after: `state(s, Main) == [(a,1),(b,2),(c,3)]`, `state(t, Main) == [(x, 1)]`; `(seq(s), seq(t)) == (ns + 1, nt + 1)`; `out.sessions` names `s` and `t` | MC-30 ("the photos back in the source session at their prior classification and position; recorded in the source session's history"); MC-6; M12 |
| C4(b) | copy undo | `main(s, [a])`; `main(t, [x])`; `R.copy_photos(&[a], t)`; `copy` = the new row id in `t`; `nt = seq(t)`; `H.undo(s)` | `store.photo(copy)` → `Err(NotFound)`; `SELECT COUNT(*) FROM compositions WHERE photo_id = copy` is 0; `state(t, Main) == [(x, 1)]`; `state(s, Main) == [(a, 1)]`; `seq(t) == nt + 1`; `H.state(s).redo == Some("Copy to <t's title>")` | MC-30 ("copy to another session → the copies removed from the target"); M12 |
| C4(c) | move undo blocked by a published source | `main(s, [a, b])`; `main(t, [])` (empty session); `R.reorder(b, 1)`; `R.move_photos(&[a], t)`; `a.mark_published(s)`; `r1 = H.undo(s)`; `r2 = H.undo(s)` | `r1` is `Err(StepCannotApply { direction: "undo", label: "Move to <t>", reason: "that session is published" })` and its `to_string()` is `Can't undo Move to <t>: that session is published`; nothing changed: `state(t, Main) == [(a, 1)]`, `state(s, Main) == [(b, 1)]`; the step is dropped: `H.state(s).undo == Some("Reorder 02 → 01")` and `r2` succeeds (`state(s, Main)` still `[(b, 1)]` since the reorder of one remaining photo reverts to the same list — assert `r2.label == "Reorder 02 → 01"`) | MC-30 ("A step that can no longer apply … is not partly applied: nothing changes, it is dropped from the list, and the app says why … The next ⌘Z goes on to the step before it"); M36 ("a step that cannot apply changes nothing") |
| C4(d) | copy undo blocked after a publish | `main(s, [a])`; `main(t, [x])`; `R.copy_photos(&[a], t)`; `a.mark_published(t)` (`published_seq = change_seq` ≥ the copy's `seq_after`); `H.undo(s)` | `Err(StepCannotApply { reason: "that session is published", .. })`; the copy row still exists in `t` at position 2; `H.state(s).undo == Some("Approve 01")` (the step before) | MC-30 (as C4(c)); MC-2 §Move and copy |
| C4(e) | move redo | from C4(a) after the undo; `H.redo(s)` | `state(s, Main) == [(a,1),(c,2)]`, `state(t, Main) == [(x,1),(b,2)]`; `H.state(s) == { Some("Move to <t>"), None }` | M36 |
| C5(a) | not steps | `s`; `a.add_photo(s, "n.ARW")` (arrival row); `a.sessions().create(item)` for a new item; `a.settings_set(json!({"theme": "dark"}))` | `H.state(s) == { None, None }` for `s` and for the new session | MC-30 §Not steps ("capture and import arrival … session create … settings") |
| C5(b) | kept across a settings rebuild | `main(s, [a])`; `a.settings_set(json!({"sku_prefix": "PX"}))` (a runtime change → `App::build`); `H' = a.history()`; `H'.state(s)`; `H'.undo(s)` | `H'.state(s).undo == Some("Approve 01")`; the undo succeeds: `state(s, Unreviewed) == [(a, None)]` | MC-30 ("kept while the app runs") |
| C5(c) | cleared on quit | `main(s, [a])`; `a.kill_and_restart()`; `a.history().state(s)` | `{ None, None }`; `a.history().undo(s)` → `Err(NothingToUndo)`; `state(s, Main) == [(a, 1)]` (the data is on disk, the history is not) | MC-30 ("quitting clears it") |
| C6(a) | events | `main(s, [a])`; `H.undo(s)`; `H.redo(s)`; read `a.events` | the `HistoryChanged` events for `s`, in order: `{ undo: Some("Approve 01"), redo: None }` (record), `{ None, Some("Approve 01") }` (undo), `{ Some("Approve 01"), None }` (redo); after C4(c)'s blocked undo one more `HistoryChanged` is emitted whose `undo` is the step before | MC-30 (the header icons "name the step they would act on"); M36 |
| C7(a) | command boundary (mock webview, `p28_tests` pattern) | `main(s, [a, b])`; `history_get {sessionId: s}`; `history_undo {sessionId: s}`; `history_get`; `history_redo`; `history_redo` | `history_get` → `{"undo":"Approve 02","redo":null}`; `history_undo` → `{"label":"Approve 02","direction":"undone","active_photo_id": b, "sessions":[{"id": s, …}]}`; then `{"undo":"Approve 01","redo":"Approve 02"}`; the first `history_redo` → `{"label":"Approve 02","direction":"redone", …}`; the second → error `{"code":"nothing_to_redo", …}` | MC-30; master plan §6.7 |
| C7(b) | `photos_undo` compatibility (mock webview) | `main(s, [a])`; `R.delete(a)`; `photos_undo {sessionId: s}` three times | first → `{"sessions":[{"id": s, …}],"active_photo_id": a}` and `state(s, Main) == [(a, 1)]` (the delete undone); second → `active_photo_id: a` and `state(s, Unreviewed) == [(a, None)]` (the classify undone); third → error `{"code":"nothing_to_undo", …}` | `[rule]` master plan §6.7 (`photos_undo` answers in its old shape until P41); MC-30 |

## Delegated choices
Module layout inside `history.rs` (one file or a `history/` folder); whether `label_*` helpers take the tx or precomputed names; the exact
`Display` of `NothingToRedo`; the DTO field names beyond those fixed above; how `HistoryService` reads touched sessions for `outcome.sessions`
(`ReviewService::result` pattern); whether a transfer spanning several source sessions records one step per source (recommended) or refuses
to record; the `HistoryChanged` payload after a dropped step (emit once, after the drop).

## Manual try-out
(The frontend still uses the old ⌘Z path until P41; these steps check the backend through the running app.)
1. Start the app, open a session, approve three photos with A, then press ⌘Z three times: each press takes the newest approval back (the photo
   returns to unreviewed and the stage shows it); a fourth ⌘Z says "Nothing to undo."
2. Delete a photo, drag another to a new place, press ⌘Z: the drag is undone first (the order), ⌘Z again brings the deleted photo back at its
   old place. In Capture One the colour tags follow (green/blue/none) within a few seconds.
3. Move a photo to another session, go back, press ⌘Z: the photo is back where it was.
4. Change a setting (Settings → SKU prefix), return to the session: ⌘Z still undoes the steps from before. Quit and relaunch: ⌘Z says
   "Nothing to undo."

## Notes
- **Planner readings (reversible; for the narrative's "Choices made for you")**:
  (1) A step is undone only if its photos are still where the step left them (same session, same classification); a photo that another
  *step* moved is handled by undoing that step first (LIFO), and a photo changed by anything else makes the step unusable: it is dropped with
  "Can't undo …: that photo already changed".
  (2) Undoing a copy deletes the copied rows outright (they never existed before the step); it is refused once a publish has completed in the
  target after the copy (the copies may be on the shop), with "that session is published". Undoing a move back into a published, clean
  session is refused the same way (MC-2's own rule for moving into a published session).
  (3) Step names use the photo's position in Main or Condition ("Approve 03"), the target session's product title for moves and copies,
  "photo"/"photos" where there is no position; the exact words are in §Labels and are the contract the rows assert.
  (4) Undo of a classify or delete makes the (first) restored photo active (MC-5); other steps leave the active photo alone.
  (5) The history survives a settings change (the app rebuilds its services; the lists are carried over) and is empty after a relaunch.
  (6) The old single-slot column `sessions.undo_slot` stays in the schema, unused (migrations are never rewritten).
- **Mechanism-check findings this plan builds on** (handoff `handoff_polish_and_undo_plan_r1.md` §Mechanism check): `Store::transaction`
  rolls back on `Err` (`store.rs:1245–1255`), so one closure per reversal gives "not partly applied" for free; `touch_session` is the only
  `change_seq` writer for review steps (`:539`); `enqueue_mirror_tag` coalesces to one queued job per RAW (`:467–476`), so the mirror on undo is
  the same job classify uses; `copy_photo` also copies the composition row (`:514–525`) — hence `delete_photo_row` removes both; the only
  `App`-rebuild path is `apply_settings` → `prepare` → `App::build` (`lib.rs:144–176, 268–286`), where `events.transfer_target_to` shows how a
  live handle is carried — the history store is carried the same way; no code path other than `ReviewService` writes `photos.classification`
  or `position` (grep `set_photo_review|set_positions`: `review.rs` only), so recording inside `ReviewService` covers every route (keys, buttons,
  menus, drag, `PhotosService::assign_unassigned`, the phone remote later).
- **Hazards**: `HistoryChanged` must be emitted through the **current** App's `EventHub` — `History::with_store(store, next.events.clone())`
  in `build`, never a clone of the old `History` (its hub has no target after `transfer_target_to`). Recording after the commit, not inside the
  closure (a rolled-back transaction must record nothing). `undo_insert(group, id, k)` expects `group` **without** the photo (as `review.rs:130`
  does after `remove_at`). The `commands_are_async` test counts `#[tauri::command]` against `pub async fn` — the three new commands are async.
  `photos_undo` must keep its `ReviewResponse` shape or `npm run typecheck` goes red (the frontend is untouched in this phase).
- **Tests that change their expectation**: the six p02 call sites listed in Read first (API rename only; assertions unchanged). No other test
  may change.
- **Projection** is on (`projection: yes`): MC-30 is a new load-bearing mechanism (charter rule 6), and the step model's shape is shared with
  P40 — the projectionist checks that every reversal rule above is decidable from the code and that the `App::build` thread is complete.
- Registry additions made by the planner in master plan §6.4, §6.6, §6.7, §6.10. No intention-book change.
- Estimated size: ~16 paths, ~1 300 lines (≈ 650 core/infra/commands, ≈ 650 tests).

## Review log

