---
phase: A09_research_publish_hold
title: Publishing with unchecked suggestions — weak guesses are left out, a suggestion that replaced a preset or Shopify value makes that one item wait until you decide, and the other items of the same publish go ahead
difficulty: high
projection: yes
depends_on: [A08_research_feedback_and_undo]
---
# A09 — Suggestions in a publish: withheld values, conflicts and the held item (08B MC-A10; MA8, MA17; amends 07A MC-10)

## Goal
At confirmation, split each item's values into frozen, withheld (`ai` + `suggested` + `low`, no prior) and pending
(conflicts); record the origin of each; refuse the confirmation when what the popover showed is no longer true;
start an item that has pending keys at the new step `held`, where it sends nothing; freeze each pending key at its
resolution and release the item at the last one; keep the other items of the confirmation running. Backend only.
**Not in this phase:** the popover and row-label UI (A20); research runs (the conflicts in this phase's tests are
made with `ai_suggest`).

## Read first
`master_plan.md` §3, §6.5–§6.7, §7.5 · `master_plan_research.md` §3, §6.3–§6.6, §7.4 · book `01` (H1, H4), `14` (M5,
M14, M17, MA8, MA17), `07` (publish steps), `07A` **MC-10** (frozen confirmation and its research pointer), MC-11
(resume, startup re-queue), MC-13 (never clears), `05A` MC-7 and its research pointer (line 205), `08A` §7,
`08B` **MC-A10** (all), MC-A12 (resolution in the action's transaction), MC-A1 (conflict) · code
`crates/core/src/services/publish.rs` (`frozen_attributes` 472, `preview` 518, `confirm` 592–720, `run_item` 758,
`cancel` 241, `retry`), `crates/core/src/domain/{publish.rs (PublishItem), session.rs (PublishStep, RowLabel)}`,
`crates/core/src/mechanisms/status.rs`, `crates/core/src/services/attributes.rs` (A08's actions),
`apps/desktop/src-tauri/src/commands/publish.rs`, `crates/testkit/src/fake_commerce.rs`.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/infra/src/sqlite/migrations/a02_research_publish.sql`, `crates/core/src/mechanisms/research/
publish_partition.rs`, `crates/testkit/tests/suite/research/a09_research_publish_hold.rs` (+ `mod` line).
Edited: `crates/core/src/domain/{publish.rs, session.rs}`, `crates/core/src/mechanisms/status.rs`,
`crates/core/src/services/{publish.rs, attributes.rs, history.rs}`, `crates/core/src/ports/store.rs`,
`crates/infra/src/sqlite/store.rs`, `apps/desktop/src-tauri/src/commands/publish.rs`,
`apps/desktop/src/services/{bindings.ts, publish.ts}`, the publish UI call site that passes the shown set
(`features/publish/PublishPopover.tsx`: pass what the preview returned; no new UI), every `match` over `PublishStep`
and `RowLabel` in Rust and TypeScript, own tracker row.

## Core mechanisms (unit tests allowed)
`publish_partition::partition` · `mechanisms::status::{derive_status, row_label}` (existing, extended with `held`).

## Ordered tasks
1. Migration `a02_research_publish.sql` (§6.4) in the research block, after `a01`.
2. `PublishStep::Held` (ordered before `Prepare`); `RowLabel::WaitingForYou(n)`. `derive_status` counts `held` as
   publishing; the label is placed before `Publishing n%`; n = pending keys without a resolution (derived).
3. `partition(values, definitions)`: frozen = live, valid rows, except withheld (`ai` + `suggested` + `low` with no
   prior) and pending (`is_conflict`); a row with a prior that is not a conflict is frozen at any confidence.
   `origin` holds provenance, status, confidence and prior for every frozen, withheld or pending key.
4. `preview` returns, per item, `n_withheld` and the conflicts (key, label, prior value and origin, suggestion,
   confidence). `confirm(session_ids, summary, shown)` recomputes both from the rows it reads **inside the
   confirming transaction**; a difference for any item refuses the whole confirmation with `AI suggestions changed
   while you were confirming` and writes nothing. Otherwise it writes `frozen_attributes`, `pending_conflicts`
   (also part of `confirmed_summary`) and `frozen_attribute_origin`; an item with pending keys starts at `Held` and
   **no job is queued for it**.
5. Resolution: A08's actions (`accept`, `keep_previous`, `set_value`, `clear`), when the key is pending on a `held`
   item of that product, write the key's resolved canonical value, action and time into `resolved_attributes` in
   the same transaction (`clear` records "no value"). The transaction that resolves the last pending key sets the
   step to `Prepare`, records `released_at` and queues the job. Undoing a resolution while the item is still held
   removes the entry (the key is pending again).
6. `run_item` sends, for a released item, the frozen values plus each resolved key's value (a "no value" key is not
   sent). `cancel` on a held item sets `Cancelled` at once. The startup re-queue leaves held items held.

## Criteria (8 criteria, 20 rows)
Boundary: `PublishService::{preview, confirm, cancel}`, the attribute actions, `TestApp::tick`, and the mutations
`FakeCommerce` recorded. "Sends" means a recorded `attributes_write`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `confirm` → `publish_items` | one item with rows: `human`, `default`, `integration`, `ai` + `accepted`, `ai` + `suggested` high, `ai` + `suggested` medium | all six in `frozen_attributes`; step `prepare` | MA17 |
| C1(b) | same | plus `ai` + `suggested` `low` with no prior | absent from `frozen_attributes`; `n_withheld` = 1; origin recorded for it | MA8 |
| C1(c) | same | plus a conflict at `low` and a conflict at high | both absent from `frozen_attributes`, both in `pending_conflicts` with prior value, origin, suggestion and confidence; neither counted as withheld | MA17 |
| C1(d) | same | plus `ai` + `suggested` `low` whose prior is the same value under `S` | frozen | MC-A10 |
| C1(e) | same | a cleared row and an invalid row | neither frozen (as before this phase) | M17 |
| C2(a) | `confirm` re-check | preview shows `n_withheld` 1; a second low suggestion is written; confirm with the shown set | refused with `AI suggestions changed while you were confirming`; no `publish_runs` row | MA17 (R-IA22) |
| C2(b) | same | preview shows conflict keys `{k1}`; `k1` is resolved before confirm | refused the same way; no run row | MA17 |
| C3(a) | held item | two sessions confirmed together, the first with one conflict | first item step `held`, no job row for it, no mutation for it after `tick` until idle; the second item reaches `done` with its gallery and values written | MA17 |
| C3(b) | H1/M5 | C3(a) | the run's `confirmed_at` precedes every recorded mutation | M17 |
| C4(a) | resolution, two pending keys | resolve `k1` with Keep previous | `resolved_attributes` holds `k1` = the prior's value with action and time; step still `held`; no mutation | MA17 |
| C4(b) | same | then research writes a new suggestion on `k1`; then resolve `k2` with Use suggestion | step `prepare`, `released_at` set, one job queued; after `tick` the shop holds `k1` = the prior's value and `k2` = the suggestion; the later suggestion on `k1` was not sent | MA17 |
| C4(c) | same | a held item whose only pending key is resolved by Edit to H; by Clear | Edit: released, H sent; Clear: released, the key is not in any write | MA17, M20 |
| C4(d) | undo | a held item with two pending keys: resolve `k1`, then undo | `k1` has no entry in `resolved_attributes`; the item is still `held` with two pending keys | MC-A10 |
| C5(a) | `derive_status`, `row_label` | latest step `held`, two pending keys, one resolved | `Publishing`; `WaitingForYou(1)` | M14 |
| C5(b) | session list | a session whose latest item is `held` | not offered Publish again (status `publishing`) | M14 |
| C6(a) | `cancel` | a held item | step `cancelled` immediately; no mutation ever recorded; the conflicts are still conflicts in the form | MC-A10 |
| C6(b) | restart | `kill_and_restart` with a held item | still `held`, no job row | MA17, H4 |
| C6(c) | restart | resolve the last key, then `kill_and_restart` before any tick | step `prepare` with its job; the item then reaches `done` once (no duplicate product or media) | H4, M18 |
| C7(a) | conflict after confirmation | an item confirmed with no conflict; research then creates a conflict on key k before the update step runs | the item is not held; k is sent with the value frozen at confirmation | MA17 |
| C8(a) | MA8 evidence | C4(b)'s finished item | `frozen_attribute_origin` and `resolved_attributes` together account for every key sent; no key sent was `ai` + `suggested` + `low` without a prior at confirmation | MA8 |

## Delegated choices
JSON layouts of the three new columns (key → object) · how `shown` is passed (`[{session_id, n_withheld,
conflict_keys}]`) · the progress percentage of `held` (0) · where the release function lives (`PublishService`,
called from the attribute actions through a narrow trait so `attributes.rs` does not depend on the whole service).

## Manual try-out
On scratch data with the test shop connected: nothing looks different yet (the popover text and the conflict
buttons arrive in A20). Publishing an item without suggestions works exactly as before.

## Notes
- Hazard: adding `Held` reaches every exhaustive `match` on `PublishStep` (`publish.rs` lines 114, 264–281,
  767–813; `status.rs`; the TypeScript unions). `Held` is never an "active" step for the startup re-queue.
- Hazard: `confirm` today builds the preview outside the transaction and re-checks photos inside it; the new
  re-check follows the same pattern and must read the attribute rows **inside** (`publish.rs` 640–648).
- The 07A MC-10 skip of photos deleted or moved since still applies when a released item runs (C4 keeps one such
  photo out if the implementer wants a combined row; not required).
- Review mode: this phase's module, `p13_publish_runs::`, `p14_publish_update::`, `p64_publish_speed_and_visibility::`,
  `p03_session_state_list::`, `cargo test -p psm-core mechanisms::status`, the publish UI tests, clippy, fmt, typecheck.

## Review log
