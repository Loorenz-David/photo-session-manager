---
plan: R5_research_app_seams
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-12
actor: opus
---

# R5 research app seams — review r1

**Verdict: APPROVED_WITH_NOTES.** All 42 criterion rows (12 criteria, 2 units) hold at their boundaries. The production code follows 08B MC-A10 and MC-A12, the plan's orchestrator decisions (L13–L15, P6, Q2, T1–T3, N1–N9, S1–S4, F1), and the carried R2 note (one conflict count). No blocking findings (B1–B4). Three notes went to `docs/implementation/backlog_research/R5_research_app_seams.md`, where the plan's Review mode puts research notes; `backlog.md` got one pointer line.

## ⚠ OWNER DECISIONS REQUIRED (0)

None. Nothing needs the owner.

## Verified correct

**Gate.** The Tracker line read IMPLEMENTED. Checkpoints exist: U1 `c8b22ae`, U2 `1e56542`, closing `6d0a520`.

**Perimeter.** `git show --stat` of each checkpoint equals the handoff's declared lists exactly. U1 has 20 paths, including the declared extras `Cargo.lock` and `crates/testkit/src/lib.rs`. U2 has 36 paths, including the 15 testkit files whose `confirm` call sites changed (each a rename plus a `use` line). The closing commit holds only the plan's Tracker line and Review log, plus the implementer handoff. The working tree was clean at `82f5613` (`6d0a520` plus the review prompt).

**Checks** (Review mode, each once, one cargo command at a time, in the foreground):

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test suite research::r5_` | 40 passed (U1 14 + U2 26) |
| `cargo test -p psm-testkit --test suite -- p39_undo_history_core:: p40_undo_history_values:: p10_attributes:: p13_publish_runs:: p14_publish_update:: p64_publish_speed_and_visibility:: p03_session_state_list::` | 167 passed |
| `cargo test -p psm-desktop r5_` | 2 passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` | clean |
| `npx vitest run src/features/form src/features/publish src/features/compose` | 32 files, 268 passed |

The implementer's full-suite stamp stands (862/862, run in sequential parts to fit the session's command limit). The one `npm test` failure is the known `p33_split_grid` C4(d) flake; that file is untouched by R5 and passed when rerun alone.

**U1: feedback, conflict actions, guarded undo/redo (A08).**
- **One path for every action.** Every photographer action goes through `AttributesService::act`. It takes `History.operation`, then opens the store transaction, then records the step with `record_held` under the same guard (T1). The merge decision is made once, inside the transaction (Q2).
- **Feedback rows** (`note_action`):
  - written only when the row before was `ai` + `suggested`/`accepted` (S3);
  - action Accepted, KeptPrevious, Cleared or Edited, carrying the suggested value and confidence from before the action;
  - final value: null for Clear, the prior's value for Keep previous, the new canonical value for Edit;
  - `step_token` is the step's UUID.
- **Wrong-state actions** (`accept` on a non-suggestion, `keep_previous` on a non-conflict) are refused with `InvalidValue` before any write. Nothing is recorded: no row, no step, no feedback.
- **Clear and human edits:** `clear` and human `set_value` drop `ai_prior`. Non-human `set_value` writes no feedback.
- **Accept all** skips `low` suggestions and every `is_conflict`. When a key has no definition, a row with a prior is skipped (fail closed).
- **Research writes** (`writes::suggest`) never go through `act`, so they make no step and no feedback (C2(a)).
- **Guard.** Before anything applies, every `FieldState::Row` field is compared with `same_row` (every column except `updated_at`): undo compares with the after-row, redo with the before-row. On any difference the result is `StepCannotApply` with "that field was changed by research", and the existing MC-30 path drops the step. Title fields restore as before.
- **Feedback across undo/redo.** Undo reads the step's feedback rows, deletes them, and keeps them on the step. Redo re-inserts them with fresh ids (T2).
- **Coalescing.** A coalesced Edit updates its step's `edited` row's `final_value`. An edit never merges when it wrote feedback or recorded a resolution (N1).
- **Commands** `attribute_ai_accept` and `attribute_conflict_resolve` are thin: they check the action/value shape, then call the service.
- **UI.** Looks right calls `acceptSuggestion` and is shown only on `ai_status === 'suggested'`.

**U2: withheld, pending, held (A09).**
- **`partition`** matches MC-A10 line by line:
  - withheld = `ai` + `suggested` + `low` with no prior;
  - pending = a conflict that is in the layout and has a definition;
  - left out = a conflict outside the layout, or a prior-carrying suggestion without a definition;
  - frozen = every other live, valid row, including a prior that is not a conflict, at any confidence.
- **`confirm`:**
  - recomputes the partition inside the confirming transaction (`partition_in`) and compares it with `shown` per session (a missing entry counts as 0/∅, S2);
  - refuses with `SuggestionsChanged` before anything is written;
  - writes `frozen_attribute_origin` and `pending_conflicts`;
  - starts an item with pending keys at `Held` and skips only its enqueue; stock is still frozen (P3).
- **`resolve_pending_in`** is called unconditionally:
  - from `note_action`, which covers accept, keep_previous, human set_value, clear and accept_all;
  - from redo, for any changed field (T3).
  It writes `{value, action, at, step_token}`, with value null when the row after is not live and valid (N4). An existing entry is never changed. The last resolution calls `release_publish_item`: `prepare` plus `released_at` plus the job, in the same transaction (H4).
- **`unresolve_in`** removes only the undone token's entries, and only while the item is held.
- **`PublishItem::run_attributes()`** (frozen plus resolved, "no value" removed) is read by `ensure_export`'s `run_staged` and by `update`: `write_set`, the condition-key check, description and tags (N2).
- **Cancel and restart.** Cancelling a held item is one conditional UPDATE (`WHERE step='held'`). `activity` reports `can_cancel` true on `Held`. `active()` still excludes `Held`, so the startup re-queue leaves held items alone.
- **One count helper.** `unresolved_conflicts` is used by the session list (`store.rs`) and by the activity text (`publish.rs`), which closes the carried R2 note.
- **One conflict rule.** `psm_research::rules::write_table::is_conflict` delegates to core.

**Tests.** I read each of the 42 rows' tests against its production path. Each observes the row's exact outcome at the row's boundary: the stored row, the `research_feedback` rows, history state and tokens, the `publish_items` columns, job rows, `FakeShop` writes, products, stock and stock requests, and the staged/exported `item.json`. Each would fail on a wrong outcome. For example:
- C3(a)–(c): an unguarded undo would restore A over B and fail the row-equality assertion.
- C4(b): an id-preserving re-insert, or no re-insert, fails the token and count checks.
- C1(f): a partition that froze conflicts would put `k` in `frozen_attributes`.
- C4(c) Clear: an `out.remove` missing from `run_attributes` cannot pass the "kb in no write" check, because the key is never frozen and its entry's value is null.
- C6(c): a duplicate run would show a second product or media.

The core unit tests cover only the plan's named mechanisms (`partition`, `is_conflict`). There are no orphan tests and no tests coupled to the implementation. The UI test edits are the ones the plan names.

**Hard constraints.**
- H1: nothing is sent before a recorded confirmation. A held item's keys are sent only after recorded resolutions (MA17).
- H2, H3, H5: not touched.
- H4: release and its job commit together; restart rows C6(b) and C6(c) pass.
- H6: the resolution is a no-op without a held item; the form works without commerce.
- H7: the `"shopify"` definition read in `partition_in` is the existing pattern that `preview` and `update` already use, as plan task 3 prescribes.
- H8: core has no Tauri dependency, and the commands are thin.

**Assumptions** 1–13 were read against 08B MC-A10/MC-A12 and the plan's delegated and decided choices. None contradicts the book. The owner-relevant ones are in the narrative's "Choices made for you".

**Probe declaration.** No production or test file was edited (flow §4: no mutation regime). This review wrote only: the plan's Tracker line and Review log entry, `docs/implementation/backlog_research/R5_research_app_seams.md` (new), one pointer line in `docs/implementation/backlog.md`, `docs/implementation/narratives/R5_research_app_seams.md` (new), and this handoff. There were no database or state side effects outside test temp dirs.

## Findings

None blocking (B1–B4). Notes only (below).

## Notes written to backlog

In `docs/implementation/backlog_research/R5_research_app_seams.md`:
1. **Release emits no app event** (→ R6·U3). The held session's label changes only on the next list read.
2. **Undoing a resolution after release** brings the conflict back in the form, while the run keeps the resolved value. This conforms to the contract (MC-A10: "while the item is still held"). → R6·U3 / 08B, to say what the form shows.
3. **`p15_publish_ui.test.tsx` C5(a)** asserts a shown set for session 7 while confirming sessions 1 and 2. The cause is the reused mock preview; the fixture is incoherent, the assertion is correct for it.

## Carry-forward dispositions

| Note | Destination |
|---|---|
| 1 release event | R6·U3 |
| 2 undo after release, form display | R6·U3 (book 08B if a rule is wanted) |
| 3 p15 fixture | any later touch of `p15_publish_ui.test.tsx` |

## Lessons for the plans

- **Plan vs prompt backlog path.** The review prompt template says `docs/implementation/backlog.md`; research plans name `backlog_research/<plan>.md`. I followed the R2/R4 precedent: notes in the research file, one pointer line in `backlog.md`. The template could name the research path for R-plans.
- **Row A09·C4 is silent on undo after release.** Its rows cover undo and redo only while the item is held. The case is contract-defined, but it is not visible in any row. R6·U3's plan should carry it when it designs the conflict UI.
