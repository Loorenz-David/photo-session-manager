---
phase: P18_pending_state_ui
title: Pending-state architecture — busy actions, skeleton loading, local-first item search
difficulty: high
projection: no
depends_on: [P17_real_run_fixes]
---
# P18 — Pending-state architecture: busy actions, skeleton loading, local-first item search

## Goal
Give the frontend one shared pending-state architecture (real run R3): every action button shows an immediate busy
state and blocks a double submit; pages and cards render skeleton placeholders while their data loads; one reusable
primitive (hook + Skeleton components) is used by every existing page and action, with plain visuals now and a
single place to restyle later. Make item search local-first (R5 frontend half): local results on every keystroke,
shop search debounced separately, stale responses dropped. **Not in this phase:** the designed skeleton visuals
(owner: "design which can be implemented later"), the review buttons' busy state (local, instantaneous actions;
see Delegated choices), backend changes beyond the search split.

## Read first
`docs/implementation/real_run_2026-10-26.md` R3, R5 · master plan §6.1 (`apps/desktop` layout), §6.7 (commands),
§6.9 (frontend tests) · book `01`, `14`, `04` S-A (sessions list), S-B (new session, item search), `11` §Frontend ↔
backend (components call `services/`, never `invoke`) · evidence `design_handoff.md` §Components (shadcn primitives;
the Skeleton is undesigned: plain `bg-muted` pulse) · P17 plan (commands are async; the backend no longer blocks).

## Files expected to change
`apps/desktop/src/lib/pending.ts` (new: `useAction`, `usePageData`), `apps/desktop/src/components/ui/{skeleton.tsx (new, hand-written shadcn Skeleton: `data-slot="skeleton"`), action-button.tsx (new: `ActionButton`)}`,
`apps/desktop/src/components/pending/{SkeletonRows.tsx, SkeletonTiles.tsx, SkeletonFields.tsx} (new)`,
`apps/desktop/src/features/sessions/{SessionsPage.tsx, NewSessionDialog.tsx}`, `apps/desktop/src/features/workspace/{WorkspacePage.tsx, PhotoPanel.tsx or PhotoGrid.tsx, Stage.tsx, PickerDialogs.tsx, ItemDetailsCard.tsx}`,
`apps/desktop/src/features/{health/HealthPage.tsx, settings/SettingsPage.tsx, shell/Header.tsx, publish/{PublishPopover.tsx, BatchQueue.tsx, FailurePopover.tsx}, form/FormDialog.tsx}`,
`apps/desktop/src/services/items.ts` (`searchItems` = local, new `searchItemsShop`), `apps/desktop/src/services/bindings.ts` (regenerated),
`apps/desktop/src/features/pending/__tests__/p18_pending_state.test.tsx` (new), `apps/desktop/src/features/sessions/__tests__/p18_item_search.test.tsx` (new),
`apps/desktop/src/styles/globals.css` (skeleton tokens, one block),
`crates/core/src/services/items.rs` (`search_local`, `search_shop`; `search` = both, kept for the SKU proposal and existing callers), `apps/desktop/src-tauri/src/commands/items.rs` (`items_search` → local only; new `items_search_shop`), `apps/desktop/src-tauri/src/lib.rs` (command registered), `crates/testkit/tests/p18_item_search.rs` (new).

## Core mechanisms (unit tests allowed)
None.

## Ordered tasks
1. **Primitive.** `useAction(fn) -> {run, busy, error}`: `run` ignores calls while `busy`; sets `busy` before awaiting
   `fn`; on rejection calls `onError` (default `toast.error(message)`) and clears `busy`. `ActionButton` wraps the
   shadcn `Button`: props `action: () => Promise<unknown>`, `onError?`; renders `aria-busy="true"`, `disabled` and a
   plain busy label (`…` suffix or spinner) while pending. `usePageData(loader, deps) -> {data, loading, error,
   reload}`: `loading` is true only until the first result (a `reload` keeps the data shown); stale results dropped
   by ticket. `Skeleton` (`data-slot="skeleton"`, `aria-hidden`) and the three compositions `SkeletonRows(n)`,
   `SkeletonTiles(n)`, `SkeletonFields(n)`; a loading container carries `aria-busy="true"`.
2. **Pages.** SessionsPage: `SkeletonRows(6)` in place of the table until `listSessions` first resolves (the empty
   states stay as they are after). WorkspacePage: a stage skeleton and `SkeletonTiles(8)` in the photo panel until
   `getSessionView` first resolves. HealthPage, SettingsPage, FormDialog, PublishPopover, ItemDetailsCard: skeleton
   rows/fields replace every `Loading…` text (grep `Loading…` → 0 outside toasts).
3. **Actions.** Every button whose handler awaits a `services/*` call goes through `useAction`/`ActionButton`: New
   session "Create session" and result rows (replace the dialog's own `busy`), Header Export / Publish / Assign
   product, Health "Retry checks", Settings "Save settings" and "Choose…", PublishPopover "Publish n images" and
   Cancel (disabled while busy — backlog P15 note), BatchQueue, FailurePopover Retry, picker dialogs' pick, Form
   dialog saves and AI buttons. Review buttons and keyboard review actions are excluded (Delegated choices).
4. **Search split (R5).** `ItemsService::search_local(query)` (DB only, never calls the commerce adapter) and
   `search_shop(query)` (adapter `item_find`, minus linked items and SKUs already local — today's merge rule);
   `search` = local then shop (used by `next_sku`'s callers and the SKU conflict check). Commands: `items_search` →
   `search_local`; new `items_search_shop` → `search_shop` (`no_integration` → empty list, not an error). Frontend:
   `searchItems` (local, debounce ≤ 100 ms) and `searchItemsShop` (debounce 300 ms); NewSessionDialog lists local
   hits at once with the heading `n found · searching the shop…`, appends shop hits when they arrive, and drops a
   shop response whose query is no longer the current one (ticket); the SKU conflict check queries both.
   `PickerDialogs` (Assign product) does the same. Register `items_search_shop` in master plan §6.7 (done).
5. Bindings, tests, typecheck.

## Criteria (3 criteria, 9 rows; 9 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `<ActionButton action={fn}>` clicked (vitest, Testing Library) | `fn` returns a promise the test resolves later | while pending: `aria-busy="true"` and `disabled`; a second click while pending leaves `fn` called once; after resolve: no `aria-busy`, enabled | `[rule]` real run R3 (busy state, no double submit) |
| C1(b) | same | `fn` rejects with `Error("nope")` | button enabled again; `onError` received the error (default path: `toast.error` called with `nope`) | `[rule]` R3 |
| C2(a) | a test component over `usePageData(loader)` rendering `<SkeletonRows>` while loading | `loader` pending, then resolved `{n: 1}`; then `reload()` with a second pending promise | while pending: the container has `aria-busy="true"` and ≥ 1 `[data-slot="skeleton"]`; after resolve: content rendered, 0 skeletons; during `reload()`: content still rendered, 0 skeletons | `[rule]` R3 (pages show skeletons on first load, never flash on refresh) |
| C2(b) | `<SessionsPage>` | `listSessions` pending, then resolved with 2 rows | before resolve: ≥ 3 `[data-slot="skeleton"]` and neither "No sessions yet" nor a session row; after: 2 session rows, 0 skeletons | `[rule]` R3; ch04 S-A |
| C2(c) | `<WorkspacePage>` | `getSessionView` pending, then resolved with 2 photos | before: ≥ 4 skeleton tiles in the photo panel and one stage skeleton; after: 2 photo tiles, 0 skeletons | `[rule]` R3 |
| C2(d) | `<HealthPage>` and `<SettingsPage>` | `getHealth` / `getSettings` pending, then resolved (P16 fixtures) | before: ≥ 1 `[data-slot="skeleton"]` and no text `Loading…`; after: the P16 content | `[rule]` R3; ch12 |
| C3(a) | `ItemsService::search_local("sku")` and `search_shop("sku")` (Rust, testkit) | fake commerce with remote hit `SKU-9`; local item `SKU-1` | `search_local` → `[SKU-1]` and the fake records no `item_find`; `search_shop` → `[SKU-9]` with origin `linked`, `id 0`, and the fake records one `item_find` | `[rule]` real run R5 (local results first); ch04 S-B |
| C3(b) | `<NewSessionDialog open>` typing `ab` (fake timers) | `searchItems` resolves `[local A]` at once; `searchItemsShop` pending | after the local debounce: row `local A` listed and heading `1 found · searching the shop…`; `searchItemsShop` not yet called before 300 ms; after it resolves `[shop B]`: both rows listed, heading `2 found · 1 in the shop` | `[rule]` R5; ch04 S-B |
| C3(c) | typing `a` then `ab` within the shop debounce; the shop response for `a` resolves after the one for `ab` | shop(`a`) → `[shop A]`, shop(`ab`) → `[shop AB]` | the list shows `shop AB` and never `shop A` | `[rule]` R5 (stale responses dropped) |

## Delegated choices
The busy visual (ellipsis vs. spinner), skeleton sizes and the pulse animation, the exact debounce values within the
bounds above, whether `SkeletonTiles` reuses the tile grid's CSS, keeping the review buttons (A/C/⌫, swipe) without a
busy state because they return in milliseconds and must not interrupt keyboard flow, and the heading wording when the
shop search fails (`n found · shop unavailable`).

## Manual try-out
1. Launch with a few sessions: the list shows grey placeholder rows for a moment, then the rows (no empty-state
   flash). Open a session: placeholder tiles, then the photos.
2. New session → type a title: local matches appear as you type; shop matches a moment later; the letters never lag.
3. Press "Create session", "Save settings", "Retry checks", "Export" or "Publish": the button dims with a busy mark
   at once and a second click does nothing until the action finishes.
4. Health and Settings pages show placeholders instead of "Loading…".

## Notes
- Visual design of skeletons and busy states is deferred by the owner; this phase fixes the architecture so a later
  design pass touches `skeleton.tsx`, `action-button.tsx` and one CSS block only.
- Estimated size: ~20 files, ~900 lines.

## Review log

- **2026-10-26 implement r1, codex.** Added shared `useAction`/`ActionButton`, `usePageData`, plain Skeleton and row/tile/field compositions; first-load placeholders on sessions, workspace, health, settings, form, publish preview and item card; pending locks and busy labels on service-backed actions. Split item search into local SQLite and delayed shop commands, kept the combined service method for existing callers, and regenerated bindings. New session and item picker show local hits first and discard stale shop replies. One boundary test per criterion row: 8 frontend tests and 1 Rust test. Workspace suite, clippy, fmt, frontend suite and typecheck pass.
  - **Assumptions:** Local search waits 75 ms after typing; shop search waits 300 ms. Busy buttons use an ellipsis. A failed shop search says `shop unavailable`. Review shortcuts remain immediate, as delegated. Skeleton sizes and pulse are plain semantic-token placeholders.
  - **Deviations:** Edited `form/AiPopover.tsx`, `MetafieldInput.tsx`, `ResearchPanel.tsx`, `IdentityEditor.tsx`, `CustomizeMode.tsx`, `publish/NotificationsDrawer.tsx`, `sessions/StatusPill.tsx`, `SessionRow.tsx`, `shell/CameraDot.tsx` and prior frontend test mocks in P07/P08/P13 to carry the required busy state through nested service actions, remove the last loading text, and adapt those tests to the new search service export. `workspace/PhotoPanel.tsx` was the listed alternative to `PhotoGrid.tsx` and receives the import button state. No live camera or shop call was needed.
- **2026-10-26 review r1, opus. CHANGES_REQUESTED.** Gate: tracker `IMPLEMENTED`, checkpoint `9270643` present; its 42 paths equal the handoff's list (the handoff itself committed by the runner). Re-ran `cargo test --workspace` (407 pass), clippy `-D warnings`, `fmt --check`, `npm run typecheck`, `npm test` (100 pass): all exit 0; `bindings.ts` unchanged after regeneration. `Loading…` grep → 0 in `src/` outside the C2(d) test.
  - **Verified correct:** `useAction` ref lock + busy/error/onError (C1a/b); `ActionButton` disabled + `aria-busy` + `…`; `usePageData` first-load-only `loading`, ticketed reload (C2a); SessionsPage `SkeletonRows(6)` in place of table/empty state (C2b); WorkspacePage stage skeleton + `SkeletonTiles(8)` until the first view, load error still first (C2c); Health `SkeletonRows(4)`, Settings `SkeletonFields(6)` (C2d); `search_local` DB-only, `search_shop` = `item_find` minus linked externals and SKUs already local, no commerce → `[]`, `search` = both (C3a); commands thin over the service, `items_search_shop` registered and bound; NewSessionDialog 75 ms local / 300 ms shop, `alive` + current-title check drops a late older reply (C3b/c); SKU conflict check queries both; Assign-product/Assign-to-product now adopt a shop hit (`id 0`) before assigning (would otherwise have called `assign_unassigned(0)`). H1–H8: no new external write, no-integration path returns `[]` (H6), logic in `ItemsService` (H8); the `"shopify"` literal in `search_shop` moved from `search`, not added (P13 H7 note). Assumptions: all four within Delegated choices; none contradicts the book.
  - **F1 (B1, route: production).** `NewSessionDialog.tsx` and `PickerDialogs.tsx::ItemPickerDialog` gate shop hits on `debouncedShop === title && shopState === 'done'`, and neither clears `shopResults`/`shopState` when a new shop query starts. Reviewer probe (scratch vitest, deleted): type `a` (shop → `[shop A]`), then `ab` with the `ab` shop reply pending → after the local debounce the heading is `0 found · searching the shop…` (correct), but once the `ab` shop request is sent the list shows `shop A` again under heading `1 found · 1 in the shop` until the `ab` reply lands. Same in both dialogs. Violates plan task 4 / row C3(b) (heading `searching the shop…` until the current query's shop hits arrive) and C3(c)'s rule (a previous query's shop hits are never shown for the current one). Correction: key the shop hits to the query they answer (e.g. store `shopFor` with the results and show them only when `shopFor === title`), or reset to `searching`/`[]` when a shop query starts. Files: `apps/desktop/src/features/sessions/NewSessionDialog.tsx`, `apps/desktop/src/features/workspace/PickerDialogs.tsx`, optionally `apps/desktop/src/features/sessions/__tests__/p18_item_search.test.tsx` (the C3(c) test may add this ordering).
  - **Notes → backlog:** shop failures swallowed in `search_shop` (heading `shop unavailable` mostly unreachable); FormDialog shared AI-action lock drops a concurrent accept silently and no longer reloads after a failure; C2(d) test counts skeletons across both pages; C1(b) test covers only the explicit `onError` path; WorkspacePage keeps its own loader instead of `usePageData`.
  - **Probe declaration:** one scratch file `apps/desktop/src/features/sessions/__tests__/zz_review_probe.test.tsx` created and deleted three times; `git status` clean after. No production file touched.
