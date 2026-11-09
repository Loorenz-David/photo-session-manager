---
plan: P59_item_type_from_shopify_product_type
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-09
actor: opus
---
# P59 re-review r2: APPROVED_WITH_NOTES

Fix 1 (`a5ce4ab`) resolves both r1 blocking findings as their corrections said. The perimeter is within r1's allowed files, and suite, lint and typecheck are green. No blocking item remains, so there is nothing to triage. The verdict carries notes because r1's five backlog notes stay open (already in `backlog.md`). This round adds no new note.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Review history

r1 (`c69c482`) settled C1(a), C1(b), C2(c), C3(a), C4(a), H1/H4/H6-adjacent publish semantics, H7 and H8. Those areas are not re-verified here. r1 left F1 open (New Session blocked on a failed linked-item refresh, B1/H6) and F2 open (C2(a) asserted the fake's filtering, B2), plus five notes → backlog.

## Gate

- Tracker row: `IMPLEMENTED`, note "fix 1: F1 … F2 …". Fix checkpoint `a5ce4ab51658273fa211e8f1bd3cb532b2304001` is in `git log`.
- Tree: HEAD `4ba8bf5` = `a5ce4ab` + the re-review prompt only (`git diff --stat a5ce4ab 4ba8bf5`: one prompt file). `git status --porcelain` was clean at entry.

## Perimeter check

`git show --stat a5ce4ab`: 7 paths, equal to the fix handoff's "Files changed".

| Path | Allowed by |
|---|---|
| `apps/desktop/src/features/sessions/NewSessionDialog.tsx` | F1 |
| `apps/desktop/src/features/sessions/__tests__/p59_item_type_picker.test.tsx` | F1 |
| `crates/testkit/tests/p59_item_type_from_shopify_product_type.rs` | F2 |
| `apps/desktop/src-tauri/src/lib.rs` (removes `mod p59_tests` only, −60 lines) | F2 |
| `docs/implementation/master_plan.md` (P59 tracker row only) | fix role's own row |
| `docs/implementation/plans/P59_…md` (Review log entry only) | fix role's own entry |
| `docs/implementation/handoffs/implementer/handoff_P59_…_fix_r2.md` | fix handoff |

Nothing outside the perimeter. No production code changed apart from the F1 site.

## Evidence (one run on the HEAD `4ba8bf5` tree; code = `a5ce4ab`)

- `cargo test --workspace`: 863 passed / 0 failed. Same count as r1: one test removed (`p59_tests::c2a_…` in lib.rs) and one added (`c2a_shopify_adapter_returns_sorted_distinct_nonblank_product_types`). Log `/tmp/p59_r2_cargo_test.log`. An earlier attempt was cut off by the session's background time limit before it reported; that one is not counted.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- `npm run typecheck`: clean.
- `npm test`: 59 files / 422 tests passed. The known-intermittent `p33_split_grid` test did not fire this run.

**Not B4.**

## Findings resolved

- **F1: resolved as corrected.** `openAction` now wraps `refreshItem` for `origin === 'linked'` in `try/catch` and falls back to the stored item, then calls `createSession(local.id)`. This is the same best-effort pattern `FormDialog` uses (`refreshItem(...).catch(() => undefined)`). The only other `refreshItem` caller is `FormDialog`, already best effort. The adopt path (`id === 0 && external_id`, a shop-only hit) still surfaces its error. That is right: such an item exists only in the shop, so it cannot be opened without it. New boundary test `opens a linked item with its local data when shop refresh fails`: `refreshItem` rejects, then the test asserts `createSession` was called with `7` and `onOpened` with `3`. Can it fail? Under the pre-fix code, the rejection aborted `openAction` before `createSession`. r1 reproduced exactly this with a scratch test, so this test fails on the old behaviour. H6 restored: with no integration (`NoIntegration`) or the shop unreachable, an existing linked item opens.
- **F2: resolved as corrected (first option).** The new testkit test drives the real `ShopifyAdapter::item_types` through `FakeTransport`. Its `ProductTypes` reply has nodes `["Lamps","Chairs","","Lamps"]`, `hasNextPage:false`, and the test asserts `["Chairs","Lamps"]`. The operation name matches `graphql::PRODUCT_TYPES` (`query ProductTypes`). The adapter (`adapter.rs` `item_types`) collects into a `BTreeSet` and filters `trim().is_empty()`, so sort, dedupe and blank-drop are all production behaviour now under assertion. Can it fail? Without sorting, the result would be `Lamps, Chairs`. Without dedupe, a list (not a set) gives 3 entries. Without blank-drop, `""` sorts first. Each one breaks the `assert_eq`. The fake-backed Tauri test was removed, so no orphan or fixture-asserting C2(a) test remains (`grep c2a` in the P59 files: one hit).

## Changed seam, read adversarially

- `catch {}` swallows every refresh error, not just shop errors. If the local item were gone, `createSession(local.id)` would then fail and surface through `useAction`'s error handler, so nothing is hidden that matters. When the refresh fails, the session opens on the stored type, and the form's own mount refresh tries again. Acceptable, no note.
- Passing glance: nothing else seen wrong.

## Triage (flow.md §5)

| Item | Class | Disposition |
|---|---|---|
| (none: F1 and F2 resolved, no new blocking item) | — | — |

No MUST, so the verdict is `APPROVED_WITH_NOTES`.

## Remaining findings

None blocking. Open notes carried from r1, already in `docs/implementation/backlog.md` under "P59 reviewer r1":

| Note | Destination |
|---|---|
| C2(b) listing clause ("the dialog lists the shop's types") unasserted | backlog (next phase touching the New Session picker) |
| NULL `item_type_base` lets a first refresh overwrite a local edit | backlog (the `beyo.item_type` source change, which rewrites this mapping) |
| Local-origin published items never refresh on form open | backlog (same) |
| Header picker commits free text while the type list is loading | backlog (next phase touching the picker) |
| Native `<datalist>` instead of the form's searchable select | owner's manual try-out on the work Mac |

## Mutation-probe declaration

No probes (flow.md §4: the regime is off, no production edits). This session's only writes: this handoff, the narrative `docs/implementation/narratives/P59_item_type_from_shopify_product_type.md`, the P59 tracker row in `docs/implementation/master_plan.md`, and the r2 Review log entry in the plan file. Scratch output went to `/tmp/p59_r2_cargo_test.log` (outside the repo). No commits.

## Lessons for the plans

- When a plan puts filtering semantics on a port call ("distinct, sorted, empty strings dropped"), the criterion row should name the boundary where that is proven: the adapter, or the service if the service owns it. Otherwise, a test through the generic fake proves the fake. F2 came from this ambiguity.
- "Refresh reuses the existing read paths" should say "best effort, never blocks" whenever H6 applies. F1 came from leaving that unsaid.
