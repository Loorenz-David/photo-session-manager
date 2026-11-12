---
plan: R5_research_app_seams
role: implement
round: 1
state: DONE
date: 2026-11-12
actor: opus
---

# R5 research app seams — implement r1

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## What was built

The two places where the app's own behaviour changes for research are now in place. There is still no new UI (R6). Research is not wired yet (R7), so in the app nothing writes suggestions.

**What you do with a suggestion is remembered (Unit 1).**
- **Looks right** now *accepts* the suggestion. The field stays an AI value marked accepted; before, it became your own edit. On an accepted field the popover offers only Clear.
- A conflict has three actions: **Keep previous**, **Use suggestion** and **Edit**. A conflict is a suggestion that replaced a preset default or a Shopify value and differs from it. Each action is one undo step.
- Every action on a suggestion writes one feedback row in the same save: accept, keep previous, clear, edit, and Accept all (which skips low-confidence suggestions and conflicts). A later research run uses these rows; for example, it never suggests a value you cleared or kept away.
- **Undo and Redo never erase a newer suggestion.** A step is applied only if every field it touches still holds what the step left (undo) or found (redo). Otherwise nothing changes, the step is dropped, and the notice says "that field was changed by research". Undoing a step removes its feedback rows, and redoing puts them back.

**Publishing with unchecked suggestions (Unit 2).**
- At confirmation, each item's values are split three ways:
  - frozen: sent as today;
  - withheld: a low-confidence suggestion that replaced nothing, never published;
  - pending: a conflict, which waits for you.
- The preview reports the withheld count and the conflicts. Confirm re-checks both inside its save and refuses with "AI suggestions changed while you were confirming" if either changed.
- An item with conflicts starts at the new step **held**. It sends nothing and runs no job. The other items of the same confirmation go ahead.
- Resolving a conflict (any action, in the form or later in the popover) records that field's value for this publish. The last resolution releases the item, and its job starts.
- Undoing a resolution while the item waits makes the field wait again. Redoing it records it again.
- Cancel on a held item is immediate. A held item stays held after a restart.
- The run sends the frozen values plus each resolved value. A field resolved with Clear, or with a value that doesn't fit, is never sent.

Checkpoints: U1 `c8b22ae`, U2 `1e56542`.

## Tests → criterion rows
Suite file U1: `crates/testkit/tests/suite/research/r5_feedback_and_undo.rs`; U2: `…/research/r5_publish_hold.rs`. Every row has one test at the boundary it names; each asserts the row's exact outcome.

| Row | Test | Shape vs row |
|---|---|---|
| A08·C1(a) | `a08_c1a_accept_writes_accepted_feedback_with_step_token` | row, prior null, one feedback row with every field, token = last undo step's token |
| A08·C1(b) | `a08_c1b_accept_all_skips_low_and_conflicts` | count 1, three rows' status, one feedback row |
| A08·C1(c) | `a08_c1c_keep_previous_restores_prior_and_joins_cleared_set` | whole row equal (but `updated_at`), feedback, later suggest → `MatchesCleared` |
| A08·C1(d) | `a08_c1d_clear_suggested_and_accepted_rows` | both rows cleared, prior null, one `cleared` row each, final null |
| A08·C1(e) | `a08_c1e_human_edit_of_suggestion_is_edited_feedback` | as row |
| A08·C1(f) | `a08_c1f_actions_on_non_suggestion_rows_write_no_feedback` | the three cases, no new feedback |
| A08·C2(a) | `a08_c2a_research_write_is_no_feedback_and_no_step` | as row |
| A08·C3(a) | `a08_c3a_undo_refused_after_research_write` | notice, row unchanged, step gone |
| A08·C3(b) | `a08_c3b_redo_refused_after_research_replaced_suggestion` | as row |
| A08·C3(c) | `a08_c3c_two_field_undo_refused_when_one_field_changed` | as row (`fill_defaults`) |
| A08·C3(d) | `a08_c3d_undo_of_unchanged_field_restores_row` | equal but `updated_at` |
| A08·C4(a) | `a08_c4a_undo_deletes_feedback_and_cleared_value_returns` | feedback gone, `Unchanged`, row untouched |
| A08·C4(b) | `a08_c4b_redo_reinserts_feedback_with_same_token` | same token, includes the other session's clear between |
| A08·C4(c) | `a08_c4c_feedback_outlives_the_step_limit` | 26 rows, undo depth 25 |
| A08·C5(a) | desktop `r5_command_tests::r5_a08_c5a_attribute_commands_are_one_step_each` (Tauri mock runtime) | one step per command; rows of C1(a), C1(c), C1(a), C1(e) |
| A08·C5(b) | `p11_form_ui.test.tsx` "C5(b), A08·C5(b) popover…" | accept called with item and key, `setAttributeValue` not; accepted row: no Looks right, Clear stays |
| A09·C1(a) | `a09_c1a_every_confirmed_kind_of_row_is_frozen` | as row |
| A09·C1(b) | `a09_c1b_low_suggestion_without_prior_is_withheld` | absent, `n_withheld` 1, origin recorded |
| A09·C1(c) | `a09_c1c_conflicts_at_any_confidence_are_pending_not_withheld` | pending with prior, origin, suggestion, confidence; `n_withheld` 0 |
| A09·C1(d) | `a09_c1d_low_suggestion_same_as_its_prior_is_frozen` | made by the row's path (B over D, then d at low) |
| A09·C1(e) | `a09_c1e_cleared_and_invalid_rows_are_not_frozen` | as row |
| A09·C1(f) | `a09_c1f_conflict_outside_the_layout_is_left_out` | as row |
| A09·C1(g) | `a09_c1g_conflict_whose_definition_was_dropped_is_left_out` | as row; the local export `item.json` (the swapped staged file) checked after the run |
| A09·C2(a) | `a09_c2a_withheld_count_changed_refuses_the_confirmation` + desktop `r5_a09_c2a_suggestions_changed_is_its_own_code` | both cases, no run row, code `suggestions_changed` |
| A09·C2(b) | `a09_c2b_conflict_resolved_before_confirm_refuses_the_confirmation` | as row |
| A09·C3(a) | `a09_c3a_held_item_sends_nothing_while_the_other_publishes` | as row |
| A09·C3(b) | `a09_c3b_every_mutation_follows_the_confirmation` | the p13 form |
| A09·C4(a) | `a09_c4a_keep_previous_records_the_prior_and_still_waits` | as row |
| A09·C4(b) | `a09_c4b_last_resolution_releases_with_resolved_values_and_stock` | as row, including stock request and `FakeShop.stock` |
| A09·C4(c) | `a09_c4c_edit_sends_the_edit_and_clear_sends_nothing` | two items, both cases |
| A09·C4(d) | `a09_c4d_undo_of_a_resolution_makes_the_key_wait_again` | as row |
| A09·C4(e) | `a09_c4e_later_edit_and_its_undo_leave_the_entry` | as row |
| A09·C4(f) | `a09_c4f_redo_records_the_resolution_again` | as row |
| A09·C4(g) | `a09_c4g_typing_on_a_held_item_resolves_at_the_first_save` | as row |
| A09·C4(j) | `a09_c4j_redo_after_confirmation_is_the_resolution` | as row |
| A09·C4(h) | `a09_c4h_accept_all_resolves_a_key_that_is_no_longer_a_conflict` | as row |
| A09·C4(i) | `a09_c4i_keep_previous_of_an_invalid_shop_value_sends_nothing` | an integer field whose shop value `"many"` is read invalid |
| A09·C6(a) | `a09_c6a_cancel_of_a_held_item_is_immediate` | as row |
| A09·C6(b) | `a09_c6b_held_item_stays_held_across_a_restart` | as row |
| A09·C6(c) | `a09_c6c_release_survives_a_restart_and_runs_once` | as row (one product, one media) |
| A09·C7(a) | `a09_c7a_conflict_after_confirmation_does_not_hold` | as row |
| A09·C8(a) | `a09_c8a_origin_and_resolutions_account_for_every_sent_key` | as row, with a withheld key present |

Core mechanisms (unit tests, `crates/core/src/mechanisms/publish_partition.rs`): `is_conflict_needs_a_differing_prior_on_a_suggestion`, `partition_splits_frozen_withheld_pending`, `partition_leaves_out_conflicts_outside_layout_or_without_definition`.
Existing tests changed only as the plan says: `p22_form_tidy.test.tsx` C4(a) (Looks right → accept command), `p15_publish_ui.test.tsx` and `p32_reframe.test.tsx` (preview literal, shown set), the 42 testkit `confirm` calls. No test without a row.

## Assumptions
1. **Step token** is a UUID v4 made per action in `AttributesService::act` (delegated). Feedback rows from an action without a history still get one.
2. **Step labels**: "Accept <label>", "Keep previous <label>"; Use suggestion is Accept, Edit on a conflict is "Edit <label>" (delegated; "Clear <label>" style).
3. **Feedback** is written for `set_value` only when the provenance is human. An `ai` row without a confidence is recorded as `low` (no such row exists).
4. **No merge** when the edit writes a feedback row or records a resolution, so every feedback row's token names a step of its own (the L15 rule, applied at its smallest).
5. **`accept_all`** treats a suggestion with a prior whose key has no definition as a conflict and skips it (fail closed, as `partition` does).
6. **JSON layouts (delegated):**
   - `pending_conflicts[key]` = the conflict as the preview shows it (`key, label, prior_value, prior_valid, prior_provenance, suggestion, confidence`; values canonical JSON text);
   - `frozen_attribute_origin[key]` = `{class: frozen|withheld|pending, provenance, status, confidence, prior}`;
   - `resolved_attributes[key]` = `{value, action, at, step_token}` (`value` null = "no value").
7. **`shown`** is `[{session_id, n_withheld, conflict_keys}]` (delegated). Conflict keys are compared as a set.
8. **Merging at read time** (implementer's choice): `PublishItem::run_attributes()` is read by `ensure_export` (`run_staged`) and `update` (`write_set`, plus its condition, description and tags reads).
9. **Which held item a resolution acts on**: for each of the item's sessions, the session's latest publish item, as the label reads it, if held; the newest of those (F1).
10. **Redo resolution**: the value comes from the entry the step recorded, else from the step's after-row (live and valid → its value, else "no value"). The action comes from that entry, else the step's kept feedback row, else edited or cleared by liveness.
11. **Cancel of a held item** runs in a store transaction (`… WHERE step='held'`). If a resolution released it in between, the ordinary cancel path runs.
12. **Wrong-state actions** are refused with `invalid_value` and write nothing (orchestrator decision; no row tests it). This covers accept on a non-suggestion, Keep previous on a non-conflict, and Edit without a value or a value with another action.
13. **`confirm_as_shown`**: a preview that fails is passed on as an empty shown set, so `confirm` reports its own error as before.

## Deviations
- **`ConfirmAsShown`** is a testkit trait on `PublishService`, with `TestApp::confirm_as_shown` delegating to it. The plan names only the `TestApp` method, but several call sites hold the service itself or a second app (`b`). Each site got one rename plus one `use` line.
- **Files beyond the unit lists:** `Cargo.lock` (testkit → psm-research) and `crates/testkit/src/lib.rs` (re-exports `ConfirmAsShown`, `RESEARCH_INTEGRATION`).
- **Closing gate run in parts:** `cargo test --workspace` ran as the same tests in sequence, never in parallel. The parts were `--workspace --exclude psm-testkit`, `-p psm-testkit --lib --bins`, and the suite in two filters: 455 + 407 = 862 of 862. This kept each command inside the session's 10-minute command limit.
- **Tracker and handoff actor** is `opus` (this session ran on Claude Opus), not `codex`.

## Files changed
U1 (`c8b22ae`):
- `Cargo.lock`
- `apps/desktop/src-tauri/src/{commands/attributes.rs, lib.rs}`
- `apps/desktop/src/features/form/{AiPopover.tsx, FormDialog.tsx, __tests__/p11_form_ui.test.tsx, __tests__/p22_form_tidy.test.tsx}`
- `apps/desktop/src/services/{attributes.ts, bindings.ts}`
- `crates/core/src/mechanisms/{mod.rs, publish_partition.rs (new)}`
- `crates/core/src/ports/store.rs`
- `crates/core/src/services/{attributes.rs, history.rs}`
- `crates/infra/src/sqlite/store.rs`
- `crates/testkit/{Cargo.toml, src/lib.rs, src/test_app.rs, tests/suite/research/mod.rs, tests/suite/research/r5_feedback_and_undo.rs (new)}`

U2 (`1e56542`):
- `apps/desktop/src-tauri/src/{commands/items.rs, commands/publish.rs, lib.rs}`
- `apps/desktop/src/features/compose/__tests__/p32_reframe.test.tsx`
- `apps/desktop/src/features/publish/{PublishPopover.tsx, __tests__/p15_publish_ui.test.tsx}`
- `apps/desktop/src/services/{bindings.ts, publish.ts}`
- `crates/core/src/{domain/publish.rs, error.rs, mechanisms/publish_partition.rs, ports/store.rs, services/attributes.rs, services/history.rs, services/publish.rs}`
- `crates/infra/src/sqlite/store.rs`
- `crates/research/src/rules/write_table.rs`
- `crates/testkit/src/{lib.rs, test_app.rs}`
- `crates/testkit/tests/suite/research/{mod.rs, r5_publish_hold.rs (new)}`
- `crates/testkit/tests/suite/{p13,p14,p15,p16,p17,p21,p25,p30,p59,p61,p62,p64,p65,p66,p69}_*.rs` (the `confirm` call sites only)

Closing (this commit): `docs/implementation/plans/R5_research_app_seams.md` (Tracker line, Review log), this handoff.

Mutation-probe files: none (flow §4: no mutation regime). No architecture graph in this repo.

## Checkpoint SHA
U1 `c8b22ae` · U2 `1e56542` · closing (tracker, Review log, handoff) is the commit after `1e56542`.

## Stamp
Tree: `1e56542` (code), run once at the end, one command after another.

| Command | Result |
|---|---|
| `cargo test --workspace --exclude psm-testkit` | green (every package; research engine 122, research lib 120, core lib 82, …) |
| `cargo test -p psm-testkit --lib --bins` | green (0 tests) |
| `cargo test -p psm-testkit --test suite -- p0 p1 p2` | 455 passed, 0 failed |
| `cargo test -p psm-testkit --test suite -- p3 p4 p5 p6 research` | 407 passed, 0 failed (455 + 407 = 862, the whole suite) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cd apps/desktop && npm run typecheck` | clean |
| `npm test` | 465/466. The one failure is the known flake `p33_split_grid.test.tsx` C4(d) (backlog rows P35/P37/P41 → P33; file untouched by R5); the file alone: 7/7 green |

Per-unit commands (all green):
- U1:
  - `research::r5_feedback_and_undo::` 14, `p39` 24, `p40` 18, `p10` 31
  - `psm-desktop r5_` 1 (2 after U2)
  - `vitest src/features/form` 48, typecheck
- U2:
  - `research::r5_publish_hold::` 26, `p13` 36, `p14` 22, `p64` 8
  - `psm-research --lib rules::write_table` 3
  - `vitest src/features/publish src/features/compose` 220, typecheck
  - also `p03` 28 and the core `publish_partition` unit tests 3

## Manual try-out
On scratch data (`cd apps/desktop && PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`) with the test shop connected:
1. Open an item's form, edit a field, then Undo and Redo: it behaves as before.
2. Publish an item without suggestions: it publishes exactly as before.
3. The new rules (conflicts, the held item, release on the last resolution) are visible in `cargo test -p psm-testkit --test suite research::r5_publish_hold::` and `… research::r5_feedback_and_undo::`. The buttons for them arrive in R6, and research starts writing suggestions in R7.

I did not run the app by hand in this session; the try-out is unchanged from the plan.

## Notes for the coordinator
- The release writes no app event. A held session's row label changes on the next list read, so R6·U3 may want a `session_changed` / `publish_progress` event at release.
- `History::record` (photo, composition steps) now decides coalescing through the same `coalesces` predicate and `record_held`. Its behaviour is unchanged, and p39 and p40 are green.
