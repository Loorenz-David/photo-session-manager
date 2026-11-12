---
plan: R5_research_app_seams
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-12
actor: opus (pipeline-projectionist)
---
# Projection of R5_research_app_seams (round 0, attempt 3: the plan amended from N1–N9)

The plan is close. Every fix from the last two projections is in place, and 38 of the 39 test rows can be written
exactly as they stand. Three gaps are left that a user would notice. After this change, pressing "Looks right"
on a suggestion you already accepted shows an error. A publish that waits for you cannot be cancelled from the
screen, because the progress view never offers Cancel for it. And a new product that waited could be created
without the stock you chose. None of this needs the owner: each one is a sentence or two in the plan, settled by
contracts the owner already ratified. The implementer prompt can be compiled once the orchestrator applies the
ledger below.

## ⚠ OWNER DECISIONS REQUIRED (0)

Nothing needs the owner. Each ledger row follows from 08B MC-A10/MC-A12 as ratified, or is a delegation the
orchestrator can grant.

## Gate check

- Intention status header: **RATIFIED** (`docs/intention/intention.md:3`). 08B carries MC-A10 and MC-A12 with
  changelog 6, which covers redo of a resolution.
- `depends_on: [R3_research_rules]`: Tracker **APPROVED** (`plans/R3_research_rules.md:10`).
- No upstream gate handoff for R5 is in `OWNER_DECISIONS_PENDING`. Attempts 1 and 2 were archived with 0 cards each
  (`archive/R5_research_app_seams_20261112-163138/`, `…-164046/`). The gate passes.
- **Attempt-2 ledger N1–N9 all landed:** N1 is in U2 task 5 ("never coalesced into") and row A09·C4(g). N2 is in
  task 6 (`run_staged` and `write_set`) and the export clause of A09·C4(b). N3 is task 5 (`accept_all`) and
  A09·C4(h). N4 is task 5 ("no value") and A09·C4(i). N5 is A09·C3(a), now observable through `FakeShop.products`
  and `FakeShop.values`. N6 is the U1 Files helper note. N7 is task 5 (the step keeps its entries). N8 and N9 are
  under "Decided by the orchestrator". I do not re-open them. Counts are derived: U1 has 5 criteria and 16 rows, U2
  has 7 and 23, the total is 12 and 39, and this matches the plan's table.

## Decision ledger

| # | Decision point (where the artifacts stop determining the next step) | Class | Proposed routing |
|---|---|---|---|
| P1 | **"Looks right" on an accepted suggestion becomes an error.** The AI pill and its popover, including **Looks right**, render for *every* `provenance === 'ai'` row (`MetafieldInput.tsx:149` `showAi`, `:159`; `AiPopover.tsx:58`). After U1, Looks right calls `attribute_ai_accept`, and the orchestrator's L13 decision refuses `accept` on any row that is not `ai` + `suggested` with `Error::InvalidValue`. The accepted row stays `ai`, so the pill stays after the first click. A second click, or a click on any row accepted through Accept all, now shows the toast "Could not accept: invalid_value: …" (`FormDialog.tsx:314`). Today the same click succeeds, because it writes a human value. R6·U3 task 5 replaces Looks right only on *conflict* rows, so the regression ships with R5 and stays after R6. MC-A12 lists Looks right only from `ai` + `suggested`. | plan gap | Amend U1 task 6 and Files: the popover offers **Looks right** only when `ai_status === 'suggested'` (one condition in `AiPopover.tsx` or `MetafieldInput.tsx`; add the file to U1 Files). Add a clause to A08·C5(b): "on an `ai` + `accepted` row the popover offers no Looks right; Clear stays". Fold a note into R6·U3, which edits `AiPopover.tsx` too. The alternative, if the orchestrator prefers the backend, is to amend L13 so that `accept` on `ai` + `accepted` returns Ok and writes nothing (no step, no feedback). |
| P2 | **A held item cannot be cancelled from the UI.** `PublishService::activity` sets `running = active(item.step)` (`publish.rs:262`), and `active()` excludes `Held` (`:111–116`). So `can_cancel` is `false` for a held item (`:331`), and `PublishProgress.tsx:49` shows Cancel only when `can_cancel`. MC-A10 says "On a `held` item, cancel sets `cancelled` at once". Row A09·C6(a) tests only the service call, and R6·U3 adds no cancel affordance. The contract's cancel would be unreachable for the photographer, and the item would wait for ever unless every conflict is resolved. | plan gap | Amend U2 task 6: "`activity` of a `held` item reports `can_cancel: true` (and `retry` none)". Add to A09·C6(a)'s input or outcome: "before the cancel, `activity` reports step `held` and `can_cancel` true". This is backend only and stays in U2's perimeter (`services/publish.rs`). |
| P3 | **The stock of a held new product.** In `confirm`, after the transaction, one loop both freezes the stock choice and queues the job: `set_publish_item_stock` then `enqueue_publish_item` (`publish.rs:724–729`). Task 4 says only "**no job is queued for it**". The obvious edit, `if held { continue }`, also skips the stock freeze. The released item is then created with the default or no stock instead of the chosen one. No row observes stock on a held item, so this fails silently on the shop's inventory. | plan gap | Amend U2 task 4: "for a held item only the enqueue is skipped; its stock is frozen at confirmation like any other item's (07A MC-10, changelog 45)". Optional clause on A09·C3(a) or C4(b), with a new product whose form chose a stock: the released product is created with that location and quantity (observable as `FakeShop.stock[product id]` and `stock_requests`, `fake_commerce.rs:72–75`). |
| P4 | **U2's perimeter omits two files it must edit.** (1) `crates/core/src/domain/publish.rs`: `PublishPreviewItem` (`:133`) gains `n_withheld` and `conflicts`. That file also holds `PublishItem`, the natural home of the carried note's single "pending keys without a resolution" helper, which `infra store.rs:1704` and `publish.rs:276` both need. (2) `crates/testkit/src/test_app.rs`: `TestApp::confirm_as_shown` is named in U2 Files but its file is listed only in U1's. | plan gap (perimeter) | Add both paths to U2 Files. |
| P5 | **What the refusal looks like at the command boundary.** The plan fixes the sentence `AI suggestions changed while you were confirming` but not the `Error` variant. As `Error::InvalidValue` it reaches the UI as `{code: "invalid_value", message: "invalid_value: AI suggestions changed…"}` (`error.rs:35`; `commands/items.rs:38+`). R6·U3 task 3 and A20·C3(c) must recognise this refusal, and only this one, to rebuild the preview. With a generic code that means matching on message text. | free choice (crosses into R6) | Decide in writing: a dedicated variant (e.g. `Error::SuggestionsChanged`, `#[error("AI suggestions changed while you were confirming")]`, command code `suggestions_changed`). A09·C2(a)/(b) then assert the variant, and R6·U3 keys on the code. Or state "`InvalidValue` with exactly that text; R6 matches the message". Either is fine if written down. |
| P6 | **What redo re-inserts.** Task 5 and MC-A12 say "redoing re-inserts them" (the feedback rows). Under the L15 decision, a coalesced edit updates the feedback row's `final_value` after the step was recorded. If the step kept a copy of its rows at action time, a redo restores the stale typing prefix. | free choice | Delegate in writing: "undo reads the rows carrying the token before deleting them and keeps them on the step; redo re-inserts exactly those". No row is needed. |

## Reality checks

- **Paths.** Every edited path in both Files lists exists: `commands/attributes.rs`, `lib.rs`, `FormDialog.tsx`,
  `p11_form_ui.test.tsx`, `p22_form_tidy.test.tsx`, `test_app.rs`, `mechanisms/mod.rs`, `fake_commerce.rs`,
  `p15_publish_ui.test.tsx`, `p32_reframe.test.tsx`, `research/mod.rs`. The new files are marked new. The two
  omissions are P4.
- **The `confirm` perimeter was recounted.** There are 42 call sites in exactly the 15 listed testkit files: p13 has
  12, p14 2, p15_publish_backend 2, p16 1, p17 1, p21 2, p25 1, p30 1, p59 3, p61 2, p62 2, p64 5, p65 5, p66 1 and
  p69 2. There is no caller in `crates/research` or `src-tauri` besides `commands/publish.rs:124`. `PublishPreviewItem`
  is constructed in Rust only at `publish.rs:578`.
- **Note for the helper (no ledger row).** `p62_retention_after_publish.rs:710–716` expects `confirm` on a cleared
  session to fail with `Error::FilesCleared`. `confirm_as_shown` runs `preview` first. Today `preview` does not check
  clearing and should succeed, so the expected error is unchanged. If it ever errors, the helper must surface
  `confirm`'s error and not its own.
- **Run-path reads of the frozen list.** `item.frozen_attributes` is read at `publish.rs:1143` (`run_staged`),
  `:1638` (`write_set`), `:1641` (condition key), `:1788` (description) and `:1799` (tags). Task 6 names only the
  first two. The other three are safe: research never writes Product-group fields or media lists
  (`write_table.rs:15–29` `never_written`), so description, tags and the condition key are never pending. The early
  export (`start_next_in_batch` → `start_export_ahead` → `ensure_export`) goes through the same `run_staged` call
  and filters `step == Prepare` on active items, so a held item never gets an early staged export with the
  un-merged values.
- **The testkit depends on `psm-research` (U1).** `psm-research` already dev-depends on `psm-testkit`
  (`crates/research/Cargo.toml`), so this adds a dev-dependency cycle, which cargo allows. Its own `--lib` unit
  tests do not use `psm_testkit` (none in `crates/research/src`), so the duplicate-crate type mismatch cannot occur.
  `tests/engine` links the same library as the testkit.
- **One session per item.** `SessionsService::create` returns the existing session of an item
  (`sessions.rs:177–180`). "The held item of that product" in task 5 therefore names at most one session's publish
  item. A second confirmation of a held session is not offered (MC-A10 status rule).
- **The worker is a poll loop** (`integrations/src/lib.rs`, sleeps of 20–250 ms). A job queued in the attribute
  command's transaction is picked up without any extra wake-up, so the release needs no wiring beyond the
  `StoreTx` enqueue that task 5 already names.
- **Retention.** Retention clears a session only when its latest step is `Done` (`mechanisms/retention.rs:48`), so a
  waiting item's files are safe. Its masters are not in `masters_needed` (`storage.rs:1163–1174`, active items
  only), but masters are rebuildable and are re-rendered at prepare. That costs time, not correctness. No row.
- **Bookkeeping drift (coordinator, no effect on the implementer).** `master_plan_research.md:75` still says R5
  has "12 / 34" rows, but the plan now has 12 / 39. `research_row_mapping.md:250–253` lists A09·C4(a)–(d) only, not
  C4(e)–(i).

## Criteria decidability (can the test be written now with one exact outcome? is the fixture producible?)

| Row | Decidable | Producible | Note |
|---|---|---|---|
| A08·C1(a)–(f) | yes | yes | `research_feedback` store methods exist (`infra store.rs:901–966`); `Feedback.confidence` is non-optional, and every `ai` row the app writes has one |
| A08·C2(a) | yes | yes | |
| A08·C3(a)–(d) | yes | yes | reason text: `StepCannotApply.reason` ("Can't undo <label>: <reason>", `error.rs:47`) |
| A08·C4(a)–(c) | yes | yes | (c) needs 26 writable definitions, which can be imported in the test |
| A08·C5(a) | yes | yes | precedent `#[cfg(test)]` modules with `tauri::test::mock_builder` exist (`src-tauri/src/lib.rs:20`, `:191`) |
| A08·C5(b) | yes | yes | add P1's clause; `p22` C4(a) currently mocks `setAttributeValue` for both Looks right clicks and must switch to the accept service function |
| A09·C1(a)–(e) | yes | yes | (d) path verified in attempt 2 (`write_table.rs` keeps the prior when research replaces its own suggestion) |
| A09·C2(a), (b) | yes | yes | P5 decides which variant they assert |
| A09·C3(a) | yes | yes | both items must be new products for "exactly one product" (a conflict on a new product has a preset prior) |
| A09·C3(b) | yes | yes | |
| A09·C4(a)–(i) | yes | yes | (i): `apply_remote` stores a non-fitting shop value verbatim with `valid = false`, provenance `integration` (`attributes.rs:744–747`) |
| A09·C6(a) | yes | yes | add P2's clause |
| A09·C6(b), (c) | yes | yes | |
| A09·C7(a), C8(a) | yes | yes | |

Rule-17 scope: no row's observable belongs to an external dependency. Every fixture is our store, the research
crate's write door, or `FakeCommerce`, all read in the installed tree.

## Trace verification (both directions)

- Every trace cell resolves in `14_measurement_ledger.md`: M5 (line 9), M17 (29), M18 (30), M20 (32), MA8 (71),
  MA17 (87) and MA18 (88). MC-A10 and MC-A12 are in 08B. Each says what its rows assert. A09·C1(e) traces to M17
  ("runs send only frozen content"), which is adequate for "cleared and invalid rows are not frozen".
- The entries claimed for R5 (`master_plan_research.md` §5) are MA8 (C1(b), C8(a)), MA17 (C1(a), C1(c), C2, C3,
  C4) and MA18 (A08·C1, C3, C4). Each is served by at least one row.

## Write perimeter of this session

- Created: `docs/implementation/handoffs/reviewer/handoff_R5_research_app_seams_projection_r0.md` (this file).
- Nothing else: no change to the plan, intention, code, tracker, archgraph or prompts, and no commit. The commands I
  ran only read (`git log/show/status`, `grep`, `sed`, `ls`, `cat`, `wc`). A stray background `grep` was stopped.

## Appendix (non-authoritative): skeleton notes, discarded

These notes are not guidance for the implementer. They are kept only so the coordinator can see where the ledger
came from.
- U1 `accept` path: an `attribute_ai_accept` command calls `accept(item, key)`, which opens a transaction, checks
  the row is `ai` + `suggested`, writes it as `accepted` with the prior set to none, inserts a feedback row, and (in
  U2) calls `resolve_pending_in`. After that it calls `record(step{token})`. The form's AI popover renders for
  `ai` rows of any status, which is where P1 came from.
- U2 `confirm`: after the transaction, a loop over `(id, session)` sets the stock and then, if the item is not
  held, enqueues it (P3). For a held item, `activity` gives `running = false`, so `can_cancel = false` (P2).
