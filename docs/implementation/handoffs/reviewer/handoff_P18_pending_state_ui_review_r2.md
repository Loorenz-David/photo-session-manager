---
plan: P18_pending_state_ui
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-26
actor: opus
---
# P18 re-review r2: APPROVED_WITH_NOTES

Fix 1 (`bcd4f51`) resolves F1 as the correction described, and stays inside the allowed files. The build is green. No
blocking item remains, so there is nothing to triage. The five r1 notes stay in the backlog, and I added one cosmetic note
seen in passing. Tracker → `APPROVED`. Narrative written.

## ⚠ OWNER DECISIONS REQUIRED (0)

none

## Review history

- **r1** settled all 9 criterion rows (C1a–C3c), the backend search split, H1–H8, the assumptions and the declared
  deviations. It raised one blocking finding: F1 (B1, production), stale shop hits shown as the current query's finished
  result in `NewSessionDialog` and `ItemPickerDialog`.
- **This round** is delta-scoped to the F1 seam. Settled areas are not re-verified.

## Perimeter check

- Gate: the tracker row was `IMPLEMENTED` with the fix-1 note, and checkpoint `bcd4f51`
  (`CHECKPOINT (not approved): fix P18 stale shop hits`) is in `git log`.
- `git show --stat bcd4f51` lists `apps/desktop/src/features/sessions/NewSessionDialog.tsx`,
  `apps/desktop/src/features/workspace/PickerDialogs.tsx` and
  `apps/desktop/src/features/sessions/__tests__/p18_item_search.test.tsx`. These are exactly F1's allowed files.
- `git diff bcd4f51 HEAD -- apps crates` is empty. Only runner commits (fix handoff, tracker row, Review log entry,
  re-review prompt) follow the checkpoint, and each matches what the fix handoff declared.
- **Result:** no out-of-perimeter change.
- Suite, run once on the fix tree:

  | Command | Result |
  |---|---|
  | `cargo test --workspace` | 407 passed, 0 failed |
  | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
  | `cargo fmt --all -- --check` | exit 0 |
  | `npm run typecheck` | exit 0 |
  | `npm test` | 9 files, 102 passed |

  `git status` was clean before and after. This matches the fix handoff's stamp.

## Findings resolved

**F1: resolved as corrected (the "key shop hits to the query they answer" option, plus the reset).**

- **What changed.** Both dialogs now store `shopFor` and set it only when a shop reply for the still-current query is
  accepted. Shop hits render only when `debouncedShop === title && shopFor === title && shopState === 'done'`. Each new
  shop effect also sets `shopState` to `searching`. `shopFor` is cleared on open and on an empty query.
- **Case table I traced on the new code:**

  | Sequence | Outcome |
  |---|---|
  | `a` done → `ab` typed, between the local debounce and the shop debounce | `a` hits hidden (`debouncedShop ≠ title`), heading `searching the shop…` |
  | `a` done → `ab` shop request in flight | hidden (`shopFor = 'a' ≠ 'ab'`). This is r1's failing step 3. |
  | `ab` reply lands | shown, heading `n in the shop` |
  | `ab` in flight → back to `a` | the late `ab` reply is dropped by the current-title check. `a` is re-queried and stays hidden until its reply lands, because the effect reset to `searching`. |
  | `a` done → `ab` → `a`, all inside 300 ms | `debouncedShop` never leaves `a`, no new query is sent, and the `a` hits stay shown. This is correct, because they answer the current text. |
  | Older reply lands after the newer one (original C3(c)) | still dropped |

- **Tests.** The two new C3(c) cases (one per dialog) resolve `a`, start `ab` with its reply held, and assert that
  `shop A` is absent with heading `0 found · searching the shop…`, then that `shop AB` appears once the reply lands.
  - The middle assertion is exactly the state the r1 probe showed wrong, so on the pre-fix code it would fail (judged by
    reading the code, without editing production).
  - They test the outcome at the boundary and pass the over-engineering test.
  - Both trace to row C3(c). The original C3(b) and C3(c) tests still pass.

## Triage

No blocking item remains, so there is nothing to classify as MUST or DROP.

| Item | Class | Destination |
|---|---|---|
| (none) | — | — |

## Remaining findings

None blocking.

**Note, seen in passing, written to the backlog.** The `shop unavailable` heading is gated on
`debouncedShop === title && shopState === 'unavailable'`, and the reset to `searching` happens in the effect after the
render in which `debouncedShop` catches up. After a failed shop query, the next query can therefore show
`shop unavailable` for one render. This is cosmetic, and only reachable when the shop command itself fails (r1 note 1).

**Carry-forward dispositions** (all in `docs/implementation/backlog.md`, P18 rows):

| Note | Destination |
|---|---|
| `search_shop` swallows adapter errors | backlog (next Shopify-touching phase) |
| FormDialog shared AI-action lock | backlog (next form phase) |
| C2(d) counts skeletons across both pages | backlog (test) |
| C1(b) default toast path verified by reading only | backlog (test) |
| WorkspacePage keeps its own loader | backlog (design pass on pending states) |
| `shop unavailable` one-render flash (new) | backlog (with the `search_shop` note) |

## Lessons for the plans

None new. r1's lesson stands: for stale-response rows, list both orderings.

## Write perimeter and probe declaration

- Written:
  - `docs/implementation/master_plan.md` (P18 tracker row only → `APPROVED`)
  - `docs/implementation/plans/P18_pending_state_ui.md` (Review log entry r2)
  - `docs/implementation/backlog.md` (1 P18 row appended)
  - `docs/implementation/narratives/P18_pending_state_ui.md` (new)
  - this handoff
- No probes. No production or test file touched. No database or state side effect.
- No commits: the runner commits these files.
