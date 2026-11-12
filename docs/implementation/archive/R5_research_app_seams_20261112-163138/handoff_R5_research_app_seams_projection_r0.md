---
plan: R5_research_app_seams
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-12
actor: opus (pipeline-projectionist)
---
# Projection of R5_research_app_seams (round 0)

The plan can be built, and most of its 34 rows can be tested exactly as written. But a few things need fixing
before the implementer starts. Two linked undo rows (A08·C4(a) and C4(b)) cannot both pass with any setup the
code can produce. The rule that lets a waiting publish go ahead is assigned to an optional connection that a
correct-looking build could leave unconnected in the real app. The plan does not say what undo and redo do to a
resolution beyond the simplest case. And changing how a publish is confirmed reaches about twenty test files the
plan does not list. None of this needs the owner: every item is either a plan paragraph the orchestrator can
amend, or a short addition to the contract that follows from what the owner has already decided (ledger L4).
Once the ledger is routed, the implementer prompt can be compiled.

## ⚠ OWNER DECISIONS REQUIRED (0)

Nothing needs the owner. L4 is routed to the orchestrator as a contract wording addition. If the orchestrator
judges it a material semantic change, it becomes a card at that point. It is not one here.

## Gate check

- Intention status header: **RATIFIED** (`docs/intention/intention.md` line 3); 08B "CONTRACTS RATIFIED" (changelog 3).
- `depends_on: [R3_research_rules]`: the Tracker line reads **APPROVED** (`plans/R3_research_rules.md:10`). R4 is APPROVED too.
- No upstream gate handoff for R5 is pending owner decisions. Gate passes.

## Decision ledger

| # | Decision point (where the artifacts stop determining the next step) | Class | Proposed routing |
|---|---|---|---|
| L1 | **A08·C4(a)→C4(b) cannot both hold.** After "clear A; undo", the row is `ai`+`suggested` A again. A `writes::suggest` of A with the same content returns `WriteOutcome::Unchanged`, not "written", because `same_content` runs after the cleared check (`crates/research/src/writes.rs` `suggest_in`). With any different content (confidence, reason, sources) it *is* written, and that changes the row. C4(b) is then "C4(a), then redo", so the redo meets a row that differs from the step's before-row, and the A08·C3 guard drops the step. The feedback row is never re-inserted, and C4(b)'s expected outcome fails. | plan gap | Amend C4(a)'s outcome to: "`writes::suggest` of A with the original confidence, reason and sources returns `Unchanged` (not `MatchesCleared`); the row is untouched". C4(b) stays chained and then holds. Optional strengthening for C4(b)'s second clause: after the redo the stored row is itself `ai`+`cleared` A, so "A is refused as cleared" has two sufficient causes (stored cleared value and feedback row). The "feedback row exists again with the same token" clause carries the row. Say so, or drop the refusal clause. |
| L2 | **Release reachability (rule 10, silent failure).** The Delegated choices place the release in `PublishService`, "called from the attribute actions through a narrow trait". An injected trait has to be wired into every `AttributesService` construction: `commands/attributes.rs::with_service` and `attributes_definitions_list`, `TestApp::attributes`, and any other. A build whose tests construct the service with the hook while the command path does not would pass every A09·C4 row, and in the app a held item would never release. No row resolves through the command. The release must also run inside the action's transaction, so it needs a `&mut dyn StoreTx` anyway. | plan gap | Replace the delegated bullet with: "the hold step is a core function over the transaction (e.g. `services::publish::resolve_pending_in(tx, item_id, key, resolution, step_token, now)`), called **unconditionally** by `accept`, `keep_previous`, `set_value` (human) and `clear` inside their own transaction; no injected collaborator". Queueing needs a `StoreTx` enqueue method (today `enqueue_publish_item` exists only on `Store`, `ports/store.rs:294`). Files already include both store files. If an injected trait is kept anyway, add a row: `attribute_conflict_resolve` (command) on the last pending key of a held item → step `prepare`, one job. |
| L3 | **Which undo removes a resolution entry, and does a later action on a resolved key overwrite it?** Take a held item with k1 resolved by Keep previous (step S1), then a further human edit of k1 (step S2) while still held. MC-A10 says "from then on, later writes to that field are not in this run", so S2 must not change the entry. Undoing S2 must not remove k1's entry either: only undoing S1, the resolution, does. The plan says neither. An implementation keyed on "an undo touching a resolved key of a held item" would remove the entry on undoing S2. k1 would then be pending against a row that is no longer a conflict, and the session would show "Waiting for you · 1 conflict" with nothing in the form to resolve. | plan gap (the contract decides it) | Amend U2 task 5: "a resolution entry stores the resolving step's token; an action on a key that already has an entry changes nothing in `resolved_attributes`; undo removes only the entries carrying the undone step's token (as with feedback rows)". Optional row (A09·C4(e)): resolve k1, edit k1 again, undo once → the entry is unchanged and the item still waits for k2 only. |
| L4 | **Redo of a resolution while the item is still held.** MC-A12 says undo removes the record, and that redo re-inserts *feedback* rows. It says nothing about the *resolution entry* on redo. If redo does not re-record it, then after "Keep previous, undo, redo" the row is the kept default (no conflict, so no Keep previous/Use suggestion button in the form) while the key is pending. The item waits indefinitely, which is the same stuck state as L3. | intention gap | Route to the orchestrator: add one sentence to 08B MC-A10/MC-A12: "Redoing a resolution while the item is still `held` records it again (with the step's token) and releases the item when it was the last pending key". This is the same rule as the feedback rows, and it follows from the owner's "the job is stalled until resolved". Then add a U2 row (e.g. A09·C4(f): two pending keys; resolve k1, undo, redo → k1's entry present, still held; resolve k2 → released). |
| L5 | **A08·C5(a) has no runnable home.** Command tests live in `apps/desktop/src-tauri/src/lib.rs` `#[cfg(test)]` modules through the Tauri mock runtime (precedent: `r2_research_command_tests::f_c1a_…`, lib.rs:191–265). The testkit cannot call desktop commands. The plan names only the testkit file. The unit test commands have no `cargo test -p psm-desktop …`. Fixture: `keep_previous` needs a row with `ai_prior`. Core's `ai_suggest` writes `ai_prior: None`, and `master_plan_research.md` §3 Isolation lets only the testkit use `psm_research::writes`. | plan gap | Amend U1: C5(a)'s test is a `#[cfg(test)]` module in `apps/desktop/src-tauri/src/lib.rs` (already in Files). The conflict fixture comes from a testkit helper that calls `psm_research::writes::suggest` (testkit file to be named in Files, e.g. `crates/testkit/src/test_app.rs`). Add `cargo test -p psm-desktop r5_` (or the module name) to U1's Unit test commands and to the Review mode. Also note that bindings regenerate with `cargo test -p psm-desktop export_bindings` (master_plan §7). |
| L6 | **Unit order: U1 needs `is_conflict`, which U2 creates.** U1 task 3 (`accept_all` skips every conflict) and `keep_previous`/`use_suggestion` need the conflict rule in core. The plan creates `mechanisms/publish_partition.rs` (`is_conflict`) only in U2. The per-unit checkpoint commits "that unit's paths only". | plan gap | Move the creation of `psm_core::mechanisms::publish_partition::is_conflict` (+ the `mod.rs` line) into U1's Files and tasks. U2 adds `partition` and the research delegation. Alternatively, let U1 call `sameness::same` inline and U2 replace it. Either way, say so. |
| L7 | **Perimeter of the `confirm` signature and preview widening.** `PublishService::confirm(session_ids, summary, shown)` reaches 42 call sites in 15 testkit files: p13, p14, p15, p16, p17, p21, p25, p30, p59, p61, p62, p64, p65, p66, p69. `apps/desktop/src/features/publish/__tests__/p15_publish_ui.test.tsx:51,77` asserts `publishConfirm` `toHaveBeenCalledWith([7], summary)`, which fails with a third argument. Its preview literal (line 31) and `features/compose/__tests__/p32_reframe.test.tsx:130` lack the new `n_withheld`/`conflicts` fields, so `npm run typecheck` fails (tsconfig includes `src`). None of these files is in U2's Files. | plan gap | Add them to U2's Files ("mechanical: pass the shown set"), ideally through one testkit helper (e.g. `TestApp::confirm_as_shown(ids, summary)`, which takes the shown set from a fresh `preview`). **Do not** make `shown` an `Option` that skips the re-check: that is a fail-open seam on H1/R-IA22. |
| L8 | **A09·C1(d) fixture path.** Writing a value `S`-equal to a live default D directly returns `AlreadyHolds` (`write_table.rs` `decide_write`), so "ai+suggested low whose prior is the same value under S" cannot be made that way. It can be made by replacing research's own suggestion: suggest B over D (prior D), then suggest `d` (S-equal to D) at `low` (the prior is kept, `same` holds → not a conflict). | plan gap (producibility) | Add the construction path to C1(d)'s Input cell, so the row is not hand-built with `put_attribute_value`. |
| L9 | **Carried note "one conflict count" has no row and an unproducible fixture.** "Test that a resolved key that is no longer pending does not change n" needs `resolved ⊄ pending`, which no action produces: a resolution only writes pending keys. A test with a producible fixture (2 pending, 1 resolved) gives n = 1 under both today's `len − len` and the set difference, so it cannot tell them apart. As written, the test either traces to no row (orphan, charter rule 16) or uses a hand-inserted `PublishItem`. | plan gap | Keep the code instruction (one core helper "pending keys without a resolution", used by `store.rs` ~1704 and `publish.rs::activity` ~276). Drop the test clause, or turn it into a row whose fixture is explicitly a direct `insert_publish_item` and which traces to MC-A10 "n = derived". The orchestrator chooses. |
| L10 | **A09·C3(a) "no mutation for it".** `FakeCommerce` records `Mutation {op, run_id, at}` (`fake_commerce.rs:32`). Both items share one run, so a mutation cannot be attributed to an item by run id. | plan gap (decidability) | Name the observable: e.g. "the held session's item is a new product: no product with its SKU in `FakeShop.products`, no `values` entry for it; every recorded mutation belongs to the second item's product". |
| L11 | **A09·C3(b) "precedes".** Under `ManualClock` the confirmation and the mutations can share a millisecond. The p13 precedent asserts `confirmed_at <= m.at` (`p13_publish_runs.rs:410`). | plan gap (wording) | Write "`confirmed_at` ≤ every recorded mutation's `at`" (p13's form), or advance the clock between confirm and tick. |
| L12 | **A09·C7(a) "k is sent".** The update step writes only changed values (07A MC-13 / M20). If the shop already holds D, nothing is sent and "sent" fails. | plan gap (fixture) | State the fixture: the item is a new product (or the shop does not hold D for k). |
| L13 | **Actions on a row in the wrong state.** `accept` on a row that is not `ai`+`suggested`; `keep_previous` with no `ai_prior` or not a conflict; `attribute_conflict_resolve(edit)` without a value, or a value with another action. | free choice | Delegate explicitly: refuse with `Error::InvalidValue`, write nothing (no step, no feedback row). `value` is JSON text as in `attribute_values_set`; `action` is a snake_case enum `keep_previous \| use_suggestion \| edit`. |
| L14 | **The guard and `FieldState::Title`.** Task 5 says "for every field … the current row equals the expected row". Title fields have no row, and research never writes the title (MC-A1). | free choice | Delegate: the guard compares `FieldState::Row` fields. Title fields restore as today (p40 `c4d_title_undo_redo` unchanged). |
| L15 | **Coalesced typing and the `edited` feedback value.** The form sends `attribute_values_set` 300 ms after the last keystroke (`p11_form_ui.test.tsx:190`). Consecutive "Edit <label>" steps on one key coalesce into the first step (`history.rs` `coalesce_attribute`). The first edit of an `ai` row writes `edited` with that call's value, and the later calls (on a now-`human` row) write none. So `final_value` can be a typing prefix, which 08B MC-A11's `feedback.json` would carry. | free choice | Delegate explicitly, one of: (a) when an edit coalesces into a step whose token owns an `edited` feedback row, update that row's `final_value` in the same transaction; (b) record it as a known limitation. Recommend (a). No row needed. |

## Reality checks

- **Files exist / marked new:** every edited path exists. `mechanisms/publish_partition.rs`, `research/r5_feedback_and_undo.rs` and `research/r5_publish_hold.rs` are new and correctly marked. Missing from the perimeter: see L5 (testkit helper), L6 (U1 needs `publish_partition.rs` + `mod.rs`) and L7 (15 testkit files, two TS test files).
- **Cited sections resolve:** 08B MC-A1/A10/A12 say what the plan claims. The 05A pointers at lines 205 (MC-7) and 307 (MC-30) exist. `research_row_mapping.md` and `backlog_research/` exist. Trace IDs M5, M14, M17, M18, M20, MA8, MA17, MA18 and H1/H4 resolve in chapter 14 / chapter 01.
- **Line citations drifted by about 5 lines** (no impact, implementer can find them): `attributes.rs` `accept_all` 516 (plan 511), `clear` 551 (546). `publish.rs` `frozen_attributes` 478 (472), `preview` 524 (518), `confirm` 598–730 (592–720), `run_item` 768 (758), the attrs loop inside the confirm transaction ~644–652 (640–648). `history.rs` attribute arm 714–758 ✓. `cancel` 241 ✓.
- **Prior-phase outputs verified in code:** `Value.ai_prior` and the `AiPrior` shape. The feedback store methods `insert_feedback`, `delete_feedback_by_step`, `reinsert_feedback`, `feedback_by_key` (`ports/store.rs:106–113`). The four `publish_items` columns read and written (`infra store.rs:484–500, 766`). `PublishStep::Held` in `status.rs` and in `activity`. Startup re-queue (`active_publish_items`) already excludes `held` (`infra store.rs:2116`), so A09·C6(b) is mostly a regression guard. `psm_research::writes::{suggest, lower}` exist with the signatures the rows need (`integration` must be `Some(...)` and the key in the item's layout, non-Product step, writable). `psm-integrations` already depends on `psm-research` and the testkit on `psm-integrations`, so a direct testkit → `psm-research` dependency adds no new cycle (the dev-dep cycle of §7.4 already builds; R3/R4 approved).
- **Existing behaviour the guard must not break:** `p40` c4a/c4c/c5a/c5b undo rows over unchanged fields, so they keep passing under the guard. `fill_defaults` (P67) records one "Fill defaults" step with `default` after-rows, so A08·C3(c) is producible: a research write over a live `default` row is allowed (MC-A1).
- **Today's code the plan changes:** `cancel` refuses non-active steps (`publish.rs:243`), so A09·C6(a) needs a `Held` branch. `confirm` enqueues jobs after its transaction (`publish.rs` end of `confirm`), so held items are simply not enqueued there. Release enqueues inside the resolving transaction (L2).

## Criteria decidability (per row: can the test be written now with one exact outcome? is the fixture producible?)

| Row | Decidable now | Producible | Note |
|---|---|---|---|
| A08·C1(a) | yes | yes (default → `writes::suggest` gives the prior) | |
| A08·C1(b) | yes | yes | needs L6 (conflict rule in U1) |
| A08·C1(c) | yes | yes | the refusal has one cause (feedback; stored row is a live default): good |
| A08·C1(d) | yes | yes | |
| A08·C1(e) | yes | yes | |
| A08·C1(f) | yes | yes | |
| A08·C2(a) | yes | yes | "history state" = `HistoryService::state` |
| A08·C3(a) | yes | yes | `ai`+`cleared` A → B allowed (B ∉ C) |
| A08·C3(b) | yes | yes | |
| A08·C3(c) | yes | yes | |
| A08·C3(d) | yes | yes | |
| A08·C4(a) | **no** | — | L1 |
| A08·C4(b) | **no** (chained to C4(a)) | — | L1 |
| A08·C4(c) | yes | yes (26 imported text definitions) | |
| A08·C5(a) | **no** (no home, no command, no prior fixture) | — | L5 |
| A08·C5(b) | yes | yes | p11 C5(b) and p22 C4(a) assertions change (in Files) |
| A09·C1(a) | yes | yes | |
| A09·C1(b) | yes | yes | |
| A09·C1(c) | yes (JSON layout delegated) | yes | |
| A09·C1(d) | yes | yes, only by the path in L8 | L8 |
| A09·C1(e) | yes | yes | |
| A09·C2(a) | yes | yes | |
| A09·C2(b) | yes | yes | |
| A09·C3(a) | **partly** | yes | L10 |
| A09·C3(b) | yes with L11 wording | yes | L11 |
| A09·C4(a) | yes | yes | |
| A09·C4(b) | yes | yes (new suggestion on the kept-default k1 must not be in C) | |
| A09·C4(c) | yes | yes | two sub-cases, two items |
| A09·C4(d) | yes | yes | depends on L3 for which undo removes |
| A09·C6(a) | yes | yes | |
| A09·C6(b) | yes | yes | |
| A09·C6(c) | yes | yes | |
| A09·C7(a) | yes with L12 fixture | yes | L12 |
| A09·C8(a) | yes | yes | |

Rule-17 scope: no row's observable is owned by an external dependency. Every fixture is our own store, the research crate's write door or `FakeCommerce`. The producibility pass above covers our own write paths (L1, L8, L9).

## Trace verification (both directions)

- Every row's trace cell resolves. The rows trace to MA18 / MC-A12 (U1) and MA17 / MA8 / M17 / M20 / H4 / M18 / MC-A10 (U2). Each entry says what the row asserts. Two looser fits, no defect: A08·C1(b) (skipping conflicts in Accept all) traces to MA18 but serves MA17/MA8 more directly. A09·C3(b) cites M17 for an H1 ordering fact that M5 states.
- The ledger entries this milestone claims (`master_plan_research.md` §5: MA8 R5·U2, MA17 R5·U2, MA18 R5·U1) are each served by at least one row. The MA17 clause "no mutation precedes the resolution of its last pending key" is served by C3(a) + C4(a) + C4(b) together. MA18's "exactly for the actions currently applied" is served by C4(a)–(c). Once added, the L4 rows serve it for resolutions too.

## Write perimeter of this session

- Created: `docs/implementation/handoffs/reviewer/handoff_R5_research_app_seams_projection_r0.md` (this file).
- Nothing else: no plan, intention, code, tracker line or archgraph change; no commit.
- The only command run was a read-only `grep`. A broad `grep -r` over the repository was started by mistake, walked `target/`, and was stopped after its 120 s timeout. It wrote nothing.

## Appendix (non-authoritative): skeleton notes, discarded

Not guidance for the implementer: kept only so the coordinator can see what the ledger was derived from.
- U1: `Step` gains a token (every `Step` literal). `write()` gets the token before the transaction so the feedback insert sits inside it. The history attribute arm compares `current` against `after` (undo) or `before` (redo) with `updated_at` masked, for every `Row` field first, then applies. On success it calls `delete_feedback_by_step` / `reinsert_feedback`; for re-insert the step must carry the feedback rows it owns, or they must be soft-deleted.
- U2: `partition(values, definitions) -> {frozen, withheld, pending, origin}`. `confirm` recomputes `(n_withheld, conflict_keys)` inside the transaction per item and compares with `shown`. A held item is not enqueued. `resolve_pending_in(tx, …)` writes the entry and, on the last key, `set step prepare + released_at + enqueue` in the same transaction. `run_item` merges resolved values over the frozen ones, minus "no value".
