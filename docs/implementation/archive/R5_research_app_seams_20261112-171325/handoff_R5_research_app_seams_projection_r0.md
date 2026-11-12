---
plan: R5_research_app_seams
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-12
actor: opus (pipeline-projectionist)
---
# Projection of R5_research_app_seams (round 0, attempt 5: the plan amended from Q1–Q4)

The plan is close. All four fixes from the last round landed, and each of the 40 test rows can be written now
with one exact expected result. Four small points remain. Two are one-sentence plan edits: one rule in the plan
contradicts the contract it comes from, and one check could be skipped by accident. The other two are narrow
contract wordings the orchestrator can settle from earlier decisions: whether editing a cleared suggestion
counts as feedback, and what happens when a waiting item's field disappears from its form after you confirmed.
Nothing needs the owner. Once the orchestrator routes S1–S4, the implementer prompt can be compiled.

## ⚠ OWNER DECISIONS REQUIRED (0)

Nothing needs the owner. S1 and S2 are plan edits. S3 and S4 go to the orchestrator, each with a recommended
reading that follows 08B's own enumeration (S3) or owner card 1 as changelogs 6 and 8 completed it (S4). If the
orchestrator judges either reading a new choice, the card belongs in the milestone narrative (flow §6).

## Gate check

- Intention header **RATIFIED** (`docs/intention/intention.md:3`); 08B **CONTRACTS RATIFIED**, changelog 1–8.
- `depends_on: [R3_research_rules]`: Tracker **APPROVED** (`plans/R3_research_rules.md:10`).
- No upstream gate handoff for R5 sits in `OWNER_DECISIONS_PENDING`; attempts 1–4 are archived with 0 cards each
  (`archive/R5_research_app_seams_20261112-{163138,164046,165843,170701}/`). Gate passes.
- **Attempt-4 ledger Q1–Q4 all landed** (plan lines 44–45): Q1 → U2 Files (`commands/items.rs`) and the
  A09·C2(a) code clause; Q2 → U1 "Decided by the orchestrator" (last bullet); Q3, Q4 → U2 task 3 and row
  A09·C1(f); 08B MC-A10 "Only resolvable keys wait" and changelog 8 exist (`08B:521–523`, `:780–783`). Not re-opened.
- **Counts re-derived from the tables.** U1: C1(a–f), C2(a), C3(a–d), C4(a–c), C5(a–b) = 5 criteria, 16 rows.
  U2: C1(a–f), C2(a–b), C3(a–b), C4(a–i), C6(a–c), C7(a), C8(a) = 7 criteria, 24 rows. Total 12 / 40, matching
  plan line 24.

## Decision ledger

| # | Decision point | Class | Proposed routing |
|---|---|---|---|
| S1 | **Task 3's no-definition sentence contradicts 08B.** Plan line 192–193: "A row whose key has no active definition is never pending; it is **frozen or withheld by the other rules, as today**". 08B MC-A10 (`:521–523`, changelog 8): "A conflict whose key is not in the item's current layout, **or has no active definition**, is not pending: it is left out of the run (**not frozen**, not held, not counted as withheld)". The case is producible: `refresh_schema` → `replace_attribute_definitions` deletes definition rows only, never value rows (`crates/infra/src/sqlite/store.rs:826–841`), so a conflict row (`ai` + `suggested` + `ai_prior`) on a dropped key survives. Under the plan the unresolved suggestion lands in `frozen_attributes`; `write_set` never sends it (`mechanisms/attribute_write.rs:14–25`), but `run_staged` writes `frozen_attributes` into the staged `item.json` that becomes the local export (`services/publish.rs:1143`). Under the contract it is absent. Second gap in the same sentence: `is_conflict` needs a definition for `S` (`research/src/rules/write_table.rs:91–98`), and the plan doesn't say how a definition-less row is recognised as a conflict. | plan gap | Replace the sentence with: "An `ai` + `suggested` row with an `ai_prior` whose key has no active definition is left out of the run like an out-of-layout conflict (not frozen, not pending, not withheld); every other row with no definition is frozen or withheld as today, and `write_set` never sends it." (This sidesteps `S`: no definition means no type, so every prior-carrying suggestion is left out. That is the fail-closed reading of "unchecked replacement", MA17.) Optional row A09·C1(g): conflict on `k`, then `refresh_schema` with `FakeShop.definitions` minus `k`; confirm → `k` absent from `frozen_attributes` and `pending_conflicts`, step `prepare`; trace MC-A10 (changelog 8). Note for the implementer, no edit needed: built-in definitions don't matter to `partition`, because research never writes a Product-step field (`write_table.rs:15–29`). So `attribute_definitions("shopify")`, as `preview` and `update` use it today, is a sufficient definition set. |
| S2 | **The re-check can be skipped through a missing `shown` entry.** Task 4 forbids an `Option` that skips the re-check (fail-open on H1, R-IA22). The shape is delegated as `[{session_id, n_withheld, conflict_keys}]` (line 268–269), and nothing says what happens when a confirmed session has no entry. The natural code, `if let Some(s) = shown.iter().find(..) { compare }`, reopens the door task 4 closed. With the current UI no test would notice, because the popover always passes every item. | plan gap | One sentence in task 4: "A session in `session_ids` with no entry in `shown` is compared as `{n_withheld: 0, conflict_keys: ∅}`, never skipped." That reading refuses exactly when the item has something to show. Optional clause on A09·C2(a): "confirm with an empty `shown` for an item with `n_withheld` 1 → refused the same way". |
| S3 | **Does a human edit of an `ai` + `cleared` row write feedback?** U1 task 4 (line 88): feedback "when the row before the action had provenance `ai`". MC-A12's opening sentence and MA18 also say provenance `ai`, but MC-A12's table enumerates before-states `ai` + `suggested` / `ai` + `accepted` only (`08B:607–612`). The path is common: the photographer clears suggestion A, then types H. `set_value` writes over a non-live row with no live check for a human (`attributes.rs:417–470`; `write()` reads the cleared row as `stored`, `:340–343`). Branch 1 writes `edited` (suggested A, final H), so one suggestion gets two verdicts (`cleared` + `edited`) in the evaluation data. Branch 2 writes nothing. The cleared set and undo come out the same either way: `edited` doesn't feed `C`, and the token rules hold. No row decides it: A08·C1(f) covers `human` and `default` rows only. | intention gap (narrow; orchestrator) | **Recommended:** the enumerated table wins (precedent: changelog 4, "the enumerated rows win"). No feedback row when the before-row is `ai` + `cleared`, because the suggestion's verdict was already recorded by its clear. Edit task 4 to "provenance `ai` and status `suggested` or `accepted`". Add a clause to A08·C1(f): "an `ai` + `cleared` row edited → no feedback row". Add a wording line to 08B MC-A12's first sentence. **Fallback:** keep "provenance `ai`" and add the opposite clause to C1(f), so the choice is pinned either way. |
| S4 | **A pending key leaves the form after confirmation.** Changelog 8 decides "only resolvable keys wait" at confirmation (`partition`, task 3). After confirmation the item is `held`, and nothing locks the form. Three things can still remove pending key `k` from what the photographer can resolve: Customize fields (`presets_set`, global per item type, `attributes.rs:609`), an item-type change (including the automatic one in `read_integration_values` when the Shopify product type changed, `attributes.rs:676–688`), or a schema refresh (S1's path). The item then shows "Waiting for you · 1 conflicts" with no field to resolve. Cancel (P2) and publish again is the only way out, and at that point changelog 8 drops `k`. This is the dead end changelogs 6 and 8 closed, reopened one step later. The implementer cannot tell whether "current layout" should be re-evaluated later, in `presets_set`, `set_type` and `refresh_schema`. | intention gap (narrow; orchestrator) | **Recommended:** one sentence in 08B MC-A10 "Only resolvable keys wait": "Pending is decided at confirmation. A later layout or definition change neither releases the item nor drops a pending key; Cancel and publish again is the way out." Copy it into U2 task 3 so the implementer doesn't add re-evaluation hooks. No R5 code and no row. Add an R6·U3 note: the held label's activity popover keeps Cancel, which P2 already gives. **Alternative, not recommended:** re-evaluate on layout and definition changes. That would mean three more callers of the hold function, release-on-drop semantics, and new rows, at a cost out of scale with a rare path. |

## Reality checks

- **Paths.** Every edited path in both Files lists exists, and every new path is new except one. U2 Files lists
  `crates/core/src/mechanisms/publish_partition.rs` under "New" (line 181), but U1 creates it (line 74, task 0),
  so for U2 it is an edit. This is wording only; the unit order makes the meaning clear.
- **`confirm` perimeter recounted: 42 call sites in the 15 named files**, unchanged (p13 12, p14 2,
  p15_publish_backend 2, p16 1, p17 1, p21 2, p25 1, p30 1, p59 3, p61 2, p62 2, p64 5, p65 5, p66 1, p69 2).
  Outside the testkit the only Rust caller is `apps/desktop/src-tauri/src/commands/publish.rs`.
- **Hold-count sites exist as the carried note says.** `store.rs:1702–1704` filters `pending_conflicts` keys
  without a `resolved_attributes` entry, and `publish.rs:276–281` subtracts lengths. Both are named for the one
  helper on `PublishItem`.
- **`confirm` today** builds the preview outside the transaction, re-checks photos inside (`publish.rs:640–691`),
  and freezes stock and enqueues **after** the commit (`:722–728`). Task 4 (stock frozen for a held item, only
  the enqueue skipped) and task 5 (an in-transaction `StoreTx` enqueue at release) are both implementable against
  this shape.
- **Coalescing (Q2) is implementable.** `History::record` merges only when the redo list is empty and both labels
  start with `Edit ` on the same single key (`history.rs:153–176`, `coalesce_attribute :219–243`). The step list
  lives in memory (`HistoryStore`), so the service can read the last step inside the attribute transaction, as the
  Q2 decision requires. `Step` has no id today (`:35–38`). Where the token is generated is delegated (U1).
- **Fixtures named by the rows exist.** `FakeShop.products`, `values`, `stock`, `stock_requests` exist
  (`testkit/src/fake_commerce.rs:38–76`), as do `TestApp::tick` (`test_app.rs:518`) and `kill_and_restart`
  (`:475`). The desktop test precedent `r2_research_command_tests` is at `lib.rs:191–260`, and `psm-desktop` already
  depends on `psm-testkit` and `psm-research` (`src-tauri/Cargo.toml:18`, `:33`).
- **A09·C1(f) is producible.** Research writes only layout keys (`writes.rs:64–93` `layout_step`). A Details key
  with a preset default gets the prior through `fill_defaults`, then `writes::suggest`, then `presets_set`
  without the key (the Product-only check at `attributes.rs:609–620` does not block a Details layout).
- **Research never writes Product-step or built-in fields** (`write_table.rs:15–29`). This settles that
  `product.description`, which is sent from `frozen_attributes` by the describe step outside `write_set`
  (`publish.rs:1786–1797`), can never hold a conflict. I checked it because it would otherwise have been a silent
  MA17 path through S1's definition set.
- **Bookkeeping drift for the coordinator (no effect on the implementer).** `master_plan_research.md:75` still
  says R5 has "12 / 34"; the plan has 12 / 40. `research_row_mapping.md` lacks A09·C1(f) and C4(e)–(i).

## Criteria decidability (one exact outcome now? fixture producible?)

| Row | Decidable | Producible | Note |
|---|---|---|---|
| A08·C1(a)–(e) | yes | yes | |
| A08·C1(f) | yes | yes | S3 adds one clause (either direction) |
| A08·C2(a) | yes | yes | |
| A08·C3(a)–(d) | yes | yes | (c): `fill_defaults` records one step over both fields |
| A08·C4(a)–(c) | yes | yes | (a): the undo restores the prior, so `same_content` → `Unchanged` (verified in attempt 4; code unchanged) |
| A08·C5(a), (b) | yes | yes | |
| A09·C1(a)–(e) | yes | yes | |
| A09·C1(f) | yes | yes | see reality check; S1 may add (g) |
| A09·C2(a), (b) | yes | yes | S2 adds an optional clause |
| A09·C3(a), (b) | yes | yes | |
| A09·C4(a)–(i) | yes | yes | |
| A09·C6(a)–(c) | yes | yes | |
| A09·C7(a), C8(a) | yes | yes | |

Rule-17 scope: no row's observable is owned by an external dependency. Every fixture is our store, the research
crate's write door, `FakeCommerce` or the Tauri mock runtime, all read in the installed tree.

## Trace verification (both directions)

The trace cells are unchanged from attempt 4 except A09·C1(f) → MC-A10 (changelog 8), which resolves to
`08B:521–523` and says what the row asserts. The entries claimed for R5 (`master_plan_research.md` §5: MA8, MA17,
MA18) are each served: MA8 by A09·C1(b) and C8(a), MA17 by A09·C1(a), (c), C2, C3 and C4, MA18 by A08·C1, C3 and C4.
The optional rows proposed here trace to MC-A10 changelog 8 (S1) and MA17 / R-IA22 (S2). S3's clause belongs
under MA18.

## Write perimeter of this session

- Created: `docs/implementation/handoffs/reviewer/handoff_R5_research_app_seams_projection_r0.md` (this file).
- Nothing else: no change to the plan, intention, code, tracker, archgraph or prompts, and no commit. Commands
  run were read-only (`git log/status`, `grep`, `sed`, `cat`, `ls`, `wc`, `awk`).

## Appendix (non-authoritative): skeleton notes, discarded

These notes are not implementer guidance; they show where the ledger came from.
- U2 `partition(values, definitions, layout)`: for each live, valid row, look up the definition, then classify
  as withheld, pending or frozen. With no definition there is no `S`, and that is where S1 surfaced. The
  "layout" input is the same three-group union `form_get` and `layout_step` compute.
- `confirm(session_ids, summary, shown)`: for each session, look up its entry in `shown`. A `find` that returns
  `None` is where S2 surfaced.
- U1 `set_value` → `write()`: `stored` includes cleared rows, and task 4's "provenance `ai`" test fires on
  `ai` + `cleared`. That is where S3 surfaced.
- U2 task 5 hold function: it is called by the four attribute actions, `accept_all` and history apply. None of
  them runs when the layout changes, which is where S4 surfaced.
