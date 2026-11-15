---
plan: R5_research_app_seams
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-12
actor: opus (pipeline-projectionist)
---
# Projection of R5_research_app_seams (round 0, attempt 4: the plan amended from P1–P6)

The plan is nearly ready. All six fixes from the last round are in place, and every one of the 39 test rows can be
written now with one exact expected result. Four small points are left. Two are one-line additions to the plan: a
file the new error message must touch, and a rule for a suggestion whose field no longer exists. One is a choice
the implementer should be given in writing. The last is a narrow case: an item that waits on a suggestion for a
field the form no longer shows, so you can't resolve it there. The orchestrator can settle it from what you already
decided about conflicts, so nothing needs you right now. Once the orchestrator applies the four rows below, the
implementer prompt can be compiled.

## ⚠ OWNER DECISIONS REQUIRED (0)

Nothing needs the owner. Q1–Q3 are plan edits or delegations. Q4 is routed to the orchestrator with a recommended
reading of owner card 1 (08B changelog 3). If the orchestrator decides that reading is a new choice, the card
belongs in the phase narrative (flow §6).

## Gate check

- Intention status header: **RATIFIED** (`docs/intention/intention.md:3`). 08B: **CONTRACTS RATIFIED**, changelog 1–7.
- `depends_on: [R3_research_rules]`: Tracker **APPROVED** (`plans/R3_research_rules.md:10`).
- No upstream gate handoff for R5 is in `OWNER_DECISIONS_PENDING`. Attempts 1–3 are archived with 0 cards each
  (`archive/R5_research_app_seams_20261112-{163138,164046,165843}/`). The gate passes.
- **Attempt-3 ledger P1–P6 all landed** (plan "Amendments" list, lines 42–43). P1 is in U1 Files (`AiPopover.tsx`),
  task 6 and A08·C5(b). P2 is in U2 task 6 and A09·C6(a). P3 is in U2 task 4 and A09·C4(b). P4 is in U2 Files
  (`domain/publish.rs`, `test_app.rs`). P5 is in U2 task 4 and A09·C2(a). P6 is under U1 "Decided by the
  orchestrator". I do not re-open them.
- **Counts were re-derived** from the criteria tables. U1 has C1(a–f), C2(a), C3(a–d), C4(a–c) and C5(a–b): 5
  criteria and 16 rows. U2 has C1(a–e), C2(a–b), C3(a–b), C4(a–i), C6(a–c), C7(a) and C8(a): 7 criteria and 23 rows.
  That totals 12 and 39 and matches the plan's table (line 24).

## Decision ledger

| # | Decision point (where the artifacts stop determining the next step) | Class | Proposed routing |
|---|---|---|---|
| Q1 | **The new refusal error needs a code at the command boundary, in a file outside the perimeter.** P5 names `Error::SuggestionsChanged` with command code `suggestions_changed`. The `Error` → `CommandError` mapping is an exhaustive `match` in `apps/desktop/src-tauri/src/commands/items.rs:39–` (`Error::InvalidValue(_) => "invalid_value"` at `:71`). It has no `_` arm, so the new variant must get a line there. That file is not in U2 Files (plan line 175). The build forces the edit, so the gap is not silent. But the code string is what R6·U3 keys on, and R6's UI tests mock that string. A misspelt code would leave both milestones green while the popover never rebuilds. | plan gap (perimeter) | Add `apps/desktop/src-tauri/src/commands/items.rs` (one match arm, code `suggestions_changed`) to U2 Files. Optional, at the orchestrator's choice: add a clause to A09·C2(a), checked in the desktop `r5_` test module, that `CommandError::from(Error::SuggestionsChanged).code == "suggestions_changed"`. This is the only place the cross-milestone string is pinned. |
| Q2 | **When coalescing is decided, compared with the transaction that must act on it.** L15 says a coalesced edit updates the earlier `edited` feedback row's `final_value` "in the same transaction". N1 says a step that recorded a resolution is never coalesced into. Today `write()` commits its transaction first (`attributes.rs:334–361`). Only afterwards does `record()` → `History::record` decide whether to merge (`history.rs:158–161`, `coalesce_attribute` `:219–243`). So inside the transaction the service does not yet know whether the edit will coalesce, or into which step's token. If the decision is made in two places (a guess inside the transaction and `coalesce_attribute` afterwards), the two can disagree. The feedback row would then be updated while the step stays separate, or the reverse. | free choice | Delegate in writing: "The coalescing decision is made once, inside the action's transaction: the service reads the session's last undo step through `History` (the predicate of `coalesce_attribute` plus N1), updates the feedback row when it merges, and hands that same decision to `History::record`, which does not re-decide. The mechanism, a lock or a returned flag, is the implementer's." No row is needed. |
| Q3 | **A conflict row whose key has no active definition.** `partition(values, definitions)` needs the definition to run `is_conflict`, because sameness is per-type (`write_table.rs:91–98`). A schema refresh (`refresh_schema` → `replace_attribute_definitions`, `attributes.rs:229–242`) can drop a definition after research wrote an `ai` + `suggested` row with a prior on it. Task 3 does not say what such a row is. If it is treated as pending, the item is held on a key that `write_set` would never send (it walks definitions only, `mechanisms/attribute_write.rs:14–25`). The form cannot show that key either (`form_get` lists definitions only, `attributes.rs:959–983`). The item would wait until cancelled, with "Waiting for you · 1 conflicts" and nothing to resolve. | plan gap | Add one sentence to U2 task 3: "A row whose key has no active definition is never pending; it is frozen or withheld by the other rules, as today, and `write_set` never sends it." No new row is needed, because the outcome is invisible to the shop. A one-line clause on A09·C1(c) is optional. |
| Q4 | **A conflict on a field that is not in the item's current layout.** Research writes only keys in the current layout (`writes.rs` `layout_step` → `never_written(step: None)`, `write_table.rs:15–16`). Afterwards the photographer can change the item type or edit the preset (Customize fields), and the row stays. MC-A10 defines pending by the row, not by the layout, so the item is held. The form shows layout fields only (`form_get`, `attributes.rs:967–983`), and R6 resolves held items "in the form" (R6 plan lines 31–34, U3). So the photographer sees "Waiting for you · 1 conflicts" and has nothing to click. The only way out is Cancel (P2), then Publish again and resolve the conflict in the confirmation popover. MC-A10's own "n to review" count is limited to the layout, and `write_set` does send out-of-layout keys that have a definition. So the book covers the review side of the layout question but not the hold side. | intention gap (narrow; the orchestrator can settle it) | Route to the orchestrator. **Recommended reading** (completes owner card 1, "stalled until resolved", which assumes resolution is possible; precedent: 08B changelog 6): "a conflict whose key is not in the item's current layout is not pending: it is left out of the run, not held and not counted as withheld. The shop keeps its value (07A MC-13); a new product gets none for that key." 08B MC-A10 gets a changelog line, and R5 task 3 gets the layout as an input to `partition`, plus one row such as A09·C1(f): "a conflict on a key outside the layout → absent from `frozen_attributes` and `pending_conflicts`; step `prepare`". **Fallback:** keep MC-A10 as written and record the dead end in `backlog_research/` for R7. R5 then changes nothing for this row. |

## Reality checks

- **Paths.** Every edited path in both Files lists exists, and every new path is marked new. The only omission is Q1.
  `crates/core/src/domain/publish.rs` (`PublishPreviewItem` at `:133`, `PublishItem` fields `:163–166`) and
  `crates/testkit/src/test_app.rs` are now in U2 Files, as P4 required.
- **The `confirm` perimeter was recounted:** 42 call sites in exactly the 15 named testkit files. That is p13 12,
  p14 2, p15_publish_backend 2, p16 1, p17 1, p21 2, p25 1, p30 1, p59 3, p61 2, p62 2, p64 5, p65 5, p66 1 and p69 2.
  p69 calls `service(a).confirm(sessions, …)` at `:217` and `:424`. `service(a)` is `a.app.publish` (`:141–143`),
  the same service `confirm_as_shown` will use. Outside the testkit, the only callers are `commands/publish.rs:124`
  and the TypeScript `publishConfirm` (`services/publish.ts:12`, `PublishPopover.tsx:27`). Frontend preview
  literals (`details_total`) appear only in `p15_publish_ui.test.tsx` and `p32_reframe.test.tsx`, both in Files.
- **P1 is implementable as written.** `AiPopover` receives the stored row (`MetafieldInput.tsx:159`
  `value={stored}`), and the TS `AttributeValue` carries `ai_status` (`bindings.ts:269`). The tests that click
  Looks right are exactly `p11` C5(b) (`:300–316`) and `p22` C4(a) (`:35–57`), both in U1 Files. `p22` C4(a)
  mocks `setAttributeValue` for both clicks and must switch to the accept service function.
- **P2 is implementable.** `can_cancel: running && …` (`publish.rs:331`). `cancel` refuses non-active steps
  (`:243`), and task 6 changes both. `run_item` returns early for a non-active step (`:770`), so if a cancel races
  a release, the stale job finds `Cancelled` and does nothing. Nothing is sent.
- **P3 is implementable.** The stock freeze and the enqueue are separate statements in one loop
  (`publish.rs:724–729`).
- **The startup re-queue cannot touch a held item.** `requeue_running_jobs` re-queues job rows only
  (`infra store.rs:1626`). The resume scan selects `step IN ('prepare','upload','update','describe')`
  (`infra store.rs:2116`). A held item has no job row and `held` is not in that list.
- **No other writer replaces a pending conflict without resolving it.** `apply_remote` skips live `ai` rows
  (`attributes.rs:726–729`). `fill_defaults` skips live rows (`:896`). `set_value` is called in production only
  with `Provenance::Human` (`commands/attributes.rs:294`). Research's `suggest` may replace it, which MC-A10
  allows (the key stays pending, or becomes a non-conflict, as A09·C4(h) covers).
- **Research write fixtures behave as the rows expect.** A08·C4(a): after the undo, the row is `ai` + `suggested`
  A with its prior, so `decide_write` → `Write{prior}` → `same_content` → `Unchanged` (`writes.rs:208–210`;
  `write_table.rs:66`). A08·C1(c): Keep previous leaves a `default` row and a `kept_previous` feedback row, so
  `cleared_set` holds A → `MatchesCleared` (`writes.rs:100–123`). A08·C3(a): writing B over `ai` + `cleared` A
  succeeds (`write_table.rs:67`; B ∉ cleared set). A09·C1(d): stored `ai` + `suggested` B keeps its prior D
  (`write_table.rs:66`), and d is S-equal to D, so the row is not a conflict.
- **The command test for A08·C5(a) is decidable.** `history_get` is callable through the mock runtime and returns the
  undo label (precedent `lib.rs:487–490`, `c7b_attribute_command_undo_restores_absence` `:492–527`). "Exactly one
  undo step" means one label, and after one `history_undo` there is none.
- **Dependency cycle.** `psm-research` dev-depends on `psm-testkit` (`crates/research/Cargo.toml:21`). U1 makes
  `psm-testkit` depend on `psm-research`. Cargo allows a dev-dependency cycle. `psm-desktop` already depends on
  both (`src-tauri/Cargo.toml:18`, `:33`). Attempt 3 checked the duplicate-crate risk for `--lib` unit tests. R5
  adds no research unit test that imports the testkit.
- **Bookkeeping drift for the coordinator, unchanged since attempt 3 (no effect on the implementer).**
  `master_plan_research.md:75` still says R5 has "12 / 34" rows; the plan has 12 / 39.
  `research_row_mapping.md:250–253` lists A09·C4(a)–(d) only.

## Criteria decidability (can the test be written now with one exact outcome? is the fixture producible?)

| Row | Decidable | Producible | Note |
|---|---|---|---|
| A08·C1(a)–(f) | yes | yes | |
| A08·C2(a) | yes | yes | |
| A08·C3(a)–(d) | yes | yes | (c): `fill_defaults` records one step over both fields (`attributes.rs:876–926`) |
| A08·C4(a)–(c) | yes | yes | see the fixture checks above |
| A08·C5(a) | yes | yes | `history_get` gives the step count at the command boundary |
| A08·C5(b) | yes | yes | |
| A09·C1(a)–(e) | yes | yes | Q4 would add (f) if the recommended reading is taken |
| A09·C2(a), (b) | yes | yes | Q1 adds an optional clause for the code string |
| A09·C3(a), (b) | yes | yes | |
| A09·C4(a)–(i) | yes | yes | (g) depends on N1 and Q2: the first step recorded a resolution, so it is never coalesced into |
| A09·C6(a)–(c) | yes | yes | |
| A09·C7(a), C8(a) | yes | yes | |

Rule-17 scope: no row's observable belongs to an external dependency. Every fixture is our store, the research
crate's write door, `FakeCommerce` or the Tauri mock runtime, and all of them were read in the installed tree.

## Trace verification (both directions)

The trace cells are unchanged since attempt 3, which verified them against `14_measurement_ledger.md` (M5, M17, M18,
M20, MA8, MA17, MA18) and 08B MC-A10/MC-A12. The entries claimed for R5 in `master_plan_research.md` §5 are MA8,
MA17 and MA18. Each is served by at least one row: MA8 by A09·C1(b) and C8(a), MA17 by A09·C1(a), C1(c), C2, C3
and C4, and MA18 by A08·C1, C3 and C4. If the optional rows from Q1, Q4 and Q3 are added, they trace to MA17 (Q1,
Q4) and to MC-A10 (Q3).

## Write perimeter of this session

- Created: `docs/implementation/handoffs/reviewer/handoff_R5_research_app_seams_projection_r0.md` (this file).
- Nothing else: no change to the plan, intention, code, tracker, archgraph or prompts, and no commit. The commands
  I ran only read (`git log/status`, `grep`, `sed`, `awk`, `cat`, `ls`, `wc`).

## Appendix (non-authoritative): skeleton notes, discarded

These notes are not guidance for the implementer. They are kept only so the coordinator can see where the ledger
came from.
- U1 edit path: `set_value` → `write()`, which runs one transaction (row and activity) → `record()` →
  `History::record`, which may coalesce. The feedback insert and L15's `final_value` update must sit inside
  `write()`'s transaction, but the merge decision sits after it. That is the source of Q2.
- U2 `partition`: for each live, valid row, look up its definition, then classify it as withheld, pending
  (`is_conflict`) or frozen. Q3 is the lookup that fails. Q4 is the layout that the classification never consults.
