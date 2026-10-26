---
plan: P18_pending_state_ui
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-26
actor: opus
---
# P18 review r1: CHANGES_REQUESTED (1 blocking finding)

The pending-state primitives and the backend search split are correct. All 9 criterion rows hold, and the build is green.
There is one blocking finding (B1). In both item-search dialogs, the previous query's shop hits come back into the list,
labelled as finished, while the current query's shop request is still running. The fix is local to two components.

## ⚠ OWNER DECISIONS REQUIRED (0)

none

## Gate and stamp

- Tracker showed `IMPLEMENTED`. Checkpoint `9270643` (`CHECKPOINT (not approved): implement P18 pending state UI`) is in `git log`.
- `git show --stat 9270643`: the 42 committed paths equal the handoff's declared list. The handoff file itself is the only
  declared path outside the checkpoint, and the runner committed it in `eba1cc7`. No extra or missing paths.
- I re-ran everything on the checkpoint tree with a clean worktree: `cargo test --workspace` 407 passed, 0 failed ·
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0 · `cargo fmt --all -- --check` exit 0 ·
  `npm run typecheck` exit 0 · `npm test` 9 files / 100 tests passed. `bindings.ts` is unchanged after the suite's
  `export_bindings` regeneration (`git status` clean).

## Verified correct

| Row | Production that makes it true | Test verdict |
|---|---|---|
| C1(a) | `lib/pending.ts::useAction`: a ref lock (`locked.current`) blocks a second `run` in the same render, `busy` is set before the await, and the `finally` clears both. `ActionButton` sets `disabled = disabled \|\| busy`, `aria-busy="true"` and an `…` suffix. | Clicks against a held promise. Asserts `aria-busy`, disabled, `fn` called once after the second click, then enabled again. It would fail without the lock or without the busy state. |
| C1(b) | The `catch` sets `error` and calls `onError` (default `toast.error(message)`). The `finally` re-enables. | Rejects with `Error('nope')`. Asserts `onError` received it and the button is enabled with no `aria-busy`. Holds; the default toast path is only verified by reading (backlog note). |
| C2(a) | `usePageData`: `loading` starts `true` and is cleared only by a ticket-current result. `reload` never sets `loading` again. Unmount or a dependency change bumps the ticket. | Two skeletons and `aria-busy` before the first result, 0 after, and still 0 with content shown during `reload()`. |
| C2(b) | `SessionsPage` renders `SkeletonRows n=6` in an `aria-busy` table while `loading`. The empty state and rows render only after. Filter and query changes reload without skeletons. | Asserts ≥ 3 skeletons, no "No sessions yet", no rows before; 2 rows and 0 skeletons after. |
| C2(c) | `WorkspacePage` returns a stage `Skeleton` plus `SkeletonTiles n=8` in the `Photos` panel while `!state.view`. A load error still takes precedence. | Asserts 8 skeletons in the panel, one in `.stage-column`, then 2 photo tiles and 0 skeletons. |
| C2(d) | Health: `SkeletonRows n=4` plus a path `Skeleton`. Settings: `SkeletonFields n=6`. `grep Loading…` in `src/` returns 0 hits outside the test. | Asserts no `Loading…` and loaded content after. The skeleton count is checked across both pages together (backlog note). |
| C3(a) | `ItemsService::search_local` is DB-only. `search_shop` returns `[]` for an empty query or no commerce. Otherwise it returns `item_find` hits minus linked external ids and minus SKUs held by any local item. `search` = local followed by shop. The `items_search` command is local-only. `items_search_shop` is new, registered in `lib.rs` and bound. | `c3a` asserts `[SKU-1]` with no `item_find` recorded, then `[SKU-9]` with `id 0`, `Linked`, and exactly one `item_find`. Each assertion fails on the defect it names. |
| C3(b) | `NewSessionDialog`: 75 ms local debounce, 300 ms shop debounce. The local list is gated by `searchedFor === title`. The heading is `n found · searching the shop…` until shop hits are shown. | Asserts row and heading at 75 ms, no shop call before 300 ms, `2 found · 1 in the shop` after. It holds for the row's input; see F1 for the variation where it fails. |
| C3(c) | `alive` (effect cleanup) plus `currentTitle.current === debouncedShop` drop a late reply for an older query. | Shop(`a`) resolves after shop(`ab`), and `shop A` is never shown. Holds for that ordering. |

Also verified:
- The SKU conflict check queries both searches.
- `ItemPickerDialog` (Assign product) splits the search the same way.
- Both assign flows now call `adoptItem` for a shop hit (`id 0`) before `assignUnassigned` / `createSession`, where the old
  code would have passed id 0. I judge this sound.
- Move/copy pickers and Assign now close only after the service call succeeds.
- `PublishPopover` Cancel is disabled while the publish runs (closes the P15 backlog note).

**Hard constraints H1–H8.**
- H1: the only new remote call is the read-only `item_find`.
- H2–H5: untouched.
- H6: with no integration, `search_shop` returns `[]` and the dialogs show local results.
- H7: the `"shopify"` literal in `search_shop` was moved from the old `search`, not added. It is already covered by the P13 backlog note.
- H8: the commands are thin wrappers over `ItemsService`.

No live services in tests: everything runs on the fakes.

**Assumptions** (all inside the plan's Delegated choices; none contradicts the book):
- Debounce 75 ms local (≤ 100) and 300 ms shop.
- Ellipsis busy mark.
- `shop unavailable` heading on a failed shop search.
- Review buttons and keyboard review stay immediate.

**Deviations.** The extra nested files (AiPopover, MetafieldInput, ResearchPanel, IdentityEditor, CustomizeMode,
NotificationsDrawer, StatusPill, SessionRow, CameraDot, the P07/P08/P13 test mocks) are declared, and each carries a busy
state or a mock export the phase requires.

## Findings

### F1: blocking (B1), route: production. Previous query's shop hits are shown as the current query's finished result

- **What is wrong.** `NewSessionDialog.tsx` computes `shopDone = debouncedShop === title && shopState === 'done'` and shows
  `shopResults` when it is true. `ItemPickerDialog` in `PickerDialogs.tsx` uses the same logic. When a new shop query starts
  (`debouncedShop` catches up to the new title), neither `shopResults` nor `shopState` is reset. The previous query's hits
  therefore satisfy `shopDone` again until the new reply lands.
- **Evidence.** I ran a scratch vitest probe (written, run and deleted; no production edit) on both dialogs:
  1. Type `a`. The shop returns `[shop A]` and the heading reads `1 found · 1 in the shop`.
  2. Type `ab` with the `ab` shop reply held. After 80 ms the heading is `0 found · searching the shop…` and `shop A` is hidden, which is correct.
  3. After the 300 ms debounce, while `ab` is in flight, the heading is `1 found · 1 in the shop` and `shop A` is listed again.

  The probe failed identically for the New session dialog and the Assign-product picker.
- **Authority.** Plan task 4 ("lists local hits at once with the heading `n found · searching the shop…`, appends shop hits
  when they arrive, drops a shop response whose query is no longer the current one"). Row C3(b): the heading reads
  `searching the shop…` until the current query's shop hits arrive. Row C3(c)'s rule (R5): a previous query's shop hits
  never appear for the current query.
- **Correction.** Key shop hits to the query they answer, for example by storing `{for: debouncedShop, items}` and showing
  them only when `for === title`. Alternatively, clear `shopResults` and set `shopState` to `searching` at the start of each
  shop effect. Apply the same change in both dialogs.
- **Files the fix may touch.** `apps/desktop/src/features/sessions/NewSessionDialog.tsx`,
  `apps/desktop/src/features/workspace/PickerDialogs.tsx`, and optionally
  `apps/desktop/src/features/sessions/__tests__/p18_item_search.test.tsx` (the C3(c) test may add the "older reply landed
  first" ordering).

No other blocking findings.

## Notes written to backlog

1. `search_shop` swallows adapter errors, so `shop unavailable` appears only when the command itself fails. During a shop
   outage the dialog reads "0 in the shop".
2. FormDialog's single shared AI-action lock silently drops a concurrent accept or clear on another field, and closes its
   popover as if the action worked. A failed action also no longer reloads the form.
3. The C2(d) test counts skeletons across Health and Settings together.
4. The C1(b) test exercises only the explicit `onError` path.
5. WorkspacePage keeps its own loader instead of `usePageData`.

## Lessons for the plans

- When a row covers stale-response handling, enumerate both orderings: older reply lands late, and older reply landed
  first and is still on screen when the next query starts. C3(c) named only the first, and the defect sits in the second.

## Write perimeter and probe declaration

- Written:
  - `docs/implementation/master_plan.md` (P18 tracker row only)
  - `docs/implementation/plans/P18_pending_state_ui.md` (Review log entry)
  - `docs/implementation/backlog.md` (5 P18 rows appended)
  - this handoff
- Probe: `apps/desktop/src/features/sessions/__tests__/zz_review_probe.test.tsx` was created, run and deleted three times.
  `git status` showed no residue. No production file was touched, and there is no database or state side effect.
- No commits: the runner commits these files.
