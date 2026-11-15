---
plan: R5_research_app_seams
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-12
actor: opus (pipeline-projectionist)
---
# Projection of R5_research_app_seams (round 0, attempt 2: the amended plan)

The amended plan holds up: the fifteen fixes from the first projection landed, and 34 of its 36 rows can now be tested
exactly as written. A few new problems turned up, and they all sit on the publish side. The worst: typing a
correction into a waiting item's conflicting field would publish whatever was typed at the first pause. Also, a
released item's local export would leave out the values the photographer resolved. One row asks for something the
fake shop cannot show. None of this needs the owner. Each item is a plan paragraph the orchestrator can amend from
what the owner has already decided. Once the ledger is routed, the implementer prompt can be compiled.

## ⚠ OWNER DECISIONS REQUIRED (0)

Nothing needs the owner. N1 and N4 settle what counts as a resolution. Both follow from the owner's answer to card 1
("the job is stalled until resolved"): what is sent is what the photographer resolved. They are routed to the
orchestrator as plan amendments.

## Gate check

- Intention status header: **RATIFIED** (`docs/intention/intention.md:3`); 08B "CONTRACTS RATIFIED", changelog 6
  (redo of a resolution) present (`08B_research_contracts.md:761`).
- `depends_on: [R3_research_rules]`: Tracker **APPROVED** (`plans/R3_research_rules.md:10`); R4 APPROVED too.
- No upstream gate handoff for R5 is in `OWNER_DECISIONS_PENDING` (attempt 1's projection had 0 cards and is archived
  under `archive/R5_research_app_seams_20261112-163138/`). Gate passes.
- Attempt-1 ledger L1–L15: each amendment is present in the plan where its "Amendments after projection r0" list
  says. They are not re-litigated here.

## Decision ledger

| # | Decision point (where the artifacts stop determining the next step) | Class | Proposed routing |
|---|---|---|---|
| N1 | **Typing into a pending field resolves at the first pause.** The form saves through `attribute_values_set` 300 ms after the last keystroke. Consecutive `Edit <label>` steps on one key coalesce into the first step (`history.rs:219–243`, `coalesce_attribute`, same item, one field, same key). U2 task 5 says `set_value` (human) calls the hold step unconditionally, and that "an action on a key that already has an entry changes nothing in `resolved_attributes`". So on a held item, the first debounced call records the entry with a typing prefix ("Wa"). When this was the last pending key, that same transaction releases the item and queues its job. Later calls ("Walnut") coalesce into the same step but leave the entry alone, so the shop gets "Wa". L15's decision updates the *feedback* row's `final_value` on coalesce, so after it the feedback says "Walnut" while the entry and the shop say "Wa". R6·U3 row A20·C5(a) ("Edit puts the field in edit mode and the typed value is saved as a human edit") would ship exactly this path. No R5 row exercises it. | plan gap (+ fold-forward to R6·U3) | Amend U2 task 5: "a step that recorded a resolution is never coalesced into. A later edit of the key is a new step, and changes nothing in the entry (MC-A10)". This keeps the history, the feedback row and the entry in agreement, and makes L15 apply only to non-resolving steps. Fold into R6·U3: a conflict field's Edit saves once, on explicit commit (Enter, blur or Save), never on the 300 ms debounce. Optional row (A09·C4(g)): a held item with one pending key; `set_value` "Wa", then `set_value` "Walnut". Outcome: two undo steps; the entry is "Wa", which pins the rule. The R6 commit rule is what keeps "Wa" from being the real value. Alternative if the orchestrator prefers: coalesced edits update the entry while the item is still `held`. That does not help the one-conflict item, which is released at the first call. |
| N2 | **Where "sends" applies on the run path.** U2 task 6 says `run_item` sends "the frozen values plus each resolved key's value". Two run-path reads use `item.frozen_attributes`: `ensure_export` → `export.run_staged(…, &item.frozen_attributes)` (`publish.rs:1143`), whose staged `item.json` (`export.rs:960`) becomes the local export at `publish_swap` when the item reaches done (`publish.rs:836–838`); and `update` → `write_set(&item.frozen_attributes, …)` (`publish.rs:1638`). An implementer who merges only in `update` passes every A09·C4 row (they observe the shop). The released item's local export then silently lacks every resolved key. | plan gap | Amend task 6: "the values a released item uses are `frozen_attributes` plus each resolved key's value, with 'no value' keys left out. They are used by every run-path read of the frozen list: `ensure_export`'s `run_staged` (the staged `item.json`) and `update`'s `write_set`. Merging at release, or at read time, is the implementer's choice." Optional clause on A09·C4(b): the item's export `item.json` holds `k1` = the prior's value and `k2` = the suggestion. |
| N3 | **Accept all on a pending key that is no longer a conflict.** A pending key stops being a conflict when research replaces its own suggestion with a value `S`-equal to the prior. This is the same path as A09·C1(d): the prior is kept and `is_conflict` is false. Accept all accepts it (U1 task 3 skips only `low` and conflicts). Accept all is an MC-A12 action with `accepted` feedback, but U2 task 5 lists only `accept`, `keep_previous`, `set_value` and `clear` as callers of the hold step. The key stays pending against an accepted row. The session shows "Waiting for you · 1 conflict", and the form shows no conflict to resolve. | plan gap | Amend task 5: "`accept_all` calls the hold step for each key it accepts, in its transaction". The contract already counts Accept all as a photographer action. Optional row: two pending keys; research turns `k1` into a non-conflict at medium; `accept_all`; then `k1` has an entry with action `accepted`. |
| N4 | **A resolution whose row is not valid.** Task 5 writes "the key's resolved canonical value". Two actions leave a live but invalid row. (1) Keep previous restores an invalid `integration` prior: MC-A1 lets research write over "integration, live, valid or not", and `decide_write` stores `valid` in `ai_prior`. (2) An Edit that does not fit is stored verbatim with `valid = false` (`set_value` → `stored_form`). An invalid row has no canonical value. Sending its verbatim text breaks the frozen rule ("every live, valid row"; A09·C1(e)). | plan gap (the frozen rule decides it) | Amend task 5: "a resolution whose row after the action is not live and valid records 'no value' (never sent, like Clear)". This mirrors MC-A10's frozen rule and 07A MC-13. For an invalid integration prior, the shop already holds that value, so not sending it is right. Optional row: keep previous of an invalid integration prior on the last pending key: the item is released and the key is in no write. Producible: a linked item, a FakeShop value that does not fit the definition, `read_integration_values`, then `writes::suggest`. |
| N5 | **A09·C3(a) "every recorded mutation belongs to the second item's product" is not observable.** `FakeCommerce` records `Mutation {op, run_id, at}` (`fake_commerce.rs:32–36`), and both items share one run. `attribute_writes` is `Vec<Vec<WriteValue>>` with no product id (`fake_commerce.rs:514–516`). This is attempt 1's L10 again: the amendment kept a clause the fake cannot decide. | plan gap (decidability) | Replace the clause with: "`FakeShop.products` holds exactly one product, with the second item's SKU, and `FakeShop.values` has entries only under that product's id". Both are observable (`RemoteItem.sku`, `values` keyed by external id). |
| N6 | **Desktop C5(a) fixture: the key must be visible on the command path.** The commands build `AttributesService` with the integration named in settings (`commands/attributes.rs:226–243`, `settings.integration.kind`). `TestApp::with_fake_document()` has none, so only the built-in Product fields are active, and research never writes those. The conflict helper writes through `writes::suggest` with `Some("shopify")`, so `attribute_ai_accept` on that key would fail "unknown attribute". | plan gap (minor; findable, but costs a debugging loop) | Add to U1 Files' helper note: "the helper imports the definition, and the C5(a) test configures settings so `integration.kind` is that integration before `AppState::new`". |
| N7 | **Redo re-records a resolution: from what?** The redo must write the value and action (`kept_previous` / `accepted` / `edited` / `cleared`) again. `Step` carries neither today. The after-row alone does not name the action: Keep previous restores a `default` or `integration` row. | free choice | Delegate in writing: "the attribute step keeps what it needs to re-record its resolutions (e.g. the entries it wrote, as it keeps its feedback rows for re-insert)". |
| N8 | **"`pending_conflicts` (also part of `confirmed_summary`)".** `confirmed_summary` is the summary *text* (07A MC-10: "the exact text shown"), built by `mechanisms::publish::publish_summary` (`mechanisms/publish.rs:60`). The plan does not say whether R5 adds the withheld and waiting lines to that text, or whether the stored column is "the part". R6·U3 draws those lines in the popover. | free choice (+ R6 fold) | Delegate: "in R5 the `pending_conflicts` column is the record. `publish_summary` is unchanged". Fold into R6·U3: if the popover shows the withheld and waiting lines, `summary` (the text confirmed) carries them. |
| N9 | **Preview `details_filled` with withheld or pending rows.** `preview` counts Details keys present in the live-row map (`publish.rs:566–577`). Withheld and pending rows count as filled although they will not be sent. | free choice | Delegate: "`details_filled` unchanged in R5". |

## Reality checks

- **Files exist or are marked new.** Every edited path exists. New: `mechanisms/publish_partition.rs` (U1 creates
  `is_conflict`, U2 adds `partition`), `research/r5_feedback_and_undo.rs`, `research/r5_publish_hold.rs`. The
  suite's `research` module exists (`tests/suite/main.rs:61`, `research/mod.rs`). The `confirm` perimeter is
  verified exactly: 42 call sites in the 15 listed testkit files. The plan's `p15` is `p15_publish_backend.rs`, the
  same prefix. Outside the testkit, the only callers are `commands/publish.rs:124`, `services/publish.ts:12–13`,
  `PublishPopover.tsx:27`, `bindings.ts:136`, and the two TS tests named. `create_item:` preview literals appear
  only in those two TS tests. No caller is missing.
- **Code facts the rows rely on, verified.** `writes::suggest_in` runs `decide_write` (cleared set) before
  `same_content` (`writes.rs:197–223`), so A08·C4(a)'s `Unchanged` is reachable and discriminating. The cleared set
  reads feedback `cleared`/`kept_previous` (`writes.rs:103–124`), so A08·C1(c) has one cause. `decide_write` keeps
  the prior on `ai`+`suggested` and refuses nothing for `S`-equal-to-prior, so A09·C1(d)'s path is producible
  (`write_table.rs:58–87`). `coalesce_attribute` matches only `Edit `-prefixed steps, so A09·C4(e)'s Keep previous
  then edit are two steps. `active_publish_items` excludes `held` (`infra store.rs:2116`). The startup re-queue runs
  on every `kill_and_restart` (`integrations/src/lib.rs:534–540`, `start_backup` true), so A09·C6(b) does exercise
  it. `earlier_in_prepare` and `start_next_in_batch` filter `step == Prepare` on active items
  (`publish.rs:1150–1188`), so a held first item does not block the second (A09·C3(a)). `cancel` refuses
  non-active steps today (`publish.rs:243`), so A09·C6(a) needs a `Held` branch, as the plan's hazard note says.
  `history.rs` `p40` `c4d_title_undo_redo` exists (`p40_undo_history_values.rs:376`). `psm_core::mechanisms::sameness::same`
  exists (`sameness.rs:10`). The desktop crate has `psm-testkit` as a dev-dependency (`src-tauri/Cargo.toml:33`).
- **Bookkeeping drift (coordinator, no implementer impact).** `master_plan_research.md:75` still says R5 has
  "12 / 34" rows; the plan now has 36 (16 + 20, counted). `research_row_mapping.md` lists A09·C4(a)–(d) only
  (lines 250–253), not the new C4(e)/(f). The plan says that file "says where every row went".

## Criteria decidability (can the test be written now with one exact outcome? is the fixture producible?)

| Row | Decidable | Producible | Note |
|---|---|---|---|
| A08·C1(a)–(f) | yes | yes | (b) needs U1's `is_conflict` (now in U1) |
| A08·C2(a) | yes | yes | |
| A08·C3(a)–(d) | yes | yes | (c) via `fill_defaults` (default after-rows, research may write over) |
| A08·C4(a) | yes | yes | amended L1 verified against `writes.rs` order |
| A08·C4(b) | yes | yes | redo meets an unchanged row (Unchanged writes nothing) |
| A08·C4(c) | yes | yes | |
| A08·C5(a) | yes | yes, with N6 | |
| A08·C5(b) | yes | yes | |
| A09·C1(a)–(c), (e) | yes | yes | |
| A09·C1(d) | yes | yes | path in the cell verified |
| A09·C2(a), (b) | yes | yes | |
| A09·C3(a) | **partly** | yes | N5 |
| A09·C3(b) | yes | yes | p13 form |
| A09·C4(a)–(d) | yes | yes | |
| A09·C4(e) | yes | yes | Keep previous is not `Edit`-labelled, so no coalescing |
| A09·C4(f) | yes | yes | depends on N7's delegation for how |
| A09·C6(a)–(c) | yes | yes | |
| A09·C7(a) | yes | yes | |
| A09·C8(a) | yes | yes | |

Rule-17 scope: no row's observable is owned by an external dependency. Every fixture is our store, the research
crate's write door, or `FakeCommerce`, all read in the installed tree above.

## Trace verification (both directions)

- Every trace cell resolves (`14_measurement_ledger.md`: M20 line 32, MA8 71, MA17 87, MA18 88; MC-A10/A12 in 08B).
  The new rows C4(e), (f) trace to MC-A10 changelog 6, which says what they assert.
- Claimed ledger entries (master plan research §5): MA8 (R5·U2), MA17 (R5·U2), MA18 (R5·U1). Each is served by at
  least one row. MA17's "each pending key is sent with the value recorded at its resolution" is served for the shop
  by C4(b)/(c). N1 is where that clause can still fail in practice, through the typing path no row exercises.

## Write perimeter of this session

- Created: `docs/implementation/handoffs/reviewer/handoff_R5_research_app_seams_projection_r0.md` (this file).
- Nothing else. No plan, intention, code, tracker, archgraph or prompt change, and no commit. Commands were
  read-only (`git log/show/status`, `grep`, `sed`, `ls`, `cat`).

## Appendix (non-authoritative): skeleton notes, discarded

Not guidance for the implementer. Kept only so the coordinator can see what the ledger came from.
- U2 hold step: `resolve_pending_in(tx, item_id, key, resolution, token, now)` → find the `held` publish item of the
  item's session(s) → if `key ∈ pending ∧ key ∉ resolved` write the entry → if all pending keys are resolved: step
  `prepare`, `released_at`, `StoreTx` enqueue. Undo path in `history.rs` apply: delete entries with the token when
  the item is `held`. Redo path: re-record from what the step kept (N7).
- Run path: an effective map `frozen ∪ resolved − no_value`, fed to `run_staged` and `write_set` (N2).
