---
plan: P18_pending_state_ui
role: implement
round: 1
state: DONE
date: 2026-10-26
actor: codex
---
# P18 implementer handoff

## What was built

Actions now show a busy mark immediately, disable while their service call runs, and reject duplicate submits. Sessions, workspace, Health, Settings, product details, the form, and publish review show plain skeleton placeholders during first load. Item search returns local matches after 75 ms, searches the shop after 300 ms, then joins the results. Old shop responses are ignored. The backend exposes a local-only `items_search` and a separate `items_search_shop`; the typed bindings were regenerated.

## Tests → criterion rows

| Row | Boundary test and exact assertion |
|---|---|
| C1(a) | `p18_pending_state.test.tsx` “C1(a)”: ActionButton is disabled and aria-busy while a held promise runs; second click calls once; it enables after resolve. |
| C1(b) | Same file “C1(b)”: rejected Error("nope") reaches onError and the button enables. |
| C2(a) | Same file “C2(a)”: usePageData page has two skeletons and aria-busy before first result; no skeletons with content after it or during reload. |
| C2(b) | Same file “C2(b)”: SessionsPage has placeholders and no empty state/row before resolution; exactly two rows and no skeletons after. |
| C2(c) | Same file “C2(c)”: WorkspacePage has eight photo placeholders and a stage placeholder before resolution; two photo tiles and no skeletons after. |
| C2(d) | Same file “C2(d)”: Health and Settings have skeletons and no Loading text before resolution; both show their content after. |
| C3(a) | `p18_item_search.rs` “c3a”: local call returns SKU-1 without item_find; shop call returns only SKU-9 with id 0/linked and one item_find. |
| C3(b) | `p18_item_search.test.tsx` “C3(b)”: local A and the searching heading appear after 75 ms; no shop call before 300 ms; shop B joins with the final heading. |
| C3(c) | Same file “C3(c)”: shop response for a after ab never adds shop A; shop AB remains. |

## Assumptions

- Local debounce is 75 ms and shop debounce is 300 ms.
- The busy mark is an ellipsis; skeletons use semantic muted colour and one pulse animation.
- A shop lookup failure says `shop unavailable`; local results stay visible.
- Review buttons and keyboard review actions remain immediate, as delegated by the plan.

## Deviations

- Additional nested components and old test mocks were changed to give AI, identity, camera, notification, retry, picker, and import actions busy states and to remove the last Loading text. These are listed under Files changed and explained in the plan Review log.
- The existing P07/P08/P13 mocks were adapted to export `searchItemsShop`; the P13 remote-hit fixture now comes from that service.
- No live camera, Capture One, or Shopify call was made. This phase's automated tests use fakes only.

## Files changed

Checkpoint paths (all 42):
- `apps/desktop/src-tauri/src/commands/items.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/components/pending/SkeletonFields.tsx`
- `apps/desktop/src/components/pending/SkeletonRows.tsx`
- `apps/desktop/src/components/pending/SkeletonTiles.tsx`
- `apps/desktop/src/components/ui/action-button.tsx`
- `apps/desktop/src/components/ui/skeleton.tsx`
- `apps/desktop/src/features/form/AiPopover.tsx`
- `apps/desktop/src/features/form/CustomizeMode.tsx`
- `apps/desktop/src/features/form/FormDialog.tsx`
- `apps/desktop/src/features/form/IdentityEditor.tsx`
- `apps/desktop/src/features/form/MetafieldInput.tsx`
- `apps/desktop/src/features/form/ResearchPanel.tsx`
- `apps/desktop/src/features/health/HealthPage.tsx`
- `apps/desktop/src/features/pending/__tests__/p18_pending_state.test.tsx`
- `apps/desktop/src/features/publish/BatchQueue.tsx`
- `apps/desktop/src/features/publish/FailurePopover.tsx`
- `apps/desktop/src/features/publish/NotificationsDrawer.tsx`
- `apps/desktop/src/features/publish/PublishPopover.tsx`
- `apps/desktop/src/features/sessions/NewSessionDialog.tsx`
- `apps/desktop/src/features/sessions/SessionRow.tsx`
- `apps/desktop/src/features/sessions/SessionsPage.tsx`
- `apps/desktop/src/features/sessions/StatusPill.tsx`
- `apps/desktop/src/features/sessions/__tests__/p07_shell_sessions_ui.test.tsx`
- `apps/desktop/src/features/sessions/__tests__/p13_publish_runs.test.tsx`
- `apps/desktop/src/features/sessions/__tests__/p18_item_search.test.tsx`
- `apps/desktop/src/features/settings/SettingsPage.tsx`
- `apps/desktop/src/features/shell/CameraDot.tsx`
- `apps/desktop/src/features/shell/Header.tsx`
- `apps/desktop/src/features/workspace/ItemDetailsCard.tsx`
- `apps/desktop/src/features/workspace/PhotoPanel.tsx`
- `apps/desktop/src/features/workspace/PickerDialogs.tsx`
- `apps/desktop/src/features/workspace/WorkspacePage.tsx`
- `apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx`
- `apps/desktop/src/lib/pending.ts`
- `apps/desktop/src/services/bindings.ts`
- `apps/desktop/src/services/items.ts`
- `apps/desktop/src/styles/globals.css`
- `crates/core/src/services/items.rs`
- `crates/testkit/tests/p18_item_search.rs`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P18_pending_state_ui.md`
- `docs/implementation/handoffs/implementer/handoff_P18_pending_state_ui_implement_r1.md` (this handoff, written after the checkpoint so it can name the SHA; the runner commits handoffs).

## Checkpoint SHA

`9270643` — `CHECKPOINT (not approved): implement P18 pending state UI`.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS, 407 Rust tests; no failures. |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS. |
| `cargo fmt --all -- --check` | PASS. |
| `cd apps/desktop && npm test` | PASS, 100 frontend tests; no failures. An earlier run failed because a P15 mock did not export `pickAndImport`; the import callback was adjusted and the suite passed. |
| `cd apps/desktop && npm run typecheck` | PASS. |
| `cargo test -p psm-desktop export_bindings` | PASS; generated bindings committed. |
| `git diff --check` | PASS before the checkpoint. |

The stamp ran on the code in checkpoint `9270643`; the checkpoint then added the tracker and Review log entry. There were no known pre-existing failures.

## Manual try-out

1. Launch the app with a few sessions. The Sessions list shows grey rows briefly, then real rows without an empty-state flash. Open a session to see stage and photo tile placeholders until its photos load.
2. Choose New session and type a title or SKU. Local matches appear first; shop matches join after a short delay without slowing the letters.
3. Press Create session, Save settings, Retry checks, Export, or Publish. The action dims with an ellipsis and a second click is ignored until it completes.
4. Open Health and Settings; they show placeholders while loading, then their usual content.

## ⚠ OWNER DECISIONS REQUIRED (0)

none

