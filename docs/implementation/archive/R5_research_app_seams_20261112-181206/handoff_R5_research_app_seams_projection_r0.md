---
plan: R5_research_app_seams
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-12
actor: pipeline-projectionist (Opus 5.5), attempt 7
---
# Projection of R5_research_app_seams (round 0, attempt 7)

Most of this plan is ready to build. All 41 test rows can be written as they stand, and every setup they need can
be produced by the code that exists today. Three gaps remain, all in undo and redo. First, the plan leaves the
locking choice open, and one of the two options it suggests can freeze the app. Second, a redo can fail forever
after the photographer works in another session. Third, the contracts do not say what happens when a redo of a
step made before Publish touches a field the publish is waiting on. That case can leave an item waiting on a field
that no longer shows a conflict. None of this needs you. Each gap is the smallest rule that finishes rules you
already approved, so the orchestrator can settle them like the earlier rounds and then start the build.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. T3 is an intention gap, but it completes the owner's card-1 answer in the same way 08B
changelog 6, 8 and 9 did: the orchestrator writes the wording, and no new product choice is made.

## Decision ledger

| # | Decision point | Classification | Proposed routing |
|---|---|---|---|
| T1 | Lock order between an attribute action's store transaction and `History` (U1 "Decided by the orchestrator (Q2)": "a lock or a returned flag is the implementer's") | **plan gap** (one of the two permitted options deadlocks) | Amend U1 Q2 (text below). |
| T2 | How redo re-inserts the feedback rows it kept (U1 task 5, P6 "redo re-inserts exactly those"; the existing port method `reinsert_feedback` keeps the old `id`) | **plan gap** (silent permanent redo failure) | Amend U1 P6 decision. Optional extra clause on A08·C4(b) (below). |
| T3 | A **redo of a step recorded before the confirmation** whose before-row is the conflict now pending on a `held` item | **intention gap** (MC-A10 and MC-A12 cover redo of a resolution, not a redo that becomes one) | Orchestrator adds an 08B wording (MC-A10/MC-A12 changelog 10), then U2 task 5 and one row (below). No owner card. |
| T4 | U1 perimeter: there is no store method to read feedback rows by token or to update `final_value` (Q2 coalescing, P6), and `ports/store.rs` and `sqlite/store.rs` are not in U1's Files | **plan gap** (perimeter) | Add both files to U1 Files, or state that the existing methods are to be used (below). |
| T5 | U2 perimeter: A09·C2(a) puts a test in the desktop `r5_` module in `apps/desktop/src-tauri/src/lib.rs`, which is not in U2's Files. U2 also lists `publish_partition.rs` as **New**, but U1 already created it. | **plan gap** (wording) | Add `lib.rs` to U2's Edited list and move `publish_partition.rs` from New to Edited. |
| F1 | Which held item a resolution acts on when a session has more than one `held` item. The backend `confirm` does not refuse a session that is already held; only the UI hides Publish. | **free choice** | Delegate explicitly in U2 Delegated choices: "the session's latest publish item, as the row label reads it". |

## Findings

### T1: Q2 permits a lock order that deadlocks (U1, "Decided by the orchestrator", Q2 bullet)
- What exists: `SqliteStore::transaction` holds the connection mutex for the whole closure
  (`crates/infra/src/sqlite/store.rs:1651–1660`). `HistoryService::reverse` takes `History.operation` first and then
  opens the store transaction (`crates/core/src/services/history.rs:269` then `:286`). `History::record` takes
  `operation` itself (`history.rs:154`), and `std::sync::Mutex` is not reentrant.
- Q2 says coalescing is decided "once, inside the action's transaction … the mechanism (a lock or a returned flag)
  is the implementer's". If an implementer takes `History.operation` inside the action's transaction, the action
  holds connection → operation while a concurrent undo holds operation → connection, and both block forever.
  Commands run on `spawn_blocking` threads, so form typing (debounced writes) and Cmd+Z do overlap. The
  returned-flag option avoids the deadlock but leaves a gap between commit and `record`. An undo in that gap pops
  the step the decision merged into. `record` "does not re-decide", so it merges into the wrong step, or the
  feedback row's `final_value` was updated for a step that is no longer the last one.
- **Amendment (replace the Q2 mechanism sentence):** "The action takes `History.operation` **before** it opens its
  store transaction and holds it until the step is recorded, as `retire_photo` and `reverse` do (order:
  operation → store). `History` gains a record variant that runs under that held guard. Nothing acquires
  `operation` inside a store transaction." No new row; the rows already cover the behaviour, and the hazard is a
  lock order.

### T2: Redo re-insert with the old id fails forever after another session's action (U1 task 5, P6)
- What exists: `research_feedback.id INTEGER PRIMARY KEY` without `AUTOINCREMENT` (`crates/infra/src/sqlite/ensure.rs:32–33`),
  so SQLite gives a new row `max(id)+1` and reuses the id of a deleted maximum row. `reinsert_feedback` inserts
  with the stored `id` (`store.rs:963–966`). History is per session (`History.store: HashMap<session_id, …>`), so an
  action in session 2 clears only session 2's redo stack.
- Failure: in session 1, clear suggestion A (feedback id 7, the newest), then undo (row 7 deleted). In session 2,
  clear another suggestion (the new feedback row gets id 7). In session 1, redo: `reinsert_feedback(id 7)` hits a
  PRIMARY KEY conflict, so `reverse` returns a database error. That is not `StepCannotApply`, so the step stays on
  the redo stack and every later redo fails the same way. MA18 ("feedback rows exist exactly for the actions …
  currently applied") cannot hold.
- Nothing references feedback ids (searched `crates/`, `apps/desktop/src-tauri/`), and A08·C4(b) asserts the token,
  not the id.
- **Amendment (P6 bullet):** "redo re-inserts them with `insert_feedback` (a fresh id); the old id is not kept."
- Optional row clause, cheap and discriminating, on A08·C4(b): "… and the same holds when, between the undo and
  the redo, a clear in another session wrote a feedback row".

### T3: A redo of a pre-confirmation step can empty a pending key with no resolution (U2 task 5; 08B MC-A10/MC-A12)
- Path: research writes conflict A on `k` (prior D). The photographer clears `k` (step S: before = the conflict,
  after = `ai` + `cleared`, prior dropped by U1 task 3), then undoes S. The guard holds and the row is the conflict
  again. The photographer then publishes, so `k` is pending and the item is `held`. Then redo S: the guard compares
  the current row with S's before-row (the conflict), they match, and the row becomes `ai` + `cleared`. The same
  happens with Edit, Keep previous, Use suggestion and Accept all in place of Clear. Confirmation does not touch
  the history (no history reference in `services/publish.rs` or `commands/publish.rs`), so the redo stack survives
  it. Only redo can reach this. No action produces a conflict as an after-row, so an undo cannot.
- Plan as written: task 5 re-records only "the entries it wrote" on redo, and S wrote none. The item stays `held`,
  and its label still says "Waiting for you · 1 conflicts", but `k` is no longer a conflict. Keep previous and Use
  suggestion are refused by the U1 decided rule (`InvalidValue`), so only Edit or Cancel gets the item out, and the
  form shows nothing to resolve. This is the trap that 08B changelog 8 removed for layout changes.
- The contracts say how a resolution is undone and redone (changelog 6). They do not say what a redo does when it
  replays an action that becomes a resolution.
- **Proposed 08B wording (orchestrator, completes card 1 as changelog 6 and 8 did):** "A redo that changes a key
  pending without an entry on a `held` item is that key's resolution, recorded with the redo's step token as the
  action it replays (Clear → no value; Edit → the value, no value when it does not fit; Keep previous → the prior's
  value; Use suggestion / Accept all → the suggestion). It releases the item when it was the last pending key."
  This is implementable from what the step already keeps: S's before-row is `ai` + `suggested`, so S carries the
  feedback row giving the action and the final value (P6). In history apply, it is the same `resolve_pending_in`
  call task 5 already requires for redoing a resolution.
- Proposed row (U2, criterion C4): **A09·C4(j)**, boundary redo. Input: conflict on `k`, Clear, undo, confirm (held,
  one pending key), redo. Exact outcome: `k` has an entry "no value" with the redo step's token, the step is
  `prepare` with one job, and `k` is in no write after `tick`. Trace MC-A10 (changelog 10).
- Alternative, if the orchestrator prefers not to add a rule: such a redo is refused like a guard failure (nothing
  changes, the step is dropped, and the notice needs its own text). I recommend the first rule because it reuses
  existing code and adds no new notice text.

### T4: U1 perimeter lacks the store files its decided rules imply (U1 Files)
- Q2 updates the `edited` feedback row's `final_value` on coalescing, and P6 has undo read the rows that carry the
  step's token. `StoreTx` has `feedback_by_key`, `insert_feedback`, `delete_feedback_by_step` and
  `reinsert_feedback` (`ports/store.rs:106–113`), but no read by token and no update. Both rules can be built from
  the existing methods: filter `feedback_by_key` by token for each field of the step, and update by delete plus
  insert inside the same transaction. The plan does not say which way to go, and the store files appear only in
  U2's Files.
- **Amendment:** add `crates/core/src/ports/store.rs` and `crates/infra/src/sqlite/store.rs` to U1 Files
  ("feedback by step token; update `final_value`"). Alternatively, write the existing-methods route into the Q2/P6
  bullets.

### T5: U2 Files wording (U2 Files)
Add `apps/desktop/src-tauri/src/lib.rs` (the `r5_` module gains the A09·C2(a) code assertion). Move
`crates/core/src/mechanisms/publish_partition.rs` from New to Edited, because U1 task 0 creates it.

## Reality checks (all pass unless listed above)
- **Gates.** Intention map RATIFIED (2026-10-24). 08A RATIFIED and 08B CONTRACTS RATIFIED (2026-11-12). The tracker
  for `depends_on` R3 reads APPROVED (2026-11-12). No upstream handoff for R5 is in OWNER_DECISIONS_PENDING. Attempt
  6 left no handoff (STATUS_research).
- **Paths.** Every Edited path in U1 and U2 exists. The New paths do not exist yet. Read-first line numbers have
  drifted by about +5 in `attributes.rs` (`accept_all` 516, `clear` 551) and by +6 to +11 in `publish.rs`
  (`frozen_attributes` 478, `preview` 524, `confirm` 598–731, `run_item` 768). The plan already says to find
  functions by name. Correct as cited: `history.rs` attribute arm 714–759, `ports/store.rs` `enqueue_publish_item`
  294, `store.rs` session-list held count 1701–1705, `publish.rs` `activity` Held text 276–281.
- **Counts.** There are 42 `confirm(` call sites across the 15 named testkit files (12+2+2+1+1+2+1+1+3+2+2+5+5+1+2),
  plus the command. `publishConfirm` and preview literals appear only in `p15_publish_ui.test.tsx` and
  `p32_reframe.test.tsx`, as listed. The research `is_conflict` has no caller outside `write_table.rs`, so the
  one-line delegation is safe.
- **Prior phases' outputs, checked in code.** `publish_items` has `frozen_attribute_origin`, `pending_conflicts`,
  `resolved_attributes` and `released_at` (`store.rs:484, 766`). `PublishStep::Held` maps to `"held"`. The session
  list fills `unresolved_conflicts` as pending minus resolved. `activity` has the Held arm. `research_feedback` and
  its four port methods exist. `writes::{suggest, lower}` exist with the outcomes the rows name.
  `active_publish_items` lists only `prepare, upload, update, describe`, so the startup re-queue
  (`integrations/src/lib.rs:536–538`), `earlier_in_prepare`, `start_next_in_batch`, the masters-needed set and
  `prepare_masters::busy` all leave `held` out. That is correct (a held item must not block Capture One), and no
  change is needed.
- **Dependency graph.** `psm-research` dev-depends on `psm-testkit`, and U1 makes `psm-testkit` depend on
  `psm-research`. Cargo allows this dev-dependency cycle, and only research's integration tests use testkit, so no
  duplicate-type problem arises. `psm-desktop` already dev-depends on `psm-testkit` and depends on `psm-research`,
  so the A08·C5(a) test has its precedent (`lib.rs` `p39_tests`, mock runtime).
- **Cited sections resolve.** `master_plan_research.md` §6.3, §6.4, §6.6, §7.2, §7.4. `master_plan.md` §6.5–§6.7,
  §7.5. 05A research pointers at lines 205 (MC-A10) and 307 (MC-A12). `research_row_mapping.md`. The
  `backlog_research/` folder exists.

## Criteria decidability (both questions, per row)
All 41 rows can be written now with one exact outcome. The checks below confirm that each fixture whose shape is
owned outside our code can be produced by the installed code:
- **A08·C1(c), C4(a).** After Keep previous, `writes::suggest` of A returns `MatchesCleared`, because `cleared_set`
  includes `KeptPrevious` suggested values (`writes.rs`). After an undone clear, the feedback is gone and the row is
  `ai` + `suggested` A again. `decide_write` then returns `Write`, and `same_content` makes it `Unchanged`, so the
  row discriminates.
- **A08·C3(a), C3(b).** The write table lets research write B over `ai` + `cleared` A (B is not in the cleared set)
  and replace an `ai` + `suggested` A, so both fixtures can be produced.
- **A08·C3(c).** `fill_defaults` records one two-field step of `default` rows, and research may write over a live
  `default`, provided the keys are in a non-Product preset of the item's type with defaults.
- **A08·C4(c).** Needs 26 imported definitions (the shop fixture has fewer). This is a fixture-building detail, not a
  gap.
- **A09·C1(d).** `decide_write` keeps the prior when replacing `ai` + `suggested`, and the `AlreadyHolds` check
  applies only to `default`/`integration` rows, so a `low` value S-equal to the prior ("oak" against "Oak", text
  `name_key`) can be produced and is not a conflict.
- **A09·C1(g).** `refresh_schema` replaces the definition set, so the fixture can be produced.
- **A09·C4(i).** `read_integration_values` stores an ill-fitting shop value with `valid = false`. The write table
  allows research over a live `integration` row "valid or not" and saves the invalid prior, so the fixture can be
  produced.
- **A09·C2(a).** `CommandError` has a public `code` field (`commands/items.rs:28–32`).

One wording note, not a gap. In A08·C1(d), if the `ai` + `accepted` B is made with `accept`, that action also
writes an `accepted` feedback row. "Feedback `cleared` … twice" counts only the cleared rows, and the test should
assert it that way.

## Trace verification
Every trace cell resolves and supports its row: MA8, MA17, MA18 and M5/M17/M18/M20 in chapter 14, and MC-A10,
MC-A12 and 07A MC-13 in the book. Every ledger entry the milestone claims is served. MA17 is served by A09·C1–C4 and
C7, MA8 by A09·C1(b) and C8(a), and MA18 by A08·C1–C4. T2 and T3 are cases where MA18 and MA17 can fail with the
plan as written, which is why each is paired with an amendment.
